use std::collections::HashMap;
use std::path::Path;

use crate::install::JavaInstall;
use crate::version::JavaVersion;

fn default_java_path() -> &'static str {
    #[cfg(target_os = "windows")]
    { "javaw" }
    #[cfg(not(target_os = "windows"))]
    { "java" }
}

pub fn make_java_ptr(path: String, id: String, arch: String) -> JavaInstall {
    JavaInstall::new(JavaVersion::new(&id), arch, path)
}

pub fn get_default_java() -> JavaInstall {
    JavaInstall::new(
        JavaVersion::new("java"),
        "unknown".to_string(),
        default_java_path().to_string(),
    )
}

fn add_javas_from_env(mut javas: Vec<String>) -> Vec<String> {
    if let Ok(env) = std::env::var("KCRAFT_JAVA_PATHS") {
        #[cfg(target_os = "windows")]
        let paths: Vec<&str> = env.replace('\\', "/").split(';').collect();
        #[cfg(not(target_os = "windows"))]
        let paths: Vec<&str> = env.split(':').collect();

        for p in paths {
            let p = p.trim();
            if !p.is_empty() && !javas.contains(&p.to_string()) {
                javas.push(p.to_string());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(path_env) = std::env::var("PATH") {
            let paths = path_env.replace('\\', "/");
            for p in paths.split(';') {
                let candidate = format!("{}/javaw.exe", p);
                if !javas.contains(&candidate) {
                    javas.push(candidate);
                }
            }
        }
    }

    javas
}

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
                    let delimiter = if cfg!(target_os = "windows") { ';' } else { ':' };
                    let target_items: Vec<&str> = value.split(delimiter).collect();
                    let to_remove: Vec<&str> = launcher_val.split(delimiter).collect();

                    let filtered: Vec<&str> = target_items
                        .into_iter()
                        .filter(|item| !to_remove.contains(item))
                        .collect();
                    final_value = filtered.join(&delimiter.to_string());
                    tracing::debug!("Env: stripped {} from {} to {}", strip_key, value, final_value);
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

fn scan_java_dir(dir_path: &Path) -> Vec<String> {
    let mut javas = Vec::new();
    if !dir_path.exists() {
        return javas;
    }

    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() || path.is_symlink() {
                continue;
            }

            let jre_bin = path.join("jre/bin/java");
            let bin = path.join("bin/java");

            if jre_bin.exists() {
                javas.push(jre_bin.to_string_lossy().to_string());
            }
            if bin.exists() {
                javas.push(bin.to_string_lossy().to_string());
            }
        }
    }
    javas
}

#[cfg(target_os = "linux")]
pub fn find_java_paths() -> Vec<String> {
    let mut javas = Vec::new();
    javas.push(get_default_java().path.clone());

    let scan_dirs = [
        "/usr/java",
        "/usr/lib/jvm",
        "/usr/lib64/jvm",
        "/usr/lib32/jvm",
        "/usr/lib64",
        "/usr/lib",
        "/opt",
        "/opt/jdk",
        "/opt/jdks",
        "/app/jdk",
    ];

    for dir in &scan_dirs {
        javas.extend(scan_java_dir(Path::new(dir)));
    }

    let java_path = "java";
    javas.extend(scan_java_dir(Path::new(java_path)));

    if let Ok(appimage) = std::env::var("APPIMAGE") {
        javas.extend(scan_java_dir(&Path::new(&appimage).join("usr/lib/jvm")));
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let sdkman_dir = std::env::var("SDKMAN_DIR")
        .unwrap_or_else(|_| format!("{}/.sdkman", home));
    javas.extend(scan_java_dir(&Path::new(&sdkman_dir).join("candidates/java")));

    let asdf_dir = std::env::var("ASDF_DATA_DIR")
        .or_else(|_| std::env::var("ASDF_DIR"))
        .unwrap_or_else(|_| format!("{}/.asdf", home));
    javas.extend(scan_java_dir(&Path::new(&asdf_dir).join("installs/java")));

    javas.sort();
    javas.dedup();
    add_javas_from_env(javas)
}

#[cfg(target_os = "macos")]
pub fn find_java_paths() -> Vec<String> {
    let mut javas = Vec::new();
    javas.push(get_default_java().path.clone());

    let hardcoded = [
        "/Applications/Xcode.app/Contents/Applications/Application Loader.app/Contents/MacOS/itms/java/bin/java",
        "/Library/Internet Plug-Ins/JavaAppletPlugin.plugin/Contents/Home/bin/java",
        "/System/Library/Frameworks/JavaVM.framework/Versions/Current/Commands/java",
    ];

    for p in &hardcoded {
        if Path::new(p).exists() {
            javas.push(p.to_string());
        }
    }

    let library_jvm = Path::new("/Library/Java/JavaVirtualMachines/");
    if let Ok(entries) = std::fs::read_dir(library_jvm) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            javas.push(format!("{}/{}/Contents/Home/bin/java", library_jvm.display(), name));
            javas.push(format!("{}/{}/Contents/Home/jre/bin/java", library_jvm.display(), name));
        }
    }

    let system_jvm = Path::new("/System/Library/Java/JavaVirtualMachines/");
    if let Ok(entries) = std::fs::read_dir(system_jvm) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            javas.push(format!("{}/{}/Contents/Home/bin/java", system_jvm.display(), name));
            javas.push(format!("{}/{}/Contents/Commands/java", system_jvm.display(), name));
        }
    }

    javas.sort();
    javas.dedup();
    add_javas_from_env(javas)
}

