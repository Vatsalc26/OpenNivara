use chrono::Utc;
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PackManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub category: String,
    pub description: String,
    #[serde(default)]
    pub homepage: String,
    #[serde(default)]
    pub license: String,
    pub compatibility: PackCompatibility,
    pub contents: PackContents,
    pub safety: PackSafety,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PackCompatibility {
    pub opennivara_min_version: String,
    #[serde(default)]
    pub opennivara_max_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PackContents {
    pub preferences: bool,
    pub contexts: bool,
    pub style_presets: bool,
    pub profile_templates: bool,
    pub tool_presets: bool,
    pub workspace_map_rules: bool,
    pub prompt_behaviors: bool,
    pub command_snippets: bool,
    pub theme: bool,
    #[serde(default)]
    pub skills: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PackSafety {
    pub contains_executable_code: bool,
    pub modifies_tool_permissions: bool,
    pub requires_network: bool,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPacksFile {
    pub schema_version: u32,
    pub installed: Vec<InstalledPack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPack {
    pub id: String,
    pub name: String,
    pub version: String,
    pub installed_at: String,
    pub source: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackPreview {
    pub manifest: PackManifest,
    pub source_path: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub additions: PackAdditionsSummary,
    pub safety_summary: PackSafetySummary,
    #[serde(default)]
    pub skill_previews: Vec<crate::skills::manifest::SkillManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackAdditionsSummary {
    pub preferences_count: usize,
    pub contexts_count: usize,
    pub style_presets_count: usize,
    pub themes_count: usize,
    pub command_snippets_count: usize,
    pub workspace_rules_count: usize,
    pub profile_templates_count: usize,
    pub tool_presets_count: usize,
    pub skills_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackSafetySummary {
    pub allowed_to_install: bool,
    pub risk_level: String,
    pub modifies_tool_permissions: bool,
    pub contains_executable_code: bool,
    pub requires_network: bool,
}

/// Enforces and validates a parsed PackManifest.
pub fn validate_pack_manifest(manifest: &PackManifest) -> Result<(), String> {
    if manifest.schema_version != 1 {
        return Err("Pack manifest schema_version must be exactly 1.".to_string());
    }

    // ID verification: lowercase snake_case
    if manifest.id.is_empty() {
        return Err("Pack ID cannot be empty.".to_string());
    }
    for c in manifest.id.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' && c != '-' {
            return Err("Pack ID must only contain lowercase alphanumeric characters, underscores or hyphens.".to_string());
        }
    }

    // Version verification
    if Version::parse(&manifest.version).is_err() {
        return Err(format!(
            "Pack version '{}' is not a valid semver string.",
            manifest.version
        ));
    }

    // Compatibility check
    if Version::parse(&manifest.compatibility.opennivara_min_version).is_err() {
        return Err(format!(
            "compatibility.opennivara_min_version '{}' is not a valid semver string.",
            manifest.compatibility.opennivara_min_version
        ));
    }
    if !manifest.compatibility.opennivara_max_version.is_empty()
        && Version::parse(&manifest.compatibility.opennivara_max_version).is_err()
    {
        return Err(format!(
            "compatibility.opennivara_max_version '{}' is not a valid semver string.",
            manifest.compatibility.opennivara_max_version
        ));
    }

    // Security constraints
    if manifest.safety.contains_executable_code {
        return Err(
            "Packs containing native or executable code are disabled in Marketplace v1."
                .to_string(),
        );
    }
    if manifest.safety.modifies_tool_permissions {
        return Err(
            "Packs attempting to modify tool permissions are strictly blocked in Marketplace v1."
                .to_string(),
        );
    }

    let level = manifest.safety.risk_level.to_lowercase();
    if level != "low" && level != "medium" && level != "high" {
        return Err("risk_level must be one of: low, medium, high.".to_string());
    }

    Ok(())
}

/// Generates a complete preview of a pack from its local directory path.
pub fn preview_pack_from_path(path: PathBuf) -> anyhow::Result<PackPreview> {
    let manifest_path = path.join("pack.toml");
    if !manifest_path.exists() {
        return Err(anyhow::anyhow!(
            "The selected folder does not contain a pack.toml manifest."
        ));
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| anyhow::anyhow!("Failed to read pack.toml: {}", e))?;

    let manifest: PackManifest = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse pack.toml: {}", e))?;

    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // 1. Validation
    if let Err(e) = validate_pack_manifest(&manifest) {
        errors.push(e);
    }

    // 2. Addition counts & content presence check
    let mut preferences_count = 0;
    let mut contexts_count = 0;
    let mut style_presets_count = 0;
    let mut themes_count = 0;
    let mut command_snippets_count = 0;
    let mut workspace_rules_count = 0;
    let mut profile_templates_count = 0;
    let mut tool_presets_count = 0;
    let mut skills_count = 0;
    let mut skill_previews = Vec::new();

    // Check preferences
    if manifest.contents.preferences {
        let file = path.join("preferences.toml");
        if file.exists() {
            if let Ok(toml_content) = fs::read_to_string(&file) {
                if let Ok(val) = toml::from_str::<toml::Value>(&toml_content) {
                    if let Some(arr) = val.get("sections").and_then(|s| s.as_array()) {
                        preferences_count = arr.len();
                    }
                }
            }
        } else {
            warnings.push(
                "pack.toml declares 'preferences = true' but preferences.toml is missing."
                    .to_string(),
            );
        }
    }

    // Check contexts
    if manifest.contents.contexts {
        let file = path.join("contexts.toml");
        if file.exists() {
            if let Ok(toml_content) = fs::read_to_string(&file) {
                if let Ok(val) = toml::from_str::<toml::Value>(&toml_content) {
                    if let Some(arr) = val.get("contexts").and_then(|c| c.as_array()) {
                        contexts_count = arr.len();
                    }
                }
            }
        } else {
            warnings.push(
                "pack.toml declares 'contexts = true' but contexts.toml is missing.".to_string(),
            );
        }
    }

    // Check style presets
    if manifest.contents.style_presets {
        let file = path.join("style.toml");
        if file.exists() {
            style_presets_count = 1;
        } else {
            warnings.push(
                "pack.toml declares 'style_presets = true' but style.toml is missing.".to_string(),
            );
        }
    }

    // Check themes
    if manifest.contents.theme {
        let file = path.join("theme.toml");
        if file.exists() {
            themes_count = 1;
        } else {
            warnings
                .push("pack.toml declares 'theme = true' but theme.toml is missing.".to_string());
        }
    }

    // Check command snippets
    if manifest.contents.command_snippets {
        let file = path.join("commands.toml");
        if file.exists() {
            if let Ok(toml_content) = fs::read_to_string(&file) {
                if let Ok(val) = toml::from_str::<toml::Value>(&toml_content) {
                    if let Some(arr) = val.get("commands").and_then(|c| c.as_array()) {
                        command_snippets_count = arr.len();
                    }
                }
            }
        } else {
            warnings.push(
                "pack.toml declares 'command_snippets = true' but commands.toml is missing."
                    .to_string(),
            );
        }
    }

    // Check workspace rules
    if manifest.contents.workspace_map_rules {
        let file = path.join("workspace_rules.toml");
        if file.exists() {
            if let Ok(toml_content) = fs::read_to_string(&file) {
                if let Ok(val) = toml::from_str::<toml::Value>(&toml_content) {
                    if let Some(arr) = val.get("rules").and_then(|r| r.as_array()) {
                        workspace_rules_count = arr.len();
                    }
                }
            }
        } else {
            warnings.push("pack.toml declares 'workspace_map_rules = true' but workspace_rules.toml is missing.".to_string());
        }
    }

    // Check profile templates
    if manifest.contents.profile_templates {
        warnings.push(
            "Profile templates declared. V1 lets you preview, but will not automatically apply."
                .to_string(),
        );
        profile_templates_count = 1;
    }

    // Check tool presets
    if manifest.contents.tool_presets {
        warnings.push("This pack includes tool presets. Tool preset installation is disabled in Marketplace v1.".to_string());
        tool_presets_count = 1;
    }

    if manifest.contents.skills {
        let skills_dir = path.join("skills");
        if skills_dir.exists() {
            let tool_registry = crate::tools::ToolRegistry::new(true);
            let mut skill_paths: Vec<_> = fs::read_dir(&skills_dir)
                .map(|entries| {
                    entries
                        .flatten()
                        .map(|entry| entry.path())
                        .filter(|entry_path| {
                            entry_path.extension().and_then(|ext| ext.to_str()) == Some("toml")
                        })
                        .collect()
                })
                .unwrap_or_default();
            skill_paths.sort();
            skills_count = skill_paths.len();
            for skill_path in skill_paths {
                match crate::config_store::read_toml_file::<crate::skills::manifest::SkillManifest>(
                    &skill_path,
                ) {
                    Ok(mut skill) => {
                        skill.pack_id = Some(skill.pack_id.unwrap_or_else(|| manifest.id.clone()));
                        match crate::skills::manifest::validate_skill_manifest(
                            &skill,
                            &tool_registry,
                        ) {
                            Ok(skill_warnings) => {
                                warnings.extend(skill_warnings.into_iter().map(|warning| {
                                    format!(
                                        "{}: {}",
                                        skill_path
                                            .file_name()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or("skill.toml"),
                                        warning
                                    )
                                }));
                                skill_previews.push(skill);
                            }
                            Err(err) => errors.push(format!(
                                "{}: {}",
                                skill_path
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or("skill.toml"),
                                err
                            )),
                        }
                    }
                    Err(err) => errors.push(format!(
                        "{}: failed to parse skill manifest: {}",
                        skill_path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("skill.toml"),
                        err
                    )),
                }
            }
        } else {
            warnings.push("pack.toml declares 'skills = true' but skills/ is missing.".to_string());
        }
    }

    let allowed = errors.is_empty();

    Ok(PackPreview {
        manifest: manifest.clone(),
        source_path: path.to_string_lossy().to_string(),
        warnings,
        errors,
        additions: PackAdditionsSummary {
            preferences_count,
            contexts_count,
            style_presets_count,
            themes_count,
            command_snippets_count,
            workspace_rules_count,
            profile_templates_count,
            tool_presets_count,
            skills_count,
        },
        safety_summary: PackSafetySummary {
            allowed_to_install: allowed,
            risk_level: manifest.safety.risk_level.clone(),
            modifies_tool_permissions: manifest.safety.modifies_tool_permissions,
            contains_executable_code: manifest.safety.contains_executable_code,
            requires_network: manifest.safety.requires_network,
        },
        skill_previews,
    })
}

/// Previews an installed pack from its copied marketplace directory.
pub fn preview_installed_pack(pack_id: &str) -> anyhow::Result<PackPreview> {
    let target_dir = super::get_packs_dir()?.join(pack_id);
    if !target_dir.exists() {
        return Err(anyhow::anyhow!("Pack '{}' is not installed.", pack_id));
    }
    preview_pack_from_path(target_dir)
}

fn collect_skill_ids_from_dir(
    skills_dir: &Path,
    source: String,
    out: &mut HashMap<String, String>,
) -> anyhow::Result<()> {
    if !skills_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(skills_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let skill: crate::skills::manifest::SkillManifest =
            crate::config_store::read_toml_file(&path)?;
        out.entry(skill.id).or_insert_with(|| source.clone());
    }
    Ok(())
}

fn collect_existing_skill_sources(
    exclude_pack_id: &str,
) -> anyhow::Result<HashMap<String, String>> {
    let mut existing = HashMap::new();

    let user_path = crate::skills::registry::get_user_skills_path()?;
    if user_path.exists() {
        let file: crate::skills::manifest::SkillsFile =
            crate::config_store::read_toml_file(&user_path)?;
        for skill in file.skills {
            existing
                .entry(skill.id)
                .or_insert_with(|| "user skills".to_string());
        }
    }

    let installed_file = list_installed_packs()?;
    let packs_dir = super::get_packs_dir()?;
    for pack in installed_file.installed {
        if pack.id == exclude_pack_id {
            continue;
        }
        collect_skill_ids_from_dir(
            &packs_dir.join(&pack.id).join("skills"),
            format!("pack \"{}\"", pack.id),
            &mut existing,
        )?;
    }

    Ok(existing)
}

fn ensure_pack_skill_ids_can_install(preview: &PackPreview) -> anyhow::Result<()> {
    let mut incoming_ids = HashSet::new();
    for skill in &preview.skill_previews {
        if !incoming_ids.insert(skill.id.clone()) {
            return Err(anyhow::anyhow!(
                "Cannot install pack \"{}\": skill ID \"{}\" appears more than once in the incoming pack.",
                preview.manifest.id,
                skill.id
            ));
        }
    }

    let existing = collect_existing_skill_sources(&preview.manifest.id)?;
    let mut conflicts = Vec::new();
    for skill in &preview.skill_previews {
        if let Some(source) = existing.get(&skill.id) {
            conflicts.push(format!(
                "skill ID \"{}\" already exists in {}",
                skill.id, source
            ));
        }
    }
    conflicts.sort();

    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Cannot install pack \"{}\": {}.",
            preview.manifest.id,
            conflicts.join("; ")
        ))
    }
}

/// Reads the tracking list of installed packs.
pub fn list_installed_packs() -> anyhow::Result<InstalledPacksFile> {
    let path = super::get_installed_packs_path()?;
    if !path.exists() {
        super::init_marketplace()?;
    }
    crate::config_store::read_toml_file::<InstalledPacksFile>(&path)
}

/// Copies a validated pack into the marketplace storage, registering it in installed_packs.toml.
pub fn install_pack_from_path(path: PathBuf) -> anyhow::Result<InstalledPack> {
    // 1. Generate preview and force safety validation
    let preview = preview_pack_from_path(path.clone())?;
    if !preview.safety_summary.allowed_to_install {
        return Err(anyhow::anyhow!(
            "Pack validation failed. Errors: {:?}",
            preview.errors
        ));
    }
    ensure_pack_skill_ids_can_install(&preview)?;

    let pack_id = &preview.manifest.id;
    let target_dir = super::get_packs_dir()?.join(pack_id);

    // 2. Read installed packs log
    let mut installed_file = list_installed_packs()?;

    // V1 Rule: Block duplicate installs of the same version, allow update flow/re-install otherwise
    let duplicate_idx = installed_file
        .installed
        .iter()
        .position(|p| p.id == *pack_id);
    if let Some(idx) = duplicate_idx {
        let existing = &installed_file.installed[idx];
        if existing.version == preview.manifest.version {
            // Already installed the same version, do not block re-install but log warning or allow it
        }
    }

    // 3. Clean and copy files
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .map_err(|e| anyhow::anyhow!("Failed to clear existing pack location: {}", e))?;
    }
    fs::create_dir_all(&target_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create directory for pack: {}", e))?;

    // Copy pack files
    let copy_files = [
        "pack.toml",
        "preferences.toml",
        "contexts.toml",
        "style.toml",
        "theme.toml",
        "commands.toml",
        "workspace_rules.toml",
        "README.md",
    ];

    for file_name in &copy_files {
        let src_file = path.join(file_name);
        if src_file.exists() {
            let dest_file = target_dir.join(file_name);
            fs::copy(&src_file, &dest_file)
                .map_err(|e| anyhow::anyhow!("Failed to copy pack file '{}': {}", file_name, e))?;
        }
    }

    let skills_src = path.join("skills");
    if skills_src.exists() {
        let skills_dest = target_dir.join("skills");
        fs::create_dir_all(&skills_dest)
            .map_err(|e| anyhow::anyhow!("Failed to create skills directory: {}", e))?;
        for entry in fs::read_dir(&skills_src)? {
            let entry = entry?;
            let src_file = entry.path();
            if src_file.is_file() {
                fs::copy(&src_file, skills_dest.join(entry.file_name())).map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to copy skill manifest '{}': {}",
                        src_file.display(),
                        e
                    )
                })?;
            }
        }
    }

    // 4. Update index metadata
    let new_pack = InstalledPack {
        id: pack_id.clone(),
        name: preview.manifest.name.clone(),
        version: preview.manifest.version.clone(),
        installed_at: Utc::now().to_rfc3339(),
        source: path.to_string_lossy().to_string(),
        enabled: true,
    };

    if let Some(idx) = duplicate_idx {
        installed_file.installed[idx] = new_pack.clone();
    } else {
        installed_file.installed.push(new_pack.clone());
    }

    let installed_packs_path = super::get_installed_packs_path()?;
    crate::config_store::save_toml_file(&installed_packs_path, &installed_file)?;

    Ok(new_pack)
}

