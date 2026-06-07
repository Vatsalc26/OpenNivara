use crate::state::types::{
    ApprovalStatus, BeginExecutionResult, CreatePendingApprovalInput, DenyApprovalInput,
    DenyApprovalResult, MarkToolExecutedInput, MarkToolFailedInput, PendingApproval,
    PendingTurnPhase, PendingTurnState,
};
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension, Transaction};

const INTERRUPTED_EXECUTION_MESSAGE: &str =
    "Execution was interrupted before completion could be confirmed.";

pub fn actor_can_approve(actor_id: &str) -> bool {
    actor_id == "desktop_owner" || actor_id == "cli_owner" || actor_id.starts_with("telegram_")
}

pub fn create_pending_approval_with_turn(
    conn: &mut Connection,
    input: CreatePendingApprovalInput,
    turn: PendingTurnState,
) -> anyhow::Result<PendingApproval> {
    let approval_id = crate::runtime::ids::new_approval_id();
    let now = Utc::now().to_rfc3339();
    let turn_json = serde_json::to_string(&turn)?;

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO pending_approvals (
            id, session_id, request_id, turn_id, user_message_id, tool_call_id,
            surface, actor_id, operation_name, classification, status, summary,
            operation_target, reason, arguments_preview_json, created_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'pending', ?11, ?12, ?13, ?14, ?15
        )",
        params![
            approval_id,
            input.session_id,
            input.request_id,
            input.turn_id,
            input.user_message_id,
            input.tool_call_id,
            input.surface.as_str(),
            input.actor_id,
            input.operation_name,
            input.classification,
            input.summary,
            input.operation_target,
            input.reason,
            input.arguments_preview_json,
            now
        ],
    )?;
    tx.execute(
        "INSERT INTO pending_turns (
            approval_id, session_id, request_id, turn_id, user_message_id,
            provider_id, model_id, phase, resume_payload_json, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'awaiting_approval', ?8, ?9, ?10)",
        params![
            approval_id,
            turn.session_id,
            turn.request_id,
            turn.turn_id,
            turn.user_message_id,
            turn.provider_id,
            turn.model_id,
            turn_json,
            now,
            now
        ],
    )?;
    insert_event_message(
        &tx,
        &input.session_id,
        input.surface.as_str(),
        Some(&input.actor_id),
        "approval_required",
        &approval_id,
        input.summary.as_deref(),
    )?;
    tx.commit()?;

    load_approval(conn, &approval_id)?.ok_or_else(|| anyhow::anyhow!("created approval not found"))
}

pub fn begin_execution_once(
    conn: &mut Connection,
    approval_id: &str,
    session_id: &str,
    approving_actor_id: &str,
) -> anyhow::Result<BeginExecutionResult> {
    let Some(approval) = load_approval(conn, approval_id)? else {
        return Ok(BeginExecutionResult::NotFound);
    };
    if approval.session_id != session_id {
        return Ok(BeginExecutionResult::WrongSession);
    }
    if !actor_can_approve(approving_actor_id) {
        return Ok(BeginExecutionResult::ActorNotAllowed);
    }
    if approval.status != ApprovalStatus::Pending.as_str() {
        return Ok(begin_result_for_status(&approval.status));
    }

    let Some((phase, turn)) = load_pending_turn(conn, approval_id)? else {
        return Ok(BeginExecutionResult::MissingPendingTurn);
    };
    if phase != PendingTurnPhase::AwaitingApproval.as_str() {
        return Ok(BeginExecutionResult::InvalidPhase);
    }

    let changed = conn.execute(
        "UPDATE pending_approvals
         SET status = 'executing', execution_started_at = ?1
         WHERE id = ?2 AND session_id = ?3 AND status = 'pending'",
        params![Utc::now().to_rfc3339(), approval_id, session_id],
    )?;
    if changed != 1 {
        let current = load_approval(conn, approval_id)?
            .map(|approval| begin_result_for_status(&approval.status))
            .unwrap_or(BeginExecutionResult::NotFound);
        return Ok(current);
    }

    let approval = load_approval(conn, approval_id)?
        .ok_or_else(|| anyhow::anyhow!("approval disappeared after execution transition"))?;
    Ok(BeginExecutionResult::Started {
        approval: Box::new(approval),
        turn: Box::new(turn),
    })
}

