use app_core::account::{MinecraftEntitlement, Validity};

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
