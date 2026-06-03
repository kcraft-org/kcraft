use std::path::Path;

use crate::instance::Instance;
use crate::modplatform::modrinth::MODRINTH_BASE_URL;
use crate::MinecraftError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModPlatform {
    CurseForge,
    Modrinth,
}

impl ModPlatform {
    pub fn name(&self) -> &'static str {
        match self {
            ModPlatform::CurseForge => "curseforge",
            ModPlatform::Modrinth => "modrinth",
        }
    }

    pub fn api_url(&self) -> &'static str {
        match self {
            ModPlatform::CurseForge => "https://api.curseforge.com/v1",
            ModPlatform::Modrinth => MODRINTH_BASE_URL,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModUpdate {
    pub mod_id: String,
    pub platform: ModPlatform,
    pub current_version: String,
    pub latest_version: String,
    pub mod_name: String,
    pub download_url: String,
}

pub struct ModUpdater {
    api_key: Option<String>,
}

impl ModUpdater {
    pub fn new(api_key: Option<String>) -> Self {
        ModUpdater { api_key }
    }

    pub fn check_for_updates(
        &self,
        instance: &Instance,
    ) -> std::result::Result<Vec<ModUpdate>, MinecraftError> {
        let mods_root = instance.mods_root();
        let mods_dir = Path::new(&mods_root);
        if !mods_dir.exists() {
            return Ok(Vec::new());
        }

        let mut updates = Vec::new();
        let entries: Vec<_> = std::fs::read_dir(mods_dir)
            .map_err(MinecraftError::Io)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "jar"))
            .collect();

        for entry in &entries {
            let path = entry.path();
            let metadata = match self.read_mod_metadata(&path) {
                Some(m) => m,
                None => continue,
            };

            if let Some(latest) = self.query_latest_version(&metadata.0, &metadata.1)? {
                if latest != metadata.2 {
                    let dl_url = self.get_download_url(&metadata.0, &metadata.1)?;
                    updates.push(ModUpdate {
                        mod_id: metadata.0.clone(),
                        platform: metadata.1,
                        current_version: metadata.2,
                        latest_version: latest,
                        mod_name: path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        download_url: dl_url,
                    });
                }
            }
        }

        Ok(updates)
    }

    pub fn apply_updates(
        &self,
        instance: &Instance,
        updates: &[ModUpdate],
    ) -> std::result::Result<(), MinecraftError> {
        let mods_root = instance.mods_root();
        let mods_dir = Path::new(&mods_root);
        let backup_dir = mods_dir.join(".kcraft_backup");
        kcraft_fs::ensure_folder_exists(&backup_dir)?;

        for update in updates {
            let mod_path = Self::find_mod_file(mods_dir, &update.mod_id);
            if let Some(ref old_path) = mod_path {
                let backup_path = backup_dir.join(
                    old_path
                        .file_name()
                        .unwrap_or_else(|| std::ffi::OsStr::new("unknown.jar")),
                );
                std::fs::copy(old_path, &backup_path)?;
            }

            let file_name = update.download_url.rsplit('/').next().unwrap_or("mod.jar");
            let dest_path = mods_dir.join(file_name);

            let response = reqwest::blocking::get(&update.download_url)
                .map_err(|e| MinecraftError::Net(kcraft_net::NetError::Network(e.to_string())))?;
            let status = response.status();
            if !status.is_success() {
                return Err(MinecraftError::ModUpdate(format!(
                    "Failed to download {}: HTTP {}",
                    update.mod_id,
                    status.as_u16()
                )));
            }
            let bytes = response
                .bytes()
                .map_err(|e| MinecraftError::Net(kcraft_net::NetError::Network(e.to_string())))?;

            kcraft_fs::write(&dest_path, &bytes)?;

            if let Some(ref old_path) = mod_path {
                if old_path != &dest_path {
                    std::fs::remove_file(old_path)?;
                }
            }
        }

        Ok(())
    }

    fn read_mod_metadata(&self, path: &Path) -> Option<(String, ModPlatform, String)> {
        let file = std::fs::File::open(path).ok()?;
        let mut archive = zip::ZipArchive::new(file).ok()?;

        if let Ok(mut entry) = archive.by_name("fabric.mod.json") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut entry, &mut content).ok()?;
            let json: serde_json::Value = serde_json::from_str(&content).ok()?;
            let mod_id = json.get("id")?.as_str()?.to_string();
            let version = json
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            return Some((mod_id, ModPlatform::Modrinth, version));
        }

        if let Ok(mut entry) = archive.by_name("META-INF/mods.toml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut entry, &mut content).ok()?;
            for line in content.lines() {
                if let Some(stripped) = line.trim().strip_prefix("modId=") {
                    let mod_id = stripped.trim_matches('"').to_string();
                    return Some((mod_id, ModPlatform::CurseForge, "unknown".to_string()));
                }
            }
        }

        if let Ok(mut entry) = archive.by_name("META-INF/neoforge.mods.toml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut entry, &mut content).ok()?;
            for line in content.lines() {
                if let Some(stripped) = line.trim().strip_prefix("modId=") {
                    let mod_id = stripped.trim_matches('"').to_string();
                    return Some((mod_id, ModPlatform::CurseForge, "unknown".to_string()));
                }
            }
        }

        let fname = path.file_stem()?.to_string_lossy().to_string();
        Some((fname, ModPlatform::Modrinth, "unknown".to_string()))
    }

    fn query_latest_version(
        &self,
        mod_id: &str,
        platform: &ModPlatform,
    ) -> std::result::Result<Option<String>, MinecraftError> {
        match platform {
            ModPlatform::Modrinth => {
                let url = format!("{}/project/{}/version", MODRINTH_BASE_URL, mod_id);
                let client = reqwest::blocking::Client::new();
                let resp = client.get(&url).send().map_err(|e| {
                    MinecraftError::Net(kcraft_net::NetError::Network(e.to_string()))
                })?;
                if !resp.status().is_success() {
                    return Ok(None);
                }
                let versions: Vec<serde_json::Value> = resp
                    .json()
                    .map_err(|e| MinecraftError::Serialization(e.to_string()))?;
                if let Some(first) = versions.first() {
                    let version = first
                        .get("version_number")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if version.is_empty() {
                        return Ok(None);
                    }
                    Ok(Some(version))
                } else {
                    Ok(None)
                }
            }
            ModPlatform::CurseForge => {
                let url = format!(
                    "https://api.curseforge.com/v1/mods/{}/files?pageSize=1",
                    mod_id
                );
                let client = reqwest::blocking::Client::new();
                let mut req = client.get(&url);
                if let Some(ref key) = self.api_key {
                    req = req.header("x-api-key", key);
                }
                let resp = req.send().map_err(|e| {
                    MinecraftError::Net(kcraft_net::NetError::Network(e.to_string()))
                })?;
                if !resp.status().is_success() {
                    return Ok(None);
                }
                let body: serde_json::Value = resp
                    .json()
                    .map_err(|e| MinecraftError::Serialization(e.to_string()))?;
                if let Some(data) = body.get("data").and_then(|v| v.as_array()) {
                    if let Some(first) = data.first() {
                        let version = first
                            .get("displayName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if version.is_empty() {
                            return Ok(None);
                        }
                        return Ok(Some(version));
                    }
                }
                Ok(None)
            }
        }
    }

    fn get_download_url(
        &self,
        mod_id: &str,
        platform: &ModPlatform,
    ) -> std::result::Result<String, MinecraftError> {
        match platform {
            ModPlatform::Modrinth => {
                let url = format!("{}/project/{}/version", MODRINTH_BASE_URL, mod_id);
                let client = reqwest::blocking::Client::new();
                let resp = client.get(&url).send().map_err(|e| {
                    MinecraftError::Net(kcraft_net::NetError::Network(e.to_string()))
                })?;
                if !resp.status().is_success() {
                    return Err(MinecraftError::ModUpdate(format!(
                        "Failed to query Modrinth for {}",
                        mod_id
                    )));
                }
                let versions: Vec<serde_json::Value> = resp
                    .json()
                    .map_err(|e| MinecraftError::Serialization(e.to_string()))?;
                if let Some(first) = versions.first() {
                    if let Some(files) = first.get("files").and_then(|v| v.as_array()) {
                        if let Some(file) = files.first() {
                            if let Some(dl_url) = file.get("url").and_then(|v| v.as_str()) {
                                return Ok(dl_url.to_string());
                            }
                        }
                    }
                }
                Err(MinecraftError::ModUpdate(format!(
                    "No download URL found for {}",
                    mod_id
                )))
            }
            ModPlatform::CurseForge => {
                let url = format!(
                    "https://api.curseforge.com/v1/mods/{}/files?pageSize=1",
                    mod_id
                );
                let client = reqwest::blocking::Client::new();
                let mut req = client.get(&url);
                if let Some(ref key) = self.api_key {
                    req = req.header("x-api-key", key);
                }
                let resp = req.send().map_err(|e| {
                    MinecraftError::Net(kcraft_net::NetError::Network(e.to_string()))
                })?;
                if !resp.status().is_success() {
                    return Err(MinecraftError::ModUpdate(format!(
                        "Failed to query CurseForge for {}",
                        mod_id
                    )));
                }
                let body: serde_json::Value = resp
                    .json()
                    .map_err(|e| MinecraftError::Serialization(e.to_string()))?;
                if let Some(data) = body.get("data").and_then(|v| v.as_array()) {
                    if let Some(first) = data.first() {
                        if let Some(dl_url) = first.get("downloadUrl").and_then(|v| v.as_str()) {
                            return Ok(dl_url.to_string());
                        }
                    }
                }
                Err(MinecraftError::ModUpdate(format!(
                    "No download URL found for CurseForge mod {}",
                    mod_id
                )))
            }
        }
    }

    fn find_mod_file(mods_dir: &Path, mod_id: &str) -> Option<std::path::PathBuf> {
        let entries = std::fs::read_dir(mods_dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jar") {
                let fname = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if fname.contains(mod_id.to_lowercase().as_str()) {
                    return Some(path);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_platform_name() {
        assert_eq!(ModPlatform::CurseForge.name(), "curseforge");
        assert_eq!(ModPlatform::Modrinth.name(), "modrinth");
    }

    #[test]
    fn test_mod_platform_api_url() {
        assert!(ModPlatform::CurseForge.api_url().contains("curseforge"));
        assert!(ModPlatform::Modrinth.api_url().contains("modrinth"));
    }
}
