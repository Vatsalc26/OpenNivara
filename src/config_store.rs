use std::fs;
use std::path::Path;

/// Generic function to read and deserialize a TOML file into a typed Rust struct.
pub fn read_toml_file<T: serde::de::DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", path.display()));
    }
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read TOML file '{}': {}", path.display(), e))?;
    let data: T = toml::from_str(&content).map_err(|e| {
        anyhow::anyhow!("Failed to parse TOML format in '{}': {}", path.display(), e)
    })?;
    Ok(data)
}

/// Generic function to serialize a typed Rust struct and save it as a TOML file.
pub fn save_toml_file<T: serde::Serialize>(path: &Path, data: &T) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create directories for '{}': {}",
                path.display(),
                e
            )
        })?;
    }
    let content = toml::to_string_pretty(data)
        .map_err(|e| anyhow::anyhow!("Failed to serialize TOML data: {}", e))?;
    fs::write(path, content)
        .map_err(|e| anyhow::anyhow!("Failed to write TOML file '{}': {}", path.display(), e))?;
    Ok(())
}
