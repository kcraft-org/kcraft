use std::path::Path;

use crate::instance::Instance;
use crate::modplatform::flame;
use crate::MinecraftError;

pub trait ModpackImporter {
    fn import(
        &self,
        path: &Path,
        instances_dir: &Path,
    ) -> std::result::Result<Instance, MinecraftError>;
}

pub struct FtbImporter;

impl FtbImporter {
    pub fn new() -> Self {
        FtbImporter
    }
}

impl Default for FtbImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ModpackImporter for FtbImporter {
    fn import(
        &self,
        path: &Path,
        instances_dir: &Path,
    ) -> std::result::Result<Instance, MinecraftError> {
        if !path.exists() {
            return Err(MinecraftError::NotFound(format!(
                "FTB modpack path not found: {}",
                path.display()
            )));
        }

        ::fs::ensure_folder_exists(instances_dir)?;

        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported_ftb_pack")
            .to_string();

        let extract_dir = fs::dir_name_from_string(&file_stem, instances_dir);
        ::fs::ensure_folder_exists(&extract_dir)?;

        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        archive.extract(&extract_dir)?;

        let manifest_path = extract_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(MinecraftError::Import(format!(
                "No manifest.json found in FTB modpack: {}",
                path.display()
            )));
        }

        let manifest = flame::flame_load_manifest(manifest_path.to_string_lossy().as_ref())
            .map_err(|e| MinecraftError::Import(format!("Failed to parse manifest: {}", e)))?;

        let name = if manifest.name.is_empty() {
            file_stem.clone()
        } else {
            manifest.name.clone()
        };

        let mut instance = Instance::new(extract_dir.to_string_lossy().as_ref(), &name);

        instance.managed_pack = true;
        instance.managed_pack_type = "ftb".to_string();
        instance.managed_pack_name = name.clone();

        if !manifest.minecraft_version.is_empty() {
            instance.components.set_component_version(
                "net.minecraft",
                &manifest.minecraft_version,
                true,
            );
        }

        for loader_id in &manifest.mod_loaders {
            if loader_id.contains("forge") {
                instance
                    .components
                    .set_component_version("net.minecraftforge", loader_id, false);
            } else if loader_id.contains("fabric") {
                instance.components.set_component_version(
                    "net.fabricmc.fabric-loader",
                    loader_id,
                    false,
                );
            } else if loader_id.contains("quilt") {
                instance.components.set_component_version(
                    "org.quiltmc.quilt-loader",
                    loader_id,
                    false,
                );
            }
        }

        if !manifest.overrides.is_empty() {
            let overrides_src = extract_dir.join(&manifest.overrides);
            if overrides_src.exists() {
                let mc_dir = extract_dir.join(".minecraft");
                ::fs::ensure_folder_exists(&mc_dir)?;
                fs::merge_folders(&mc_dir, &overrides_src)?;
            }
        }

        instance.save_now();
        instance.components.save_now();

        Ok(instance)
    }
}

pub struct TechnicImporter;

impl TechnicImporter {
    pub fn new() -> Self {
        TechnicImporter
    }
}

impl Default for TechnicImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ModpackImporter for TechnicImporter {
    fn import(
        &self,
        path: &Path,
        instances_dir: &Path,
    ) -> std::result::Result<Instance, MinecraftError> {
        if !path.exists() {
            return Err(MinecraftError::NotFound(format!(
                "Technic modpack path not found: {}",
                path.display()
            )));
        }

        ::fs::ensure_folder_exists(instances_dir)?;

        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported_technic_pack")
            .to_string();

        let extract_dir = fs::dir_name_from_string(&file_stem, instances_dir);
        ::fs::ensure_folder_exists(&extract_dir)?;

        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let mut _mc_version = String::new();

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(MinecraftError::Zip)?;
            let entry_name = entry.name().to_string();

            if entry_name.starts_with("bin/modpack.jar") || entry_name.ends_with("modpack.jar") {
                let dest = extract_dir.join("jarmods").join("modpack.jar");
                fs::ensure_file_path_exists(&dest)?;
                let mut data = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut data)?;
                fs::write(&dest, &data)?;
            }

            if entry_name == "config" {
                let dest = extract_dir.join(".minecraft").join("config");
                ::fs::ensure_folder_exists(&dest)?;
                let mut data = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut data)?;
                fs::write(dest.join("config_backup"), &data)?;
            }
        }

        // Full extraction
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(MinecraftError::Zip)?;
            let entry_name = entry.name().to_string();

            let dest_path =
                if entry_name.starts_with("minecraft/") || entry_name.starts_with(".minecraft/") {
                    let relative = entry_name
                        .strip_prefix("minecraft/")
                        .or_else(|| entry_name.strip_prefix(".minecraft/"))
                        .unwrap_or(&entry_name);
                    extract_dir.join(".minecraft").join(relative)
                } else if entry_name.starts_with("bin/")
                    || entry_name.starts_with("config/")
                    || entry_name.starts_with("mods/")
                {
                    extract_dir.join(".minecraft").join(&entry_name)
                } else {
                    extract_dir.join(&entry_name)
                };

            if entry.is_dir() {
                ::fs::ensure_folder_exists(&dest_path)?;
            } else {
                fs::ensure_file_path_exists(&dest_path)?;
                let mut data = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut data)?;
                fs::write(&dest_path, &data)?;
            }

            if entry_name == "manifest.json" {
                if let Ok(content) =
                    String::from_utf8(std::fs::read(&dest_path).unwrap_or_default())
                {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(_mc) = json.get("minecraft").and_then(|v| v.as_str()) {}
                    }
                }
            }
        }

        let instance = Instance::new(extract_dir.to_string_lossy().as_ref(), &file_stem);

        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ftb_importer_missing_path() {
        let importer = FtbImporter::new();
        let result = importer.import(Path::new("/nonexistent/path.zip"), Path::new("/tmp"));
        assert!(result.is_err());
    }

    #[test]
    fn test_technic_importer_missing_path() {
        let importer = TechnicImporter::new();
        let result = importer.import(Path::new("/nonexistent/path.zip"), Path::new("/tmp"));
        assert!(result.is_err());
    }

    #[test]
    fn test_import_trait_object() {
        let importers: Vec<Box<dyn ModpackImporter>> = vec![
            Box::new(FtbImporter::new()),
            Box::new(TechnicImporter::new()),
        ];
        assert_eq!(importers.len(), 2);
    }
}
