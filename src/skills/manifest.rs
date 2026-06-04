use crate::tools::ToolRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsFile {
    pub schema_version: u32,
    #[serde(default)]
    pub skills: Vec<SkillManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub schema_version: u32,
    pub id: String,
    #[serde(default)]
    pub pack_id: Option<String>,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub category: String,
    pub route_policy: SkillRoutePolicy,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub required_any: Vec<String>,
    #[serde(default)]
    pub negative_triggers: Vec<String>,
    #[serde(default)]
    pub examples: Vec<String>,
    pub min_score: u32,
    pub prompt: SkillPrompt,
    pub tools: SkillToolPolicy,
    pub safety: SkillSafety,
    #[serde(default)]
    pub metadata: SkillMetadata,
    #[serde(default)]
    pub store_preview: SkillStorePreview,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillMetadata {
    #[serde(default)]
    pub audience: Vec<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub exam: Option<String>,
    #[serde(default)]
    pub exam_stage: Option<String>,
    #[serde(default)]
    pub language_style: Vec<String>,
    #[serde(default)]
    pub last_reviewed_at: Option<String>,
    #[serde(default)]
    pub freshness_sensitive: bool,
    #[serde(default)]
    pub official_source_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillStorePreview {
    #[serde(default)]
    pub best_for: Vec<String>,
    #[serde(default)]
    pub not_for: Vec<String>,
    #[serde(default)]
    pub sample_prompts: Vec<String>,
    #[serde(default, alias = "what_it_will_do")]
    pub what_it_does: Vec<String>,
    #[serde(default)]
    pub what_it_will_not_do: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillRoutePolicy {
    Auto,
    ManualOnly,
    ExplicitOnly,
    SuggestOnly,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPrompt {
    pub role: String,
    pub instructions: String,
    #[serde(default)]
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillToolPolicy {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSafety {
    pub risk_level: String,
    pub requires_confirmation: bool,
    pub allows_file_write: bool,
    pub allows_shell: bool,
    pub allows_network: bool,
    #[serde(default)]
    pub requires_fresh_info: bool,
}

pub fn validate_skills_file(
    file: &SkillsFile,
    tool_registry: &ToolRegistry,
) -> Result<Vec<String>, String> {
    if file.schema_version != 1 {
        return Err("Skills file schema_version must be exactly 1.".to_string());
    }

    let mut ids = HashSet::new();
    let mut warnings = Vec::new();
    for skill in &file.skills {
        if !ids.insert(skill.id.clone()) {
            return Err(format!("Duplicate skill ID '{}'.", skill.id));
        }
        warnings.extend(validate_skill_manifest(skill, tool_registry)?);
    }
    Ok(warnings)
}

pub fn validate_skill_manifest(
    skill: &SkillManifest,
    tool_registry: &ToolRegistry,
) -> Result<Vec<String>, String> {
    if skill.schema_version != 1 {
        return Err(format!(
            "Skill '{}' schema_version must be exactly 1.",
            skill.id
        ));
    }
    if !is_safe_skill_id(&skill.id) {
        return Err(format!(
            "Skill ID '{}' must use lowercase letters, digits, underscores, or hyphens.",
            skill.id
        ));
    }
    if skill.route_policy == SkillRoutePolicy::Auto
        && skill.aliases.is_empty()
        && skill.triggers.is_empty()
        && skill.examples.is_empty()
    {
        return Err(format!(
            "Auto skill '{}' requires at least one alias, trigger, or example.",
            skill.id
        ));
    }
    if skill.route_policy == SkillRoutePolicy::Auto && skill.min_score == 0 {
        return Err(format!("Auto skill '{}' requires min_score > 0.", skill.id));
    }
    if skill.safety.allows_shell {
        return Err(format!(
            "Skill '{}' cannot enable shell access in Skills v1.",
            skill.id
        ));
    }
    if skill.safety.allows_file_write {
        return Err(format!(
            "Skill '{}' cannot enable file writes in Skills v1.",
            skill.id
        ));
    }

    for tool_name in &skill.tools.allow {
        if tool_registry.definition(tool_name).is_none() {
            return Err(format!(
                "Skill '{}' references unknown tool '{}'.",
                skill.id, tool_name
            ));
        }
    }
    for tool_name in &skill.tools.deny {
        if tool_registry.definition(tool_name).is_none() && !is_reserved_deny_only_tool(tool_name) {
            return Err(format!(
                "Skill '{}' references unknown tool '{}'.",
                skill.id, tool_name
            ));
        }
    }

    if skill.metadata.freshness_sensitive && !skill.safety.requires_fresh_info {
        return Err(format!(
            "Skill '{}' sets metadata.freshness_sensitive but safety.requires_fresh_info is false.",
            skill.id
        ));
    }
    if skill.safety.requires_fresh_info && skill.metadata.official_source_labels.is_empty() {
        return Err(format!(
            "Skill '{}' requires fresh info but metadata.official_source_labels is empty.",
            skill.id
        ));
    }

    let mut warnings = Vec::new();
    if skill.safety.allows_network {
        warnings.push(format!(
            "Skill '{}' declares network metadata, but Skills v1 exposes no network tools.",
            skill.id
        ));
    }
    if skill.store_preview.sample_prompts.is_empty() {
        warnings.push(format!(
            "Skill '{}' has no store_preview.sample_prompts.",
            skill.id
        ));
    }
    if skill.store_preview.best_for.is_empty() {
        warnings.push(format!(
            "Skill '{}' has no store_preview.best_for.",
            skill.id
        ));
    }
    if skill.metadata.freshness_sensitive && skill.store_preview.what_it_will_not_do.is_empty() {
        warnings.push(format!(
            "Skill '{}' is freshness-sensitive but has no store_preview.what_it_will_not_do.",
            skill.id
        ));
    }
    Ok(warnings)
}

fn is_reserved_deny_only_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "write_file" | "run_command" | "open_url" | "open_app"
    )
}

pub fn is_safe_skill_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}
