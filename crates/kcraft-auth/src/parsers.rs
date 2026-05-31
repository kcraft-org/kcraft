use std::collections::HashMap;
use base64::Engine;
use kcraft_core::account::{MinecraftEntitlement, MinecraftProfile, Skin, Cape, Token, Validity};

pub fn parse_x_token_response(data: &[u8]) -> Option<Token> {
    let json: serde_json::Value = serde_json::from_slice(data).ok()?;
    let obj = json.as_object()?;

    let token_str = obj.get("Token")?.as_str()?;
    let not_after = obj.get("NotAfter")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp());

    let issue_instant = obj.get("IssueInstant")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp());

    let mut extra = HashMap::new();
    if let Some(claims) = obj.get("DisplayClaims") {
        if let Some(xui) = claims.get("xui").and_then(|v| v.as_array()) {
            if let Some(first) = xui.first().and_then(|v| v.as_object()) {
                if let Some(uhs) = first.get("uhs").and_then(|v| v.as_str()) {
                    extra.insert("uhs".to_string(), uhs.to_string());
                }
            }
        }
    }

    Some(Token {
        token: Some(token_str.to_string()),
        not_after,
        issue_instant,
        refresh_token: None,
        extra,
        validity: Validity::Certain,
        persistent: true,
    })
}

pub fn parse_mojang_response(data: &[u8]) -> Option<Token> {
    let json: serde_json::Value = serde_json::from_slice(data).ok()?;
    let obj = json.as_object()?;
    let access_token = obj.get("access_token")?.as_str()?;

    let mut extra = HashMap::new();
    if let Some(username) = obj.get("username").and_then(|v| v.as_str()) {
        extra.insert("userName".to_string(), username.to_string());
    }

    let expires_in = obj.get("expires_in").and_then(|v| v.as_i64()).unwrap_or(86400);
    let issue_instant = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs() as i64);

    Some(Token {
        token: Some(access_token.to_string()),
        not_after: issue_instant.map(|i| i + expires_in),
        issue_instant,
        refresh_token: None,
        extra,
        validity: Validity::Certain,
        persistent: true,
    })
}

pub fn parse_minecraft_profile(data: &[u8]) -> Option<MinecraftProfile> {
    let json: serde_json::Value = serde_json::from_slice(data).ok()?;
    let obj = json.as_object()?;
    let id = obj.get("id")?.as_str()?.to_string();
    let name = obj.get("name")?.as_str()?.to_string();
    let mut skin = Skin::default();
    let mut capes = Vec::new();
    let mut current_cape = String::new();

    if let Some(skins) = obj.get("skins").and_then(|v| v.as_array()) {
        for s in skins {
            if let Some(skin_obj) = s.as_object() {
                let variant = skin_obj.get("variant").and_then(|v| v.as_str()).unwrap_or("classic");
                let skin_url = skin_obj.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let skin_id = skin_obj.get("id").and_then(|v| v.as_str()).unwrap_or("");
                if variant.contains("slim") {
                    skin = Skin::new(skin_id.to_string(), skin_url.to_string(), "slim".to_string());
                } else {
                    skin = Skin::new(skin_id.to_string(), skin_url.to_string(), "classic".to_string());
                }
            }
        }
    }

    if let Some(capes_arr) = obj.get("capes").and_then(|v| v.as_array()) {
        for c in capes_arr {
            if let Some(cape_obj) = c.as_object() {
                capes.push(Cape {
                    id: cape_obj.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    url: cape_obj.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    alias: String::new(),
                    data: None,
                });
                current_cape = cape_obj.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            }
        }
    }

    Some(MinecraftProfile {
        id, name, skin, current_cape, capes,
        validity: Validity::Certain,
    })
}

pub fn parse_minecraft_profile_mojang(data: &[u8]) -> Option<MinecraftProfile> {
    let json: serde_json::Value = serde_json::from_slice(data).ok()?;
    let obj = json.as_object()?;
    let id = obj.get("id")?.as_str()?.to_string();
    let name = obj.get("name")?.as_str()?.to_string();
    let mut skin = Skin::default();

    if let Some(properties) = obj.get("properties").and_then(|v| v.as_array()) {
        for prop in properties {
            if let Some(prop_obj) = prop.as_object() {
                if prop_obj.get("name").and_then(|v| v.as_str()) == Some("textures") {
                    if let Some(value) = prop_obj.get("value").and_then(|v| v.as_str()) {
                        if let Ok(decoded) = base64_decode(value) {
                            if let Ok(textures_json) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                                if let Some(textures) = textures_json.get("textures").and_then(|v| v.as_object()) {
                                    if let Some(skin_val) = textures.get("SKIN") {
                                        skin.url = skin_val.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        if let Some(meta) = skin_val.get("metadata") {
                                            if meta.get("model").and_then(|v| v.as_str()) == Some("slim") {
                                                skin.variant = "slim".to_string();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Some(MinecraftProfile {
        id, name, skin, current_cape: String::new(), capes: Vec::new(),
        validity: Validity::Certain,
    })
}

pub fn parse_minecraft_entitlements(data: &[u8]) -> Option<MinecraftEntitlement> {
    let json: serde_json::Value = serde_json::from_slice(data).ok()?;
    let obj = json.as_object()?;
    let mut owns = false;
    let mut can_play = false;

    if let Some(items) = obj.get("items").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                match name {
                    "product_minecraft" => owns = true,
                    "game_minecraft" => can_play = true,
                    _ => {}
                }
            }
        }
    }

    Some(MinecraftEntitlement {
        owns_minecraft: owns,
        can_play_minecraft: can_play,
        validity: Validity::Certain,
    })
}

pub fn parse_yggdrasil_response(json: &serde_json::Value, client_token: &str) -> Option<Token> {
    let obj = json.as_object()?;
    let access_token = obj.get("accessToken")?.as_str()?;
    let returned_client_token = obj.get("clientToken").and_then(|v| v.as_str()).unwrap_or(client_token);

    let mut extra = HashMap::new();
    extra.insert("clientToken".to_string(), returned_client_token.to_string());

    if let Some(profile) = obj.get("selectedProfile") {
        if let Some(profile_obj) = profile.as_object() {
            if let Some(pid) = profile_obj.get("id").and_then(|v| v.as_str()) {
                extra.insert("profileId".to_string(), pid.to_string());
            }
            if let Some(pname) = profile_obj.get("name").and_then(|v| v.as_str()) {
                extra.insert("profileName".to_string(), pname.to_string());
            }
        }
    }

    if let Some(available) = obj.get("availableProfiles") {
        if let Some(arr) = available.as_array() {
            for profile in arr {
                if let Some(po) = profile.as_object() {
                    if let Some(pid) = po.get("id").and_then(|v| v.as_str()) {
                        extra.insert("availableProfileId".to_string(), pid.to_string());
                    }
                }
            }
        }
    }

    Some(Token {
        token: Some(access_token.to_string()),
        not_after: None,
        issue_instant: Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs() as i64).unwrap_or(0)),
        refresh_token: None,
        extra,
        validity: Validity::Certain,
        persistent: true,
    })
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|e| e.to_string())
}
