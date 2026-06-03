use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryMode {
    Off,
    #[default]
    AskBeforeSaving,
    AutoSaveLowRisk,
    FullLifeJournal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemorySettings {
    pub schema_version: u32,
    pub mode: MemoryMode,
    pub pause_memory: bool,
    pub private_chat: bool,
    pub allow_location_memories: bool,
    pub sensitive_approval_required: bool,
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            mode: MemoryMode::AskBeforeSaving,
            pause_memory: false,
            private_chat: false,
            allow_location_memories: false,
            sensitive_approval_required: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateMemorySource {
    pub source_type: String,
    pub source_ref: Option<String>,
    pub source_text: String,
    pub source_quote: Option<String>,
    pub session_id: Option<String>,
    pub message_id: Option<String>,
    pub observed_at: String,
    pub timezone: String,
    pub sensitivity: String,
    pub privacy_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemorySource {
    pub id: String,
    pub source_type: String,
    pub source_ref: Option<String>,
    pub source_text: String,
    pub source_quote: Option<String>,
    pub session_id: Option<String>,
    pub message_id: Option<String>,
    pub created_at: String,
    pub observed_at: String,
    pub timezone: String,
    pub sensitivity: String,
    pub privacy_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateMemoryItem {
    pub memory_type: String,
    pub title: String,
    pub summary: String,
    pub details_json: String,
    pub status: String,
    pub confidence: f64,
    pub user_verified: bool,
    pub sensitivity: String,
    pub visibility: String,
    pub source_id: String,
    pub observed_at: String,
    pub happened_at: Option<String>,
    pub due_at: Option<String>,
    pub timezone: String,
    pub time_precision: String,
    pub natural_time_phrase: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemoryItem {
    pub id: String,
    pub memory_type: String,
    pub title: String,
    pub summary: String,
    pub details_json: String,
    pub status: String,
    pub confidence: f64,
    pub user_verified: bool,
    pub sensitivity: String,
    pub visibility: String,
    pub source_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub observed_at: String,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub happened_at: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub due_at: Option<String>,
    pub completed_at: Option<String>,
    pub timezone: String,
    pub time_precision: String,
    pub natural_time_phrase: Option<String>,
    pub recurrence_rule: Option<String>,
    pub superseded_by: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemorySearchQuery {
    pub query: Option<String>,
    pub memory_type: Option<String>,
    pub status: Option<String>,
    pub domain: Option<String>,
    pub facet_type: Option<String>,
    pub label: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemorySearchResult {
    pub item: MemoryItem,
    pub score: f64,
    pub reason: String,
    pub answerability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemoryStatus {
    pub db_path: String,
    pub initialized: bool,
    pub schema_version: u32,
    pub item_count: u32,
    pub proposal_count: u32,
    pub vector_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemoryExtractionProposal {
    pub id: String,
    pub source_id: String,
    pub proposal_json: String,
    pub sensitivity: String,
    pub confidence: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct IntentClassification {
    pub labels: Vec<String>,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ContextCompilerInput {
    pub user_message: String,
    pub session_id: Option<String>,
    pub message_id: Option<String>,
    pub runtime_context: crate::runtime::context::RuntimeContext,
    pub model_context_limit: u32,
    pub reserved_output_tokens: u32,
    pub privacy_mode: MemoryMode,
    pub enabled_sources: Vec<String>,
    pub current_workspace_context: Option<String>,
    pub current_route_context: Option<String>,
    pub manual_context_overrides: Vec<String>,
    #[serde(default)]
    pub pinned_context_ids: Vec<String>,
    #[serde(default)]
    pub explicit_skill_id: Option<String>,
    #[serde(default)]
    pub pack_hint: Option<String>,
    #[serde(default)]
    pub ui_selected_skill_id: Option<String>,
    #[serde(default)]
    pub session_pinned_skill_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TokenBudgetSection {
    pub section: String,
    pub priority: u32,
    pub estimated_tokens: u32,
    pub included: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TokenBudgetReport {
    pub model_context_limit: u32,
    pub reserved_output_tokens: u32,
    pub input_budget_tokens: u32,
    pub estimated_prompt_tokens: u32,
    pub trimmed_sections: Vec<String>,
    pub sections: Vec<TokenBudgetSection>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PromptAudit {
    pub id: String,
    pub session_id: Option<String>,
    pub message_id: Option<String>,
    pub user_message: String,
    pub compiled_context_json: String,
    pub included_memory_ids_json: String,
    pub included_task_ids_json: String,
    pub included_workspace_refs_json: String,
    pub token_budget_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CompilerSelectedSkill {
    pub id: String,
    pub pack_id: Option<String>,
    pub name: String,
    pub score: u32,
    pub reason: String,
    pub allowed_tools: Vec<String>,
    pub denied_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CompilerSkillCandidate {
    pub id: String,
    pub name: String,
    pub score: u32,
    pub accepted: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CompilerActiveTheme {
    pub id: String,
    pub name: String,
    pub ui_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ContextCompilerOutput {
    pub system_policy: String,
    pub current_user_message: String,
    pub recent_conversation_window: String,
    pub session_summary: String,
    pub profile_brief: String,
    pub style_brief: String,
    pub preference_brief: String,
    pub memory_brief: String,
    pub task_reminder_brief: String,
    pub workspace_brief: String,
    pub route_brief: String,
    pub raw_prompt: String,
    pub token_budget_report: TokenBudgetReport,
    pub audit: PromptAudit,
    pub intent: IntentClassification,
    pub included_memory_ids: Vec<String>,
    pub included_graph_edge_ids: Vec<String>,
    pub runtime_decision: String,
    pub location_decision: String,
    pub profile_sent: Vec<String>,
    pub style_sent: Vec<String>,
    pub preferences_sent: Vec<String>,
    pub contexts_sent: Vec<String>,
    pub contexts_pinned: Vec<String>,
    pub contexts_not_sent: Vec<String>,
    pub selected_skills: Vec<CompilerSelectedSkill>,
    pub skill_candidates: Vec<CompilerSkillCandidate>,
    pub skill_warnings: Vec<String>,
    pub warnings: Vec<String>,
    pub active_mode: String,
    pub active_packs: Vec<String>,
    pub active_theme: Option<CompilerActiveTheme>,
    pub style_source_pack: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateMemoryFacet {
    pub memory_id: String,
    pub domain: String,
    pub facet_type: String,
    pub label: String,
    pub details_json: String,
    pub sensitivity: String,
    pub confidence: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemoryFacet {
    pub id: String,
    pub memory_id: String,
    pub domain: String,
    pub facet_type: String,
    pub label: String,
    pub details_json: String,
    pub sensitivity: String,
    pub confidence: f64,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}
