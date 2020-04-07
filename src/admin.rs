use crate::apikey::get_users_email_keys;
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

#[derive(Debug, FromForm, Serialize)]
pub(crate) struct AddLectureQuestionForm {
  q_id: u64,
  q_prompt: String,
}

#[derive(Debug, FromForm)]
pub(crate) struct AdminLecAdd {
  lec_id: i64,
  lec_label: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct User {
  email: String,
  apikey: String,
  is_admin: i64,
}

#[derive(Serialize)]
struct UserContext {
  users: Vec<User>,
  parent: &'static str,
}

#[derive(Serialize)]
struct QuestionContext {
  questions: Vec<AddLectureQuestionForm>,
  parent: &'static str,
  lec_id: String,
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
pub(crate) fn lec(_adm: Admin, num: i64, backend: State<Arc<Mutex<NoriaBackend>>>) -> Template {
  let mut bg = backend.lock().unwrap();
  let mut qs_view = bg.handle.view("qs_by_lec").unwrap().into_sync();
  let lookup = qs_view
                      .lookup(&[num.into()], true)
                      .expect("failed to look up the user in a personal table");
  let curr_questions: Vec<_> = lookup
                      .into_iter()
                      .map(|r| AddLectureQuestionForm {
                        q_id: r[1].clone().into(),
                        q_prompt: r[2].clone().into(),
                      })
                      .collect();
  let ctx = QuestionContext {
    lec_id: format!("{}", num),
    parent: "layout",
    questions: curr_questions,
  };
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

#[get("/")]
pub(crate) fn get_registered_users(
  _adm: Admin,
  backend: State<Arc<Mutex<NoriaBackend>>>
) -> Template {
  let mut bg = backend.lock().unwrap();
  let email_keys = get_users_email_keys(&mut bg);
  let mut users = Vec::new();
  for email in email_keys.iter() {
    let mut personal_view = bg
      .handle
      .view(format!("userinfo_from_{}", email))
      .unwrap()
      .into_sync();
    let result = personal_view
      .lookup(&[0.into()], true)
      .expect("failed to look up the user in a personal table");

    let curr_users: Vec<_> = result
      .into_iter()
      .map(|r| User {
        email: r[0].clone().into(),
        apikey: r[2].clone().into(),
        is_admin: r[1].clone().into(),
      })
      .collect();
    for user in curr_users {
      users.push(user);
    }
  }


  let ctx = UserContext {
    users: users,
    parent: "layout",
  };
  Template::render("admin/users", &ctx)
}
