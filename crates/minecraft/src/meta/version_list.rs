use std::collections::HashMap;

use super::version::Version;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
