pub mod version;
pub mod install;
pub mod utils;
pub mod checker;

pub use version::*;
pub use install::*;
pub use utils::*;
pub use checker::*;

#[derive(Debug, thiserror::Error)]
pub enum JavaError {
    #[error("Java not found")]
    NotFound,
    #[error("Invalid Java version: {0}")]
    InvalidVersion(String),
    #[error("Checker process failed: {0}")]
    CheckerFailed(String),
    #[error("Checker timed out")]
    CheckerTimeout,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
