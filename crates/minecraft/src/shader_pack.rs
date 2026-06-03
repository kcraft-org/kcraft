use std::path::{Path, PathBuf};

use crate::instance::Instance;
use crate::MinecraftError;

#[derive(Debug, Clone)]
pub struct ShaderPack {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

pub fn scan_shader_packs(
    instance: &Instance,
) -> std::result::Result<Vec<ShaderPack>, MinecraftError> {
    let sp_dir_str = instance.shader_packs_dir();
    let sp_dir = Path::new(&sp_dir_str);
    if !sp_dir.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();
    let entries = std::fs::read_dir(sp_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if name.is_empty() || name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            let size = calculate_dir_size(&path);
            packs.push(ShaderPack { name, path, size });
        } else if path.extension().is_some_and(|ext| ext == "zip") {
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            packs.push(ShaderPack { name, path, size });
        }
    }

    packs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(packs)
}

pub fn install_shader_pack(
    instance: &Instance,
    source: &Path,
) -> std::result::Result<(), MinecraftError> {
    if !source.exists() {
        return Err(MinecraftError::NotFound(format!(
            "Shader pack source not found: {}",
            source.display()
        )));
    }

    let sp_dir_str = instance.shader_packs_dir();
    let sp_dir = Path::new(&sp_dir_str);
    kcraft_fs::ensure_folder_exists(sp_dir)?;

    let dest_name = source
        .file_name()
        .ok_or_else(|| MinecraftError::InvalidInput("Source path has no filename".to_string()))?;
    let dest = sp_dir.join(dest_name);

    if dest.exists() {
        return Err(MinecraftError::AlreadyExists(format!(
            "Shader pack '{}' already installed",
            dest_name.to_string_lossy()
        )));
    }

    if source.is_dir() {
        kcraft_fs::copy_dir_recursive(source, &dest, &[])?;
    } else {
        std::fs::copy(source, &dest)?;
    }

    Ok(())
}

fn calculate_dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                total += p.metadata().map(|m| m.len()).unwrap_or(0);
            } else if p.is_dir() {
                total += calculate_dir_size(&p);
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_scan_shader_packs_empty() {
        let tmp = std::env::temp_dir().join("kcraft_sp_empty");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("minecraft")).unwrap();
        let instance = Instance::new(tmp.to_string_lossy().as_ref(), "test");
        let packs = scan_shader_packs(&instance).unwrap();
        assert!(packs.is_empty());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_shader_pack() {
        let tmp = std::env::temp_dir().join("kcraft_sp_install");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("minecraft")).unwrap();
        let instance = Instance::new(tmp.to_string_lossy().as_ref(), "test");

        let src = tmp.join("BSL_v8.zip");
        fs::write(&src, "fake shader zip").unwrap();
        install_shader_pack(&instance, &src).unwrap();

        let sp_dir_str = instance.shader_packs_dir();
        let sp_dir = Path::new(&sp_dir_str);
        assert!(sp_dir.join("BSL_v8.zip").exists());
        let _ = fs::remove_dir_all(&tmp);
    }
}
