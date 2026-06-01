use std::collections::HashMap;
use std::path::Path;

use crate::assets::assets_utils;

use super::Instance;

impl Instance {
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
