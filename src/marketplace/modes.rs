use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModesFile {
    pub schema_version: u32,
    pub active_mode: String,
    pub modes: Vec<OpenNivaraMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenNivaraMode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled_pack_ids: Vec<String>,
    pub theme_id: Option<String>,
    pub style_pack_id: Option<String>,
}

#[derive(Deserialize)]
struct RawOpenNivaraMode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled_pack_ids: Vec<String>,
    pub theme_id: Option<String>,
    pub style_pack_id: Option<String>,
    pub style_preset_id: Option<String>,
}

#[derive(Deserialize)]
struct ModesFileRaw {
    pub schema_version: u32,
    pub active_mode: String,
    pub modes: Vec<RawOpenNivaraMode>,
}

/// Reads the modes.toml file and automatically runs style_preset_id -> style_pack_id migration.
pub fn read_modes() -> anyhow::Result<ModesFile> {
    let path = super::get_modes_path()?;
    if !path.exists() {
        init_modes()?;
    }

    let raw = crate::config_store::read_toml_file::<ModesFileRaw>(&path)?;
    let mut migrated = false;
    let mut modes = Vec::new();

    for mode in raw.modes {
        let mut style_pack_id = mode.style_pack_id.clone();
        if style_pack_id.is_none() && mode.style_preset_id.is_some() {
            style_pack_id = mode.style_preset_id.clone();
            migrated = true;
        }
        if mode.style_preset_id.is_some() {
            migrated = true; // Mark as migrated so style_preset_id gets pruned on save
        }

        modes.push(OpenNivaraMode {
            id: mode.id,
            name: mode.name,
            description: mode.description,
            enabled_pack_ids: mode.enabled_pack_ids,
            theme_id: mode.theme_id,
            style_pack_id,
        });
    }

    let file = ModesFile {
        schema_version: raw.schema_version,
        active_mode: raw.active_mode,
        modes,
    };

    if migrated {
        let _ = save_modes(&file);
    }

    Ok(file)
}

/// Reads the modes.toml file without creating it or committing migration changes.
pub fn read_modes_readonly() -> anyhow::Result<ModesFile> {
    let path = super::get_modes_path()?;
    if !path.exists() {
        return Ok(ModesFile {
            schema_version: 1,
            active_mode: "default".to_string(),
            modes: vec![default_mode()],
        });
    }

    let raw = crate::config_store::read_toml_file::<ModesFileRaw>(&path)?;
    let mut modes = Vec::new();

    for mode in raw.modes {
        let mut style_pack_id = mode.style_pack_id.clone();
        if style_pack_id.is_none() && mode.style_preset_id.is_some() {
            style_pack_id = mode.style_preset_id.clone();
        }

        modes.push(OpenNivaraMode {
            id: mode.id,
            name: mode.name,
            description: mode.description,
            enabled_pack_ids: mode.enabled_pack_ids,
            theme_id: mode.theme_id,
            style_pack_id,
        });
    }

    Ok(ModesFile {
        schema_version: raw.schema_version,
        active_mode: raw.active_mode,
        modes,
    })
}

/// Saves changes back to the modes.toml file.
pub fn save_modes(modes: &ModesFile) -> anyhow::Result<()> {
    let path = super::get_modes_path()?;
    crate::config_store::save_toml_file(&path, modes)
}

fn default_mode() -> OpenNivaraMode {
    OpenNivaraMode {
        id: "default".to_string(),
        name: "Default".to_string(),
        description: "Default OpenNivara behavior with no extra packs.".to_string(),
        enabled_pack_ids: vec![],
        theme_id: None,
        style_pack_id: None,
    }
}

fn builtin_backed_modes() -> Vec<OpenNivaraMode> {
    vec![
        OpenNivaraMode {
            id: "coding".to_string(),
            name: "Coding Mode".to_string(),
            description: "Focus on technical development and rapid prototyping.".to_string(),
            enabled_pack_ids: vec!["coding_basics".to_string()],
            theme_id: Some("coding_cyan".to_string()),
            style_pack_id: Some("coding_basics".to_string()),
        },
        OpenNivaraMode {
            id: "study".to_string(),
            name: "Study Coach Mode".to_string(),
            description: "Focused learning, facts synthesis, and friendly explanations."
                .to_string(),
            enabled_pack_ids: vec!["study_coach".to_string()],
            theme_id: Some("calm_focus".to_string()),
            style_pack_id: Some("study_coach".to_string()),
        },
    ]
}

