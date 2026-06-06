#![allow(dead_code)]
use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Helper structure representing a session record.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub status: String,
    pub source_created: String,
    pub active: bool,
}

/// Helper structure representing a message record.
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct DbMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub source: String,
    pub content: String,
    pub created_at: String,
    pub metadata_json: Option<String>,
}

/// Helper structure representing a pending approval.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PendingApproval {
    pub id: String,
    pub session_id: String,
    pub source: String,
    pub request_json: String,
    pub status: String,
    pub created_at: String,
    pub expires_at: String,
}

/// Resolves the absolute path where the opennivara_state.sqlite database should reside.
pub fn get_state_db_path() -> anyhow::Result<PathBuf> {
    Ok(crate::config_paths::config_dir()?.join("opennivara_state.sqlite"))
}

/// Initializes the state database and creates all tables if they don't exist.
pub fn init_db() -> anyhow::Result<Connection> {
    let db_path = get_state_db_path()?;

    // Ensure parent folder structure exists
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open state SQLite database: {}", e))?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    // Create sessions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            status TEXT NOT NULL,
            source_created TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1
        );",
        [],
    )?;

    // Create messages table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            source TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            metadata_json TEXT,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );",
        [],
    )?;

    // Create active_sessions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS active_sessions (
            user_key TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );",
        [],
    )?;

    // Create pending_approvals table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pending_approvals (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            source TEXT NOT NULL,
            request_json TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );",
        [],
    )?;

    // Create session_pinned_contexts table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS session_pinned_contexts (
            session_id TEXT NOT NULL,
            context_id TEXT NOT NULL,
            pinned_at TEXT NOT NULL,
            PRIMARY KEY(session_id, context_id),
            FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
        );",
        [],
    )?;

    // Create session_pinned_skills table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS session_pinned_skills (
            session_id TEXT NOT NULL,
            skill_id TEXT NOT NULL,
            pinned_at TEXT NOT NULL,
            PRIMARY KEY(session_id, skill_id),
            FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
        );",
        [],
    )?;

    // Create indices
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id);",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON sessions(updated_at);",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pending_approvals_status ON pending_approvals(status);",
        [],
    )?;

    Ok(conn)
}

/// Creates a new session in the database and returns the generated session ID.
pub fn create_session(
    conn: &Connection,
    source: &str,
    title: Option<&str>,
) -> anyhow::Result<String> {
    let session_id = Uuid::new_v4().to_string();
    let current_time = Utc::now().to_rfc3339();
    let session_title = title.unwrap_or("New Conversation");

    conn.execute(
        "INSERT INTO sessions (id, title, created_at, updated_at, status, source_created, active)
         VALUES (?1, ?2, ?3, ?4, 'active', ?5, 1)",
        params![
            session_id,
            session_title,
            current_time,
            current_time,
            source
        ],
    )?;

    Ok(session_id)
}

/// Stores a new user or model message in the database and updates the session's updated_at timestamp.
pub fn store_message(
    conn: &Connection,
    session_id: &str,
    role: &str,
    source: &str,
    content: &str,
    metadata_json: Option<&str>,
) -> anyhow::Result<DbMessage> {
    let msg_id = crate::runtime::ids::new_message_id();
    let current_time = Utc::now().to_rfc3339();

    // Insert message
    conn.execute(
        "INSERT INTO messages (id, session_id, role, source, content, created_at, metadata_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            msg_id,
            session_id,
            role,
            source,
            content,
            current_time,
            metadata_json
        ],
    )?;

    // Update session updated_at
    conn.execute(
        "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
        params![current_time, session_id],
    )?;

    Ok(DbMessage {
        id: msg_id,
        session_id: session_id.to_string(),
        role: role.to_string(),
        source: source.to_string(),
        content: content.to_string(),
        created_at: current_time,
        metadata_json: metadata_json.map(str::to_string),
    })
}

