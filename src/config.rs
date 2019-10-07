use std::fs;
use std::io::{Error, ErrorKind, Read};
use toml;

#[derive(Debug, Clone)]
pub struct Config {
    /// Textual identifier for class
    pub class: String,
    /// Staff email addresses
    pub staff: Vec<String>,
}

pub(crate) fn parse(path: &str) -> Result<Config, Error> {
    let mut f = fs::File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;

    let value = match toml::Parser::new(&buf).parse() {
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "failed to parse config!",
            ))
        }
        Some(v) => v,
    };

    Ok(Config {
        class: value.get("class").unwrap().as_str().unwrap().into(),
        staff: value
            .get("staff")
            .unwrap()
            .as_slice()
            .unwrap()
            .into_iter()
            .map(|v| v.as_str().unwrap().into())
            .collect(),
    })
}
