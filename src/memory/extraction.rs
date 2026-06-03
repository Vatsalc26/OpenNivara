use super::db;
use super::privacy;
use super::types::{CreateMemoryItem, CreateMemorySource, MemoryExtractionProposal, MemoryMode};
use rusqlite::{params, Connection};

pub fn extract_proposals_for_message(
    conn: &Connection,
    message: &str,
    session_id: Option<String>,
    mode: MemoryMode,
) -> anyhow::Result<Vec<MemoryExtractionProposal>> {
    if matches!(mode, MemoryMode::Off) {
        return Ok(vec![]);
    }

    let now = chrono::Utc::now().to_rfc3339();
    let source = db::create_source(
        conn,
        &CreateMemorySource {
            source_type: "chat".into(),
            source_ref: None,
            source_text: message.to_string(),
            source_quote: Some(message.chars().take(240).collect()),
            session_id,
            message_id: None,
            observed_at: now,
            timezone: "UTC".into(),
            sensitivity: "normal".into(),
            privacy_scope: "local".into(),
        },
    )?;

    let lower = message.to_lowercase();
    let mut proposals = Vec::new();
    if lower.contains("buy") || lower.contains("need to") || lower.contains("remember") {
        let sensitivity =
            if privacy::is_sensitive_category("relationships") && lower.contains("friend") {
                "relationships"
            } else {
                "normal"
            };
        let proposal_json = serde_json::json!({
            "proposed_memories": [],
            "proposed_tasks": [{
                "title": message,
                "status": "planned",
                "source_quote": message
            }],
            "ambiguities": if lower.contains("friend") { vec!["friend entity unresolved"] } else { Vec::<&str>::new() },
            "confidence": 0.65
        })
        .to_string();
        proposals.push(db::create_proposal(
            conn,
            &source.id,
            &proposal_json,
            sensitivity,
            0.65,
        )?);
    }
    Ok(proposals)
}

pub fn list_memory_proposals(conn: &Connection) -> anyhow::Result<Vec<MemoryExtractionProposal>> {
    db::list_proposals(conn)
}

pub fn reject_memory_proposal(conn: &Connection, proposal_id: &str) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE memory_proposals SET status = 'rejected', updated_at = datetime('now') WHERE id = ?1",
        [proposal_id],
    )?;
    Ok(())
}

pub fn approve_memory_proposal(conn: &Connection, proposal_id: &str) -> anyhow::Result<()> {
    let (source_id, proposal_json, sensitivity): (String, String, String) = conn.query_row(
        "SELECT source_id, proposal_json, sensitivity FROM memory_proposals WHERE id = ?1",
        [proposal_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let value: serde_json::Value = serde_json::from_str(&proposal_json)?;
    if let Some(tasks) = value
        .get("proposed_tasks")
        .and_then(serde_json::Value::as_array)
    {
        for task in tasks {
            let title = task
                .get("title")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("Untitled task")
                .to_string();
            let status = task
                .get("status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("planned")
                .to_string();
            db::create_memory_item(
                conn,
                &CreateMemoryItem {
                    memory_type: "task".into(),
                    title: title.clone(),
                    summary: format!("{title} ({status}, not confirmed complete)"),
                    details_json: task.to_string(),
                    status,
                    confidence: value
                        .get("confidence")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.65),
                    user_verified: true,
                    sensitivity: sensitivity.clone(),
                    visibility: "private".into(),
                    source_id: source_id.clone(),
                    observed_at: chrono::Utc::now().to_rfc3339(),
                    happened_at: None,
                    due_at: None,
                    timezone: "UTC".into(),
                    time_precision: "unknown".into(),
                    natural_time_phrase: None,
                    tags: vec!["proposal".into()],
                },
            )?;
        }
    }
    conn.execute(
        "UPDATE memory_proposals SET status = 'approved', updated_at = datetime('now') WHERE id = ?1",
        params![proposal_id],
    )?;
    Ok(())
}
