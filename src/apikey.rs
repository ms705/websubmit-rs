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
use async_trait::async_trait;

/// (username, apikey)
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

#[async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ApiKeyError;
    async fn from_request(request: &'a Request<'r>) -> request::Outcome<ApiKey, Self::Error> {
        let be = request.guard::<State<Arc<Mutex<NoriaBackend>>>>().unwrap();
        let key: String = request
            .cookies()
            .get("apikey")
            .and_then(|cookie| cookie.value().parse().ok()).unwrap();

        match check_api_key(&be, &key).await {
                Ok(user) => Some(ApiKey { user, key }),
                Err(_) => None,
        }
        .into_outcome((Status::Unauthorized, ApiKeyError::Missing))
    }
}

#[post("/", data = "<data>")]
pub(crate) async fn generate(
    data: Form<ApiKeyRequest>,
    backend: State<'_,Arc<Mutex<NoriaBackend>>>,
    config: State<'_,Config>,
) -> Template {
    // generate an API key from email address
    let mut hasher = Sha256::new();
    hasher.input_str(&data.email);
    // add a secret to make API keys unforgeable without access to the server
    hasher.input_str(&config.secret);
    let hash = hasher.result_str();

    let is_admin = if config.staff.contains(&data.email) {
        1.into()
    } else {
        0.into()
    };

    // insert into Noria if not exists
    let mut bg = backend.lock().unwrap();
    let mut table = bg.handle.table("users").await.unwrap();
    table
        .insert(vec![
            data.email.as_str().into(),
            hash.as_str().into(),
            is_admin,
        ]);

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

pub(crate) async fn check_api_key(
    backend: &Arc<Mutex<NoriaBackend>>,
    key: &str,
) -> Result<String, ApiKeyError> {
    let mut bg = backend.lock().unwrap();
    let mut v = bg.handle.view("users_by_apikey").await.unwrap();
    match v.lookup(&[key.into()], true).await {
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
pub(crate) async fn check(
    data: Form<ApiKeySubmit>,
    mut cookies: Cookies<'_>,
    backend: State<'_, Arc<Mutex<NoriaBackend>>>,
) -> Redirect {
    // check that the API key exists and set cookie
    let res = check_api_key(&*backend, &data.key).await;
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
