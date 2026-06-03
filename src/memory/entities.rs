use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EntityResolutionResult {
    pub resolved_entity_id: Option<String>,
    pub candidates: Vec<String>,
    pub confidence: f64,
    pub resolution_status: String,
    pub needs_user_clarification: bool,
}

pub fn resolve_entity_mention(
    conn: &Connection,
    mention: &str,
) -> anyhow::Result<EntityResolutionResult> {
    let like = mention.trim().to_lowercase();
    if like.is_empty() {
        return Ok(unresolved());
    }

    let mut stmt = conn.prepare(
        "SELECT e.id
         FROM entities e
         LEFT JOIN entity_aliases a ON a.entity_id = e.id
         WHERE e.deleted_at IS NULL
           AND (lower(e.display_name) = ?1 OR lower(e.canonical_name) = ?1 OR lower(a.alias) = ?1)
         GROUP BY e.id
         LIMIT 5",
    )?;
    let candidates = stmt
        .query_map([like], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(match candidates.len() {
        0 => unresolved(),
        1 => EntityResolutionResult {
            resolved_entity_id: Some(candidates[0].clone()),
            candidates,
            confidence: 0.9,
            resolution_status: "resolved".into(),
            needs_user_clarification: false,
        },
        _ => EntityResolutionResult {
            resolved_entity_id: None,
            candidates,
            confidence: 0.5,
            resolution_status: "ambiguous".into(),
            needs_user_clarification: true,
        },
    })
}

pub fn list_ambiguous_entities(conn: &Connection) -> anyhow::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT entity_id FROM memory_entities WHERE resolution_status = 'ambiguous'",
    )?;
    let entities = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entities)
}

pub fn approve_entity_resolution(
    conn: &Connection,
    memory_id: &str,
    entity_id: &str,
    role: &str,
) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE memory_entities
         SET resolution_status = 'resolved', confidence = 1.0
         WHERE memory_id = ?1 AND entity_id = ?2 AND role = ?3",
        params![memory_id, entity_id, role],
    )?;
    Ok(())
}

pub fn merge_entities(
    conn: &Connection,
    from_entity_id: &str,
    to_entity_id: &str,
) -> anyhow::Result<String> {
    conn.execute(
        "UPDATE memory_entities SET entity_id = ?2 WHERE entity_id = ?1",
        params![from_entity_id, to_entity_id],
    )?;
    conn.execute(
        "UPDATE entity_aliases SET entity_id = ?2 WHERE entity_id = ?1",
        params![from_entity_id, to_entity_id],
    )?;
    conn.execute(
        "UPDATE entities SET deleted_at = datetime('now') WHERE id = ?1",
        [from_entity_id],
    )?;
    Ok(format!("merge_{}", Uuid::new_v4()))
}

pub fn split_entity(_conn: &Connection, entity_id: &str) -> anyhow::Result<String> {
    Ok(format!("split_required_for_{entity_id}"))
}

fn unresolved() -> EntityResolutionResult {
    EntityResolutionResult {
        resolved_entity_id: None,
        candidates: vec![],
        confidence: 0.0,
        resolution_status: "unresolved".into(),
        needs_user_clarification: true,
    }
}
