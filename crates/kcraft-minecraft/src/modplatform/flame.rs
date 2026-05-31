use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modplatform::mod_index::{IndexedPack, IndexedVersion, ModpackAuthor};
use crate::modplatform::Provider;

pub const FLAME_BASE_URL: &str = "https://api.curseforge.com/v1";

pub fn flame_load_indexed_pack(obj: &serde_json::Value) -> IndexedPack {
    let mut pack = IndexedPack::new();
    pack.provider = Provider::Flame.name().to_string();

    if let Some(id) = obj.get("id").and_then(|v| v.as_i64()) {
        pack.addon_id = id.to_string();
    }
    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
        pack.name = name.to_string();
    }
    if let Some(slug) = obj.get("slug").and_then(|v| v.as_str()) {
        pack.slug = slug.to_string();
    }
    if let Some(summary) = obj.get("summary").and_then(|v| v.as_str()) {
        pack.description = summary.to_string();
    }

    if let Some(links) = obj.get("links").and_then(|v| v.as_object()) {
        if let Some(url) = links.get("websiteUrl").and_then(|v| v.as_str()) {
            pack.website_url = url.to_string();
        }
        if let Some(body) = obj.get("body").and_then(|v| v.as_str()) {
            pack.extra_data.body = body.to_string();
        }
    }

    if let Some(logo) = obj.get("logo").and_then(|v| v.as_object()) {
        if let Some(title) = logo.get("title").and_then(|v| v.as_str()) {
            pack.logo_name = title.to_string();
        }
        if let Some(url) = logo.get("thumbnailUrl").and_then(|v| v.as_str()) {
            pack.logo_url = url.to_string();
        }
    }

    if let Some(authors) = obj.get("authors").and_then(|v| v.as_array()) {
        for author in authors {
            let name = author.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let url = author.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            pack.authors.push(ModpackAuthor { name, url });
        }
    }

    pack
}

pub fn flame_load_body(obj: &serde_json::Value) -> String {
    obj.get("body").and_then(|v| v.as_str()).unwrap_or("").to_string()
}

pub fn flame_load_indexed_pack_version(obj: &serde_json::Value, load_changelog: bool) -> IndexedVersion {
    let mut ver = IndexedVersion::new();

    if let Some(id) = obj.get("modId").and_then(|v| v.as_i64()) {
        ver.addon_id = id.to_string();
    }
    if let Some(id) = obj.get("id").and_then(|v| v.as_i64()) {
        ver.file_id = id.to_string();
    }
    if let Some(date) = obj.get("fileDate").and_then(|v| v.as_str()) {
        ver.date = date.to_string();
    }
    if let Some(name) = obj.get("displayName").and_then(|v| v.as_str()) {
        ver.version = name.to_string();
    }
    if let Some(url) = obj.get("downloadUrl").and_then(|v| v.as_str()) {
        ver.download_url = url.to_string();
    }
    if let Some(name) = obj.get("fileName").and_then(|v| v.as_str()) {
        ver.file_name = name.to_string();
    }

    if let Some(versions) = obj.get("gameVersions").and_then(|v| v.as_array()) {
        for v in versions {
            if let Some(s) = v.as_str() {
                if s.contains('.') {
                    ver.mc_versions.push(s.to_string());
                }
            }
        }
    }

    if let Some(hashes) = obj.get("hashes").and_then(|v| v.as_array()) {
        for hash in hashes {
            if let Some(algo) = hash.get("algo").and_then(|v| v.as_i64()) {
                if algo == 1 {
                    ver.hash_type = "sha1".to_string();
                } else if algo == 2 {
                    ver.hash_type = "md5".to_string();
                } else {
                    continue;
                }
                if let Some(val) = hash.get("value").and_then(|v| v.as_str()) {
                    ver.hash = val.to_string();
                }
                break;
            }
        }
    }

    if load_changelog {
        if let Some(changelog) = obj.get("changelog").and_then(|v| v.as_str()) {
            ver.changelog = changelog.to_string();
        }
    }

    ver
}

pub fn flame_load_indexed_pack_versions(versions: &mut IndexedPack, arr: &[serde_json::Value]) {
    for obj in arr {
        let ver = flame_load_indexed_pack_version(obj, false);
        versions.versions.push(ver);
    }
    versions.versions_loaded = true;
}

