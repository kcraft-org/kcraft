use std::path::Path;

use crate::instance::Instance;

#[derive(Debug, Clone)]
pub struct InstanceName {
    original_name: String,
    original_version: String,
    modified_name: Option<String>,
}

impl InstanceName {
    pub fn new(name: &str, version: &str) -> Self {
        InstanceName {
            original_name: name.to_string(),
            original_version: version.to_string(),
            modified_name: None,
        }
    }

    pub fn modified_name(&self) -> String {
        self.modified_name
            .clone()
            .unwrap_or_else(|| self.original_name.clone())
    }

    pub fn original_name(&self) -> &str {
        &self.original_name
    }

    pub fn name(&self) -> String {
        match &self.modified_name {
            Some(m) => format!("{} {}", m, self.original_version),
            None => format!("{} {}", self.original_name, self.original_version),
        }
    }

    pub fn version(&self) -> &str {
        &self.original_version
    }

    pub fn set_name(&mut self, name: &str) {
        self.modified_name = Some(name.to_string());
    }
}

#[derive(Debug, Clone)]
pub struct InstanceTask {
    staging_path: String,
    inst_icon: String,
    inst_group: String,
    instance_name: InstanceName,
    override_existing: bool,
}

impl InstanceTask {
    pub fn new(name: &str, version: &str) -> Self {
        InstanceTask {
            staging_path: String::new(),
            inst_icon: "default".to_string(),
            inst_group: String::new(),
            instance_name: InstanceName::new(name, version),
            override_existing: false,
        }
    }

    pub fn set_staging_path(&mut self, path: &str) {
        self.staging_path = path.to_string();
    }
    pub fn staging_path(&self) -> &str {
        &self.staging_path
    }
    pub fn set_icon(&mut self, icon: &str) {
        self.inst_icon = icon.to_string();
    }
    pub fn icon(&self) -> &str {
        &self.inst_icon
    }
    pub fn set_group(&mut self, group: &str) {
        self.inst_group = group.to_string();
    }
    pub fn group(&self) -> &str {
        &self.inst_group
    }
    pub fn should_override(&self) -> bool {
        self.override_existing
    }
    pub fn set_override(&mut self, override_: bool) {
        self.override_existing = override_;
    }
    pub fn name(&self) -> String {
        self.instance_name.modified_name()
    }
    pub fn set_instance_name(&mut self, name: &str) {
        self.instance_name.set_name(name);
    }

    pub fn execute(&mut self) -> Result<(), String> {
        Err("Not implemented".to_string())
    }
}

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

#[derive(Debug, Clone)]
pub struct InstanceCopyTask {
    base: InstanceTask,
    instance_root: String,
    matcher: Option<String>,
    keep_playtime: bool,
}

impl InstanceCopyTask {
    pub fn new(instance_root: &str, copy_saves: bool, keep_playtime: bool) -> Self {
        let matcher = if !copy_saves {
            Some("[.]?minecraft/saves".to_string())
        } else {
            None
        };
        InstanceCopyTask {
            base: InstanceTask::new("", ""),
            instance_root: instance_root.to_string(),
            matcher,
            keep_playtime,
        }
    }

    pub fn execute(&mut self) -> Result<(), String> {
        let src = Path::new(&self.instance_root);
        let dst = Path::new(self.base.staging_path());

        copy_dir(src, dst, self.matcher.as_deref())?;

        let mut instance = Instance::new(dst.to_string_lossy().as_ref(), "");
        instance.load_specific_settings();

        self.base.set_instance_name(&instance.name);
        self.base.set_icon(&instance.icon_key);

        if !self.keep_playtime {
            instance.total_time_played = 0;
            instance.last_launch_time = 0;
        }

        instance.save_now();
        Ok(())
    }
}

pub struct InstanceImportTask {
    base: InstanceTask,
    source_url: String,
    archive_path: String,
}

