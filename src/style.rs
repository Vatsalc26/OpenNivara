use crate::{config_paths, config_store};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenNivaraStyle {
    pub schema_version: u32,
    pub communication: CommunicationStyle,
    pub coding: CodingStyle,
    pub formatting: FormattingStyle,
    pub behavior: BehaviorStyle,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunicationStyle {
    pub tone: String,
    pub detail_level: String,
    pub use_examples: bool,
    pub use_step_by_step: bool,
    pub avoid_unexplained_jargon: bool,
    pub ask_fewer_questions: bool,
    pub prefer_actionable_answers: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodingStyle {
    pub show_simple_solution_first: bool,
    pub explain_after_code: bool,
    pub prefer_mvp_architecture: bool,
    pub avoid_overengineering: bool,
    pub use_beginner_comments: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormattingStyle {
    pub use_markdown: bool,
    pub use_short_sections: bool,
    pub include_next_step: bool,
    pub avoid_long_walls_of_text: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BehaviorStyle {
    pub be_honest_about_uncertainty: bool,
    pub do_not_pretend_to_have_done_things: bool,
    pub do_not_reveal_private_context_unless_relevant: bool,
}

impl OpenNivaraStyle {
    pub fn to_compact_context_string(&self) -> String {
        format!(
            "Stylistic Guidelines:\n\
             - Tone: {}. Detail: {}.\n\
             - Directives: Examples={}, Step-by-step={}, Avoid Jargon={}, Ask Fewer Questions={}, Actionable={}.\n\
             - Coding: Simple solution first={}, Explain after code={}, MVP architecture={}, Avoid overengineering={}, Comments={}.\n\
             - Formatting: Markdown={}, Short sections={}, Include next steps={}, Avoid walls of text={}.\n\
             - Integrity: Be honest={}, Confirm tool actions first={}, Keep contexts private={}.",
            self.communication.tone,
            self.communication.detail_level,
            self.communication.use_examples,
            self.communication.use_step_by_step,
            self.communication.avoid_unexplained_jargon,
            self.communication.ask_fewer_questions,
            self.communication.prefer_actionable_answers,
            self.coding.show_simple_solution_first,
            self.coding.explain_after_code,
            self.coding.prefer_mvp_architecture,
            self.coding.avoid_overengineering,
            self.coding.use_beginner_comments,
            self.formatting.use_markdown,
            self.formatting.use_short_sections,
            self.formatting.include_next_step,
            self.formatting.avoid_long_walls_of_text,
            self.behavior.be_honest_about_uncertainty,
            self.behavior.do_not_pretend_to_have_done_things,
            self.behavior.do_not_reveal_private_context_unless_relevant
        )
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.communication.tone.is_empty() {
            return Err(anyhow::anyhow!("Communication tone cannot be empty."));
        }
        Ok(())
    }
}

pub fn get_style_path() -> anyhow::Result<PathBuf> {
    Ok(config_paths::config_dir()?.join("style.toml"))
}

pub fn save_style(style: &OpenNivaraStyle) -> anyhow::Result<()> {
    style.validate()?;
    let path = get_style_path()?;
    config_store::save_toml_file(&path, style)?;
    Ok(())
}

pub fn init_style() -> anyhow::Result<String> {
    let path = get_style_path()?;

    if path.exists() {
        return Ok(format!(
            "Style configuration file already exists at:\n  {}\n\nYou can edit it directly.",
            path.display()
        ));
    }

    let default_style = OpenNivaraStyle {
        schema_version: 2,
        communication: CommunicationStyle {
            tone: "clear, direct, beginner-friendly".to_string(),
            detail_level: "medium".to_string(),
            use_examples: true,
            use_step_by_step: true,
            avoid_unexplained_jargon: true,
            ask_fewer_questions: true,
            prefer_actionable_answers: true,
        },
        coding: CodingStyle {
            show_simple_solution_first: true,
            explain_after_code: true,
            prefer_mvp_architecture: true,
            avoid_overengineering: true,
            use_beginner_comments: true,
        },
        formatting: FormattingStyle {
            use_markdown: true,
            use_short_sections: true,
            include_next_step: false,
            avoid_long_walls_of_text: true,
        },
        behavior: BehaviorStyle {
            be_honest_about_uncertainty: true,
            do_not_pretend_to_have_done_things: true,
            do_not_reveal_private_context_unless_relevant: true,
        },
    };

    save_style(&default_style)?;

    Ok(format!(
        "Successfully initialized your OpenNivara V2 style guidelines at:\n  {}",
        path.display()
    ))
}

pub fn read_style() -> anyhow::Result<OpenNivaraStyle> {
    let path = get_style_path()?;

    if !path.exists() {
        init_style()?;
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read style file '{}': {}", path.display(), e))?;

    // Standard parse
    if let Ok(style) = toml::from_str::<OpenNivaraStyle>(&content) {
        if style.schema_version == 2 {
            style.validate()?;
            return Ok(style);
        }
    }

    // Migration V1 -> V2 Style check
    let v1_val: toml::Value = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse style TOML during migration check: {}", e))?;

    let tone = v1_val
        .get("communication")
        .and_then(|c| c.get("tone"))
        .and_then(|v| v.as_str())
        .unwrap_or("clear, direct, beginner-friendly")
        .to_string();
    let detail_level = v1_val
        .get("communication")
        .and_then(|c| c.get("detail_level"))
        .and_then(|v| v.as_str())
        .unwrap_or("medium")
        .to_string();
    let use_examples = v1_val
        .get("communication")
        .and_then(|c| c.get("use_examples"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let use_step_by_step = v1_val
        .get("communication")
        .and_then(|c| c.get("use_step_by_step"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let avoid_unexplained_jargon = v1_val
        .get("communication")
        .and_then(|c| c.get("avoid_unexplained_jargon"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let ask_fewer_questions = v1_val
        .get("communication")
        .and_then(|c| c.get("ask_fewer_questions"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let prefer_actionable_answers = v1_val
        .get("communication")
        .and_then(|c| c.get("prefer_actionable_answers"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let show_simple_solution_first = v1_val
        .get("coding_style")
        .and_then(|c| c.get("show_simple_solution_first"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let explain_after_code = v1_val
        .get("coding_style")
        .and_then(|c| c.get("explain_after_code"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let prefer_mvp_architecture = v1_val
        .get("coding_style")
        .and_then(|c| c.get("prefer_mvp_architecture"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let avoid_overengineering = v1_val
        .get("coding_style")
        .and_then(|c| c.get("avoid_overengineering"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let use_beginner_comments = v1_val
        .get("coding_style")
        .and_then(|c| c.get("use_beginner_comments"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let use_markdown = v1_val
        .get("output")
        .and_then(|c| c.get("use_markdown"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let use_short_sections = v1_val
        .get("output")
        .and_then(|c| c.get("use_short_sections"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let include_next_step = v1_val
        .get("output")
        .and_then(|c| c.get("include_next_step"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let avoid_long_walls_of_text = v1_val
        .get("output")
        .and_then(|c| c.get("avoid_long_walls_of_text"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let be_honest_about_uncertainty = v1_val
        .get("behavior")
        .and_then(|c| c.get("be_honest_about_uncertainty"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let do_not_pretend_to_have_done_things = v1_val
        .get("behavior")
        .and_then(|c| c.get("do_not_pretend_to_have_done_things"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let do_not_reveal_private_context_unless_relevant = v1_val
        .get("behavior")
        .and_then(|c| c.get("do_not_reveal_private_context_unless_relevant"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let migrated_style = OpenNivaraStyle {
        schema_version: 2,
        communication: CommunicationStyle {
            tone,
            detail_level,
            use_examples,
            use_step_by_step,
            avoid_unexplained_jargon,
            ask_fewer_questions,
            prefer_actionable_answers,
        },
        coding: CodingStyle {
            show_simple_solution_first,
            explain_after_code,
            prefer_mvp_architecture,
            avoid_overengineering,
            use_beginner_comments,
        },
        formatting: FormattingStyle {
            use_markdown,
            use_short_sections,
            include_next_step,
            avoid_long_walls_of_text,
        },
        behavior: BehaviorStyle {
            be_honest_about_uncertainty,
            do_not_pretend_to_have_done_things,
            do_not_reveal_private_context_unless_relevant,
        },
    };

    save_style(&migrated_style)?;

    Ok(migrated_style)
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn style_path_uses_isolated_test_config_dir() {
        let _lock = crate::config_paths::TEST_CONFIG_ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        std::env::set_var("OPENNIVARA_TEST_CONFIG_DIR", temp_dir.path());

        let path = get_style_path().expect("style path");

        assert_eq!(path, temp_dir.path().join("style.toml"));
        std::env::remove_var("OPENNIVARA_TEST_CONFIG_DIR");
    }
}