pub fn flame_load_manifest(filepath: &str) -> Result<FlameManifest, String> {
    let content = std::fs::read_to_string(filepath)
        .map_err(|e| format!("Cannot read manifest: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse manifest: {}", e))?;

    let mut manifest = FlameManifest::new();

    if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
        manifest.name = name.to_string();
    }
    if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
        manifest.version = version.to_string();
    }
    if let Some(author) = json.get("author").and_then(|v| v.as_str()) {
        manifest.author = author.to_string();
    }
    if let Some(overrides) = json.get("overrides").and_then(|v| v.as_str()) {
        manifest.overrides = overrides.to_string();
    }

    if let Some(mc) = json.get("minecraft").and_then(|v| v.as_object()) {
        if let Some(ver) = mc.get("version").and_then(|v| v.as_str()) {
            manifest.minecraft_version = ver.to_string();
        }
        if let Some(loaders) = mc.get("modLoaders").and_then(|v| v.as_array()) {
            for loader in loaders {
                if let Some(id) = loader.get("id").and_then(|v| v.as_str()) {
                    manifest.mod_loaders.push(id.to_string());
                }
            }
        }
    }

    if let Some(files) = json.get("files").and_then(|v| v.as_array()) {
        for file in files {
            let project_id = file.get("projectID").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let file_id = file.get("fileID").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let required = file.get("required").and_then(|v| v.as_bool()).unwrap_or(true);

            let mut pack_file = FlamePackFile::new(project_id, file_id);
            pack_file.required = required;
            manifest.files.insert(manifest.files.len() as i32, pack_file);
        }
    }

    manifest.is_loaded = true;
    Ok(manifest)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlamePackFile {
    pub project_id: i32,
    pub file_id: i32,
    pub required: bool,
    pub hash: String,
    pub website_url: String,
    pub resolved: bool,
    pub file_name: String,
    pub url: String,
    pub target_folder: String,
}

impl FlamePackFile {
    pub fn new(project_id: i32, file_id: i32) -> Self {
        FlamePackFile {
            project_id, file_id,
            required: true,
            hash: String::new(),
            website_url: String::new(),
            resolved: false,
            file_name: String::new(),
            url: String::new(),
            target_folder: "mods".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameManifest {
    pub manifest_type: String,
    pub manifest_version: i32,
    pub minecraft_version: String,
    pub mod_loaders: Vec<String>,
    pub name: String,
    pub version: String,
    pub author: String,
    pub files: HashMap<i32, FlamePackFile>,
    pub overrides: String,
    pub is_loaded: bool,
}

impl FlameManifest {
    pub fn new() -> Self {
        FlameManifest {
            manifest_type: "minecraftModpack".to_string(),
            manifest_version: 1,
            minecraft_version: String::new(),
            mod_loaders: Vec::new(),
            name: String::new(),
            version: String::new(),
            author: String::new(),
            files: HashMap::new(),
            overrides: "overrides".to_string(),
            is_loaded: false,
        }
    }
}

impl Default for FlameManifest {
    fn default() -> Self { Self::new() }
}

pub fn flame_api_search_url(args: &crate::modplatform::SearchArgs) -> String {
    let mut url = format!("{}/mods/search?gameId=432&classId=6&index={}&pageSize=25",
                          FLAME_BASE_URL, args.offset);
    if !args.search.is_empty() {
        url.push_str(&format!("&searchFilter={}", urlencoding(&args.search)));
    }
    if !args.sorting.is_empty() {
        let sort_field = match args.sorting.as_str() {
            "popularity" => "2",
            "last_updated" => "3",
            "name" => "4",
            "downloads" => "6",
            _ => "2",
        };
        url.push_str(&format!("&sortField={}", sort_field));
    }
    if let Some(loader) = args.loaders.first() {
        let loader_id = match loader {
            crate::modplatform::ModLoaderType::Forge => "1",
            crate::modplatform::ModLoaderType::Fabric | crate::modplatform::ModLoaderType::Quilt => "4",
            crate::modplatform::ModLoaderType::NeoForge => "6",
            _ => "",
        };
        if !loader_id.is_empty() {
            url.push_str(&format!("&modLoaderType={}", loader_id));
        }
    }
    if let Some(ver) = args.versions.first() {
        url.push_str(&format!("&gameVersion={}", urlencoding(ver)));
    }
    url
}

pub fn flame_mod_info_url(id: &str) -> String {
    format!("{}/mods/{}", FLAME_BASE_URL, id)
}

pub fn flame_versions_url(addon_id: &str, mc_version: &str, loader: Option<&str>) -> String {
    let mut url = format!("{}/mods/{}/files?pageSize=10000", FLAME_BASE_URL, addon_id);
    if !mc_version.is_empty() {
        url.push_str(&format!("&gameVersion={}", urlencoding(mc_version)));
    }
    if let Some(loader) = loader {
        let loader_id = match loader {
            "forge" => "1",
            "fabric" | "quilt" => "4",
            "neoforge" => "6",
            _ => "",
        };
        if !loader_id.is_empty() {
            url.push_str(&format!("&modLoaderType={}", loader_id));
        }
    }
    url
}

pub fn flame_fingerprint_url() -> String {
    format!("{}/fingerprints", FLAME_BASE_URL)
}

pub fn flame_files_url() -> String {
    format!("{}/mods/files", FLAME_BASE_URL)
}

pub fn flame_mods_url() -> String {
    format!("{}/mods", FLAME_BASE_URL)
}

fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push_str("%20"),
            c => {
                for b in c.to_string().bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}
