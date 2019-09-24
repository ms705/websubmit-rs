use crate::backend::{DataType, NoriaBackend};
use rocket::request::Form;
use rocket::response::NamedFile;
use rocket::State;
use rocket_contrib::templates::{handlebars, Template};
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

#[derive(Debug, FromForm)]
pub(crate) struct QuestionConfig {
    num_qs: u8,
}

#[get("/lec/<num>")]
pub(crate) fn admin(num: u8, backend: State<Arc<Mutex<NoriaBackend>>>) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("LEC_NUM", num);
    Template::render("admin/lec", &ctx)
}

#[post("/lec/<num>", data = "<data>")]
pub(crate) fn admin_submit(
    num: u8,
    data: Form<QuestionConfig>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> String {
    String::from("Lecture updated")
}
