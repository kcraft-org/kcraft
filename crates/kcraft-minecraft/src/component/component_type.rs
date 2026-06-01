use serde::{Deserialize, Serialize};

use crate::launch_profile::LaunchProfile;
use crate::version_file::VersionFile;
use crate::MetaRequire;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub uid: String,
    pub version: String,
    pub dependency_only: bool,
    pub important: bool,
    pub disabled: bool,
    pub order: Option<i32>,
    pub cached_name: Option<String>,
    pub cached_version: Option<String>,
    pub cached_requires: Vec<MetaRequire>,
    pub cached_conflicts: Vec<MetaRequire>,
    pub cached_volatile: bool,
    pub file: Option<VersionFile>,
    pub loaded: bool,
}

impl Component {
    pub fn new(uid: &str) -> Self {
        Component {
            uid: uid.to_string(),
            version: String::new(),
            dependency_only: false,
            important: false,
            disabled: false,
            order: None,
            cached_name: None,
            cached_version: None,
            cached_requires: Vec::new(),
            cached_conflicts: Vec::new(),
            cached_volatile: false,
            file: None,
            loaded: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        !self.disabled
    }

    pub fn can_be_disabled(&self) -> bool {
        !self.important
    }

    pub fn is_moveable(&self) -> bool {
        !self.important
    }

    pub fn is_customizable(&self) -> bool {
        self.file.is_some()
    }

    pub fn is_revertible(&self) -> bool {
        false
    }

    pub fn is_removable(&self) -> bool {
        !self.important && !self.dependency_only
    }

    pub fn is_custom(&self) -> bool {
        self.file.is_some()
    }

    pub fn is_version_changeable(&self) -> bool {
        true
    }

    pub fn apply_to(&self, profile: &mut LaunchProfile, os: &crate::OpSys) {
        if self.disabled {
            return;
        }
        if let Some(ref vf) = self.file {
            profile.apply_version_file(vf, os);
        }
    }

    pub fn get_id(&self) -> &str {
        &self.uid
    }

    pub fn get_name(&self) -> &str {
        self.cached_name.as_deref().unwrap_or(&self.uid)
    }

    pub fn get_version(&self) -> &str {
        self.cached_version.as_deref().unwrap_or(&self.version)
    }
}
