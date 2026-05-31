use std::path::PathBuf;

use kcraft_core::account::{AccountData, AccountState, AccountType, Validity};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{AuthFlow, AuthError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountFile {
    #[serde(rename = "formatVersion")]
    format_version: u32,
    accounts: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct MinecraftAccount {
    pub data: AccountData,
    pub active: bool,
}

impl MinecraftAccount {
    pub fn new(data: AccountData) -> Self {
        MinecraftAccount { data, active: false }
    }

    pub fn create_offline(username: &str) -> Self {
        let mut data = AccountData::default();
        let mut flow = crate::OfflineFlow::new(username.to_string());
        let _ = flow.execute(&mut data);
        MinecraftAccount { data, active: false }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn should_refresh(&self) -> bool {
        crate::flow::should_refresh(&self.data)
    }

    pub fn save_to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        map.insert("type".to_string(), serde_json::Value::String(
            match self.data.account_type {
                AccountType::Msa => "MSA".to_string(),
                AccountType::AuthlibInjector => "Authlib-Injector".to_string(),
                AccountType::Offline => "Offline".to_string(),
            }
        ));

        if self.data.account_type == AccountType::Msa {
            map.insert("msa-client-id".to_string(), serde_json::Value::String(
                self.data.msa_client_id.clone()
            ));
            if self.data.msa_token.token.is_some() {
                map.insert("msa".to_string(), token_to_json(&self.data.msa_token));
            }
            if self.data.user_token.token.is_some() {
                map.insert("utoken".to_string(), token_to_json(&self.data.user_token));
            }
            if self.data.xbox_api_token.token.is_some() {
                map.insert("xrp-main".to_string(), token_to_json(&self.data.xbox_api_token));
            }
            if self.data.mojangservices_token.token.is_some() {
                map.insert("xrp-mc".to_string(), token_to_json(&self.data.mojangservices_token));
            }
        }

        if self.data.account_type == AccountType::AuthlibInjector {
            map.insert("authlibInjectorUrl".to_string(), serde_json::Value::String(
                self.data.authlib_injector_base_url.clone()
            ));
        }

        map.insert("ygg".to_string(), token_to_json(&self.data.yggdrasil_token));

        let profile = serde_json::json!({
            "id": self.data.minecraft_profile.id,
            "name": self.data.minecraft_profile.name,
            "skin": {
                "id": self.data.minecraft_profile.skin.id,
                "url": self.data.minecraft_profile.skin.url,
                "variant": self.data.minecraft_profile.skin.variant,
                "data": self.data.minecraft_profile.skin.data.as_deref().unwrap_or(""),
            },
            "capes": self.data.minecraft_profile.capes.iter().map(|c| serde_json::json!({
                "id": c.id,
                "url": c.url,
                "alias": c.alias,
            })).collect::<Vec<_>>(),
            "cape": self.data.minecraft_profile.current_cape,
        });
        map.insert("profile".to_string(), profile);

        let entitlement = serde_json::json!({
            "ownsMinecraft": self.data.minecraft_entitlement.owns_minecraft,
            "canPlayMinecraft": self.data.minecraft_entitlement.can_play_minecraft,
        });
        map.insert("entitlement".to_string(), entitlement);

        map.insert("active".to_string(), serde_json::Value::Bool(self.active));

        serde_json::Value::Object(map)
    }

    pub fn load_from_json(json: &serde_json::Value) -> Option<Self> {
        let obj = json.as_object()?;
        let type_str = obj.get("type")?.as_str()?;
        let account_type = match type_str {
            "MSA" => AccountType::Msa,
            "Authlib-Injector" => AccountType::AuthlibInjector,
            _ => AccountType::Offline,
        };

        let mut data = AccountData {
            account_type,
            ..Default::default()
        };

        if account_type == AccountType::Msa {
            if let Some(client_id) = obj.get("msa-client-id").and_then(|v| v.as_str()) {
                data.msa_client_id = client_id.to_string();
            }
            if let Some(msa) = obj.get("msa") {
                data.msa_token = token_from_json(msa);
            }
            if let Some(utoken) = obj.get("utoken") {
                data.user_token = token_from_json(utoken);
            }
            if let Some(xrp_main) = obj.get("xrp-main") {
                data.xbox_api_token = token_from_json(xrp_main);
            }
            if let Some(xrp_mc) = obj.get("xrp-mc") {
                data.mojangservices_token = token_from_json(xrp_mc);
            }
        }

        if account_type == AccountType::AuthlibInjector {
            if let Some(url) = obj.get("authlibInjectorUrl").and_then(|v| v.as_str()) {
                data.authlib_injector_base_url = url.to_string();
            }
        }

        if let Some(ygg) = obj.get("ygg") {
            data.yggdrasil_token = token_from_json(ygg);
        }

        if let Some(profile) = obj.get("profile") {
            if let Some(profile_obj) = profile.as_object() {
                data.minecraft_profile.id = profile_obj.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                data.minecraft_profile.name = profile_obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if let Some(skin) = profile_obj.get("skin") {
                    if let Some(skin_obj) = skin.as_object() {
                        data.minecraft_profile.skin.id = skin_obj.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        data.minecraft_profile.skin.url = skin_obj.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        data.minecraft_profile.skin.variant = skin_obj.get("variant").and_then(|v| v.as_str()).unwrap_or("classic").to_string();
                        data.minecraft_profile.skin.data = skin_obj.get("data").and_then(|v| v.as_str()).map(|s| s.to_string());
                    }
                }
                if let Some(capes) = profile_obj.get("capes").and_then(|v| v.as_array()) {
                    for c in capes {
                        if let Some(co) = c.as_object() {
                            data.minecraft_profile.capes.push(kcraft_core::account::Cape {
                                id: co.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                url: co.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                alias: co.get("alias").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                data: None,
                            });
                        }
                    }
                }
                data.minecraft_profile.current_cape = profile_obj.get("cape").and_then(|v| v.as_str()).unwrap_or("").to_string();
            }
        }

        if let Some(entitlement) = obj.get("entitlement") {
            if let Some(eo) = entitlement.as_object() {
                data.minecraft_entitlement.owns_minecraft = eo.get("ownsMinecraft").and_then(|v| v.as_bool()).unwrap_or(false);
                data.minecraft_entitlement.can_play_minecraft = eo.get("canPlayMinecraft").and_then(|v| v.as_bool()).unwrap_or(false);
            }
        }

        let active = obj.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
        data.validity = Validity::Assumed;
        data.account_state = AccountState::Online;

        Some(MinecraftAccount { data, active })
    }
}

fn token_to_json(token: &kcraft_core::account::Token) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if let Some(iat) = token.issue_instant {
        map.insert("iat".to_string(), serde_json::Value::Number(serde_json::Number::from(iat)));
    }
    if let Some(exp) = token.not_after {
        map.insert("exp".to_string(), serde_json::Value::Number(serde_json::Number::from(exp)));
    }
    if let Some(ref t) = token.token {
        map.insert("token".to_string(), serde_json::Value::String(t.clone()));
    }
    if let Some(ref rt) = token.refresh_token {
        map.insert("refresh_token".to_string(), serde_json::Value::String(rt.clone()));
    }
    if !token.extra.is_empty() {
        let extra_map: serde_json::Map<String, serde_json::Value> = token.extra.iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();
        map.insert("extra".to_string(), serde_json::Value::Object(extra_map));
    }
    serde_json::Value::Object(map)
}

fn token_from_json(json: &serde_json::Value) -> kcraft_core::account::Token {
    let obj = match json.as_object() {
        Some(o) => o,
        None => return kcraft_core::account::Token::default(),
    };

    kcraft_core::account::Token {
        issue_instant: obj.get("iat").and_then(|v| v.as_i64()),
        not_after: obj.get("exp").and_then(|v| v.as_i64()),
        token: obj.get("token").and_then(|v| v.as_str()).map(|s| s.to_string()),
        refresh_token: obj.get("refresh_token").and_then(|v| v.as_str()).map(|s| s.to_string()),
        extra: obj.get("extra")
            .and_then(|v| v.as_object())
            .map(|m| m.iter().map(|(k, v)| {
                (k.clone(), v.as_str().unwrap_or("").to_string())
            }).collect())
            .unwrap_or_default(),
        validity: Validity::Assumed,
        persistent: true,
    }
}

#[derive(Debug, Clone)]
pub struct AccountList {
    accounts: Vec<MinecraftAccount>,
    default_account: Option<usize>,
    file_path: PathBuf,
    autosave: bool,
}

impl AccountList {
    pub fn new(file_path: PathBuf) -> Self {
        let mut list = AccountList {
            accounts: Vec::new(),
            default_account: None,
            file_path,
            autosave: false,
        };
        let _ = list.load();
        list
    }

    pub fn count(&self) -> usize {
        self.accounts.len()
    }

    pub fn at(&self, index: usize) -> Option<&MinecraftAccount> {
        self.accounts.get(index)
    }

    pub fn get_account_by_profile_name(&self, name: &str) -> Option<&MinecraftAccount> {
        self.accounts.iter().find(|a| a.data.profile_name() == name)
    }

    pub fn get_account_by_profile_id(&self, id: &str) -> Option<&MinecraftAccount> {
        self.accounts.iter().find(|a| a.data.profile_id() == id)
    }

    pub fn add_account(&mut self, account: MinecraftAccount) {
        self.accounts.push(account);
        if self.autosave {
            let _ = self.save();
        }
    }

    pub fn remove_account(&mut self, index: usize) {
        if index < self.accounts.len() {
            self.accounts.remove(index);
            if self.default_account == Some(index) {
                self.default_account = None;
            } else if let Some(default) = self.default_account {
                if index < default {
                    self.default_account = Some(default - 1);
                }
            }
            if self.autosave {
                let _ = self.save();
            }
        }
    }

    pub fn default_account(&self) -> Option<&MinecraftAccount> {
        self.default_account.and_then(|i| self.accounts.get(i))
    }

    pub fn set_default_account(&mut self, index: usize) {
        if index < self.accounts.len() {
            if let Some(old) = self.default_account {
                if old < self.accounts.len() {
                    self.accounts[old].active = false;
                }
            }
            self.accounts[index].active = true;
            self.default_account = Some(index);
            if self.autosave {
                let _ = self.save();
            }
        }
    }

    pub fn profile_names(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.data.profile_name().to_string()).collect()
    }

    pub fn any_account_is_valid(&self) -> bool {
        self.accounts.iter().any(|a| a.data.validity == Validity::Certain)
    }

    pub fn load(&mut self) -> Result<()> {
        if !self.file_path.exists() {
            return Ok(());
        }
        let data = std::fs::read_to_string(&self.file_path)
            .map_err(|e| AuthError::Network(format!("Failed to read accounts file: {}", e)))?;

        let file: AccountFile = serde_json::from_str(&data)
            .map_err(|e| AuthError::Serialization(e.to_string()))?;

        self.accounts.clear();
        self.default_account = None;

        for (i, json) in file.accounts.iter().enumerate() {
            if let Some(account) = MinecraftAccount::load_from_json(json) {
                let is_active = json.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
                if is_active {
                    self.default_account = Some(i);
                }
                self.accounts.push(account);
            }
        }

        debug!("Loaded {} accounts from {}", self.accounts.len(), self.file_path.display());
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let accounts: Vec<serde_json::Value> = self.accounts.iter()
            .map(|a| a.save_to_json())
            .collect();

        let file = AccountFile {
            format_version: 3,
            accounts,
        };

        let data = serde_json::to_string_pretty(&file)
            .map_err(|e| AuthError::Serialization(e.to_string()))?;

        if let Some(parent) = self.file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        kcraft_fs::write(&self.file_path, data.as_bytes())
            .map_err(|e| AuthError::Network(e.to_string()))?;

        debug!("Saved {} accounts to {}", self.accounts.len(), self.file_path.display());
        Ok(())
    }

    pub fn set_autosave(&mut self, autosave: bool) {
        self.autosave = autosave;
    }
}
