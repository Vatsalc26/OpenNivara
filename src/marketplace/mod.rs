pub mod addon_settings;
pub mod builtin;
pub mod merge;
pub mod modes;
pub mod packs;
pub mod repair;
pub mod snippets;
pub mod themes;

#[cfg(test)]
pub mod tests;

use std::fs;
use std::path::PathBuf;

/// Retrieves the base configuration directory used by OpenNivara.
pub fn get_config_dir() -> anyhow::Result<PathBuf> {
    crate::config_paths::config_dir()
}

/// Retrieves the path to the marketplace directory: config/marketplace/
pub fn get_marketplace_dir() -> anyhow::Result<PathBuf> {
    Ok(get_config_dir()?.join("marketplace"))
}

/// Retrieves the path to config/marketplace/installed_packs.toml
pub fn get_installed_packs_path() -> anyhow::Result<PathBuf> {
    Ok(get_marketplace_dir()?.join("installed_packs.toml"))
}

/// Retrieves the path to config/marketplace/installed_themes.toml
pub fn get_installed_themes_path() -> anyhow::Result<PathBuf> {
    Ok(get_marketplace_dir()?.join("installed_themes.toml"))
}

/// Retrieves the path to config/marketplace/appearance.toml
pub fn get_appearance_settings_path() -> anyhow::Result<PathBuf> {
    Ok(get_marketplace_dir()?.join("appearance.toml"))
}

/// Retrieves the path to the theme-only installation directory.
pub fn get_themes_dir() -> anyhow::Result<PathBuf> {
    Ok(get_marketplace_dir()?.join("themes"))
}

/// Retrieves the path to config/marketplace/modes.toml
pub fn get_modes_path() -> anyhow::Result<PathBuf> {
    Ok(get_marketplace_dir()?.join("modes.toml"))
}

/// Retrieves the path to config/marketplace/addon_settings.toml
pub fn get_addon_settings_path() -> anyhow::Result<PathBuf> {
    Ok(get_marketplace_dir()?.join("addon_settings.toml"))
}

/// Retrieves the path to the packs installation directory: config/marketplace/packs/
pub fn get_packs_dir() -> anyhow::Result<PathBuf> {
    Ok(get_marketplace_dir()?.join("packs"))
}

/// Initializes the marketplace directories and configuration files if they are missing.
pub fn init_marketplace() -> anyhow::Result<String> {
    let mkt_dir = get_marketplace_dir()?;
    let packs_dir = get_packs_dir()?;
    let themes_dir = get_themes_dir()?;

    fs::create_dir_all(&mkt_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create marketplace folder: {}", e))?;
    fs::create_dir_all(&packs_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create packs folder: {}", e))?;
    fs::create_dir_all(&themes_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create themes folder: {}", e))?;

    // Initialize Addon settings
    addon_settings::read_addon_settings()?;

    // Trigger installed packs file initialization
    let installed_packs_path = get_installed_packs_path()?;
    let mut installed_empty = false;
    if !installed_packs_path.exists() {
        let empty_installed = packs::InstalledPacksFile {
            schema_version: 1,
            installed: vec![],
        };
        crate::config_store::save_toml_file(&installed_packs_path, &empty_installed)?;
        installed_empty = true;
    } else if let Ok(installed_file) = packs::list_installed_packs() {
        if installed_file.installed.is_empty() {
            installed_empty = true;
        }
    }

    let mut auto_installed_count = 0;
    let mut installed_pack_ids = packs::list_installed_packs()
        .map(|file| {
            file.installed
                .into_iter()
                .map(|pack| pack.id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if installed_empty {
        if let Ok(_dir) = builtin::get_builtin_packs_dir() {
            if builtin::install_builtin_pack("coding_basics").is_ok() {
                auto_installed_count += 1;
            }
            if builtin::install_builtin_pack("study_coach").is_ok() {
                auto_installed_count += 1;
            }
            installed_pack_ids = packs::list_installed_packs()
                .map(|file| {
                    file.installed
                        .into_iter()
                        .map(|pack| pack.id)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
        }
    }

    let modes_status = modes::init_modes_for_installed_builtins(&installed_pack_ids)?;
    themes::ensure_theme_store_files()?;

    Ok(format!(
        "Successfully initialized OpenNivara Marketplace.\n- Active modes: {}\n- Auto-installed packs: {}",
        modes_status, auto_installed_count
    ))
}

/// Scope-safe reset of the marketplace directory.
pub fn marketplace_reset(confirm: bool) -> Result<(), String> {
    if !confirm {
        return Err("Reset was not confirmed.".to_string());
    }

    let mkt_dir = get_marketplace_dir().map_err(|e| e.to_string())?;
    if mkt_dir.exists() {
        fs::remove_dir_all(&mkt_dir)
            .map_err(|e| format!("Failed to delete marketplace directory: {}", e))?;
    }

    // Re-initialize marketplace
    init_marketplace().map_err(|e| e.to_string())?;

    Ok(())
}
