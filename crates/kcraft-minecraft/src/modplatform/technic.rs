use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicSolderPack {
    pub recommended: String,
    pub latest: String,
    pub builds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicSolderBuildMod {
    pub name: String,
    pub version: String,
    pub md5: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicSolderBuild {
    pub minecraft: String,
    pub mods: Vec<TechnicSolderBuildMod>,
}

pub fn parse_solder_pack(json: &serde_json::Value) -> Option<TechnicSolderPack> {
    Some(TechnicSolderPack {
        recommended: json.get("recommended")?.as_str()?.to_string(),
        latest: json.get("latest")?.as_str()?.to_string(),
        builds: json.get("builds")?.as_array()?.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
    })
}

pub fn parse_solder_build(json: &serde_json::Value) -> Option<TechnicSolderBuild> {
    let mods = json.get("mods")?.as_array()?.iter().filter_map(|m| {
        Some(TechnicSolderBuildMod {
            name: m.get("name")?.as_str()?.to_string(),
            version: m.get("version")?.as_str()?.to_string(),
            md5: m.get("md5")?.as_str()?.to_string(),
            url: m.get("url")?.as_str()?.to_string(),
        })
    }).collect();

    Some(TechnicSolderBuild {
        minecraft: json.get("minecraft")?.as_str()?.to_string(),
        mods,
    })
}
