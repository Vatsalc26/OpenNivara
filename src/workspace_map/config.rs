use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// The root struct representing the workspace map settings (map.toml).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapConfig {
    pub general: GeneralConfig,
    pub database: DatabaseConfig,
    pub roots: RootsConfig,
    pub ignore: IgnoreConfig,
    pub include: IncludeConfig,
    pub legend: LegendConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub enabled: bool,
    pub max_depth: u32,
    pub max_files: u32,
    pub max_file_size_bytes: u64,
    pub follow_symlinks: bool,
    pub store_content: bool,
    pub store_previews: bool,
    pub respect_gitignore: bool,
    pub skip_hidden: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RootsConfig {
    pub allowed_roots: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgnoreConfig {
    pub blocked_globs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncludeConfig {
    pub extensions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegendConfig {
    pub directory: String,
    pub rust: String,
    pub config: String,
    pub document: String,
    pub image: String,
    pub pdf: String,
    pub data: String,
    pub blocked: String,
    pub ignored: String,
    pub unknown: String,
}

/// Finds the OS-specific path where the map.toml should reside.
pub fn get_map_path() -> anyhow::Result<PathBuf> {
    Ok(crate::config_paths::config_dir()?.join("map.toml"))
}

/// Initializes map.toml in the standard config path.
/// Does not overwrite if the file already exists.
pub fn init_map() -> anyhow::Result<String> {
    let path = get_map_path()?;

    if path.exists() {
        return Ok(format!(
            "Map configuration file already exists at:\n  {}\n\nYou can edit it directly with any text editor.",
            path.display()
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Failed to create map parent directories: {}", e))?;
    }

    let default_map_toml = r#"[general]
enabled = true
max_depth = 6
max_files = 20000
max_file_size_bytes = 5000000
follow_symlinks = false
store_content = false
store_previews = false
respect_gitignore = true
skip_hidden = true

[database]
filename = "workspace_map.sqlite"

[roots]
allowed_roots = [
  "."
]

[ignore]
blocked_globs = [
  "**/.env",
  "**/.env.*",
  "**/.ssh/**",
  "**/*secret*",
  "**/*secrets*",
  "**/*token*",
  "**/*password*",
  "**/*credential*",
  "**/*credentials*",
  "**/*.pem",
  "**/*.key",
  "**/target/**",
  "**/.git/**",
  "**/node_modules/**",
  "**/.cache/**"
]

[include]
extensions = [
  "rs",
  "toml",
  "md",
  "txt",
  "json",
  "yaml",
  "yml",
  "png",
  "jpg",
  "jpeg",
  "webp",
  "gif",
  "pdf"
]

[legend]
directory = "📁"
rust = "🦀"
config = "⚙️"
document = "📝"
image = "🖼️"
pdf = "📄"
data = "🧾"
blocked = "🚫"
ignored = "🙈"
unknown = "❓"
"#;

    fs::write(&path, default_map_toml)
        .map_err(|e| anyhow::anyhow!("Failed to write default map.toml: {}", e))?;

    Ok(format!(
        "Successfully initialized your OpenNivara map configuration at:\n  {}",
        path.display()
    ))
}

/// Reads and parses the map configuration file.
pub fn read_map() -> anyhow::Result<MapConfig> {
    let path = get_map_path()?;

    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Map configuration file not found at:\n  {}\n\nPlease run: opennivara init-map",
            path.display()
        ));
    }

    let content =
        fs::read_to_string(&path).map_err(|e| anyhow::anyhow!("Failed to read map.toml: {}", e))?;

    let config: MapConfig = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!(
            "Failed to parse map TOML. Please ensure your map file is formatted correctly.\nError: {}",
            e
        ))?;

    Ok(config)
}