fn save_initial_modes(extra_modes: Vec<OpenNivaraMode>) -> anyhow::Result<String> {
    let path = super::get_modes_path()?;
    if path.exists() {
        return Ok("Modes file already initialized.".to_string());
    }

    let mut modes = vec![default_mode()];
    modes.extend(extra_modes);

    let default_modes = ModesFile {
        schema_version: 1,
        active_mode: "default".to_string(),
        modes,
    };

    save_modes(&default_modes)?;
    Ok("Initialized successfully.".to_string())
}

/// Initializes modes.toml with only the protected Default Mode.
pub fn init_modes() -> anyhow::Result<String> {
    save_initial_modes(vec![])
}

/// Initializes modes.toml with built-in modes only when their packs are installed.
pub fn init_modes_for_installed_builtins(installed_pack_ids: &[String]) -> anyhow::Result<String> {
    let builtin_modes = builtin_backed_modes()
        .into_iter()
        .filter(|mode| {
            mode.enabled_pack_ids
                .iter()
                .all(|id| installed_pack_ids.contains(id))
        })
        .collect();
    save_initial_modes(builtin_modes)
}

/// Retrieves the current active mode.
pub fn get_active_mode() -> anyhow::Result<OpenNivaraMode> {
    let file = read_modes()?;
    let active_id = file.active_mode;

    file.modes
        .into_iter()
        .find(|m| m.id == active_id)
        .ok_or_else(|| anyhow::anyhow!("Active mode '{}' not found in config.", active_id))
}

/// Activates a target mode.
pub fn set_active_mode(mode_id: &str) -> anyhow::Result<()> {
    let mut file = read_modes()?;

    if !file.modes.iter().any(|m| m.id == mode_id) {
        return Err(anyhow::anyhow!("Mode ID '{}' does not exist.", mode_id));
    }

    file.active_mode = mode_id.to_string();
    save_modes(&file)?;
    Ok(())
}

/// Registers a new custom mode.
pub fn create_mode(mode: OpenNivaraMode) -> anyhow::Result<()> {
    if mode.id.is_empty() {
        return Err(anyhow::anyhow!("Mode ID cannot be empty."));
    }
    for c in mode.id.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' && c != '-' {
            return Err(anyhow::anyhow!("Mode ID must only contain lowercase alphanumeric characters, underscores or hyphens."));
        }
    }

    let mut file = read_modes()?;

    // Enforce unique mode ID
    if file.modes.iter().any(|m| m.id == mode.id) {
        return Err(anyhow::anyhow!("Mode ID '{}' already exists.", mode.id));
    }

    file.modes.push(mode);
    save_modes(&file)?;
    Ok(())
}

