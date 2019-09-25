use crate::backend::NoriaBackend;
use rocket::request::Form;
use rocket::State;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, FromForm)]
pub(crate) struct QuestionConfig {
    num_qs: u8,
}

#[get("/lec/<num>")]
pub(crate) fn admin(num: u8, _backend: State<Arc<Mutex<NoriaBackend>>>) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("LEC_NUM", num);
    Template::render("admin/lec", &ctx)
}

#[post("/lec/<_num>", data = "<_data>")]
pub(crate) fn admin_submit(
    _num: u8,
    _data: Form<QuestionConfig>,
    _backend: State<Arc<Mutex<NoriaBackend>>>,
) -> String {
    String::from("Lecture updated")
}
