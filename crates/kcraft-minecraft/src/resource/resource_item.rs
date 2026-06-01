use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::enums::{EnableAction, ResourceType, SortType};

#[derive(Debug, Clone)]
pub struct Resource {
    path: PathBuf,
    changed_date_time: i64,
    internal_id: String,
    name: String,
    resource_type: ResourceType,
    enabled: bool,
    is_resolving: bool,
    is_resolved: bool,
    resolution_ticket: u32,
}

impl Resource {
    pub fn new(path: &Path) -> Self {
        let metadata = path.metadata().ok();
        let changed = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let internal_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let resource_type = if path.is_dir() {
            ResourceType::Folder
        } else if path
            .extension()
            .is_some_and(|e| e == "zip" || e == "jar" || e == "disabled")
        {
            ResourceType::ZipFile
        } else if path.extension().is_some_and(|e| e == "litemod") {
            ResourceType::Litemod
        } else {
            ResourceType::SingleFile
        };

        Resource {
            path: path.to_path_buf(),
            changed_date_time: changed,
            internal_id: internal_id.clone(),
            name: internal_id,
            resource_type,
            enabled: true,
            is_resolving: false,
            is_resolved: false,
            resolution_ticket: 0,
        }
    }

    pub fn with_name(path: &Path, name: &str) -> Self {
        let mut r = Resource::new(path);
        r.name = name.to_string();
        r
    }

    pub fn fileinfo(&self) -> &Path {
        &self.path
    }
    pub fn date_time_changed(&self) -> i64 {
        self.changed_date_time
    }
    pub fn internal_id(&self) -> &str {
        &self.internal_id
    }
    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
    pub fn valid(&self) -> bool {
        !matches!(self.resource_type, ResourceType::Unknown)
    }
    pub fn should_resolve(&self) -> bool {
        !self.is_resolving && !self.is_resolved
    }
    pub fn is_resolving(&self) -> bool {
        self.is_resolving
    }
    pub fn is_resolved(&self) -> bool {
        self.is_resolved
    }
    pub fn resolution_ticket(&self) -> u32 {
        self.resolution_ticket
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn enable(&mut self, action: EnableAction) -> bool {
        match action {
            EnableAction::Enable => {
                let old = self.enabled;
                self.enabled = true;
                old != self.enabled
            }
            EnableAction::Disable => {
                let old = self.enabled;
                self.enabled = false;
                old != self.enabled
            }
            EnableAction::Toggle => {
                self.enabled = !self.enabled;
                true
            }
        }
    }

    pub fn set_resolving(&mut self, resolving: bool, ticket: u32) {
        self.is_resolving = resolving;
        self.resolution_ticket = ticket;
    }

    pub fn set_resolved(&mut self, resolved: bool) {
        self.is_resolved = resolved;
    }

    pub fn destroy(&mut self) -> bool {
        if self.path.exists() {
            let result = if self.path.is_dir() {
                std::fs::remove_dir_all(&self.path)
            } else {
                std::fs::remove_file(&self.path)
            };
            result.is_ok()
        } else {
            false
        }
    }

    pub fn compare(&self, other: &Resource, sort_type: SortType) -> std::cmp::Ordering {
        match sort_type {
            SortType::Name => self.name.cmp(&other.name),
            SortType::Date => self.changed_date_time.cmp(&other.changed_date_time),
            SortType::Enabled => self.enabled.cmp(&other.enabled),
        }
    }

    pub fn apply_filter(&self, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        let lower = filter.to_lowercase();
        self.name.to_lowercase().contains(&lower)
            || self.internal_id.to_lowercase().contains(&lower)
    }
}
