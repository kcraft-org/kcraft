use std::sync::LazyLock;

pub static BUILD_CONFIG: LazyLock<BuildConfig> = LazyLock::new(BuildConfig::default);

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BuildConfig {
    pub launcher_name: String,
    pub launcher_display_name: String,
    pub launcher_copyright: String,
    pub launcher_domain: String,
    pub launcher_config_file: String,
    pub version_major: i32,
    pub version_minor: i32,
    pub version_channel: String,
    pub updater_enabled: bool,
    #[serde(default = "BuildConfig::detect_platform")]
    pub build_platform: String,
    pub user_agent: String,
    pub user_agent_uncached: String,
    pub meta_url: String,
    pub msa_client_id: String,
    pub flame_api_key: String,
    pub imgur_client_id: String,
    pub news_rss_url: String,
    pub news_open_url: String,
    pub help_url: String,
    pub bug_tracker_url: String,
    pub translations_url: String,
    pub discord_url: String,
    pub subreddit_url: String,
    pub resource_base: String,
    pub library_base: String,
    pub auth_base: String,
    pub imgur_base_url: String,
    pub fmllibs_base_url: String,
    pub modpacksch_api_base_url: String,
    pub legacy_ftb_cdn_base_url: String,
    pub atl_download_server_url: String,
    pub atl_api_base_url: String,
    pub technic_api_base_url: String,
    pub technic_api_build: String,
    pub modrinth_staging_url: String,
    pub modrinth_prod_url: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        let yaml_str = include_str!("build_config.yaml");
        serde_yaml::from_str(yaml_str).expect("Failed to parse build_config.yaml")
    }
}

impl BuildConfig {
    pub fn version_string(&self) -> String {
        format!(
            "{}.{}.{}",
            self.version_major, self.version_minor, self.version_channel
        )
    }

    pub fn printable_version_string(&self) -> String {
        format!(
            "{} {} ({} {})",
            self.launcher_display_name,
            self.version_string(),
            self.build_platform,
            std::env::consts::ARCH
        )
    }

    pub fn detect_platform() -> String {
        if cfg!(target_os = "linux") {
            "linux".to_string()
        } else if cfg!(target_os = "macos") {
            "osx64".to_string()
        } else if cfg!(target_os = "windows") {
            "win32".to_string()
        } else {
            "unknown".to_string()
        }
    }
}
