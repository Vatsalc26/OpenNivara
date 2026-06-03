use super::types::PromptAudit;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

pub struct CreatePromptAudit {
    pub session_id: Option<String>,
    pub message_id: Option<String>,
    pub user_message: String,
    pub compiled_context_json: String,
    pub included_memory_ids_json: String,
    pub included_task_ids_json: String,
    pub included_workspace_refs_json: String,
    pub token_budget_json: String,
}

pub fn create_audit(conn: &Connection, input: CreatePromptAudit) -> anyhow::Result<PromptAudit> {
    let audit = PromptAudit {
        id: format!("audit_{}", Uuid::new_v4()),
        session_id: input.session_id,
        message_id: input.message_id,
        user_message: input.user_message,
        compiled_context_json: input.compiled_context_json,
        included_memory_ids_json: input.included_memory_ids_json,
        included_task_ids_json: input.included_task_ids_json,
        included_workspace_refs_json: input.included_workspace_refs_json,
        token_budget_json: input.token_budget_json,
        created_at: Utc::now().to_rfc3339(),
    };
    conn.execute(
        "INSERT INTO prompt_audits (
            id, session_id, message_id, user_message, compiled_context_json,
            included_memory_ids_json, included_task_ids_json, included_workspace_refs_json,
            token_budget_json, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            audit.id,
            audit.session_id,
            audit.message_id,
            audit.user_message,
            audit.compiled_context_json,
            audit.included_memory_ids_json,
            audit.included_task_ids_json,
            audit.included_workspace_refs_json,
            audit.token_budget_json,
            audit.created_at,
        ],
    )?;
    Ok(audit)
}

pub fn get_last_audit(conn: &Connection) -> anyhow::Result<Option<PromptAudit>> {
    conn.query_row(
        "SELECT id, session_id, message_id, user_message, compiled_context_json,
            included_memory_ids_json, included_task_ids_json, included_workspace_refs_json,
            token_budget_json, created_at
         FROM prompt_audits ORDER BY created_at DESC, id DESC LIMIT 1",
        [],
        |row| {
            Ok(PromptAudit {
                id: row.get(0)?,
                session_id: row.get(1)?,
                message_id: row.get(2)?,
                user_message: row.get(3)?,
                compiled_context_json: row.get(4)?,
                included_memory_ids_json: row.get(5)?,
                included_task_ids_json: row.get(6)?,
                included_workspace_refs_json: row.get(7)?,
                token_budget_json: row.get(8)?,
                created_at: row.get(9)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}