/// Uninstalls an installed pack, clearing its physical files and deregistering it.
pub fn uninstall_pack(pack_id: &str) -> anyhow::Result<()> {
    let mut installed_file = list_installed_packs()?;
    let idx = installed_file
        .installed
        .iter()
        .position(|p| p.id == pack_id)
        .ok_or_else(|| anyhow::anyhow!("Pack ID '{}' is not installed.", pack_id))?;

    // 1. Remove files
    let target_dir = super::get_packs_dir()?.join(pack_id);
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .map_err(|e| anyhow::anyhow!("Failed to delete pack files: {}", e))?;
    }

    // 2. Deregister addon settings dependencies
    if let Ok(mut addon_settings) = super::addon_settings::read_addon_settings() {
        addon_settings.enabled_packs.retain(|id| id != pack_id);
        addon_settings
            .disabled_contributions
            .retain(|key| !key.starts_with(&format!("{}:", pack_id)));
        if addon_settings.active_theme_source_pack_id.as_deref() == Some(pack_id) {
            addon_settings.active_theme_source_pack_id = None;
            addon_settings.active_theme_id = None;
        }
        let _ = super::addon_settings::save_addon_settings(&addon_settings);
    }

    // 2b. Deregister mode dependencies (legacy cleanup)
    let mut modes_file = super::modes::read_modes()?;
    for mode in &mut modes_file.modes {
        mode.enabled_pack_ids.retain(|id| id != pack_id);
    }
    super::modes::save_modes(&modes_file)?;

    // 3. Remove metadata
    installed_file.installed.remove(idx);
    let installed_packs_path = super::get_installed_packs_path()?;
    crate::config_store::save_toml_file(&installed_packs_path, &installed_file)?;

    Ok(())
}

