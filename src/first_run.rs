use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FirstRunInput {
    pub accepted_alpha_notice: bool,
    #[serde(default)]
    pub gemini_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FirstRunStatus {
    pub is_first_run: bool,
    pub required_state_ready: bool,
    pub profile_exists: bool,
    pub style_exists: bool,
    pub preferences_exists: bool,
    pub contexts_exists: bool,
    pub tools_exists: bool,
    pub memory_ready: bool,
    pub marketplace_ready: bool,
    pub skills_ready: bool,
    pub gemini_key: crate::secrets::ApiKeyStatus,
}

pub fn first_run_status() -> anyhow::Result<FirstRunStatus> {
    let profile_exists = crate::profile::get_profile_path()?.exists();
    let style_exists = crate::style::get_style_path()?.exists();
    let preferences_exists = crate::preferences::get_preferences_path()?.exists();
    let contexts_exists = crate::context::get_contexts_path()?.exists();
    let tools_exists = crate::tools::get_tools_path()?.exists();
    let memory_ready = crate::memory::db::memory_db_path()?.exists();
    let marketplace_ready = crate::marketplace::get_installed_packs_path()?.exists();
    let skills_ready = crate::skills::registry::get_enabled_skills_path()?.exists()
        && crate::skills::registry::get_user_skills_path()?.exists();
    let required_state_ready = profile_exists
        && style_exists
        && preferences_exists
        && contexts_exists
        && tools_exists
        && memory_ready
        && marketplace_ready
        && skills_ready;

    Ok(FirstRunStatus {
        is_first_run: !required_state_ready,
        required_state_ready,
        profile_exists,
        style_exists,
        preferences_exists,
        contexts_exists,
        tools_exists,
        memory_ready,
        marketplace_ready,
        skills_ready,
        gemini_key: crate::secrets::gemini_key_status()?,
    })
}

pub fn initialize_clean_first_run(input: FirstRunInput) -> anyhow::Result<FirstRunStatus> {
    if !input.accepted_alpha_notice {
        return Err(anyhow::anyhow!(
            "Alpha privacy notice must be accepted before first-run setup."
        ));
    }

    if let Some(key) = input
        .gemini_api_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
    {
        crate::secrets::save_gemini_key(key)?;
    }

    crate::profile::init_profile()?;
    crate::style::init_style()?;
    crate::preferences::init_preferences()?;
    crate::context::init_contexts()?;
    crate::tools::init_tools()?;
    crate::memory::db::status()?;
    crate::skills::registry::init_skills()?;
    crate::marketplace::init_marketplace()?;

    first_run_status()
}
