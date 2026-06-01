use std::collections::HashMap;

pub fn clean_environment() -> HashMap<String, String> {
    let ignored = [
        "JAVA_ARGS",
        "CLASSPATH",
        "CONFIGPATH",
        "JAVA_HOME",
        "JRE_HOME",
        "_JAVA_OPTIONS",
        "JAVA_OPTIONS",
        "JAVA_TOOL_OPTIONS",
    ];

    let stripped = [
        #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
        ("LD_LIBRARY_PATH", "LAUNCHER_LD_LIBRARY_PATH"),
        #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
        ("LD_PRELOAD", "LAUNCHER_LD_PRELOAD"),
        ("QT_PLUGIN_PATH", "LAUNCHER_QT_PLUGIN_PATH"),
        ("QT_FONTPATH", "LAUNCHER_QT_FONTPATH"),
    ];

    let mut env = HashMap::new();

    for (key, value) in std::env::vars_os() {
        let key = key.to_string_lossy().to_string();
        let value = value.to_string_lossy().to_string();

        if ignored.contains(&key.as_str()) {
            tracing::debug!("Env: ignoring {}={}", key, value);
            continue;
        }

        if key.starts_with("LAUNCHER_") {
            tracing::debug!("Env: ignoring {}={}", key, value);
            continue;
        }

        let mut final_value = value.clone();

        for &(strip_key, launcher_key) in &stripped {
            if key == strip_key {
                if let Ok(launcher_val) = std::env::var(launcher_key) {
                    let delimiter = if cfg!(target_os = "windows") {
                        ';'
                    } else {
                        ':'
                    };
                    let target_items: Vec<&str> = value.split(delimiter).collect();
                    let to_remove: Vec<&str> = launcher_val.split(delimiter).collect();

                    let filtered: Vec<&str> = target_items
                        .into_iter()
                        .filter(|item| !to_remove.contains(item))
                        .collect();
                    final_value = filtered.join(&delimiter.to_string());
                    tracing::debug!(
                        "Env: stripped {} from {} to {}",
                        strip_key,
                        value,
                        final_value
                    );
                }
            }
        }

        #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
        {
            if key == "XMODIFIERS" && value.contains("@im=ibus") {
                let saved = value.clone();
                final_value = value.replace("@im=ibus", "");
                tracing::debug!("Env: stripped @im=ibus from {}: {}", saved, final_value);
            }
        }

        env.insert(key, final_value);
    }

    #[cfg(target_os = "linux")]
    if !env.contains_key("LD_LIBRARY_PATH") {
        env.insert("LD_LIBRARY_PATH".to_string(), String::new());
    }

    env
}
