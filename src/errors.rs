use std::fmt;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
    Toml(String),
    Config(String),
    TaskNotFound(String),
    ParseError(String),
    CommandError(String),
    Terminal(String),
    Editor(String),
    HomeDir,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Yaml(e) => write!(f, "YAML error: {}", e),
            AppError::Toml(e) => write!(f, "TOML error: {}", e),
            AppError::Config(e) => write!(f, "Config error: {}", e),
            AppError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            AppError::ParseError(e) => write!(f, "Parse error: {}", e),
            AppError::CommandError(e) => write!(f, "Command error: {}", e),
            AppError::Terminal(e) => write!(f, "Terminal error: {}", e),
            AppError::Editor(e) => write!(f, "Editor error: {}", e),
            AppError::HomeDir => write!(f, "Could not find home directory"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e) => Some(e),
            AppError::Yaml(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(e: serde_yaml::Error) -> Self {
        AppError::Yaml(e)
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(e: toml::ser::Error) -> Self {
        AppError::Toml(e.to_string())
    }
}

impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        AppError::Toml(e.to_string())
    }
}
