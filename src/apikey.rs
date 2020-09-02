use crate::backend::NoriaBackend;
use crate::config::Config;
use crate::email;
use chrono::Local;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use noria::manual::ops::project::Project;
use noria::manual::ops::union::Union;
use noria::manual::Base;
use noria::manual::Migration;
use noria::NodeIndex;

use rocket::http::Status;
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::Form;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

/// (username, apikey)
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
pub(crate) struct ResubscribeSubmit {
    key: String,
    data: String,
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
                Ok(user) => {
                    let user = trim_email(user);
                    Some(ApiKey { user, key })
                }
                Err(_) => None,
            })
            .into_outcome((Status::Unauthorized, ApiKeyError::Missing))
    }
}

pub(crate) fn trim_email(email: String) -> String {
    email.split('@').take(1).collect::<Vec<_>>()[0].to_string()
}

#[get("/")]
pub(crate) fn update_account_settings(config: State<Config>) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("CLASS_ID", config.class.clone());
    ctx.insert("parent", String::from("layout"));
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
    println!("Apikey generated is {:?}", hash.clone());
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
                Err(ApiKeyError::Missing)
            } else if rs.len() > 1 {
                Err(ApiKeyError::Ambiguous)
            } else {
                // user email
                Ok((&rs[0][0]).into())
            }
        }
        Err(e) => Err(ApiKeyError::BackendFailure(e)),
    }
}

pub(crate) fn check_key(bg: &mut NoriaBackend, untrimmed: &str) -> Result<String, ApiKeyError> {
    let mut v = bg.handle.view("users_by_apikey").unwrap().into_sync();
    let key = untrimmed.to_string().trim_matches('\"').to_string();
    println!("Looking at this apikey {:?}", key.clone());
    let res = v.lookup(&[key.into()], true);

    match res {
        Ok(rs) => {
            if rs.len() < 1 {
                Err(ApiKeyError::Missing)
            } else if rs.len() > 1 {
                Err(ApiKeyError::Ambiguous)
            } else {
                // user email
                Ok((&rs[0][0]).into())
            }
        }
        Err(e) => Err(ApiKeyError::BackendFailure(e)),
    }
}

