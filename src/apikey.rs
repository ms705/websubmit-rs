use crate::backend::NoriaBackend;
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

pub(crate) struct ApiKey(String);

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
    BadCount,
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ApiKeyError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<ApiKey, Self::Error> {
        request
            .cookies()
            .get("apikey")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|key| ApiKey(key))
            .into_outcome((Status::Unauthorized, ApiKeyError::Missing))
    }
}

#[post("/", data = "<data>")]
pub(crate) fn generate(
    data: Form<ApiKeyRequest>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Template {
    // generate an API key from email address
    let mut hasher = Sha256::new();
    hasher.input_str(&data.email);
    // XXX(malte): need a salt or secret here to make API keys unforgeable
    let hash = hasher.result_str();

    // insert into Noria if not exists
    let mut bg = backend.lock().unwrap();
    let mut table = bg.handle.table("users").unwrap().into_sync();
    table
        .insert(vec![
            data.email.as_str().into(),
            hash.as_str().into(),
            0.into(),
        ])
        .expect("failed to insert user!");

    // return to user
    let mut ctx = HashMap::new();
    ctx.insert("APIKEY", hash);
    Template::render("apikey/generate", &ctx)
}

#[post("/", data = "<data>")]
pub(crate) fn check(
    data: Form<ApiKeySubmit>,
    mut cookies: Cookies,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Redirect {
    // check that the API key exists and set cookie
    // insert into Noria if not exists
    let mut bg = backend.lock().unwrap();
    let mut v = bg.handle.view("users_by_apikey").unwrap().into_sync();
    let res = v.lookup(&[data.key.as_str().into()], true);

    if res.is_err() {
        eprintln!("Problem communicating with Noria: {:?}", res);
        Redirect::to("/")
    } else if res.as_ref().unwrap().len() < 1 {
        eprintln!("No such API key: {}", data.key);
        Redirect::to("/")
    } else {
        let cookie = Cookie::build("apikey", data.key.clone()).path("/").finish();
        cookies.add(cookie);
        Redirect::to("/leclist")
    }
}
