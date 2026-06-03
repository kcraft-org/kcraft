pub mod account_list;
pub mod authlib_injector;
pub mod flow;
pub mod msa;
pub mod offline;
pub mod parsers;
pub mod yggdrasil;

pub use account_list::*;
pub use authlib_injector::*;
pub use flow::*;
pub use msa::*;
pub use offline::*;
pub use parsers::*;
pub use yggdrasil::*;

use app_core::account::{AccountData, AccountTaskState};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Account not found")]
    AccountNotFound,
    #[error("Offline mode")]
    Offline,
    #[error("Cancelled by user")]
    Cancelled,
    #[error("Token expired")]
    TokenExpired,
    #[error("Account migrated")]
    Migrated,
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for AuthError {
    fn from(e: serde_json::Error) -> Self {
        AuthError::Serialization(e.to_string())
    }
}

impl From<app_core::CoreError> for AuthError {
    fn from(e: app_core::CoreError) -> Self {
        AuthError::Network(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AuthError>;

pub trait AuthFlow: Send {
    fn name(&self) -> &str;
    fn execute(&mut self, data: &mut AccountData) -> Result<AccountTaskState>;
}

pub trait AuthStep: Send {
    fn describe(&self) -> &str;
    fn perform(&mut self, data: &mut AccountData) -> Result<AccountTaskState>;
}

pub fn generate_offline_uuid(username: &str) -> String {
    use md5::{Digest, Md5};
    let namespace = b"OfflinePlayer:";
    let mut hasher = Md5::new();
    hasher.update(namespace);
    hasher.update(username.as_bytes());
    let digest = hasher.finalize();

    let digest_slice = &digest[..16];
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(digest_slice);

    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    )
}
