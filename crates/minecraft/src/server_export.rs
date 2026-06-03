use std::path::{Path, PathBuf};

use crate::instance::Instance;
use crate::MinecraftError;

#[derive(Debug, Clone)]
pub struct ServerPackFile {
    pub source_path: PathBuf,
    pub dest_relative: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ServerPack {
    pub files: Vec<ServerPackFile>,
    pub server_properties: Option<String>,
    pub forge_installer: Option<PathBuf>,
}

pub fn export_server_pack(
    instance: &Instance,
    output: &Path,
) -> std::result::Result<(), MinecraftError> {
    let export_dir = output.join(format!("{}_server", sanitize_name(&instance.name)));

    if export_dir.exists() {
        std::fs::remove_dir_all(&export_dir)?;
    }
    ::fs::ensure_folder_exists(&export_dir)?;

    // Copy mods
    let mods_root = instance.mods_root();
    let mods_src = Path::new(&mods_root);
    if mods_src.exists() {
        let mods_dst = export_dir.join("mods");
        fs::copy_dir_recursive(mods_src, &mods_dst, &[])?;
    }

    // Copy config
    let config_src = Path::new(&instance.game_root()).join("config");
    if config_src.exists() {
        let config_dst = export_dir.join("config");
        fs::copy_dir_recursive(&config_src, &config_dst, &[])?;
    }

    // Copy default world if exists
    let world_dir = instance.world_dir();
    let saves_src = Path::new(&world_dir);
    if saves_src.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(saves_src)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        entries.sort_by_key(|e| e.file_name());
        if let Some(first_world) = entries.first() {
            let world_dst = export_dir.join(first_world.file_name());
            fs::copy_dir_recursive(&first_world.path(), &world_dst, &[])?;
        }
    }

    // Copy resource packs
    let rp_dir = instance.resource_packs_dir();
    let rp_src = Path::new(&rp_dir);
    if rp_src.exists() {
        let rp_dst = export_dir.join("resourcepacks");
        fs::copy_dir_recursive(rp_src, &rp_dst, &[])?;
    }

    // Create server properties
    let props_path = export_dir.join("server.properties");
    let props =
        "motd=A KCraft Server\nmax-players=20\ngamemode=survival\nenable-command-block=true\n";
    fs::write(&props_path, props.as_bytes())?;

    // Create launcher scripts
    let mc_version = instance
        .components
        .get_component_version("net.minecraft")
        .unwrap_or("unknown");

    write_sh_script(&export_dir, mc_version, &instance.java_path)?;
    write_bat_script(&export_dir, mc_version)?;

    // Create eula.txt
    let eula_path = export_dir.join("eula.txt");
    fs::write(&eula_path, b"eula=false\n")?;

    // Zip the export
    let zip_path = output.join(format!("{}_server.zip", sanitize_name(&instance.name)));
    let file = std::fs::File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    add_dir_to_zip(&mut zip, &export_dir, "", &options)?;
    zip.finish()?;

    // Clean up the temp directory
    let _ = std::fs::remove_dir_all(&export_dir);

    Ok(())
}

fn write_sh_script(
    export_dir: &Path,
    mc_version: &str,
    java_path: &str,
) -> std::result::Result<(), MinecraftError> {
    let java = if java_path.is_empty() {
        "java".to_string()
    } else {
        java_path.to_string()
    };

    let content = format!(
        "#!/bin/sh\n# KCraft Server Launcher\n# Minecraft {}\n\ncd \"$(dirname \"$0\")\"\n{} -Xmx2G -Xms1G -jar server.jar nogui\n",
        mc_version, java
    );

    let sh_path = export_dir.join("start.sh");
    fs::write(&sh_path, content.as_bytes())?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&sh_path) {
            let mode = meta.permissions().mode();
            let _ =
                std::fs::set_permissions(&sh_path, std::fs::Permissions::from_mode(mode | 0o111));
        }
    }

    Ok(())
}

fn write_bat_script(
    export_dir: &Path,
    mc_version: &str,
) -> std::result::Result<(), MinecraftError> {
    let content = format!(
        "@echo off\r\nREM KCraft Server Launcher\r\nREM Minecraft {}\r\n\r\njava -Xmx2G -Xms1G -jar server.jar nogui\r\npause\r\n",
        mc_version
    );

    let bat_path = export_dir.join("start.bat");
    fs::write(&bat_path, content.as_bytes())?;
    Ok(())
}

fn add_dir_to_zip(
    mut zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    prefix: &str,
    options: &zip::write::FileOptions<'_, ()>,
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
        let zip_path = if prefix.is_empty() {
            relative.to_path_buf()
        } else {
            Path::new(prefix).join(relative)
        };

        if entry.file_type().is_dir() {
            zip.add_directory(zip_path.to_string_lossy().as_ref(), *options)?;
        } else {
            zip.start_file(zip_path.to_string_lossy().as_ref(), *options)?;
            let data = std::fs::read(entry.path())?;
            std::io::Write::write_all(&mut zip, &data)?;
        }
    }
    Ok(())
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("My Modpack!"), "My_Modpack_");
        assert_eq!(sanitize_name("hello-world"), "hello-world");
    }

    #[test]
    fn test_export_empty_instance() {
        let tmp = std::env::temp_dir().join("kcraft_export_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("minecraft")).unwrap();
        fs::create_dir_all(tmp.join("minecraft").join("mods")).unwrap();
        fs::create_dir_all(tmp.join("minecraft").join("saves")).unwrap();

        let instance = Instance::new(tmp.to_string_lossy().as_ref(), "TestExport");
        let output = tmp.join("output");
        fs::create_dir_all(&output).unwrap();

        let result = export_server_pack(&instance, &output);
        assert!(result.is_ok());

        let zip_path = output.join("TestExport_server.zip");
        assert!(zip_path.exists());

        let _ = fs::remove_dir_all(&tmp);
    }
}
