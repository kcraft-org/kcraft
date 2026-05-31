use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::instance::MinecraftInstance;
use crate::component::PackProfile;
use crate::OpSys;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMode {
    Launch,
    Resolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetMode {
    Offline,
    Online,
}

pub struct ComponentUpdate {
    mode: UpdateMode,
    abort_flag: Arc<AtomicBool>,
}

impl ComponentUpdate {
    pub fn new(mode: UpdateMode, _netmode: NetMode) -> Self {
        ComponentUpdate {
            mode,
            abort_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn execute(&self, components: &mut PackProfile) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for index in 0..components.components().len() {
            if self.abort_flag.load(Ordering::SeqCst) {
                return Err(vec!["Update aborted".to_string()]);
            }

            let component = &components.components()[index];
            if component.disabled {
                continue;
            }

            if !component.loaded {
                let result = self.load_component(component);
                if let Err(e) = result {
                    errors.push(format!("Failed to load component '{}': {}", component.uid, e));
                    if self.mode == UpdateMode::Launch {
                        return Err(errors);
                    }
                }
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }

    fn load_component(&self, _component: &crate::Component) -> Result<(), String> {
        // In offline mode, try local files
        // In online mode, fetch from remote
        Ok(())
    }

    pub fn abort(&self) {
        self.abort_flag.store(true, Ordering::SeqCst);
    }

    pub fn can_abort(&self) -> bool {
        true
    }
}

pub struct MinecraftUpdate {
    instance: MinecraftInstance,
    abort_flag: Arc<AtomicBool>,
}

impl MinecraftUpdate {
    pub fn new(instance: MinecraftInstance) -> Self {
        MinecraftUpdate {
            instance,
            abort_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn execute(&mut self) -> Result<(), String> {
        self.run_folders_task()?;
        self.run_libraries_task()?;
        self.run_assets_task()?;
        self.run_fml_libraries_task()?;
        Ok(())
    }

    fn run_folders_task(&self) -> Result<(), String> {
        let dirs = [
            self.instance.game_root(),
            self.instance.bin_root(),
            self.instance.get_native_path(),
            self.instance.mods_root(),
            self.instance.core_mods_dir(),
            self.instance.resource_packs_dir(),
            self.instance.jar_mods_dir(),
            Path::new(&self.instance.game_root()).join("libraries").to_string_lossy().to_string(),
            Path::new(&self.instance.game_root()).join("versions").to_string_lossy().to_string(),
            Path::new(&self.instance.game_root()).join("assets").join("objects").to_string_lossy().to_string(),
            Path::new(&self.instance.game_root()).join("assets").join("indexes").to_string_lossy().to_string(),
        ];

        for dir in &dirs {
            if self.abort_flag.load(Ordering::SeqCst) {
                return Err("Update aborted".to_string());
            }
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
        }

        Ok(())
    }

    fn run_libraries_task(&mut self) -> Result<(), String> {
        let profile = match self.instance.components.get_profile() {
            Some(p) => p,
            None => return Err("No profile loaded".to_string()),
        };

        let mut libraries_to_download: Vec<crate::Library> = Vec::new();

        for lib in &profile.libraries {
            if !lib.is_active(&OpSys::current()) {
                continue;
            }
            libraries_to_download.push(lib.clone());
        }

        for lib in &profile.native_libraries {
            if !lib.is_active(&OpSys::current()) {
                continue;
            }
            libraries_to_download.push(lib.clone());
        }

        if let Some(ref main_jar) = profile.main_jar {
            libraries_to_download.push(main_jar.clone());
        }

        for lib in &libraries_to_download {
            if self.abort_flag.load(Ordering::SeqCst) {
                return Err("Update aborted".to_string());
            }

            let lib_path = lib.local_path();
            let path = Path::new(&self.instance.game_root()).join(&lib_path);

            if path.exists() {
                continue;
            }

            if let Some(dl_info) = lib.download_info() {
                if let Some(ref url) = dl_info.url {
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| format!("Failed to create directory: {}", e))?;
                    }

                    let result = download_file(url, &path);
                    if let Err(e) = result {
                        return Err(format!("Failed to download {}: {}", lib.name, e));
                    }
                }
            }
        }

        Ok(())
    }

    fn run_assets_task(&self) -> Result<(), String> {
        let profile = match self.instance.components.get_profile() {
            Some(p) => p,
            None => return Err("No profile loaded".to_string()),
        };

        let assets_dir = Path::new(&self.instance.game_root()).join("assets");
        let indexes_dir = assets_dir.join("indexes");
        let objects_dir = assets_dir.join("objects");

        let index_path = indexes_dir.join(format!("{}.json", profile.assets));

        if !index_path.exists() {
            // Download the asset index
            let index_url = format!(
                "https://piston-meta.mojang.com/mc/assets/{}",
                profile.assets
            );

            std::fs::create_dir_all(&indexes_dir)
                .map_err(|e| format!("Failed to create indexes dir: {}", e))?;

            download_file(&index_url, &index_path)
                .map_err(|e| format!("Failed to download asset index: {}", e))?;
        }

        let index_content = std::fs::read_to_string(&index_path)
            .map_err(|e| format!("Failed to read asset index: {}", e))?;

        let index_json: serde_json::Value = serde_json::from_str(&index_content)
            .map_err(|e| format!("Failed to parse asset index: {}", e))?;

        if let Some(objects) = index_json.get("objects").and_then(|v| v.as_object()) {
            for (_key, obj) in objects {
                if self.abort_flag.load(Ordering::SeqCst) {
                    return Err("Update aborted".to_string());
                }

                let hash = obj.get("hash").and_then(|v| v.as_str()).unwrap_or("");
                if hash.is_empty() {
                    continue;
                }

                let prefix = &hash[..2];
                let obj_path = objects_dir.join(prefix).join(hash);

                if obj_path.exists() {
                    continue;
                }

                let url = format!("https://resources.download.minecraft.net/{}/{}", prefix, hash);

                std::fs::create_dir_all(obj_path.parent().unwrap())
                    .map_err(|e| format!("Failed to create dir: {}", e))?;

                download_file(&url, &obj_path)
                    .map_err(|e| format!("Failed to download asset {}: {}", hash, e))?;
            }
        }

        Ok(())
    }

    fn run_fml_libraries_task(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn abort(&self) {
        self.abort_flag.store(true, Ordering::SeqCst);
    }

    pub fn can_abort(&self) -> bool {
        true
    }
}

fn download_file(url: &str, path: &Path) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("HTTP error: {}", e))?
        .error_for_status()
        .map_err(|e| format!("HTTP status error: {}", e))?;

    let mut file = std::fs::File::create(path)
        .map_err(|e| format!("File error: {}", e))?;

    let content = response.bytes()
        .map_err(|e| format!("Read error: {}", e))?;

    std::io::copy(&mut content.as_ref(), &mut file)
        .map_err(|e| format!("IO error: {}", e))?;

    Ok(())
}
