pub mod version;
pub mod account;
pub mod session;
pub mod build_config;
pub mod require;
pub mod runtime;
pub mod settings;
pub mod task;

pub use version::*;
pub use account::*;
pub use session::*;
pub use build_config::*;
pub use require::*;
pub use runtime::*;
pub use settings::*;

use serde::{Deserialize, Serialize};
use std::fmt;

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Invalid version string: {0}")]
    InvalidVersion(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        CoreError::Serialization(e.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Linux,
    Osx,
    Windows,
    Unknown,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        { Platform::Linux }
        #[cfg(target_os = "macos")]
        { Platform::Osx }
        #[cfg(target_os = "windows")]
        { Platform::Windows }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        { Platform::Unknown }
    }

    pub fn classifier(&self) -> &str {
        match self {
            Platform::Linux => "linux",
            Platform::Osx => "osx",
            Platform::Windows => "windows",
            Platform::Unknown => "unknown",
        }
    }

    pub fn arch_classifier(&self) -> &str {
        let arch = std::env::consts::ARCH;
        match arch {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            "arm" => "arm",
            _ => arch,
        }
    }

    pub fn full_classifier(&self) -> String {
        format!("{}-{}", self.classifier(), self.arch_classifier())
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.classifier())
    }
}
