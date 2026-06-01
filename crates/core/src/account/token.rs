use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::Validity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    #[serde(default)]
    pub issue_instant: Option<i64>,
    #[serde(default)]
    pub not_after: Option<i64>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
    #[serde(default)]
    pub validity: Validity,
    #[serde(default = "default_true")]
    pub persistent: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Token {
    fn default() -> Self {
        Token {
            issue_instant: None,
            not_after: None,
            token: None,
            refresh_token: None,
            extra: HashMap::new(),
            validity: Validity::None,
            persistent: true,
        }
    }
}
