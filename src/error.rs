//! Single error type: a human-readable message (multiple findings joined by newlines).
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Error(pub String);

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error(format!("invalid JSON: {e}"))
    }
}

impl From<roxmltree::Error> for Error {
    fn from(e: roxmltree::Error) -> Self {
        Error(format!("invalid XML: {e}"))
    }
}

pub fn err<T>(message: impl Into<String>) -> Result<T> {
    Err(Error(message.into()))
}
