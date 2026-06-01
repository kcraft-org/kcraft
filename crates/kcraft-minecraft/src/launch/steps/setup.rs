use crate::assets::assets_utils;
use crate::launch::state::LogLevel;
use crate::launch::task::LaunchStep;
use crate::launch::task::LaunchTask;

pub struct CreateGameFoldersStep;

impl LaunchStep for CreateGameFoldersStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        let instance = &task.instance;
        let dirs = [
            instance.game_root(),
            instance.bin_root(),
            instance.get_native_path(),
            instance.mods_root(),
            instance.core_mods_dir(),
            instance.resource_packs_dir(),
            instance.jar_mods_dir(),
        ];

        for dir in &dirs {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
            task.log(&format!("Created directory: {}", dir), LogLevel::Launcher);
        }

        Ok(())
    }

    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "CreateGameFolders"
    }
}

pub struct ReconstructAssetsStep;

impl LaunchStep for ReconstructAssetsStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        let assets = task
            .instance
            .components
            .get_profile()
            .map(|p| p.assets.clone())
            .unwrap_or_default();
        let resources_dir = task.instance.resources_dir();

        task.log("Reconstructing assets...", LogLevel::Launcher);
        assets_utils::reconstruct_assets(&assets, &resources_dir);
        task.log("Assets reconstructed.", LogLevel::Launcher);
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "ReconstructAssets"
    }
}

pub struct ScanModFoldersStep;

impl LaunchStep for ScanModFoldersStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        let mods_dir = task.instance.mods_root();
        if std::path::Path::new(&mods_dir).exists() {
            if let Ok(entries) = std::fs::read_dir(&mods_dir) {
                let count = entries.flatten().filter(|e| e.path().is_file()).count();
                task.log(
                    &format!("Mods folder contains {} files", count),
                    LogLevel::Launcher,
                );
            }
        }
        let coremods_dir = task.instance.core_mods_dir();
        if std::path::Path::new(&coremods_dir).exists() {
            if let Ok(entries) = std::fs::read_dir(&coremods_dir) {
                let count = entries.flatten().filter(|e| e.path().is_file()).count();
                task.log(
                    &format!("Coremods folder contains {} files", count),
                    LogLevel::Launcher,
                );
            }
        }
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "ScanModFolders"
    }
}
