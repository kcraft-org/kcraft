use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::MetaRequire;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub lists: Vec<VersionList>,
    #[serde(skip)]
    pub uids: HashMap<String, usize>,
}

impl Index {
    pub fn new() -> Self {
        Index {
            lists: Vec::new(),
            uids: HashMap::new(),
        }
    }

    pub fn get_list(&self, uid: &str) -> Option<&VersionList> {
        self.uids.get(uid).and_then(|&i| self.lists.get(i))
    }

    pub fn get_list_mut(&mut self, uid: &str) -> Option<&mut VersionList> {
        let idx = self.uids.get(uid).copied()?;
        self.lists.get_mut(idx)
    }

    pub fn get_version(&self, uid: &str, version: &str) -> Option<&Version> {
        self.get_list(uid)?.get_version(version)
    }

    pub fn has_uid(&self, uid: &str) -> bool {
        self.uids.contains_key(uid)
    }

    pub fn merge(&mut self, other: &Index) {
        for other_list in &other.lists {
            let uid = &other_list.uid;
            if let Some(existing) = self.get_list_mut(uid) {
                existing.merge_from_index(other_list);
            } else {
                let idx = self.lists.len();
                self.lists.push(other_list.clone());
                self.uids.insert(uid.clone(), idx);
            }
        }
    }

    pub fn parse_index(&mut self, json: serde_json::Value) {
        let root = json.as_object().expect("expected object");
        let lists_val = root
            .get("versionLists")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        for list_val in lists_val {
            let mut vl = VersionList::new();
            vl.parse(list_val);
            let uid = vl.uid.clone();
            if !self.uids.contains_key(&uid) {
                self.uids.insert(uid.clone(), self.lists.len());
                self.lists.push(vl);
            }
        }
    }

    pub fn merge_from_list(&mut self, uid: &str, list: &VersionList) {
        if let Some(existing) = self.get_list_mut(uid) {
            existing.merge(list);
        }
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionList {
    pub uid: String,
    pub name: String,
    pub versions: Vec<Version>,
    #[serde(skip)]
    pub lookup: HashMap<String, usize>,
}

impl Default for VersionList {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionList {
    pub fn new() -> Self {
        VersionList {
            uid: String::new(),
            name: String::new(),
            versions: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    pub fn with_uid(uid: &str) -> Self {
        let mut vl = VersionList::new();
        vl.uid = uid.to_string();
        vl
    }

    pub fn get_version(&self, version: &str) -> Option<&Version> {
        self.lookup.get(version).and_then(|&i| self.versions.get(i))
    }

    pub fn get_version_mut(&mut self, version: &str) -> Option<&mut Version> {
        let idx = self.lookup.get(version).copied()?;
        self.versions.get_mut(idx)
    }

    pub fn has_version(&self, version: &str) -> bool {
        self.lookup.contains_key(version)
    }

    pub fn add_version(&mut self, version: Version) {
        let ver = version.version.clone();
        self.lookup.insert(ver.clone(), self.versions.len());
        self.versions.push(version);
    }

    pub fn recommended(&self) -> Option<&Version> {
        self.versions.iter().find(|v| v.recommended)
    }

    pub fn merge(&mut self, other: &VersionList) {
        for other_ver in &other.versions {
            let ver = &other_ver.version;
            if let Some(existing) = self.get_version_mut(ver) {
                existing.merge(other_ver);
            } else {
                self.add_version(other_ver.clone());
            }
        }
    }

    pub fn merge_from_index(&mut self, other: &VersionList) {
        for other_ver in &other.versions {
            let ver = &other_ver.version;
            if !self.has_version(ver) {
                let mut v = other_ver.clone();
                v.provides_recommendations = false;
                self.add_version(v);
            }
        }
    }

    pub fn parse(&mut self, json: serde_json::Value) {
        let obj = json.as_object().expect("expected object");
        if let Some(uid) = obj.get("uid").and_then(|v| v.as_str()) {
            self.uid = uid.to_string();
        }
        if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
            self.name = name.to_string();
        }
        if let Some(versions) = obj.get("versions").and_then(|v| v.as_array()) {
            for ver_val in versions {
                let mut v = Version::new();
                v.parse(ver_val.clone());
                self.add_version(v);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn parse_meta_time(s: &str) -> Option<i64> {
    // Try ISO 8601 format
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp_millis());
    }
    // Try unix timestamp (seconds)
    if let Ok(ts) = s.parse::<i64>() {
        return Some(ts);
    }
    None
}

fn parse_requires(arr: &[serde_json::Value]) -> Vec<MetaRequire> {
    arr.iter()
        .filter_map(|v| {
            let obj = v.as_object()?;
            let uid = obj.get("uid")?.as_str()?;
            Some(MetaRequire {
                uid: uid.to_string(),
                equals: obj
                    .get("equals")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                suggests: obj
                    .get("suggests")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        })
        .collect()
}
