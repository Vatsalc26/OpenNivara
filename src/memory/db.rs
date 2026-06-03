use super::migrations;
use super::types::{
    CreateMemoryItem, CreateMemorySource, MemoryExtractionProposal, MemoryItem, MemorySettings,
    MemorySource, MemoryStatus,
};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub fn memory_db_path() -> anyhow::Result<PathBuf> {
    Ok(crate::config_paths::config_dir()?.join("opennivara_memory.sqlite"))
}

pub fn open_memory_db() -> anyhow::Result<Connection> {
    let path = memory_db_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(&path)
        .map_err(|err| anyhow::anyhow!("Failed to open memory SQLite database: {err}"))?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    migrations::run_migrations(&conn)?;
    ensure_default_settings(&conn)?;
    Ok(conn)
}

pub fn status() -> anyhow::Result<MemoryStatus> {
    let conn = open_memory_db()?;
    let item_count: u32 = conn.query_row(
        "SELECT count(*) FROM memory_items WHERE deleted_at IS NULL",
        [],
        |row| row.get::<_, u32>(0),
    )?;
    let proposal_count: u32 = conn.query_row(
        "SELECT count(*) FROM memory_proposals WHERE status = 'pending'",
        [],
        |row| row.get::<_, u32>(0),
    )?;
    let schema_version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    Ok(MemoryStatus {
        db_path: memory_db_path()?.to_string_lossy().into_owned(),
        initialized: true,
        schema_version,
        item_count,
        proposal_count,
        vector_enabled: cfg!(feature = "memory-vector"),
    })
}

pub fn validate() -> anyhow::Result<MemoryStatus> {
    status()
}

pub fn repair() -> anyhow::Result<MemoryStatus> {
    let conn = open_memory_db()?;
    migrations::run_migrations(&conn)?;
    drop(conn);
    status()
}

pub fn get_settings(conn: &Connection) -> anyhow::Result<MemorySettings> {
    let settings_json: Option<String> = conn
        .query_row(
            "SELECT settings_json FROM memory_settings WHERE singleton_id = 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    match settings_json {
        Some(json) => Ok(serde_json::from_str(&json)?),
        None => Ok(MemorySettings::default()),
    }
}

pub fn save_settings(conn: &Connection, settings: &MemorySettings) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO memory_settings (singleton_id, settings_json, updated_at)
         VALUES (1, ?1, ?2)
         ON CONFLICT(singleton_id) DO UPDATE SET settings_json = excluded.settings_json, updated_at = excluded.updated_at",
        params![serde_json::to_string(settings)?, now],
    )?;
    Ok(())
}

fn ensure_default_settings(conn: &Connection) -> anyhow::Result<()> {
    if get_settings(conn).is_err() {
        save_settings(conn, &MemorySettings::default())?;
    }
    let exists: Option<i64> = conn
        .query_row(
            "SELECT singleton_id FROM memory_settings WHERE singleton_id = 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if exists.is_none() {
        save_settings(conn, &MemorySettings::default())?;
    }
    Ok(())
}

pub fn create_source(
    conn: &Connection,
    input: &CreateMemorySource,
) -> anyhow::Result<MemorySource> {
    let now = Utc::now().to_rfc3339();
    let source = MemorySource {
        id: format!("src_{}", Uuid::new_v4()),
        source_type: input.source_type.clone(),
        source_ref: input.source_ref.clone(),
        source_text: input.source_text.clone(),
        source_quote: input.source_quote.clone(),
        session_id: input.session_id.clone(),
        message_id: input.message_id.clone(),
        created_at: now,
        observed_at: input.observed_at.clone(),
        timezone: input.timezone.clone(),
        sensitivity: input.sensitivity.clone(),
        privacy_scope: input.privacy_scope.clone(),
    };
    conn.execute(
        "INSERT INTO memory_sources (
            id, source_type, source_ref, source_text, source_quote, session_id, message_id,
            created_at, observed_at, timezone, sensitivity, privacy_scope
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            source.id,
            source.source_type,
            source.source_ref,
            source.source_text,
            source.source_quote,
            source.session_id,
            source.message_id,
            source.created_at,
            source.observed_at,
            source.timezone,
            source.sensitivity,
            source.privacy_scope
        ],
    )?;
    get_source(conn, &source.id)?.ok_or_else(|| anyhow::anyhow!("created source missing"))
}

