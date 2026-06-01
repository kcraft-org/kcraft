use crate::install::JavaInstall;
use crate::version::JavaVersion;

pub(super) fn default_java_path() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "javaw"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "java"
    }
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
