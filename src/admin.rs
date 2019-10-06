use crate::apikey::ApiKey;
use crate::backend::NoriaBackend;
use crate::config::Config;
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::request::Form;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub(crate) struct Admin;

#[derive(Debug)]
pub(crate) enum AdminError {
    Unauthorized,
}

impl<'a, 'r> FromRequest<'a, 'r> for Admin {
    type Error = AdminError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Admin, Self::Error> {
        let apikey = request.guard::<ApiKey>().unwrap();

        let cfg = request.guard::<State<Config>>().unwrap();

        let res = if cfg.staff.contains(&apikey.user) {
            Some(Admin)
        } else {
            None
        };

        res.into_outcome((Status::Unauthorized, AdminError::Unauthorized))
    }
}

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
pub(crate) fn lec_add(_adm: Admin) -> Template {
    Template::render("admin/lecadd", HashMap::<String, String>::new())
}

#[post("/", data = "<data>")]
pub(crate) fn lec_add_submit(
    _adm: Admin,
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
pub(crate) fn lec(_adm: Admin, num: u8, _backend: State<Arc<Mutex<NoriaBackend>>>) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("LEC_NUM", num);
    Template::render("admin/lec", &ctx)
}

#[post("/<_num>", data = "<_data>")]
pub(crate) fn lec_submit(
    _adm: Admin,
    _num: u8,
    _data: Form<QuestionConfig>,
    _backend: State<Arc<Mutex<NoriaBackend>>>,
) -> String {
    String::from("Lecture updated")
}
