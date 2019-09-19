#![feature(proc_macro_hygiene, decl_macro)]

extern crate clap;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate slog;
extern crate slog_term;

mod args;
mod backend;
mod questions;

use backend::{DataType, NoriaBackend};
use rocket::request::Form;
use rocket::response::NamedFile;
use rocket::State;
use std::io;
use std::sync::{Arc, Mutex};

pub fn new_logger() -> slog::Logger {
    use slog::Drain;
    use slog::Logger;
    use slog_term::term_full;
    Logger::root(Mutex::new(term_full()).fuse(), o!())
}

#[get("/")]
fn index() -> &'static str {
    "Hello, CSCI 2390!"
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
        .manage(b)
        .mount("/", routes![index])
        .mount(
            "/questions",
            routes![questions::questions, questions::questions_submit],
        )
        .mount("/answers", routes![questions::answers])
        .launch();
}
