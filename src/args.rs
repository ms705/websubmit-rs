use crate::config;
use clap::{App, Arg};

#[cfg_attr(rustfmt, rustfmt_skip)]
const WEBSUBMIT_USAGE: &'static str = "\
EXAMPLES:
  websubmit -i csci2390
  websubmit -i csci2390 -c csci2390-f19.toml";

#[derive(Clone, Debug)]
pub struct Args {
    pub class: String,
    pub config: config::Config,
    pub email_notification_addr: Option<String>,
}

pub fn parse_args() -> Args {
    let args = App::new("websubmit")
        .version("0.0.1")
        .about("Class submission system.")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .value_name("CONFIG_FILE")
                .default_value("websubmit.toml")
                .help("Path to the configuration file for the deployment."),
        )
        .arg(
            Arg::with_name("class")
                .short("i")
                .long("class-id")
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
        config: config::parse(args.value_of("config").expect("Failed to parse config!")),
        email_notification_addr: args.value_of("email_addr").map(String::from),
    }
}
