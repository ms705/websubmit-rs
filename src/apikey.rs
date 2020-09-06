extern crate rocket_multipart_form_data;

use crate::backend::NoriaBackend;
use crate::config::Config;
use crate::email;
use chrono::prelude::{DateTime, Local};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use noria::manual::ops::project::Project;
use noria::manual::ops::union::Union;
use noria::manual::Base;
use noria::manual::Migration;
use noria::{Modification, NodeIndex};
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::Form;
use rocket::request::{self, FromRequest, Request};
use rocket::response::status::BadRequest;
use rocket::response::Redirect;
use rocket::Data;
use rocket::State;
use rocket_contrib::templates::Template;
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(crate) struct ApiKey {
    pub user: String,
    pub key: String,
}

#[derive(Debug, FromForm)]
pub(crate) struct ApiKeyRequest {
    email: String,
}

#[derive(Debug, FromForm)]
pub(crate) struct ApiKeySubmit {
    key: String,
}

#[derive(Debug, FromForm)]
pub(crate) struct ApiKeySubmitWithPermissions {
    answers: bool,
    research: bool,
    key: String,
}

#[derive(Debug, FromForm)]
pub(crate) struct ChangePermissionsForm {
    answers: bool,
    research: bool,
}

#[derive(Debug)]
pub(crate) enum ApiKeyError {
    Ambiguous,
    Missing,
    BackendFailure(noria::error::ViewError),
}

impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ApiKeyError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<ApiKey, Self::Error> {
        let be = request.guard::<State<Arc<Mutex<NoriaBackend>>>>().unwrap();
        request
            .cookies()
            .get("apikey")
            .and_then(|cookie| cookie.value().parse().ok())
            .and_then(|key: String| match check_api_key(&be, &key) {
                Ok(user) => Some(ApiKey { user, key }),
                Err(_) => None,
            })
            .into_outcome((Status::Unauthorized, ApiKeyError::Missing))
    }
}

const DEFAULT_PERMISSIONS: u8 = 0b0000_0111;
const CORE_ON: u8 = 0b0000_0100;
const ANSWERS_ON: u8 = 0b0000_0001;
const RESEARCH_ON: u8 = 0b0000_0010;

pub(crate) fn trim_email(email: String) -> String {
    email.split('@').take(1).collect::<Vec<_>>()[0].to_string()
}

#[get("/")]
pub(crate) fn update_account_settings(
    backend: State<Arc<Mutex<NoriaBackend>>>,
    apikey: ApiKey,
) -> Template {
    let bg = &mut backend.lock().unwrap();
    let mut ctx = HashMap::new();
    ctx.insert("parent", String::from("layout"));
    let perms = current_permissions(bg, trim_email(apikey.user));
    let set_permissions = |value: &u8, bitcode: u8| -> String {
        if value & bitcode != 0 {
            String::from("checked")
        } else {
            String::from("")
        }
    };
    let answers = set_permissions(&perms, ANSWERS_ON);
    ctx.insert("answers", answers);
    let research = set_permissions(&perms, RESEARCH_ON);
    ctx.insert("research", research);

    Template::render("account", &ctx)
}

#[post("/", data = "<data>")]
pub(crate) fn generate(
    data: Form<ApiKeyRequest>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
    config: State<Config>,
) -> Template {
    let mut hasher = Sha256::new();
    hasher.input_str(&data.email);
    hasher.input_str(&config.secret);
    let hash = hasher.result_str();
    let mut bg = backend.lock().unwrap();
    create_user_shard(&mut bg, data.email.clone(), hash.as_str(), &config);

    if config.send_emails {
        email::send(
            "no-reply@csci2390-submit.cs.brown.edu".into(),
            vec![data.email.clone()],
            format!("{} API key", config.class),
            format!("Your {} API key is: {}", config.class, hash.as_str()),
        )
        .expect("failed to send API key email");
    }

    // return to user
    let mut ctx = HashMap::new();
    ctx.insert("apikey_email", data.email.clone());
    ctx.insert("parent", "layout".into());
    Template::render("apikey/generate", &ctx)
}

