use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelegramConfig {
    pub general: GeneralConfig,
    pub auth: AuthConfig,
    pub permissions: PermissionsConfig,
    pub confirmations: ConfirmationsConfig,
    pub limits: LimitsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub enabled: bool,
    pub mode: String,
    pub bot_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConfig {
    pub allowed_chat_ids: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionsConfig {
    pub allow_ask: bool,
    pub allow_chat: bool,
    pub allow_status: bool,
    pub allow_sessions: bool,
    pub allow_map_summary: bool,
    pub allow_map_search: bool,
    pub allow_map_tree: bool,
    pub allow_map_get_node: bool,
    pub allow_read_file: bool,
    pub allow_open_app: bool,
    pub allow_open_url: bool,
    pub allow_write_file: bool,
    pub allow_run_command: bool,

    #[serde(default)]
    pub allow_profile_write: bool,
    #[serde(default)]
    pub allow_style_write: bool,
    #[serde(default)]
    pub allow_preferences_write: bool,
    #[serde(default)]
    pub allow_contexts_write: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfirmationsConfig {
    pub require_confirmation_for_read_file: bool,
    pub require_confirmation_for_open_app: bool,
    pub require_confirmation_for_open_url: bool,
    pub require_confirmation_for_any_local_tool: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LimitsConfig {
    pub max_response_chars: usize,
    pub max_file_preview_chars: usize,
    pub max_messages_per_minute: u32,
}

/// Finds the OS-specific path where telegram.toml should reside.
pub fn get_telegram_path() -> anyhow::Result<PathBuf> {
    Ok(crate::config_paths::config_dir()?.join("telegram.toml"))
}

/// Initializes a default telegram.toml template if it does not already exist.
pub fn init_telegram() -> anyhow::Result<String> {
    let path = get_telegram_path()?;

    if path.exists() {
        return Ok(format!(
            "Telegram configuration file already exists at:\n  {}\n\nYou can edit it directly with any text editor.",
            path.display()
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            anyhow::anyhow!("Failed to create Telegram config parent directories: {}", e)
        })?;
    }

    let default_telegram_toml = r#"[general]
enabled = true
mode = "polling"
bot_name = "OpenNivara"

[auth]
allowed_chat_ids = []

[permissions]
allow_ask = true
allow_chat = true
allow_status = true
allow_sessions = true
allow_map_summary = true
allow_map_search = true
allow_map_tree = true
allow_map_get_node = true
allow_read_file = false
allow_open_app = false
allow_open_url = false
allow_write_file = false
allow_run_command = false
allow_profile_write = false
allow_style_write = false
allow_preferences_write = false
allow_contexts_write = false

[confirmations]
require_confirmation_for_read_file = true
require_confirmation_for_open_app = true
require_confirmation_for_open_url = true
require_confirmation_for_any_local_tool = true

[limits]
max_response_chars = 3500
max_file_preview_chars = 2000
max_messages_per_minute = 20
"#;

    fs::write(&path, default_telegram_toml)
        .map_err(|e| anyhow::anyhow!("Failed to write default telegram.toml: {}", e))?;

    Ok(format!(
        "Successfully initialized your OpenNivara Telegram configuration at:\n  {}",
        path.display()
    ))
}

/// Reads the telegram.toml file, parses it from TOML format, and returns the TelegramConfig struct.
pub fn read_telegram() -> anyhow::Result<TelegramConfig> {
    let path = get_telegram_path()?;

    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Telegram configuration file not found at:\n  {}\n\nPlease run: opennivara init-telegram",
            path.display()
        ));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read telegram.toml file: {}", e))?;

    let config: TelegramConfig = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!(
            "Failed to parse telegram TOML. Please ensure your telegram file is formatted correctly.\nError: {}",
            e
        ))?;

    Ok(config)
}

/// Checks if a tool execution is allowed for remote/Telegram requests according to the permissions policy.
pub fn is_tool_allowed(name: &str, config: &TelegramConfig) -> bool {
    // Stricter permissions for remote access
    match name {
        "get_current_dir" => true,
        "list_dir" => true,
        "file_exists" => true,
        "read_file" => config.permissions.allow_read_file,
        "map_summary" => config.permissions.allow_map_summary,
        "map_tree" => config.permissions.allow_map_tree,
        "map_search" => config.permissions.allow_map_search,
        "map_get_node" => config.permissions.allow_map_get_node,
        "open_app" => config.permissions.allow_open_app,
        "open_url" => config.permissions.allow_open_url,
        "write_file" => config.permissions.allow_write_file,
        "run_command" => config.permissions.allow_run_command,
        _ => false,
    }
}
