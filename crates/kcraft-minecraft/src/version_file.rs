use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agent::Agent;
use crate::library::Library;
use crate::mojang_download_info::MojangAssetIndexInfo;
use crate::RequireSet;

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

mod mojang_serde {
    use crate::rule::rules_from_json;
    use crate::Library;
    use crate::VersionFile;
    use serde::{self, Deserialize};

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct MojangVersionFileJson {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        #[serde(rename = "minecraftArguments")]
        minecraft_arguments: Option<String>,
        #[serde(default)]
        #[serde(rename = "mainClass")]
        main_class: Option<String>,
        #[serde(default)]
        #[serde(rename = "minimumLauncherVersion")]
        minimum_launcher_version: Option<i32>,
        #[serde(default)]
        #[serde(rename = "releaseTime")]
        release_time: Option<String>,
        #[serde(default)]
        #[serde(rename = "type")]
        type_: Option<String>,
        #[serde(default)]
        libraries: Option<Vec<serde_json::Value>>,
        #[serde(default)]
        #[serde(rename = "compatibleJavaMajors")]
        compatible_java_majors: Option<Vec<i32>>,
        #[serde(default)]
        assets: Option<String>,
        #[serde(default)]
        downloads: Option<serde_json::Value>,
        #[serde(default)]
        #[serde(rename = "assetIndex")]
        asset_index: Option<serde_json::Value>,
        #[serde(default)]
        #[serde(rename = "javaVersion")]
        java_version: Option<serde_json::Value>,
        #[serde(default)]
        arguments: Option<serde_json::Value>,
    }

    #[derive(Deserialize)]
    struct MojangLibraryJson {
        name: String,
        #[serde(default)]
        downloads: Option<serde_json::Value>,
        #[serde(default)]
        rules: Option<Vec<serde_json::Value>>,
        #[serde(default)]
        natives: Option<std::collections::HashMap<String, String>>,
        #[serde(default)]
        extract: Option<serde_json::Value>,
    }

    #[derive(Deserialize)]
    struct MojangDownloadsJson {
        #[serde(default)]
        artifact: Option<serde_json::Value>,
        #[serde(default)]
        classifiers: Option<std::collections::HashMap<String, serde_json::Value>>,
    }

    #[derive(Deserialize)]
    struct MojangArtifactJson {
        path: Option<String>,
        url: Option<String>,
        sha1: Option<String>,
        size: Option<i64>,
    }

    fn parse_library(val: &serde_json::Value) -> Option<Library> {
        let json: MojangLibraryJson = serde_json::from_value(val.clone()).ok()?;
        let mut lib = Library::new(json.name.as_str());

        if let Some(ref rules) = json.rules {
            lib.rules = rules_from_json(rules);
            lib.apply_rules = true;
        }

        lib.native_classifiers = json.natives.unwrap_or_default();

        if let Some(ref extract) = json.extract {
            if let Some(exclude) = extract.get("exclude").and_then(|v| v.as_array()) {
                lib.has_excludes = true;
                for val in exclude {
                    if let Some(s) = val.as_str() {
                        lib.extract_excludes.push(s.to_string());
                    }
                }
            }
        }

        if let Some(ref downloads) = json.downloads {
            let dl_json: MojangDownloadsJson = serde_json::from_value(downloads.clone()).ok()?;
            let mut dl_info = crate::mojang_download_info::MojangLibraryDownloadInfo::default();

            if let Some(ref artifact) = dl_json.artifact {
                let art: MojangArtifactJson = serde_json::from_value(artifact.clone()).ok()?;
                dl_info.artifact = Some(crate::mojang_download_info::MojangDownloadInfo {
                    path: art.path,
                    url: art.url,
                    sha1: art.sha1,
                    size: art.size,
                });
            }

            if let Some(ref classifiers) = dl_json.classifiers {
                let mut cls_map = std::collections::HashMap::new();
                for (key, val) in classifiers {
                    let art: MojangArtifactJson = serde_json::from_value(val.clone()).ok()?;
                    cls_map.insert(
                        key.clone(),
                        crate::mojang_download_info::MojangDownloadInfo {
                            path: art.path,
                            url: art.url,
                            sha1: art.sha1,
                            size: art.size,
                        },
                    );
                }
                dl_info.classifiers = Some(cls_map);
            }

            lib.mojang_downloads = Some(dl_info);
        }

        Some(lib)
    }

    pub fn from_json(value: serde_json::Value) -> Option<VersionFile> {
        let json: MojangVersionFileJson = serde_json::from_value(value).ok()?;
        let mut vf = VersionFile::new();

        vf.minecraft_version = json.id;
        vf.minecraft_arguments = json.minecraft_arguments;
        vf.main_class = json.main_class;
        vf.minimum_launcher_version = json.minimum_launcher_version.unwrap_or(-1);
        vf.release_time = json.release_time.and_then(|t| {
            chrono::DateTime::parse_from_rfc3339(&t)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });
        vf.type_ = json.type_;
        vf.assets = json.assets;
        vf.compatible_java_majors = json.compatible_java_majors.unwrap_or_default();

        if let Some(ref libs) = json.libraries {
            for lib_val in libs {
                if let Some(lib) = parse_library(lib_val) {
                    vf.libraries.push(lib);
                }
            }
        }

        Some(vf)
    }
}

impl MojangVersionFormat {
    pub fn from_json(value: serde_json::Value) -> Option<VersionFile> {
        mojang_serde::from_json(value)
    }
}

impl OneSixVersionFormat {
    pub fn from_json(_value: serde_json::Value) -> Option<VersionFile> {
        // TODO: implement OneSix format parsing
        None
    }
}