pub(crate) fn check_api_key(
    backend: &Arc<Mutex<NoriaBackend>>,
    key: &str,
) -> Result<String, ApiKeyError> {
    let mut bg = backend.lock().unwrap();
    let mut v = bg.handle.view("users_by_apikey").unwrap().into_sync();
    let res = v.lookup(&[key.into()], true);

    match res {
        Ok(rs) => {
            if rs.len() < 1 {
                eprintln!("No such API key: {}", key);
                Err(ApiKeyError::Missing)
            } else if rs.len() > 1 {
                eprintln!("Ambiguous API key: {}", key);
                Err(ApiKeyError::Ambiguous)
            } else {
                // user email
                Ok((&rs[0][0]).into())
            }
        }
        Err(e) => {
            eprintln!("Problem communicating with Noria: {:?}", e);
            Err(ApiKeyError::BackendFailure(e))
        }
    }
}

#[post("/", data = "<data>")]
pub(crate) fn check(
    data: Form<ApiKeySubmit>,
    mut cookies: Cookies,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Redirect {
    let key = data.key.trim_matches('\"').to_string();
    let res = check_api_key(&*backend, &key);

    if res.is_err() {
        Redirect::to("/")
    } else {
        let cookie = Cookie::build("apikey", data.key.clone()).path("/").finish();
        cookies.add(cookie);
        Redirect::to("/leclist")
    }
}

#[post("/", data = "<data>")]
pub(crate) fn check_with_permissions(
    data: Form<ApiKeySubmitWithPermissions>,
    mut cookies: Cookies,
    backend: State<Arc<Mutex<NoriaBackend>>>,
    config: State<Config>,
) -> Result<Redirect, BadRequest<String>> {
    let key = data.key.trim_matches('\"').to_string();
    let res = check_api_key(&*backend, &key);
    if res.is_err() {
        return Ok(Redirect::to("/"));
    };

    let bg = &mut backend.lock().unwrap();
    let answers: u8 = if data.answers { 1 } else { 0 };
    let research: u8 = if data.research { 1 } else { 0 };

    match change_perms(
        answers,
        research,
        ApiKey {
            user: res.unwrap(),
            key,
        },
        bg,
        &config,
    ) {
        Err(_) => {
            return Err(BadRequest(Some("backend error".to_string())));
        }
        Ok(()) => {}
    }

    let cookie = Cookie::build("apikey", data.key.clone()).path("/").finish();
    cookies.add(cookie);
    Ok(Redirect::to("/leclist"))
}

fn change_perms(
    answers: u8,
    research: u8,
    apikey: ApiKey,
    bg: &mut std::sync::MutexGuard<'_, NoriaBackend>,
    config: &State<Config>,
) -> Result<(), String> {
    let new_perms = answers | research << 1 | CORE_ON;
    let trimmed = trim_email(apikey.user.clone());

    let curr_perms = current_permissions(bg, trimmed.clone());
    if curr_perms == new_perms {
        return Ok(());
    }

    let userinfo_table_name = format!("userinfo_{}", trimmed.clone());
    let answers_table_name = format!("answers_{}", trimmed.clone());

    let answers_ni = bg
        .noria_index
        .get(&answers_table_name)
        .expect("failed to fetch the ni of userinfo")
        .clone();

    if bg.handle.change_permissions(answers_ni, new_perms).is_err() {
        return Err("backend error".to_string());
    };

    let is_admin = if config.staff.contains(&trimmed) {
        1
    } else {
        0
    };

    let mut table = bg.handle.table(&userinfo_table_name).unwrap().into_sync();
    table
        .insert_or_update(
            vec![
                apikey.user.into(),
                apikey.key.into(),
                is_admin.into(),
                (new_perms as u64).into(),
            ],
            vec![(3, Modification::Set((new_perms as u64).into()))],
        )
        .unwrap();
    Ok(())
}

fn current_permissions(bg: &mut std::sync::MutexGuard<'_, NoriaBackend>, email_key: String) -> u8 {
    let mut view = bg
        .handle
        .view(format!("permissions_{}", email_key))
        .unwrap()
        .into_sync();
    let perms = view.lookup(&[0.into()], true).unwrap();
    let answers_perms: u64 = perms[0][0].clone().into();
    answers_perms as u8
}

#[post("/", data = "<data>")]
pub(crate) fn change_permissions(
    data: Form<ChangePermissionsForm>,
    apikey: ApiKey,
    backend: State<Arc<Mutex<NoriaBackend>>>,
    config: State<Config>,
) -> Result<Redirect, BadRequest<String>> {
    let bg = &mut backend.lock().unwrap();
    let answers: u8 = if data.answers { 1 } else { 0 };
    let research: u8 = if data.research { 1 } else { 0 };
    match change_perms(answers, research, apikey, bg, &config) {
        Err(_) => return Err(BadRequest(Some("backend error".to_string()))),
        Ok(()) => Ok(Redirect::to("/leclist")),
    }
}

#[post("/", data = "<data>")]
pub(crate) fn resubscribe(
    mut cookies: Cookies,
    config: State<Config>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
    content_type: &ContentType,
    data: Data,
) -> Result<Redirect, BadRequest<String>> {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("key"),
        MultipartFormDataField::file("takeout")
            .size_limit(20 * 1024 * 1024)
            .content_type_by_string(Some(mime::TEXT_PLAIN))
            .unwrap(),
    ]);
    let mfd = MultipartFormData::parse(content_type, data, options);
    let mut multipart_form_data = match mfd {
        Err(_) => {
            return Err(BadRequest(Some(
                "make sure to insert your apikey and upload your most recent takeout".to_string(),
            )))
        }
        Ok(m) => m,
    };
    let key = multipart_form_data.texts.remove("key");
    let d = multipart_form_data.files.remove("takeout");

    if d.is_none() {
        return Err(BadRequest(Some("failed to upload file".to_string())));
    };

    let apikey = key.unwrap().remove(0).text;
    let data_path = d.unwrap().remove(0).path;
    let contents = fs::read_to_string(data_path).expect("Something went wrong reading the file");

    // compute the hash
    let mut hasher = Sha256::new();
    hasher.input_str(&contents);
    hasher.input_str(&config.secret);
    let actual_hash = hasher.result_str();

    let mut bg = backend.lock().unwrap();

    let mut exports = bg.handle.view("exports_by_apikey").unwrap().into_sync();
    let res = exports.lookup(&[apikey.clone().into()], true);

    let expected_hash: String;
    match res {
        Err(_) => {
            return Err(BadRequest(Some("backend error".to_string())));
        }
        Ok(r) => {
            if r.len() != 1 {
                return Err(BadRequest(Some("invalid apikey".to_string())));
            } else {
                expected_hash = r[0][1].clone().into();
            }
        }
    };

    if expected_hash != actual_hash {
        return Err(BadRequest(Some(
            "make sure to upload the most recent untampered takeout".to_string(),
        )));
    };

    let new_bases = bg
        .handle
        .import_data(contents)
        .expect("failed to import data");

    let cookie = Cookie::build("apikey", apikey.clone()).path("/").finish();
    cookies.add(cookie);

    // update the indices we store
    assert_eq!(new_bases.len(), 2);
    let mut v = bg.handle.view("users_by_apikey").unwrap().into_sync();
    let res = v.lookup(&[apikey.into()], true);
    if res.is_err() {
        eprintln!("failed to update noria_index with new bases");
        return Ok(Redirect::to("/leclist"));
    }
    let trimmed = trim_email(res.unwrap()[0][0].clone().into());
    let userinfo_table_name = format!("userinfo_{}", trimmed.clone());
    let answers_table_name = format!("answers_{}", trimmed);
    *bg.noria_index.get_mut(&userinfo_table_name).unwrap() = new_bases[0];
    *bg.noria_index.get_mut(&answers_table_name).unwrap() = new_bases[1];

    Ok(Redirect::to("/leclist"))
}

