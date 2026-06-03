use super::manifest::SkillManifest;
use super::selector::RouteDecision;

pub fn format_active_skills_prompt(
    decision: &RouteDecision,
    available_skills: &[SkillManifest],
) -> String {
    let selected = decision.selected_skills();
    if selected.is_empty() {
        return String::new();
    }

    let mut block = String::from("Active Skills:\n");
    for selected_skill in selected {
        if let Some(manifest) = available_skills
            .iter()
            .find(|skill| skill.id == selected_skill.id)
        {
            let label = if decision
                .primary_skill
                .as_ref()
                .map(|skill| skill.id.as_str())
                == Some(manifest.id.as_str())
            {
                "Primary Skill"
            } else {
                "Supporting Skill"
            };
            block.push_str(&format!("\n[{}: {}]\n", label, manifest.name));
            block.push_str(&format!("Purpose: {}\n", manifest.description));
            block.push_str(&format!("Role: {}\n", manifest.prompt.role));
            block.push_str("Instructions:\n");
            block.push_str(&manifest.prompt.instructions);
            block.push('\n');
            if !manifest.prompt.constraints.is_empty() {
                block.push_str("\nConstraints:\n");
                for constraint in &manifest.prompt.constraints {
                    block.push_str(&format!("- {}\n", constraint));
                }
            }
            if manifest.safety.requires_fresh_info {
                block.push_str("- This skill may require up-to-date official information. If no web tool is available, clearly say what must be verified externally.\n");
            }
        }
    }

    block
}
