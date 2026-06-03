use std::path::PathBuf;

pub struct PortableMode;

impl PortableMode {
    pub fn is_portable() -> bool {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                return parent.join(".portable").exists();
            }
        }
        false
    }

    pub fn data_dir() -> PathBuf {
        if Self::is_portable() {
            if let Ok(exe) = std::env::current_exe() {
                if let Some(parent) = exe.parent() {
                    return parent.to_path_buf();
                }
            }
        }
        Self::system_data_dir()
    }

    pub fn config_dir() -> PathBuf {
        if Self::is_portable() {
            if let Ok(exe) = std::env::current_exe() {
                if let Some(parent) = exe.parent() {
                    return parent.to_path_buf();
                }
            }
        }
        Self::system_config_dir()
    }

    #[cfg(target_os = "linux")]
    fn system_data_dir() -> PathBuf {
        if let Ok(val) = std::env::var("XDG_DATA_HOME") {
            PathBuf::from(val)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".local").join("share")
        } else {
            PathBuf::from(".")
        }
    }

    #[cfg(target_os = "macos")]
    fn system_data_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
        } else {
            PathBuf::from(".")
        }
    }

    #[cfg(target_os = "windows")]
    fn system_data_dir() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata)
        } else {
            PathBuf::from(".")
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn system_data_dir() -> PathBuf {
        PathBuf::from(".")
    }

    #[cfg(target_os = "linux")]
    fn system_config_dir() -> PathBuf {
        if let Ok(val) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(val)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config")
        } else {
            PathBuf::from(".")
        }
    }

    #[cfg(target_os = "macos")]
    fn system_config_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join("Library").join("Preferences")
        } else {
            PathBuf::from(".")
        }
    }

    #[cfg(target_os = "windows")]
    fn system_config_dir() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata)
        } else {
            PathBuf::from(".")
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn system_config_dir() -> PathBuf {
        PathBuf::from(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portable_mode_detection() {
        let _ = PortableMode::is_portable();
    }

    #[test]
    fn test_data_dir_not_empty() {
        let dir = PortableMode::data_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn test_config_dir_not_empty() {
        let dir = PortableMode::config_dir();
        assert!(!dir.as_os_str().is_empty());
    }
}