/// Sets a session as the active session for a specific user key (e.g. "cli" or "telegram_<chat_id>").
pub fn set_active_session(
    conn: &Connection,
    user_key: &str,
    session_id: &str,
) -> anyhow::Result<()> {
    let current_time = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO active_sessions (user_key, session_id, updated_at)
         VALUES (?1, ?2, ?3)",
        params![user_key, session_id, current_time],
    )?;

    Ok(())
}

/// Gets the active session ID for a specific user key.
pub fn get_active_session(conn: &Connection, user_key: &str) -> anyhow::Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT session_id FROM active_sessions WHERE user_key = ?1")?;
    let mut rows = stmt.query(params![user_key])?;

    if let Some(row) = rows.next()? {
        let session_id: String = row.get(0)?;
        Ok(Some(session_id))
    } else {
        Ok(None)
    }
}

/// Gets the most recently updated active session ID, regardless of who started it.
pub fn get_latest_active_session(conn: &Connection) -> anyhow::Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM sessions 
         WHERE status = 'active' 
         ORDER BY updated_at DESC LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let session_id: String = row.get(0)?;
        Ok(Some(session_id))
    } else {
        Ok(None)
    }
}

/// Lists all active and closed sessions in the database.
pub fn list_sessions(conn: &Connection) -> anyhow::Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, updated_at, status, source_created, active 
         FROM sessions 
         ORDER BY updated_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        let active_int: i32 = row.get(6)?;
        Ok(Session {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            status: row.get(4)?,
            source_created: row.get(5)?,
            active: active_int != 0,
        })
    })?;

    let mut sessions = Vec::new();
    for r in rows {
        sessions.push(r?);
    }
    Ok(sessions)
}

/// Loads the message history for a specific session in chronological order.
pub fn get_session_messages(conn: &Connection, session_id: &str) -> anyhow::Result<Vec<DbMessage>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, source, content, created_at, metadata_json 
         FROM messages 
         WHERE session_id = ?1 
         ORDER BY created_at ASC",
    )?;

    let rows = stmt.query_map(params![session_id], |row| {
        Ok(DbMessage {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: row.get(2)?,
            source: row.get(3)?,
            content: row.get(4)?,
            created_at: row.get(5)?,
            metadata_json: row.get(6)?,
        })
    })?;

    let mut msgs = Vec::new();
    for r in rows {
        msgs.push(r?);
    }
    Ok(msgs)
}

/// Gets single session detail by ID.
pub fn get_session(conn: &Connection, session_id: &str) -> anyhow::Result<Option<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, updated_at, status, source_created, active 
         FROM sessions 
         WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![session_id], |row| {
        let active_int: i32 = row.get(6)?;
        Ok(Session {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            status: row.get(4)?,
            source_created: row.get(5)?,
            active: active_int != 0,
        })
    })?;

    if let Some(r) = rows.next() {
        Ok(Some(r?))
    } else {
        Ok(None)
    }
}

/// Renames a session's title.
pub fn rename_session(conn: &Connection, session_id: &str, title: &str) -> anyhow::Result<()> {
    let current_time = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE sessions SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, current_time, session_id],
    )?;
    Ok(())
}

/// Closes a session by setting active to 0 and status to 'closed'.
pub fn close_session(conn: &Connection, session_id: &str) -> anyhow::Result<()> {
    let current_time = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE sessions SET active = 0, status = 'closed', updated_at = ?1 WHERE id = ?2",
        params![current_time, session_id],
    )?;
    Ok(())
}

/// Gets list of active user key sessions mappings.
pub fn get_active_sessions_list(
    conn: &Connection,
) -> anyhow::Result<Vec<(String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT user_key, session_id, updated_at FROM active_sessions ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
    let mut active = Vec::new();
    for r in rows {
        active.push(r?);
    }
    Ok(active)
}

/// Stores a pending approval request.
pub fn create_pending_approval(
    conn: &Connection,
    session_id: &str,
    source: &str,
    request_json: &str,
    expires_in_seconds: i64,
) -> anyhow::Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let created_at = now.to_rfc3339();
    let expires_at = (now + chrono::Duration::seconds(expires_in_seconds)).to_rfc3339();

    conn.execute(
        "INSERT INTO pending_approvals (id, session_id, source, request_json, status, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, 'pending', ?5, ?6)",
        params![id, session_id, source, request_json, created_at, expires_at],
    )?;

    Ok(id)
}

