pub mod agent;
pub mod assets;
pub mod clone;
pub mod component;
pub mod gradle_specifier;
pub mod import;
pub mod instance;
pub mod instance_list;
pub mod instance_task;
pub mod launch;
pub mod launch_profile;
pub mod library;
pub mod meta;
pub mod mod_update;
pub mod modplatform;
pub mod mojang_download_info;
pub mod quilt;
pub mod resolver;
pub mod resource;
pub mod resource_pack;
pub mod rule;
pub mod scripts;
pub mod server_export;
pub mod shader_pack;
pub mod update;
pub mod version_file;
pub mod version_filter_data;
pub mod world;

pub use agent::*;
pub use assets::*;
pub use clone::*;
pub use component::*;
pub use gradle_specifier::*;
pub use import::*;
pub use instance::*;
pub use launch::*;
pub use launch_profile::*;
pub use library::*;
pub use mod_update::*;
pub use mojang_download_info::*;
pub use quilt::*;
pub use resolver::*;
pub use resource_pack::*;
pub use rule::*;
pub use scripts::*;
pub use server_export::*;
pub use shader_pack::*;
pub use version_file::*;
pub use version_filter_data::*;

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub client_token: String,
    pub username: String,
    pub session: String,
    pub access_token: String,
    pub player_name: String,
    pub uuid: String,
    pub user_type: String,
    pub status: AuthSessionStatus,
    pub authlib_injector_base_url: String,
    pub auth_server_online: bool,
    pub wants_online: bool,
    pub demo: bool,
}

impl AuthSession {
    pub fn new(username: &str) -> Self {
        AuthSession {
            client_token: String::new(),
            username: username.to_string(),
            session: String::new(),
            access_token: String::new(),
            player_name: String::new(),
            uuid: String::new(),
            user_type: String::new(),
            status: AuthSessionStatus::Undetermined,
            authlib_injector_base_url: String::new(),
            auth_server_online: false,
            wants_online: false,
            demo: false,
        }
    }

    pub fn serialize_user_properties(&self) -> String {
        "{}".to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthSessionStatus {
    #[default]
    Undetermined,
    RequiresOAuth,
    RequiresPassword,
    RequiresProfileSetup,
    PlayableOffline,
    PlayableOnline,
    GoneOrMigrated,
}

#[derive(Debug, Clone)]
pub struct JavaVersion {
    string: String,
    major: i32,
    minor: i32,
    security: i32,
    parseable: bool,
    prerelease: String,
}

impl JavaVersion {
    pub fn new(version: &str) -> Self {
        let mut jv = JavaVersion {
            string: version.to_string(),
            major: 0,
            minor: 0,
            security: 0,
            parseable: false,
            prerelease: String::new(),
        };
        jv.parse();
        jv
    }

    fn parse(&mut self) {
        let s = self.string.trim();
        if s.is_empty() {
            return;
        }

        // Try "1.X.Y" format (Java <= 8)
        if let Some(stripped) = s.strip_prefix("1.") {
            let parts: Vec<&str> = stripped.splitn(3, '.').collect();
            if let Ok(m) = parts[0].parse::<i32>() {
                self.major = 1;
                self.minor = m;
                self.security = parts
                    .get(1)
                    .and_then(|p| p.parse::<i32>().ok())
                    .unwrap_or(0);
                self.parseable = true;
                return;
            }
        }

        // Try "X.Y.Z" format (Java >= 9)
        let parts: Vec<&str> = s.splitn(3, '.').collect();
        if let Ok(m) = parts[0].parse::<i32>() {
            self.major = m;
            self.minor = parts
                .get(1)
                .and_then(|p| p.parse::<i32>().ok())
                .unwrap_or(0);
            self.security = parts
                .get(2)
                .and_then(|p| p.parse::<i32>().ok())
                .unwrap_or(0);
            self.parseable = true;
        }

        // Check for prerelease
        if let Some(idx) = self.string.find(|c: char| c.is_alphabetic()) {
            self.prerelease = self.string[idx..].to_string();
        }
    }

    pub fn major(&self) -> i32 {
        self.major
    }
    pub fn minor(&self) -> i32 {
        self.minor
    }
    pub fn security(&self) -> i32 {
        self.security
    }
    pub fn is_parseable(&self) -> bool {
        self.parseable
    }

    pub fn requires_perm_gen(&self) -> bool {
        self.parseable && (self.major < 8 || (self.major == 1 && self.minor < 8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpSys {
    Linux,
    Osx,
    Windows,
    Unknown,
}

impl OpSys {
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        {
            OpSys::Linux
        }
        #[cfg(target_os = "macos")]
        {
            OpSys::Osx
        }
        #[cfg(target_os = "windows")]
        {
            OpSys::Windows
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            OpSys::Unknown
        }
    }

    pub fn classifier(&self) -> &str {
        match self {
            OpSys::Linux => "linux",
            OpSys::Osx => "osx",
            OpSys::Windows => "windows",
            OpSys::Unknown => "unknown",
        }
    }

    pub fn classifiers() -> Vec<&'static str> {
        vec!["linux", "osx", "windows"]
    }
}

impl std::fmt::Display for OpSys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.classifier())
    }
}

pub type RequireSet = Vec<MetaRequire>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct MetaRequire {
    pub uid: String,
    pub equals: Option<String>,
    pub suggests: Option<String>,
}

pub type Result<T> = std::result::Result<T, MinecraftError>;

#[derive(Debug, thiserror::Error)]
pub enum MinecraftError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("File system error: {0}")]
    Fs(#[from] kcraft_fs::FsError),
    #[error("Network error: {0}")]
    Net(#[from] kcraft_net::NetError),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Java error: {0}")]
    Java(#[from] kcraft_java::JavaError),
    #[error("Mod update error: {0}")]
    ModUpdate(String),
    #[error("Script execution error: {0}")]
    ScriptExecution(String),
    #[error("Import error: {0}")]
    Import(String),
    #[error("Backup error: {0}")]
    Backup(String),
}

impl From<serde_json::Error> for MinecraftError {
    fn from(e: serde_json::Error) -> Self {
        MinecraftError::Serialization(e.to_string())
    }
}

impl From<String> for MinecraftError {
    fn from(e: String) -> Self {
        MinecraftError::InvalidInput(e)
    }
}
pub mod cloud_sync;
