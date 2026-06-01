use std::path::Path;

pub(super) fn download_file(url: &str, path: &Path) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("HTTP error: {}", e))?
        .error_for_status()
        .map_err(|e| format!("HTTP status error: {}", e))?;

    let mut file = std::fs::File::create(path).map_err(|e| format!("File error: {}", e))?;

    let content = response.bytes().map_err(|e| format!("Read error: {}", e))?;

    std::io::copy(&mut content.as_ref(), &mut file).map_err(|e| format!("IO error: {}", e))?;

    Ok(())
}
