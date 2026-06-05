use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct ApiKeyStatus {
    pub available: bool,
    pub source: Option<String>,
    pub storage_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecretsFile {
    #[serde(default)]
    gemini_api_key: String,
}

pub fn secrets_path() -> anyhow::Result<PathBuf> {
    Ok(crate::config_paths::config_dir()?.join("secrets.toml"))
}

pub fn get_gemini_api_key() -> anyhow::Result<String> {
    if let Ok(key) = std::env::var("GEMINI_API_KEY") {
        let key = key.trim().to_string();
        if !key.is_empty() {
            return Ok(key);
        }
    }

    let path = secrets_path()?;
    if path.exists() {
        let file: SecretsFile = crate::config_store::read_toml_file(&path)?;
        let key = file.gemini_api_key.trim().to_string();
        if !key.is_empty() {
            return Ok(key);
        }
    }

    Err(anyhow::anyhow!(
        "Missing Gemini API key. Add it in desktop onboarding/settings or set GEMINI_API_KEY."
    ))
}

pub fn gemini_key_status() -> anyhow::Result<ApiKeyStatus> {
    if let Ok(key) = std::env::var("GEMINI_API_KEY") {
        if !key.trim().is_empty() {
            return Ok(ApiKeyStatus {
                available: true,
                source: Some("environment".to_string()),
                storage_note: "Loaded from GEMINI_API_KEY environment variable.".to_string(),
            });
        }
    }

    let path = secrets_path()?;
    if path.exists() {
        let file: SecretsFile = crate::config_store::read_toml_file(&path)?;
        if !file.gemini_api_key.trim().is_empty() {
            return Ok(ApiKeyStatus {
                available: true,
                source: Some("local_config".to_string()),
                storage_note:
                    "Saved in local alpha config storage. OS keychain storage is future work."
                        .to_string(),
            });
        }
    }

    Ok(ApiKeyStatus {
        available: false,
        source: None,
        storage_note: "No Gemini API key configured.".to_string(),
    })
}

pub fn save_gemini_key(secret: &str) -> anyhow::Result<()> {
    let key = secret.trim();
    if key.is_empty() {
        return Err(anyhow::anyhow!("Gemini API key cannot be empty."));
    }
    let path = secrets_path()?;
    crate::config_store::save_toml_file(
        &path,
        &SecretsFile {
            gemini_api_key: key.to_string(),
        },
    )?;
    Ok(())
}
