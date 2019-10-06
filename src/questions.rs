use crate::apikey::ApiKey;
use crate::backend::{DataType, NoriaBackend};
use rocket::request::Form;
use rocket::State;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, FromForm)]
pub(crate) struct FormInput {
    q1: String,
    sq1: String,
}

#[derive(Serialize)]
struct LectureListEntry {
    id: u64,
    label: String,
    num_qs: i64,
}

#[get("/")]
pub(crate) fn leclist(_apikey: ApiKey, backend: State<Arc<Mutex<NoriaBackend>>>) -> Template {
    let mut bg = backend.lock().unwrap();
    let mut h = bg.handle.view("leclist").unwrap().into_sync();

    let res = h
        .lookup(&[(0 as u64).into()], false)
        .expect("lecture list lookup failed");

    let lecs: Vec<_> = res
        .into_iter()
        .map(|r| LectureListEntry {
            id: r[0].clone().into(),
            label: r[1].clone().into(),
            num_qs: r[2].clone().into(),
        })
        .collect();
    let mut ctx = HashMap::new();
    ctx.insert("lectures", lecs);

    Template::render("leclist", &ctx)
}

#[get("/<num>")]
pub(crate) fn answers(num: u8, backend: State<Arc<Mutex<NoriaBackend>>>) -> String {
    let mut bg = backend.lock().unwrap();
    let mut h = bg.handle.view("answers_by_lec").unwrap().into_sync();

    let key: DataType = (num as u64).into();

    let res = h.lookup(&[key], true);

    format!("{:?}", res)
}

#[get("/<_num>")]
pub(crate) fn questions(_num: u8) -> Template {
    let ctx = HashMap::<String, String>::new();
    Template::render("questions", &ctx)
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
