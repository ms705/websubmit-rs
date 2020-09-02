#![feature(proc_macro_hygiene, decl_macro)]
#![feature(crate_visibility_modifier)]

extern crate clap;
extern crate crypto;
#[macro_use]
extern crate rocket;
extern crate lettre;
extern crate lettre_email;
#[macro_use]
extern crate slog;
extern crate slog_term;
#[macro_use]
extern crate serde_derive;

mod admin;
mod apikey;
mod args;
mod backend;
mod config;
mod email;
mod login;
mod questions;

use backend::NoriaBackend;
use rocket::http::Cookies;
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::templates::Template;
use std::sync::{Arc, Mutex};

pub fn new_logger() -> slog::Logger {
    use slog::Drain;
    use slog::Logger;
    use slog_term::term_full;
    Logger::root(Mutex::new(term_full()).fuse(), o!())
}

#[get("/")]
fn index(cookies: Cookies, backend: State<Arc<Mutex<NoriaBackend>>>) -> Redirect {
    if let Some(cookie) = cookies.get("apikey") {
        let apikey: String = cookie.value().parse().ok().unwrap();
        // TODO validate API key
        match apikey::check_api_key(&*backend, &apikey) {
            Ok(_user) => Redirect::to("/leclist"),
            Err(_) => Redirect::to("/login"),
        }
    } else {
        Redirect::to("/login")
    }
}

fn main() {
    use rocket_contrib::serve::StaticFiles;
    use rocket_contrib::templates::Engines;
    use std::path::Path;

    let args = args::parse_args();
    let noria = NoriaBackend::new(
        &format!("127.0.0.1:2181/{}", args.class),
        &args.class,
        Some(new_logger()),
    )
    .unwrap();

    let backend = Arc::new(Mutex::new(noria));

    let config = args.config;

    let template_dir = config.template_dir.clone();
    let resource_dir = config.resource_dir.clone();

    rocket::ignite()
        .attach(Template::custom(move |engines: &mut Engines| {
            engines
                .handlebars
                .register_templates_directory(".hbs", Path::new(&template_dir))
                .expect("failed to set template path!");
        }))
        .manage(backend)
        .manage(config)
        .mount("/css", StaticFiles::from(format!("{}/css", resource_dir)))
        .mount("/js", StaticFiles::from(format!("{}/js", resource_dir)))
        .mount("/", routes![index])
        .mount(
            "/questions",
            routes![questions::questions, questions::questions_submit],
        )
        .mount("/apikey/check", routes![apikey::check])
        .mount("/apikey/resubscribe", routes![apikey::resubscribe])
        .mount("/apikey/generate", routes![apikey::generate])
        .mount("/apikey/remove_data", routes![apikey::remove_data])
        .mount("/answers", routes![questions::answers])
        .mount("/leclist", routes![questions::leclist])
        .mount("/login", routes![login::login])
        .mount(
            "/admin/lec/add",
            routes![admin::lec_add, admin::lec_add_submit],
        )
        .mount("/admin/users", routes![admin::registered_users])
        .mount("/admin/lec", routes![admin::lec])
        .mount(
            "/admin/lec",
            routes![admin::lec, admin::addq, admin::editq, admin::editq_submit],
        )
        .launch();
}
