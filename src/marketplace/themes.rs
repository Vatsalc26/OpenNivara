use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenNivaraTheme {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub colors: ThemeColors,
    pub effects: ThemeEffects,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub panel: String,
    pub card: String,
    pub primary: String,
    pub accent: String,
    pub success: String,
    pub warning: String,
    pub danger: String,
    pub foreground: String,
    pub muted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeEffects {
    pub background_gradient: bool,
    pub glow: String,
    pub density: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeSafety {
    pub data_only: bool,
    pub contains_executable_code: bool,
    pub modifies_tool_security: bool,
    pub requires_network: bool,
}

impl Default for ThemeSafety {
    fn default() -> Self {
        Self {
            data_only: true,
            contains_executable_code: false,
            modifies_tool_security: false,
            requires_network: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledTheme {
    pub id: String,
    pub name: String,
    pub version: String,
    pub source_kind: String,
    pub installed_at: String,
    pub manifest_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledThemesFile {
    pub schema_version: u32,
    pub installed: Vec<InstalledTheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub schema_version: u32,
    pub active_theme_id: Option<String>,
    pub active_theme_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeStoreItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub source_kind: String,
    pub installed: bool,
    pub applied: bool,
    pub preview_colors: ThemeColors,
    pub safety: ThemeSafety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub source_kind: String,
    pub safety: ThemeSafety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePreview {
    pub manifest: ThemeManifest,
    pub theme: OpenNivaraTheme,
    pub installed: bool,
    pub applied: bool,
}

pub fn ensure_theme_store_files() -> anyhow::Result<()> {
    fs::create_dir_all(super::get_themes_dir()?)?;

    let installed_path = super::get_installed_themes_path()?;
    if !installed_path.exists() {
        crate::config_store::save_toml_file(
            &installed_path,
            &InstalledThemesFile {
                schema_version: 1,
                installed: vec![],
            },
        )?;
    }

    let appearance_path = super::get_appearance_settings_path()?;
    if !appearance_path.exists() {
        crate::config_store::save_toml_file(
            &appearance_path,
            &AppearanceSettings {
                schema_version: 1,
                active_theme_id: None,
                active_theme_source: None,
            },
        )?;
    }

    Ok(())
}

pub fn read_installed_themes() -> anyhow::Result<InstalledThemesFile> {
    ensure_theme_store_files()?;
    crate::config_store::read_toml_file::<InstalledThemesFile>(&super::get_installed_themes_path()?)
}

fn save_installed_themes(file: &InstalledThemesFile) -> anyhow::Result<()> {
    ensure_theme_store_files()?;
    crate::config_store::save_toml_file(&super::get_installed_themes_path()?, file)
}

pub fn read_appearance_settings() -> anyhow::Result<AppearanceSettings> {
    ensure_theme_store_files()?;
    crate::config_store::read_toml_file::<AppearanceSettings>(
        &super::get_appearance_settings_path()?
    )
}

fn save_appearance_settings(settings: &AppearanceSettings) -> anyhow::Result<()> {
    ensure_theme_store_files()?;
    crate::config_store::save_toml_file(&super::get_appearance_settings_path()?, settings)
}

fn read_theme_at(path: &Path) -> anyhow::Result<OpenNivaraTheme> {
    let content = fs::read_to_string(path)?;
    let theme = toml::from_str::<OpenNivaraTheme>(&content)?;
    Ok(theme)
}

fn pack_metadata_for_theme(theme_dir: &Path) -> (String, String) {
    let pack_path = theme_dir.join("pack.toml");
    if let Ok(content) = fs::read_to_string(pack_path) {
        if let Ok(manifest) = toml::from_str::<super::packs::PackManifest>(&content) {
            return (manifest.author, manifest.version);
        }
    }
    ("OpenNivara".to_string(), "1.0.0".to_string())
}

fn builtin_theme_dirs() -> Vec<PathBuf> {
    let Ok(builtin_dir) = super::builtin::get_builtin_packs_dir() else {
        return vec![];
    };

    let Ok(entries) = fs::read_dir(builtin_dir) else {
        return vec![];
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("theme.toml").exists())
        .collect()
}

fn installed_theme_path(theme_id: &str) -> anyhow::Result<PathBuf> {
    Ok(super::get_themes_dir()?.join(theme_id).join("theme.toml"))
}

fn installed_theme_is_active(theme_id: &str) -> bool {
    read_appearance_settings()
        .ok()
        .and_then(|settings| settings.active_theme_id)
        .as_deref()
        == Some(theme_id)
}

pub fn list_theme_store_items() -> anyhow::Result<Vec<ThemeStoreItem>> {
    ensure_theme_store_files()?;
    let installed = read_installed_themes()?;
    let active = read_appearance_settings()?;
    let mut items = Vec::new();

    for dir in builtin_theme_dirs() {
        let theme_path = dir.join("theme.toml");
        let theme = read_theme_at(&theme_path)?;
        let (author, version) = pack_metadata_for_theme(&dir);
        let is_installed = installed.installed.iter().any(|item| item.id == theme.id);
        let is_applied = active.active_theme_id.as_deref() == Some(theme.id.as_str());
        items.push(ThemeStoreItem {
            id: theme.id.clone(),
            name: theme.name,
            description: theme.description,
            author,
            version,
            source_kind: "builtin".to_string(),
            installed: is_installed,
            applied: is_applied,
            preview_colors: theme.colors,
            safety: ThemeSafety::default(),
        });
    }

    for installed_theme in installed.installed {
        if items.iter().any(|item| item.id == installed_theme.id) {
            continue;
        }
        let path = PathBuf::from(&installed_theme.manifest_path);
        if let Ok(theme) = read_theme_at(&path) {
            items.push(ThemeStoreItem {
                id: theme.id.clone(),
                name: theme.name,
                description: theme.description,
                author: "Local".to_string(),
                version: installed_theme.version,
                source_kind: installed_theme.source_kind,
                installed: true,
                applied: active.active_theme_id.as_deref() == Some(theme.id.as_str()),
                preview_colors: theme.colors,
                safety: ThemeSafety::default(),
            });
        }
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

pub fn preview_theme(theme_id: &str) -> anyhow::Result<ThemePreview> {
    let item = list_theme_store_items()?
        .into_iter()
        .find(|item| item.id == theme_id)
        .ok_or_else(|| anyhow::anyhow!("Theme '{}' was not found.", theme_id))?;
    let theme = load_theme_by_id(theme_id)?
        .ok_or_else(|| anyhow::anyhow!("Theme '{}' could not be loaded.", theme_id))?;
    Ok(ThemePreview {
        manifest: ThemeManifest {
            id: item.id,
            name: item.name,
            description: item.description,
            author: item.author,
            version: item.version,
            source_kind: item.source_kind,
            safety: item.safety,
        },
        theme,
        installed: item.installed,
        applied: item.applied,
    })
}

fn copy_theme_into_store(
    source_theme_path: &Path,
    source_kind: &str,
) -> anyhow::Result<InstalledTheme> {
    let theme = read_theme_at(source_theme_path)?;
    let target_dir = super::get_themes_dir()?.join(&theme.id);
    fs::create_dir_all(&target_dir)?;
    let target_path = target_dir.join("theme.toml");
    fs::copy(source_theme_path, &target_path)?;

    let (author, version) = source_theme_path
        .parent()
        .map(pack_metadata_for_theme)
        .unwrap_or_else(|| ("Local".to_string(), "1.0.0".to_string()));
    let _ = author;

    Ok(InstalledTheme {
        id: theme.id,
        name: theme.name,
        version,
        source_kind: source_kind.to_string(),
        installed_at: Utc::now().to_rfc3339(),
        manifest_path: target_path.to_string_lossy().to_string(),
    })
}

pub fn install_builtin_theme(theme_id: &str) -> anyhow::Result<InstalledTheme> {
    let source = builtin_theme_dirs()
        .into_iter()
        .find(|dir| {
            read_theme_at(&dir.join("theme.toml"))
                .map(|theme| theme.id == theme_id)
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("Built-in theme '{}' was not found.", theme_id))?;

    install_theme_from_theme_path(&source.join("theme.toml"), "builtin")
}

pub fn install_theme_from_path(path: PathBuf) -> anyhow::Result<InstalledTheme> {
    let theme_path = if path.is_dir() {
        path.join("theme.toml")
    } else {
        path
    };
    install_theme_from_theme_path(&theme_path, "local")
}

fn install_theme_from_theme_path(
    theme_path: &Path,
    source_kind: &str,
) -> anyhow::Result<InstalledTheme> {
    let installed_theme = copy_theme_into_store(theme_path, source_kind)?;
    let mut file = read_installed_themes()?;
    if let Some(existing) = file
        .installed
        .iter_mut()
        .find(|theme| theme.id == installed_theme.id)
    {
        *existing = installed_theme.clone();
    } else {
        file.installed.push(installed_theme.clone());
    }
    save_installed_themes(&file)?;
    Ok(installed_theme)
}

pub fn uninstall_theme(theme_id: &str) -> anyhow::Result<()> {
    let mut file = read_installed_themes()?;
    file.installed.retain(|theme| theme.id != theme_id);
    save_installed_themes(&file)?;

    let target_dir = super::get_themes_dir()?.join(theme_id);
    if target_dir.exists() {
        fs::remove_dir_all(target_dir)?;
    }

    if installed_theme_is_active(theme_id) {
        reset_theme()?;
    }
    Ok(())
}

pub fn apply_theme(theme_id: &str) -> anyhow::Result<()> {
    if load_theme_by_id(theme_id)?.is_none() {
        return Err(anyhow::anyhow!("Theme '{}' is not installed.", theme_id));
    }
    save_appearance_settings(&AppearanceSettings {
        schema_version: 1,
        active_theme_id: Some(theme_id.to_string()),
        active_theme_source: Some("installed".to_string()),
    })
}

pub fn reset_theme() -> anyhow::Result<()> {
    save_appearance_settings(&AppearanceSettings {
        schema_version: 1,
        active_theme_id: None,
        active_theme_source: None,
    })
}

pub fn load_theme_by_id(theme_id: &str) -> anyhow::Result<Option<OpenNivaraTheme>> {
    let installed_path = installed_theme_path(theme_id)?;
    if installed_path.exists() {
        return Ok(Some(read_theme_at(&installed_path)?));
    }

    for dir in builtin_theme_dirs() {
        let theme_path = dir.join("theme.toml");
        let theme = read_theme_at(&theme_path)?;
        if theme.id == theme_id {
            return Ok(Some(theme));
        }
    }

    Ok(None)
}

pub fn get_active_theme_ui_only() -> anyhow::Result<Option<OpenNivaraTheme>> {
    let settings = read_appearance_settings()?;
    let Some(theme_id) = settings.active_theme_id else {
        return Ok(None);
    };
    load_theme_by_id(&theme_id)
}

/// Lists all visual themes found across all installed packs.
pub fn list_themes() -> anyhow::Result<Vec<OpenNivaraTheme>> {
    let packs_dir = super::get_packs_dir()?;
    let mut themes = Vec::new();

    if !packs_dir.exists() {
        return Ok(themes);
    }

    let installed_list = super::packs::list_installed_packs()?;

    for pack in installed_list.installed {
        if !pack.enabled {
            continue;
        }
        let theme_path = packs_dir.join(&pack.id).join("theme.toml");
        if theme_path.exists() {
            if let Ok(content) = fs::read_to_string(&theme_path) {
                if let Ok(theme) = toml::from_str::<OpenNivaraTheme>(&content) {
                    themes.push(theme);
                }
            }
        }
    }

    Ok(themes)
}

/// Resolves and retrieves the theme associated with the active mode.
pub fn get_active_theme() -> anyhow::Result<Option<OpenNivaraTheme>> {
    let mode = super::modes::get_active_mode()?;

    let theme_id = match mode.theme_id {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(None),
    };

    // Find in installed packs enabled by the active mode
    let packs_dir = super::get_packs_dir()?;
    let installed_list = super::packs::list_installed_packs()?;
    for pack_id in mode.enabled_pack_ids {
        let is_enabled = installed_list
            .installed
            .iter()
            .any(|p| p.id == pack_id && p.enabled);
        if !is_enabled {
            continue;
        }
        let theme_path = packs_dir.join(&pack_id).join("theme.toml");
        if theme_path.exists() {
            if let Ok(content) = fs::read_to_string(&theme_path) {
                if let Ok(theme) = toml::from_str::<OpenNivaraTheme>(&content) {
                    if theme.id == theme_id {
                        return Ok(Some(theme));
                    }
                }
            }
        }
    }

    // Fallback: search all themes in case of mismatch
    let all_themes = list_themes()?;
    for theme in all_themes {
        if theme.id == theme_id {
            return Ok(Some(theme));
        }
    }

    Ok(None)
}

/// Resolves and retrieves the theme associated with the active addon theme.
pub fn get_active_addon_theme() -> anyhow::Result<Option<OpenNivaraTheme>> {
    let settings = super::addon_settings::read_addon_settings()?;

    let active_theme_id = match settings.active_theme_id {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(None),
    };

    let source_pack_id = match settings.active_theme_source_pack_id {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(None),
    };

    // Verify source pack is installed and enabled
    let installed_list = super::packs::list_installed_packs()?;
    let is_enabled = installed_list
        .installed
        .iter()
        .any(|p| p.id == source_pack_id && p.enabled);
    if !is_enabled {
        return Ok(None);
    }

    let packs_dir = super::get_packs_dir()?;
    let theme_path = packs_dir.join(&source_pack_id).join("theme.toml");
    if theme_path.exists() {
        let content = fs::read_to_string(&theme_path)?;
        let theme = toml::from_str::<OpenNivaraTheme>(&content)?;
        if theme.id == active_theme_id {
            return Ok(Some(theme));
        }
    }

    Ok(None)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledThemeSummary {
    pub theme_id: String,
    pub theme_name: String,
    pub description: String,
    pub source_pack_id: String,
    pub source_pack_name: String,
    pub pack_enabled: bool,
}

/// Lists all visual themes installed inside enabled/disabled packs.
pub fn list_installed_themes() -> anyhow::Result<Vec<InstalledThemeSummary>> {
    let installed_list = super::packs::list_installed_packs()?;
    let packs_dir = super::get_packs_dir()?;
    let mut summaries = Vec::new();

    for pack in installed_list.installed {
        let theme_path = packs_dir.join(&pack.id).join("theme.toml");
        if theme_path.exists() {
            if let Ok(content) = fs::read_to_string(&theme_path) {
                if let Ok(theme) = toml::from_str::<OpenNivaraTheme>(&content) {
                    summaries.push(InstalledThemeSummary {
                        theme_id: theme.id,
                        theme_name: theme.name,
                        description: theme.description,
                        source_pack_id: pack.id.clone(),
                        source_pack_name: pack.name.clone(),
                        pack_enabled: pack.enabled,
                    });
                }
            }
        }
    }

    Ok(summaries)
}
