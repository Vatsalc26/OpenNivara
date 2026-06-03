use super::selector::SelectedSkill;
use crate::tools::{ToolRegistry, ToolsConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillToolPolicyResult {
    pub allowed_tool_names: HashSet<String>,
    pub warnings: Vec<String>,
}

pub fn allowed_tools_for_selected_skills(
    selected_skills: &[SelectedSkill],
    registry: &ToolRegistry,
    config: &ToolsConfig,
) -> SkillToolPolicyResult {
    if selected_skills.is_empty() {
        return SkillToolPolicyResult {
            allowed_tool_names: registry
                .declared_tool_names(config, None)
                .into_iter()
                .collect(),
            warnings: vec![],
        };
    }

    let mut allowed = HashSet::new();
    let mut denied = HashSet::new();
    let mut warnings = Vec::new();

    for skill in selected_skills {
        for tool_name in &skill.allowed_tools {
            if registry.definition(tool_name).is_some() {
                allowed.insert(tool_name.clone());
            } else {
                warnings.push(format!(
                    "Skill '{}' requested unknown tool '{}'; ignored.",
                    skill.id, tool_name
                ));
            }
        }
        for tool_name in &skill.denied_tools {
            denied.insert(tool_name.clone());
        }
    }

    allowed.retain(|name| {
        !denied.contains(name)
            && registry.definition(name).is_some()
            && registry
                .declared_tool_names(config, None)
                .iter()
                .any(|declared| declared == name)
            && !matches!(
                name.as_str(),
                "write_file" | "run_command" | "open_url" | "open_app"
            )
    });

    SkillToolPolicyResult {
        allowed_tool_names: allowed,
        warnings,
    }
}
