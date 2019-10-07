use crate::apikey::ApiKey;
use crate::backend::{DataType, NoriaBackend};
use rocket::request::{Form, FormItems, FromForm};
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub(crate) enum LectureQuestionFormError {
    Invalid,
}

#[derive(Debug)]
pub(crate) struct LectureQuestionSubmission {
    answers: Vec<(u64, String)>,
}

#[derive(Serialize)]
struct LectureQuestion {
    id: u64,
    prompt: String,
}

#[derive(Serialize)]
struct LectureQuestionsContext {
    lec_id: u8,
    questions: Vec<LectureQuestion>,
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
        .lookup(&[(0 as u64).into()], true)
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
pub(crate) fn answers(
    _apikey: ApiKey,
    num: u8,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> String {
    let mut bg = backend.lock().unwrap();
    let mut h = bg.handle.view("answers_by_lec").unwrap().into_sync();

    let key: DataType = (num as u64).into();

    let res = h.lookup(&[key], true);

    format!("{:?}", res)
}

#[get("/<num>")]
pub(crate) fn questions(
    _apikey: ApiKey,
    num: u8,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let mut h = bg.handle.view("qs_by_lec").unwrap().into_sync();
    let key: DataType = (num as u64).into();

    let res = h
        .lookup(&[key], true)
        .expect("lecture questions lookup failed");
    let qs: Vec<_> = res
        .into_iter()
        .map(|r| LectureQuestion {
            id: r[1].clone().into(),
            prompt: r[2].clone().into(),
        })
        .collect();

    let ctx = LectureQuestionsContext {
        lec_id: num,
        questions: qs,
    };
    Template::render("questions", &ctx)
}

impl<'f> FromForm<'f> for LectureQuestionSubmission {
    // In practice, we'd use a more descriptive error type.
    type Error = LectureQuestionFormError;

    fn from_form(
        items: &mut FormItems<'f>,
        strict: bool,
    ) -> Result<LectureQuestionSubmission, Self::Error> {
        let mut answers: Vec<(u64, String)> = vec![];

        for item in items {
            let (key, value) = item.key_value_decoded();
            if key.as_str().starts_with("q_") {
                let num = u64::from_str_radix(&key.as_str()[2..], 10)
                    .map_err(|_| LectureQuestionFormError::Invalid)?;
                answers.push((num, value));
            } else {
                if strict {
                    return Err(LectureQuestionFormError::Invalid);
                } else {
                    /* allow extra value when not strict */
                }
            }
        }

        Ok(LectureQuestionSubmission { answers })
    }
}

#[post("/<num>", data = "<data>")]
pub(crate) fn questions_submit(
    apikey: ApiKey,
    num: u8,
    data: Form<LectureQuestionSubmission>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Redirect {
    let mut bg = backend.lock().unwrap();

    let num: DataType = (num as u64).into();

    let mut table = bg.handle.table("answers").unwrap().into_sync();

    for (id, answer) in &data.answers {
        let rec: Vec<DataType> = vec![
            apikey.user.clone().into(),
            num.clone(),
            (*id).into(),
            answer.clone().into(),
        ];
        format!("inserting: {:?}", rec);
        table.insert(rec).expect("failed to write answer!");
    }

    Redirect::to(format!("/answers/{}", num))
}