/// Removes a custom mode (Default Mode cannot be deleted).
pub fn delete_mode(mode_id: &str) -> anyhow::Result<()> {
    if mode_id == "default" {
        return Err(anyhow::anyhow!(
            "The Default Mode is protected and cannot be deleted."
        ));
    }

    let mut file = read_modes()?;
    let idx = file
        .modes
        .iter()
        .position(|m| m.id == mode_id)
        .ok_or_else(|| anyhow::anyhow!("Mode ID '{}' not found.", mode_id))?;

    // If deleting the active mode, fall back to default
    if file.active_mode == mode_id {
        file.active_mode = "default".to_string();
    }

    file.modes.remove(idx);
    save_modes(&file)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeActivationResult {
    pub mode_id: String,
    pub pack_id: String,
    pub added_pack: bool,
    pub applied_theme_id: Option<String>,
    pub applied_style_pack_id: Option<String>,
    pub warnings: Vec<String>,
}

pub fn add_pack_to_mode_with_activation(
    mode_id: &str,
    pack_id: &str,
    apply_theme: bool,
    apply_style: bool,
) -> anyhow::Result<ModeActivationResult> {
    if mode_id == "default" {
        return Err(anyhow::anyhow!(
            "Default Mode is protected. Create or use another mode to add packs."
        ));
    }

    // 1. Ensure target pack is installed and enabled
    let installed_file = super::packs::list_installed_packs()?;
    let pack = installed_file
        .installed
        .iter()
        .find(|p| p.id == pack_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Pack '{}' is not installed. Install it before adding it to a mode.",
                pack_id
            )
        })?;

    if !pack.enabled {
        return Err(anyhow::anyhow!(
            "Pack '{}' is installed but currently disabled.",
            pack_id
        ));
    }

    let mut file = read_modes()?;
    let mode = file
        .modes
        .iter_mut()
        .find(|m| m.id == mode_id)
        .ok_or_else(|| anyhow::anyhow!("Mode ID '{}' not found.", mode_id))?;

    let mut added_pack = false;
    if !mode.enabled_pack_ids.contains(&pack_id.to_string()) {
        mode.enabled_pack_ids.push(pack_id.to_string());
        added_pack = true;
    }

    let mut warnings = Vec::new();
    let mut applied_theme_id = None;
    let mut applied_style_pack_id = None;

    let pack_dir = super::get_packs_dir()?.join(pack_id);

    if apply_theme {
        let theme_path = pack_dir.join("theme.toml");
        if theme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&theme_path) {
                if let Ok(theme) = toml::from_str::<super::themes::OpenNivaraTheme>(&content) {
                    mode.theme_id = Some(theme.id.clone());
                    applied_theme_id = Some(theme.id);
                } else {
                    warnings.push("Failed to parse theme.toml in pack.".to_string());
                }
            } else {
                warnings.push("Failed to read theme.toml in pack.".to_string());
            }
        } else {
            warnings.push("Theme requested but no theme.toml exists in this pack.".to_string());
        }
    }

    if apply_style {
        let style_path = pack_dir.join("style.toml");
        if style_path.exists() {
            mode.style_pack_id = Some(pack_id.to_string());
            applied_style_pack_id = Some(pack_id.to_string());
        } else {
            warnings.push("Style requested but no style.toml exists in this pack.".to_string());
        }
    }

    save_modes(&file)?;

    Ok(ModeActivationResult {
        mode_id: mode_id.to_string(),
        pack_id: pack_id.to_string(),
        added_pack,
        applied_theme_id,
        applied_style_pack_id,
        warnings,
    })
}

/// Adds an installed pack to a target mode (backwards compatibility).
pub fn add_pack_to_mode(mode_id: &str, pack_id: &str) -> anyhow::Result<()> {
    add_pack_to_mode_with_activation(mode_id, pack_id, false, false)?;
    Ok(())
}

pub fn create_mode_from_pack(
    pack_id: &str,
    mode_id: &str,
    mode_name: &str,
    activate: bool,
    apply_theme: bool,
    apply_style: bool,
) -> anyhow::Result<OpenNivaraMode> {
    if mode_id.is_empty() {
        return Err(anyhow::anyhow!("Mode ID cannot be empty."));
    }
    if mode_id == "default" {
        return Err(anyhow::anyhow!(
            "Default Mode is protected and cannot be overwritten."
        ));
    }
    for c in mode_id.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' && c != '-' {
            return Err(anyhow::anyhow!("Mode ID must only contain lowercase alphanumeric characters, underscores or hyphens."));
        }
    }

    // Ensure pack is installed and enabled
    let installed_file = super::packs::list_installed_packs()?;
    let pack = installed_file
        .installed
        .iter()
        .find(|p| p.id == pack_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Pack '{}' is not installed. Install it before creating a mode from it.",
                pack_id
            )
        })?;

    if !pack.enabled {
        return Err(anyhow::anyhow!(
            "Pack '{}' is installed but currently disabled.",
            pack_id
        ));
    }

    let mut file = read_modes()?;
    if file.modes.iter().any(|m| m.id == mode_id) {
        return Err(anyhow::anyhow!("Mode ID '{}' already exists.", mode_id));
    }

    let mut theme_id = None;
    let mut style_pack_id = None;
    let pack_dir = super::get_packs_dir()?.join(pack_id);

    if apply_theme {
        let theme_path = pack_dir.join("theme.toml");
        if theme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&theme_path) {
                if let Ok(theme) = toml::from_str::<super::themes::OpenNivaraTheme>(&content) {
                    theme_id = Some(theme.id);
                }
            }
        }
    }

    if apply_style {
        let style_path = pack_dir.join("style.toml");
        if style_path.exists() {
            style_pack_id = Some(pack_id.to_string());
        }
    }

    let new_mode = OpenNivaraMode {
        id: mode_id.to_string(),
        name: mode_name.to_string(),
        description: format!("Mode created from pack {}.", pack.name),
        enabled_pack_ids: vec![pack_id.to_string()],
        theme_id,
        style_pack_id,
    };

    file.modes.push(new_mode.clone());

    if activate {
        file.active_mode = mode_id.to_string();
    }

    save_modes(&file)?;
    Ok(new_mode)
}

