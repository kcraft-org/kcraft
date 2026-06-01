use std::collections::HashSet;
use std::path::Path;

use super::list::InstanceList;
use super::types::{InstanceId, InstancePtr};
use crate::instance::Instance;

impl InstanceList {
    fn discover_instances(&self) -> Vec<InstanceId> {
        let mut ids = Vec::new();
        let dir = match std::fs::read_dir(&self.inst_dir) {
            Ok(d) => d,
            Err(_) => return ids,
        };
        for entry in dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let cfg = path.join("instance.cfg");
                if cfg.exists() {
                    if let Some(name) = path.file_name() {
                        ids.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        ids
    }

    pub(super) fn load_instance(&self, id: &InstanceId) -> Option<Instance> {
        let inst_path = Path::new(&self.inst_dir).join(id);
        let cfg_path = inst_path.join("instance.cfg");
        if !cfg_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&cfg_path).ok()?;
        let mut ini = kcraft_core::INIFile::new();
        ini.load(&content);

        let name = ini.get("name").unwrap_or("Unnamed Instance").to_string();
        let mut instance = Instance::new(&inst_path.to_string_lossy(), &name);

        if let Some(v) = ini.get("InstanceType") {
            if v != "OneSix" && !v.is_empty() {
                instance.has_broken_version = true;
            }
        }
        if let Some(v) = ini.get("iconKey") {
            instance.icon_key = v.to_string();
        }
        if let Some(v) = ini.get("notes") {
            instance.notes = v.to_string();
        }
        if let Some(v) = ini.get("totalTimePlayed") {
            instance.total_time_played = v.parse().unwrap_or(0);
        }
        if let Some(v) = ini.get("lastLaunchTime") {
            instance.last_launch_time = v.parse().unwrap_or(0);
        }
        if let Some(v) = ini.get("JavaPath") {
            instance.java_path = v.to_string();
        }
        if let Some(v) = ini.get("JavaVersion") {
            instance.java_version = v.to_string();
        }
        if let Some(v) = ini.get("MinMemAlloc") {
            instance.min_mem = v.parse().unwrap_or(512);
        }
        if let Some(v) = ini.get("MaxMemAlloc") {
            instance.max_mem = v.parse().unwrap_or(2048);
        }
        if let Some(v) = ini.get("PermGen") {
            instance.perm_gen = v.parse().unwrap_or(64);
        }
        if let Some(v) = ini.get("JvmArgs") {
            instance.jvm_args = v.to_string();
        }
        if let Some(v) = ini.get("ManagedPack") {
            instance.managed_pack = v == "true";
        }
        if let Some(v) = ini.get("ManagedPackType") {
            instance.managed_pack_type = v.to_string();
        }
        if let Some(v) = ini.get("ManagedPackID") {
            instance.managed_pack_id = v.to_string();
        }
        if let Some(v) = ini.get("ManagedPackName") {
            instance.managed_pack_name = v.to_string();
        }
        if let Some(v) = ini.get("ManagedPackVersionID") {
            instance.managed_pack_version_id = v.to_string();
        }
        if let Some(v) = ini.get("ManagedPackVersionName") {
            instance.managed_pack_version_name = v.to_string();
        }
        if let Some(v) = ini.get("hasBrokenVersion") {
            instance.has_broken_version = v == "true";
        }
        if let Some(v) = ini.get("hasUpdate") {
            instance.has_update = v == "true";
        }
        if let Some(v) = ini.get("crashed") {
            instance.crashed = v == "true";
        }

        instance.components.load_components();

        Some(instance)
    }

    pub fn load_list(&mut self) -> Result<(), String> {
        self.load_group_list();
        let discovered = self.discover_instances();

        let existing: HashSet<String> =
            self.instances.iter().map(|inst| inst.read().id()).collect();

        for id in &discovered {
            if !existing.contains(id.as_str()) {
                if let Some(instance) = self.load_instance(id) {
                    self.instances.push(InstancePtr::new(instance));
                }
            }
        }

        self.instances
            .retain(|inst| discovered.contains(&inst.read().id()));

        self.dirty = true;
        Ok(())
    }

    fn load_group_list(&mut self) {
        let groups_path = Path::new(&self.inst_dir).join("../instgroups.json");
        let content = match std::fs::read_to_string(&groups_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return,
        };
        if let Some(groups) = json.get("groups").and_then(|v| v.as_object()) {
            for (group_name, group_data) in groups {
                if let Some(instances) = group_data.get("instances").and_then(|v| v.as_array()) {
                    let hidden = group_data
                        .get("hidden")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if hidden {
                        self.collapsed_groups.insert(group_name.clone());
                    }
                    for inst_val in instances {
                        if let Some(id) = inst_val.as_str() {
                            self.instance_group_index
                                .insert(id.to_string(), group_name.clone());
                        }
                    }
                }
                self.group_name_cache.insert(group_name.clone());
            }
        }
    }

    fn save_group_list(&self) {
        let groups_path = Path::new(&self.inst_dir).join("../instgroups.json");
        let mut groups = serde_json::Map::new();
        for group_name in &self.group_name_cache {
            let mut group_obj = serde_json::Map::new();
            group_obj.insert(
                "hidden".to_string(),
                serde_json::Value::Bool(self.collapsed_groups.contains(group_name)),
            );
            let mut inst_list = Vec::new();
            for (id, g) in &self.instance_group_index {
                if g == group_name {
                    inst_list.push(serde_json::Value::String(id.clone()));
                }
            }
            group_obj.insert("instances".to_string(), serde_json::Value::Array(inst_list));
            groups.insert(group_name.clone(), serde_json::Value::Object(group_obj));
        }
        let mut root = serde_json::Map::new();
        root.insert(
            "formatVersion".to_string(),
            serde_json::Value::Number(1.into()),
        );
        root.insert("groups".to_string(), serde_json::Value::Object(groups));
        if let Ok(json) = serde_json::to_string_pretty(&root) {
            let _ = std::fs::write(&groups_path, json);
        }
    }

    pub fn save_now(&mut self) {
        if !self.dirty {
            return;
        }
        for inst in &self.instances {
            inst.read().save_now();
        }
        self.save_group_list();
        self.dirty = false;
    }
}
