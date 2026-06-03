pub mod checker;
pub mod download;
pub mod install;
pub mod utils;
pub mod version;

pub use checker::*;
pub use download::*;
pub use install::*;
pub use utils::*;
pub use version::*;

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
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("Extraction failed: {0}")]
    ExtractionFailed(String),
    #[error("Network error: {0}")]
    Network(String),
}
