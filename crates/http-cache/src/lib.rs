use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaEntry {
    pub base_id: String,
    pub base_path: String,
    pub relative_path: String,
    #[serde(default)]
    pub md5sum: String,
    #[serde(default)]
    pub etag: String,
    pub local_changed_timestamp: i64,
    #[serde(default)]
    pub remote_changed_timestamp: String,
    pub current_age: i64,
    pub max_age: i64,
    #[serde(default)]
    pub eternal: bool,
    #[serde(default = "default_stale")]
    pub stale: bool,
}

fn default_stale() -> bool {
    true
}

impl MetaEntry {
    pub fn full_path(&self) -> PathBuf {
        PathBuf::from(&self.base_path).join(&self.relative_path)
    }

    pub fn is_expired(&self, offset: i64) -> bool {
        if self.eternal {
            return false;
        }
        self.current_age >= self.max_age - offset
    }

    pub fn make_eternal(&mut self, eternal: bool) {
        self.eternal = eternal;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheIndex {
    version: String,
    entries: Vec<SerializedEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializedEntry {
    base: String,
    path: String,
    #[serde(default)]
    md5sum: String,
    #[serde(default)]
    etag: String,
    last_changed_timestamp: i64,
    #[serde(default)]
    remote_changed_timestamp: String,
    #[serde(default = "default_false")]
    eternal: bool,
    current_age: i64,
    max_age: i64,
}

fn default_false() -> bool {
    false
}

#[derive(Debug, Clone)]
struct CacheBase {
    base_path: String,
    entries: HashMap<String, MetaEntry>,
}

pub type MetaEntryPtr = Arc<RwLock<MetaEntry>>;

#[derive(Debug)]
pub struct HttpMetaCache {
    bases: RwLock<HashMap<String, CacheBase>>,
    index_path: PathBuf,
    dirty: RwLock<bool>,
}

impl HttpMetaCache {
    pub fn new(index_path: PathBuf) -> Self {
        let cache = HttpMetaCache {
            bases: RwLock::new(HashMap::new()),
            index_path,
            dirty: RwLock::new(false),
        };
        let _ = cache.load();
        cache
    }

    pub fn add_base(&self, base_id: &str, base_path: impl AsRef<Path>) {
        let base_path_str = base_path.as_ref().to_string_lossy().to_string();
        let mut bases = self.bases.write().unwrap();
        if !bases.contains_key(base_id) {
            let _ = std::fs::create_dir_all(&base_path_str);
            bases.insert(
                base_id.to_string(),
                CacheBase {
                    base_path: base_path_str,
                    entries: HashMap::new(),
                },
            );
        }
    }

    pub fn get_entry(&self, base_id: &str, path: &str) -> Option<MetaEntryPtr> {
        let bases = self.bases.read().unwrap();
        let base = bases.get(base_id)?;
        base.entries
            .get(path)
            .map(|e| Arc::new(RwLock::new(e.clone())))
    }

    pub fn resolve_entry(
        &self,
        base_id: &str,
        path: &str,
        expected_etag: Option<&str>,
    ) -> MetaEntryPtr {
        let bases = self.bases.read().unwrap();
        let entry = bases.get(base_id).and_then(|base| base.entries.get(path));

        match entry {
            Some(entry) => {
                let full_path = PathBuf::from(&entry.base_path).join(&entry.relative_path);
                if !full_path.exists() || !full_path.is_file() {
                    drop(bases);
                    return self.stale_entry(base_id, path);
                }
                if let Some(etag) = expected_etag {
                    if entry.etag != etag {
                        drop(bases);
                        return self.stale_entry(base_id, path);
                    }
                }
                if let Ok(metadata) = std::fs::metadata(&full_path) {
                    let modified = metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    if modified != entry.local_changed_timestamp {
                        if let Ok(data) = std::fs::read(&full_path) {
                            use md5::Digest;
                            let hash = md5::Md5::digest(&data);
                            let md5_hex = hash
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>();
                            if md5_hex != entry.md5sum {
                                drop(bases);
                                return self.stale_entry(base_id, path);
                            }
                        }
                    }
                }
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let file_age = if let Ok(metadata) = std::fs::metadata(&full_path) {
                    metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(current_time)
                } else {
                    current_time
                };
                let age = current_time - file_age;
                if !entry.eternal && age >= entry.max_age {
                    drop(bases);
                    return self.stale_entry(base_id, path);
                }
                let mut cloned = entry.clone();
                cloned.current_age = age;
                cloned.stale = false;
                Arc::new(RwLock::new(cloned))
            }
            None => {
                drop(bases);
                self.stale_entry(base_id, path)
            }
        }
    }

    pub fn stale_entry(&self, base_id: &str, path: &str) -> MetaEntryPtr {
        let bases = self.bases.read().unwrap();
        let base_path = bases
            .get(base_id)
            .map(|b| b.base_path.clone())
            .unwrap_or_default();
        drop(bases);
        Arc::new(RwLock::new(MetaEntry {
            base_id: base_id.to_string(),
            base_path,
            relative_path: path.to_string(),
            md5sum: String::new(),
            etag: String::new(),
            local_changed_timestamp: 0,
            remote_changed_timestamp: String::new(),
            current_age: 0,
            max_age: 0,
            eternal: false,
            stale: true,
        }))
    }

    pub fn update_entry(&self, entry: &MetaEntryPtr) {
        let entry_guard = entry.read().unwrap();
        if entry_guard.stale {
            return;
        }
        let mut bases = self.bases.write().unwrap();
        if let Some(base) = bases.get_mut(&entry_guard.base_id) {
            base.entries
                .insert(entry_guard.relative_path.clone(), entry_guard.clone());
        }
        *self.dirty.write().unwrap() = true;
    }

    pub fn evict_entry(&self, entry: &MetaEntryPtr) {
        let entry_guard = entry.read().unwrap();
        let mut bases = self.bases.write().unwrap();
        if let Some(base) = bases.get_mut(&entry_guard.base_id) {
            base.entries.remove(&entry_guard.relative_path);
        }
        *self.dirty.write().unwrap() = true;
    }

    pub fn load(&self) -> Result<(), String> {
        if !self.index_path.exists() {
            return Ok(());
        }
        let data = std::fs::read_to_string(&self.index_path)
            .map_err(|e| format!("Failed to read cache index: {}", e))?;
        let index: CacheIndex = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse cache index: {}", e))?;
        let mut bases = self.bases.write().unwrap();
        for se in index.entries {
            let base_id = se.base;
            let entry = MetaEntry {
                base_id: base_id.clone(),
                base_path: bases
                    .get(&base_id)
                    .map(|b| b.base_path.clone())
                    .unwrap_or_default(),
                relative_path: se.path,
                md5sum: se.md5sum,
                etag: se.etag,
                local_changed_timestamp: se.last_changed_timestamp,
                remote_changed_timestamp: se.remote_changed_timestamp,
                current_age: se.current_age,
                max_age: se.max_age,
                eternal: se.eternal,
                stale: false,
            };
            if let Some(base) = bases.get_mut(&base_id) {
                base.entries.insert(entry.relative_path.clone(), entry);
            }
        }
        Ok(())
    }

    pub fn save_now(&self) -> Result<(), String> {
        let bases = self.bases.read().unwrap();
        let entries: Vec<SerializedEntry> = bases
            .values()
            .flat_map(|base| {
                base.entries.values().map(|entry| SerializedEntry {
                    base: entry.base_id.clone(),
                    path: entry.relative_path.clone(),
                    md5sum: entry.md5sum.clone(),
                    etag: entry.etag.clone(),
                    last_changed_timestamp: entry.local_changed_timestamp,
                    remote_changed_timestamp: entry.remote_changed_timestamp.clone(),
                    eternal: entry.eternal,
                    current_age: entry.current_age,
                    max_age: entry.max_age,
                })
            })
            .collect();
        drop(bases);
        let index = CacheIndex {
            version: "1".to_string(),
            entries,
        };
        let data = serde_json::to_string_pretty(&index)
            .map_err(|e| format!("Failed to serialize cache index: {}", e))?;
        if let Some(parent) = self.index_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        fs::write(&self.index_path, data.as_bytes())
            .map_err(|e| format!("Failed to write cache index: {}", e))?;
        *self.dirty.write().unwrap() = false;
        Ok(())
    }

    pub fn save_eventually(&self) {
        let dirty = *self.dirty.read().unwrap();
        if dirty {
            let _ = self.save_now();
        }
    }
}