#[cfg(target_os = "windows")]
pub fn find_java_paths() -> Vec<String> {
    let mut javas: Vec<String> = Vec::new();

    fn add_from_reg(key_type: u32, key_name: &str, value_name: &str, suffix: &str) -> Vec<String> {
        let mut results = Vec::new();

        let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
        let flags = key_type | winreg::enums::KEY_READ;

        if let Ok(key) = hklm.open_subkey_with_flags(key_name, flags) {
            for name in key.enum_keys().filter_map(|k| k.ok()) {
                let subkey_path = format!("{}\\{}{}", key_name, name, suffix);
                if let Ok(subkey) = hklm.open_subkey_with_flags(&subkey_path, flags) {
                    if let Ok(val) = subkey.get_value::<String, _>(value_name) {
                        let path = format!("{}/bin/javaw.exe", val.replace('\\', "/"));
                        if !results.contains(&path) {
                            results.push(path);
                        }
                    }
                }
            }
        }
        results
    }

    let registry_keys: &[(u32, &str, &str, &str)] = &[
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\JavaSoft\\Java Runtime Environment", "JavaHome", ""),
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\JavaSoft\\Java Development Kit", "JavaHome", ""),
        (winreg::enums::KEY_WOW64_32KEY, "SOFTWARE\\JavaSoft\\Java Runtime Environment", "JavaHome", ""),
        (winreg::enums::KEY_WOW64_32KEY, "SOFTWARE\\JavaSoft\\Java Development Kit", "JavaHome", ""),
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\JavaSoft\\JRE", "JavaHome", ""),
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\JavaSoft\\JDK", "JavaHome", ""),
        (winreg::enums::KEY_WOW64_32KEY, "SOFTWARE\\JavaSoft\\JRE", "JavaHome", ""),
        (winreg::enums::KEY_WOW64_32KEY, "SOFTWARE\\JavaSoft\\JDK", "JavaHome", ""),
        (winreg::enums::KEY_WOW64_32KEY, "SOFTWARE\\AdoptOpenJDK\\JRE", "Path", "\\hotspot\\MSI"),
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\AdoptOpenJDK\\JRE", "Path", "\\hotspot\\MSI"),
        (winreg::enums::KEY_WOW64_32KEY, "SOFTWARE\\AdoptOpenJDK\\JDK", "Path", "\\hotspot\\MSI"),
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\AdoptOpenJDK\\JDK", "Path", "\\hotspot\\MSI"),
        (winreg::enums::KEY_WOW64_32KEY, "SOFTWARE\\Eclipse Foundation\\JDK", "Path", "\\hotspot\\MSI"),
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\Eclipse Foundation\\JDK", "Path", "\\hotspot\\MSI"),
        (winreg::enums::KEY_WOW64_32KEY, "SOFTWARE\\Eclipse Adoptium\\JRE", "Path", "\\hotspot\\MSI"),
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\Eclipse Adoptium\\JRE", "Path", "\\hotspot\\MSI"),
        (winreg::enums::KEY_WOW64_32KEY, "SOFTWARE\\Eclipse Adoptium\\JDK", "Path", "\\hotspot\\MSI"),
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\Eclipse Adoptium\\JDK", "Path", "\\hotspot\\MSI"),
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\Microsoft\\JDK", "Path", "\\hotspot\\MSI"),
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\Azul Systems\\Zulu", "InstallationPath", ""),
        (winreg::enums::KEY_WOW64_32KEY, "SOFTWARE\\Azul Systems\\Zulu", "InstallationPath", ""),
        (winreg::enums::KEY_WOW64_64KEY, "SOFTWARE\\BellSoft\\Liberica", "InstallationPath", ""),
        (winreg::enums::KEY_WOW64_32KEY, "SOFTWARE\\BellSoft\\Liberica", "InstallationPath", ""),
    ];

    for &(key_type, key_name, value_name, suffix) in registry_keys {
        javas.extend(add_from_reg(key_type, key_name, value_name, suffix));
    }

    let hardcoded = [
        r"C:/Program Files/Java/jre8/bin/javaw.exe",
        r"C:/Program Files/Java/jre7/bin/javaw.exe",
        r"C:/Program Files/Java/jre6/bin/javaw.exe",
        r"C:/Program Files (x86)/Java/jre8/bin/javaw.exe",
        r"C:/Program Files (x86)/Java/jre7/bin/javaw.exe",
        r"C:/Program Files (x86)/Java/jre6/bin/javaw.exe",
    ];

    for p in &hardcoded {
        if !javas.contains(&p.to_string()) && Path::new(p).exists() {
            javas.push(p.to_string());
        }
    }

    if !javas.contains(&default_java_path().to_string()) {
        javas.push(default_java_path().to_string());
    }

    javas.sort();
    javas.dedup();
    add_javas_from_env(javas)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn find_java_paths() -> Vec<String> {
    let mut javas = Vec::new();
    javas.push(get_default_java().path.clone());
    add_javas_from_env(javas)
}
