use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: i64,
}

impl AssetObject {
    pub fn get_rel_path(&self) -> String {
        format!("{}/{}", &self.hash[..2], self.hash)
    }

    pub fn get_url(&self) -> String {
        let base = std::env::var("KCRAFT_RESOURCE_BASE")
            .unwrap_or_else(|_| "https://resources.download.minecraft.net/".to_string());
        format!("{} {}", base, self.get_rel_path())
    }

    pub fn get_local_path(&self) -> String {
        format!("assets/objects/{}", self.get_rel_path())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsIndex {
    pub id: String,
    pub objects: HashMap<String, AssetObject>,
    pub is_virtual: bool,
    pub map_to_resources: bool,
}

impl AssetsIndex {
    pub fn is_legacy(&self) -> bool {
        self.id == "legacy" || self.id == "pre-1.6"
    }
}

pub mod assets_utils {
    use std::collections::{HashMap, HashSet};
    use std::path::Path;

    use super::{AssetObject, AssetsIndex};

    pub fn load_assets_index_json(id: &str, path: &Path) -> Option<AssetsIndex> {
        let content = std::fs::read_to_string(path).ok()?;
        let root: serde_json::Value = serde_json::from_str(&content).ok()?;

        let is_virtual = root
            .get("virtual")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let map_to_resources = root
            .get("map_to_resources")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut objects = HashMap::new();
        if let Some(objects_val) = root.get("objects").and_then(|v| v.as_object()) {
            for (key, val) in objects_val {
                if let Some(obj) = val.as_object() {
                    let hash = obj
                        .get("hash")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let size = obj.get("size").and_then(|v| v.as_i64()).unwrap_or(0);
                    objects.insert(key.clone(), AssetObject { hash, size });
                }
            }
        }

        Some(AssetsIndex {
            id: id.to_string(),
            objects,
            is_virtual,
            map_to_resources,
        })
    }

    fn collect_paths_from_dir(dir_path: &Path) -> HashSet<String> {
        let mut out = HashSet::new();
        if !dir_path.exists() {
            return out;
        }
        for entry in walkdir::WalkDir::new(dir_path).into_iter().flatten() {
            if entry.file_type().is_file() {
                out.insert(entry.path().to_string_lossy().to_string());
            }
        }
        out
    }

    pub fn get_assets_dir(assets_id: &str, _resources_folder: &str) -> String {
        let assets_dir = Path::new("assets");
        let virtual_root = assets_dir.join("virtual").join(assets_id);

        let index_path = assets_dir
            .join("indexes")
            .join(format!("{}.json", assets_id));
        if !index_path.exists() {
            return virtual_root.to_string_lossy().to_string();
        }

        if let Some(index) = load_assets_index_json(assets_id, &index_path) {
            if index.is_virtual {
                return virtual_root.to_string_lossy().to_string();
            }
        }
        virtual_root.to_string_lossy().to_string()
    }

    pub fn reconstruct_assets(assets_id: &str, resources_folder: &str) -> bool {
        let assets_dir = Path::new("assets");
        let index_dir = assets_dir.join("indexes");
        let object_dir = assets_dir.join("objects");
        let virtual_dir = assets_dir.join("virtual");
        let virtual_root = virtual_dir.join(assets_id);

        let index_path = index_dir.join(format!("{}.json", assets_id));
        if !index_path.exists() {
            tracing::error!("No assets index file {:?}", index_path);
            return false;
        }

        let index = match load_assets_index_json(assets_id, &index_path) {
            Some(i) => i,
            None => {
                tracing::error!("Failed to load asset index file {:?}", index_path);
                return false;
            }
        };

        let target_path = if index.is_virtual {
            Some(virtual_root.to_string_lossy().to_string())
        } else if index.map_to_resources {
            Some(resources_folder.to_string())
        } else {
            None
        };

        if let Some(ref target) = target_path {
            let present_files = collect_paths_from_dir(Path::new(target));
            for (map_path, asset_object) in &index.objects {
                let target_file_path = Path::new(target).join(map_path);
                let tlk = &asset_object.hash[..2];
                let original_path = object_dir.join(tlk).join(&asset_object.hash);

                if !original_path.exists() {
                    continue;
                }

                let target_str = target_file_path.to_string_lossy().to_string();
                if present_files.contains(&target_str) {
                    continue; // already exists
                }

                if let Some(parent) = target_file_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                match std::fs::copy(&original_path, &target_file_path) {
                    Ok(_) => {
                        tracing::debug!("Copied {:?} to {:?}", original_path, target_file_path)
                    }
                    Err(e) => tracing::warn!(
                        "Failed to copy {:?} to {:?}: {}",
                        original_path,
                        target_file_path,
                        e
                    ),
                }
            }
        }

        true
    }
}
