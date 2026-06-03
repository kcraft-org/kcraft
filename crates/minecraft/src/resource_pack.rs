use std::io::Read;
use std::path::{Path, PathBuf};

use crate::instance::Instance;
use crate::MinecraftError;

#[derive(Debug, Clone)]
pub struct ResourcePack {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub pack_format: i32,
    pub description: String,
}

pub fn scan_resource_packs(
    instance: &Instance,
) -> std::result::Result<Vec<ResourcePack>, MinecraftError> {
    let rp_dir_str = instance.resource_packs_dir();
    let rp_dir = Path::new(&rp_dir_str);
    if !rp_dir.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();
    let entries = std::fs::read_dir(rp_dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let meta_path = path.join("pack.mcmeta");
            if meta_path.exists() {
                if let Some(pack) = read_pack_meta(&path, &meta_path) {
                    packs.push(pack);
                }
            }
        } else if path.extension().is_some_and(|ext| ext == "zip") {
            if let Some(pack) = read_pack_meta_from_zip(&path) {
                packs.push(pack);
            }
        }
    }

    packs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(packs)
}

pub fn install_resource_pack(
    instance: &Instance,
    source: &Path,
) -> std::result::Result<(), MinecraftError> {
    if !source.exists() {
        return Err(MinecraftError::NotFound(format!(
            "Resource pack source not found: {}",
            source.display()
        )));
    }

    let rp_dir_str = instance.resource_packs_dir();
    let rp_dir = Path::new(&rp_dir_str);
    ::fs::ensure_folder_exists(rp_dir)?;

    let dest_name = source
        .file_name()
        .ok_or_else(|| MinecraftError::InvalidInput("Source path has no filename".to_string()))?;
    let dest = rp_dir.join(dest_name);

    if dest.exists() {
        return Err(MinecraftError::AlreadyExists(format!(
            "Resource pack '{}' already installed",
            dest_name.to_string_lossy()
        )));
    }

    if source.is_dir() {
        fs::copy_dir_recursive(source, &dest, &[])?;
    } else {
        std::fs::copy(source, &dest)?;
    }

    Ok(())
}

pub fn remove_resource_pack(
    instance: &Instance,
    name: &str,
) -> std::result::Result<(), MinecraftError> {
    let rp_dir_str = instance.resource_packs_dir();
    let rp_dir = Path::new(&rp_dir_str);
    let pack_path = rp_dir.join(name);

    if !pack_path.exists() {
        return Err(MinecraftError::NotFound(format!(
            "Resource pack '{}' not found",
            name
        )));
    }

    if pack_path.is_dir() {
        std::fs::remove_dir_all(&pack_path)?;
    } else {
        std::fs::remove_file(&pack_path)?;
    }

    Ok(())
}

pub fn enable_pack(instance: &Instance, name: &str) -> std::result::Result<(), MinecraftError> {
    let options_path = Path::new(&instance.game_root()).join("options.txt");
    let content = if options_path.exists() {
        fs::read_to_string(&options_path)?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut found = false;

    for line in &mut lines {
        if line.starts_with("resourcePacks:") {
            if !line.contains(name) && line.ends_with(']') {
                let insert_pos = line.len() - 1;
                let prefix = if line.len() > 14 && line[14..insert_pos].is_empty() {
                    String::new()
                } else {
                    ",".to_string()
                };
                line.insert_str(insert_pos, &format!("{prefix}\"{}\"", name));
            }
            found = true;
            break;
        }
    }

    if !found {
        lines.push(format!("resourcePacks:[\"{}\"]", name));
    }

    fs::write(&options_path, lines.join("\n").as_bytes())?;
    Ok(())
}

pub fn disable_pack(instance: &Instance, name: &str) -> std::result::Result<(), MinecraftError> {
    let options_path = Path::new(&instance.game_root()).join("options.txt");
    if !options_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&options_path)?;
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    for line in &mut lines {
        if line.starts_with("resourcePacks:") {
            let stripped = line
                .replace(&format!("\"{}\"", name), "")
                .replace(",]", "]")
                .replace(",,", ",");
            *line = stripped;
            break;
        }
    }

    fs::write(&options_path, lines.join("\n").as_bytes())?;
    Ok(())
}

fn read_pack_meta(dir: &Path, meta_path: &Path) -> Option<ResourcePack> {
    let content = std::fs::read_to_string(meta_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let pack = json.get("pack")?;
    let pack_format = pack
        .get("pack_format")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let description = pack
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let size = calculate_size(dir);
    let path = dir.to_path_buf();

    Some(ResourcePack {
        name,
        path,
        size,
        pack_format,
        description,
    })
}

fn read_pack_meta_from_zip(zip_path: &Path) -> Option<ResourcePack> {
    let file = std::fs::File::open(zip_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let mut entry = archive.by_name("pack.mcmeta").ok()?;
    let mut content = String::new();
    entry.read_to_string(&mut content).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let pack = json.get("pack")?;
    let pack_format = pack
        .get("pack_format")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let description = pack
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let name = zip_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let size = zip_path.metadata().ok().map(|m| m.len()).unwrap_or(0);
    let path = zip_path.to_path_buf();

    Some(ResourcePack {
        name,
        path,
        size,
        pack_format,
        description,
    })
}

fn calculate_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                total += path.metadata().map(|m| m.len()).unwrap_or(0);
            } else if path.is_dir() {
                total += calculate_size(&path);
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
    fn test_scan_resource_packs_empty() {
        let tmp = std::env::temp_dir().join("kcraft_rp_empty");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("minecraft")).unwrap();
        let instance = Instance::new(tmp.to_string_lossy().as_ref(), "test");
        let packs = scan_resource_packs(&instance).unwrap();
        assert!(packs.is_empty());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_remove_resource_pack() {
        let tmp = std::env::temp_dir().join("kcraft_rp_install");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("minecraft")).unwrap();
        let instance = Instance::new(tmp.to_string_lossy().as_ref(), "test");

        let src = tmp.join("mypack.zip");
        fs::write(&src, "fake zip content").unwrap();

        install_resource_pack(&instance, &src).unwrap();
        let rp_dir_str = instance.resource_packs_dir();
        let rp_dir = Path::new(&rp_dir_str);
        assert!(rp_dir.join("mypack.zip").exists());

        remove_resource_pack(&instance, "mypack.zip").unwrap();
        assert!(!rp_dir.join("mypack.zip").exists());

        let _ = fs::remove_dir_all(&tmp);
    }
}
