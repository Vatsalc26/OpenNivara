use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemoryMapSummary {
    pub entity_count: u32,
    pub relationship_count: u32,
    pub task_count: u32,
    pub ambiguous_link_count: u32,
}

pub fn summarize_memory_map(conn: &Connection) -> anyhow::Result<MemoryMapSummary> {
    let entity_count = count(
        conn,
        "SELECT count(*) FROM entities WHERE deleted_at IS NULL",
    )?;
    let relationship_count = count(conn, "SELECT count(*) FROM entity_relationships")?;
    let task_count = count(conn, "SELECT count(*) FROM tasks")?;
    let ambiguous_link_count = count(
        conn,
        "SELECT count(*) FROM memory_entities WHERE resolution_status = 'ambiguous'",
    )?;
    Ok(MemoryMapSummary {
        entity_count,
        relationship_count,
        task_count,
        ambiguous_link_count,
    })
}

fn count(conn: &Connection, sql: &str) -> anyhow::Result<u32> {
    conn.query_row(sql, [], |row| row.get::<_, u32>(0))
        .map_err(Into::into)
}
