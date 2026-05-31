use std::collections::HashMap;
use std::path::Path;

use crate::assets::assets_utils;
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
            let mut ini = kcraft_core::INIFile::new();
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
        let mut ini = kcraft_core::INIFile::new();
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

    pub fn get_class_path(&self) -> Vec<String> {
        let mut jars = Vec::new();
        if let Some(profile) = self.components.get_profile() {
            for lib in &profile.libraries {
                if !lib.is_native() {
                    let path = Path::new("libraries").join(lib.name.to_path(None));
                    jars.push(path.to_string_lossy().to_string());
                }
            }
            if let Some(ref mj) = profile.main_jar {
                let path = Path::new("versions")
                    .join(mj.name.version())
                    .join(format!("{}.jar", mj.name.version()));
                jars.push(path.to_string_lossy().to_string());
            }
        }
        jars
    }

    pub fn get_main_class(&self) -> String {
        self.components
            .get_profile()
            .and_then(|p| p.main_class.clone())
            .unwrap_or_else(|| "net.minecraft.client.Minecraft".to_string())
    }

    pub fn get_native_jars(&self) -> Vec<String> {
        let mut natives = Vec::new();
        if let Some(profile) = self.components.get_profile() {
            for lib in &profile.native_libraries {
                let path = Path::new("libraries").join(lib.name.to_path(None));
                natives.push(path.to_string_lossy().to_string());
            }
        }
        natives
    }

    pub fn java_arguments(&self) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        let min = self.min_mem;
        let max = self.max_mem;
        if min < max {
            args.push(format!("-Xms{}m", min));
            args.push(format!("-Xmx{}m", max));
        } else {
            args.push(format!("-Xms{}m", max));
            args.push(format!("-Xmx{}m", min));
        }

        let jv = crate::JavaVersion::new(&self.java_version);
        if jv.requires_perm_gen() && self.perm_gen != 64 {
            args.push(format!("-XX:PermSize={}m", self.perm_gen));
        }

        args.push("-Duser.language=en".to_string());

        #[cfg(target_os = "windows")]
        args.push("-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump".to_string());
        #[cfg(target_os = "macos")]
        args.push("-Xdock:icon=icon.png".to_string());

        if !self.jvm_args.is_empty() {
            for arg in self.jvm_args.split(' ') {
                let arg = arg.trim();
                if !arg.is_empty() {
                    args.push(arg.to_string());
                }
            }
        }

        args
    }

    pub fn get_variables(&self) -> HashMap<String, String> {
        let mut out = HashMap::new();
        out.insert("INST_NAME".to_string(), self.name.clone());
        out.insert("INST_ID".to_string(), self.instance_root.clone());
        out.insert(
            "INST_DIR".to_string(),
            Path::new(&self.instance_root)
                .canonicalize()
                .unwrap_or_else(|_| Path::new(&self.instance_root).to_path_buf())
                .to_string_lossy()
                .to_string(),
        );
        out.insert(
            "INST_MC_DIR".to_string(),
            Path::new(&self.game_root())
                .canonicalize()
                .unwrap_or_else(|_| Path::new(&self.game_root()).to_path_buf())
                .to_string_lossy()
                .to_string(),
        );
        out.insert("INST_JAVA".to_string(), self.java_path.clone());
        out.insert(
            "INST_JAVA_ARGS".to_string(),
            self.java_arguments().join(" "),
        );
        out
    }

    pub fn get_launch_script(
        &self,
        session: Option<&crate::AuthSession>,
        server: Option<&crate::MinecraftServerTarget>,
    ) -> String {
        let profile = match self.components.get_profile() {
            Some(p) => p,
            None => return String::new(),
        };

        let mut script = String::new();

        let main_class = self.get_main_class();
        if !main_class.is_empty() {
            script.push_str(&format!("mainClass {}\n", main_class));
        }
        if let Some(ref ac) = profile.applet_class {
            script.push_str(&format!("appletClass {}\n", ac));
        }

        if let Some(server) = server {
            script.push_str(&format!("serverAddress {}\n", server.address));
            script.push_str(&format!("serverPort {}\n", server.port));
        }

        for param in self.process_minecraft_args(session, None) {
            script.push_str(&format!("param {}\n", param));
        }

        if self.launch_maximized {
            script.push_str("windowParams max\n");
        } else {
            script.push_str(&format!(
                "windowParams {}x{}\n",
                self.window_width, self.window_height
            ));
        }

        if let Some(s) = session {
            script.push_str(&format!("userName {}\n", s.player_name));
            script.push_str(&format!("sessionId {}\n", s.session));
        }

        for file in self.get_class_path() {
            script.push_str(&format!("cp {}\n", file));
        }
        for file in self.get_native_jars() {
            script.push_str(&format!("ext {}\n", file));
        }
        script.push_str(&format!("natives {}\n", self.get_native_path()));

        for trait_name in profile.traits.iter() {
            script.push_str(&format!("traits {}\n", trait_name));
        }
        script.push_str("launcher onesix\n");

        script
    }

    pub fn process_minecraft_args(
        &self,
        session: Option<&crate::AuthSession>,
        _server: Option<&crate::MinecraftServerTarget>,
    ) -> Vec<String> {
        let profile = match self.components.get_profile() {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut args_pattern = profile.minecraft_arguments.clone().unwrap_or_default();

        for tweaker in &profile.add_tweakers {
            args_pattern.push_str(&format!(" --tweakClass {}", tweaker));
        }

        let mut token_mapping: HashMap<String, String> = HashMap::new();

        if let Some(s) = session {
            token_mapping.insert("auth_session".to_string(), s.session.clone());
            token_mapping.insert("auth_access_token".to_string(), s.access_token.clone());
            token_mapping.insert("auth_player_name".to_string(), s.player_name.clone());
            token_mapping.insert("auth_uuid".to_string(), s.uuid.clone());
            token_mapping.insert("user_properties".to_string(), s.serialize_user_properties());
            token_mapping.insert("user_type".to_string(), s.user_type.clone());
            if s.demo {
                args_pattern.push_str(" --demo");
            }
        }

        token_mapping.insert("profile_name".to_string(), self.name.clone());
        token_mapping.insert(
            "version_name".to_string(),
            profile.minecraft_version.clone().unwrap_or_default(),
        );
        token_mapping.insert(
            "version_type".to_string(),
            profile
                .type_
                .clone()
                .unwrap_or_else(|| "release".to_string()),
        );

        let abs_root_dir = Path::new(&self.game_root())
            .canonicalize()
            .unwrap_or_else(|_| Path::new(&self.game_root()).to_path_buf())
            .to_string_lossy()
            .to_string();
        token_mapping.insert("game_directory".to_string(), abs_root_dir);

        token_mapping.insert(
            "game_assets".to_string(),
            assets_utils::get_assets_dir(&profile.assets, &self.resources_dir()),
        );
        token_mapping.insert("assets_root".to_string(), "assets".to_string());
        token_mapping.insert("assets_index_name".to_string(), profile.assets.clone());

        let parts: Vec<String> = args_pattern
            .split(' ')
            .filter(|s| !s.is_empty())
            .map(|s| replace_tokens(s, &token_mapping))
            .collect();

        parts
    }

    pub fn id(&self) -> String {
        Path::new(&self.instance_root)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    pub fn can_launch(&self) -> bool {
        !self.has_broken_version && !self.crashed
    }
}

fn replace_tokens(text: &str, mapping: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut last = 0;
    let len = text.len();

    while last < len {
        if let Some(start) = text[last..].find("${") {
            let abs_start = last + start;
            if let Some(end) = text[abs_start + 2..].find('}') {
                let abs_end = abs_start + 2 + end;
                result.push_str(&text[last..abs_start]);
                let key = &text[abs_start + 2..abs_end];
                if let Some(value) = mapping.get(key) {
                    result.push_str(value);
                } else {
                    result.push_str(&text[abs_start..=abs_end]);
                }
                last = abs_end + 1;
            } else {
                result.push_str(&text[last..]);
                break;
            }
        } else {
            result.push_str(&text[last..]);
            break;
        }
    }
    result
}

#[derive(Debug, Clone)]
pub struct MinecraftServerTarget {
    pub address: String,
    pub port: u16,
}

impl MinecraftServerTarget {
    pub fn parse(full_address: &str) -> Self {
        let parts: Vec<&str> = full_address.split(':').collect();
        let address = parts[0].to_string();
        let port = parts
            .get(1)
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(25565);
        MinecraftServerTarget { address, port }
    }

    pub fn new(address: &str, port: u16) -> Self {
        MinecraftServerTarget {
            address: address.to_string(),
            port,
        }
    }
}
