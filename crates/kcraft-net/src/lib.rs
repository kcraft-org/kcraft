mod download;
mod net_job;
mod sink;
mod upload;
mod validator;

pub use download::*;
pub use net_job::*;
pub use sink::*;
pub use upload::*;
pub use validator::*;

use thiserror::Error;
#[derive(Debug, Error)]
pub enum NetError {
    #[error("HTTP error: {0} ({1})")]
    HttpError(u16, String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Cancelled")]
    Cancelled,
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Timeout")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, NetError>;

impl From<reqwest::Error> for NetError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            NetError::Timeout
        } else if let Some(status) = e.status() {
            NetError::HttpError(status.as_u16(), e.to_string())
        } else {
            NetError::Network(e.to_string())
        }
    }
}

impl From<url::ParseError> for NetError {
    fn from(e: url::ParseError) -> Self {
        NetError::InvalidUrl(e.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetMode {
    Offline,
    Online,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Inactive,
    Running,
    Succeeded,
    Failed,
    AbortedByUser,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NetEvent {
    pub job_name: String,
    pub total_actions: usize,
    pub completed_actions: usize,
    pub failed_actions: usize,
}

lazy_static::lazy_static! {
    pub static ref NET_EVENTS: tokio::sync::broadcast::Sender<NetEvent> = {
        let (tx, _) = tokio::sync::broadcast::channel(100);
        tx
    };
}

pub fn is_application_error(status: u16) -> bool {
    matches!(status, 401 | 403 | 404 | 405 | 410 | 429 | 500..=599)
}
