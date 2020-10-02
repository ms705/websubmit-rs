use crate::apikey::ApiKey;
use crate::backend::NoriaBackend;
use crate::config::Config;
use crate::questions::{LectureQuestion, LectureQuestionsContext};
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

        let res = if cfg.admins.contains(&apikey.user) {
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

#[derive(Debug, Serialize)]
pub(crate) struct User {
    email: String,
    apikey: String,
    is_admin: u8,
}

#[derive(Serialize)]
struct UserContext {
    users: Vec<User>,
    parent: &'static str,
}

#[get("/")]
pub(crate) fn lec_add(_adm: Admin) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("parent", String::from("layout"));
    Template::render("admin/lecadd", &ctx)
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
pub(crate) fn lec(_adm: Admin, num: u8, backend: State<Arc<Mutex<NoriaBackend>>>) -> Template {
    let mut bg = backend.lock().unwrap();
    let mut view = bg.handle.view("qs_by_lec").unwrap().into_sync();
    let res = view
        .lookup(&[(num as u64).into()], true)
        .expect("failed to read questions for lecture!");

    let mut qs: Vec<_> = res
        .into_iter()
        .map(|r| {
            let id: u64 = r[1].clone().into();
            LectureQuestion {
                id: id,
                prompt: r[2].clone().into(),
                answer: None,
            }
        })
        .collect();
    qs.sort_by(|a, b| b.id.cmp(&a.id));

    let ctx = LectureQuestionsContext {
        lec_id: num,
        questions: qs,
        parent: "layout",
    };
    Template::render("admin/lec", &ctx)
}

#[post("/<num>", data = "<data>")]
pub(crate) fn addq(
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
        .expect("failed to add question!");

    Redirect::to(format!("/admin/lec/{}", num))
}

#[get("/<num>/<qnum>")]
pub(crate) fn editq(
    _adm: Admin,
    num: u8,
    qnum: u8,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let mut view = bg.handle.view("qs_by_lec").unwrap().into_sync();
    let res = view
        .lookup(&[(num as u64).into()], true)
        .expect("failed to read questions for lecture!");

    let mut ctx = HashMap::new();
    for r in res {
        if r[1] == (qnum as u64).into() {
            ctx.insert("lec_qprompt", r[2].clone().into());
        }
    }
    ctx.insert("lec_id", format!("{}", num));
    ctx.insert("lec_qnum", format!("{}", qnum));
    ctx.insert("parent", String::from("layout"));
    Template::render("admin/lec_edit", &ctx)
}

#[post("/editq/<num>", data = "<data>")]
pub(crate) fn editq_submit(
    _adm: Admin,
    num: u8,
    data: Form<AddLectureQuestionForm>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Redirect {
    use noria::Modification;

    let mut bg = backend.lock().unwrap();
    let mut table = bg.handle.table("questions").unwrap().into_sync();
    table
        .update(
            vec![(num as u64).into(), (data.q_id as u64).into()],
            vec![(2, Modification::Set(data.q_prompt.to_string().into()))],
        )
        .expect("failed to update question!");

    Redirect::to(format!("/admin/lec/{}", num))
}

#[get("/")]
pub(crate) fn get_registered_users(
    _adm: Admin,
    backend: State<Arc<Mutex<NoriaBackend>>>,
    config: State<Config>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let mut h = bg.handle.view("all_users").unwrap().into_sync();

    // 0 is a bogokey
    let res = h
        .lookup(&[(0 as u64).into()], true)
        .expect("lecture list lookup failed");

    let users: Vec<_> = res
        .into_iter()
        .map(|r| User {
            email: r[0].clone().into(),
            apikey: r[2].clone().into(),
            is_admin: if config.admins.contains(&r[0].clone().into()) {
                1
            } else {
                0
            }, // r[1].clone().into(), this type conversion does not work
        })
        .collect();

    let ctx = UserContext {
        users: users,
        parent: "layout",
    };
    Template::render("admin/users", &ctx)
}
