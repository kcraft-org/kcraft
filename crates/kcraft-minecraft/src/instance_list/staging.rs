use std::path::Path;

use super::list::{InstanceList, TrashHistoryItem};
use super::types::InstanceId;
use crate::instance::Instance;
use crate::instance_task::InstanceTask;

impl InstanceList {
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
        self.instances
            .push(crate::instance_list::types::InstancePtr::new(instance));
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
                .push(crate::instance_list::types::InstancePtr::new(
                    Instance::new(&item.poly_path, ""),
                ));
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
}

pub struct InstanceStaging {
    staging_path: String,
    task: Box<InstanceTask>,
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
