use std::path::Path;

use crate::instance::Instance;
use crate::MinecraftError;

pub struct InstanceCloner;

impl InstanceCloner {
    pub fn clone_instance(
        source: &Instance,
        new_name: &str,
        instances_dir: &Path,
    ) -> std::result::Result<Instance, MinecraftError> {
        let src_path = Path::new(&source.instance_root);
        if !src_path.exists() {
            return Err(MinecraftError::NotFound(format!(
                "Source instance not found: {}",
                source.instance_root
            )));
        }

        let dest_dir = kcraft_fs::dir_name_from_string(new_name, instances_dir);

        kcraft_fs::ensure_folder_exists(dest_dir.parent().unwrap_or(Path::new(".")))?;

        let exclusions: &[&str] = &["crash-reports", "logs", "server-scripts", "server-packs"];
        process_copy(src_path, &dest_dir, exclusions)?;

        let mut cloned = Instance::new(dest_dir.to_string_lossy().as_ref(), new_name);

        let cfg_path = dest_dir.join("instance.cfg");
        if cfg_path.exists() {
            cloned.load_specific_settings();
        }

        cloned.name = new_name.to_string();
        cloned.save_now();

        Ok(cloned)
    }
}

pub fn process_copy(
    src: &Path,
    dst: &Path,
    filter: &[&str],
) -> std::result::Result<(), MinecraftError> {
    kcraft_fs::ensure_folder_exists(dst)?;

    for entry in walkdir::WalkDir::new(src)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let relative = entry.path().strip_prefix(src).map_err(|_| {
            MinecraftError::InvalidInput(format!(
                "path {} is not under src {}",
                entry.path().display(),
                src.display()
            ))
        })?;

        let should_skip = relative
            .components()
            .any(|c| filter.contains(&c.as_os_str().to_string_lossy().as_ref()));

        if should_skip {
            continue;
        }

        let dest_path = dst.join(relative);

        if entry.file_type().is_dir() {
            kcraft_fs::ensure_folder_exists(&dest_path)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest_path.parent() {
                kcraft_fs::ensure_folder_exists(parent)?;
            }
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_process_copy_with_filter() {
        let tmp = std::env::temp_dir().join("kcraft_clone_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("src")).unwrap();
        fs::write(tmp.join("src").join("mod.jar"), "mod data").unwrap();
        fs::create_dir_all(tmp.join("src").join("logs")).unwrap();
        fs::write(tmp.join("src").join("logs").join("latest.log"), "log data").unwrap();
        fs::create_dir_all(tmp.join("src").join("crash-reports")).unwrap();
        fs::write(
            tmp.join("src").join("crash-reports").join("crash.txt"),
            "crash",
        )
        .unwrap();
        fs::create_dir_all(tmp.join("src").join("saves")).unwrap();
        fs::write(tmp.join("src").join("saves").join("level.dat"), "save data").unwrap();

        let dst = tmp.join("dst");
        process_copy(&tmp.join("src"), &dst, &["logs", "crash-reports"]).unwrap();

        assert!(dst.join("mod.jar").exists());
        assert!(dst.join("saves").join("level.dat").exists());
        assert!(!dst.join("logs").exists());
        assert!(!dst.join("crash-reports").exists());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_process_copy_no_filter() {
        let tmp = std::env::temp_dir().join("kcraft_clone_test_nofilter");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("src_mods")).unwrap();
        fs::write(tmp.join("src_mods").join("a.jar"), "a").unwrap();
        fs::write(tmp.join("src_mods").join("b.jar"), "b").unwrap();

        let dst = tmp.join("dst_mods");
        process_copy(&tmp.join("src_mods"), &dst, &[]).unwrap();

        assert!(dst.join("a.jar").exists());
        assert!(dst.join("b.jar").exists());

        let _ = fs::remove_dir_all(&tmp);
    }
}
