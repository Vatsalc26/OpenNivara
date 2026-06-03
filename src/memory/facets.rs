use super::types::{CreateMemoryFacet, MemoryFacet};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct FacetFilter {
    pub memory_id: Option<String>,
    pub domain: Option<String>,
    pub facet_type: Option<String>,
    pub label: Option<String>,
}

pub fn create_facet(conn: &Connection, input: &CreateMemoryFacet) -> anyhow::Result<MemoryFacet> {
    let now = Utc::now().to_rfc3339();
    let facet = MemoryFacet {
        id: format!("facet_{}", Uuid::new_v4()),
        memory_id: input.memory_id.clone(),
        domain: input.domain.clone(),
        facet_type: input.facet_type.clone(),
        label: input.label.clone(),
        details_json: input.details_json.clone(),
        sensitivity: input.sensitivity.clone(),
        confidence: input.confidence,
        source: input.source.clone(),
        created_at: now.clone(),
        updated_at: now,
    };
    conn.execute(
        "INSERT INTO memory_facets (
            id, memory_id, domain, facet_type, label, details_json, sensitivity,
            confidence, source, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            &facet.id,
            &facet.memory_id,
            &facet.domain,
            &facet.facet_type,
            &facet.label,
            &facet.details_json,
            &facet.sensitivity,
            facet.confidence,
            &facet.source,
            &facet.created_at,
            &facet.updated_at,
        ],
    )?;
    get_facet(conn, &facet.id)?.ok_or_else(|| anyhow::anyhow!("created facet missing"))
}

pub fn get_facet(conn: &Connection, id: &str) -> anyhow::Result<Option<MemoryFacet>> {
    conn.query_row(
        "SELECT id, memory_id, domain, facet_type, label, details_json, sensitivity,
            confidence, source, created_at, updated_at
         FROM memory_facets WHERE id = ?1",
        [id],
        facet_from_row,
    )
    .optional()
    .map_err(Into::into)
}

pub fn list_facets(conn: &Connection, filter: FacetFilter) -> anyhow::Result<Vec<MemoryFacet>> {
    let mut sql = String::from(
        "SELECT id, memory_id, domain, facet_type, label, details_json, sensitivity,
            confidence, source, created_at, updated_at
         FROM memory_facets WHERE 1=1",
    );
    let mut params_vec = Vec::new();
    if let Some(memory_id) = filter.memory_id {
        sql.push_str(" AND memory_id = ?");
        params_vec.push(memory_id);
    }
    if let Some(domain) = filter.domain {
        sql.push_str(" AND domain = ?");
        params_vec.push(domain);
    }
    if let Some(facet_type) = filter.facet_type {
        sql.push_str(" AND facet_type = ?");
        params_vec.push(facet_type);
    }
    if let Some(label) = filter.label {
        sql.push_str(" AND label = ?");
        params_vec.push(label);
    }
    sql.push_str(" ORDER BY confidence DESC, updated_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), facet_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn update_facet(conn: &Connection, facet: &MemoryFacet) -> anyhow::Result<MemoryFacet> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE memory_facets
         SET domain = ?2, facet_type = ?3, label = ?4, details_json = ?5,
             sensitivity = ?6, confidence = ?7, source = ?8, updated_at = ?9
         WHERE id = ?1",
        params![
            &facet.id,
            &facet.domain,
            &facet.facet_type,
            &facet.label,
            &facet.details_json,
            &facet.sensitivity,
            facet.confidence,
            &facet.source,
            &now,
        ],
    )?;
    get_facet(conn, &facet.id)?.ok_or_else(|| anyhow::anyhow!("updated facet missing"))
}

pub fn delete_facet(conn: &Connection, id: &str) -> anyhow::Result<()> {
    conn.execute("DELETE FROM memory_facets WHERE id = ?1", [id])?;
    Ok(())
}

fn facet_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryFacet> {
    Ok(MemoryFacet {
        id: row.get(0)?,
        memory_id: row.get(1)?,
        domain: row.get(2)?,
        facet_type: row.get(3)?,
        label: row.get(4)?,
        details_json: row.get(5)?,
        sensitivity: row.get(6)?,
        confidence: row.get(7)?,
        source: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}
