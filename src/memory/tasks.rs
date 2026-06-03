use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemoryTask {
    pub memory_id: String,
    pub task_type: String,
    pub priority: i32,
    pub status: String,
    pub due_at: Option<String>,
    pub reminder_at: Option<String>,
    pub completed_at: Option<String>,
    pub checklist_json: String,
}

pub fn list_tasks(conn: &Connection, status: Option<&str>) -> anyhow::Result<Vec<MemoryTask>> {
    let sql = if status.is_some() {
        "SELECT memory_id, task_type, priority, status, due_at, reminder_at, completed_at, checklist_json
         FROM tasks WHERE status = ?1 ORDER BY due_at IS NULL, due_at ASC"
    } else {
        "SELECT memory_id, task_type, priority, status, due_at, reminder_at, completed_at, checklist_json
         FROM tasks ORDER BY due_at IS NULL, due_at ASC"
    };
    let mut stmt = conn.prepare(sql)?;
    let map = |row: &rusqlite::Row<'_>| {
        Ok(MemoryTask {
            memory_id: row.get(0)?,
            task_type: row.get(1)?,
            priority: row.get(2)?,
            status: row.get(3)?,
            due_at: row.get(4)?,
            reminder_at: row.get(5)?,
            completed_at: row.get(6)?,
            checklist_json: row.get(7)?,
        })
    };
    if let Some(status) = status {
        stmt.query_map([status], map)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    } else {
        stmt.query_map([], map)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
}

pub fn update_task_status(conn: &Connection, memory_id: &str, status: &str) -> anyhow::Result<()> {
    let completed_at = if status == "completed" {
        Some(chrono::Utc::now().to_rfc3339())
    } else {
        None
    };
    conn.execute(
        "UPDATE tasks SET status = ?2, completed_at = ?3 WHERE memory_id = ?1",
        params![memory_id, status, completed_at],
    )?;
    conn.execute(
        "UPDATE memory_items SET status = ?2, completed_at = ?3, updated_at = datetime('now') WHERE id = ?1",
        params![memory_id, status, completed_at],
    )?;
    Ok(())
}