/// Enables/disables an installed pack.
pub fn enable_pack(pack_id: &str, enabled: bool) -> anyhow::Result<()> {
    let mut installed_file = list_installed_packs()?;
    let idx = installed_file
        .installed
        .iter()
        .position(|p| p.id == pack_id)
        .ok_or_else(|| anyhow::anyhow!("Pack ID '{}' is not installed.", pack_id))?;

    installed_file.installed[idx].enabled = enabled;
    let installed_packs_path = super::get_installed_packs_path()?;
    crate::config_store::save_toml_file(&installed_packs_path, &installed_file)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackActivationCapabilities {
    pub pack_id: String,
    pub has_theme: bool,
    pub theme_id: Option<String>,
    pub theme_name: Option<String>,
    pub has_style: bool,
    pub has_preferences: bool,
    pub has_contexts: bool,
    pub has_command_snippets: bool,
    pub has_workspace_rules: bool,
}

pub fn get_pack_activation_capabilities(
    pack_id: &str,
) -> anyhow::Result<PackActivationCapabilities> {
    // verify pack is installed
    let installed_file = list_installed_packs()?;
    let pack = installed_file
        .installed
        .iter()
        .find(|p| p.id == pack_id)
        .ok_or_else(|| anyhow::anyhow!("Pack '{}' is not installed.", pack_id))?;

    // verify pack is enabled
    if !pack.enabled {
        return Err(anyhow::anyhow!(
            "Pack '{}' is installed but currently disabled.",
            pack_id
        ));
    }

    let pack_dir = super::get_packs_dir()?.join(pack_id);
    if !pack_dir.exists() {
        return Err(anyhow::anyhow!(
            "Installed pack folder for '{}' does not exist.",
            pack_id
        ));
    }

    let mut theme_id = None;
    let mut theme_name = None;
    let mut has_theme = false;

    let theme_path = pack_dir.join("theme.toml");
    if theme_path.exists() {
        has_theme = true;
        if let Ok(content) = fs::read_to_string(&theme_path) {
            if let Ok(theme) = toml::from_str::<super::themes::OpenNivaraTheme>(&content) {
                theme_id = Some(theme.id);
                theme_name = Some(theme.name);
            }
        }
    }

    Ok(PackActivationCapabilities {
        pack_id: pack_id.to_string(),
        has_theme,
        theme_id,
        theme_name,
        has_style: pack_dir.join("style.toml").exists(),
        has_preferences: pack_dir.join("preferences.toml").exists(),
        has_contexts: pack_dir.join("contexts.toml").exists(),
        has_command_snippets: pack_dir.join("commands.toml").exists(),
        has_workspace_rules: pack_dir.join("workspace_rules.toml").exists(),
    })
}
