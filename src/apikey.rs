use crate::backend::NoriaBackend;
use crate::config::Config;
use crate::email;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use rocket::http::Status;
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::Form;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
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

#[post("/", data = "<data>")]
pub(crate) fn generate(
    data: Form<ApiKeyRequest>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
    config: State<Config>,
) -> Template {
    // generate an API key from email address
    let mut hasher = Sha256::new();
    hasher.input_str(&data.email);
    // add a secret to make API keys unforgeable without access to the server
    hasher.input_str(&config.secret);
    let hash = hasher.result_str();

    let is_admin = if config.staff.contains(&data.email) {
        1
    } else {
        0
    };

    let mut bg = backend.lock().unwrap();
    create_user_shard(&mut bg, data.email.clone(), hash.as_str(), is_admin);

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

    match v.lookup(&[key.into()], true) {
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
    let res = check_api_key(&*backend, &data.key);
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
  is_admin: u64) {
  let email_digest = email.split('@').take(1).collect::<Vec<_>>()[0].to_string();
    // insert into Noria if not exists

    let mut table = bg.handle.table("users").unwrap().into_sync();
    table
        .insert(vec![
          email_digest.clone().into(),
          hash.into(),
        ])
        .expect("failed to insert user!");

    // Create user info table
    let sql = format!("CREATE TABLE userinfo_{0} (email varchar(255), apikey text, is_admin tinyint, PRIMARY KEY (apikey));\
      CREATE TABLE answers_{0} (lec int, q int, answer text, submitted_at datetime, PRIMARY KEY (lec, q));\
      QUERY userinfo_from_{0}: SELECT email, is_admin, apikey FROM userinfo_{0};\
      QUERY answers_by_lec_from_{0}: SELECT * FROM answers_{0} WHERE lec = ?;",
      email_digest.clone());

    bg.handle.extend_recipe(sql).unwrap();

    let mut userinfo_table = bg.handle.table(format!("userinfo_{}", email_digest)).unwrap().into_sync();

    userinfo_table.insert(vec![
      email.into(),
      hash.into(),
      is_admin.into(),
      ])
    .expect("failed to insert userinfo");
}