impl InstanceImportTask {
    pub fn new(source_url: &str) -> Self {
        InstanceImportTask {
            base: InstanceTask::new("", ""),
            source_url: source_url.to_string(),
            archive_path: String::new(),
        }
    }

    pub fn execute(&mut self) -> Result<(), String> {
        if self.source_url.starts_with("file://") || self.source_url.starts_with('/') {
            let path = self
                .source_url
                .strip_prefix("file://")
                .unwrap_or(&self.source_url);
            self.archive_path = path.to_string();
            self.process_zip_pack()
        } else {
            Err("Remote downloads not implemented yet".to_string())
        }
    }

    fn process_zip_pack(&mut self) -> Result<(), String> {
        let path = Path::new(&self.archive_path);
        let file = std::fs::File::open(path).map_err(|e| format!("Cannot open archive: {}", e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

        let modpack_type = self.detect_pack_type(&mut archive);

        match modpack_type {
            ModpackType::MultiMC => self.process_multimc(&mut archive),
            ModpackType::Technic => self.process_technic(&mut archive),
            ModpackType::Flame => self.process_flame(&mut archive),
            ModpackType::Modrinth => self.process_modrinth(&mut archive),
            ModpackType::Unknown => Err("Unknown modpack type".to_string()),
        }
    }

    fn detect_pack_type(&self, archive: &mut zip::ZipArchive<std::fs::File>) -> ModpackType {
        let len = archive.len();
        let mut names = Vec::new();
        for i in 0..len {
            if let Ok(entry) = archive.by_index(i) {
                names.push(entry.name().to_string());
            }
        }

        for name in &names {
            if name == "modrinth.index.json" || name.ends_with("/modrinth.index.json") {
                return ModpackType::Modrinth;
            }
            if name == "bin/modpack.jar" || name == "bin/version.json" {
                return ModpackType::Technic;
            }
        }

        for name in &names {
            if name.ends_with("manifest.json") {
                return ModpackType::Flame;
            }
            if name.ends_with("instance.cfg") {
                return ModpackType::MultiMC;
            }
        }

        ModpackType::Unknown
    }

    fn process_multimc(&self, archive: &mut zip::ZipArchive<std::fs::File>) -> Result<(), String> {
        let staging = Path::new(self.base.staging_path());
        extract_zip(archive, staging).map_err(|e| format!("Extraction failed: {}", e))
    }

    fn process_technic(&self, archive: &mut zip::ZipArchive<std::fs::File>) -> Result<(), String> {
        let staging = Path::new(self.base.staging_path());
        let game_dir = staging.join("minecraft");
        let _ = std::fs::create_dir_all(&game_dir);

        extract_zip(archive, &game_dir).map_err(|e| format!("Extraction failed: {}", e))?;

        let cfg_path = staging.join("instance.cfg");
        let mut ini = kcraft_core::INIFile::new();
        ini.set("InstanceType", "OneSix");
        ini.set("name", &self.base.name());
        ini.set("iconKey", self.base.icon());
        ini.set("notes", "Imported Technic Modpack");
        let _ = ini.save_file(&cfg_path);

        let mmc_pack_path = staging.join("mmc-pack.json");
        let components = vec![serde_json::json!({
            "uid": "net.minecraft",
            "version": "1.12.2",
            "important": true
        })];
        if let Ok(json) = serde_json::to_string_pretty(&components) {
            let _ = std::fs::write(&mmc_pack_path, json);
        }

        Ok(())
    }

    fn process_flame(&self, archive: &mut zip::ZipArchive<std::fs::File>) -> Result<(), String> {
        let staging = Path::new(self.base.staging_path());
        let game_dir = staging.join("minecraft");
        let _ = std::fs::create_dir_all(&game_dir);

        let mut manifest_val: Option<serde_json::Value> = None;
        let mut manifest_index = None;
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                if entry.name() == "manifest.json" || entry.name().ends_with("/manifest.json") {
                    manifest_index = Some(i);
                    break;
                }
            }
        }

        if let Some(i) = manifest_index {
            if let Ok(mut entry) = archive.by_index(i) {
                use std::io::Read;
                let mut data = String::new();
                let _ = entry.read_to_string(&mut data);
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                    manifest_val = Some(val);
                }
            }
        }

