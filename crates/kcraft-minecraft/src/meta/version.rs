use crate::MetaRequire;

use super::parsers::{parse_meta_time, parse_requires};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Version {
    pub uid: String,
    pub version: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub time: i64,
    pub requires: Vec<MetaRequire>,
    pub conflicts: Vec<MetaRequire>,
    #[serde(default)]
    pub volatile: bool,
    #[serde(default)]
    pub recommended: bool,
    #[serde(skip)]
    pub provides_recommendations: bool,
}

impl Version {
    pub fn new() -> Self {
        Version {
            uid: String::new(),
            version: String::new(),
            type_: "release".to_string(),
            time: 0,
            requires: Vec::new(),
            conflicts: Vec::new(),
            volatile: false,
            recommended: false,
            provides_recommendations: false,
        }
    }

    pub fn with_id(uid: &str, version: &str) -> Self {
        Version {
            uid: uid.to_string(),
            version: version.to_string(),
            type_: "release".to_string(),
            time: 0,
            requires: Vec::new(),
            conflicts: Vec::new(),
            volatile: false,
            recommended: false,
            provides_recommendations: false,
        }
    }

    pub fn descriptor(&self) -> String {
        format!("{} {}", self.uid, self.version)
    }

    pub fn name(&self) -> String {
        format!("{} {}", self.uid, self.version)
    }

    pub fn type_string(&self) -> &str {
        &self.type_
    }

    pub fn merge(&mut self, other: &Version) {
        if other.type_ != "release" {
            self.type_ = other.type_.clone();
        }
        if other.time != 0 {
            self.time = other.time;
        }
        if !other.requires.is_empty() {
            self.requires = other.requires.clone();
        }
        if !other.conflicts.is_empty() {
            self.conflicts = other.conflicts.clone();
        }
        self.volatile = other.volatile;
        self.recommended = other.recommended || self.recommended;
    }

    pub fn parse(&mut self, json: serde_json::Value) {
        let obj = json.as_object().expect("expected object");
        if let Some(v) = obj.get("version").and_then(|v| v.as_str()) {
            self.version = v.to_string();
        }
        if let Some(v) = obj.get("type").and_then(|v| v.as_str()) {
            self.type_ = v.to_string();
        }
        if let Some(v) = obj.get("time").and_then(|v| v.as_str()) {
            if let Some(parsed) = parse_meta_time(v) {
                self.time = parsed;
            }
        }
        if let Some(v) = obj.get("requires").and_then(|v| v.as_array()) {
            self.requires = parse_requires(v);
        }
        if let Some(v) = obj.get("conflicts").and_then(|v| v.as_array()) {
            self.conflicts = parse_requires(v);
        }
        if let Some(v) = obj.get("volatile").and_then(|v| v.as_bool()) {
            self.volatile = v;
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new()
    }
}
