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
pub(crate) struct AddLectureQuestionForm {
    q_id: u64,
    q_prompt: String,
}

#[derive(Debug, FromForm)]
pub(crate) struct AdminLecAdd {
    lec_id: u8,
    lec_label: String,
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
            (data.lec_id as u64).into(),
            data.lec_label.to_string().into(),
        ])
        .expect("failed to insert lecture!");

    Redirect::to("/leclist")
}

#[get("/<num>")]
pub(crate) fn lec(_adm: Admin, num: u8, _backend: State<Arc<Mutex<NoriaBackend>>>) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("lec_id", num);
    Template::render("admin/lec", &ctx)
}

#[post("/<num>", data = "<data>")]
pub(crate) fn lec_submit(
    _adm: Admin,
    num: u8,
    data: Form<AddLectureQuestionForm>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Redirect {
    let mut bg = backend.lock().unwrap();
    let mut table = bg.handle.table("questions").unwrap().into_sync();
    table
        .insert(vec![
            (num as u64).into(),
            (data.q_id as u64).into(),
            data.q_prompt.to_string().into(),
        ])
        .expect("failed to insert question!");

    Redirect::to(format!("/admin/lec/{}", num))
}
