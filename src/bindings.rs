use specta::{ts, Type};

const BINDINGS_PATH: &str = "desktop/src/generated/backendTypes.ts";

#[derive(Type)]
pub struct ProfileIdentity {
    pub display_name: String,
    pub full_name: Option<String>,
    pub gender: Option<String>,
    pub pronouns: Option<String>,
    pub date_of_birth: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Type)]
pub struct ProfileLocation {
    pub country: Option<String>,
    pub state_or_region: Option<String>,
    pub city: Option<String>,
    pub living_situation: Option<String>,
}

#[derive(Type)]
pub struct ProfileLanguages {
    pub preferred_human_language: String,
    pub other_human_languages: Vec<String>,
}

#[derive(Type)]
pub struct ProfileTechnical {
    pub coding_level: String,
    pub preferred_coding_languages: Vec<String>,
    pub current_os: Option<String>,
    pub main_editor: Option<String>,
    pub secondary_editor: Option<String>,
    pub terminal: Option<String>,
}

#[derive(Type)]
pub struct ProfilePersonal {
    pub occupation_or_role: Option<String>,
    pub education_level: Option<String>,
    pub interests: Vec<String>,
}

#[derive(Type)]
pub struct ProfilePrivacy {
    pub send_identity: bool,
    pub send_location: bool,
    pub send_gender: bool,
    pub send_technical: bool,
    pub send_personal: bool,
}

#[derive(Type)]
pub struct ProfileConfig {
    pub schema_version: u32,
    pub identity: ProfileIdentity,
    pub location: ProfileLocation,
    pub languages: ProfileLanguages,
    pub technical: ProfileTechnical,
    pub personal: ProfilePersonal,
    pub privacy: ProfilePrivacy,
}

#[derive(Type)]
pub struct CommunicationStyleConfig {
    pub tone: String,
    pub detail_level: String,
    pub use_examples: bool,
    pub use_step_by_step: bool,
    pub avoid_unexplained_jargon: bool,
    pub ask_fewer_questions: bool,
    pub prefer_actionable_answers: bool,
}

#[derive(Type)]
pub struct CodingStyleConfig {
    pub show_simple_solution_first: bool,
    pub explain_after_code: bool,
    pub prefer_mvp_architecture: bool,
    pub avoid_overengineering: bool,
    pub use_beginner_comments: bool,
}

#[derive(Type)]
pub struct FormattingStyleConfig {
    pub use_markdown: bool,
    pub use_short_sections: bool,
    pub include_next_step: bool,
    pub avoid_long_walls_of_text: bool,
}

#[derive(Type)]
pub struct BehaviorStyleConfig {
    pub be_honest_about_uncertainty: bool,
    pub do_not_pretend_to_have_done_things: bool,
    pub do_not_reveal_private_context_unless_relevant: bool,
}

#[derive(Type)]
pub struct StyleConfig {
    pub schema_version: u32,
    pub communication: CommunicationStyleConfig,
    pub coding: CodingStyleConfig,
    pub formatting: FormattingStyleConfig,
    pub behavior: BehaviorStyleConfig,
}

#[derive(Type)]
pub struct PreferenceItemConfig {
    pub item: String,
    pub strength: u8,
}

#[derive(Type)]
pub struct PreferenceSectionConfig {
    pub id: String,
    pub enabled: bool,
    pub send_policy: String,
    pub description: Option<String>,
    pub triggers: Vec<String>,
    pub required_any: Vec<String>,
    pub negative_triggers: Vec<String>,
    pub min_score: i32,
    pub likes: Vec<PreferenceItemConfig>,
    pub dislikes: Vec<PreferenceItemConfig>,
    pub notes: Vec<String>,
}

#[derive(Type)]
pub struct PreferencesConfig {
    pub schema_version: u32,
    pub sections: Vec<PreferenceSectionConfig>,
}

#[derive(Type)]
pub struct ContextEntryConfig {
    pub id: String,
    pub enabled: bool,
    pub kind: String,
    pub send_policy: String,
    pub title: String,
    pub summary: String,
    pub triggers: Vec<String>,
    pub required_any: Vec<String>,
    pub negative_triggers: Vec<String>,
    pub min_score: i32,
    pub facts: Vec<String>,
    pub rules: Vec<String>,
}

#[derive(Type)]
pub struct ContextsConfig {
    pub schema_version: u32,
    pub contexts: Vec<ContextEntryConfig>,
}

#[derive(Type)]
pub struct ActiveThemePreview {
    pub id: String,
    pub name: String,
    pub ui_only: bool,
}

#[derive(Type)]
pub struct EffectiveSettingsPreview {
    pub base_preferences: Vec<String>,
    pub base_contexts: Vec<String>,
    pub active_theme: Option<ActiveThemePreview>,
}

#[derive(Type)]
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

#[derive(Type)]
pub struct ThemeEffects {
    pub background_gradient: bool,
    pub glow: String,
    pub density: String,
}

