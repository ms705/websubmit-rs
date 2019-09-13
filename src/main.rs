#![feature(proc_macro_hygiene, decl_macro)]

extern crate clap;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate slog;
extern crate slog_term;

mod args;
mod backend;

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

#[derive(Debug, FromForm)]
struct FormInput {
    q1: String,
    sq1: String,
}

#[get("/<num>")]
fn answers(num: u8, backend: State<Arc<Mutex<NoriaBackend>>>) -> String {
    let mut bg = backend.lock().unwrap();
    let mut h = bg.handle.view("answers_by_lec").unwrap().into_sync();

    let key: DataType = (num as u64).into();

    let res = h.lookup(&[key], true);

    format!("{:?}", res)
}

#[get("/<num>")]
fn questions(num: u8) -> io::Result<NamedFile> {
    NamedFile::open(format!("static/m{}.html", num))
}

#[post("/<num>", data = "<data>")]
fn questions_submit(
    num: u8,
    data: Form<FormInput>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> String {
    let mut bg = backend.lock().unwrap();

    let num: DataType = (num as u64).into();

    let q1: Vec<DataType> = vec![
        "malte".into(),
        num.clone(),
        1.into(),
        data.q1.clone().into(),
    ];
    let sq1: Vec<DataType> = vec!["malte".into(), num, 2.into(), data.sq1.clone().into()];

    let mut table = bg.handle.table("answers").unwrap().into_sync();

    let res = table.insert(q1);
    let res2 = table.insert(sq1);

    format!("submitted: {:?} {:?}", res, res2)
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
        .mount("/questions", routes![questions, questions_submit])
        .mount("/answers", routes![answers])
        .launch();
}
