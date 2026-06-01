use std::path::Path;

use super::default::get_default_java;

#[cfg(target_os = "windows")]
use super::default::default_java_path;

fn add_javas_from_env(mut javas: Vec<String>) -> Vec<String> {
    if let Ok(env) = std::env::var("KCRAFT_JAVA_PATHS") {
        #[cfg(target_os = "windows")]
        let env_normalized = env.replace('\\', "/");
        #[cfg(target_os = "windows")]
        let paths: Vec<&str> = env_normalized.split(';').collect();
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
    let sdkman_dir = std::env::var("SDKMAN_DIR").unwrap_or_else(|_| format!("{}/.sdkman", home));
    javas.extend(scan_java_dir(
        &Path::new(&sdkman_dir).join("candidates/java"),
    ));

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
            javas.push(format!(
                "{}/{}/Contents/Home/bin/java",
                library_jvm.display(),
                name
            ));
            javas.push(format!(
                "{}/{}/Contents/Home/jre/bin/java",
                library_jvm.display(),
                name
            ));
        }
    }

    let system_jvm = Path::new("/System/Library/Java/JavaVirtualMachines/");
    if let Ok(entries) = std::fs::read_dir(system_jvm) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            javas.push(format!(
                "{}/{}/Contents/Home/bin/java",
                system_jvm.display(),
                name
            ));
            javas.push(format!(
                "{}/{}/Contents/Commands/java",
                system_jvm.display(),
                name
            ));
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
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\JavaSoft\\Java Runtime Environment",
            "JavaHome",
            "",
        ),
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\JavaSoft\\Java Development Kit",
            "JavaHome",
            "",
        ),
        (
            winreg::enums::KEY_WOW64_32KEY,
            "SOFTWARE\\JavaSoft\\Java Runtime Environment",
            "JavaHome",
            "",
        ),
        (
            winreg::enums::KEY_WOW64_32KEY,
            "SOFTWARE\\JavaSoft\\Java Development Kit",
            "JavaHome",
            "",
        ),
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\JavaSoft\\JRE",
            "JavaHome",
            "",
        ),
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\JavaSoft\\JDK",
            "JavaHome",
            "",
        ),
        (
            winreg::enums::KEY_WOW64_32KEY,
            "SOFTWARE\\JavaSoft\\JRE",
            "JavaHome",
            "",
        ),
        (
            winreg::enums::KEY_WOW64_32KEY,
            "SOFTWARE\\JavaSoft\\JDK",
            "JavaHome",
            "",
        ),
        (
            winreg::enums::KEY_WOW64_32KEY,
            "SOFTWARE\\AdoptOpenJDK\\JRE",
            "Path",
            "\\hotspot\\MSI",
        ),
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\AdoptOpenJDK\\JRE",
            "Path",
            "\\hotspot\\MSI",
        ),
        (
            winreg::enums::KEY_WOW64_32KEY,
            "SOFTWARE\\AdoptOpenJDK\\JDK",
            "Path",
            "\\hotspot\\MSI",
        ),
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\AdoptOpenJDK\\JDK",
            "Path",
            "\\hotspot\\MSI",
        ),
        (
            winreg::enums::KEY_WOW64_32KEY,
            "SOFTWARE\\Eclipse Foundation\\JDK",
            "Path",
            "\\hotspot\\MSI",
        ),
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\Eclipse Foundation\\JDK",
            "Path",
            "\\hotspot\\MSI",
        ),
        (
            winreg::enums::KEY_WOW64_32KEY,
            "SOFTWARE\\Eclipse Adoptium\\JRE",
            "Path",
            "\\hotspot\\MSI",
        ),
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\Eclipse Adoptium\\JRE",
            "Path",
            "\\hotspot\\MSI",
        ),
        (
            winreg::enums::KEY_WOW64_32KEY,
            "SOFTWARE\\Eclipse Adoptium\\JDK",
            "Path",
            "\\hotspot\\MSI",
        ),
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\Eclipse Adoptium\\JDK",
            "Path",
            "\\hotspot\\MSI",
        ),
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\Microsoft\\JDK",
            "Path",
            "\\hotspot\\MSI",
        ),
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\Azul Systems\\Zulu",
            "InstallationPath",
            "",
        ),
        (
            winreg::enums::KEY_WOW64_32KEY,
            "SOFTWARE\\Azul Systems\\Zulu",
            "InstallationPath",
            "",
        ),
        (
            winreg::enums::KEY_WOW64_64KEY,
            "SOFTWARE\\BellSoft\\Liberica",
            "InstallationPath",
            "",
        ),
        (
            winreg::enums::KEY_WOW64_32KEY,
            "SOFTWARE\\BellSoft\\Liberica",
            "InstallationPath",
            "",
        ),
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
