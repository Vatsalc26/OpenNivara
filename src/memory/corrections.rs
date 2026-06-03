use super::db;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemoryCorrection {
    pub id: String,
    pub correction_source_id: String,
    pub old_memory_id: Option<String>,
    pub new_memory_id: Option<String>,
    pub correction_type: String,
    pub reason: String,
    pub created_at: String,
}

pub fn create_correction(
    conn: &Connection,
    correction_source_id: &str,
    old_memory_id: Option<&str>,
    new_memory_id: Option<&str>,
    correction_type: &str,
    reason: &str,
) -> anyhow::Result<MemoryCorrection> {
    let correction = MemoryCorrection {
        id: format!("corr_{}", Uuid::new_v4()),
        correction_source_id: correction_source_id.to_string(),
        old_memory_id: old_memory_id.map(str::to_string),
        new_memory_id: new_memory_id.map(str::to_string),
        correction_type: correction_type.to_string(),
        reason: reason.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    conn.execute(
        "INSERT INTO memory_corrections (id, correction_source_id, old_memory_id, new_memory_id, correction_type, reason, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            &correction.id,
            &correction.correction_source_id,
            &correction.old_memory_id,
            &correction.new_memory_id,
            &correction.correction_type,
            &correction.reason,
            &correction.created_at,
        ],
    )?;
    Ok(correction)
}

pub fn retract_memory(conn: &Connection, memory_id: &str, reason: &str) -> anyhow::Result<()> {
    db::retract_memory_item(conn, memory_id, reason)
}

pub fn supersede_memory(
    conn: &Connection,
    old_memory_id: &str,
    new_memory_id: &str,
) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE memory_items SET superseded_by = ?2, status = 'retracted', updated_at = datetime('now') WHERE id = ?1",
        params![old_memory_id, new_memory_id],
    )?;
    Ok(())
}

pub fn list_corrections(conn: &Connection) -> anyhow::Result<Vec<MemoryCorrection>> {
    let mut stmt = conn.prepare(
        "SELECT id, correction_source_id, old_memory_id, new_memory_id, correction_type, reason, created_at
         FROM memory_corrections ORDER BY created_at DESC",
    )?;
    let corrections = stmt
        .query_map([], |row| {
            Ok(MemoryCorrection {
                id: row.get(0)?,
                correction_source_id: row.get(1)?,
                old_memory_id: row.get(2)?,
                new_memory_id: row.get(3)?,
                correction_type: row.get(4)?,
                reason: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(corrections)
}
