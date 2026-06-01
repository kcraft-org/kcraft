use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KraftConfig {
    pub version: String,
    pub minecraft: MinecraftConfig,
    pub authentication: AuthenticationConfig,
    pub modplatforms: ModPlatformsConfig,
    pub network: NetworkConfig,
    pub java: JavaConfig,
    pub launcher: LauncherConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftConfig {
    pub cdn: MinecraftCDN,
    pub paths: MinecraftPaths,
    pub defaults: MinecraftDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftCDN {
    pub base_url: String,
    pub resources: String,
    pub assets: String,
    pub libraries: String,
    pub version_manifest: String,
    pub version_list: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftPaths {
    pub instances_dir: String,
    pub java_dir: String,
    pub cache_dir: String,
    pub logs_dir: String,
    pub assets_dir: String,
    pub libraries_dir: String,
    pub natives_dir: String,
    pub game_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftDefaults {
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub jvm_args: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    pub microsoft: MicrosoftAuth,
    pub yggdrasil: YggdrasilAuth,
    pub ely_by: ElyByAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftAuth {
    pub client_id: String,
    pub redirect_uri: String,
    pub authorize_url: String,
    pub token_url: String,
    pub xbox_auth_url: String,
    pub xbox_xsts_url: String,
    pub minecraft_auth_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YggdrasilAuth {
    pub base_url: String,
    pub authenticate: String,
    pub refresh: String,
    pub validate: String,
    pub signout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElyByAuth {
    pub base_url: String,
    pub authenticate: String,
    pub refresh: String,
    pub validate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModPlatformsConfig {
    pub modrinth: ModrinthConfig,
    pub curseforge: CurseForgeConfig,
    pub atlauncher: AtLauncherConfig,
    pub ftb: FtbConfig,
    pub technic: TechnicConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthConfig {
    pub api_base: String,
    pub cdn_base: String,
    pub search_path: String,
    pub project_path: String,
    pub version_path: String,
    pub download_path: String,
    pub page_size: u32,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeConfig {
    pub api_base: String,
    pub client_id: String,
    pub api_key: String,
    pub minecraft_game_id: u32,
    pub mod_class_id: u32,
    pub search_path: String,
    pub get_mod: String,
    pub get_files: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtLauncherConfig {
    pub api_base: String,
    pub packs_path: String,
    pub pack_versions_path: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtbConfig {
    pub api_base: String,
    pub modpacks_path: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicConfig {
    pub api_base: String,
    pub modpacks_path: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub timeouts: NetworkTimeouts,
    pub cache: NetworkCache,
    pub concurrent_downloads: u32,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTimeouts {
    pub connection_secs: u64,
    pub read_secs: u64,
    pub write_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCache {
    pub metadata_expiry_days: u32,
    pub http_cache_dir: String,
    pub max_cache_size_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaConfig {
    pub search_paths: JavaSearchPaths,
    pub memory: JavaMemory,
    pub agent: JavaAgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaSearchPaths {
    pub linux: Vec<String>,
    pub macos: Vec<String>,
    pub windows: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaMemory {
    pub min_mb: u32,
    pub max_mb: u32,
    pub permgen_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaAgent {
    pub authlib_injector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub name: String,
    pub version: String,
    pub user_agent: String,
    pub data_dir: String,
    pub config_file: String,
    pub log_level: String,
    pub log_file: String,
}

impl KraftConfig {
    pub fn from_yaml<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(ConfigError::Io)?;
        serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::Parse(e.to_string()))
    }

    pub fn to_yaml(&self) -> Result<String, ConfigError> {
        serde_yaml::to_string(self)
            .map_err(|e| ConfigError::Serialization(e.to_string()))
    }
}
