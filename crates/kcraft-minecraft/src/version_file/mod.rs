use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agent::Agent;
use crate::library::Library;
use crate::mojang_download_info::MojangAssetIndexInfo;
use crate::RequireSet;

mod mojang_serde;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionFile {
    pub order: i32,
    pub name: Option<String>,
    pub uid: Option<String>,
    pub version: Option<String>,
    pub depends_on_minecraft_version: Option<String>,
    pub minimum_launcher_version: i32,
    pub minecraft_version: Option<String>,
    pub main_class: Option<String>,
    pub applet_class: Option<String>,
    pub minecraft_arguments: Option<String>,
    pub additional_jvm_arguments: Vec<String>,
    pub compatible_java_majors: Vec<i32>,
    pub type_: Option<String>,
    pub release_time: Option<chrono::DateTime<chrono::Utc>>,
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
    pub assets: Option<String>,
    pub add_tweakers: Vec<String>,
    pub libraries: Vec<Library>,
    pub maven_files: Vec<Library>,
    pub agents: Vec<Agent>,
    pub main_jar: Option<Library>,
    pub traits: Vec<String>,
    pub jar_mods: Vec<Library>,
    pub mods: Vec<Library>,
    pub required: RequireSet,
    pub conflicts: RequireSet,
    pub volatile: bool,
    pub mojang_downloads: HashMap<String, crate::mojang_download_info::MojangDownloadInfo>,
    pub mojang_asset_index: Option<MojangAssetIndexInfo>,
}

impl VersionFile {
    pub fn new() -> Self {
        VersionFile {
            order: 0,
            name: None,
            uid: None,
            version: None,
            depends_on_minecraft_version: None,
            minimum_launcher_version: -1,
            minecraft_version: None,
            main_class: None,
            applet_class: None,
            minecraft_arguments: None,
            additional_jvm_arguments: Vec::new(),
            compatible_java_majors: Vec::new(),
            type_: None,
            release_time: None,
            update_time: None,
            assets: None,
            add_tweakers: Vec::new(),
            libraries: Vec::new(),
            maven_files: Vec::new(),
            agents: Vec::new(),
            main_jar: None,
            traits: Vec::new(),
            jar_mods: Vec::new(),
            mods: Vec::new(),
            required: Vec::new(),
            conflicts: Vec::new(),
            volatile: false,
            mojang_downloads: HashMap::new(),
            mojang_asset_index: None,
        }
    }
}

impl Default for VersionFile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MojangVersionFormat;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneSixVersionFormat;

impl MojangVersionFormat {
    pub fn from_json(value: serde_json::Value) -> Option<VersionFile> {
        mojang_serde::from_json(value)
    }
}

impl OneSixVersionFormat {
    pub fn from_json(_value: serde_json::Value) -> Option<VersionFile> {
        tracing::warn!("OneSix format parsing is not yet implemented");
        None
    }
}
