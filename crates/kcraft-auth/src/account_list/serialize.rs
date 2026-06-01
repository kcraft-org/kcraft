use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AccountFile {
    #[serde(rename = "formatVersion")]
    pub(crate) format_version: u32,
    pub(crate) accounts: Vec<serde_json::Value>,
}
