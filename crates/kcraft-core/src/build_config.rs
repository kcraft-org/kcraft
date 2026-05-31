use std::sync::LazyLock;

pub static BUILD_CONFIG: LazyLock<BuildConfig> = LazyLock::new(BuildConfig::default);

#[derive(Debug, Clone)]
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
        BuildConfig {
            launcher_name: "kcraft".to_string(),
            launcher_display_name: "KCraft".to_string(),
            launcher_copyright: "KCraft Contributors".to_string(),
            launcher_domain: "github.com/kcraft-org".to_string(),
            launcher_config_file: "kcraft.cfg".to_string(),
            version_major: 1,
            version_minor: 0,
            version_channel: "0".to_string(),
            updater_enabled: false,
            build_platform: Self::detect_platform(),
            user_agent: "KCraft/1.0".to_string(),
            user_agent_uncached: "KCraft/1.0".to_string(),
            meta_url: "https://raw.githubusercontent.com/kcraft-org/kcraft-meta/master/v1/"
                .to_string(),
            msa_client_id: String::new(),
            flame_api_key: String::new(),
            imgur_client_id: String::new(),
            news_rss_url: "https://github.com/kcraft-org/kcraft/releases.atom".to_string(),
            news_open_url: "https://github.com/kcraft-org/kcraft/releases".to_string(),
            help_url: "https://github.com/kcraft-org/kcraft/issues".to_string(),
            bug_tracker_url: "https://github.com/kcraft-org/kcraft/issues".to_string(),
            translations_url: "https://github.com/kcraft-org/kcraft".to_string(),
            discord_url: "https://github.com/kcraft-org/kcraft/discussions".to_string(),
            subreddit_url: "https://github.com/kcraft-org/kcraft/discussions".to_string(),
            resource_base: "https://resources.download.minecraft.net/".to_string(),
            library_base: "https://libraries.minecraft.net/".to_string(),
            auth_base: "https://authserver.mojang.com/".to_string(),
            imgur_base_url: "https://api.imgur.com/3/".to_string(),
            fmllibs_base_url: "https://maven.minecraftforge.net/".to_string(),
            modpacksch_api_base_url: "https://api.modpacks.ch/".to_string(),
            legacy_ftb_cdn_base_url: "https://dist.creeper.host/FTB2/".to_string(),
            atl_download_server_url: "https://download.nodecdn.net/containers/atl/".to_string(),
            atl_api_base_url: "https://api.atlauncher.com/v1/".to_string(),
            technic_api_base_url: "https://api.technicpack.net/".to_string(),
            technic_api_build: "multimc".to_string(),
            modrinth_staging_url: "https://staging-api.modrinth.com/v2".to_string(),
            modrinth_prod_url: "https://api.modrinth.com/v2".to_string(),
        }
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
