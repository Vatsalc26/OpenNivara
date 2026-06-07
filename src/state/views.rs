use crate::state::types::{ApprovalStatus, PendingTurnPhase};
use crate::tools::ToolPreviewEnvelope;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ApprovalView {
    pub approval_id: String,
    pub session_id: String,
    pub request_id: String,
    pub turn_id: String,
    pub status: ApprovalStatus,
    pub phase: Option<PendingTurnPhase>,
    pub operation_name: String,
    pub classification: String,
    pub summary: String,
    pub operation_target: Option<String>,
    pub reason: String,
    pub preview: ToolPreviewEnvelope,
    pub full_arguments_json: Option<serde_json::Value>,
    pub can_approve: bool,
    pub can_deny: bool,
    pub can_resume_continuation: bool,
    pub can_retry_tool_execution: bool,
    pub result_summary: Option<String>,
    pub error_message: Option<String>,
    pub last_resume_error: Option<String>,
    pub resume_attempt_count: u32,
}

pub fn get_approval_view(
    conn: &rusqlite::Connection,
    approval_id: &str,
) -> anyhow::Result<Option<ApprovalView>> {
    let Some(approval) = crate::state::approvals::get_pending_approval(conn, approval_id)? else {
        return Ok(None);
    };
    let turn = crate::state::approvals::get_pending_turn_state(conn, approval_id)?;
    let phase = crate::state::approvals::get_pending_turn_phase(conn, approval_id)?;
    Ok(Some(approval_view_from_parts(approval, turn, phase)))
}

pub fn list_approval_views_for_session(
    conn: &rusqlite::Connection,
    session_id: &str,
) -> anyhow::Result<Vec<ApprovalView>> {
    crate::state::approvals::list_pending_approvals_for_session(conn, session_id)?
        .into_iter()
        .map(|approval| {
            let turn = crate::state::approvals::get_pending_turn_state(conn, &approval.id)?;
            let phase = crate::state::approvals::get_pending_turn_phase(conn, &approval.id)?;
            Ok(approval_view_from_parts(approval, turn, phase))
        })
        .collect()
}

fn approval_view_from_parts(
    approval: crate::state::types::PendingApproval,
    turn: Option<crate::state::types::PendingTurnState>,
    phase: Option<PendingTurnPhase>,
) -> ApprovalView {
    let preview_details = approval
        .arguments_preview_json
        .as_deref()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or(serde_json::Value::Null);
    let full_arguments_json = turn
        .map(|turn| turn.pending_tool_call)
        .filter(|value| !value.is_null());
    let status = ApprovalStatus::from_db_value(&approval.status).unwrap_or(ApprovalStatus::Failed);
    let can_approve = status == ApprovalStatus::Pending;
    let can_deny = status == ApprovalStatus::Pending;
    let can_resume_continuation =
        matches!(status, ApprovalStatus::Denied | ApprovalStatus::Executed);
    let summary = approval
        .summary
        .unwrap_or_else(|| format!("{} requires approval.", approval.operation_name));
    let preview = ToolPreviewEnvelope {
        schema_version: 1,
        tool_name: approval.operation_name.clone(),
        preview_kind: "generic".to_string(),
        operation_target: approval.operation_target.clone(),
        summary: summary.clone(),
        details: preview_details,
    };

    ApprovalView {
        approval_id: approval.id,
        session_id: approval.session_id,
        request_id: approval.request_id,
        turn_id: approval.turn_id,
        status,
        phase,
        operation_name: approval.operation_name,
        classification: approval.classification,
        summary,
        operation_target: approval.operation_target,
        reason: approval.reason.unwrap_or_default(),
        preview,
        full_arguments_json,
        can_approve,
        can_deny,
        can_resume_continuation,
        can_retry_tool_execution: false,
        result_summary: approval.result_summary,
        error_message: approval.error_message,
        last_resume_error: approval.last_resume_error,
        resume_attempt_count: approval.resume_attempt_count.try_into().unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use crate::state::approvals::create_pending_approval_with_turn;
    use crate::state::db::open_state_db_at;
    use crate::state::messages::{store_message, StoreMessageInput};
    use crate::state::sessions::{create_session, CreateSessionInput};
    use crate::state::types::{CreatePendingApprovalInput, MessageRole, PendingTurnState, Surface};
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn approval_view_uses_preview_and_pending_turn_arguments() {
        let dir = tempdir().unwrap();
        let mut conn = open_state_db_at(dir.path().join("state.sqlite")).unwrap();
        let session = create_session(
            &conn,
            CreateSessionInput {
                title: None,
                surface_created: Surface::Cli,
                actor_id_created: Some("cli_owner".into()),
            },
        )
        .unwrap();
        let message = store_message(
            &conn,
            StoreMessageInput {
                session_id: session.id.clone(),
                role: MessageRole::User,
                surface: Surface::Cli,
                actor_id: Some("cli_owner".into()),
                content: "write".into(),
                metadata_json: None,
            },
        )
        .unwrap();
        let turn = PendingTurnState {
            request_id: "req_view".into(),
            turn_id: "turn_view".into(),
            request_envelope: json!({}),
            session_id: session.id.clone(),
            user_message_id: message.id.clone(),
            model_messages_so_far: json!([]),
            declared_tools: json!([]),
            pending_tool_call: json!({"arguments":{"path":"notes.txt","content":"hello"}}),
            compiled_context_audit_id: None,
            selected_skill_ids: vec![],
            pinned_context_ids: vec![],
            provider_id: "mock".into(),
            model_id: "mock-model".into(),
            generation_config: json!({}),
            provider_state_json: json!({}),
            current_round: 1,
            max_rounds: 4,
        };
        let approval = create_pending_approval_with_turn(
            &mut conn,
            CreatePendingApprovalInput {
                session_id: session.id.clone(),
                request_id: "req_view".into(),
                turn_id: "turn_view".into(),
                user_message_id: message.id,
                tool_call_id: "toolcall_view".into(),
                surface: Surface::Cli,
                actor_id: "cli_owner".into(),
                operation_name: "write_file".into(),
                classification: "local_modify".into(),
                summary: Some("Write notes".into()),
                operation_target: Some("notes.txt".into()),
                reason: Some("Local write".into()),
                arguments_preview_json: Some(r#"{"path":"notes.txt"}"#.into()),
            },
            turn,
        )
        .unwrap();

        let view = super::get_approval_view(&conn, &approval.id)
            .unwrap()
            .unwrap();

        assert_eq!(view.approval_id, approval.id);
        assert_eq!(view.preview.details["path"], "notes.txt");
        assert_eq!(
            view.full_arguments_json.unwrap()["arguments"]["content"],
            "hello"
        );
        assert!(view.can_approve);
        assert!(view.can_deny);
        assert!(!view.can_resume_continuation);
        assert!(!view.can_retry_tool_execution);
    }
}
