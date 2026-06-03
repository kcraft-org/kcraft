use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::PortableMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeKind {
    Dark,
    Light,
    #[default]
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub kind: ThemeKind,
    pub primary_color: String,
    pub accent_color: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig {
            kind: ThemeKind::System,
            primary_color: "#1a1a2e".to_string(),
            accent_color: "#e94560".to_string(),
        }
    }
}

impl ThemeConfig {
    fn config_path() -> PathBuf {
        PortableMode::config_dir().join("kcraft").join("theme.json")
    }

    pub fn load() -> crate::Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(ThemeConfig::default());
        }
        Ok(serde_json::from_str(&std::fs::read_to_string(&path)?)?)
    }

    pub fn save(&self) -> crate::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_defaults() {
        let config = ThemeConfig::default();
        assert_eq!(config.kind, ThemeKind::System);
        assert_eq!(config.primary_color, "#1a1a2e");
        assert_eq!(config.accent_color, "#e94560");
    }

    #[test]
    fn test_theme_kind_serde() {
        let json = serde_json::to_string(&ThemeKind::Dark).unwrap();
        assert_eq!(json, "\"dark\"");
        let kind: ThemeKind = serde_json::from_str("\"light\"").unwrap();
        assert_eq!(kind, ThemeKind::Light);
    }
}
