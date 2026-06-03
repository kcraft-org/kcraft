use std::io::Read;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::MinecraftError;

#[derive(Debug, Clone)]
pub struct WorldInfo {
    pub name: String,
    pub seed: i64,
    pub game_version: String,
    pub last_played: DateTime<Utc>,
    pub size_bytes: u64,
    pub player_count: i32,
}

pub fn read_world_info(world_dir: &Path) -> std::result::Result<WorldInfo, MinecraftError> {
    if !world_dir.is_dir() {
        return Err(MinecraftError::NotFound(format!(
            "World directory not found: {}",
            world_dir.display()
        )));
    }

    let level_dat_path = world_dir.join("level.dat");
    if !level_dat_path.exists() {
        return Err(MinecraftError::NotFound(format!(
            "level.dat not found in {}",
            world_dir.display()
        )));
    }

    let data = std::fs::read(&level_dat_path)?;
    let mut decoder = flate2::read::GzDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| MinecraftError::Parse(format!("Failed to decompress level.dat: {}", e)))?;

    let content = String::from_utf8_lossy(&decompressed);

    let name = extract_nbt_string(&content, "LevelName").unwrap_or_else(|| {
        world_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    let seed = extract_nbt_long(&content, "RandomSeed").unwrap_or(0);
    let game_version = extract_nbt_string(&content, "Version").unwrap_or_else(|| {
        extract_nbt_string(&content, "version").unwrap_or_else(|| {
            extract_nbt_string(&content, "DataVersion")
                .map(|v| format!("DataVersion {}", v))
                .unwrap_or_default()
        })
    });

    let last_played_ts = extract_nbt_long(&content, "LastPlayed").unwrap_or(0);
    let last_played = if last_played_ts > 0 {
        let secs = last_played_ts / 1000;
        let nsecs = ((last_played_ts % 1000) * 1_000_000) as u32;
        DateTime::from_timestamp(secs, nsecs).unwrap_or_default()
    } else {
        Utc::now()
    };

    let player_count = extract_player_count(&content);

    let size_bytes = calculate_dir_size(world_dir);

    // Build the game_version from available data
    let final_version = if game_version.is_empty() {
        let data_ver = extract_nbt_long(&content, "DataVersion").unwrap_or(0);
        format!("DataVersion {}", data_ver)
    } else {
        game_version
    };

    Ok(WorldInfo {
        name,
        seed,
        game_version: final_version,
        last_played,
        size_bytes,
        player_count,
    })
}

fn extract_nbt_string(content: &str, key: &str) -> Option<String> {
    // Look for string in NBT: key:"value" or key 'value'
    let patterns = [
        format!(r#""{}"\s*:\s*"([^"]*)""#, regex::escape(key)),
        format!(r#"{}:\s*"([^"]*)""#, regex::escape(key)),
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(content) {
                return Some(caps[1].to_string());
            }
        }
    }

    // Fallback: scan lines
    for line in content.lines() {
        if line.contains(key) {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let val = parts[1..].join(":").trim().to_string();
                let val = val.trim_matches('"').trim_matches('\'').to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }
    }

    None
}

fn extract_nbt_long(content: &str, key: &str) -> Option<i64> {
    let patterns = [
        format!(r#""{}"\s*:\s*(-?\d+)"#, regex::escape(key)),
        format!(r#"{}:\s*(-?\d+)"#, regex::escape(key)),
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(content) {
                if let Ok(val) = caps[1].parse::<i64>() {
                    return Some(val);
                }
            }
        }
    }

    // Fallback: scan lines
    for line in content.lines() {
        if line.contains(key) {
            for word in line.split(|c: char| !c.is_ascii_digit() && c != '-') {
                if !word.is_empty() {
                    if let Ok(val) = word.parse::<i64>() {
                        return Some(val);
                    }
                }
            }
        }
    }

    None
}

fn extract_player_count(content: &str) -> i32 {
    // Try to find "Bukkit" or "Players" section with count
    if let Some(count) = extract_nbt_long(content, "Players") {
        return count as i32;
    }
    0
}

fn calculate_dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                total += path.metadata().map(|m| m.len()).unwrap_or(0);
            } else if path.is_dir() {
                total += calculate_dir_size(&path);
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_world_info_missing() {
        let tmp = std::env::temp_dir().join("kcraft_world_info_missing");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let result = read_world_info(&tmp);
        assert!(result.is_err());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_extract_nbt_string() {
        let content = r#""LevelName":"My World""#;
        assert_eq!(
            extract_nbt_string(content, "LevelName"),
            Some("My World".to_string())
        );
    }

    #[test]
    fn test_extract_nbt_long() {
        let content = r#""RandomSeed":1234567890"#;
        assert_eq!(extract_nbt_long(content, "RandomSeed"), Some(1234567890));
    }
}
