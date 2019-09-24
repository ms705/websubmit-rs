use crate::backend::{DataType, NoriaBackend};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use rocket::request::Form;
use rocket::response::NamedFile;
use rocket::State;
use rocket_contrib::templates::{handlebars, Template};
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

#[derive(Debug, FromForm)]
pub(crate) struct ApiKeyRequest {
    email: String,
}

#[derive(Debug, FromForm)]
pub(crate) struct ApiKeySubmit {
    key: String,
}

#[post("/", data = "<data>")]
pub(crate) fn generate(
    data: Form<ApiKeyRequest>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Template {
    // generate an API key from email address
    let mut hasher = Sha256::new();
    hasher.input_str(&data.email);
    let hash = hasher.result_str();

    // insert into Noria if not exists
    let mut bg = backend.lock().unwrap();
    let mut table = bg.handle.table("users").unwrap().into_sync();
    table.insert(vec![data.email.as_str().into(), hash.as_str().into()]);

    // return to user
    let mut ctx = HashMap::new();
    ctx.insert("APIKEY", hash);
    Template::render("apikey/generate", &ctx)
}

#[post("/", data = "<data>")]
pub(crate) fn check(data: Form<ApiKeySubmit>, backend: State<Arc<Mutex<NoriaBackend>>>) -> String {
    // check that the API key exists and set cookie
    // insert into Noria if not exists
    let mut bg = backend.lock().unwrap();
    let mut v = bg.handle.view("users_by_apikey").unwrap().into_sync();
    let res = v.lookup(&[data.key.as_str().into()], true);

    if res.is_err() {
        format!("Problem communicating with Noria: {:?}", res)
    } else if res.as_ref().unwrap().len() < 1 {
        String::from("No such API key!")
    } else {
        format!("Yo, {}!", res.unwrap()[0][0])
    }
}
