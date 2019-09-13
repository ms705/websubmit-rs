use clap::{App, Arg};

#[cfg_attr(rustfmt, rustfmt_skip)]
const WEBSUBMIT_USAGE: &'static str = "\
EXAMPLES:
  websubmit -c csci2390";

#[derive(Clone, Debug)]
pub struct Args {
    pub class: String,
    pub email_notification_addr: Option<String>,
}

pub fn parse_args() -> Args {
    let args = App::new("websubmit")
        .version("0.0.1")
        .about("Class submission system.")
        .arg(
            Arg::with_name("class")
                .short("c")
                .long("class")
                .takes_value(true)
                .value_name("CLASS_ID")
                .required(true)
                .help("Short textual identifier for the class hosted (used as Noria deployment name)."),
        )
        .arg(
            Arg::with_name("email_addr")
                .long("email_addr")
                .takes_value(true)
                .required(false)
                .help("Email address to send notifications to"),
        )
        .after_help(WEBSUBMIT_USAGE)
        .get_matches();

    Args {
        class: String::from(args.value_of("class").unwrap()),
        email_notification_addr: args.value_of("email_addr").map(String::from),
    }
}