/// Resolves a pending approval (sets status to 'approved' or 'denied').
pub fn update_pending_approval(conn: &Connection, id: &str, status: &str) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE pending_approvals SET status = ?1 WHERE id = ?2",
        params![status, id],
    )?;
    Ok(())
}

pub fn pin_context(conn: &Connection, session_id: &str, context_id: &str) -> anyhow::Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO session_pinned_contexts (session_id, context_id, pinned_at)
         VALUES (?1, ?2, ?3)",
        params![session_id, context_id, now],
    )?;
    Ok(())
}

pub fn unpin_context(conn: &Connection, session_id: &str, context_id: &str) -> anyhow::Result<()> {
    conn.execute(
        "DELETE FROM session_pinned_contexts WHERE session_id = ?1 AND context_id = ?2",
        params![session_id, context_id],
    )?;
    Ok(())
}

pub fn list_pinned_contexts(conn: &Connection, session_id: &str) -> anyhow::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT context_id FROM session_pinned_contexts WHERE session_id = ?1 ORDER BY pinned_at ASC"
    )?;
    let rows = stmt.query_map(params![session_id], |row| row.get(0))?;
    let mut pinned = Vec::new();
    for r in rows {
        pinned.push(r?);
    }
    Ok(pinned)
}

pub fn pin_skill(conn: &Connection, session_id: &str, skill_id: &str) -> anyhow::Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO session_pinned_skills (session_id, skill_id, pinned_at)
         VALUES (?1, ?2, ?3)",
        params![session_id, skill_id, now],
    )?;
    Ok(())
}

pub fn unpin_skill(conn: &Connection, session_id: &str, skill_id: &str) -> anyhow::Result<()> {
    conn.execute(
        "DELETE FROM session_pinned_skills WHERE session_id = ?1 AND skill_id = ?2",
        params![session_id, skill_id],
    )?;
    Ok(())
}

pub fn list_pinned_skills(conn: &Connection, session_id: &str) -> anyhow::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT skill_id FROM session_pinned_skills WHERE session_id = ?1 ORDER BY pinned_at ASC",
    )?;
    let rows = stmt.query_map(params![session_id], |row| row.get(0))?;
    let mut pinned = Vec::new();
    for r in rows {
        pinned.push(r?);
    }
    Ok(pinned)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn in_memory_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute(
            "CREATE TABLE sessions (
                id TEXT PRIMARY KEY,
                title TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                status TEXT NOT NULL,
                source_created TEXT NOT NULL,
                active INTEGER NOT NULL DEFAULT 1
            );",
            [],
        )
        .expect("sessions table");
        conn.execute(
            "CREATE TABLE messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                source TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                metadata_json TEXT,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            );",
            [],
        )
        .expect("messages table");
        conn
    }

    #[test]
    fn store_message_returns_inserted_db_message() {
        let conn = in_memory_conn();
        conn.execute(
            "INSERT INTO sessions (id, title, created_at, updated_at, status, source_created, active)
             VALUES ('sess_test', 'Test', '2026-06-07T00:00:00Z', '2026-06-07T00:00:00Z', 'active', 'CLI', 1)",
            [],
        )
        .expect("insert session");

        let message = store_message(
            &conn,
            "sess_test",
            "user",
            "CLI",
            "hello",
            Some(r#"{"client":"test"}"#),
        )
        .expect("store message");

        assert!(message.id.starts_with("msg_"));
        assert_eq!(message.session_id, "sess_test");
        assert_eq!(message.role, "user");
        assert_eq!(message.source, "CLI");
        assert_eq!(message.content, "hello");
        assert_eq!(
            message.metadata_json.as_deref(),
            Some(r#"{"client":"test"}"#)
        );

        let loaded = get_session_messages(&conn, "sess_test").expect("messages");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, message.id);
    }
}
