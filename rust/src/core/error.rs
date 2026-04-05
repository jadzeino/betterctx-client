use thiserror::Error;

#[derive(Error, Debug)]
pub enum LeanCtxError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("config error: {0}")]
    Config(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("tool execution failed: {0}")]
    ToolExecution(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("{0}")]
    Other(String),
}

impl From<toml::de::Error> for LeanCtxError {
    fn from(e: toml::de::Error) -> Self {
        LeanCtxError::Config(e.to_string())
    }
}

impl From<serde_json::Error> for LeanCtxError {
    fn from(e: serde_json::Error) -> Self {
        LeanCtxError::Parse(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, LeanCtxError>;
