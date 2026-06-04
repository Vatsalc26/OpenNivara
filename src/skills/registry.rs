use super::manifest::{validate_skill_manifest, SkillManifest, SkillRoutePolicy, SkillsFile};
use super::selector::{select_skill_route, RouteDecision, SkillRouteRequest};
use crate::tools::ToolRegistry;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnabledSkillsFile {
    pub schema_version: u32,
    #[serde(default)]
    pub skills: Vec<SkillState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillState {
    pub skill_id: String,
    #[serde(default)]
    pub pack_id: Option<String>,
    pub enabled: bool,
    #[serde(default)]
    pub route_policy_override: Option<SkillRoutePolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub id: String,
    pub pack_id: Option<String>,
    pub name: String,
    pub description: String,
    pub category: String,
    pub enabled: bool,
    pub route_policy: SkillRoutePolicy,
    pub risk_level: String,
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub denied_tools: Vec<String>,
    #[serde(default)]
    pub exam: Option<String>,
    #[serde(default)]
    pub exam_stage: Option<String>,
    #[serde(default)]
    pub audience: Vec<String>,
    #[serde(default)]
    pub language_style: Vec<String>,
    #[serde(default)]
    pub freshness_sensitive: bool,
    #[serde(default)]
    pub official_source_labels: Vec<String>,
    #[serde(default)]
    pub best_for: Vec<String>,
    #[serde(default)]
    pub not_for: Vec<String>,
}

pub fn get_skills_dir() -> anyhow::Result<PathBuf> {
    Ok(crate::config_paths::config_dir()?.join("skills"))
}

pub fn get_user_skills_path() -> anyhow::Result<PathBuf> {
    Ok(get_skills_dir()?.join("skills.toml"))
}

pub fn get_enabled_skills_path() -> anyhow::Result<PathBuf> {
    Ok(get_skills_dir()?.join("enabled_skills.toml"))
}

pub fn init_skills() -> anyhow::Result<String> {
    fs::create_dir_all(get_skills_dir()?)?;
    let enabled_path = get_enabled_skills_path()?;
    if !enabled_path.exists() {
        crate::config_store::save_toml_file(
            &enabled_path,
            &EnabledSkillsFile {
                schema_version: 1,
                skills: vec![],
            },
        )?;
    }
    let user_path = get_user_skills_path()?;
    if !user_path.exists() {
        crate::config_store::save_toml_file(
            &user_path,
            &SkillsFile {
                schema_version: 1,
                skills: vec![],
            },
        )?;
    }
    Ok(format!(
        "Successfully initialized OpenNivara skills configuration at:\n  {}",
        get_skills_dir()?.display()
    ))
}

pub fn read_enabled_skills() -> anyhow::Result<EnabledSkillsFile> {
    let path = get_enabled_skills_path()?;
    if !path.exists() {
        init_skills()?;
    }
    crate::config_store::read_toml_file(&path)
}

pub fn save_enabled_skills(file: &EnabledSkillsFile) -> anyhow::Result<()> {
    crate::config_store::save_toml_file(&get_enabled_skills_path()?, file)
}

pub fn set_skill_enabled(skill_id: &str, enabled: bool) -> anyhow::Result<()> {
    let skills = load_available_skills()?;
    let manifest = skills
        .iter()
        .find(|skill| skill.id == skill_id)
        .ok_or_else(|| anyhow::anyhow!("Skill '{}' is not installed.", skill_id))?;
    let mut file = read_enabled_skills()?;
    if let Some(state) = file
        .skills
        .iter_mut()
        .find(|state| state.skill_id == skill_id)
    {
        state.enabled = enabled;
        state.pack_id = manifest.pack_id.clone();
    } else {
        file.skills.push(SkillState {
            skill_id: skill_id.to_string(),
            pack_id: manifest.pack_id.clone(),
            enabled,
            route_policy_override: None,
        });
    }
    save_enabled_skills(&file)
}

pub fn list_skill_summaries() -> anyhow::Result<Vec<SkillSummary>> {
    let skills = load_available_skills()?;
    let mut summaries: Vec<_> = skills.iter().map(skill_summary).collect();
    summaries.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(summaries)
}

pub fn get_skill(skill_id: &str) -> anyhow::Result<SkillManifest> {
    load_available_skills()?
        .into_iter()
        .find(|skill| skill.id == skill_id)
        .ok_or_else(|| anyhow::anyhow!("Skill '{}' is not installed.", skill_id))
}

pub fn test_route(message: String) -> anyhow::Result<RouteDecision> {
    let skills = load_routable_skills()?;
    Ok(select_skill_route(
        &skills,
        SkillRouteRequest {
            message,
            explicit_skill_id: None,
            pack_hint: None,
            ui_selected_skill_id: None,
            session_pinned_skill_ids: vec![],
        },
    ))
}

pub fn load_routable_skills() -> anyhow::Result<Vec<SkillManifest>> {
    Ok(load_available_skills()?
        .into_iter()
        .filter(|skill| skill.enabled)
        .collect())
}

pub fn load_available_skills() -> anyhow::Result<Vec<SkillManifest>> {
    let tool_registry = ToolRegistry::new(true);
    let mut skills = Vec::new();

    let user_path = get_user_skills_path()?;
    if user_path.exists() {
        let file: SkillsFile = crate::config_store::read_toml_file(&user_path)?;
        for skill in file.skills {
            validate_skill_manifest(&skill, &tool_registry).map_err(|e| anyhow::anyhow!(e))?;
            skills.push(skill);
        }
    }

    let pack_enabled = installed_pack_enabled_map()?;
    let installed_pack_ids: HashSet<_> = pack_enabled.keys().cloned().collect();
    let packs_dir = crate::marketplace::get_packs_dir()?;
    if packs_dir.exists() {
        for entry in fs::read_dir(packs_dir)? {
            let entry = entry?;
            let pack_dir = entry.path();
            if !pack_dir.is_dir() {
                continue;
            }
            let pack_id = entry.file_name().to_string_lossy().to_string();
            if !installed_pack_ids.contains(&pack_id) {
                continue;
            }
            skills.extend(load_pack_skills(&pack_dir, &pack_id)?);
        }
    }

    validate_global_unique_skill_ids(&skills)?;
    apply_user_state(skills)
}

fn validate_global_unique_skill_ids(skills: &[SkillManifest]) -> anyhow::Result<()> {
    let mut seen: HashMap<&str, Vec<String>> = HashMap::new();
    for skill in skills {
        let source = skill
            .pack_id
            .as_ref()
            .map(|pack_id| format!("pack '{}'", pack_id))
            .unwrap_or_else(|| "user skills".to_string());
        seen.entry(skill.id.as_str()).or_default().push(source);
    }

    let mut duplicates: Vec<String> = seen
        .into_iter()
        .filter_map(|(id, sources)| {
            if sources.len() > 1 {
                Some(format!("{} ({})", id, sources.join(", ")))
            } else {
                None
            }
        })
        .collect();
    duplicates.sort();

    if duplicates.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Duplicate skill IDs are not allowed in Skills v1. Duplicates: {}",
            duplicates.join("; ")
        ))
    }
}

