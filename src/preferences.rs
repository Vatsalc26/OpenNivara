use crate::{config_paths, config_store};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreferencesFile {
    pub schema_version: u32,
    #[serde(default)]
    pub sections: Vec<PreferenceSection>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreferenceSection {
    pub id: String,
    pub enabled: bool,
    pub send_policy: String, // always, manual, session_pinned, triggered_strict, never, disabled
    pub description: Option<String>,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub required_any: Vec<String>,
    #[serde(default)]
    pub negative_triggers: Vec<String>,
    pub min_score: u32,
    #[serde(default)]
    pub likes: Vec<PreferenceItem>,
    #[serde(default)]
    pub dislikes: Vec<PreferenceItem>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreferenceItem {
    pub item: String,
    pub strength: u8,
}

impl PreferencesFile {
    pub fn validate(&self) -> anyhow::Result<()> {
        let mut ids = std::collections::HashSet::new();
        for section in &self.sections {
            if section.id.is_empty() {
                return Err(anyhow::anyhow!("Preference Section ID cannot be empty."));
            }
            if section.id != section.id.to_lowercase() || section.id.contains(' ') {
                return Err(anyhow::anyhow!(
                    "Preference Section ID '{}' must be lowercase and snake_case.",
                    section.id
                ));
            }
            if !ids.insert(section.id.clone()) {
                return Err(anyhow::anyhow!(
                    "Duplicate Preference Section ID found: '{}'.",
                    section.id
                ));
            }

            let valid_policies = [
                "always",
                "manual",
                "session_pinned",
                "triggered_strict",
                "never",
                "disabled",
            ];
            if !valid_policies.contains(&section.send_policy.as_str()) {
                return Err(anyhow::anyhow!(
                    "Invalid send_policy '{}' in section '{}'. Allowed: always, manual, session_pinned, triggered_strict, never, disabled.",
                    section.send_policy, section.id
                ));
            }

            for like in &section.likes {
                if like.strength < 1 || like.strength > 5 {
                    return Err(anyhow::anyhow!(
                        "Invalid strength ({}) for liked item '{}' in section '{}'. Must be between 1 and 5.",
                        like.strength, like.item, section.id
                    ));
                }
            }

            for dislike in &section.dislikes {
                if dislike.strength < 1 || dislike.strength > 5 {
                    return Err(anyhow::anyhow!(
                        "Invalid strength ({}) for disliked item '{}' in section '{}'. Must be between 1 and 5.",
                        dislike.strength, dislike.item, section.id
                    ));
                }
            }

            if section.send_policy == "triggered_strict" {
                if section.triggers.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Triggered_strict section '{}' must have at least one trigger keyword.",
                        section.id
                    ));
                }
                if section.min_score < 1 {
                    return Err(anyhow::anyhow!(
                        "Triggered_strict section '{}' min_score must be at least 1.",
                        section.id
                    ));
                }
            }
        }
        Ok(())
    }
}

pub fn get_preferences_path() -> anyhow::Result<PathBuf> {
    Ok(config_paths::config_dir()?.join("preferences.toml"))
}

pub fn save_preferences(preferences: &PreferencesFile) -> anyhow::Result<()> {
    preferences.validate()?;
    let path = get_preferences_path()?;
    config_store::save_toml_file(&path, preferences)?;
    Ok(())
}

pub fn init_preferences() -> anyhow::Result<String> {
    let path = get_preferences_path()?;

    if path.exists() {
        return Ok(format!(
            "Preferences file already exists at:\n  {}\n\nYou can edit it directly.",
            path.display()
        ));
    }

    let default_preferences = PreferencesFile {
        schema_version: 2,
        sections: vec![
            PreferenceSection {
                id: "coding_help".to_string(),
                enabled: true,
                send_policy: "triggered_strict".to_string(),
                description: Some("Preferences for coding help".to_string()),
                triggers: vec![
                    "code".to_string(),
                    "coding".to_string(),
                    "rust".to_string(),
                    "cargo".to_string(),
                    "compiler".to_string(),
                    "bug".to_string(),
                    "error".to_string(),
                ],
                required_any: vec![
                    "code".to_string(),
                    "rust".to_string(),
                    "cargo".to_string(),
                    "compiler".to_string(),
                    "bug".to_string(),
                    "error".to_string(),
                ],
                negative_triggers: vec![],
                min_score: 2,
                likes: vec![
                    PreferenceItem {
                        item: "step-by-step explanations".to_string(),
                        strength: 5,
                    },
                    PreferenceItem {
                        item: "simple working code first".to_string(),
                        strength: 5,
                    },
                ],
                dislikes: vec![PreferenceItem {
                    item: "over-engineered architecture".to_string(),
                    strength: 4,
                }],
                notes: vec![
                    "Prefer simple MVP architecture before advanced abstractions.".to_string(),
                ],
            },
            PreferenceSection {
                id: "food".to_string(),
                enabled: true,
                send_policy: "triggered_strict".to_string(),
                description: Some("Food preferences".to_string()),
                triggers: vec![
                    "food".to_string(),
                    "restaurant".to_string(),
                    "eat".to_string(),
                    "lunch".to_string(),
                    "dinner".to_string(),
                    "recipe".to_string(),
                    "cook".to_string(),
                    "meal".to_string(),
                ],
                required_any: vec![],
                negative_triggers: vec![],
                min_score: 2,
                likes: vec![
                    PreferenceItem {
                        item: "spicy food".to_string(),
                        strength: 4,
                    },
                    PreferenceItem {
                        item: "quick meals".to_string(),
                        strength: 3,
                    },
                ],
                dislikes: vec![PreferenceItem {
                    item: "very sweet food".to_string(),
                    strength: 3,
                }],
                notes: vec!["Use these preferences only for food recommendations.".to_string()],
            },
        ],
    };

    save_preferences(&default_preferences)?;

    Ok(format!(
        "Successfully initialized your OpenNivara V2 preferences at:\n  {}",
        path.display()
    ))
}

