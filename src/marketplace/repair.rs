use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceRepairReport {
    pub repaired: bool,
    pub actions: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceStatus {
    pub marketplace_dir: String,
    pub installed_count: usize,
    pub enabled_count: usize,
    pub disabled_count: usize,
    pub modes_count: usize,
    pub active_mode_id: String,
    pub active_mode_name: String,
    pub active_theme_id: Option<String>,
    pub active_theme_name: Option<String>,
    pub missing_pack_ids: Vec<String>,
    pub disabled_packs_in_active_mode: Vec<String>,
    pub builtin_packs_available: Vec<String>,
    pub builtin_resource_path_checked: String,
    pub builtin_resource_path_exists: bool,
}

/// Audits and automatically repairs the marketplace folders and configurations.
pub fn marketplace_repair(dry_run: bool) -> anyhow::Result<MarketplaceRepairReport> {
    let mut actions = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut repaired = false;

    let mkt_dir = super::get_marketplace_dir()?;
    let packs_dir = super::get_packs_dir()?;

    // 1. Check directories existence
    if !mkt_dir.exists() {
        if !dry_run {
            fs::create_dir_all(&mkt_dir)?;
        }
        actions.push(format!(
            "Created marketplace directory at {}",
            mkt_dir.display()
        ));
        repaired = true;
    }
    if !packs_dir.exists() {
        if !dry_run {
            fs::create_dir_all(&packs_dir)?;
        }
        actions.push(format!(
            "Created packs installation directory at {}",
            packs_dir.display()
        ));
        repaired = true;
    }

    // 2. Check installed_packs.toml
    let installed_path = super::get_installed_packs_path()?;
    if !installed_path.exists() {
        if !dry_run {
            let empty_installed = super::packs::InstalledPacksFile {
                schema_version: 1,
                installed: vec![],
            };
            crate::config_store::save_toml_file(&installed_path, &empty_installed)?;
        }
        actions.push("Initialized empty installed_packs.toml file.".to_string());
        repaired = true;
    }

    // 3. Check modes.toml
    let modes_path = super::get_modes_path()?;
    if !modes_path.exists() {
        if !dry_run {
            let installed_pack_ids = if installed_path.exists() {
                super::packs::list_installed_packs()
                    .map(|file| {
                        file.installed
                            .into_iter()
                            .map(|pack| pack.id)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            } else {
                vec![]
            };
            super::modes::init_modes_for_installed_builtins(&installed_pack_ids)?;
        }
        actions.push(
            "Initialized default and built-in modes.toml file from installed packs.".to_string(),
        );
        repaired = true;
    }

    // 4. Load configs for deeper audit (gracefully handle missing files in dry-run)
    let mut installed_data = if installed_path.exists() {
        super::packs::list_installed_packs().unwrap_or(super::packs::InstalledPacksFile {
            schema_version: 1,
            installed: vec![],
        })
    } else {
        super::packs::InstalledPacksFile {
            schema_version: 1,
            installed: vec![],
        }
    };

    let mut modes_data = if modes_path.exists() {
        super::modes::read_modes().unwrap_or(super::modes::ModesFile {
            schema_version: 1,
            active_mode: "default".to_string(),
            modes: vec![],
        })
    } else {
        super::modes::ModesFile {
            schema_version: 1,
            active_mode: "default".to_string(),
            modes: vec![],
        }
    };

    // 5. Ensure active mode exists
    let active_mode_exists = modes_data
        .modes
        .iter()
        .any(|m| m.id == modes_data.active_mode);
    if !active_mode_exists && !modes_data.modes.is_empty() {
        let old_active = modes_data.active_mode.clone();
        modes_data.active_mode = "default".to_string();
        actions.push(format!(
            "Active mode '{}' was missing; reset to 'default' mode.",
            old_active
        ));
        repaired = true;
    }

    // 6. Audit modes for missing referenced packs
    for mode in &mut modes_data.modes {
        let mut install_actions = Vec::new();
        for pack_id in &mode.enabled_pack_ids {
            let is_installed = installed_data.installed.iter().any(|p| p.id == *pack_id);
            if !is_installed {
                // If it's a known built-in pack, try to auto-install it
                let is_builtin = pack_id == "coding_basics" || pack_id == "study_coach";
                if is_builtin {
                    if let Ok(_builtin_dir) = super::builtin::get_builtin_packs_dir() {
                        if !dry_run {
                            match super::builtin::install_builtin_pack(pack_id) {
                                Ok(pack) => {
                                    install_actions.push(pack.id.clone());
                                    actions.push(format!("Automatically re-installed missing built-in pack '{}' for Mode '{}'.", pack.id, mode.name));
                                    repaired = true;
                                }
                                Err(e) => {
                                    errors.push(format!(
                                        "Failed to auto-install missing built-in pack '{}': {}",
                                        pack_id, e
                                    ));
                                }
                            }
                        } else {
                            actions.push(format!(
                                "Planned auto-install of missing built-in pack '{}' for Mode '{}'.",
                                pack_id, mode.name
                            ));
                            repaired = true;
                        }
                    } else {
                        warnings.push(format!("Mode '{}' references missing built-in pack '{}', but built-in source folder could not be located.", mode.name, pack_id));
                    }
                } else {
                    warnings.push(format!("Mode '{}' references missing pack '{}'. Use Import Local Pack to resolve this.", mode.name, pack_id));
                }
            }
        }

        // If we installed some builtins, reload installed packs index
        if !install_actions.is_empty() && !dry_run {
            if let Ok(fresh_installed) = super::packs::list_installed_packs() {
                installed_data = fresh_installed;
            }
        }
    }

    // 7. Verify active theme availability if applicable
    let active_mode = modes_data
        .modes
        .iter()
        .find(|m| m.id == modes_data.active_mode);
    if let Some(mode) = active_mode {
        if let Some(ref theme_id) = mode.theme_id {
            if !theme_id.is_empty() {
                match super::themes::get_active_theme() {
                    Ok(Some(theme)) => {
                        if theme.id != *theme_id {
                            warnings.push(format!(
                                "Theme ID mismatch: Mode expects '{}', loaded '{}'.",
                                theme_id, theme.id
                            ));
                        }
                    }
                    Ok(None) => {
                        warnings.push(format!("Theme '{}' referenced by active mode '{}' was not found in any enabled packs.", theme_id, mode.name));
                    }
                    Err(e) => {
                        warnings.push(format!("Error loading theme '{}': {}", theme_id, e));
                    }
                }
            }
        }

        // Verify style preset pack exists
        if let Some(ref style_pack_id) = mode.style_pack_id {
            if !style_pack_id.is_empty() {
                let pack_exists = installed_data
                    .installed
                    .iter()
                    .any(|p| p.id == *style_pack_id && p.enabled);
                if !pack_exists {
                    warnings.push(format!("Style source pack '{}' referenced by active mode '{}' is missing or disabled. Base style will be used.", style_pack_id, mode.name));
                }
            }
        }
    }

    // Save repaired modes if changed
    if repaired && !dry_run {
        super::modes::save_modes(&modes_data)?;
    }

    Ok(MarketplaceRepairReport {
        repaired,
        actions,
        warnings,
        errors,
    })
}

/// Gathers complete diagnostic state information about the local marketplace.
pub fn marketplace_status() -> anyhow::Result<MarketplaceStatus> {
    let mkt_dir = super::get_marketplace_dir()?;
    let installed_file = super::packs::list_installed_packs()?;
    let modes_file = super::modes::read_modes()?;

    let mut enabled_count = 0;
    let mut disabled_count = 0;
    for pack in &installed_file.installed {
        if pack.enabled {
            enabled_count += 1;
        } else {
            disabled_count += 1;
        }
    }

    let active_mode = modes_file
        .modes
        .iter()
        .find(|m| m.id == modes_file.active_mode);
    let (active_mode_name, active_theme_id) = match active_mode {
        Some(m) => (m.name.clone(), m.theme_id.clone()),
        None => ("Unknown".to_string(), None),
    };

    let mut active_theme_name = None;
    if let Ok(Some(theme)) = super::themes::get_active_theme() {
        active_theme_name = Some(theme.name);
    }

    // Check for referenced missing packs across all modes
    let mut missing_pack_ids = Vec::new();
    for mode in &modes_file.modes {
        for pack_id in &mode.enabled_pack_ids {
            let is_installed = installed_file.installed.iter().any(|p| p.id == *pack_id);
            if !is_installed && !missing_pack_ids.contains(pack_id) {
                missing_pack_ids.push(pack_id.clone());
            }
        }
    }

    // Check for disabled packs referenced in the active mode
    let mut disabled_packs_in_active_mode = Vec::new();
    if let Some(m) = active_mode {
        for pack_id in &m.enabled_pack_ids {
            let p = installed_file
                .installed
                .iter()
                .find(|pack| pack.id == *pack_id);
            if let Some(pack) = p {
                if !pack.enabled {
                    disabled_packs_in_active_mode.push(pack_id.clone());
                }
            }
        }
    }

    // Check builtin pack availability in release/repo paths
    let mut builtin_packs_available = Vec::new();
    let (builtin_resource_path_checked, builtin_resource_path_exists) =
        match super::builtin::get_builtin_packs_dir() {
            Ok(path) => (path.to_string_lossy().into_owned(), path.exists()),
            Err(_) => ("unavailable".to_string(), false),
        };
    if let Ok(summaries) = super::builtin::list_builtin_packs() {
        for s in summaries {
            builtin_packs_available.push(s.id);
        }
    }

    Ok(MarketplaceStatus {
        marketplace_dir: mkt_dir.to_string_lossy().into_owned(),
        installed_count: installed_file.installed.len(),
        enabled_count,
        disabled_count,
        modes_count: modes_file.modes.len(),
        active_mode_id: modes_file.active_mode,
        active_mode_name,
        active_theme_id,
        active_theme_name,
        missing_pack_ids,
        disabled_packs_in_active_mode,
        builtin_packs_available,
        builtin_resource_path_checked,
        builtin_resource_path_exists,
    })
}

pub fn marketplace_status_readonly() -> anyhow::Result<MarketplaceStatus> {
    let mkt_dir = super::get_marketplace_dir()?;
    let installed_path = super::get_installed_packs_path()?;

    let installed_file = if installed_path.exists() {
        crate::config_store::read_toml_file::<super::packs::InstalledPacksFile>(&installed_path)
            .unwrap_or(super::packs::InstalledPacksFile {
                schema_version: 1,
                installed: vec![],
            })
    } else {
        super::packs::InstalledPacksFile {
            schema_version: 1,
            installed: vec![],
        }
    };

    let modes_file = super::modes::read_modes_readonly()?;

    let mut enabled_count = 0;
    let mut disabled_count = 0;
    for pack in &installed_file.installed {
        if pack.enabled {
            enabled_count += 1;
        } else {
            disabled_count += 1;
        }
    }

    let active_mode = modes_file
        .modes
        .iter()
        .find(|m| m.id == modes_file.active_mode);
    let (active_mode_name, active_theme_id) = match active_mode {
        Some(m) => (m.name.clone(), m.theme_id.clone()),
        None => ("Unknown".to_string(), None),
    };

    let mut active_theme_name = None;
    if let Some(ref theme_id) = active_theme_id {
        if let Some(m) = active_mode {
            let packs_dir = super::get_packs_dir()?;
            for pack_id in &m.enabled_pack_ids {
                let is_enabled = installed_file
                    .installed
                    .iter()
                    .any(|p| p.id == *pack_id && p.enabled);
                if !is_enabled {
                    continue;
                }
                let theme_path = packs_dir.join(pack_id).join("theme.toml");
                if theme_path.exists() {
                    if let Ok(content) = fs::read_to_string(&theme_path) {
                        if let Ok(theme) =
                            toml::from_str::<super::themes::OpenNivaraTheme>(&content)
                        {
                            if theme.id == *theme_id {
                                active_theme_name = Some(theme.name);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // Check for referenced missing packs across all modes
    let mut missing_pack_ids = Vec::new();
    for mode in &modes_file.modes {
        for pack_id in &mode.enabled_pack_ids {
            let is_installed = installed_file.installed.iter().any(|p| p.id == *pack_id);
            if !is_installed && !missing_pack_ids.contains(pack_id) {
                missing_pack_ids.push(pack_id.clone());
            }
        }
    }

    // Check for disabled packs referenced in the active mode
    let mut disabled_packs_in_active_mode = Vec::new();
    if let Some(m) = active_mode {
        for pack_id in &m.enabled_pack_ids {
            let p = installed_file
                .installed
                .iter()
                .find(|pack| pack.id == *pack_id);
            if let Some(pack) = p {
                if !pack.enabled {
                    disabled_packs_in_active_mode.push(pack_id.clone());
                }
            }
        }
    }

    // Check builtin pack availability
    let mut builtin_packs_available = Vec::new();
    let (builtin_resource_path_checked, builtin_resource_path_exists) =
        match super::builtin::get_builtin_packs_dir() {
            Ok(path) => (path.to_string_lossy().into_owned(), path.exists()),
            Err(_) => ("unavailable".to_string(), false),
        };
    if let Ok(summaries) = super::builtin::list_builtin_packs() {
        for s in summaries {
            builtin_packs_available.push(s.id);
        }
    }

    Ok(MarketplaceStatus {
        marketplace_dir: mkt_dir.to_string_lossy().into_owned(),
        installed_count: installed_file.installed.len(),
        enabled_count,
        disabled_count,
        modes_count: modes_file.modes.len(),
        active_mode_id: modes_file.active_mode,
        active_mode_name,
        active_theme_id,
        active_theme_name,
        missing_pack_ids,
        disabled_packs_in_active_mode,
        builtin_packs_available,
        builtin_resource_path_checked,
        builtin_resource_path_exists,
    })
}
