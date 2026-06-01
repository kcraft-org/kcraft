use std::collections::{HashMap, HashSet};

use super::list::InstanceList;
use super::types::{GroupId, InstanceId, InstancePtr};

impl InstanceList {
    pub fn new(inst_dir: &str) -> Self {
        InstanceList {
            inst_dir: inst_dir.to_string(),
            instances: Vec::new(),
            group_name_cache: HashSet::new(),
            collapsed_groups: HashSet::new(),
            instance_group_index: HashMap::new(),
            trash_history: std::collections::VecDeque::new(),
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
