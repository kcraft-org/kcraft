use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use crate::instance::Instance;

pub type InstanceId = String;
pub type GroupId = String;
pub type InstanceLocator = (InstancePtr, usize);

#[derive(Debug, Clone)]
pub struct InstancePtr {
    inner: std::sync::Arc<std::sync::RwLock<Instance>>,
}

impl InstancePtr {
    pub fn new(instance: Instance) -> Self {
        InstancePtr {
            inner: std::sync::Arc::new(std::sync::RwLock::new(instance)),
        }
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, Instance> {
        self.inner.read().unwrap()
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, Instance> {
        self.inner.write().unwrap()
    }
}

#[derive(Debug, Clone)]
struct TrashHistoryItem {
    id: InstanceId,
    poly_path: String,
    trash_path: String,
    group_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupsState {
    NotLoaded,
    Steady,
    Dirty,
}

#[derive(Debug, Clone)]
pub struct InstanceList {
    inst_dir: String,
    instances: Vec<InstancePtr>,
    group_name_cache: HashSet<String>,
    collapsed_groups: HashSet<String>,
    instance_group_index: HashMap<InstanceId, GroupId>,
    trash_history: VecDeque<TrashHistoryItem>,
    dirty: bool,
}

impl InstanceList {
    pub fn new(inst_dir: &str) -> Self {
        InstanceList {
            inst_dir: inst_dir.to_string(),
            instances: Vec::new(),
            group_name_cache: HashSet::new(),
            collapsed_groups: HashSet::new(),
            instance_group_index: HashMap::new(),
            trash_history: VecDeque::new(),
            dirty: false,
        }
    }

    pub fn count(&self) -> usize {
        self.instances.len()
    }

    pub fn at(&self, i: usize) -> Option<InstancePtr> {
        self.instances.get(i).cloned()
    }

    pub fn get_instance_by_id(&self, id: &str) -> Option<InstancePtr> {
        self.instances
            .iter()
            .find(|inst| inst.read().id() == id)
            .cloned()
    }

    pub fn get_instance_index_by_id(&self, id: &str) -> Option<usize> {
        self.instances
            .iter()
            .position(|inst| inst.read().id() == id)
    }

    pub fn get_groups(&self) -> HashSet<String> {
        self.group_name_cache.clone()
    }

    pub fn get_instance_group(&self, id: &InstanceId) -> Option<GroupId> {
        self.instance_group_index.get(id).cloned()
    }

    pub fn set_instance_group(&mut self, id: &InstanceId, name: &GroupId) {
        self.instance_group_index.insert(id.clone(), name.clone());
        self.group_name_cache.insert(name.clone());
        self.dirty = true;
    }

    pub fn delete_group(&mut self, name: &GroupId) {
        self.instance_group_index.retain(|_, v| v != name);
        self.collapsed_groups.remove(name);
        self.group_name_cache.remove(name);
        self.dirty = true;
    }

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

    fn load_instance(&self, id: &InstanceId) -> Option<Instance> {
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

    pub fn get_staged_instance_path(&self) -> String {
        let temp_dir = Path::new(&self.inst_dir).join(".LAUNCHER_TEMP");
        let _ = std::fs::create_dir_all(&temp_dir);
        loop {
            let dir_name = format!("staging_{}", uuid::Uuid::new_v4());
            let path = temp_dir.join(&dir_name);
            if !path.exists() {
                let _ = std::fs::create_dir_all(&path);
                return path.to_string_lossy().to_string();
            }
        }
    }

    pub fn commit_staged_instance(
        &mut self,
        key_path: &str,
        name: &str,
        group_name: &str,
        should_override: bool,
    ) -> bool {
        let key = Path::new(key_path);
        let id = match key.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return false,
        };
        let final_path = Path::new(&self.inst_dir).join(&id);
        if final_path.exists() {
            if should_override {
                let _ = std::fs::remove_dir_all(&final_path);
            } else {
                let _ = std::fs::remove_dir_all(key_path);
                return false;
            }
        }
        if std::fs::rename(key_path, &final_path).is_err() {
            return false;
        }
        let mut instance = match self.load_instance(&id) {
            Some(i) => i,
            None => return false,
        };
        instance.name = name.to_string();
        if !group_name.is_empty() {
            self.instance_group_index
                .insert(id.clone(), group_name.to_string());
            self.group_name_cache.insert(group_name.to_string());
        }
        self.instances.push(InstancePtr::new(instance));
        self.dirty = true;
        true
    }

    pub fn destroy_staging_path(&self, key_path: &str) -> bool {
        let path = Path::new(key_path);
        if path.exists() {
            std::fs::remove_dir_all(key_path).is_ok()
        } else {
            true
        }
    }

    pub fn trash_instance(&mut self, id: &InstanceId) -> bool {
        let pos = self
            .instances
            .iter()
            .position(|inst| inst.read().id() == *id);
        let inst = match pos {
            Some(p) => self.instances.remove(p),
            None => return false,
        };
        let inst_path = inst.read().instance_root.clone();
        let trash_dir = Path::new(&self.inst_dir).join(".LAUNCHER_TRASH");
        let _ = std::fs::create_dir_all(&trash_dir);
        let trash_name = format!("{}_{}", id, chrono::Utc::now().timestamp());
        let trash_path = trash_dir.join(&trash_name);
        if std::fs::rename(&inst_path, &trash_path).is_ok() {
            let group = self.instance_group_index.remove(id);
            self.trash_history.push_back(TrashHistoryItem {
                id: id.clone(),
                poly_path: inst_path,
                trash_path: trash_path.to_string_lossy().to_string(),
                group_name: group.unwrap_or_default(),
            });
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn trashed_something(&self) -> bool {
        !self.trash_history.is_empty()
    }

    pub fn undo_trash_instance(&mut self) -> bool {
        let item = match self.trash_history.pop_back() {
            Some(i) => i,
            None => return false,
        };
        if std::fs::rename(&item.trash_path, &item.poly_path).is_ok() {
            if !item.group_name.is_empty() {
                self.instance_group_index
                    .insert(item.id.clone(), item.group_name.clone());
            }
            self.instances
                .push(InstancePtr::new(Instance::new(&item.poly_path, "")));
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn delete_instance(&mut self, id: &InstanceId) {
        self.instances.retain(|inst| inst.read().id() != *id);
        let _ = std::fs::remove_dir_all(Path::new(&self.inst_dir).join(id));
        self.instance_group_index.remove(id);
        self.dirty = true;
    }

    pub fn wrap_instance_task(&self, _task: crate::instance_task::InstanceTask) -> InstanceStaging {
        let staging_path = self.get_staged_instance_path();
        InstanceStaging {
            staging_path,
            task: Box::new(_task),
        }
    }

    pub fn is_group_collapsed(&self, group_name: &str) -> bool {
        self.collapsed_groups.contains(group_name)
    }

    pub fn set_group_collapsed(&mut self, group_name: &str, collapsed: bool) {
        if collapsed {
            self.collapsed_groups.insert(group_name.to_string());
        } else {
            self.collapsed_groups.remove(group_name);
        }
        self.dirty = true;
    }
}

pub struct InstanceStaging {
    staging_path: String,
    task: Box<crate::instance_task::InstanceTask>,
}

impl InstanceStaging {
    pub fn staging_path(&self) -> &str {
        &self.staging_path
    }

    pub fn execute(&mut self) -> Result<(), String> {
        self.task.set_staging_path(&self.staging_path);
        self.task.execute()
    }
}
