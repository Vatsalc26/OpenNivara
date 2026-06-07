use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Surface {
    Desktop,
    Cli,
    Telegram,
}

impl Surface {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Cli => "cli",
            Self::Telegram => "telegram",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    Tool,
    Event,
    System,
}

impl MessageRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
            Self::Event => "event",
            Self::System => "system",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[specta(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Denied,
    Executing,
    Executed,
    Failed,
    Completed,
}

impl ApprovalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Denied => "denied",
            Self::Executing => "executing",
            Self::Executed => "executed",
            Self::Failed => "failed",
            Self::Completed => "completed",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "denied" => Some(Self::Denied),
            "executing" => Some(Self::Executing),
            "executed" => Some(Self::Executed),
            "failed" => Some(Self::Failed),
            "completed" => Some(Self::Completed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[specta(rename_all = "snake_case")]
pub enum PendingTurnPhase {
    AwaitingApproval,
    ToolExecutedAwaitingModel,
    DeniedAwaitingModel,
}

impl PendingTurnPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AwaitingApproval => "awaiting_approval",
            Self::ToolExecutedAwaitingModel => "tool_executed_awaiting_model",
            Self::DeniedAwaitingModel => "denied_awaiting_model",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "awaiting_approval" => Some(Self::AwaitingApproval),
            "tool_executed_awaiting_model" => Some(Self::ToolExecutedAwaitingModel),
            "denied_awaiting_model" => Some(Self::DeniedAwaitingModel),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub id: String,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub status: String,
    pub surface_created: String,
    pub actor_id_created: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSessionInput {
    pub title: Option<String>,
    pub surface_created: Surface,
    pub actor_id_created: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DbMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub surface: String,
    pub actor_id: Option<String>,
    pub content: String,
    pub created_at: String,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreMessageInput {
    pub session_id: String,
    pub role: MessageRole,
    pub surface: Surface,
    pub actor_id: Option<String>,
    pub content: String,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreEventMessageInput {
    pub session_id: String,
    pub surface: Surface,
    pub actor_id: Option<String>,
    pub event_json: String,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PendingTurnState {
    pub request_id: String,
    pub turn_id: String,
    pub request_envelope: serde_json::Value,
    pub session_id: String,
    pub user_message_id: String,
    pub model_messages_so_far: serde_json::Value,
    pub declared_tools: serde_json::Value,
    pub pending_tool_call: serde_json::Value,
    pub compiled_context_audit_id: Option<String>,
    pub selected_skill_ids: Vec<String>,
    pub pinned_context_ids: Vec<String>,
    pub provider_id: String,
    pub model_id: String,
    pub generation_config: serde_json::Value,
    pub provider_state_json: serde_json::Value,
    pub current_round: u32,
    pub max_rounds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePendingApprovalInput {
    pub session_id: String,
    pub request_id: String,
    pub turn_id: String,
    pub user_message_id: String,
    pub tool_call_id: String,
    pub surface: Surface,
    pub actor_id: String,
    pub operation_name: String,
    pub classification: String,
    pub summary: Option<String>,
    pub operation_target: Option<String>,
    pub reason: Option<String>,
    pub arguments_preview_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PendingApproval {
    pub id: String,
    pub session_id: String,
    pub request_id: String,
    pub turn_id: String,
    pub user_message_id: String,
    pub tool_call_id: String,
    pub surface: String,
    pub actor_id: String,
    pub operation_name: String,
    pub classification: String,
    pub status: String,
    pub summary: Option<String>,
    pub operation_target: Option<String>,
    pub reason: Option<String>,
    pub arguments_preview_json: Option<String>,
    pub result_summary: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
    pub resolved_by_actor_id: Option<String>,
    pub execution_started_at: Option<String>,
    pub execution_finished_at: Option<String>,
    pub completed_at: Option<String>,
    pub resume_attempt_count: i64,
    pub last_resume_error: Option<String>,
    pub last_resume_attempt_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BeginExecutionResult {
    Started {
        approval: Box<PendingApproval>,
        turn: Box<PendingTurnState>,
    },
    NotFound,
    WrongSession,
    ActorNotAllowed,
    AlreadyDenied,
    AlreadyExecuting,
    AlreadyExecuted,
    AlreadyFailed,
    AlreadyCompleted,
    MissingPendingTurn,
    InvalidPhase,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DenyApprovalInput {
    pub approval_id: String,
    pub session_id: String,
    pub actor_id: String,
    pub denied_turn: PendingTurnState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DenyApprovalResult {
    pub approval: PendingApproval,
    pub turn: PendingTurnState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkToolExecutedInput {
    pub approval_id: String,
    pub updated_turn: PendingTurnState,
    pub result_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkToolFailedInput {
    pub approval_id: String,
    pub error_message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_type_values_match_schema_values() {
        assert_eq!(Surface::Cli.as_str(), "cli");
        assert_eq!(MessageRole::Assistant.as_str(), "assistant");
        assert_eq!(ApprovalStatus::Completed.as_str(), "completed");
        assert_eq!(
            PendingTurnPhase::ToolExecutedAwaitingModel.as_str(),
            "tool_executed_awaiting_model"
        );
    }
}
