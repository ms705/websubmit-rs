use crate::backend::NoriaBackend;
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, FromForm)]
pub(crate) struct QuestionConfig {
    num_qs: u8,
}

#[derive(Debug, FromForm)]
pub(crate) struct AdminLecAdd {
    lecid: u8,
    leclabel: String,
}

#[get("/")]
pub(crate) fn lec_add() -> Template {
    Template::render("admin/lecadd", HashMap::<String, String>::new())
}

#[post("/", data = "<data>")]
pub(crate) fn lec_add_submit(
    data: Form<AdminLecAdd>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Redirect {
    // insert into Noria if not exists
    let mut bg = backend.lock().unwrap();
    let mut table = bg.handle.table("lectures").unwrap().into_sync();
    table
        .insert(vec![
            (data.lecid as u64).into(),
            data.leclabel.to_string().into(),
        ])
        .expect("failed to insert lecture!");

    Redirect::to("/leclist")
}

#[get("/<num>")]
pub(crate) fn lec(num: u8, _backend: State<Arc<Mutex<NoriaBackend>>>) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("LEC_NUM", num);
    Template::render("admin/lec", &ctx)
}

#[post("/<_num>", data = "<_data>")]
pub(crate) fn lec_submit(
    _num: u8,
    _data: Form<QuestionConfig>,
    _backend: State<Arc<Mutex<NoriaBackend>>>,
) -> String {
    String::from("Lecture updated")
}
