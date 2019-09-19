use crate::backend::{DataType, NoriaBackend};
use rocket::request::Form;
use rocket::response::NamedFile;
use rocket::State;
use std::io;
use std::sync::{Arc, Mutex};

#[derive(Debug, FromForm)]
pub(crate) struct FormInput {
    q1: String,
    sq1: String,
}

#[get("/<num>")]
pub(crate) fn answers(num: u8, backend: State<Arc<Mutex<NoriaBackend>>>) -> String {
    let mut bg = backend.lock().unwrap();
    let mut h = bg.handle.view("answers_by_lec").unwrap().into_sync();

    let key: DataType = (num as u64).into();

    let res = h.lookup(&[key], true);

    format!("{:?}", res)
}

#[get("/<num>")]
pub(crate) fn questions(num: u8) -> io::Result<NamedFile> {
    NamedFile::open(format!("static/m{}.html", num))
}

#[post("/<num>", data = "<data>")]
pub(crate) fn questions_submit(
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
