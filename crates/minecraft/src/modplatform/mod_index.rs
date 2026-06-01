use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackAuthor {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationData {
    pub id: String,
    pub platform: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtraPackData {
    pub donate: Vec<DonationData>,
    pub issues_url: String,
    pub source_url: String,
    pub wiki_url: String,
    pub discord_url: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedVersion {
    pub addon_id: String,
    pub file_id: String,
    pub version: String,
    pub version_number: String,
    pub mc_versions: Vec<String>,
    pub download_url: String,
    pub date: String,
    pub file_name: String,
    pub loaders: Vec<String>,
    pub hash_type: String,
    pub hash: String,
    pub is_preferred: bool,
    pub changelog: String,
}

impl IndexedVersion {
    pub fn new() -> Self {
        IndexedVersion {
            addon_id: String::new(),
            file_id: String::new(),
            version: String::new(),
            version_number: String::new(),
            mc_versions: Vec::new(),
            download_url: String::new(),
            date: String::new(),
            file_name: String::new(),
            loaders: Vec::new(),
            hash_type: String::new(),
            hash: String::new(),
            is_preferred: false,
            changelog: String::new(),
        }
    }
}

impl Default for IndexedVersion {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedPack {
    pub addon_id: String,
    pub provider: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub authors: Vec<ModpackAuthor>,
    pub logo_name: String,
    pub logo_url: String,
    pub website_url: String,
    pub versions_loaded: bool,
    pub versions: Vec<IndexedVersion>,
    pub extra_data_loaded: bool,
    pub extra_data: ExtraPackData,
}

impl IndexedPack {
    pub fn new() -> Self {
        IndexedPack {
            addon_id: String::new(),
            provider: String::new(),
            name: String::new(),
            slug: String::new(),
            description: String::new(),
            authors: Vec::new(),
            logo_name: String::new(),
            logo_url: String::new(),
            website_url: String::new(),
            versions_loaded: false,
            versions: Vec::new(),
            extra_data_loaded: false,
            extra_data: ExtraPackData::default(),
        }
    }
}

impl Default for IndexedPack {
    fn default() -> Self {
        Self::new()
    }
}