pub(crate) fn create_user_shard(
    bg: &mut std::sync::MutexGuard<'_, NoriaBackend>,
    email: String,
    hash: &str,
    config: &State<Config>,
) {
    let email_key = trim_email(email.clone());
    let user_email_key = email_key.clone();

    let is_admin = if config.staff.contains(&email_key) {
        1
    } else {
        0
    };
    let unions_created = if bg.handle.outputs().unwrap().contains_key("all_users") {
        true
    } else {
        false
    };

    let mut unions = None;
    if unions_created {
        unions = Some(bg.unions.unwrap());
    }
    let (userinfo, answers, union_index) = bg.handle.migrate(move |mig| {
        let userinfo = mig.add_base_with_permissions(
            format!("userinfo_{}", user_email_key.clone()),
            &["email", "apikey", "is_admin", "perms"],
            Base::default().with_key(vec![1]),
            Some(CORE_ON),
        );
        let permissions = mig.add_ingredient(
            format!("permissions_{}", user_email_key.clone()),
            &["perms", "bogokey"],
            Project::new(userinfo, &[3], Some(vec![0.into()]), None),
        );
        mig.maintain_anonymous(permissions, &[1]);

        let answers = mig.add_base_with_permissions(
            format!("answers_{}", user_email_key.clone()),
            &["email_key", "lec", "q", "answer", "submitted_at"],
            Base::default().with_key(vec![1, 2]),
            Some(DEFAULT_PERMISSIONS),
        );
        let my_answers_for_lec = mig.add_ingredient(
            format!("my_answers_for_lec_{}", user_email_key.clone()),
            &["email_key", "lec", "q", "answer"],
            Project::new(answers, &[0, 1, 2, 3], None, None),
        );
        mig.maintain_anonymous(my_answers_for_lec, &[1]);

        let create_union = |mig: &mut Migration,
                            source: NodeIndex,
                            e: Vec<usize>,
                            name: &str,
                            name_by_feature: &str,
                            fields: &[&str],
                            key: &[usize],
                            with_permissions: Option<u8>|
         -> NodeIndex {
            let mut emits = HashMap::new();
            emits.insert(source, e.clone());
            let u = Union::new(emits);
            let union_node = match with_permissions {
                Some(p) => mig.add_ingredient_with_permissions(name, fields, u, Some(p)),
                None => mig.add_ingredient(name, fields, u),
            };

            let by_feature = mig.add_ingredient(
                name_by_feature,
                fields,
                Project::new(union_node, &e, None, None),
            );
            mig.maintain_anonymous(by_feature, key);
            union_node
        };

        if !unions_created {
            let answers_union = create_union(
                mig,
                answers,
                vec![0, 1, 2, 3, 4],
                "answers_union",
                "answers_by_lec",
                &["email_key", "lec", "q", "answer", "submitted_at"],
                &[1],
                Some(CORE_ON),
            );

            let info_union = create_union(
                mig,
                userinfo,
                vec![0, 1, 2],
                "users_union",
                "users_by_apikey",
                &["email_key", "apikey", "is_admin"],
                &[1],
                None,
            );
            let all_users = mig.add_ingredient(
                "all_users",
                &["email_key", "apikey", "is_admin"],
                Project::new(info_union, &[0, 1, 2], Some(vec![0.into()]), None),
            );

            let faq = mig.add_ingredient_with_permissions(
                "faq",
                &["email_key", "lec", "q", "answer", "submitted_at"],
                Project::new(answers_union, &[0, 1, 2, 3, 4], None, None),
                Some(ANSWERS_ON),
            );
            mig.maintain_anonymous(faq, &[1]);
            mig.maintain_anonymous(all_users, &[3]);
            (userinfo, answers, Some((answers_union, info_union)))
        } else {
            mig.add_parent(answers, unions.unwrap().0, vec![0, 1, 2, 3, 4]);
            mig.add_parent(userinfo, unions.unwrap().1, vec![0, 1, 2]);
            (userinfo, answers, unions)
        }
    });

    bg.noria_index
        .entry(format!("userinfo_{}", email_key.clone()))
        .or_insert(userinfo.index() as u32);
    bg.noria_index
        .entry(format!("answers_{}", email_key.clone()))
        .or_insert(answers.index() as u32);

    if !unions_created {
        bg.unions = union_index;
    }
    let mut userinfo_table = bg
        .handle
        .table(format!("userinfo_{}", email_key))
        .unwrap()
        .into_sync();
    userinfo_table
        .insert(vec![
            email.into(),
            hash.into(),
            is_admin.into(),
            (DEFAULT_PERMISSIONS as u64).into(),
        ])
        .expect("failed to insert userinfo");
}

