use std::collections::HashMap;

use super::version::Version;
use super::version_list::VersionList;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
