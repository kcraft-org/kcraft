use std::path::{Path, PathBuf};

use kcraft_net::{Download, NetMode};
use serde::Deserialize;
use url::Url;

use crate::JavaError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JavaDistributions {
    Adoptium,
    Zulu,
    GraalVM,
    Microsoft,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JavaRelease {
    pub version: crate::JavaVersion,
    pub url: String,
    pub filename: String,
    pub checksum: String,
    pub os: String,
    pub arch: String,
}

pub struct JavaDownloader;

impl JavaDownloader {
    pub async fn fetch_available_versions(major: i32) -> Result<Vec<JavaRelease>, JavaError> {
        let api_url = format!(
            "https://api.adoptium.net/v3/assets/feature_releases/{}/ga",
            major
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&api_url)
            .send()
            .await
            .map_err(|e| JavaError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(JavaError::DownloadFailed(format!(
                "Adoptium API returned status {}",
                response.status()
            )));
        }

        let releases: Vec<AdoptiumRelease> = response.json().await.map_err(|e| {
            JavaError::DownloadFailed(format!("Failed to parse API response: {}", e))
        })?;

        let current_os = current_os_classifier();
        let current_arch = current_arch_classifier();

        let mut result = Vec::new();
        for release in &releases {
            let version_str = release
                .version_data
                .semver
                .as_deref()
                .or(release.version_data.openjdk_version.as_deref())
                .map(|s| s.to_string())
                .unwrap_or_else(|| release.version_data.major.to_string());
            let java_version = crate::JavaVersion::new(&version_str);

            if let Some(ref binaries) = release.binaries {
                for binary in binaries {
                    let bin_os = binary.os.as_deref().unwrap_or("");
                    let bin_arch = binary.architecture.as_deref().unwrap_or("");

                    let os_match =
                        bin_os == current_os || (current_os == "mac" && bin_os == "macos");
                    let arch_match = bin_arch == current_arch;
                    let is_jdk = binary.image_type.as_deref() == Some("jdk");

                    if os_match && arch_match && is_jdk {
                        let dl_url = binary
                            .package
                            .as_ref()
                            .and_then(|p| p.link.as_ref())
                            .cloned()
                            .unwrap_or_default();
                        let filename = binary
                            .package
                            .as_ref()
                            .and_then(|p| p.name.as_ref())
                            .cloned()
                            .unwrap_or_default();
                        let checksum = binary
                            .package
                            .as_ref()
                            .and_then(|p| p.checksum.as_ref())
                            .cloned()
                            .unwrap_or_default();

                        result.push(JavaRelease {
                            version: java_version.clone(),
                            url: dl_url,
                            filename,
                            checksum,
                            os: bin_os.to_string(),
                            arch: bin_arch.to_string(),
                        });
                    }
                }
            }
        }

        Ok(result)
    }

    pub async fn download_and_install(
        major: i32,
        install_dir: &Path,
    ) -> Result<PathBuf, JavaError> {
        let releases = Self::fetch_available_versions(major).await?;

        if releases.is_empty() {
            return Err(JavaError::DownloadFailed(format!(
                "No releases found for Java {}",
                major
            )));
        }

        let release = releases.into_iter().next().unwrap();

        let download_url = Url::parse(&release.url)
            .map_err(|e| JavaError::DownloadFailed(format!("Invalid URL: {}", e)))?;

        let archive_dir = install_dir.join("archives");
        std::fs::create_dir_all(&archive_dir).map_err(JavaError::Io)?;

        let archive_path = archive_dir.join(&release.filename);
        let mut download = Download::make_file(download_url, archive_path.clone());

        download
            .execute(NetMode::Online)
            .await
            .map_err(|e| JavaError::DownloadFailed(e.to_string()))?;

        extract_archive(&archive_path, install_dir).map_err(JavaError::ExtractionFailed)?;

        let _ = std::fs::remove_file(&archive_path);
        let _ = std::fs::remove_dir(&archive_dir);

        let java_binary = find_java_binary(install_dir).ok_or(JavaError::NotFound)?;

        Ok(java_binary)
    }
}

fn extract_archive(archive_path: &Path, dest: &Path) -> std::result::Result<(), String> {
    let file =
        std::fs::File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(dest)
        .map_err(|e| format!("Failed to extract archive: {}", e))?;
    Ok(())
}

fn find_java_binary(dir: &Path) -> Option<PathBuf> {
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let filename = path.file_name()?.to_str()?;
        if !filename.eq_ignore_ascii_case("java") && !filename.eq_ignore_ascii_case("java.exe") {
            continue;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = path.metadata() {
                if meta.permissions().mode() & 0o111 != 0 {
                    return Some(path.to_path_buf());
                }
            }
        }
        #[cfg(not(unix))]
        {
            return Some(path.to_path_buf());
        }
    }
    None
}

fn current_os_classifier() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(target_os = "macos")]
    {
        "mac"
    }
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unknown"
    }
}

fn current_arch_classifier() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    }
}

pub fn needs_java(minecraft_version: &str) -> i32 {
    let parts: Vec<&str> = minecraft_version.split('.').collect();
    let major: i32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: i32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch: i32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    if major < 1 {
        return 8;
    }

    match (major, minor, patch) {
        (1, n, _) if n < 12 => 8,
        (1, 12..=16, _) => 8,
        (1, 17, p) if p <= 1 => 16,
        (1, 18..=20, p) if p <= 4 => 17,
        _ => 21,
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AdoptiumRelease {
    version_data: AdoptiumVersionData,
    binaries: Option<Vec<AdoptiumBinary>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AdoptiumVersionData {
    major: i32,
    semver: Option<String>,
    openjdk_version: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AdoptiumBinary {
    os: Option<String>,
    architecture: Option<String>,
    image_type: Option<String>,
    package: Option<AdoptiumPackage>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AdoptiumPackage {
    link: Option<String>,
    name: Option<String>,
    checksum: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_java_pre_112() {
        assert_eq!(needs_java("1.7.10"), 8);
        assert_eq!(needs_java("1.8.9"), 8);
        assert_eq!(needs_java("1.11.2"), 8);
    }

    #[test]
    fn test_needs_java_112_to_116() {
        assert_eq!(needs_java("1.12.2"), 8);
        assert_eq!(needs_java("1.16.5"), 8);
    }

    #[test]
    fn test_needs_java_117() {
        assert_eq!(needs_java("1.17"), 16);
        assert_eq!(needs_java("1.17.1"), 16);
    }

    #[test]
    fn test_needs_java_118_to_1204() {
        assert_eq!(needs_java("1.18"), 17);
        assert_eq!(needs_java("1.19.2"), 17);
        assert_eq!(needs_java("1.20.4"), 17);
    }

    #[test]
    fn test_needs_java_1205_plus() {
        assert_eq!(needs_java("1.20.5"), 21);
        assert_eq!(needs_java("1.21"), 21);
        assert_eq!(needs_java("1.21.1"), 21);
    }

    #[test]
    fn test_needs_java_unknown() {
        assert_eq!(needs_java("2.0"), 21);
    }
}