pub fn update_mode_theme(mode_id: &str, theme_id: Option<String>) -> anyhow::Result<()> {
    if mode_id == "default" {
        return Err(anyhow::anyhow!(
            "Default Mode theme/style is protected and cannot be updated."
        ));
    }

    let mut file = read_modes()?;
    let mode = file
        .modes
        .iter_mut()
        .find(|m| m.id == mode_id)
        .ok_or_else(|| anyhow::anyhow!("Mode ID '{}' not found.", mode_id))?;

    if let Some(ref t_id) = theme_id {
        if t_id.is_empty() {
            mode.theme_id = None;
        } else {
            // theme_id must exist in one enabled pack in that mode
            let installed_file = super::packs::list_installed_packs()?;
            let mut found = false;
            let packs_dir = super::get_packs_dir()?;
            for enabled_pack_id in &mode.enabled_pack_ids {
                let pack = installed_file
                    .installed
                    .iter()
                    .find(|p| p.id == *enabled_pack_id);
                if let Some(pack) = pack {
                    if pack.enabled {
                        let theme_path = packs_dir.join(enabled_pack_id).join("theme.toml");
                        if theme_path.exists() {
                            if let Ok(content) = std::fs::read_to_string(&theme_path) {
                                if let Ok(theme) =
                                    toml::from_str::<super::themes::OpenNivaraTheme>(&content)
                                {
                                    if theme.id == *t_id {
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !found {
                return Err(anyhow::anyhow!(
                    "Theme ID '{}' does not exist in any enabled packs associated with this mode.",
                    t_id
                ));
            }
            mode.theme_id = Some(t_id.clone());
        }
    } else {
        mode.theme_id = None;
    }

    save_modes(&file)?;
    Ok(())
}

pub fn update_mode_style_pack(mode_id: &str, style_pack_id: Option<String>) -> anyhow::Result<()> {
    if mode_id == "default" {
        return Err(anyhow::anyhow!(
            "Default Mode theme/style is protected and cannot be updated."
        ));
    }

    let mut file = read_modes()?;
    let mode = file
        .modes
        .iter_mut()
        .find(|m| m.id == mode_id)
        .ok_or_else(|| anyhow::anyhow!("Mode ID '{}' not found.", mode_id))?;

    if let Some(ref s_id) = style_pack_id {
        if s_id.is_empty() {
            mode.style_pack_id = None;
        } else {
            // style_pack_id must exist in enabled_pack_ids of this mode
            if !mode.enabled_pack_ids.contains(s_id) {
                return Err(anyhow::anyhow!(
                    "Style source pack '{}' is not enabled in this mode.",
                    s_id
                ));
            }
            // style_pack_id must be installed and enabled
            let installed_file = super::packs::list_installed_packs()?;
            let pack = installed_file
                .installed
                .iter()
                .find(|p| p.id == *s_id)
                .ok_or_else(|| anyhow::anyhow!("Pack '{}' is not installed.", s_id))?;

            if !pack.enabled {
                return Err(anyhow::anyhow!(
                    "Pack '{}' is disabled and cannot be used as a style source.",
                    s_id
                ));
            }

            // must have style.toml
            let pack_dir = super::get_packs_dir()?.join(s_id);
            if !pack_dir.join("style.toml").exists() {
                return Err(anyhow::anyhow!(
                    "Pack '{}' does not contain a style.toml.",
                    s_id
                ));
            }

            mode.style_pack_id = Some(s_id.clone());
        }
    } else {
        mode.style_pack_id = None;
    }

    save_modes(&file)?;
    Ok(())
}

/// Removes a pack ID from a target mode.
pub fn remove_pack_from_mode(mode_id: &str, pack_id: &str) -> anyhow::Result<()> {
    let mut file = read_modes()?;
    let mode = file
        .modes
        .iter_mut()
        .find(|m| m.id == mode_id)
        .ok_or_else(|| anyhow::anyhow!("Mode ID '{}' not found.", mode_id))?;

    mode.enabled_pack_ids.retain(|id| id != pack_id);
    save_modes(&file)?;
    Ok(())
}
