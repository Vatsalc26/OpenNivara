use super::tasks::MemoryTask;
use rusqlite::{params, Connection};

pub fn create_reminder(
    conn: &Connection,
    memory_id: &str,
    reminder_at: &str,
) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE tasks SET reminder_at = ?2 WHERE memory_id = ?1",
        params![memory_id, reminder_at],
    )?;
    Ok(())
}

pub fn list_due_reminders(conn: &Connection, now_utc: &str) -> anyhow::Result<Vec<MemoryTask>> {
    let mut stmt = conn.prepare(
        "SELECT memory_id, task_type, priority, status, due_at, reminder_at, completed_at, checklist_json
         FROM tasks
         WHERE reminder_at IS NOT NULL
           AND reminder_at <= ?1
           AND status IN ('planned', 'active', 'uncertain')
         ORDER BY reminder_at ASC",
    )?;
    let reminders = stmt
        .query_map([now_utc], |row| {
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
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(reminders)
}
