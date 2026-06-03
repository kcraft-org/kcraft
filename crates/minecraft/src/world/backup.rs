use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use zip::write::FileOptions;
use zip::ZipWriter;

use crate::instance::Instance;
use crate::MinecraftError;

#[derive(Debug, Clone)]
pub struct WorldBackup {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub size_bytes: u64,
    pub path: PathBuf,
    pub world_name: String,
}

pub fn create_backup(
    instance: &Instance,
    world_name: &str,
    backup_dir: &Path,
) -> std::result::Result<WorldBackup, MinecraftError> {
    let world_dir_str = instance.world_dir();
    let saves_dir = Path::new(&world_dir_str);
    let world_src = saves_dir.join(world_name);

    if !world_src.exists() {
        return Err(MinecraftError::NotFound(format!(
            "World '{}' not found in saves directory",
            world_name
        )));
    }

    kcraft_fs::ensure_folder_exists(backup_dir)?;

    let timestamp = Utc::now();
    let ts_str = timestamp.format("%Y%m%d_%H%M%S").to_string();
    let safe_name: String = world_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let backup_filename = format!("{}_{}.zip", safe_name, ts_str);
    let backup_path = backup_dir.join(&backup_filename);

    let file = fs::File::create(&backup_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    add_dir_to_zip(&mut zip, &world_src, world_name, &options)?;

    zip.finish()?;

    let size_bytes = backup_path.metadata().map(|m| m.len()).unwrap_or(0);

    let metadata_path = backup_path.with_extension("json");
    let meta = serde_json::json!({
        "name": backup_filename,
        "timestamp": timestamp.to_rfc3339(),
        "world_name": world_name,
        "size_bytes": size_bytes,
    });
    kcraft_fs::write(&metadata_path, &serde_json::to_vec_pretty(&meta)?)?;

    Ok(WorldBackup {
        name: backup_filename,
        timestamp,
        size_bytes,
        path: backup_path,
        world_name: world_name.to_string(),
    })
}

pub fn list_backups(instance: &Instance) -> std::result::Result<Vec<WorldBackup>, MinecraftError> {
    let world_dir_str = instance.world_dir();
    let saves_dir = Path::new(&world_dir_str);
    let backup_dir = saves_dir
        .parent()
        .unwrap_or(Path::new("."))
        .join(".kcraft_backups");
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    let entries = fs::read_dir(&backup_dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "zip") {
            let fname = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            let metadata_path = path.with_extension("json");
            let (timestamp, world_name, size_bytes) = if metadata_path.exists() {
                if let Ok(content) = kcraft_fs::read_to_string(&metadata_path) {
                    if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                        let ts = meta
                            .get("timestamp")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                            .unwrap_or_else(|| {
                                path.metadata()
                                    .and_then(|m| m.created())
                                    .ok()
                                    .map(|t| {
                                        let d: DateTime<Utc> = t.into();
                                        d
                                    })
                                    .unwrap_or_default()
                            });
                        let wn = meta
                            .get("world_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&fname)
                            .to_string();
                        let sz = meta
                            .get("size_bytes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or_else(|| path.metadata().map(|m| m.len()).unwrap_or(0));
                        (ts, wn, sz)
                    } else {
                        fallback_metadata(&path, &fname)
                    }
                } else {
                    fallback_metadata(&path, &fname)
                }
            } else {
                fallback_metadata(&path, &fname)
            };

            backups.push(WorldBackup {
                name: fname,
                timestamp,
                size_bytes,
                path,
                world_name,
            });
        }
    }

    backups.sort_by_key(|k| std::cmp::Reverse(k.timestamp));
    Ok(backups)
}

fn fallback_metadata(path: &Path, fname: &str) -> (DateTime<Utc>, String, u64) {
    let ts = path
        .metadata()
        .and_then(|m| m.created())
        .ok()
        .map(|t| {
            let d: DateTime<Utc> = t.into();
            d
        })
        .unwrap_or_default();
    let sz = path.metadata().map(|m| m.len()).unwrap_or(0);
    (ts, fname.to_string(), sz)
}

pub fn restore_backup(
    backup: &WorldBackup,
    instance: &Instance,
) -> std::result::Result<(), MinecraftError> {
    if !backup.path.exists() {
        return Err(MinecraftError::NotFound(format!(
            "Backup file not found: {}",
            backup.path.display()
        )));
    }

    let world_dir_str = instance.world_dir();
    let saves_dir = Path::new(&world_dir_str);
    let world_dir = saves_dir.join(&backup.world_name);

    if world_dir.exists() {
        let safety_ts = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let safety_dir =
            saves_dir.join(format!("{}.safety_backup_{}", backup.world_name, safety_ts));
        fs::rename(&world_dir, &safety_dir).map_err(MinecraftError::Io)?;
    }

    let file = fs::File::open(&backup.path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    if world_dir.exists() {
        fs::remove_dir_all(&world_dir)?;
    }

    archive.extract(saves_dir)?;

    Ok(())
}

fn add_dir_to_zip(
    mut zip: &mut ZipWriter<fs::File>,
    dir: &Path,
    prefix: &str,
    options: &FileOptions<'_, ()>,
) -> std::result::Result<(), MinecraftError> {
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(dir).map_err(|_| {
            MinecraftError::InvalidInput(format!(
                "path {} is not under dir {}",
                entry.path().display(),
                dir.display()
            ))
        })?;
        let zip_path = Path::new(prefix).join(relative);

        if entry.file_type().is_dir() {
            zip.add_directory(zip_path.to_string_lossy().as_ref(), *options)?;
        } else {
            zip.start_file(zip_path.to_string_lossy().as_ref(), *options)?;
            let data = fs::read(entry.path())?;
            std::io::Write::write_all(&mut zip, &data)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_backup_roundtrip() {
        let tmp = std::env::temp_dir().join("kcraft_backup_test");
        let _ = fs::remove_dir_all(&tmp);
        let instance = Instance::new(tmp.to_string_lossy().as_ref(), "test");
        let saves_dir_str = instance.world_dir();
        let saves_dir = Path::new(&saves_dir_str);
        fs::create_dir_all(saves_dir.join("MyWorld")).unwrap();
        fs::write(saves_dir.join("MyWorld").join("level.dat"), "level data").unwrap();
        fs::write(saves_dir.join("MyWorld").join("icon.png"), "icon").unwrap();

        let backup_dir = saves_dir
            .parent()
            .unwrap_or(Path::new("."))
            .join(".kcraft_backups");
        let backup = create_backup(&instance, "MyWorld", &backup_dir).unwrap();
        assert!(backup.path.exists());
        assert!(backup.size_bytes > 0);
        assert_eq!(backup.world_name, "MyWorld");

        fs::remove_dir_all(saves_dir.join("MyWorld")).unwrap();

        restore_backup(&backup, &instance).unwrap();
        assert!(saves_dir.join("MyWorld").join("level.dat").exists());
        assert!(saves_dir.join("MyWorld").join("icon.png").exists());

        let backups = list_backups(&instance).unwrap();
        assert!(!backups.is_empty());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_list_backups_empty() {
        let tmp = std::env::temp_dir().join("kcraft_backup_empty_test");
        let _ = fs::remove_dir_all(&tmp);
        let instance = Instance::new(tmp.to_string_lossy().as_ref(), "test");
        let saves_dir_str = instance.world_dir();
        let saves_dir = Path::new(&saves_dir_str);
        fs::create_dir_all(saves_dir).unwrap();
        let backups = list_backups(&instance).unwrap();
        assert!(backups.is_empty());
        let _ = fs::remove_dir_all(&tmp);
    }
}