#[post("/", data = "<data>")]
pub(crate) fn check(
    data: Form<ApiKeySubmit>,
    mut cookies: Cookies,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Redirect {
    // check that the API key exists and set cookie
    let key = data.key.trim_matches('\"').to_string();
    let res = check_api_key(&*backend, &key);
    match res {
        Err(ApiKeyError::BackendFailure(ref err)) => {
            eprintln!("Problem communicating with Noria: {:?}", err);
        }
        Err(ApiKeyError::Missing) => {
            eprintln!("No such API key: {}", data.key);
        }
        Err(ApiKeyError::Ambiguous) => {
            eprintln!("Ambiguous API key: {}", data.key);
        }
        Ok(_) => (),
    }

    if res.is_err() {
        Redirect::to("/")
    } else {
        let cookie = Cookie::build("apikey", data.key.clone()).path("/").finish();
        cookies.add(cookie);
        Redirect::to("/leclist")
    }
}

#[post("/", data = "<data>")]
pub(crate) fn resubscribe(
    data: Form<ResubscribeSubmit>,
    mut cookies: Cookies,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Redirect {
    let mut bg = backend.lock().unwrap();
    bg.handle
        .import_data(data.data.clone().to_string())
        .expect("failed to import data");

    let res = check_key(&mut bg, &data.key);
    match res {
        Err(ApiKeyError::BackendFailure(ref err)) => {
            eprintln!("Problem communicating with Noria: {:?}", err);
        }
        Err(ApiKeyError::Missing) => {
            eprintln!("No such API key: {}", data.key);
        }
        Err(ApiKeyError::Ambiguous) => {
            eprintln!("Ambiguous API key: {}", data.key);
        }
        Ok(_) => (),
    }

    if res.is_err() {
        Redirect::to("/")
    } else {
        let cookie = Cookie::build("apikey", data.key.clone()).path("/").finish();
        cookies.add(cookie);
        Redirect::to("/leclist")
    }
}

pub(crate) fn create_user_shard(
    bg: &mut std::sync::MutexGuard<'_, NoriaBackend>,
    email: String,
    hash: &str,
    config: &State<Config>,
) {
    let email_key = trim_email(email.clone());
    let user_email_key = email_key.clone(); // for migration

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
        let userinfo = mig.add_base(
            format!("userinfo_{}", user_email_key.clone()),
            &["email", "apikey", "is_admin"],
            Base::default()
                .with_key(vec![1])
                .anonymize_with_resub_key(vec![1]),
        );
        let userinfo_from = mig.add_ingredient(
            format!("userinfo_from_{}", user_email_key.clone()),
            &["email", "is_admin", "apikey", "bogokey"],
            Project::new(userinfo, &[0, 2, 1], Some(vec![0.into()]), None),
        );
        mig.maintain_anonymous(userinfo_from, &[3]);

        let answers = mig.add_base(
            format!("answers_{}", user_email_key.clone()),
            &["email_key", "lec", "q", "answer", "submitted_at"],
            Base::default()
                .with_key(vec![1, 2])
                .anonymize_with_resub_key(vec![3]),
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
                            key: &[usize]|
         -> NodeIndex {
            let mut emits = HashMap::new();
            emits.insert(source, e.clone());
            let u = Union::new(emits);
            let union_node = mig.add_ingredient(name, fields, u);
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
            );

            let answers_by_q_and_apikey = mig.add_ingredient(
                "answers_by_q_and_emailkey",
                &["email_key", "lec", "q", "answer", "submitted_at"],
                Project::new(answers_union, &[0, 1, 2, 3, 4], None, None),
            );
            mig.maintain_anonymous(answers_by_q_and_apikey, &[0, 2]);

            let info_union = create_union(
                mig,
                userinfo,
                vec![0, 1, 2],
                "users_union",
                "users_by_apikey",
                &["email_key", "apikey", "is_admin"],
                &[1],
            );
            let all_users = mig.add_ingredient(
                "all_users",
                &["email_key", "apikey", "is_admin"],
                Project::new(info_union, &[0, 1, 2], Some(vec![0.into()]), None),
            );
            mig.maintain_anonymous(all_users, &[3]);
            (userinfo, answers, Some((answers_union, info_union)))
        } else {
            mig.add_parent(answers, unions.unwrap().0, vec![0, 1, 2, 3, 4]);
            mig.add_parent(userinfo, unions.unwrap().1, vec![0, 1, 2]);
            (userinfo, answers, unions)
        }
    });

    bg.name_to_nodeIndex
        .entry(format!("userinfo_{}", email_key.clone()))
        .or_insert(userinfo);
    bg.name_to_nodeIndex
        .entry(format!("answers_{}", email_key.clone()))
        .or_insert(answers);

    if !unions_created {
        bg.unions = union_index;
    }
    let mut userinfo_table = bg
        .handle
        .table(format!("userinfo_{}", email_key))
        .unwrap()
        .into_sync();
    userinfo_table
        .insert(vec![email.into(), hash.into(), is_admin.into()])
        .expect("failed to insert userinfo");
}

#[post("/")]
pub(crate) fn remove_data(
    backend: State<Arc<Mutex<NoriaBackend>>>,
    apikey: ApiKey,
    config: State<Config>,
) -> Template {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("un_times.txt")
        .unwrap();
    let mut data = OpenOptions::new()
        .append(true)
        .create(true)
        .open("imported_data.txt")
        .unwrap();

    let start = Local::now().naive_local();
    let bg = &mut backend.lock().unwrap();
    let userinfo_table_name = format!("userinfo_{}", apikey.user);
    let answers_table_name = format!("answers_{}", apikey.user);

    let info_ni = bg
        .name_to_nodeIndex
        .get(&userinfo_table_name)
        .expect("failed to fetch the ni of userinfo")
        .clone();
    let answers_ni = bg
        .name_to_nodeIndex
        .get(&answers_table_name)
        .expect("failed to fetch the ni of answers")
        .clone();

    let data_string = bg
        .handle
        .export_data(vec![(info_ni.index() as u32), (answers_ni.index() as u32)])
        .expect("failed to get data from Noria");
    bg.handle
        .unsubscribe(info_ni.index() as u32)
        .expect("failed to remove base userinfo");
    bg.handle
        .unsubscribe(answers_ni.index() as u32)
        .expect("failed to remove base answers");

    let time = &format!("{:?}#{:?}\n", start, Local::now().naive_local());
    write!(&mut file, "{}", time).expect("failed to write to un_times.txt");
    let data_write = &format!("{:?}*{}\n", apikey.key, data_string);
    write!(&mut data, "{}", data_write).expect("failed to write imported data");

    let mut ctx = HashMap::new();
    ctx.insert("CLASS_ID", config.class.clone());
    ctx.insert("parent", String::from("layout"));
    Template::render("login", &ctx)
}
