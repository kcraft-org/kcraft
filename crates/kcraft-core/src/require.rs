use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Require {
    pub uid: String,
    #[serde(default)]
    pub equals_version: Option<String>,
    #[serde(default)]
    pub suggests: Option<String>,
}

impl Require {
    pub fn new(uid: String) -> Self {
        Require {
            uid,
            equals_version: None,
            suggests: None,
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.equals_version = Some(version);
        self
    }

    pub fn with_suggestion(mut self, version: String) -> Self {
        self.suggests = Some(version);
        self
    }
}

impl Ord for Require {
    fn cmp(&self, other: &Self) -> Ordering {
        self.uid.cmp(&other.uid)
    }
}

impl PartialOrd for Require {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub type RequireSet = Vec<Require>;
