use crate::context::ContextEntry;
use crate::preferences::PreferenceSection;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePackBundle {
    pub active_mode_id: String,
    pub active_mode_name: String,
    pub active_pack_ids: Vec<String>,
    pub preferences: Vec<PreferenceSection>,
    pub contexts: Vec<ContextEntry>,
    pub theme_id: Option<String>,
    pub style_pack_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonContributionPreview {
    pub pack_id: String,
    pub pack_name: String,
    pub contribution_id: String,
    pub title: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickPromptContributionPreview {
    pub pack_id: String,
    pub pack_name: String,
    pub prompt_id: String,
    pub title: String,
    pub description: String,
    pub prompt_body: String,
    pub category: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveSettingsPreview {
    pub base_preferences: Vec<PreferenceSection>,
    pub addon_preferences: Vec<AddonContributionPreview>,

    pub base_contexts: Vec<ContextEntry>,
    pub addon_contexts: Vec<AddonContributionPreview>,

    pub addon_quick_prompts: Vec<QuickPromptContributionPreview>,

    pub active_theme_id: Option<String>,
    pub active_theme_name: Option<String>,
    pub active_theme_source_pack_id: Option<String>,

    pub active_style_pack_id: Option<String>,
    pub active_style_pack_name: Option<String>,

    pub disabled_contributions: Vec<String>,
    pub enabled_packs: Vec<String>,
}

/// Dynamic context bundle assembler. Walks through all packs activated by the addon settings,
/// and aggregates any optional preferences or contexts they declare that are not explicitly disabled.
pub fn get_active_pack_bundle() -> anyhow::Result<ActivePackBundle> {
    let settings = super::addon_settings::read_addon_settings()?;
    let packs_dir = super::get_packs_dir()?;
    let mut preferences = Vec::new();
    let mut contexts = Vec::new();

    let installed_file = super::packs::list_installed_packs().ok();

    // Iterate through active pack ids
    for pack_id in &settings.enabled_packs {
        // Ensure pack is installed and enabled
        if let Some(ref inst) = installed_file {
            if let Some(pack) = inst.installed.iter().find(|p| p.id == *pack_id) {
                if !pack.enabled {
                    continue; // Skip disabled packs
                }
            } else {
                continue; // Skip if missing/not installed
            }
        }

        let pack_folder = packs_dir.join(pack_id);
        if !pack_folder.exists() {
            continue; // Skip if physical folder is missing
        }

        // 1. Load preferences
        let pref_path = pack_folder.join("preferences.toml");
        if pref_path.exists() {
            if let Ok(content) = fs::read_to_string(&pref_path) {
                if let Ok(file_data) =
                    toml::from_str::<crate::preferences::PreferencesFile>(&content)
                {
                    for section in file_data.sections {
                        let key = format!("{}:preference:{}", pack_id, section.id);
                        let is_disabled = settings.disabled_contributions.contains(&key);
                        if section.enabled && !is_disabled {
                            preferences.push(section);
                        }
                    }
                }
            }
        }

        // 2. Load contexts
        let ctx_path = pack_folder.join("contexts.toml");
        if ctx_path.exists() {
            if let Ok(content) = fs::read_to_string(&ctx_path) {
                if let Ok(file_data) = toml::from_str::<crate::context::ContextsFile>(&content) {
                    for entry in file_data.contexts {
                        let key = format!("{}:context:{}", pack_id, entry.id);
                        let is_disabled = settings.disabled_contributions.contains(&key);
                        if entry.enabled && !is_disabled {
                            contexts.push(entry);
                        }
                    }
                }
            }
        }
    }

    // Determine the style pack ID (e.g. find first enabled pack that has style and isn't disabled in contributions)
    let mut active_style_pack_id = None;
    for pack_id in &settings.enabled_packs {
        let pack_folder = packs_dir.join(pack_id);
        if pack_folder.join("style.toml").exists() {
            let key = format!("{}:style:style", pack_id);
            if !settings.disabled_contributions.contains(&key) {
                active_style_pack_id = Some(pack_id.clone());
                break;
            }
        }
    }

    Ok(ActivePackBundle {
        active_mode_id: "addons".to_string(),
        active_mode_name: "Addons Config".to_string(),
        active_pack_ids: settings.enabled_packs.clone(),
        preferences,
        contexts,
        theme_id: settings.active_theme_id.clone(),
        style_pack_id: active_style_pack_id,
    })
}

/// Dynamic effective settings preview generator for user-facing Android-like Settings view.
pub fn get_effective_settings_preview() -> anyhow::Result<EffectiveSettingsPreview> {
    let settings = super::addon_settings::read_addon_settings()?;
    let base_prefs = crate::preferences::read_preferences()?.sections;
    let base_contexts = crate::context::read_contexts()?.contexts;

    let mut addon_preferences = Vec::new();
    let mut addon_contexts = Vec::new();
    let mut addon_quick_prompts = Vec::new();

    let installed_file = super::packs::list_installed_packs()?;
    let packs_dir = super::get_packs_dir()?;

    for pack in &installed_file.installed {
        let pack_folder = packs_dir.join(&pack.id);
        if !pack_folder.exists() {
            continue;
        }

        let is_pack_enabled = settings.enabled_packs.contains(&pack.id) && pack.enabled;

        // 1. Load preferences
        let pref_path = pack_folder.join("preferences.toml");
        if pref_path.exists() {
            if let Ok(content) = fs::read_to_string(&pref_path) {
                if let Ok(file_data) =
                    toml::from_str::<crate::preferences::PreferencesFile>(&content)
                {
                    for section in file_data.sections {
                        let key = format!("{}:preference:{}", pack.id, section.id);
                        let is_disabled = settings.disabled_contributions.contains(&key);
                        addon_preferences.push(AddonContributionPreview {
                            pack_id: pack.id.clone(),
                            pack_name: pack.name.clone(),
                            contribution_id: section.id.clone(),
                            title: section.id.clone(),
                            description: section.description.unwrap_or_default(),
                            enabled: is_pack_enabled && !is_disabled,
                        });
                    }
                }
            }
        }

        // 2. Load contexts
        let ctx_path = pack_folder.join("contexts.toml");
        if ctx_path.exists() {
            if let Ok(content) = fs::read_to_string(&ctx_path) {
                if let Ok(file_data) = toml::from_str::<crate::context::ContextsFile>(&content) {
                    for entry in file_data.contexts {
                        let key = format!("{}:context:{}", pack.id, entry.id);
                        let is_disabled = settings.disabled_contributions.contains(&key);
                        addon_contexts.push(AddonContributionPreview {
                            pack_id: pack.id.clone(),
                            pack_name: pack.name.clone(),
                            contribution_id: entry.id.clone(),
                            title: entry.title.clone(),
                            description: entry.summary.clone(),
                            enabled: is_pack_enabled && !is_disabled,
                        });
                    }
                }
            }
        }

        // 3. Load quick prompts
        let commands_path = pack_folder.join("commands.toml");
        if commands_path.exists() {
            if let Ok(content) = fs::read_to_string(&commands_path) {
                if let Ok(file_data) = toml::from_str::<super::snippets::CommandsFile>(&content) {
                    for cmd in file_data.commands {
                        let key = format!("{}:quick_prompt:{}", pack.id, cmd.id);
                        let is_disabled = settings.disabled_contributions.contains(&key);
                        addon_quick_prompts.push(QuickPromptContributionPreview {
                            pack_id: pack.id.clone(),
                            pack_name: pack.name.clone(),
                            prompt_id: cmd.id.clone(),
                            title: cmd.title.clone(),
                            description: cmd.description.clone(),
                            prompt_body: cmd.prompt.clone(),
                            category: cmd.category.clone(),
                            enabled: is_pack_enabled && !is_disabled,
                        });
                    }
                }
            }
        }
    }

    // Theme name check
    let mut active_theme_name = None;
    if settings.active_theme_id.is_some() {
        if let Some(ref source_pack) = settings.active_theme_source_pack_id {
            let theme_path = packs_dir.join(source_pack).join("theme.toml");
            if theme_path.exists() {
                if let Ok(content) = fs::read_to_string(&theme_path) {
                    if let Ok(theme) = toml::from_str::<super::themes::OpenNivaraTheme>(&content) {
                        active_theme_name = Some(theme.name);
                    }
                }
            }
        }
    }

    // Style pack name check
    let mut active_style_pack_id = None;
    let mut active_style_pack_name = None;
    for pack in &installed_file.installed {
        let pack_folder = packs_dir.join(&pack.id);
        if pack_folder.join("style.toml").exists() {
            let is_pack_enabled = settings.enabled_packs.contains(&pack.id) && pack.enabled;
            let key = format!("{}:style:style", pack.id);
            let is_disabled = settings.disabled_contributions.contains(&key);
            if is_pack_enabled && !is_disabled {
                active_style_pack_id = Some(pack.id.clone());
                active_style_pack_name = Some(pack.name.clone());
                break;
            }
        }
    }

    Ok(EffectiveSettingsPreview {
        base_preferences: base_prefs,
        addon_preferences,
        base_contexts,
        addon_contexts,
        addon_quick_prompts,
        active_theme_id: settings.active_theme_id.clone(),
        active_theme_name,
        active_theme_source_pack_id: settings.active_theme_source_pack_id.clone(),
        active_style_pack_id,
        active_style_pack_name,
        disabled_contributions: settings.disabled_contributions.clone(),
        enabled_packs: settings.enabled_packs.clone(),
    })
}
