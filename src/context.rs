use crate::{config_paths, config_store};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextsFile {
    pub schema_version: u32,
    #[serde(default)]
    pub contexts: Vec<ContextEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextEntry {
    pub id: String,
    pub enabled: bool,
    pub kind: String,        // project, goal, learning, personal, work, custom
    pub send_policy: String, // always, manual, session_pinned, triggered_strict, never, disabled
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub required_any: Vec<String>,
    #[serde(default)]
    pub negative_triggers: Vec<String>,
    pub min_score: u32,
    #[serde(default)]
    pub facts: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
}

impl ContextsFile {
    pub fn validate(&self) -> anyhow::Result<()> {
        let mut ids = std::collections::HashSet::new();
        for entry in &self.contexts {
            if entry.id.is_empty() {
                return Err(anyhow::anyhow!("Context ID cannot be empty."));
            }
            if entry.id != entry.id.to_lowercase() || entry.id.contains(' ') {
                return Err(anyhow::anyhow!(
                    "Context ID '{}' must be lowercase and snake_case.",
                    entry.id
                ));
            }
            if !ids.insert(entry.id.clone()) {
                return Err(anyhow::anyhow!(
                    "Duplicate Context ID found: '{}'.",
                    entry.id
                ));
            }
            let valid_kinds = ["project", "goal", "learning", "personal", "work", "custom"];
            if !valid_kinds.contains(&entry.kind.as_str()) {
                return Err(anyhow::anyhow!(
                    "Invalid kind '{}' in context '{}'. Allowed: project, goal, learning, personal, work, custom.",
                    entry.kind, entry.id
                ));
            }
            let valid_policies = [
                "always",
                "manual",
                "session_pinned",
                "triggered_strict",
                "never",
                "disabled",
            ];
            if !valid_policies.contains(&entry.send_policy.as_str()) {
                return Err(anyhow::anyhow!(
                    "Invalid send_policy '{}' in context '{}'. Allowed: always, manual, session_pinned, triggered_strict, never, disabled.",
                    entry.send_policy, entry.id
                ));
            }
            if entry.send_policy == "triggered_strict" {
                if entry.triggers.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Triggered_strict context '{}' must have at least one trigger keyword.",
                        entry.id
                    ));
                }
                if entry.min_score < 1 {
                    return Err(anyhow::anyhow!(
                        "Triggered_strict context '{}' min_score must be at least 1.",
                        entry.id
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Finds the OS-specific path where the contexts.toml should reside.
pub fn get_contexts_path() -> anyhow::Result<PathBuf> {
    Ok(config_paths::config_dir()?.join("contexts.toml"))
}

/// Reads the contexts file, parses it from TOML format, and validates it.
pub fn read_contexts() -> anyhow::Result<ContextsFile> {
    let path = get_contexts_path()?;
    if !path.exists() {
        init_contexts()?;
    }
    let contexts: ContextsFile = config_store::read_toml_file(&path)?;
    contexts.validate()?;
    Ok(contexts)
}

/// Saves the contexts file after performing validation checks.
pub fn save_contexts(contexts: &ContextsFile) -> anyhow::Result<()> {
    contexts.validate()?;
    let path = get_contexts_path()?;
    config_store::save_toml_file(&path, contexts)?;
    Ok(())
}

/// Initializes contexts.toml in the standard config path if it doesn't already exist.
pub fn init_contexts() -> anyhow::Result<String> {
    let path = get_contexts_path()?;

    if path.exists() {
        return Ok(format!(
            "Contexts configuration file already exists at:\n  {}\n\nYou can edit it directly.",
            path.display()
        ));
    }

    let default_contexts = ContextsFile {
        schema_version: 1,
        contexts: vec![],
    };

    config_store::save_toml_file(&path, &default_contexts)?;

    Ok(format!(
        "Successfully initialized your OpenNivara contexts configuration at:\n  {}",
        path.display()
    ))
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn contexts_path_uses_isolated_test_config_dir() {
        let _lock = crate::config_paths::TEST_CONFIG_ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        std::env::set_var("OPENNIVARA_TEST_CONFIG_DIR", temp_dir.path());

        let path = get_contexts_path().expect("contexts path");

        assert_eq!(path, temp_dir.path().join("contexts.toml"));
        std::env::remove_var("OPENNIVARA_TEST_CONFIG_DIR");
    }

    #[test]
    #[serial]
    fn init_contexts_creates_empty_clean_state() {
        let _lock = crate::config_paths::TEST_CONFIG_ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        std::env::set_var("OPENNIVARA_TEST_CONFIG_DIR", temp_dir.path());

        init_contexts().expect("init contexts");
        let contexts = read_contexts().expect("read contexts");

        assert!(contexts.contexts.is_empty());
        assert!(!std::fs::read_to_string(get_contexts_path().unwrap())
            .unwrap()
            .contains("opennivara_project"));

        std::env::remove_var("OPENNIVARA_TEST_CONFIG_DIR");
    }
}
