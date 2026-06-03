use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSnippet {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub prompt: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsFile {
    pub schema_version: u32,
    pub commands: Vec<CommandSnippet>,
}

/// Aggregates all command snippets loaded from packs that are enabled.
pub fn get_active_command_snippets() -> anyhow::Result<Vec<CommandSnippet>> {
    let settings = super::addon_settings::read_addon_settings()?;
    let packs_dir = super::get_packs_dir()?;
    let installed_list = super::packs::list_installed_packs()?;
    let mut snippets = Vec::new();

    for pack_id in settings.enabled_packs {
        let is_enabled = installed_list
            .installed
            .iter()
            .any(|p| p.id == pack_id && p.enabled);
        if !is_enabled {
            continue;
        }
        let commands_path = packs_dir.join(&pack_id).join("commands.toml");
        if commands_path.exists() {
            if let Ok(content) = fs::read_to_string(&commands_path) {
                if let Ok(file_data) = toml::from_str::<CommandsFile>(&content) {
                    for snippet in file_data.commands {
                        let key = format!("{}:quick_prompt:{}", pack_id, snippet.id);
                        let is_disabled = settings.disabled_contributions.contains(&key);
                        if !is_disabled {
                            snippets.push(snippet);
                        }
                    }
                }
            }
        }
    }

    Ok(snippets)
}
