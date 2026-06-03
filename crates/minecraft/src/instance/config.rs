use std::path::Path;

use super::{Instance, InstanceSettings, PackProfile};

impl Instance {
    pub fn new(instance_root: &str, name: &str) -> Self {
        Instance {
            instance_root: instance_root.to_string(),
            name: name.to_string(),
            icon_key: "default".to_string(),
            notes: String::new(),
            last_launch_time: 0,
            total_time_played: 0,
            managed_pack: false,
            managed_pack_type: String::new(),
            managed_pack_id: String::new(),
            managed_pack_name: String::new(),
            managed_pack_version_id: String::new(),
            managed_pack_version_name: String::new(),
            has_broken_version: false,
            has_update: false,
            crashed: false,
            java_path: String::new(),
            java_version: String::new(),
            min_mem: 512,
            max_mem: 2048,
            perm_gen: 64,
            jvm_args: String::new(),
            window_width: 854,
            window_height: 480,
            launch_maximized: false,
            use_native_openal: false,
            use_native_glfw: false,
            enable_feral_gamemode: false,
            enable_mangohud: false,
            use_discrete_gpu: false,
            close_after_launch: false,
            quit_after_game_stop: false,
            join_server_on_launch: false,
            join_server_address: String::new(),
            components: PackProfile::new(instance_root),
            settings: InstanceSettings::new(),
            server_address: None,
            server_port: None,
        }
    }

    pub fn load_specific_settings(&mut self) {
        let cfg_path = Path::new(&self.instance_root).join("instance.cfg");
        if let Ok(content) = std::fs::read_to_string(&cfg_path) {
            let mut ini = app_core::INIFile::new();
            ini.load(&content);

            if let Some(v) = ini.get("name") {
                self.name = v.to_string();
            }
            if let Some(v) = ini.get("iconKey") {
                self.icon_key = v.to_string();
            }
            if let Some(v) = ini.get("notes") {
                self.notes = v.to_string();
            }
            if let Some(v) = ini.get("totalTimePlayed") {
                self.total_time_played = v.parse().unwrap_or(0);
            }
            if let Some(v) = ini.get("lastLaunchTime") {
                self.last_launch_time = v.parse().unwrap_or(0);
            }
            if let Some(v) = ini.get("JavaPath") {
                self.java_path = v.to_string();
            }
            if let Some(v) = ini.get("JavaVersion") {
                self.java_version = v.to_string();
            }
            if let Some(v) = ini.get("MinMemAlloc") {
                self.min_mem = v.parse().unwrap_or(512);
            }
            if let Some(v) = ini.get("MaxMemAlloc") {
                self.max_mem = v.parse().unwrap_or(2048);
            }
            if let Some(v) = ini.get("PermGen") {
                self.perm_gen = v.parse().unwrap_or(64);
            }
            if let Some(v) = ini.get("JvmArgs") {
                self.jvm_args = v.to_string();
            }
            if let Some(v) = ini.get("ManagedPack") {
                self.managed_pack = v == "true";
            }
            if let Some(v) = ini.get("ManagedPackType") {
                self.managed_pack_type = v.to_string();
            }
            if let Some(v) = ini.get("ManagedPackID") {
                self.managed_pack_id = v.to_string();
            }
            if let Some(v) = ini.get("ManagedPackName") {
                self.managed_pack_name = v.to_string();
            }
            if let Some(v) = ini.get("ManagedPackVersionID") {
                self.managed_pack_version_id = v.to_string();
            }
            if let Some(v) = ini.get("ManagedPackVersionName") {
                self.managed_pack_version_name = v.to_string();
            }
        }
    }

    pub fn load_managed_pack(&self) {
        // loads managed pack data from patches
    }

    pub fn save_now(&self) {
        let cfg_path = Path::new(&self.instance_root).join("instance.cfg");
        let mut ini = app_core::INIFile::new();
        ini.set("InstanceType", "OneSix");
        ini.set("name", &self.name);
        ini.set("iconKey", &self.icon_key);
        ini.set("notes", &self.notes);
        ini.set("totalTimePlayed", &self.total_time_played.to_string());
        ini.set("lastLaunchTime", &self.last_launch_time.to_string());
        ini.set("JavaPath", &self.java_path);
        ini.set("JavaVersion", &self.java_version);
        ini.set("MinMemAlloc", &self.min_mem.to_string());
        ini.set("MaxMemAlloc", &self.max_mem.to_string());
        ini.set("PermGen", &self.perm_gen.to_string());
        ini.set("JvmArgs", &self.jvm_args);
        if self.managed_pack {
            ini.set("ManagedPack", "true");
            ini.set("ManagedPackType", &self.managed_pack_type);
            ini.set("ManagedPackID", &self.managed_pack_id);
            ini.set("ManagedPackName", &self.managed_pack_name);
            ini.set("ManagedPackVersionID", &self.managed_pack_version_id);
            ini.set("ManagedPackVersionName", &self.managed_pack_version_name);
        }
        let _ = ini.save_file(&cfg_path);
    }
}
