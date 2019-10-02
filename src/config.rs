//use toml;

#[derive(Debug, Clone)]
pub struct Config {
    /// Textual identifier for class
    pub class: String,
    /// Staff email addresses
    pub staff: Vec<String>,
}

pub(crate) fn parse(_path: &str) -> Config {
    Config {
        class: "csci2390".into(),
        staff: vec!["malte@cs.brown.edu".into()],
    }
}
