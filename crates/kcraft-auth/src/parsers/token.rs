use kcraft_core::account::{Token, Validity};
use std::collections::HashMap;

pub fn parse_x_token_response(data: &[u8]) -> Option<Token> {
    let json: serde_json::Value = serde_json::from_slice(data).ok()?;
    let obj = json.as_object()?;

    let token_str = obj.get("Token")?.as_str()?;
    let not_after = obj
        .get("NotAfter")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp());

    let issue_instant = obj
        .get("IssueInstant")
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

    let expires_in = obj
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .unwrap_or(86400);
    let issue_instant = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64);

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

pub fn parse_yggdrasil_response(json: &serde_json::Value, client_token: &str) -> Option<Token> {
    let obj = json.as_object()?;
    let access_token = obj.get("accessToken")?.as_str()?;
    let returned_client_token = obj
        .get("clientToken")
        .and_then(|v| v.as_str())
        .unwrap_or(client_token);

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
        issue_instant: Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        ),
        refresh_token: None,
        extra,
        validity: Validity::Certain,
        persistent: true,
    })
}