        let name = manifest_val
            .as_ref()
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("CurseForge Pack");

        let mc_version = manifest_val
            .as_ref()
            .and_then(|v| v.get("minecraft"))
            .and_then(|v| v.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("1.20.1");

        let loader = manifest_val
            .as_ref()
            .and_then(|v| v.get("minecraft"))
            .and_then(|v| v.get("modLoaders"))
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|l| l.get("id"))
            .and_then(|id| id.as_str())
            .unwrap_or("");

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| format!("Entry error: {}", e))?;
            let entry_name = entry.name().to_string();
            if entry_name.starts_with("overrides/") {
                let relative_path = entry_name.strip_prefix("overrides/").unwrap();
                if relative_path.is_empty() {
                    continue;
                }
                let out_path = game_dir.join(relative_path);
                if entry.is_dir() {
                    let _ = std::fs::create_dir_all(&out_path);
                } else {
                    if let Some(parent) = out_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if let Ok(mut outfile) = std::fs::File::create(&out_path) {
                        use std::io::{Read, Write};
                        let mut data = Vec::new();
                        let _ = entry.read_to_end(&mut data);
                        let _ = outfile.write_all(&data);
                    }
                }
            }
        }

        let cfg_path = staging.join("instance.cfg");
        let mut ini = kcraft_core::INIFile::new();
        ini.set("InstanceType", "OneSix");
        ini.set("name", name);
        ini.set("iconKey", self.base.icon());
        ini.set("notes", "Imported CurseForge Modpack");
        let _ = ini.save_file(&cfg_path);

        let mmc_pack_path = staging.join("mmc-pack.json");
        let mut components = vec![serde_json::json!({
            "uid": "net.minecraft",
            "version": mc_version,
            "important": true
        })];

        if !loader.is_empty() {
            let (loader_uid, loader_ver) = if loader.starts_with("forge-") {
                (
                    "net.minecraftforge",
                    loader.strip_prefix("forge-").unwrap_or(loader),
                )
            } else if loader.starts_with("fabric-") {
                (
                    "net.fabricmc.fabric-loader",
                    loader.strip_prefix("fabric-").unwrap_or(loader),
                )
            } else {
                ("net.minecraftforge", loader)
            };
            components.push(serde_json::json!({
                "uid": loader_uid,
                "version": loader_ver,
                "important": true
            }));
        }

        if let Ok(json) = serde_json::to_string_pretty(&components) {
            let _ = std::fs::write(&mmc_pack_path, json);
        }

        Ok(())
    }

    fn process_modrinth(&self, archive: &mut zip::ZipArchive<std::fs::File>) -> Result<(), String> {
        let staging = Path::new(self.base.staging_path());
        let game_dir = staging.join("minecraft");
        let _ = std::fs::create_dir_all(&game_dir);

        let mut index_val: Option<serde_json::Value> = None;
        let mut index_i = None;
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                if entry.name() == "modrinth.index.json"
                    || entry.name().ends_with("/modrinth.index.json")
                {
                    index_i = Some(i);
                    break;
                }
            }
        }

        if let Some(i) = index_i {
            if let Ok(mut entry) = archive.by_index(i) {
                use std::io::Read;
                let mut data = String::new();
                let _ = entry.read_to_string(&mut data);
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                    index_val = Some(val);
                }
            }
        }

        let name = index_val
            .as_ref()
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Modrinth Pack");

        let mc_version = index_val
            .as_ref()
            .and_then(|v| v.get("dependencies"))
            .and_then(|d| d.get("minecraft"))
            .and_then(|v| v.as_str())
            .unwrap_or("1.20.1");

        let fabric_loader = index_val
            .as_ref()
            .and_then(|v| v.get("dependencies"))
            .and_then(|d| d.get("fabric-loader"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let forge_loader = index_val
            .as_ref()
            .and_then(|v| v.get("dependencies"))
            .and_then(|d| d.get("forge"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| format!("Entry error: {}", e))?;
            let entry_name = entry.name().to_string();
            let relative_path = if entry_name.starts_with("overrides/") {
                entry_name.strip_prefix("overrides/")
            } else if entry_name.starts_with("client-overrides/") {
                entry_name.strip_prefix("client-overrides/")
            } else {
                None
            };

            if let Some(rel_path) = relative_path {
                if rel_path.is_empty() {
                    continue;
                }
                let out_path = game_dir.join(rel_path);
                if entry.is_dir() {
                    let _ = std::fs::create_dir_all(&out_path);
                } else {
                    if let Some(parent) = out_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if let Ok(mut outfile) = std::fs::File::create(&out_path) {
                        use std::io::{Read, Write};
                        let mut data = Vec::new();
                        let _ = entry.read_to_end(&mut data);
                        let _ = outfile.write_all(&data);
                    }
                }
            }
        }

        let cfg_path = staging.join("instance.cfg");
        let mut ini = kcraft_core::INIFile::new();
        ini.set("InstanceType", "OneSix");
        ini.set("name", name);
        ini.set("iconKey", self.base.icon());
        ini.set("notes", "Imported Modrinth Modpack");
        let _ = ini.save_file(&cfg_path);

        let mmc_pack_path = staging.join("mmc-pack.json");
        let mut components = vec![serde_json::json!({
            "uid": "net.minecraft",
            "version": mc_version,
            "important": true
        })];

        if !fabric_loader.is_empty() {
            components.push(serde_json::json!({
                "uid": "net.fabricmc.fabric-loader",
                "version": fabric_loader,
                "important": true
            }));
        } else if !forge_loader.is_empty() {
            components.push(serde_json::json!({
                "uid": "net.minecraftforge",
                "version": forge_loader,
                "important": true
            }));
        }

        if let Ok(json) = serde_json::to_string_pretty(&components) {
            let _ = std::fs::write(&mmc_pack_path, json);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModpackType {
    Unknown,
    MultiMC,
    Technic,
    Flame,
    Modrinth,
}

fn copy_dir(src: &Path, dst: &Path, matcher: Option<&str>) -> Result<(), String> {
    let _ = std::fs::create_dir_all(dst);
    for entry in std::fs::read_dir(src).map_err(|e| format!("Cannot read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Cannot read entry: {}", e))?;
        let path = entry.path();

        if let Some(pattern) = matcher {
            if let Some(name) = path.to_str() {
                if name.contains(pattern) {
                    continue;
                }
            }
        }

        let file_name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        let dst_path = dst.join(&file_name);

        if path.is_dir() {
            copy_dir(&path, &dst_path, matcher)?;
        } else if path.is_file() {
            let _ = std::fs::copy(&path, &dst_path);
        }
    }
    Ok(())
}

fn extract_zip(archive: &mut zip::ZipArchive<std::fs::File>, dst: &Path) -> Result<(), String> {
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Entry error: {}", e))?;
        let entry_path = entry.name().to_string();
        let out_path = dst.join(&entry_path);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| format!("Cannot create dir: {}", e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create parent: {}", e))?;
            }
            let mut outfile = std::fs::File::create(&out_path)
                .map_err(|e| format!("Cannot create file: {}", e))?;
            use std::io::Read;
            let mut data = Vec::new();
            entry
                .read_to_end(&mut data)
                .map_err(|e| format!("Read error: {}", e))?;
            use std::io::Write;
            outfile
                .write_all(&data)
                .map_err(|e| format!("Write error: {}", e))?;
        }
    }
    Ok(())
}