pub fn read_preferences() -> anyhow::Result<PreferencesFile> {
    let path = get_preferences_path()?;

    if !path.exists() {
        init_preferences()?;
    }

    let content = fs::read_to_string(&path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read preferences file '{}': {}",
            path.display(),
            e
        )
    })?;

    // Standard parse
    if let Ok(prefs) = toml::from_str::<PreferencesFile>(&content) {
        if prefs.schema_version == 2 {
            prefs.validate()?;
            return Ok(prefs);
        }
    }

    // Migration V1 -> V2 Preferences check
    let v1_val: toml::Value = toml::from_str(&content).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse preferences TOML during migration check: {}",
            e
        )
    })?;

    let mut migrated_sections = Vec::new();
    if let Some(sections_array) = v1_val.get("sections").and_then(|s| s.as_array()) {
        for sec in sections_array {
            let id = sec
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let description = sec
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mut triggers = Vec::new();
            if let Some(trig_array) = sec.get("triggers").and_then(|t| t.as_array()) {
                for t in trig_array {
                    if let Some(s) = t.as_str() {
                        triggers.push(s.to_string());
                    }
                }
            }

            let mut likes = Vec::new();
            if let Some(likes_array) = sec.get("likes").and_then(|l| l.as_array()) {
                for l in likes_array {
                    let item = l
                        .get("item")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let strength =
                        l.get("strength").and_then(|v| v.as_integer()).unwrap_or(3) as u8;
                    likes.push(PreferenceItem { item, strength });
                }
            }

            let mut dislikes = Vec::new();
            if let Some(dislikes_array) = sec.get("dislikes").and_then(|l| l.as_array()) {
                for l in dislikes_array {
                    let item = l
                        .get("item")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let strength =
                        l.get("strength").and_then(|v| v.as_integer()).unwrap_or(3) as u8;
                    dislikes.push(PreferenceItem { item, strength });
                }
            }

            let mut notes = Vec::new();
            if let Some(notes_array) = sec.get("notes").and_then(|n| n.as_array()) {
                for n in notes_array {
                    if let Some(s) = n.as_str() {
                        notes.push(s.to_string());
                    }
                }
            }

            let required_any = if id == "coding" {
                vec![
                    "code".to_string(),
                    "rust".to_string(),
                    "cargo".to_string(),
                    "compiler".to_string(),
                    "bug".to_string(),
                    "error".to_string(),
                ]
            } else {
                vec![]
            };

            let v2_id = if id == "coding" {
                "coding_help".to_string()
            } else {
                id
            };

            migrated_sections.push(PreferenceSection {
                id: v2_id,
                enabled: true,
                send_policy: "triggered_strict".to_string(),
                description,
                triggers,
                required_any,
                negative_triggers: vec![],
                min_score: 2,
                likes,
                dislikes,
                notes,
            });
        }
    }

    let migrated_prefs = PreferencesFile {
        schema_version: 2,
        sections: migrated_sections,
    };

    save_preferences(&migrated_prefs)?;

    Ok(migrated_prefs)
}

