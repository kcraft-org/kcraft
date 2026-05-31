use crate::modplatform::mod_index::{IndexedPack, IndexedVersion, ModpackAuthor};
use crate::modplatform::Provider;

pub const MODRINTH_BASE_URL: &str = "https://api.modrinth.com/v2";

pub fn modrinth_load_indexed_pack(obj: &serde_json::Value) -> IndexedPack {
    let mut pack = IndexedPack::new();
    pack.provider = Provider::Modrinth.name().to_string();

    if let Some(id) = obj.get("project_id").and_then(|v| v.as_str()) {
        pack.addon_id = id.to_string();
    } else if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
        pack.addon_id = id.to_string();
    }

    if let Some(title) = obj.get("title").and_then(|v| v.as_str()) {
        pack.name = title.to_string();
    }

    if let Some(desc) = obj.get("description").and_then(|v| v.as_str()) {
        pack.description = desc.to_string();
    }

    if let Some(slug) = obj.get("slug").and_then(|v| v.as_str()) {
        pack.slug = slug.to_string();
        pack.website_url = format!("https://modrinth.com/mod/{}", slug);
    }

    if let Some(icon) = obj.get("icon_url").and_then(|v| v.as_str()) {
        pack.logo_url = icon.to_string();
    }

    if let Some(author) = obj.get("author").and_then(|v| v.as_str()) {
        pack.authors.push(ModpackAuthor {
            name: author.to_string(),
            url: format!("https://modrinth.com/user/{}", author),
        });
    }

    pack
}

pub fn modrinth_load_extra_pack_data(obj: &serde_json::Value, pack: &mut IndexedPack) {
    if let Some(issues) = obj.get("issues_url").and_then(|v| v.as_str()) {
        pack.extra_data.issues_url = issues.to_string();
    }
    if let Some(source) = obj.get("source_url").and_then(|v| v.as_str()) {
        pack.extra_data.source_url = source.to_string();
    }
    if let Some(wiki) = obj.get("wiki_url").and_then(|v| v.as_str()) {
        pack.extra_data.wiki_url = wiki.to_string();
    }
    if let Some(discord) = obj.get("discord_url").and_then(|v| v.as_str()) {
        pack.extra_data.discord_url = discord.to_string();
    }

    if let Some(donations) = obj.get("donation_urls").and_then(|v| v.as_array()) {
        for d in donations {
            let id = d.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let platform = d.get("platform").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let url = d.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            pack.extra_data.donate.push(crate::modplatform::DonationData { id, platform, url });
        }
    }

    pack.extra_data_loaded = true;
}

pub fn modrinth_load_indexed_pack_version(
    obj: &serde_json::Value,
    preferred_hash_type: &str,
    _preferred_file_name: Option<&str>,
) -> IndexedVersion {
    let mut ver = IndexedVersion::new();

    if let Some(pid) = obj.get("project_id").and_then(|v| v.as_str()) {
        ver.addon_id = pid.to_string();
    }
    if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
        ver.file_id = id.to_string();
    }
    if let Some(date) = obj.get("date_published").and_then(|v| v.as_str()) {
        ver.date = date.to_string();
    }
    if let Some(game_versions) = obj.get("game_versions").and_then(|v| v.as_array()) {
        for v in game_versions {
            if let Some(s) = v.as_str() {
                ver.mc_versions.push(s.to_string());
            }
        }
    }
    if let Some(loaders) = obj.get("loaders").and_then(|v| v.as_array()) {
        for l in loaders {
            if let Some(s) = l.as_str() {
                ver.loaders.push(s.to_string());
            }
        }
    }
    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
        ver.version = name.to_string();
    }
    if let Some(vn) = obj.get("version_number").and_then(|v| v.as_str()) {
        ver.version_number = vn.to_string();
    }
    if let Some(changelog) = obj.get("changelog").and_then(|v| v.as_str()) {
        ver.changelog = changelog.to_string();
    }

    if let Some(files) = obj.get("files").and_then(|v| v.as_array()) {
        for file in files {
            let primary = file.get("primary").and_then(|v| v.as_bool()).unwrap_or(false);
            let is_matching = primary || _preferred_file_name.is_some_and(|pref| {
                file.get("filename").and_then(|v| v.as_str()) == Some(pref)
            });

            if is_matching || (!primary && ver.download_url.is_empty()) {
                if let Some(url) = file.get("url").and_then(|v| v.as_str()) {
                    ver.download_url = url.to_string();
                }
                if let Some(fname) = file.get("filename").and_then(|v| v.as_str()) {
                    ver.file_name = fname.to_string();
                }
                if let Some(hashes) = file.get("hashes").and_then(|v| v.as_object()) {
                    if let Some(h) = hashes.get(preferred_hash_type).and_then(|v| v.as_str()) {
                        ver.hash_type = preferred_hash_type.to_string();
                        ver.hash = h.to_string();
                    } else {
                        for (k, v) in hashes {
                            if let Some(h) = v.as_str() {
                                ver.hash_type = k.clone();
                                ver.hash = h.to_string();
                                break;
                            }
                        }
                    }
                }
                if primary { break; }
            }
        }
    }

    ver
}

pub fn modrinth_search_url(args: &crate::modplatform::SearchArgs) -> String {
    let mut url = format!("{}/search?offset={}&limit=25", MODRINTH_BASE_URL, args.offset);
    if !args.search.is_empty() {
        url.push_str(&format!("&query={}", urlencoding(&args.search)));
    }
    if !args.sorting.is_empty() {
        url.push_str(&format!("&index={}", args.sorting));
    }

    let mut facets: Vec<String> = Vec::new();
    if !args.loaders.is_empty() {
        let loader_strs: Vec<String> = args.loaders.iter()
            .map(|l| format!("\"categories:{}\"", l.to_string()))
            .collect();
        facets.push(format!("[{}]", loader_strs.join(",")));
    }
    if !args.versions.is_empty() {
        let ver_strs: Vec<String> = args.versions.iter()
            .map(|v| format!("\"versions:{}\"", v))
            .collect();
        facets.push(format!("[{}]", ver_strs.join(",")));
    }
    facets.push("[\"project_type:mod\"]".to_string());

    if !facets.is_empty() {
        url.push_str(&format!("&facets={}", urlencoding(&format!("[{}]", facets.join(",")))));
    }

    url
}

pub fn modrinth_project_url(id: &str) -> String {
    format!("{}/project/{}", MODRINTH_BASE_URL, id)
}

pub fn modrinth_projects_url(ids: &[String]) -> String {
    let ids_json = serde_json::Value::Array(ids.iter().map(|s| serde_json::Value::String(s.clone())).collect());
    format!("{}/projects?ids={}", MODRINTH_BASE_URL, urlencoding(&ids_json.to_string()))
}

pub fn modrinth_versions_url(addon_id: &str, mc_versions: &[String], loaders: &[String]) -> String {
    let mut url = format!("{}/project/{}/version", MODRINTH_BASE_URL, addon_id);
    if !mc_versions.is_empty() {
        let versions_json = serde_json::Value::Array(
            mc_versions.iter().map(|v| serde_json::Value::String(v.clone())).collect()
        );
        url.push_str(&format!("?game_versions={}", urlencoding(&versions_json.to_string())));
    }
    if !loaders.is_empty() {
        let sep = if mc_versions.is_empty() { "?" } else { "&" };
        let loaders_json = serde_json::Value::Array(
            loaders.iter().map(|l| serde_json::Value::String(l.clone())).collect()
        );
        url.push_str(&format!("{}loaders={}", sep, urlencoding(&loaders_json.to_string())));
    }
    url
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
