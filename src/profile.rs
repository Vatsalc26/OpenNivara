use crate::{config_paths, config_store};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub schema_version: u32,
    pub identity: Identity,
    pub location: Location,
    pub languages: Languages,
    pub technical: Technical,
    pub personal: Personal,
    pub privacy: Privacy,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Identity {
    pub display_name: String,
    #[serde(default)]
    pub full_name: String,
    #[serde(default)]
    pub gender: String,
    #[serde(default)]
    pub pronouns: String,
    #[serde(default)]
    pub date_of_birth: String,
    pub timezone: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub state_or_region: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub living_situation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Languages {
    pub preferred_human_language: String,
    #[serde(default)]
    pub other_human_languages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Technical {
    pub coding_level: String,
    pub preferred_coding_languages: Vec<String>,
    pub current_os: String,
    pub main_editor: String,
    pub secondary_editor: String,
    #[serde(default)]
    pub terminal: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Personal {
    pub occupation_or_role: String,
    #[serde(default)]
    pub education_level: String,
    #[serde(default)]
    pub interests: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Privacy {
    pub send_identity: bool,
    pub send_location: bool,
    pub send_gender: bool,
    pub send_technical: bool,
    pub send_personal: bool,
}

impl Profile {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.identity.display_name.is_empty() {
            return Err(anyhow::anyhow!("Display name cannot be empty."));
        }
        if self.languages.preferred_human_language.is_empty() {
            return Err(anyhow::anyhow!("Preferred human language cannot be empty."));
        }
        let valid_levels = ["beginner", "intermediate", "expert", "new to programming"];
        let coding_level_lower = self.technical.coding_level.to_lowercase();
        if !valid_levels.contains(&coding_level_lower.as_str()) {
            // Keep user value but warning or allow standard values
        }
        Ok(())
    }

    pub fn to_compact_context_string(&self) -> String {
        let mut context = "User Profile Context:\n".to_string();

        if self.privacy.send_identity {
            context.push_str(&format!(" - Name: {}\n", self.identity.display_name));
            if !self.identity.full_name.is_empty() {
                context.push_str(&format!(" - Full Name: {}\n", self.identity.full_name));
            }
            if !self.identity.timezone.is_empty() {
                context.push_str(&format!(" - Timezone: {}\n", self.identity.timezone));
            }
        } else {
            if !self.identity.timezone.is_empty() {
                context.push_str(&format!(" - Timezone: {}\n", self.identity.timezone));
            }
        }

        if self.privacy.send_location {
            if !self.location.country.is_empty() {
                context.push_str(&format!(" - Country: {}\n", self.location.country));
            }
            if !self.location.state_or_region.is_empty() {
                context.push_str(&format!(" - Region: {}\n", self.location.state_or_region));
            }
            if !self.location.city.is_empty() {
                context.push_str(&format!(" - City: {}\n", self.location.city));
            }
            if !self.location.living_situation.is_empty() {
                context.push_str(&format!(
                    " - Living Situation: {}\n",
                    self.location.living_situation
                ));
            }
        }

        if self.privacy.send_gender {
            if !self.identity.gender.is_empty() {
                context.push_str(&format!(" - Gender: {}\n", self.identity.gender));
            }
            if !self.identity.pronouns.is_empty() {
                context.push_str(&format!(" - Pronouns: {}\n", self.identity.pronouns));
            }
        }

        if self.privacy.send_technical {
            context.push_str(&format!(
                " - Experience Level: {}\n",
                self.technical.coding_level
            ));
            if !self.technical.preferred_coding_languages.is_empty() {
                context.push_str(&format!(
                    " - Preferred Programming Languages: {}\n",
                    self.technical.preferred_coding_languages.join(", ")
                ));
            }
            if !self.technical.current_os.is_empty() {
                context.push_str(&format!(" - OS: {}\n", self.technical.current_os));
            }
            if !self.technical.main_editor.is_empty() {
                context.push_str(&format!(" - Main Editor: {}\n", self.technical.main_editor));
            }
            if !self.technical.secondary_editor.is_empty() {
                context.push_str(&format!(
                    " - Secondary Editor: {}\n",
                    self.technical.secondary_editor
                ));
            }
            if !self.technical.terminal.is_empty() {
                context.push_str(&format!(" - Terminal: {}\n", self.technical.terminal));
            }
        }

        if self.privacy.send_personal {
            if !self.personal.occupation_or_role.is_empty() {
                context.push_str(&format!(
                    " - Occupation/Role: {}\n",
                    self.personal.occupation_or_role
                ));
            }
            if !self.personal.education_level.is_empty() {
                context.push_str(&format!(
                    " - Education Level: {}\n",
                    self.personal.education_level
                ));
            }
            if !self.personal.interests.is_empty() {
                context.push_str(&format!(
                    " - Interests: {}\n",
                    self.personal.interests.join(", ")
                ));
            }
        }

        if !self.languages.preferred_human_language.is_empty() {
            context.push_str(&format!(
                " - Preferred Human Language: {}\n",
                self.languages.preferred_human_language
            ));
        }

        context
    }
}

pub fn get_profile_path() -> anyhow::Result<PathBuf> {
    Ok(config_paths::config_dir()?.join("profile.toml"))
}

pub fn save_profile(profile: &Profile) -> anyhow::Result<()> {
    profile.validate()?;
    let path = get_profile_path()?;
    config_store::save_toml_file(&path, profile)?;
    Ok(())
}

pub fn init_profile() -> anyhow::Result<String> {
    let path = get_profile_path()?;

    if path.exists() {
        return Ok(format!(
            "Profile file already exists at:\n  {}\n\nYou can edit it directly.",
            path.display()
        ));
    }

    let default_profile = Profile {
        schema_version: 2,
        identity: Identity {
            display_name: "Example User".to_string(),
            full_name: "".to_string(),
            gender: "".to_string(),
            pronouns: "".to_string(),
            date_of_birth: "".to_string(),
            timezone: "UTC".to_string(),
        },
        location: Location {
            country: "".to_string(),
            state_or_region: "".to_string(),
            city: "".to_string(),
            living_situation: "".to_string(),
        },
        languages: Languages {
            preferred_human_language: "English".to_string(),
            other_human_languages: vec![],
        },
        technical: Technical {
            coding_level: "".to_string(),
            preferred_coding_languages: vec![],
            current_os: "".to_string(),
            main_editor: "".to_string(),
            secondary_editor: "".to_string(),
            terminal: "".to_string(),
        },
        personal: Personal {
            occupation_or_role: "".to_string(),
            education_level: "".to_string(),
            interests: vec![],
        },
        privacy: Privacy {
            send_identity: true,
            send_location: false,
            send_gender: false,
            send_technical: true,
            send_personal: false,
        },
    };

    save_profile(&default_profile)?;

    // We also trigger contexts initialization to migrate goals or ensure contexts file exists
    let _ = crate::context::init_contexts();

    Ok(format!(
        "Successfully initialized your OpenNivara V2 profile at:\n  {}",
        path.display()
    ))
}

pub fn read_profile() -> anyhow::Result<Profile> {
    let path = get_profile_path()?;

    if !path.exists() {
        init_profile()?;
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read profile file '{}': {}", path.display(), e))?;

    // Attempt standard deserialize
    if let Ok(profile) = toml::from_str::<Profile>(&content) {
        if profile.schema_version == 2 {
            profile.validate()?;
            return Ok(profile);
        }
    }

    // Migration V1 -> V2 logic
    let v1_val: toml::Value = toml::from_str(&content).map_err(|e| {
        anyhow::anyhow!("Failed to parse profile TOML during migration check: {}", e)
    })?;

    let name = v1_val
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Example User")
        .to_string();
    let timezone = v1_val
        .get("timezone")
        .and_then(|v| v.as_str())
        .unwrap_or("UTC")
        .to_string();
    let role = v1_val
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let coding_level = v1_val
        .get("preferences")
        .and_then(|p| p.get("coding_level"))
        .and_then(|v| v.as_str())
        .unwrap_or("beginner")
        .to_string();

    let favorite_language = v1_val
        .get("preferences")
        .and_then(|p| p.get("favorite_language"))
        .and_then(|v| v.as_str())
        .unwrap_or("Rust")
        .to_string();

    // Check for V1 goals to migrate
    let mut migrated_goals = Vec::new();
    if let Some(goals_array) = v1_val
        .get("personal_context")
        .and_then(|p| p.get("goals"))
        .and_then(|g| g.as_array())
    {
        for goal in goals_array {
            if let Some(s) = goal.as_str() {
                migrated_goals.push(s.to_string());
            }
        }
    }

    let migrated_profile = Profile {
        schema_version: 2,
        identity: Identity {
            display_name: name,
            full_name: "".to_string(),
            gender: "".to_string(),
            pronouns: "".to_string(),
            date_of_birth: "".to_string(),
            timezone,
        },
        location: Location {
            country: "".to_string(),
            state_or_region: "".to_string(),
            city: "".to_string(),
            living_situation: "".to_string(),
        },
        languages: Languages {
            preferred_human_language: "English".to_string(),
            other_human_languages: vec![],
        },
        technical: Technical {
            coding_level,
            preferred_coding_languages: vec![favorite_language],
            current_os: "".to_string(),
            main_editor: "".to_string(),
            secondary_editor: "".to_string(),
            terminal: "".to_string(),
        },
        personal: Personal {
            occupation_or_role: role,
            education_level: "".to_string(),
            interests: vec![],
        },
        privacy: Privacy {
            send_identity: true,
            send_location: false,
            send_gender: false,
            send_technical: true,
            send_personal: false,
        },
    };

    // Save V2 migrated profile
    save_profile(&migrated_profile)?;

    // Handle V1 goals migration into contexts.toml
    if !migrated_goals.is_empty() {
        if let Ok(mut contexts_file) = crate::context::read_contexts() {
            // Remove existing user_goals context if present to avoid duplicates
            contexts_file.contexts.retain(|c| c.id != "user_goals");

            contexts_file.contexts.push(crate::context::ContextEntry {
                id: "user_goals".to_string(),
                enabled: true,
                kind: "goal".to_string(),
                send_policy: "session_pinned".to_string(),
                title: "Personal Learning and Career Goals".to_string(),
                summary: "The user's direct active learning and career milestone goals."
                    .to_string(),
                triggers: vec![
                    "goal".to_string(),
                    "learn".to_string(),
                    "career".to_string(),
                    "plan".to_string(),
                ],
                required_any: vec![],
                negative_triggers: vec![],
                min_score: 1,
                facts: migrated_goals,
                rules: vec![],
            });

            let _ = crate::context::save_contexts(&contexts_file);
        }
    }

    Ok(migrated_profile)
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn profile_path_uses_isolated_test_config_dir() {
        let _lock = crate::config_paths::TEST_CONFIG_ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        std::env::set_var("OPENNIVARA_TEST_CONFIG_DIR", temp_dir.path());

        let path = get_profile_path().expect("profile path");

        assert_eq!(path, temp_dir.path().join("profile.toml"));
        std::env::remove_var("OPENNIVARA_TEST_CONFIG_DIR");
    }
}