pub fn deny_approval_and_update_turn(
    conn: &mut Connection,
    input: DenyApprovalInput,
) -> anyhow::Result<DenyApprovalResult> {
    if !actor_can_approve(&input.actor_id) {
        anyhow::bail!("actor cannot approve or deny operations");
    }

    let tx = conn.transaction()?;
    let (session_id, surface, status, phase): (String, String, String, String) = tx.query_row(
        "SELECT pending_approvals.session_id, pending_approvals.surface,
                pending_approvals.status, pending_turns.phase
         FROM pending_approvals
         JOIN pending_turns ON pending_turns.approval_id = pending_approvals.id
         WHERE pending_approvals.id = ?1",
        [&input.approval_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?;
    if session_id != input.session_id {
        anyhow::bail!("approval belongs to a different session");
    }
    if status != ApprovalStatus::Pending.as_str()
        || phase != PendingTurnPhase::AwaitingApproval.as_str()
    {
        anyhow::bail!("approval cannot be denied from status {status} and phase {phase}");
    }

    let now = Utc::now().to_rfc3339();
    let turn_json = serde_json::to_string(&input.denied_turn)?;
    tx.execute(
        "UPDATE pending_approvals
         SET status = 'denied', resolved_at = ?1, resolved_by_actor_id = ?2
         WHERE id = ?3 AND status = 'pending'",
        params![now, input.actor_id, input.approval_id],
    )?;
    tx.execute(
        "UPDATE pending_turns
         SET phase = 'denied_awaiting_model', resume_payload_json = ?1, updated_at = ?2
         WHERE approval_id = ?3 AND phase = 'awaiting_approval'",
        params![turn_json, now, input.approval_id],
    )?;
    insert_event_message(
        &tx,
        &session_id,
        &surface,
        Some(&input.actor_id),
        "approval_denied",
        &input.approval_id,
        None,
    )?;
    tx.commit()?;

    let approval = load_approval(conn, &input.approval_id)?
        .ok_or_else(|| anyhow::anyhow!("denied approval not found"))?;
    let (_, turn) = load_pending_turn(conn, &input.approval_id)?
        .ok_or_else(|| anyhow::anyhow!("denied pending turn not found"))?;
    Ok(DenyApprovalResult { approval, turn })
}

pub fn mark_tool_executed_and_update_turn(
    conn: &mut Connection,
    input: MarkToolExecutedInput,
) -> anyhow::Result<PendingTurnState> {
    let tx = conn.transaction()?;
    let (session_id, surface, actor_id): (String, String, String) = tx.query_row(
        "SELECT session_id, surface, actor_id FROM pending_approvals WHERE id = ?1",
        [&input.approval_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    let now = Utc::now().to_rfc3339();
    let changed_approval = tx.execute(
        "UPDATE pending_approvals
         SET status = 'executed', result_summary = ?1, execution_finished_at = ?2
         WHERE id = ?3 AND status = 'executing'",
        params![input.result_summary, now, input.approval_id],
    )?;
    let turn_json = serde_json::to_string(&input.updated_turn)?;
    let changed_turn = tx.execute(
        "UPDATE pending_turns
         SET phase = 'tool_executed_awaiting_model', resume_payload_json = ?1, updated_at = ?2
         WHERE approval_id = ?3 AND phase = 'awaiting_approval'",
        params![turn_json, now, input.approval_id],
    )?;
    if changed_approval != 1 || changed_turn != 1 {
        anyhow::bail!("approval was not in executing/awaiting state");
    }

    insert_event_message(
        &tx,
        &session_id,
        &surface,
        Some(&actor_id),
        "approval_executed",
        &input.approval_id,
        None,
    )?;
    tx.commit()?;
    Ok(input.updated_turn)
}

pub fn mark_tool_failed(conn: &mut Connection, input: MarkToolFailedInput) -> anyhow::Result<()> {
    let tx = conn.transaction()?;
    let (session_id, surface, actor_id): (String, String, String) = tx.query_row(
        "SELECT session_id, surface, actor_id FROM pending_approvals WHERE id = ?1",
        [&input.approval_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let changed = tx.execute(
        "UPDATE pending_approvals
         SET status = 'failed', error_message = ?1, execution_finished_at = ?2
         WHERE id = ?3 AND status = 'executing'",
        params![
            input.error_message,
            Utc::now().to_rfc3339(),
            input.approval_id
        ],
    )?;
    if changed != 1 {
        anyhow::bail!("approval is not executing");
    }
    insert_event_message(
        &tx,
        &session_id,
        &surface,
        Some(&actor_id),
        "approval_failed",
        &input.approval_id,
        None,
    )?;
    tx.commit()?;
    Ok(())
}

pub fn mark_resume_failed(
    conn: &Connection,
    approval_id: &str,
    error_message: &str,
) -> anyhow::Result<()> {
    let changed = conn.execute(
        "UPDATE pending_approvals
         SET resume_attempt_count = resume_attempt_count + 1,
             last_resume_error = ?1,
             last_resume_attempt_at = ?2
         WHERE id = ?3 AND status IN ('executed', 'denied')",
        params![error_message, Utc::now().to_rfc3339(), approval_id],
    )?;
    if changed != 1 {
        anyhow::bail!("approval is not resumable");
    }
    Ok(())
}

pub fn mark_approval_completed(
    conn: &mut Connection,
    approval_id: &str,
    _final_assistant_message_id: &str,
) -> anyhow::Result<()> {
    let tx = conn.transaction()?;
    let changed = tx.execute(
        "UPDATE pending_approvals
         SET status = 'completed', completed_at = ?1
         WHERE id = ?2 AND status = 'executed'",
        params![Utc::now().to_rfc3339(), approval_id],
    )?;
    if changed != 1 {
        anyhow::bail!("approval is not executed");
    }
    tx.execute(
        "DELETE FROM pending_turns WHERE approval_id = ?1",
        [approval_id],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn complete_denied_turn(
    conn: &mut Connection,
    approval_id: &str,
    _final_assistant_message_id: &str,
) -> anyhow::Result<()> {
    let tx = conn.transaction()?;
    let status: Option<String> = tx
        .query_row(
            "SELECT status FROM pending_approvals WHERE id = ?1",
            [approval_id],
            |row| row.get(0),
        )
        .optional()?;
    if status.as_deref() != Some(ApprovalStatus::Denied.as_str()) {
        anyhow::bail!("approval is not denied");
    }
    tx.execute(
        "DELETE FROM pending_turns WHERE approval_id = ?1",
        [approval_id],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn delete_pending_turn(conn: &Connection, approval_id: &str) -> anyhow::Result<()> {
    conn.execute(
        "DELETE FROM pending_turns WHERE approval_id = ?1",
        [approval_id],
    )?;
    Ok(())
}

pub fn get_pending_approval(
    conn: &Connection,
    approval_id: &str,
) -> anyhow::Result<Option<PendingApproval>> {
    load_approval(conn, approval_id)
}

pub fn get_pending_turn_state(
    conn: &Connection,
    approval_id: &str,
) -> anyhow::Result<Option<PendingTurnState>> {
    Ok(load_pending_turn(conn, approval_id)?.map(|(_, turn)| turn))
}

pub fn get_pending_turn_phase(
    conn: &Connection,
    approval_id: &str,
) -> anyhow::Result<Option<PendingTurnPhase>> {
    Ok(load_pending_turn(conn, approval_id)?
        .and_then(|(phase, _)| PendingTurnPhase::from_db_value(&phase)))
}

pub fn list_pending_approvals_for_session(
    conn: &Connection,
    session_id: &str,
) -> anyhow::Result<Vec<PendingApproval>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, request_id, turn_id, user_message_id, tool_call_id,
                surface, actor_id, operation_name, classification, status, summary,
                operation_target, reason, arguments_preview_json, result_summary,
                error_message, created_at, resolved_at, resolved_by_actor_id,
                execution_started_at, execution_finished_at, completed_at,
                resume_attempt_count, last_resume_error, last_resume_attempt_at
         FROM pending_approvals
         WHERE session_id = ?1 AND status != 'completed'
         ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([session_id], approval_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn cleanup_completed_pending_turns(conn: &Connection) -> anyhow::Result<usize> {
    conn.execute(
        "DELETE FROM pending_turns
         WHERE approval_id IN (
             SELECT id FROM pending_approvals WHERE status = 'completed'
         )",
        [],
    )
    .map_err(Into::into)
}

pub fn recover_stale_executing_approvals(
    conn: &mut Connection,
    stale_after: Duration,
) -> anyhow::Result<usize> {
    recover_stale_executing_approvals_at(conn, Utc::now(), stale_after)
}

pub fn recover_stale_executing_approvals_at(
    conn: &mut Connection,
    now: chrono::DateTime<Utc>,
    stale_after: Duration,
) -> anyhow::Result<usize> {
    let cutoff = (now - stale_after).to_rfc3339();
    conn.execute(
        "UPDATE pending_approvals
         SET status = 'failed',
             error_message = ?1,
             execution_finished_at = ?2
         WHERE status = 'executing'
           AND execution_started_at IS NOT NULL
           AND execution_started_at <= ?3",
        params![INTERRUPTED_EXECUTION_MESSAGE, now.to_rfc3339(), cutoff],
    )
    .map_err(Into::into)
}

fn begin_result_for_status(status: &str) -> BeginExecutionResult {
    match status {
        "denied" => BeginExecutionResult::AlreadyDenied,
        "executing" => BeginExecutionResult::AlreadyExecuting,
        "executed" => BeginExecutionResult::AlreadyExecuted,
        "failed" => BeginExecutionResult::AlreadyFailed,
        "completed" => BeginExecutionResult::AlreadyCompleted,
        _ => BeginExecutionResult::NotFound,
    }
}

fn load_pending_turn(
    conn: &Connection,
    approval_id: &str,
) -> anyhow::Result<Option<(String, PendingTurnState)>> {
    conn.query_row(
        "SELECT phase, resume_payload_json FROM pending_turns WHERE approval_id = ?1",
        [approval_id],
        |row| {
            let phase: String = row.get(0)?;
            let payload: String = row.get(1)?;
            Ok((phase, payload))
        },
    )
    .optional()?
    .map(|(phase, payload)| {
        let turn = serde_json::from_str(&payload)?;
        Ok((phase, turn))
    })
    .transpose()
}

fn load_approval(conn: &Connection, approval_id: &str) -> anyhow::Result<Option<PendingApproval>> {
    conn.query_row(
        "SELECT id, session_id, request_id, turn_id, user_message_id, tool_call_id,
                surface, actor_id, operation_name, classification, status, summary,
                operation_target, reason, arguments_preview_json, result_summary,
                error_message, created_at, resolved_at, resolved_by_actor_id,
                execution_started_at, execution_finished_at, completed_at,
                resume_attempt_count, last_resume_error, last_resume_attempt_at
         FROM pending_approvals
         WHERE id = ?1",
        [approval_id],
        approval_from_row,
    )
    .optional()
    .map_err(Into::into)
}

fn approval_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PendingApproval> {
    Ok(PendingApproval {
        id: row.get(0)?,
        session_id: row.get(1)?,
        request_id: row.get(2)?,
        turn_id: row.get(3)?,
        user_message_id: row.get(4)?,
        tool_call_id: row.get(5)?,
        surface: row.get(6)?,
        actor_id: row.get(7)?,
        operation_name: row.get(8)?,
        classification: row.get(9)?,
        status: row.get(10)?,
        summary: row.get(11)?,
        operation_target: row.get(12)?,
        reason: row.get(13)?,
        arguments_preview_json: row.get(14)?,
        result_summary: row.get(15)?,
        error_message: row.get(16)?,
        created_at: row.get(17)?,
        resolved_at: row.get(18)?,
        resolved_by_actor_id: row.get(19)?,
        execution_started_at: row.get(20)?,
        execution_finished_at: row.get(21)?,
        completed_at: row.get(22)?,
        resume_attempt_count: row.get(23)?,
        last_resume_error: row.get(24)?,
        last_resume_attempt_at: row.get(25)?,
    })
}

fn insert_event_message(
    tx: &Transaction<'_>,
    session_id: &str,
    surface: &str,
    actor_id: Option<&str>,
    event_type: &str,
    approval_id: &str,
    summary: Option<&str>,
) -> anyhow::Result<()> {
    let content = serde_json::to_string(&serde_json::json!({
        "event_type": event_type,
        "approval_id": approval_id,
        "summary": summary,
    }))?;
    tx.execute(
        "INSERT INTO messages (
            id, session_id, role, surface, actor_id, content, created_at, metadata_json
        ) VALUES (?1, ?2, 'event', ?3, ?4, ?5, ?6, NULL)",
        params![
            crate::runtime::ids::new_message_id(),
            session_id,
            surface,
            actor_id,
            content,
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::state::db::open_state_db_at;
    use crate::state::messages::{store_message, StoreMessageInput};
    use crate::state::sessions::{create_session, CreateSessionInput};
    use crate::state::types::{
        ApprovalStatus, BeginExecutionResult, CreatePendingApprovalInput, DenyApprovalInput,
        MarkToolExecutedInput, MarkToolFailedInput, MessageRole, PendingTurnPhase,
        PendingTurnState, Surface,
    };
    use rusqlite::Connection;
    use serde_json::json;
    use tempfile::{tempdir, TempDir};

    struct Setup {
        _dir: TempDir,
        conn: Connection,
        session_id: String,
        user_message_id: String,
    }

    fn setup() -> Setup {
        let dir = tempdir().unwrap();
        let conn = open_state_db_at(dir.path().join("state.sqlite")).unwrap();
        let session = create_session(
            &conn,
            CreateSessionInput {
                title: Some("Approval".into()),
                surface_created: Surface::Cli,
                actor_id_created: Some("cli_owner".into()),
            },
        )
        .unwrap();
        let user_message = store_message(
            &conn,
            StoreMessageInput {
                session_id: session.id.clone(),
                role: MessageRole::User,
                surface: Surface::Cli,
                actor_id: Some("cli_owner".into()),
                content: "create notes".into(),
                metadata_json: None,
            },
        )
        .unwrap();

        Setup {
            _dir: dir,
            conn,
            session_id: session.id,
            user_message_id: user_message.id,
        }
    }

    fn pending_turn(session_id: &str, message_id: &str) -> PendingTurnState {
        PendingTurnState {
            request_id: "req_test".into(),
            turn_id: "turn_test".into(),
            request_envelope: json!({"surface":"cli"}),
            session_id: session_id.into(),
            user_message_id: message_id.into(),
            model_messages_so_far: json!([{"role":"user","content":"create notes"}]),
            declared_tools: json!([{"name":"write_file"}]),
            pending_tool_call: json!({"id":"toolcall_test","name":"write_file"}),
            compiled_context_audit_id: Some("audit_1".into()),
            selected_skill_ids: vec!["skill_a".into()],
            pinned_context_ids: vec!["ctx_a".into()],
            provider_id: "mock".into(),
            model_id: "mock-model".into(),
            generation_config: json!({"temperature":0}),
            provider_state_json: json!({}),
            current_round: 1,
            max_rounds: 4,
        }
    }

    fn approval_input(session_id: &str, message_id: &str) -> CreatePendingApprovalInput {
        CreatePendingApprovalInput {
            session_id: session_id.into(),
            request_id: "req_test".into(),
            turn_id: "turn_test".into(),
            user_message_id: message_id.into(),
            tool_call_id: "toolcall_test".into(),
            surface: Surface::Cli,
            actor_id: "cli_owner".into(),
            operation_name: "write_file".into(),
            classification: "local_modify".into(),
            summary: Some("Write notes.txt".into()),
            operation_target: Some("notes.txt".into()),
            reason: Some("File creation changes local disk".into()),
            arguments_preview_json: Some(r#"{"path":"notes.txt"}"#.into()),
        }
    }

    fn count_rows(conn: &Connection, table_name: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table_name}"), [], |row| {
            row.get(0)
        })
        .unwrap()
    }

    fn count_event_messages(conn: &Connection, event_type: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE role = 'event' AND content LIKE ?1",
            [format!("%\"event_type\":\"{event_type}\"%")],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn create_pending(setup: &mut Setup) -> String {
        let approval = super::create_pending_approval_with_turn(
            &mut setup.conn,
            approval_input(&setup.session_id, &setup.user_message_id),
            pending_turn(&setup.session_id, &setup.user_message_id),
        )
        .unwrap();
        approval.id
    }

    #[test]
    fn create_pending_approval_with_turn_is_atomic_and_writes_event() {
        let mut setup = setup();

        let approval = super::create_pending_approval_with_turn(
            &mut setup.conn,
            approval_input(&setup.session_id, &setup.user_message_id),
            pending_turn(&setup.session_id, &setup.user_message_id),
        )
        .unwrap();

        assert!(approval.id.starts_with("appr_"));
        assert_eq!(approval.status, ApprovalStatus::Pending.as_str());
        assert_eq!(count_rows(&setup.conn, "pending_approvals"), 1);
        assert_eq!(count_rows(&setup.conn, "pending_turns"), 1);
        assert_eq!(count_event_messages(&setup.conn, "approval_required"), 1);
    }

    #[test]
    fn failed_pending_approval_creation_rolls_back_all_approval_writes() {
        let mut setup = setup();
        let mut input = approval_input(&setup.session_id, &setup.user_message_id);
        input.user_message_id = "msg_missing".into();

        let result = super::create_pending_approval_with_turn(
            &mut setup.conn,
            input,
            pending_turn(&setup.session_id, &setup.user_message_id),
        );

        assert!(result.is_err());
        assert_eq!(count_rows(&setup.conn, "pending_approvals"), 0);
        assert_eq!(count_rows(&setup.conn, "pending_turns"), 0);
        assert_eq!(count_rows(&setup.conn, "messages"), 1);
    }

    #[test]
    fn begin_execution_once_enforces_same_session_actor_and_single_use() {
        let mut setup = setup();
        let approval_id = create_pending(&mut setup);

        assert!(matches!(
            super::begin_execution_once(
                &mut setup.conn,
                &approval_id,
                "wrong_session",
                "cli_owner"
            )
            .unwrap(),
            BeginExecutionResult::WrongSession
        ));
        assert!(matches!(
            super::begin_execution_once(
                &mut setup.conn,
                &approval_id,
                &setup.session_id,
                "stranger"
            )
            .unwrap(),
            BeginExecutionResult::ActorNotAllowed
        ));

        let started = super::begin_execution_once(
            &mut setup.conn,
            &approval_id,
            &setup.session_id,
            "cli_owner",
        )
        .unwrap();
        assert!(matches!(started, BeginExecutionResult::Started { .. }));

        assert!(matches!(
            super::begin_execution_once(
                &mut setup.conn,
                &approval_id,
                &setup.session_id,
                "cli_owner"
            )
            .unwrap(),
            BeginExecutionResult::AlreadyExecuting
        ));
    }

    #[test]
    fn approval_execution_recovery_transitions_are_enforced() {
        let mut setup = setup();
        let approval_id = create_pending(&mut setup);
        let _ = super::begin_execution_once(
            &mut setup.conn,
            &approval_id,
            &setup.session_id,
            "cli_owner",
        )
        .unwrap();
        let mut updated_turn = pending_turn(&setup.session_id, &setup.user_message_id);
        updated_turn.model_messages_so_far = json!(["tool_result_appended"]);

        super::mark_tool_executed_and_update_turn(
            &mut setup.conn,
            MarkToolExecutedInput {
                approval_id: approval_id.clone(),
                updated_turn,
                result_summary: Some("Wrote notes.txt".into()),
            },
        )
        .unwrap();

        let (status, phase): (String, String) = setup
            .conn
            .query_row(
                "SELECT pending_approvals.status, pending_turns.phase
                 FROM pending_approvals
                 JOIN pending_turns ON pending_turns.approval_id = pending_approvals.id
                 WHERE pending_approvals.id = ?1",
                [&approval_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, ApprovalStatus::Executed.as_str());
        assert_eq!(phase, PendingTurnPhase::ToolExecutedAwaitingModel.as_str());

        assert!(matches!(
            super::begin_execution_once(
                &mut setup.conn,
                &approval_id,
                &setup.session_id,
                "cli_owner"
            )
            .unwrap(),
            BeginExecutionResult::AlreadyExecuted
        ));

        super::mark_resume_failed(&setup.conn, &approval_id, "provider timed out").unwrap();
        let (status_after_failure, attempts): (String, i64) = setup
            .conn
            .query_row(
                "SELECT status, resume_attempt_count FROM pending_approvals WHERE id = ?1",
                [&approval_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(status_after_failure, ApprovalStatus::Executed.as_str());
        assert_eq!(attempts, 1);

        super::mark_approval_completed(&mut setup.conn, &approval_id, "msg_final").unwrap();
        let completed_status: String = setup
            .conn
            .query_row(
                "SELECT status FROM pending_approvals WHERE id = ?1",
                [&approval_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(completed_status, ApprovalStatus::Completed.as_str());
        assert_eq!(count_rows(&setup.conn, "pending_turns"), 0);
        assert_eq!(count_rows(&setup.conn, "pending_approvals"), 1);
    }

    #[test]
    fn denial_keeps_audit_row_and_removes_turn_after_completion() {
        let mut setup = setup();
        let approval_id = create_pending(&mut setup);
        let mut denied_turn = pending_turn(&setup.session_id, &setup.user_message_id);
        denied_turn.model_messages_so_far = json!(["approval_denied"]);

        let result = super::deny_approval_and_update_turn(
            &mut setup.conn,
            DenyApprovalInput {
                approval_id: approval_id.clone(),
                session_id: setup.session_id.clone(),
                actor_id: "cli_owner".into(),
                denied_turn,
            },
        )
        .unwrap();

        assert_eq!(result.approval.status, ApprovalStatus::Denied.as_str());
        assert_eq!(count_event_messages(&setup.conn, "approval_denied"), 1);

        super::complete_denied_turn(&mut setup.conn, &approval_id, "msg_final").unwrap();
        let status: String = setup
            .conn
            .query_row(
                "SELECT status FROM pending_approvals WHERE id = ?1",
                [&approval_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, ApprovalStatus::Denied.as_str());
        assert_eq!(count_rows(&setup.conn, "pending_turns"), 0);
        assert_eq!(count_rows(&setup.conn, "pending_approvals"), 1);
    }

    #[test]
    fn failed_and_completed_cleanup_helpers_preserve_audit_rows() {
        let mut setup = setup();
        let approval_id = create_pending(&mut setup);
        let _ = super::begin_execution_once(
            &mut setup.conn,
            &approval_id,
            &setup.session_id,
            "cli_owner",
        )
        .unwrap();

        super::mark_tool_failed(
            &mut setup.conn,
            MarkToolFailedInput {
                approval_id: approval_id.clone(),
                error_message: "tool failed".into(),
            },
        )
        .unwrap();

        let status: String = setup
            .conn
            .query_row(
                "SELECT status FROM pending_approvals WHERE id = ?1",
                [&approval_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, ApprovalStatus::Failed.as_str());

        let removed = super::cleanup_completed_pending_turns(&setup.conn).unwrap();
        assert_eq!(removed, 0);
        super::delete_pending_turn(&setup.conn, &approval_id).unwrap();
        assert_eq!(count_rows(&setup.conn, "pending_turns"), 0);
        assert_eq!(count_rows(&setup.conn, "pending_approvals"), 1);
    }

    #[test]
    fn stale_executing_recovery_marks_failed_without_retry() {
        let mut setup = setup();
        let approval_id = create_pending(&mut setup);
        setup
            .conn
            .execute(
                "UPDATE pending_approvals
                 SET status = 'executing', execution_started_at = '2026-06-07T00:00:00+00:00'
                 WHERE id = ?1",
                [&approval_id],
            )
            .unwrap();

        let changed = super::recover_stale_executing_approvals_at(
            &mut setup.conn,
            chrono::DateTime::parse_from_rfc3339("2026-06-07T00:11:00+00:00")
                .unwrap()
                .with_timezone(&chrono::Utc),
            chrono::Duration::minutes(10),
        )
        .unwrap();

        assert_eq!(changed, 1);
        let (status, error): (String, String) = setup
            .conn
            .query_row(
                "SELECT status, error_message FROM pending_approvals WHERE id = ?1",
                [&approval_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, ApprovalStatus::Failed.as_str());
        assert!(error.contains("interrupted"));
    }
}
