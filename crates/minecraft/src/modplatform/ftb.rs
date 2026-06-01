use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpacksCHModpack {
    pub id: i32,
    pub name: String,
    pub synopsis: String,
    pub description: String,
    pub pack_type: String,
    pub featured: bool,
    pub installs: i32,
    pub plays: i32,
    pub updated: i64,
    pub refreshed: i64,
    pub art: Vec<Art>,
    pub authors: Vec<Author>,
    pub versions: Vec<VersionInfo>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpacksCHVersion {
    pub id: i32,
    pub parent: i32,
    pub name: String,
    pub pack_type: String,
    pub installs: i32,
    pub plays: i32,
    pub updated: i64,
    pub refreshed: i64,
    pub specs: Specs,
    pub targets: Vec<VersionTarget>,
    pub files: Vec<VersionFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Art {
    pub url: String,
    pub r#type: String,
    pub width: i32,
    pub height: i32,
    pub compression: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub url: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: i32,
    pub name: String,
    pub r#type: String,
    pub updated: i64,
    pub installs: i32,
    pub plays: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specs {
    pub minimum: i32,
    pub recommended: i32,
    pub maximum: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionTarget {
    pub id: i32,
    pub r#type: String,
    pub name: String,
    pub version: String,
    pub updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionFile {
    pub id: i32,
    pub file_type: String,
    pub path: String,
    pub name: String,
    pub version: String,
    pub url: String,
    pub sha1: String,
    pub size: i32,
    pub client_only: bool,
    pub server_only: bool,
    pub optional: bool,
    pub updated: i64,
    pub curseforge: Option<CurseForgeRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeRef {
    pub project: i32,
    pub file: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyFTBModpack {
    pub name: String,
    pub description: String,
    pub author: String,
    pub old_versions: Vec<String>,
    pub current_version: String,
    pub mc_version: String,
    pub mods: String,
    pub logo: String,
    pub dir: String,
    pub file: String,
    pub bugged: bool,
    pub broken: bool,
    pub pack_code: String,
}