pub fn get_source(conn: &Connection, source_id: &str) -> anyhow::Result<Option<MemorySource>> {
    conn.query_row(
        "SELECT id, source_type, source_ref, source_text, source_quote, session_id, message_id,
            created_at, observed_at, timezone, sensitivity, privacy_scope
         FROM memory_sources WHERE id = ?1",
        [source_id],
        |row| {
            Ok(MemorySource {
                id: row.get(0)?,
                source_type: row.get(1)?,
                source_ref: row.get(2)?,
                source_text: row.get(3)?,
                source_quote: row.get(4)?,
                session_id: row.get(5)?,
                message_id: row.get(6)?,
                created_at: row.get(7)?,
                observed_at: row.get(8)?,
                timezone: row.get(9)?,
                sensitivity: row.get(10)?,
                privacy_scope: row.get(11)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

pub fn create_memory_item(
    conn: &Connection,
    input: &CreateMemoryItem,
) -> anyhow::Result<MemoryItem> {
    let now = Utc::now().to_rfc3339();
    let item = MemoryItem {
        id: format!("mem_{}", Uuid::new_v4()),
        memory_type: input.memory_type.clone(),
        title: input.title.clone(),
        summary: input.summary.clone(),
        details_json: input.details_json.clone(),
        status: input.status.clone(),
        confidence: input.confidence,
        user_verified: input.user_verified,
        sensitivity: input.sensitivity.clone(),
        visibility: input.visibility.clone(),
        source_id: input.source_id.clone(),
        created_at: now.clone(),
        updated_at: now,
        observed_at: input.observed_at.clone(),
        valid_from: None,
        valid_until: None,
        happened_at: input.happened_at.clone(),
        starts_at: None,
        ends_at: None,
        due_at: input.due_at.clone(),
        completed_at: None,
        timezone: input.timezone.clone(),
        time_precision: input.time_precision.clone(),
        natural_time_phrase: input.natural_time_phrase.clone(),
        recurrence_rule: None,
        superseded_by: None,
        deleted_at: None,
    };
    conn.execute(
        "INSERT INTO memory_items (
            id, memory_type, title, summary, details_json, status, confidence, user_verified,
            sensitivity, visibility, source_id, created_at, updated_at, observed_at,
            happened_at, due_at, timezone, time_precision, natural_time_phrase
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
        params![
            item.id,
            item.memory_type,
            item.title,
            item.summary,
            item.details_json,
            item.status,
            item.confidence,
            if item.user_verified { 1 } else { 0 },
            item.sensitivity,
            item.visibility,
            item.source_id,
            item.created_at,
            item.updated_at,
            item.observed_at,
            item.happened_at,
            item.due_at,
            item.timezone,
            item.time_precision,
            item.natural_time_phrase,
        ],
    )?;

    if input.memory_type == "task" {
        conn.execute(
            "INSERT OR IGNORE INTO tasks (memory_id, task_type, priority, status, due_at, reminder_at, completed_at, checklist_json)
             VALUES (?1, 'todo', 3, ?2, ?3, NULL, NULL, '[]')",
            params![item.id, item.status, item.due_at],
        )?;
    }

    refresh_fts_for_item(conn, &item.id, &input.tags)?;
    get_memory_item(conn, &item.id)?.ok_or_else(|| anyhow::anyhow!("created memory item missing"))
}

pub fn get_memory_item(conn: &Connection, id: &str) -> anyhow::Result<Option<MemoryItem>> {
    conn.query_row(
        "SELECT id, memory_type, title, summary, details_json, status, confidence, user_verified,
            sensitivity, visibility, source_id, created_at, updated_at, observed_at, valid_from,
            valid_until, happened_at, starts_at, ends_at, due_at, completed_at, timezone,
            time_precision, natural_time_phrase, recurrence_rule, superseded_by, deleted_at
         FROM memory_items WHERE id = ?1",
        [id],
        memory_item_from_row,
    )
    .optional()
    .map_err(Into::into)
}

pub fn list_memory_items(conn: &Connection, limit: u32) -> anyhow::Result<Vec<MemoryItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, memory_type, title, summary, details_json, status, confidence, user_verified,
            sensitivity, visibility, source_id, created_at, updated_at, observed_at, valid_from,
            valid_until, happened_at, starts_at, ends_at, due_at, completed_at, timezone,
            time_precision, natural_time_phrase, recurrence_rule, superseded_by, deleted_at
         FROM memory_items WHERE deleted_at IS NULL ORDER BY observed_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit], memory_item_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn update_memory_item(conn: &Connection, item: &MemoryItem) -> anyhow::Result<MemoryItem> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE memory_items
         SET title = ?2, summary = ?3, details_json = ?4, status = ?5, confidence = ?6,
             user_verified = ?7, sensitivity = ?8, visibility = ?9, updated_at = ?10,
             happened_at = ?11, due_at = ?12, completed_at = ?13, natural_time_phrase = ?14,
             superseded_by = ?15, deleted_at = ?16
         WHERE id = ?1",
        params![
            item.id,
            item.title,
            item.summary,
            item.details_json,
            item.status,
            item.confidence,
            if item.user_verified { 1 } else { 0 },
            item.sensitivity,
            item.visibility,
            now,
            item.happened_at,
            item.due_at,
            item.completed_at,
            item.natural_time_phrase,
            item.superseded_by,
            item.deleted_at,
        ],
    )?;
    refresh_fts_for_item(conn, &item.id, &[])?;
    get_memory_item(conn, &item.id)?.ok_or_else(|| anyhow::anyhow!("updated memory item missing"))
}

pub fn retract_memory_item(conn: &Connection, id: &str, reason: &str) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE memory_items SET status = 'retracted', updated_at = ?2 WHERE id = ?1",
        params![id, now],
    )?;
    conn.execute("DELETE FROM memory_fts WHERE memory_id = ?1", [id])?;
    conn.execute(
        "INSERT INTO memory_corrections (id, correction_source_id, old_memory_id, new_memory_id, correction_type, reason, created_at)
         VALUES (?1, ?2, ?3, NULL, 'retract', ?4, ?5)",
        params![
            format!("corr_{}", Uuid::new_v4()),
            "__system__",
            id,
            reason,
            now
        ],
    )
    .ok();
    Ok(())
}

