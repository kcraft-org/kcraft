pub mod atlauncher;
pub mod download_task;
pub mod flame;
pub mod ftb;
pub mod hash_utils;
pub mod mod_index;
pub mod modrinth;
pub mod packwiz;
pub mod technic;

pub use flame::*;
pub use hash_utils::*;
pub use mod_index::*;
pub use modrinth::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Provider {
    Modrinth,
    Flame,
}

impl Provider {
    pub fn name(&self) -> &'static str {
        match self {
            Provider::Modrinth => "modrinth",
            Provider::Flame => "curseforge",
        }
    }

    pub fn readable_name(&self) -> &'static str {
        match self {
            Provider::Modrinth => "Modrinth",
            Provider::Flame => "CurseForge",
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "modrinth" => Provider::Modrinth,
            "curseforge" | "flame" => Provider::Flame,
            _ => Provider::Modrinth,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModLoaderType {
    Unspecified = 0,
    Forge = 1,
    Cauldron = 2,
    LiteLoader = 4,
    Fabric = 8,
    Quilt = 16,
    NeoForge = 32,
}

impl ModLoaderType {
    pub fn to_string(&self) -> &'static str {
        match self {
            ModLoaderType::Forge => "forge",
            ModLoaderType::NeoForge => "neoforge",
            ModLoaderType::Cauldron => "cauldron",
            ModLoaderType::LiteLoader => "liteloader",
            ModLoaderType::Fabric => "fabric",
            ModLoaderType::Quilt => "quilt",
            ModLoaderType::Unspecified => "",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchArgs {
    pub offset: i32,
    pub search: String,
    pub sorting: String,
    pub loaders: Vec<ModLoaderType>,
    pub versions: Vec<String>,
}

impl Default for SearchArgs {
    fn default() -> Self {
        SearchArgs {
            offset: 0,
            search: String::new(),
            sorting: "relevance".to_string(),
            loaders: Vec::new(),
            versions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VersionSearchArgs {
    pub addon_id: String,
    pub mc_versions: Vec<String>,
    pub loaders: Vec<ModLoaderType>,
}
