use std::path::Path;

use super::task::InstanceTask;

pub struct InstanceCreationTask {
    base: InstanceTask,
    files_to_remove: Vec<String>,
    error_message: String,
}

impl InstanceCreationTask {
    pub fn new(name: &str, version: &str) -> Self {
        InstanceCreationTask {
            base: InstanceTask::new(name, version),
            files_to_remove: Vec::new(),
            error_message: String::new(),
        }
    }

    pub fn task(&self) -> &InstanceTask {
        &self.base
    }
    pub fn task_mut(&mut self) -> &mut InstanceTask {
        &mut self.base
    }
    pub fn get_error(&self) -> &str {
        &self.error_message
    }

    pub fn update_instance(&mut self) -> bool {
        false
    }

    pub fn create_instance(&mut self) -> bool {
        false
    }

    pub fn execute(&mut self) -> Result<(), String> {
        if self.update_instance() {
            return Ok(());
        }
        if self.create_instance() {
            return Ok(());
        }
        if self.base.should_override() {
            for path in &self.files_to_remove {
                let _ = std::fs::remove_file(path);
            }
            return Ok(());
        }
        Err(self.error_message.clone())
    }
}

pub struct VanillaInstanceCreationTask {
    base: InstanceCreationTask,
    version: String,
    _using_loader: bool,
    loader: String,
    loader_version: String,
}

impl VanillaInstanceCreationTask {
    pub fn new(version: &str) -> Self {
        VanillaInstanceCreationTask {
            base: InstanceCreationTask::new(version, version),
            version: version.to_string(),
            _using_loader: false,
            loader: String::new(),
            loader_version: String::new(),
        }
    }

    pub fn with_loader(version: &str, loader: &str, loader_version: &str) -> Self {
        VanillaInstanceCreationTask {
            base: InstanceCreationTask::new(version, version),
            version: version.to_string(),
            _using_loader: true,
            loader: loader.to_string(),
            loader_version: loader_version.to_string(),
        }
    }

    pub fn create_instance(&mut self) -> bool {
        let staging = self.base.base.staging_path().to_string();
        if staging.is_empty() {
            self.base.error_message = "No staging path set".to_string();
            return false;
        }

        let cfg_path = Path::new(&staging).join("instance.cfg");
        let mut ini = kcraft_core::INIFile::new();
        ini.set("InstanceType", "OneSix");
        ini.set("name", &self.base.base.name());
        ini.set("iconKey", self.base.base.icon());
        ini.set("notes", "");

        if !self.base.base.group().is_empty() {
            ini.set("Group", self.base.base.group());
        }

        if ini.save_file(&cfg_path).is_err() {
            self.base.error_message = "Failed to write instance.cfg".to_string();
            return false;
        }

        let patches_path = Path::new(&staging).join("patches");
        let _ = std::fs::create_dir_all(&patches_path);

        let components = self.build_components_json();
        let mmc_pack_path = Path::new(&staging).join("mmc-pack.json");
        if let Ok(json) = serde_json::to_string_pretty(&components) {
            let _ = std::fs::write(&mmc_pack_path, json);
        }

        true
    }

    fn build_components_json(&self) -> Vec<serde_json::Value> {
        let mut components = Vec::new();

        components.push(serde_json::json!({
            "uid": "net.minecraft",
            "version": self.version,
            "important": true
        }));

        if self._using_loader {
            let mut loader_comp = serde_json::json!({
                "uid": self.loader,
                "version": self.loader_version,
                "important": true
            });
            if self.loader.contains("forge") || self.loader.contains("neoforged") {
                if let serde_json::Value::Object(ref mut obj) = loader_comp {
                    obj.insert("order".to_string(), serde_json::json!(1));
                }
            }
            components.push(loader_comp);
        }

        components
    }
}
