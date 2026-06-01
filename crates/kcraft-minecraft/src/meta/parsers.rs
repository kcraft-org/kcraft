use crate::MetaRequire;

pub(super) fn parse_meta_time(s: &str) -> Option<i64> {
    // Try ISO 8601 format
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp_millis());
    }
    // Try unix timestamp (seconds)
    if let Ok(ts) = s.parse::<i64>() {
        return Some(ts);
    }
    None
}

pub(super) fn parse_requires(arr: &[serde_json::Value]) -> Vec<MetaRequire> {
    arr.iter()
        .filter_map(|v| {
            let obj = v.as_object()?;
            let uid = obj.get("uid")?.as_str()?;
            Some(MetaRequire {
                uid: uid.to_string(),
                equals: obj
                    .get("equals")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                suggests: obj
                    .get("suggests")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        })
        .collect()
}