#[derive(Type)]
pub struct OpenNivaraTheme {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub colors: ThemeColors,
    pub effects: ThemeEffects,
}

#[derive(Type)]
pub struct ThemeSafety {
    pub data_only: bool,
    pub contains_executable_code: bool,
    pub modifies_tool_security: bool,
    pub requires_network: bool,
}

#[derive(Type)]
pub struct ThemeManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub source_kind: String,
    pub safety: ThemeSafety,
}

#[derive(Type)]
pub struct InstalledTheme {
    pub id: String,
    pub name: String,
    pub version: String,
    pub source_kind: String,
    pub installed_at: String,
    pub manifest_path: String,
}

#[derive(Type)]
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

#[derive(Type)]
pub struct ThemePreview {
    pub manifest: ThemeManifest,
    pub theme: OpenNivaraTheme,
    pub installed: bool,
    pub applied: bool,
}

#[derive(Type)]
pub struct WorkspaceMapSummary {
    pub root_path: String,
    pub total_files: u32,
    pub total_dirs: u32,
    pub total_blocked: u32,
    pub total_ignored: u32,
}

#[derive(Type)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub updated_at: String,
    pub status: String,
    pub surface_created: String,
    pub actor_id_created: Option<String>,
    pub active: bool,
}

#[derive(Type)]
pub struct ToolSecuritySummary {
    pub enabled: bool,
    pub max_tool_rounds: u32,
    pub allowed_roots: Vec<String>,
    pub blocked_patterns: Vec<String>,
}

