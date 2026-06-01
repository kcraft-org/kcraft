pub fn read_zip_entry<R: std::io::Read>(mut entry: R) -> Result<String, String> {
    let mut content = String::new();
    entry
        .read_to_string(&mut content)
        .map_err(|e| e.to_string())?;
    Ok(content)
}
