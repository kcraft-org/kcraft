use crate::JavaVersion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaInstall {
    pub id: JavaVersion,
    pub arch: String,
    pub path: String,
    pub recommended: bool,
}

impl JavaInstall {
    pub fn new(id: JavaVersion, arch: String, path: String) -> Self {
        JavaInstall {
            id,
            arch,
            path,
            recommended: false,
        }
    }

    pub fn descriptor(&self) -> String {
        self.id.to_string()
    }

    pub fn name(&self) -> String {
        self.id.to_string()
    }

    pub fn type_string(&self) -> &str {
        &self.arch
    }
}

impl PartialEq for JavaInstall {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for JavaInstall {}

impl Ord for JavaInstall {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.id.cmp(&self.id)
    }
}

impl PartialOrd for JavaInstall {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
