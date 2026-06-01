use std::collections::{HashMap, HashSet, VecDeque};

use super::types::{GroupId, InstanceId, InstancePtr};

#[derive(Debug, Clone)]
pub(super) struct TrashHistoryItem {
    pub(super) id: InstanceId,
    pub(super) poly_path: String,
    pub(super) trash_path: String,
    pub(super) group_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupsState {
    NotLoaded,
    Steady,
    Dirty,
}

#[derive(Debug, Clone)]
pub struct InstanceList {
    pub(super) inst_dir: String,
    pub(super) instances: Vec<InstancePtr>,
    pub(super) group_name_cache: HashSet<String>,
    pub(super) collapsed_groups: HashSet<String>,
    pub(super) instance_group_index: HashMap<InstanceId, GroupId>,
    pub(super) trash_history: VecDeque<TrashHistoryItem>,
    pub(super) dirty: bool,
}
