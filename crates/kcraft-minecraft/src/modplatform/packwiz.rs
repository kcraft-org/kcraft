use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::modplatform::mod_index::{IndexedPack, IndexedVersion};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackwizMod {
    pub slug: String,
    pub name: String,
    pub filename: String,
    pub side: String,
    pub mode: String,
    pub url: String,
    pub hash_format: String,
    pub hash: String,
    pub provider: String,
    pub file_id: String,
    pub project_id: String,
    pub do_updates: String,
}

impl PackwizMod {
    pub fn new() -> Self {
        PackwizMod {
            slug: String::new(), name: String::new(), filename: String::new(),
            side: "both".to_string(), mode: "url".to_string(),
            url: String::new(), hash_format: String::new(), hash: String::new(),
            provider: String::new(), file_id: String::new(), project_id: String::new(),
            do_updates: "true".to_string(),
        }
    }
}

impl Default for PackwizMod { fn default() -> Self { Self::new() } }

pub fn create_mod_format_from_indexed(
    _index_dir: &Path,
    mod_pack: &IndexedPack,
    mod_version: &IndexedVersion,
) -> PackwizMod {
    let mut m = PackwizMod::new();
    m.name = mod_version.file_name.trim_end_matches(".jar").to_string();
    m.filename = mod_version.file_name.clone();
    m.url = mod_version.download_url.clone();
    m.hash_format = mod_version.hash_type.clone();
    m.hash = mod_version.hash.clone();
    m.provider = mod_pack.provider.clone();
    m.project_id = mod_pack.addon_id.clone();
    m.file_id = mod_version.file_id.clone();
    m
}

pub fn update_mod_index(index_dir: &Path, m: &PackwizMod) {
    let file_path = index_dir.join(format!("{}.pw.toml", m.slug));
    let content = format!(
        r#"name = "{}"
filename = "{}"
side = "{}"

[download]
mode = "{}"
url = "{}"
hash-format = "{}"
hash = "{}"

[update]
[update.{}]
file-id = {}
project-id = {}
"#,
        m.name, m.filename, m.side, m.mode, m.url, m.hash_format, m.hash,
        m.provider, m.file_id, m.project_id
    );
    let _ = std::fs::write(&file_path, content);
}

pub fn delete_mod_index_by_slug(index_dir: &Path, slug: &str) {
    let file_path = index_dir.join(format!("{}.pw.toml", slug));
    if file_path.exists() {
        let _ = std::fs::remove_file(&file_path);
    }
}

pub fn get_index_for_mod_by_slug(index_dir: &Path, slug: &str) -> Option<PackwizMod> {
    let file_path = index_dir.join(format!("{}.pw.toml", slug));
    if !file_path.exists() { return None; }
    parse_pw_toml(&std::fs::read_to_string(file_path).ok()?)
}

fn parse_pw_toml(content: &str) -> Option<PackwizMod> {
    let mut m = PackwizMod::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') { continue; }
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim();
            let val = line[eq + 1..].trim().trim_matches('"');
            match key {
                "name" => m.name = val.to_string(),
                "filename" => m.filename = val.to_string(),
                "side" => m.side = val.to_string(),
                "url" => m.url = val.to_string(),
                "hash-format" => m.hash_format = val.to_string(),
                "hash" => m.hash = val.to_string(),
                "file-id" => m.file_id = val.to_string(),
                "project-id" => m.project_id = val.to_string(),
                "mode" => m.mode = val.to_string(),
                _ => {}
            }
        }
    }

    Some(m)
}
