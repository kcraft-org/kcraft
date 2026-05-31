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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackProfile {
    components: Vec<Component>,
    instance_root: String,
    profile: Option<LaunchProfile>,
    dirty: bool,
}

impl PackProfile {
    pub fn new(instance_root: &str) -> Self {
        PackProfile {
            components: Vec::new(),
            instance_root: instance_root.to_string(),
            profile: None,
            dirty: false,
        }
    }

    pub fn building_from_scratch(&mut self) {
        self.components.clear();
        self.dirty = true;
    }

    pub fn component(&self, id: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.uid == id)
    }

    pub fn component_by_index(&self, index: usize) -> Option<&Component> {
        self.components.get(index)
    }

    pub fn append_component(&mut self, component: Component) {
        self.components.push(component);
        self.dirty = true;
    }

    pub fn insert_component(&mut self, index: usize, component: Component) {
        let idx = index.min(self.components.len());
        self.components.insert(idx, component);
        self.dirty = true;
    }

    pub fn remove_component(&mut self, index: usize) -> Option<Component> {
        if index < self.components.len() {
            self.dirty = true;
            Some(self.components.remove(index))
        } else {
            None
        }
    }

    pub fn remove_component_by_id(&mut self, id: &str) -> Option<Component> {
        if let Some(pos) = self.components.iter().position(|c| c.uid == id) {
            self.dirty = true;
            Some(self.components.remove(pos))
        } else {
            None
        }
    }

    pub fn move_component(&mut self, index: usize, direction: MoveDirection) {
        if index >= self.components.len() {
            return;
        }
        let new_index = match direction {
            MoveDirection::Up => {
                if index == 0 {
                    return;
                }
                index - 1
            }
            MoveDirection::Down => {
                if index >= self.components.len() - 1 {
                    return;
                }
                index + 1
            }
        };
        self.components.swap(index, new_index);
        self.dirty = true;
    }

    pub fn set_component_version(&mut self, uid: &str, version: &str, important: bool) -> bool {
        if let Some(comp) = self.components.iter_mut().find(|c| c.uid == uid) {
            comp.version = version.to_string();
            comp.important = important;
            self.dirty = true;
            true
        } else {
            let mut comp = Component::new(uid);
            comp.version = version.to_string();
            comp.important = important;
            self.components.push(comp);
            self.dirty = true;
            true
        }
    }

    pub fn get_component_version(&self, uid: &str) -> Option<&str> {
        self.components
            .iter()
            .find(|c| c.uid == uid)
            .map(|c| c.version.as_str())
    }

    pub fn resolve(&mut self, os: &crate::OpSys) {
        let mut profile = LaunchProfile::new();

        for component in &self.components {
            if !component.disabled {
                component.apply_to(&mut profile, os);
            }
        }

        self.profile = Some(profile);
    }

    pub fn get_profile(&self) -> Option<&LaunchProfile> {
        self.profile.as_ref()
    }

    pub fn components(&self) -> &[Component] {
        &self.components
    }

    pub fn components_mut(&mut self) -> &mut Vec<Component> {
        &mut self.components
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn save_now(&mut self) {
        if !self.dirty {
            return;
        }
        self.save_components();
        self.dirty = false;
    }

    fn save_components(&self) {
        let path = std::path::Path::new(&self.instance_root).join("mmc-pack.json");
        if let Ok(json) = serde_json::to_string_pretty(&self.components) {
            let _ = std::fs::write(&path, json);
        }
    }

    pub fn load_components(&mut self) {
        let path = std::path::Path::new(&self.instance_root).join("mmc-pack.json");
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(components) = serde_json::from_str::<Vec<Component>>(&content) {
                self.components = components;
                self.dirty = false;
                return;
            }
        }
        self.components.clear();
        self.dirty = false;
    }

    pub fn is_loaded(&self) -> bool {
        !self.components.is_empty()
    }

    pub fn get_mod_loaders(&self) -> Vec<String> {
        let mut loaders = Vec::new();
        for component in &self.components {
            match component.uid.as_str() {
                "net.minecraftforge" => loaders.push("forge".to_string()),
                "net.neoforged" => loaders.push("neoforge".to_string()),
                "net.fabricmc.fabric-loader" => loaders.push("fabric".to_string()),
                "org.quiltmc.quilt-loader" => loaders.push("quilt".to_string()),
                _ => {}
            }
        }
        loaders
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
}

pub fn components_file_path(instance_root: &str) -> String {
    std::path::Path::new(instance_root)
        .join("mmc-pack.json")
        .to_string_lossy()
        .to_string()
}

pub fn patches_pattern(instance_root: &str) -> String {
    std::path::Path::new(instance_root)
        .join("patches")
        .join("*.json")
        .to_string_lossy()
        .to_string()
}
