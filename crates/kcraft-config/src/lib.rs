pub mod ini;
pub mod setting;
pub mod settings_object;

pub use ini::*;
pub use setting::*;
pub use settings_object::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Setting not found: {0}")]
    SettingNotFound(String),
    #[error("Type mismatch for setting {0}")]
    TypeMismatch(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for ConfigError {
    fn from(e: serde_json::Error) -> Self {
        ConfigError::Serialization(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ConfigError>;
