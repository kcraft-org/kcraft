use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ATLPackType {
    Public,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ATLModType {
    Root, Forge, Jar, Mods, Flan, Dependency, Ic2Lib, DenLib,
    Coremods, MCPC, Plugins, Extract, Decomp, TexturePack,
    ResourcePack, ShaderPack, TexturePackExtract, ResourcePackExtract,
    Millenaire, Unknown,
}

impl std::str::FromStr for ATLModType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "root" => ATLModType::Root,
            "forge" => ATLModType::Forge,
            "jar" => ATLModType::Jar,
            "mods" => ATLModType::Mods,
            "flan" => ATLModType::Flan,
            "dependency" => ATLModType::Dependency,
            "ic2lib" => ATLModType::Ic2Lib,
            "denlib" => ATLModType::DenLib,
            "coremods" => ATLModType::Coremods,
            "mcpc" => ATLModType::MCPC,
            "plugins" => ATLModType::Plugins,
            "extract" => ATLModType::Extract,
            "decomp" => ATLModType::Decomp,
            "texturepack" => ATLModType::TexturePack,
            "resourcepack" => ATLModType::ResourcePack,
            "shaderpack" => ATLModType::ShaderPack,
            "texturepackextract" => ATLModType::TexturePackExtract,
            "resourcepackextract" => ATLModType::ResourcePackExtract,
            "millenaire" => ATLModType::Millenaire,
            _ => ATLModType::Unknown,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ATLShareCodeMod {
    pub selected: bool,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ATLShareCode {
    pub pack: String,
    pub version: String,
    pub mods: Vec<ATLShareCodeMod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ATLIndexedPack {
    pub id: i32,
    pub position: i32,
    pub name: String,
    pub pack_type: ATLPackType,
    pub versions: Vec<ATLIndexedVersion>,
    pub system: bool,
    pub description: String,
    pub safe_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ATLIndexedVersion {
    pub version: String,
    pub minecraft: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ATLVersionMod {
    pub name: String,
    pub version: String,
    pub url: String,
    pub file: String,
    pub md5: String,
    pub download_type: String,
    pub mod_type: ATLModType,
    pub extract_to: String,
    pub description: String,
    pub optional: bool,
    pub recommended: bool,
    pub selected: bool,
    pub hidden: bool,
    pub library: bool,
    pub group: String,
    pub depends: Vec<String>,
    pub colour: String,
    pub warning: String,
    pub client: bool,
}
