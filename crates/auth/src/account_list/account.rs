use app_core::account::{AccountData, AccountState, AccountType, Validity};

use crate::AuthFlow;

use super::util::{token_from_json, token_to_json};

#[derive(Debug, Clone)]
pub struct MinecraftAccount {
    pub data: AccountData,
    pub active: bool,
}

impl MinecraftAccount {
    pub fn new(data: AccountData) -> Self {
        MinecraftAccount {
            data,
            active: false,
        }
    }

    pub fn create_offline(username: &str) -> Self {
        let mut data = AccountData::default();
        let mut flow = crate::OfflineFlow::new(username.to_string());
        let _ = flow.execute(&mut data);
        MinecraftAccount {
            data,
            active: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn should_refresh(&self) -> bool {
        crate::flow::should_refresh(&self.data)
    }

    pub fn save_to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        map.insert(
            "type".to_string(),
            serde_json::Value::String(match self.data.account_type {
                AccountType::Msa => "MSA".to_string(),
                AccountType::AuthlibInjector => "Authlib-Injector".to_string(),
                AccountType::Offline => "Offline".to_string(),
            }),
        );

        if self.data.account_type == AccountType::Msa {
            map.insert(
                "msa-client-id".to_string(),
                serde_json::Value::String(self.data.msa_client_id.clone()),
            );
            if self.data.msa_token.token.is_some() {
                map.insert("msa".to_string(), token_to_json(&self.data.msa_token));
            }
            if self.data.user_token.token.is_some() {
                map.insert("utoken".to_string(), token_to_json(&self.data.user_token));
            }
            if self.data.xbox_api_token.token.is_some() {
                map.insert(
                    "xrp-main".to_string(),
                    token_to_json(&self.data.xbox_api_token),
                );
            }
            if self.data.mojangservices_token.token.is_some() {
                map.insert(
                    "xrp-mc".to_string(),
                    token_to_json(&self.data.mojangservices_token),
                );
            }
        }

        if self.data.account_type == AccountType::AuthlibInjector {
            map.insert(
                "authlibInjectorUrl".to_string(),
                serde_json::Value::String(self.data.authlib_injector_base_url.clone()),
            );
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
                data.minecraft_profile.id = profile_obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                data.minecraft_profile.name = profile_obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if let Some(skin) = profile_obj.get("skin") {
                    if let Some(skin_obj) = skin.as_object() {
                        data.minecraft_profile.skin.id = skin_obj
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        data.minecraft_profile.skin.url = skin_obj
                            .get("url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        data.minecraft_profile.skin.variant = skin_obj
                            .get("variant")
                            .and_then(|v| v.as_str())
                            .unwrap_or("classic")
                            .to_string();
                        data.minecraft_profile.skin.data = skin_obj
                            .get("data")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                    }
                }
                if let Some(capes) = profile_obj.get("capes").and_then(|v| v.as_array()) {
                    for c in capes {
                        if let Some(co) = c.as_object() {
                            data.minecraft_profile.capes.push(app_core::account::Cape {
                                id: co
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                url: co
                                    .get("url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                alias: co
                                    .get("alias")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                data: None,
                            });
                        }
                    }
                }
                data.minecraft_profile.current_cape = profile_obj
                    .get("cape")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
            }
        }

        if let Some(entitlement) = obj.get("entitlement") {
            if let Some(eo) = entitlement.as_object() {
                data.minecraft_entitlement.owns_minecraft = eo
                    .get("ownsMinecraft")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                data.minecraft_entitlement.can_play_minecraft = eo
                    .get("canPlayMinecraft")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
            }
        }

        let active = obj.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
        data.validity = Validity::Assumed;
        data.account_state = AccountState::Online;

        Some(MinecraftAccount { data, active })
    }
}
