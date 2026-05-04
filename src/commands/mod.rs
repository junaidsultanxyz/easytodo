pub mod parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    New {
        title: String,
        description: String,
        due: Option<String>,
    },
    Open(String),
    Edit {
        id: String,
        fields: Vec<(String, String)>,
    },
    Delete(String),
    Done(String),
    Undone(String),
    Clone(String),
    List(Option<String>),
    Config {
        key: Option<String>,
        value: Option<String>,
    },
    Migrate(String),
    Reload,
    Quit,
}