/// Formats the selected relevant preference sections into a clean text block
pub fn format_relevant_preferences(sections: &[PreferenceSection]) -> String {
    if sections.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    output.push_str("Relevant Private Preference Context:\n\n");

    for section in sections {
        output.push_str(&format!("Section: {}\n", section.id));
        if let Some(desc) = &section.description {
            output.push_str(&format!("Description: {}\n", desc));
        }

        if !section.likes.is_empty() {
            output.push_str("Likes:\n");
            for like in &section.likes {
                output.push_str(&format!("- {} (strength: {})\n", like.item, like.strength));
            }
        }

        if !section.dislikes.is_empty() {
            output.push_str("Dislikes:\n");
            for dislike in &section.dislikes {
                output.push_str(&format!(
                    "- {} (strength: {})\n",
                    dislike.item, dislike.strength
                ));
            }
        }

        if !section.notes.is_empty() {
            output.push_str("Guidance Notes:\n");
            for note in &section.notes {
                output.push_str(&format!("- {}\n", note));
            }
        }
        output.push_str("\n---\n\n");
    }

    output.push_str(
        "Preference strength meaning:\n\
         1 = weak signal\n\
         3 = moderate signal\n\
         5 = strong signal\n\n\
         Use these preferences only when relevant. Strong dislikes should usually be avoided unless the user explicitly asks for them. Likes should rank suggestions, not completely block other good answers."
    );

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn preferences_path_uses_isolated_test_config_dir() {
        let _lock = crate::config_paths::TEST_CONFIG_ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        std::env::set_var("OPENNIVARA_TEST_CONFIG_DIR", temp_dir.path());

        let path = get_preferences_path().expect("preferences path");

        assert_eq!(path, temp_dir.path().join("preferences.toml"));
        std::env::remove_var("OPENNIVARA_TEST_CONFIG_DIR");
    }

    #[test]
    fn test_preferences_validation() {
        // Valid file
        let valid_sec = PreferenceSection {
            id: "valid_section_id".to_string(),
            enabled: true,
            send_policy: "always".to_string(),
            description: None,
            triggers: vec![],
            required_any: vec![],
            negative_triggers: vec![],
            min_score: 0,
            likes: vec![PreferenceItem {
                item: "rust".to_string(),
                strength: 5,
            }],
            dislikes: vec![PreferenceItem {
                item: "python".to_string(),
                strength: 1,
            }],
            notes: vec![],
        };
        let valid_file = PreferencesFile {
            schema_version: 1,
            sections: vec![valid_sec],
        };
        assert!(valid_file.validate().is_ok());

        // Invalid empty ID
        let mut invalid_sec = valid_file.sections[0].clone();
        invalid_sec.id = "".to_string();
        let invalid_file = PreferencesFile {
            schema_version: 1,
            sections: vec![invalid_sec],
        };
        assert!(invalid_file.validate().is_err());

        // Invalid uppercase ID
        let mut invalid_sec2 = valid_file.sections[0].clone();
        invalid_sec2.id = "Uppercase".to_string();
        let invalid_file2 = PreferencesFile {
            schema_version: 1,
            sections: vec![invalid_sec2],
        };
        assert!(invalid_file2.validate().is_err());

        // Invalid send_policy
        let mut invalid_sec3 = valid_file.sections[0].clone();
        invalid_sec3.send_policy = "invalid_policy".to_string();
        let invalid_file3 = PreferencesFile {
            schema_version: 1,
            sections: vec![invalid_sec3],
        };
        assert!(invalid_file3.validate().is_err());

        // Invalid strength (too high)
        let mut invalid_sec4 = valid_file.sections[0].clone();
        invalid_sec4.likes[0].strength = 6;
        let invalid_file4 = PreferencesFile {
            schema_version: 1,
            sections: vec![invalid_sec4],
        };
        assert!(invalid_file4.validate().is_err());

        // Invalid strength (too low)
        let mut invalid_sec5 = valid_file.sections[0].clone();
        invalid_sec5.likes[0].strength = 0;
        let invalid_file5 = PreferencesFile {
            schema_version: 1,
            sections: vec![invalid_sec5],
        };
        assert!(invalid_file5.validate().is_err());
    }

    #[test]
    fn test_format_relevant_preferences() {
        let sec = PreferenceSection {
            id: "my_likes".to_string(),
            enabled: true,
            send_policy: "always".to_string(),
            description: Some("desc".to_string()),
            triggers: vec![],
            required_any: vec![],
            negative_triggers: vec![],
            min_score: 0,
            likes: vec![PreferenceItem {
                item: "rust".to_string(),
                strength: 5,
            }],
            dislikes: vec![PreferenceItem {
                item: "java".to_string(),
                strength: 2,
            }],
            notes: vec!["note1".to_string()],
        };

        let formatted = format_relevant_preferences(&[sec]);
        assert!(formatted.contains("Section: my_likes"));
        assert!(formatted.contains("Description: desc"));
        assert!(formatted.contains("- rust (strength: 5)"));
        assert!(formatted.contains("- java (strength: 2)"));
        assert!(formatted.contains("- note1"));
    }
}