pub fn delete_memory_item(conn: &Connection, id: &str) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE memory_items SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![id, now],
    )?;
    conn.execute("DELETE FROM memory_fts WHERE memory_id = ?1", [id])?;
    Ok(())
}

pub fn create_proposal(
    conn: &Connection,
    source_id: &str,
    proposal_json: &str,
    sensitivity: &str,
    confidence: f64,
) -> anyhow::Result<MemoryExtractionProposal> {
    let now = Utc::now().to_rfc3339();
    let proposal = MemoryExtractionProposal {
        id: format!("prop_{}", Uuid::new_v4()),
        source_id: source_id.to_string(),
        proposal_json: proposal_json.to_string(),
        sensitivity: sensitivity.to_string(),
        confidence,
        status: "pending".into(),
        created_at: now.clone(),
    };
    conn.execute(
        "INSERT INTO memory_proposals (id, source_id, proposal_json, sensitivity, confidence, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            proposal.id,
            proposal.source_id,
            proposal.proposal_json,
            proposal.sensitivity,
            proposal.confidence,
            proposal.status,
            now,
        ],
    )?;
    Ok(proposal)
}

pub fn list_proposals(conn: &Connection) -> anyhow::Result<Vec<MemoryExtractionProposal>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_id, proposal_json, sensitivity, confidence, status, created_at
         FROM memory_proposals ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(MemoryExtractionProposal {
            id: row.get(0)?,
            source_id: row.get(1)?,
            proposal_json: row.get(2)?,
            sensitivity: row.get(3)?,
            confidence: row.get(4)?,
            status: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn refresh_fts_for_item(conn: &Connection, memory_id: &str, tags: &[String]) -> anyhow::Result<()> {
    conn.execute("DELETE FROM memory_fts WHERE memory_id = ?1", [memory_id])?;
    let source_text: String = conn.query_row(
        "SELECT s.source_text
         FROM memory_items i JOIN memory_sources s ON s.id = i.source_id
         WHERE i.id = ?1",
        [memory_id],
        |row| row.get(0),
    )?;
    let item = get_memory_item(conn, memory_id)?.ok_or_else(|| anyhow::anyhow!("missing item"))?;
    conn.execute(
        "INSERT INTO memory_fts (memory_id, title, summary, source_text, entity_names, tags)
         VALUES (?1, ?2, ?3, ?4, '', ?5)",
        params![
            memory_id,
            item.title,
            item.summary,
            source_text,
            tags.join(" ")
        ],
    )?;
    Ok(())
}

pub(crate) fn memory_item_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryItem> {
    Ok(MemoryItem {
        id: row.get(0)?,
        memory_type: row.get(1)?,
        title: row.get(2)?,
        summary: row.get(3)?,
        details_json: row.get(4)?,
        status: row.get(5)?,
        confidence: row.get(6)?,
        user_verified: row.get::<_, i64>(7)? != 0,
        sensitivity: row.get(8)?,
        visibility: row.get(9)?,
        source_id: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
        observed_at: row.get(13)?,
        valid_from: row.get(14)?,
        valid_until: row.get(15)?,
        happened_at: row.get(16)?,
        starts_at: row.get(17)?,
        ends_at: row.get(18)?,
        due_at: row.get(19)?,
        completed_at: row.get(20)?,
        timezone: row.get(21)?,
        time_precision: row.get(22)?,
        natural_time_phrase: row.get(23)?,
        recurrence_rule: row.get(24)?,
        superseded_by: row.get(25)?,
        deleted_at: row.get(26)?,
    })
}
