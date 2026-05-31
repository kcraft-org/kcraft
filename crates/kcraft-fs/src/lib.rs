use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("Path not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

pub type Result<T> = std::result::Result<T, FsError>;

pub fn write(path: impl AsRef<Path>, data: &[u8]) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        ensure_folder_exists(parent)?;
    }
    let tmp_path = path.with_extension("tmp");
    {
        let mut tmp = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)?;
        tmp.write_all(data)?;
        tmp.sync_all()?;
    }
    fs::rename(&tmp_path, path)?;
    Ok(())
}

pub fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let mut file = File::open(path.as_ref())?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

pub fn read_to_string(path: impl AsRef<Path>) -> Result<String> {
    let data = read(path)?;
    String::from_utf8(data).map_err(|e| FsError::InvalidPath(e.to_string()))
}

pub fn ensure_folder_exists(path: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(path.as_ref())?;
    Ok(())
}

pub fn ensure_file_path_exists(path: impl AsRef<Path>) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        ensure_folder_exists(parent)?;
    }
    Ok(())
}

pub fn update_timestamp(path: impl AsRef<Path>) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| FsError::Io(io::Error::other(e)))?;
    filetime::set_file_mtime(path.as_ref(), filetime::FileTime::from_unix_time(now.as_secs() as i64, 0))
        .map_err(FsError::Io)
}

pub fn delete(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    if src.is_dir() {
        copy_dir_recursive(src, dst, &[])?;
    } else {
        ensure_file_path_exists(dst)?;
        fs::copy(src, dst)?;
    }
    Ok(())
}

pub type PathPredicate = Box<dyn Fn(&Path) -> bool>;

pub fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    blacklist: &[PathPredicate],
) -> Result<()> {
    ensure_folder_exists(dst)?;
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(src).unwrap();
        let dest_path = dst.join(relative);
        let should_skip = blacklist.iter().any(|f| f(entry.path()));
        if should_skip {
            continue;
        }
        if entry.file_type().is_dir() {
            ensure_folder_exists(&dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

pub fn path_combine(base: impl AsRef<Path>, part: impl AsRef<Path>) -> PathBuf {
    let base = base.as_ref();
    let part = part.as_ref();
    let combined = if part.is_absolute() {
        part.canonicalize().unwrap_or_else(|_| part.to_path_buf())
    } else {
        base.join(part)
    };
    path_clean::clean(&combined)
}

pub fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let cleaned = path_clean::clean(path);
    if cleaned.is_relative() {
        if let Ok(cwd) = std::env::current_dir() {
            let abs = cwd.join(&cleaned);
            path_clean::clean(&abs)
        } else {
            cleaned
        }
    } else {
        cleaned
    }
}

pub fn dir_name_from_string(name: &str, in_dir: impl AsRef<Path>) -> PathBuf {
    let in_dir = in_dir.as_ref();
    let sanitized: String = name
        .chars()
        .map(|c| {
            if "\\/?<>:;*|!+\r\n".contains(c) {
                '-'
            } else {
                c
            }
        })
        .collect();
    let mut candidate = in_dir.join(&sanitized);
    let mut counter = 1;
    while candidate.exists() {
        candidate = in_dir.join(format!("{} ({})", sanitized, counter));
        counter += 1;
    }
    candidate
}

pub fn remove_invalid_filename_chars(name: &str, substitute: char) -> String {
    name.chars()
        .map(|c| {
            if "\\/?<>:;*|!+\r\n".contains(c) {
                substitute
            } else {
                c
            }
        })
        .collect()
}

pub fn resolve_executable(name_or_path: impl AsRef<Path>) -> Option<PathBuf> {
    let path = name_or_path.as_ref();
    if path.is_absolute() && path.is_file() {
        return Some(path.to_path_buf());
    }
    let path_str = path.to_string_lossy();
    if !path_str.contains(std::path::MAIN_SEPARATOR) {
        if let Ok(found) = which::which(path_str.as_ref()) {
            return Some(found);
        }
    }
    if path.is_file() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = path.metadata() {
                if meta.permissions().mode() & 0o111 != 0 {
                    return Some(path.to_path_buf());
                }
            }
        }
        #[cfg(not(unix))]
        {
            return Some(path.to_path_buf());
        }
    }
    None
}

pub fn get_desktop_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let xdg = PathBuf::from(&home).join("Desktop");
            if xdg.exists() {
                return xdg;
            }
            return PathBuf::from(&home).join("Desktop");
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            return PathBuf::from(profile).join("Desktop");
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(&home).join("Desktop");
        }
    }
    PathBuf::from(".")
}

pub fn merge_folders(dst: impl AsRef<Path>, src: impl AsRef<Path>) -> Result<()> {
    let dst = dst.as_ref();
    let src = src.as_ref();
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(src).unwrap();
        let dest_path = dst.join(relative);
        if entry.file_type().is_dir() {
            ensure_folder_exists(&dest_path)?;
        } else {
            ensure_file_path_exists(&dest_path)?;
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

pub fn check_problematic_path_java(path: impl AsRef<Path>) -> bool {
    path.as_ref().to_string_lossy().contains('!')
}
