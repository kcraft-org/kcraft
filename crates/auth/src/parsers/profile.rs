use base64::Engine;
use kcraft_core::account::{Cape, MinecraftProfile, Skin, Validity};

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
                let variant = skin_obj
                    .get("variant")
                    .and_then(|v| v.as_str())
                    .unwrap_or("classic");
                let skin_url = skin_obj.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let skin_id = skin_obj.get("id").and_then(|v| v.as_str()).unwrap_or("");
                if variant.contains("slim") {
                    skin = Skin::new(
                        skin_id.to_string(),
                        skin_url.to_string(),
                        "slim".to_string(),
                    );
                } else {
                    skin = Skin::new(
                        skin_id.to_string(),
                        skin_url.to_string(),
                        "classic".to_string(),
                    );
                }
            }
        }
    }

    if let Some(capes_arr) = obj.get("capes").and_then(|v| v.as_array()) {
        for c in capes_arr {
            if let Some(cape_obj) = c.as_object() {
                capes.push(Cape {
                    id: cape_obj
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    url: cape_obj
                        .get("url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    alias: String::new(),
                    data: None,
                });
                current_cape = cape_obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
            }
        }
    }

    Some(MinecraftProfile {
        id,
        name,
        skin,
        current_cape,
        capes,
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
                            if let Ok(textures_json) =
                                serde_json::from_slice::<serde_json::Value>(&decoded)
                            {
                                if let Some(textures) =
                                    textures_json.get("textures").and_then(|v| v.as_object())
                                {
                                    if let Some(skin_val) = textures.get("SKIN") {
                                        skin.url = skin_val
                                            .get("url")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        if let Some(meta) = skin_val.get("metadata") {
                                            if meta.get("model").and_then(|v| v.as_str())
                                                == Some("slim")
                                            {
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
        id,
        name,
        skin,
        current_cape: String::new(),
        capes: Vec::new(),
        validity: Validity::Certain,
    })
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|e| e.to_string())
}
