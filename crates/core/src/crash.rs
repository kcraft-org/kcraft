use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::PortableMode;

static INITIALIZED: Mutex<bool> = Mutex::new(false);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReport {
    pub timestamp: String,
    pub version: String,
    pub os: String,
    pub arch: String,
    pub message: String,
    pub panic_message: String,
    pub backtrace: String,
    pub log_snippet: String,
    pub instance_id: String,
}

impl CrashReport {
    pub fn save(&self) -> crate::Result<PathBuf> {
        let dir = crashes_dir();
        std::fs::create_dir_all(&dir)?;
        let filename = format!(
            "crash_{}.json",
            self.timestamp.replace(':', "-").replace(' ', "_")
        );
        let path = dir.join(&filename);
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(path)
    }
}

pub struct CrashReporter;

impl CrashReporter {
    pub fn init() {
        let mut initialized = INITIALIZED.lock().unwrap();
        if *initialized {
            return;
        }
        *initialized = true;

        let version = crate::BUILD_CONFIG.version_string();
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();

        std::panic::set_hook(Box::new(move |panic_info| {
            let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
            let panic_message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                String::new()
            };

            let message = if let Some(location) = panic_info.location() {
                format!(
                    "Panic at {}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            } else {
                "Panic at unknown location".to_string()
            };

            let backtrace_str = format!("{:?}", backtrace::Backtrace::new());

            let log_snippet = read_log_snippet().unwrap_or_default();

            let report = CrashReport {
                timestamp,
                version: version.clone(),
                os: os.clone(),
                arch: arch.clone(),
                message,
                panic_message,
                backtrace: backtrace_str,
                log_snippet,
                instance_id: String::new(),
            };

            if let Err(e) = report.save() {
                eprintln!("Failed to save crash report: {}", e);
            }
        }));
    }
}

pub fn crashes_dir() -> PathBuf {
    PortableMode::data_dir().join("kcraft").join("crashes")
}

fn read_log_snippet() -> Option<String> {
    let candidates = [
        PortableMode::data_dir()
            .join("kcraft")
            .join("logs")
            .join("latest.log"),
        PortableMode::data_dir().join("kcraft").join("latest.log"),
    ];

    for log_path in &candidates {
        if log_path.exists() {
            if let Ok(content) = std::fs::read_to_string(log_path) {
                let lines: Vec<&str> = content.lines().collect();
                let start = lines.len().saturating_sub(50);
                let snippet = lines[start..].join("\n");
                if !snippet.is_empty() {
                    return Some(snippet);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_report_save() {
        let report = CrashReport {
            timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            version: "1.0.0".to_string(),
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            message: "test panic".to_string(),
            panic_message: "test".to_string(),
            backtrace: "backtrace".to_string(),
            log_snippet: String::new(),
            instance_id: String::new(),
        };
        let path = report.save().unwrap();
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_crashes_dir() {
        let dir = crashes_dir();
        assert!(dir.to_string_lossy().contains("crashes"));
    }
}