#[post("/")]
pub(crate) fn remove_data(
    backend: State<Arc<Mutex<NoriaBackend>>>,
    apikey: ApiKey,
    config: State<Config>,
    mut cookies: Cookies,
) -> Redirect {
    let bg = &mut backend.lock().unwrap();

    let (info_ni, answers_ni) = export_data(bg, apikey, config);

    bg.handle
        .unsubscribe(info_ni)
        .expect("failed to remove base userinfo");
    bg.handle
        .unsubscribe(answers_ni)
        .expect("failed to remove base answers");

    cookies.remove_private(Cookie::named("apikey"));

    Redirect::to("/login")
}

pub(crate) fn export_data(
    bg: &mut std::sync::MutexGuard<'_, NoriaBackend>,
    apikey: ApiKey,
    config: State<Config>,
) -> (u32, u32) {
    let ts: DateTime<Local> = Local::now();
    let path_str = format!("attachment-{}-{}.txt", apikey.user, ts);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path_str.clone())
        .unwrap();

    let userinfo_table_name = format!("userinfo_{}", trim_email(apikey.user.clone()));
    let answers_table_name = format!("answers_{}", trim_email(apikey.user.clone()));

    let info_ni = bg
        .noria_index
        .get(&userinfo_table_name)
        .expect("failed to fetch the ni of userinfo")
        .clone();

    let answers_ni = bg
        .noria_index
        .get(&answers_table_name)
        .expect("failed to fetch the ni of answers")
        .clone();

    let data_string = bg
        .handle
        .export_data(vec![info_ni, answers_ni])
        .expect("failed to get data from Noria");
    write!(&mut file, "{}", data_string).expect("failed to write export");

    if config.send_emails {
        email::send_with_attachment(
            "no-reply@csci2390-submit.cs.brown.edu".into(),
            apikey.user,
            format!("{} websubmit takeout", config.class),
            format!("Find your {} data attached.", config.class,),
            path_str,
        )
        .expect("failed to send API key email");
    }

    // update the most recent hash we store
    let mut hasher = Sha256::new();
    hasher.input_str(&data_string);
    hasher.input_str(&config.secret);
    let hash = hasher.result_str();

    let mut table = bg.handle.table("exports").unwrap().into_sync();
    table
        .insert_or_update(
            vec![apikey.key.clone().into(), hash.clone().into()],
            vec![(1, Modification::Set(hash.clone().into()))],
        )
        .unwrap();
    (info_ni, answers_ni)
}

#[post("/")]
pub(crate) fn export(
    backend: State<Arc<Mutex<NoriaBackend>>>,
    apikey: ApiKey,
    config: State<Config>,
) -> Redirect {
    let mut bg = backend.lock().unwrap();
    export_data(&mut bg, apikey, config);
    Redirect::to("/leclist")
}
