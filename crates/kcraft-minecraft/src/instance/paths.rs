use std::path::Path;

use super::Instance;

impl Instance {
    pub fn game_root(&self) -> String {
        let mc_dir = Path::new(&self.instance_root).join("minecraft");
        let dot_mc_dir = Path::new(&self.instance_root).join(".minecraft");
        if mc_dir.exists() && !dot_mc_dir.exists() {
            mc_dir.to_string_lossy().to_string()
        } else {
            dot_mc_dir.to_string_lossy().to_string()
        }
    }

    pub fn bin_root(&self) -> String {
        Path::new(&self.game_root())
            .join("bin")
            .to_string_lossy()
            .to_string()
    }

    pub fn get_native_path(&self) -> String {
        Path::new(&self.instance_root)
            .join("natives")
            .to_string_lossy()
            .to_string()
    }

    pub fn get_local_library_path(&self) -> String {
        Path::new(&self.instance_root)
            .join("libraries")
            .to_string_lossy()
            .to_string()
    }

    pub fn jar_mods_dir(&self) -> String {
        Path::new(&self.instance_root)
            .join("jarmods")
            .to_string_lossy()
            .to_string()
    }

    pub fn mods_root(&self) -> String {
        Path::new(&self.game_root())
            .join("mods")
            .to_string_lossy()
            .to_string()
    }

    pub fn core_mods_dir(&self) -> String {
        Path::new(&self.game_root())
            .join("coremods")
            .to_string_lossy()
            .to_string()
    }

    pub fn resource_packs_dir(&self) -> String {
        Path::new(&self.game_root())
            .join("resourcepacks")
            .to_string_lossy()
            .to_string()
    }

    pub fn texture_packs_dir(&self) -> String {
        Path::new(&self.game_root())
            .join("texturepacks")
            .to_string_lossy()
            .to_string()
    }

    pub fn shader_packs_dir(&self) -> String {
        Path::new(&self.game_root())
            .join("shaderpacks")
            .to_string_lossy()
            .to_string()
    }

    pub fn world_dir(&self) -> String {
        Path::new(&self.game_root())
            .join("saves")
            .to_string_lossy()
            .to_string()
    }

    pub fn resources_dir(&self) -> String {
        Path::new(&self.game_root())
            .join("resources")
            .to_string_lossy()
            .to_string()
    }
}
