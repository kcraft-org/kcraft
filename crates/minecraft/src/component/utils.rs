pub fn components_file_path(instance_root: &str) -> String {
    std::path::Path::new(instance_root)
        .join("mmc-pack.json")
        .to_string_lossy()
        .to_string()
}

pub fn patches_pattern(instance_root: &str) -> String {
    std::path::Path::new(instance_root)
        .join("patches")
        .join("*.json")
        .to_string_lossy()
        .to_string()
}
