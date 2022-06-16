use crate::admin::Admin;
use crate::apikey::ApiKey;
use crate::backend::{MySqlBackend, Value};
use crate::config::Config;
use crate::email;
use chrono::naive::NaiveDateTime;
use chrono::Local;
use mysql::from_value;
use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

//pub(crate) enum LectureQuestionFormError {
//   Invalid,
//}

#[derive(Debug, FromForm)]
pub(crate) struct LectureQuestionSubmission {
    answers: HashMap<u64, String>,
}

#[derive(Serialize)]
pub(crate) struct LectureQuestion {
    pub id: u64,
    pub prompt: String,
    pub answer: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct LectureQuestionsContext {
    pub lec_id: u8,
    pub questions: Vec<LectureQuestion>,
    pub parent: &'static str,
}

#[derive(Serialize)]
struct LectureAnswer {
    id: u64,
    user: String,
    answer: String,
    time: Option<NaiveDateTime>,
}

#[derive(Serialize)]
struct LectureAnswersContext {
    lec_id: u8,
    answers: Vec<LectureAnswer>,
    parent: &'static str,
}

#[derive(Serialize)]
struct LectureListEntry {
    id: u64,
    label: String,
    num_qs: u64,
    num_answered: u64,
}

#[derive(Serialize)]
struct LectureListContext {
    admin: bool,
    lectures: Vec<LectureListEntry>,
    parent: &'static str,
}

#[get("/")]
pub(crate) fn leclist(
    apikey: ApiKey,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT lectures.id, lectures.label, lec_qcount.qcount \
         FROM lectures \
         LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec)",
        vec![],
    );
    drop(bg);

    let user = apikey.user.clone();
    let admin = config.admins.contains(&user);

    let lecs: Vec<_> = res
        .into_iter()
        .map(|r| LectureListEntry {
            id: from_value(r[0].clone()),
            label: from_value(r[1].clone()),
            num_qs: if r[2] == Value::NULL {
                0u64
            } else {
                from_value(r[2].clone())
            },
            num_answered: 0u64,
        })
        .collect();

    let ctx = LectureListContext {
        admin: admin,
        lectures: lecs,
        parent: "layout",
    };

    Template::render("leclist", &ctx)
}

#[get("/<num>")]
pub(crate) fn answers(
    _admin: Admin,
    num: u8,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let key: Value = (num as u64).into();
    let res = bg.prep_exec("SELECT email, lec, q, answer, submitted_at FROM answers WHERE lec = ?", vec![key]);
    drop(bg);
    let answers: Vec<_> = res
        .into_iter()
        .map(|r| LectureAnswer {
            id: from_value(r[2].clone()),
            user: from_value(r[0].clone()),
            answer: from_value(r[3].clone()),
            time:  NaiveDateTime::parse_from_str(&String::from_utf8(from_value::<Vec<u8>>(r[4].clone())).unwrap().chars().collect::<Vec<char>>()[..19].iter().collect::<String>(), "%Y-%m-%d %H:%M:%S").ok()
        })
        .collect();
    let ctx = LectureAnswersContext {
        lec_id: num,
        answers: answers,
        parent: "layout",
    };
    Template::render("answers", &ctx)
}

#[get("/<num>")]
pub(crate) fn questions(
    apikey: ApiKey,
    num: u8,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    use std::collections::HashMap;

    let mut bg = backend.lock().unwrap();
    let key: Value = (num as u64).into();

    let answers_res = bg.prep_exec(
        "SELECT answers.email, answers.lec, answers.q, answers.answer, answers.submitted_at FROM answers WHERE answers.lec = ? AND answers.email = ?",
        vec![(num as u64).into(), apikey.user.clone().into()],
    );
    let mut answers = HashMap::new();

    for r in answers_res {
        let id: u64 = from_value(r[2].clone());
        let atext: String = from_value(r[3].clone());
        answers.insert(id, atext);
    }
    let res = bg.prep_exec("SELECT lec , q , question  FROM questions WHERE lec = ?", vec![key]);
    drop(bg);
    let mut qs: Vec<_> = res
        .into_iter()
        .map(|r| {
            let id: u64 = from_value(r[1].clone());
            let answer = answers.get(&id).map(|s| s.to_owned());
            LectureQuestion {
                id: id,
                prompt: from_value(r[2].clone()),
                answer: answer,
            }
        })
        .collect();
    qs.sort_by(|a, b| a.id.cmp(&b.id));

    let ctx = LectureQuestionsContext {
        lec_id: num,
        questions: qs,
        parent: "layout",
    };
    Template::render("questions", &ctx)
}

#[post("/<num>", data = "<data>")]
pub(crate) fn questions_submit(
    apikey: ApiKey,
    num: u8,
    data: Form<LectureQuestionSubmission>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
) -> Redirect {
    let mut bg = backend.lock().unwrap();
    let vnum: Value = (num as u64).into();
    let ts: Value = Local::now().naive_local().to_string().into();

    for (id, answer) in &data.answers {
        let rec: Vec<Value> = vec![
            format!("{}-{}-{}", apikey.user.clone(), num, (*id)).into(),
            apikey.user.clone().into(),
            vnum.clone(),
            (*id).into(),
            answer.clone().into(),
            ts.clone(),
        ];
        bg.replace("answers", rec);
    }

    let answer_log = format!(
        "{}",
        data.answers
            .iter()
            .map(|(i, t)| format!("Question {}:\n{}", i, t))
            .collect::<Vec<_>>()
            .join("\n-----\n")
    );
    if config.send_emails {
        let recipients = if num < 90 {
            config.staff.clone()
        } else {
            config.admins.clone()
        };

        email::send(
            bg.log.clone(),
            apikey.user.clone(),
            recipients,
            format!("{} meeting {} questions", config.class, num),
            answer_log,
        )
        .expect("failed to send email");
    }
    drop(bg);

    Redirect::to("/leclist")
}
