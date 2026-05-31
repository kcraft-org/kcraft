use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VersionRole {
    Name,
    ParentVersion,
    Branch,
    Type,
    Architecture,
    Path,
    Time,
    Recommended,
    Latest,
    VersionId,
    Sort,
}

#[derive(Debug, Clone)]
pub struct VersionEntry {
    pub name: String,
    pub parent_version: String,
    pub branch: String,
    pub version_type: String,
    pub architecture: String,
    pub path: String,
    pub time: String,
    pub recommended: bool,
    pub latest: bool,
    pub version_id: String,
    pub sort_order: i32,
}

pub type FilterMap = HashMap<VersionRole, Box<dyn kcraft_core::filter::Filter + Send + Sync>>;

pub struct VersionFilterModel {
    entries: Vec<VersionEntry>,
    all_entries: Vec<VersionEntry>,
    filters: FilterMap,
    has_recommended: bool,
    has_latest: bool,
    current_version: String,
}

impl VersionFilterModel {
    pub fn new() -> Self {
        VersionFilterModel {
            entries: Vec::new(),
            all_entries: Vec::new(),
            filters: HashMap::new(),
            has_recommended: false,
            has_latest: false,
            current_version: String::new(),
        }
    }

    pub fn set_entries(&mut self, entries: Vec<VersionEntry>) {
        self.all_entries = entries;
        self.apply_filters();
    }

    pub fn entries(&self) -> &[VersionEntry] {
        &self.entries
    }

    pub fn all_entries(&self) -> &[VersionEntry] {
        &self.all_entries
    }

    pub fn apply_filters(&mut self) {
        if self.filters.is_empty() {
            self.entries = self.all_entries.clone();
            return;
        }
        self.entries = self.all_entries.iter().filter(|entry| {
            for (role, filter) in &self.filters {
                let value = match role {
                    VersionRole::Name => &entry.name,
                    VersionRole::ParentVersion => &entry.parent_version,
                    VersionRole::Branch => &entry.branch,
                    VersionRole::Type => &entry.version_type,
                    VersionRole::Architecture => &entry.architecture,
                    VersionRole::Path => &entry.path,
                    VersionRole::Time => &entry.time,
                    _ => continue,
                };
                if !filter.accepts(value) {
                    return false;
                }
            }
            true
        }).cloned().collect();
    }

    pub fn set_filter(&mut self, role: VersionRole, filter: Box<dyn kcraft_core::filter::Filter + Send + Sync>) {
        self.filters.insert(role, filter);
        self.apply_filters();
    }

    pub fn clear_filters(&mut self) {
        self.filters.clear();
        self.entries = self.all_entries.clone();
    }

    pub fn get_recommended(&self) -> Option<&VersionEntry> {
        if !self.has_recommended {
            return self.entries.first();
        }
        self.entries.iter().find(|e| e.recommended)
            .or_else(|| self.entries.first())
    }

    pub fn get_version(&self, version: &str) -> Option<&VersionEntry> {
        self.entries.iter().find(|e| e.name == version)
    }

    pub fn set_current_version(&mut self, version: String) {
        self.current_version = version;
    }

    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    pub fn set_has_recommended(&mut self, v: bool) { self.has_recommended = v; }
    pub fn has_recommended(&self) -> bool { self.has_recommended }
    pub fn set_has_latest(&mut self, v: bool) { self.has_latest = v; }
    pub fn has_latest(&self) -> bool { self.has_latest }
    pub fn filters(&self) -> &FilterMap { &self.filters }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

impl Default for VersionFilterModel {
    fn default() -> Self { Self::new() }
}
