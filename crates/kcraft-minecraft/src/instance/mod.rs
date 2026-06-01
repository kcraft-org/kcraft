use std::collections::HashMap;

use crate::component::PackProfile;

#[derive(Debug, Clone)]
pub struct InstanceSettings {
    pub settings: HashMap<String, String>,
}

impl InstanceSettings {
    pub fn new() -> Self {
        InstanceSettings {
            settings: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.settings.get(key).cloned()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.settings.insert(key.to_string(), value.to_string());
    }
}

impl Default for InstanceSettings {
    fn default() -> Self {
        Self::new()
    }
}

pub type MinecraftInstance = Instance;

#[derive(Debug, Clone)]
pub struct Instance {
    pub instance_root: String,
    pub name: String,
    pub icon_key: String,
    pub notes: String,
    pub last_launch_time: i64,
    pub total_time_played: i64,
    pub managed_pack: bool,
    pub managed_pack_type: String,
    pub managed_pack_id: String,
    pub managed_pack_name: String,
    pub managed_pack_version_id: String,
    pub managed_pack_version_name: String,
    pub has_broken_version: bool,
    pub has_update: bool,
    pub crashed: bool,
    pub java_path: String,
    pub java_version: String,
    pub min_mem: i32,
    pub max_mem: i32,
    pub perm_gen: i32,
    pub jvm_args: String,
    pub window_width: i32,
    pub window_height: i32,
    pub launch_maximized: bool,
    pub use_native_openal: bool,
    pub use_native_glfw: bool,
    pub enable_feral_gamemode: bool,
    pub enable_mangohud: bool,
    pub use_discrete_gpu: bool,
    pub close_after_launch: bool,
    pub quit_after_game_stop: bool,
    pub join_server_on_launch: bool,
    pub join_server_address: String,
    pub components: PackProfile,
    pub settings: InstanceSettings,
    pub server_address: Option<String>,
    pub server_port: Option<u16>,
}

mod config;
mod launch;
mod paths;
mod server_target;

pub use server_target::MinecraftServerTarget;

impl Instance {
    pub fn id(&self) -> String {
        std::path::Path::new(&self.instance_root)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    pub fn can_launch(&self) -> bool {
        !self.has_broken_version && !self.crashed
    }
}
