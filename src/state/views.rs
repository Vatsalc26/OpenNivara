#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ApprovalView {
    pub approval_id: String,
    pub session_id: String,
    pub operation_name: String,
    pub classification: String,
    pub status: String,
    pub summary: Option<String>,
    pub operation_target: Option<String>,
    pub reason: Option<String>,
    pub preview_json: serde_json::Value,
    pub full_arguments_json: serde_json::Value,
    pub can_approve: bool,
    pub can_deny: bool,
    pub can_continue: bool,
}

pub fn get_approval_view(
    conn: &rusqlite::Connection,
    approval_id: &str,
) -> anyhow::Result<Option<ApprovalView>> {
    let Some(approval) = crate::state::approvals::get_pending_approval(conn, approval_id)? else {
        return Ok(None);
    };
    let turn = crate::state::approvals::get_pending_turn_state(conn, approval_id)?;
    Ok(Some(approval_view_from_parts(approval, turn)))
}

pub fn list_approval_views_for_session(
    conn: &rusqlite::Connection,
    session_id: &str,
) -> anyhow::Result<Vec<ApprovalView>> {
    crate::state::approvals::list_pending_approvals_for_session(conn, session_id)?
        .into_iter()
        .map(|approval| {
            let turn = crate::state::approvals::get_pending_turn_state(conn, &approval.id)?;
            Ok(approval_view_from_parts(approval, turn))
        })
        .collect()
}

fn approval_view_from_parts(
    approval: crate::state::types::PendingApproval,
    turn: Option<crate::state::types::PendingTurnState>,
) -> ApprovalView {
    let preview_json = approval
        .arguments_preview_json
        .as_deref()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or(serde_json::Value::Null);
    let full_arguments_json = turn
        .map(|turn| turn.pending_tool_call)
        .unwrap_or(serde_json::Value::Null);
    let can_approve = approval.status == "pending";
    let can_deny = approval.status == "pending";
    let can_continue = matches!(approval.status.as_str(), "denied" | "executed");

    ApprovalView {
        approval_id: approval.id,
        session_id: approval.session_id,
        operation_name: approval.operation_name,
        classification: approval.classification,
        status: approval.status,
        summary: approval.summary,
        operation_target: approval.operation_target,
        reason: approval.reason,
        preview_json,
        full_arguments_json,
        can_approve,
        can_deny,
        can_continue,
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
        assert_eq!(view.preview_json["path"], "notes.txt");
        assert_eq!(view.full_arguments_json["arguments"]["content"], "hello");
        assert!(view.can_approve);
        assert!(view.can_deny);
        assert!(!view.can_continue);
    }
}
