use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonSettings {
    pub schema_version: u32,
    pub active_theme_id: Option<String>,
    pub active_theme_source_pack_id: Option<String>,
    pub enabled_packs: Vec<String>,
    pub disabled_contributions: Vec<String>, // key format: "pack_id:type:id"
}

pub fn get_addon_settings_path() -> anyhow::Result<PathBuf> {
    Ok(super::get_marketplace_dir()?.join("addon_settings.toml"))
}

pub fn read_addon_settings() -> anyhow::Result<AddonSettings> {
    let path = get_addon_settings_path()?;
    if !path.exists() {
        let default_settings = AddonSettings {
            schema_version: 1,
            active_theme_id: None,
            active_theme_source_pack_id: None,
            enabled_packs: vec![],
            disabled_contributions: vec![],
        };
        crate::config_store::save_toml_file(&path, &default_settings)?;
        return Ok(default_settings);
    }
    crate::config_store::read_toml_file::<AddonSettings>(&path)
}

pub fn save_addon_settings(settings: &AddonSettings) -> anyhow::Result<()> {
    let path = get_addon_settings_path()?;
    crate::config_store::save_toml_file(&path, settings)
}

pub fn toggle_pack_enabled(pack_id: &str, enabled: bool) -> anyhow::Result<()> {
    let mut settings = read_addon_settings()?;
    if enabled {
        if !settings.enabled_packs.contains(&pack_id.to_string()) {
            settings.enabled_packs.push(pack_id.to_string());
        }
    } else {
        settings.enabled_packs.retain(|id| id != pack_id);
    }
    save_addon_settings(&settings)
}

pub fn toggle_contribution_enabled(
    pack_id: &str,
    contribution_type: &str,
    contribution_id: &str,
    enabled: bool,
) -> anyhow::Result<()> {
    let mut settings = read_addon_settings()?;
    let key = format!("{}:{}:{}", pack_id, contribution_type, contribution_id);
    if enabled {
        settings.disabled_contributions.retain(|k| k != &key);
    } else {
        if !settings.disabled_contributions.contains(&key) {
            settings.disabled_contributions.push(key);
        }
    }
    save_addon_settings(&settings)
}

pub fn set_active_theme(
    theme_id: Option<String>,
    source_pack_id: Option<String>,
) -> anyhow::Result<()> {
    let mut settings = read_addon_settings()?;
    settings.active_theme_id = theme_id;
    settings.active_theme_source_pack_id = source_pack_id;
    save_addon_settings(&settings)
}

pub fn has_legacy_modes_file() -> bool {
    super::get_modes_path().map(|p| p.exists()).unwrap_or(false)
}

pub fn migrate_modes_to_addons() -> anyhow::Result<()> {
    let old_modes_path = super::get_modes_path()?;
    if !old_modes_path.exists() {
        return Ok(());
    }

    if let Ok(modes_file) = super::modes::read_modes() {
        let active_mode_id = &modes_file.active_mode;
        if let Some(active_mode) = modes_file.modes.iter().find(|m| m.id == *active_mode_id) {
            let mut settings = read_addon_settings()?;

            // 1. Migrate enabled packs
            settings.enabled_packs = active_mode.enabled_pack_ids.clone();

            // 2. Migrate active theme
            settings.active_theme_id = active_mode.theme_id.clone();
            if let Some(ref theme_id) = active_mode.theme_id {
                for pack_id in &active_mode.enabled_pack_ids {
                    if let Ok(caps) = super::packs::get_pack_activation_capabilities(pack_id) {
                        if caps.has_theme && caps.theme_id.as_ref() == Some(theme_id) {
                            settings.active_theme_source_pack_id = Some(pack_id.clone());
                            break;
                        }
                    }
                }
            }

            // 3. Migrate active style pack
            if let Some(ref active_style_pack_id) = active_mode.style_pack_id {
                for pack_id in &active_mode.enabled_pack_ids {
                    if pack_id != active_style_pack_id {
                        let key = format!("{}:style:style", pack_id);
                        if !settings.disabled_contributions.contains(&key) {
                            settings.disabled_contributions.push(key);
                        }
                    }
                }
            }

            save_addon_settings(&settings)?;
        }
    }

    let backup_path = old_modes_path.with_extension("toml.old");
    let _ = fs::rename(&old_modes_path, &backup_path);

    Ok(())
}