fn load_pack_skills(pack_dir: &Path, pack_id: &str) -> anyhow::Result<Vec<SkillManifest>> {
    let skills_dir = pack_dir.join("skills");
    if !skills_dir.exists() {
        return Ok(vec![]);
    }
    let tool_registry = ToolRegistry::new(true);
    let mut skills = Vec::new();
    for entry in fs::read_dir(skills_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let mut skill: SkillManifest = crate::config_store::read_toml_file(&path)?;
        skill.pack_id = Some(skill.pack_id.unwrap_or_else(|| pack_id.to_string()));
        validate_skill_manifest(&skill, &tool_registry).map_err(|e| anyhow::anyhow!(e))?;
        skill.enabled = false;
        skills.push(skill);
    }
    Ok(skills)
}

fn apply_user_state(mut skills: Vec<SkillManifest>) -> anyhow::Result<Vec<SkillManifest>> {
    let pack_enabled = installed_pack_enabled_map()?;
    let state = read_enabled_skills().unwrap_or(EnabledSkillsFile {
        schema_version: 1,
        skills: vec![],
    });
    for skill in &mut skills {
        let parent_pack_enabled = skill
            .pack_id
            .as_ref()
            .and_then(|pack_id| pack_enabled.get(pack_id))
            .copied()
            .unwrap_or(true);
        if !parent_pack_enabled {
            skill.enabled = false;
            continue;
        }
        if let Some(skill_state) = state
            .skills
            .iter()
            .find(|state| state.skill_id == skill.id && state.pack_id == skill.pack_id)
        {
            skill.enabled = skill_state.enabled;
            if let Some(policy) = &skill_state.route_policy_override {
                skill.route_policy = policy.clone();
            }
        } else if skill.pack_id.is_some() {
            skill.enabled = false;
        }
    }
    Ok(skills)
}

fn installed_pack_enabled_map() -> anyhow::Result<HashMap<String, bool>> {
    let installed = crate::marketplace::packs::list_installed_packs().unwrap_or(
        crate::marketplace::packs::InstalledPacksFile {
            schema_version: 1,
            installed: vec![],
        },
    );
    Ok(installed
        .installed
        .into_iter()
        .map(|pack| (pack.id, pack.enabled))
        .collect())
}

fn skill_summary(skill: &SkillManifest) -> SkillSummary {
    SkillSummary {
        id: skill.id.clone(),
        pack_id: skill.pack_id.clone(),
        name: skill.name.clone(),
        description: skill.description.clone(),
        category: skill.category.clone(),
        enabled: skill.enabled,
        route_policy: skill.route_policy.clone(),
        risk_level: skill.safety.risk_level.clone(),
        allowed_tools: skill.tools.allow.clone(),
        denied_tools: skill.tools.deny.clone(),
        exam: skill.metadata.exam.clone(),
        exam_stage: skill.metadata.exam_stage.clone(),
        audience: skill.metadata.audience.clone(),
        language_style: skill.metadata.language_style.clone(),
        freshness_sensitive: skill.metadata.freshness_sensitive,
        official_source_labels: skill.metadata.official_source_labels.clone(),
        best_for: skill.store_preview.best_for.clone(),
        not_for: skill.store_preview.not_for.clone(),
    }
}
