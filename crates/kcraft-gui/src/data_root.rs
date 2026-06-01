use std::path::PathBuf;

pub fn data_root() -> PathBuf {
    let path = dirs::data_dir().map(|d| d.join("kcraft"));
    match path {
        Some(p) => p,
        None => {
            let fallback = dirs::home_dir()
                .map(|h| h.join(".local").join("share").join("kcraft"))
                .unwrap_or_else(|| PathBuf::from("kcraft_data"));
            tracing::warn!(
                "Could not determine platform data directory, falling back to: {}",
                fallback.display()
            );
            fallback
        }
    }
}