pub fn generated_typescript() -> anyhow::Result<String> {
    let conf = Default::default();
    let exports = [
        // Runtime / engine / approval contracts
        ts::export::<crate::engine::Surface>(&conf)?,
        ts::export::<crate::engine::EngineResponseKind>(&conf)?,
        ts::export::<crate::engine::EngineResponse>(&conf)?,
        ts::export::<crate::engine::ApprovalActionResponse>(&conf)?,
        ts::export::<crate::state::views::ApprovalView>(&conf)?,
        ts::export::<crate::state::types::ApprovalStatus>(&conf)?,
        ts::export::<crate::state::types::PendingTurnPhase>(&conf)?,
        ts::export::<crate::tools::ToolPreviewEnvelope>(&conf)?,
        ts::export::<crate::tools::ToolExecutionStatus>(&conf)?,
        ts::export::<crate::tools::ToolOutputTruncation>(&conf)?,
        ts::export::<crate::tools::ToolExecutionResult>(&conf)?,
        ts::export::<crate::tools::ModelVisibleToolError>(&conf)?,
        ts::export::<crate::tools::ModelVisibleToolResult>(&conf)?,
        ts::export::<crate::error::ErrorKind>(&conf)?,
        ts::export::<crate::error::UserFacingError>(&conf)?,
        // Existing profile/style/preferences/context/theme/session contracts
        ts::export::<ProfileIdentity>(&conf)?,
        ts::export::<ProfileLocation>(&conf)?,
        ts::export::<ProfileLanguages>(&conf)?,
        ts::export::<ProfileTechnical>(&conf)?,
        ts::export::<ProfilePersonal>(&conf)?,
        ts::export::<ProfilePrivacy>(&conf)?,
        ts::export::<ProfileConfig>(&conf)?,
        ts::export::<CommunicationStyleConfig>(&conf)?,
        ts::export::<CodingStyleConfig>(&conf)?,
        ts::export::<FormattingStyleConfig>(&conf)?,
        ts::export::<BehaviorStyleConfig>(&conf)?,
        ts::export::<StyleConfig>(&conf)?,
        ts::export::<PreferenceItemConfig>(&conf)?,
        ts::export::<PreferenceSectionConfig>(&conf)?,
        ts::export::<PreferencesConfig>(&conf)?,
        ts::export::<ContextEntryConfig>(&conf)?,
        ts::export::<ContextsConfig>(&conf)?,
        ts::export::<ActiveThemePreview>(&conf)?,
        ts::export::<EffectiveSettingsPreview>(&conf)?,
        ts::export::<ThemeColors>(&conf)?,
        ts::export::<ThemeEffects>(&conf)?,
        ts::export::<OpenNivaraTheme>(&conf)?,
        ts::export::<ThemeSafety>(&conf)?,
        ts::export::<ThemeManifest>(&conf)?,
        ts::export::<InstalledTheme>(&conf)?,
        ts::export::<ThemeStoreItem>(&conf)?,
        ts::export::<ThemePreview>(&conf)?,
        ts::export::<WorkspaceMapSummary>(&conf)?,
        ts::export::<SessionSummary>(&conf)?,
        ts::export::<ToolSecuritySummary>(&conf)?,
        ts::export::<crate::secrets::ApiKeyStatus>(&conf)?,
        ts::export::<crate::first_run::FirstRunInput>(&conf)?,
        ts::export::<crate::first_run::FirstRunStatus>(&conf)?,
        ts::export::<crate::memory::types::MemoryMode>(&conf)?,
        ts::export::<crate::memory::types::MemorySettings>(&conf)?,
        ts::export::<crate::memory::types::EffectivePrivacyPolicy>(&conf)?,
        ts::export::<crate::memory::types::CreateMemorySource>(&conf)?,
        ts::export::<crate::memory::types::MemorySource>(&conf)?,
        ts::export::<crate::memory::types::CreateMemoryItem>(&conf)?,
        ts::export::<crate::memory::types::MemoryItem>(&conf)?,
        ts::export::<crate::memory::types::MemorySearchQuery>(&conf)?,
        ts::export::<crate::memory::types::MemorySearchResult>(&conf)?,
        ts::export::<crate::memory::types::MemoryStatus>(&conf)?,
        ts::export::<crate::memory::types::MemoryExtractionProposal>(&conf)?,
        ts::export::<crate::memory::types::IntentClassification>(&conf)?,
        ts::export::<crate::memory::types::ContextCompilerInput>(&conf)?,
        ts::export::<crate::memory::types::TokenBudgetSection>(&conf)?,
        ts::export::<crate::memory::types::TokenBudgetReport>(&conf)?,
        ts::export::<crate::memory::types::PromptAudit>(&conf)?,
        ts::export::<crate::memory::types::CompilerSelectedSkill>(&conf)?,
        ts::export::<crate::memory::types::CompilerSkillCandidate>(&conf)?,
        ts::export::<crate::memory::types::CompilerActiveTheme>(&conf)?,
        ts::export::<crate::memory::types::ContextCompilerOutput>(&conf)?,
        ts::export::<crate::memory::types::CreateMemoryFacet>(&conf)?,
        ts::export::<crate::memory::types::MemoryFacet>(&conf)?,
        ts::export::<crate::memory::graph::MemoryGraphNode>(&conf)?,
        ts::export::<crate::memory::graph::MemoryGraphEdge>(&conf)?,
        ts::export::<crate::memory::graph::MemoryGraphContext>(&conf)?,
        ts::export::<crate::memory::graph::MemoryGraphStatus>(&conf)?,
        ts::export::<crate::memory::graph::CreateGraphEdge>(&conf)?,
        ts::export::<crate::memory::tasks::MemoryTask>(&conf)?,
        ts::export::<crate::memory::corrections::MemoryCorrection>(&conf)?,
        ts::export::<crate::memory::entities::EntityResolutionResult>(&conf)?,
        ts::export::<crate::memory::embeddings::EmbeddingStatus>(&conf)?,
        ts::export::<crate::memory::maps::MemoryMapSummary>(&conf)?,
        ts::export::<crate::runtime::context::RelativeDateContext>(&conf)?,
        ts::export::<crate::runtime::context::RuntimeContext>(&conf)?,
        ts::export::<crate::runtime::location::LocationContext>(&conf)?,
        ts::export::<crate::runtime::location::CreateSavedPlace>(&conf)?,
        ts::export::<crate::runtime::location::SavedPlace>(&conf)?,
        ts::export::<crate::runtime::location::LocationObservationInput>(&conf)?,
        ts::export::<crate::runtime::model_registry::ModelContextInfo>(&conf)?,
        ts::export::<crate::runtime::model_registry::ModelUsageMetadata>(&conf)?,
    ];

    Ok(format!(
        "// This file is generated by `cargo test bindings_are_current`.\n// Do not edit it by hand.\n\n{}\n",
        exports.join("\n\n")
    ))
}

pub fn write_typescript_bindings() -> anyhow::Result<()> {
    let out = generated_typescript()?;
    let path = std::path::Path::new(BINDINGS_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, out)?;
    Ok(())
}

pub fn bindings_path() -> &'static str {
    BINDINGS_PATH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindings_are_current() {
        let expected = generated_typescript().expect("generate bindings");
        let actual = std::fs::read_to_string(bindings_path()).unwrap_or_default();
        if actual != expected {
            write_typescript_bindings().expect("write bindings");
            panic!(
                "{} was stale and has been regenerated. Re-run this test.",
                bindings_path()
            );
        }
    }

    #[test]
    fn bindings_include_shared_approval_and_engine_contracts() {
        let generated = generated_typescript().expect("generate bindings");

        for expected in [
            "export type EngineResponse",
            "export type EngineResponseKind = \"answer\" | \"approval_required\"",
            "export type ApprovalView",
            "export type ToolPreviewEnvelope",
            "export type UserFacingError",
            "export type ErrorKind",
            "export type ApprovalStatus = \"pending\" | \"denied\" | \"executing\" | \"executed\" | \"failed\" | \"completed\"",
            "export type PendingTurnPhase = \"awaiting_approval\" | \"tool_executed_awaiting_model\" | \"denied_awaiting_model\"",
            "surface_created",
            "actor_id_created",
        ] {
            assert!(generated.contains(expected), "missing {expected}");
        }
        assert!(!generated.contains("source_created"));
    }
}
