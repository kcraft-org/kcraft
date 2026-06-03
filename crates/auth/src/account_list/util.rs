use app_core::account::{Token, Validity};

pub(crate) fn token_to_json(token: &Token) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if let Some(iat) = token.issue_instant {
        map.insert(
            "iat".to_string(),
            serde_json::Value::Number(serde_json::Number::from(iat)),
        );
    }
    if let Some(exp) = token.not_after {
        map.insert(
            "exp".to_string(),
            serde_json::Value::Number(serde_json::Number::from(exp)),
        );
    }
    if let Some(ref t) = token.token {
        map.insert("token".to_string(), serde_json::Value::String(t.clone()));
    }
    if let Some(ref rt) = token.refresh_token {
        map.insert(
            "refresh_token".to_string(),
            serde_json::Value::String(rt.clone()),
        );
    }
    if !token.extra.is_empty() {
        let extra_map: serde_json::Map<String, serde_json::Value> = token
            .extra
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();
        map.insert("extra".to_string(), serde_json::Value::Object(extra_map));
    }
    serde_json::Value::Object(map)
}

pub(crate) fn token_from_json(json: &serde_json::Value) -> Token {
    let obj = match json.as_object() {
        Some(o) => o,
        None => return Token::default(),
    };

    Token {
        issue_instant: obj.get("iat").and_then(|v| v.as_i64()),
        not_after: obj.get("exp").and_then(|v| v.as_i64()),
        token: obj
            .get("token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        refresh_token: obj
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        extra: obj
            .get("extra")
            .and_then(|v| v.as_object())
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        validity: Validity::Assumed,
        persistent: true,
    }
}
