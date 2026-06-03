use std::path::Path;

use serde::Deserialize;

use crate::MinecraftError;

pub const QUILT_META_URL: &str = "https://meta.quiltmc.org/v3";

#[derive(Debug, Clone, Deserialize)]
pub struct QuiltLoaderVersion {
    pub separator: Option<String>,
    pub build: Option<i32>,
    pub maven: Option<String>,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuiltMetaEntry {
    pub loader: QuiltLoaderVersion,
    pub intermediary: QuiltIntermediary,
    pub launcher_meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuiltIntermediary {
    pub maven: Option<String>,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct QuiltInstaller {
    mc_version: String,
    loader_version: String,
}

impl QuiltInstaller {
    pub fn new(mc_version: &str, loader_version: &str) -> Self {
        QuiltInstaller {
            mc_version: mc_version.to_string(),
            loader_version: loader_version.to_string(),
        }
    }

    pub fn fetch_loader_versions(
        mc_version: &str,
    ) -> std::result::Result<Vec<QuiltLoaderVersion>, MinecraftError> {
        let url = format!("{}/versions/loader/{}", QUILT_META_URL, mc_version);
        let resp = reqwest::blocking::get(&url)
            .map_err(|e| MinecraftError::Net(kcraft_net::NetError::Network(e.to_string())))?;
        if !resp.status().is_success() {
            return Err(MinecraftError::Net(kcraft_net::NetError::HttpError(
                resp.status().as_u16(),
                format!("Failed to fetch Quilt loader versions for {}", mc_version),
            )));
        }
        let entries: Vec<QuiltMetaEntry> = resp
            .json()
            .map_err(|e| MinecraftError::Serialization(e.to_string()))?;
        Ok(entries.into_iter().map(|e| e.loader).collect())
    }

    pub fn install(&self, instance_root: &Path) -> std::result::Result<(), MinecraftError> {
        let url = format!(
            "{}/versions/loader/{}/{}/profile/json",
            QUILT_META_URL, self.mc_version, self.loader_version
        );
        let resp = reqwest::blocking::get(&url)
            .map_err(|e| MinecraftError::Net(kcraft_net::NetError::Network(e.to_string())))?;
        if !resp.status().is_success() {
            return Err(MinecraftError::Net(kcraft_net::NetError::HttpError(
                resp.status().as_u16(),
                format!(
                    "Failed to fetch Quilt profile for {} {}",
                    self.mc_version, self.loader_version
                ),
            )));
        }
        let profile_json: serde_json::Value = resp
            .json()
            .map_err(|e| MinecraftError::Serialization(e.to_string()))?;

        let libraries_dir = instance_root.join("libraries");
        if let Some(libraries) = profile_json.get("libraries").and_then(|v| v.as_array()) {
            for lib in libraries {
                if let Some(downloads) = lib.get("downloads").and_then(|v| v.as_object()) {
                    if let Some(artifact) = downloads.get("artifact") {
                        let path = artifact.get("path").and_then(|v| v.as_str()).unwrap_or("");
                        let url = artifact.get("url").and_then(|v| v.as_str()).unwrap_or("");
                        if !path.is_empty() && !url.is_empty() {
                            let dest = libraries_dir.join(path);
                            if !dest.exists() {
                                if let Some(parent) = dest.parent() {
                                    kcraft_fs::ensure_folder_exists(parent)?;
                                }
                                let resp = reqwest::blocking::get(url).map_err(|e| {
                                    MinecraftError::Net(kcraft_net::NetError::Network(
                                        e.to_string(),
                                    ))
                                })?;
                                if !resp.status().is_success() {
                                    return Err(MinecraftError::Net(
                                        kcraft_net::NetError::HttpError(
                                            resp.status().as_u16(),
                                            format!("Failed to download {}", url),
                                        ),
                                    ));
                                }
                                let bytes = resp.bytes().map_err(|e| {
                                    MinecraftError::Net(kcraft_net::NetError::Network(
                                        e.to_string(),
                                    ))
                                })?;
                                kcraft_fs::write(&dest, &bytes)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn mc_version(&self) -> &str {
        &self.mc_version
    }

    pub fn loader_version(&self) -> &str {
        &self.loader_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quilt_meta_url() {
        assert_eq!(QUILT_META_URL, "https://meta.quiltmc.org/v3");
    }
}
