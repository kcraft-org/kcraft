use serde::{Deserialize, Serialize};

use super::types::Validity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skin {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub variant: String,
    #[serde(default)]
    pub data: Option<String>,
}

impl Skin {
    pub fn new(id: String, url: String, variant: String) -> Self {
        Skin {
            id,
            url,
            variant,
            data: None,
        }
    }
}

impl Default for Skin {
    fn default() -> Self {
        Skin {
            id: String::new(),
            url: String::new(),
            variant: "classic".to_string(),
            data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cape {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub alias: String,
    #[serde(default)]
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftEntitlement {
    #[serde(default)]
    pub owns_minecraft: bool,
    #[serde(default)]
    pub can_play_minecraft: bool,
    #[serde(default)]
    pub validity: Validity,
}

impl Default for MinecraftEntitlement {
    fn default() -> Self {
        MinecraftEntitlement {
            owns_minecraft: false,
            can_play_minecraft: false,
            validity: Validity::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfile {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub skin: Skin,
    #[serde(default)]
    pub current_cape: String,
    #[serde(default)]
    pub capes: Vec<Cape>,
    #[serde(default)]
    pub validity: Validity,
}

impl Default for MinecraftProfile {
    fn default() -> Self {
        MinecraftProfile {
            id: String::new(),
            name: String::new(),
            skin: Skin::default(),
            current_cape: String::new(),
            capes: Vec::new(),
            validity: Validity::None,
        }
    }
}
