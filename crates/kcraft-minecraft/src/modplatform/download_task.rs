use std::fs;

use crate::modplatform::mod_index::{IndexedPack, IndexedVersion};
use crate::modplatform::Provider;

#[derive(Debug, thiserror::Error)]
pub enum ModDownloadError {
    #[error("Network error: {0}")]
    Network(#[from] kcraft_net::NetError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Download failed: HTTP {0}")]
    DownloadFailed(u16),
    #[error("No download URL available")]
    NoDownloadUrl,
}

pub struct ModDownloadTask {
    pack: IndexedPack,
    version: IndexedVersion,
    mods_dir: std::path::PathBuf,
}

impl ModDownloadTask {
    pub fn new(pack: IndexedPack, version: IndexedVersion, mods_dir: impl Into<std::path::PathBuf>) -> Self {
        ModDownloadTask {
            pack,
            version,
            mods_dir: mods_dir.into(),
        }
    }

    pub fn from_indexed(pack: IndexedPack, version: IndexedVersion, mods_dir: impl Into<std::path::PathBuf>) -> Self {
        Self::new(pack, version, mods_dir)
    }

    pub fn version(&self) -> &IndexedVersion {
        &self.version
    }

    pub fn pack(&self) -> &IndexedPack {
        &self.pack
    }

    pub fn provider(&self) -> Provider {
        Provider::from_name(&self.pack.provider)
    }

    pub async fn execute(&self) -> std::result::Result<std::path::PathBuf, ModDownloadError> {
        if self.version.download_url.is_empty() {
            return Err(ModDownloadError::NoDownloadUrl);
        }

        let url = url::Url::parse(&self.version.download_url)
            .map_err(|_| ModDownloadError::InvalidUrl(self.version.download_url.clone()))?;

        let file_name = if self.version.file_name.is_empty() {
            url.path_segments()
                .and_then(|mut s| s.next_back())
                .unwrap_or("mod.jar")
                .to_string()
        } else {
            self.version.file_name.clone()
        };

        let dest_path = self.mods_dir.join(&file_name);
        
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        self.version.download_url.hash(&mut hasher);
        let hash = hasher.finish();

        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("kcraft")
            .join("mods");

        if !cache_dir.exists() {
            let _ = fs::create_dir_all(&cache_dir);
        }

        let cache_file = cache_dir.join(format!("{:x}_{}", hash, file_name));

        if !cache_file.exists() {
            let response = reqwest::get(url.as_str()).await
                .map_err(|e| ModDownloadError::Network(kcraft_net::NetError::Network(e.to_string())))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ModDownloadError::DownloadFailed(status.as_u16()));
            }

            let bytes = response.bytes().await
                .map_err(|e| ModDownloadError::Network(kcraft_net::NetError::Network(e.to_string())))?;

            let tmp_cache = cache_file.with_extension("tmp");
            fs::write(&tmp_cache, &bytes)?;
            fs::rename(&tmp_cache, &cache_file)?;
        }

        if let Some(parent) = dest_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        // Attempt hardlink, fallback to copy
        if fs::hard_link(&cache_file, &dest_path).is_err() {
            fs::copy(&cache_file, &dest_path)?;
        }

        Ok(dest_path)
    }
}
