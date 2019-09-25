#![feature(proc_macro_hygiene, decl_macro)]

extern crate clap;
#[macro_use]
extern crate rocket;
extern crate crypto;
#[macro_use]
extern crate slog;
extern crate slog_term;

mod admin;
mod apikey;
mod args;
mod backend;
mod questions;

use backend::NoriaBackend;
use rocket::http::Cookies;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn new_logger() -> slog::Logger {
    use slog::Drain;
    use slog::Logger;
    use slog_term::term_full;
    Logger::root(Mutex::new(term_full()).fuse(), o!())
}

#[get("/")]
fn index(cookies: Cookies) -> Template {
    if let Some(_apikey) = cookies.get("apikey") {
        // TODO validate API key
        //check_apikey(apikey)
        Template::render("leclist", HashMap::<String, String>::new())
    } else {
        Template::render("login", HashMap::<String, String>::new())
    }
}

fn main() {
    let args = args::parse_args();

    let b = Arc::new(Mutex::new(
        NoriaBackend::new(
            &format!("127.0.0.1:2181/{}", args.class),
            Some(new_logger()),
        )
        .unwrap(),
    ));

    rocket::ignite()
        .attach(Template::fairing())
        .manage(b)
        .mount("/", routes![index])
        .mount(
            "/questions",
            routes![questions::questions, questions::questions_submit],
        )
        .mount("/apikey/check", routes![apikey::check])
        .mount("/apikey/generate", routes![apikey::generate])
        .mount("/answers", routes![questions::answers])
        .mount("/leclist", routes![questions::leclist])
        .mount("/admin", routes![admin::admin])
        .launch();
}
