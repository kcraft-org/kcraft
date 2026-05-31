use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MojangDownloadInfo {
    pub path: Option<String>,
    pub url: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MojangLibraryDownloadInfo {
    pub artifact: Option<MojangDownloadInfo>,
    pub classifiers: Option<HashMap<String, MojangDownloadInfo>>,
}

impl MojangLibraryDownloadInfo {
    pub fn get_download_info(&self, classifier: Option<&str>) -> Option<&MojangDownloadInfo> {
        match classifier {
            Some(c) => self.classifiers.as_ref()?.get(c),
            None => self.artifact.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MojangAssetIndexInfo {
    pub id: String,
    pub sha1: Option<String>,
    pub size: Option<i64>,
    pub total_size: Option<i64>,
    pub url: Option<String>,
    pub known: bool,
}

impl MojangAssetIndexInfo {
    pub fn new(id: String) -> Self {
        let url = if id == "legacy" {
            "https://piston-meta.mojang.com/mc/assets/legacy/c0fd82e8ce9fbc93119e40d96d5a4e62cfa3f729/legacy.json".to_string()
        } else {
            format!("https://s3.amazonaws.com/Minecraft.Download/indexes/{}.json", id)
        };
        MojangAssetIndexInfo {
            id,
            sha1: None,
            size: None,
            total_size: None,
            url: Some(url),
            known: false,
        }
    }
}
