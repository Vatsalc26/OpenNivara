use super::db;
use super::types::{MemoryItem, MemorySearchQuery, MemorySearchResult};
use rusqlite::{params, Connection, Row};

pub fn search_memory(
    conn: &Connection,
    query: &MemorySearchQuery,
) -> anyhow::Result<Vec<MemorySearchResult>> {
    let limit = if query.limit == 0 { 20 } else { query.limit };
    let items = if let Some(text) = query
        .query
        .as_deref()
        .filter(|text| !text.trim().is_empty())
    {
        search_fts(conn, text, query, limit)?
    } else {
        search_structured(conn, query, limit)?
    };

    let mut results = Vec::new();
    for (item, score, reason) in items {
        if !matches_facet_filters(conn, &item.id, query)? {
            continue;
        }
        results.push(MemorySearchResult {
            answerability: answerability_for_status(&item.status),
            item,
            score,
            reason,
        });
    }

    Ok(results)
}

fn search_fts(
    conn: &Connection,
    text: &str,
    query: &MemorySearchQuery,
    limit: u32,
) -> anyhow::Result<Vec<(MemoryItem, f64, String)>> {
    let fts_query = to_fts_query(text);
    if fts_query.is_empty() {
        return search_structured(conn, query, limit);
    }

    let mut sql = String::from(
        "SELECT i.id, i.memory_type, i.title, i.summary, i.details_json, i.status, i.confidence,
            i.user_verified, i.sensitivity, i.visibility, i.source_id, i.created_at, i.updated_at,
            i.observed_at, i.valid_from, i.valid_until, i.happened_at, i.starts_at, i.ends_at,
            i.due_at, i.completed_at, i.timezone, i.time_precision, i.natural_time_phrase,
            i.recurrence_rule, i.superseded_by, i.deleted_at,
            bm25(memory_fts) AS rank
         FROM memory_fts
         JOIN memory_items i ON i.id = memory_fts.memory_id
         WHERE memory_fts MATCH ?1
           AND i.deleted_at IS NULL
           AND i.status != 'retracted'",
    );
    if query.memory_type.is_some() {
        sql.push_str(" AND i.memory_type = ?2");
    }
    if query.status.is_some() {
        sql.push_str(if query.memory_type.is_some() {
            " AND i.status = ?3"
        } else {
            " AND i.status = ?2"
        });
    }
    sql.push_str(" ORDER BY i.user_verified DESC, i.confidence DESC, rank ASC LIMIT ?");
    let limit_param = if query.memory_type.is_some() && query.status.is_some() {
        4
    } else if query.memory_type.is_some() || query.status.is_some() {
        3
    } else {
        2
    };
    sql.push_str(&limit_param.to_string());

    let mut stmt = conn.prepare(&sql)?;
    let mapped = match (&query.memory_type, &query.status) {
        (Some(memory_type), Some(status)) => stmt.query_map(
            params![fts_query, memory_type, status, limit],
            memory_item_with_rank,
        )?,
        (Some(memory_type), None) => stmt.query_map(
            params![fts_query, memory_type, limit],
            memory_item_with_rank,
        )?,
        (None, Some(status)) => {
            stmt.query_map(params![fts_query, status, limit], memory_item_with_rank)?
        }
        (None, None) => stmt.query_map(params![fts_query, limit], memory_item_with_rank)?,
    };

    let mut out = Vec::new();
    for row in mapped {
        let (item, rank) = row?;
        let mut score = item.confidence + if item.user_verified { 0.2 } else { 0.0 };
        score += 1.0 / (1.0 + rank.abs());
        out.push((item, score, "FTS match with structured filters".into()));
    }
    Ok(out)
}

fn search_structured(
    conn: &Connection,
    query: &MemorySearchQuery,
    limit: u32,
) -> anyhow::Result<Vec<(MemoryItem, f64, String)>> {
    let mut sql = String::from(
        "SELECT id, memory_type, title, summary, details_json, status, confidence, user_verified,
            sensitivity, visibility, source_id, created_at, updated_at, observed_at, valid_from,
            valid_until, happened_at, starts_at, ends_at, due_at, completed_at, timezone,
            time_precision, natural_time_phrase, recurrence_rule, superseded_by, deleted_at
         FROM memory_items
         WHERE deleted_at IS NULL AND status != 'retracted'",
    );
    if query.memory_type.is_some() {
        sql.push_str(" AND memory_type = ?1");
    }
    if query.status.is_some() {
        sql.push_str(if query.memory_type.is_some() {
            " AND status = ?2"
        } else {
            " AND status = ?1"
        });
    }
    let limit_param = if query.memory_type.is_some() && query.status.is_some() {
        3
    } else if query.memory_type.is_some() || query.status.is_some() {
        2
    } else {
        1
    };
    sql.push_str(" ORDER BY user_verified DESC, confidence DESC, observed_at DESC LIMIT ?");
    sql.push_str(&limit_param.to_string());

    let mut stmt = conn.prepare(&sql)?;
    let mapped = match (&query.memory_type, &query.status) {
        (Some(memory_type), Some(status)) => stmt.query_map(
            params![memory_type, status, limit],
            db::memory_item_from_row,
        )?,
        (Some(memory_type), None) => {
            stmt.query_map(params![memory_type, limit], db::memory_item_from_row)?
        }
        (None, Some(status)) => stmt.query_map(params![status, limit], db::memory_item_from_row)?,
        (None, None) => stmt.query_map(params![limit], db::memory_item_from_row)?,
    };

    let mut out = Vec::new();
    for item in mapped {
        let item = item?;
        let score = item.confidence + if item.user_verified { 0.2 } else { 0.0 };
        out.push((item, score, "structured filters".into()));
    }
    Ok(out)
}

fn memory_item_with_rank(row: &Row<'_>) -> rusqlite::Result<(MemoryItem, f64)> {
    Ok((db::memory_item_from_row(row)?, row.get::<_, f64>(27)?))
}

pub fn answerability_for_status(status: &str) -> String {
    match status {
        "completed" => "confirmed",
        "planned" | "active" => "planned_only",
        "uncertain" => "ambiguous",
        "retracted" => "contradicted",
        "cancelled" | "missed" => "contradicted",
        _ => "inferred",
    }
    .to_string()
}

fn matches_facet_filters(
    conn: &Connection,
    memory_id: &str,
    query: &MemorySearchQuery,
) -> anyhow::Result<bool> {
    if query.domain.is_none() && query.facet_type.is_none() && query.label.is_none() {
        return Ok(true);
    }

    let mut sql = String::from("SELECT count(*) FROM memory_facets WHERE memory_id = ?");
    let mut params_vec = vec![memory_id.to_string()];
    if let Some(domain) = &query.domain {
        sql.push_str(" AND domain = ?");
        params_vec.push(domain.clone());
    }
    if let Some(facet_type) = &query.facet_type {
        sql.push_str(" AND facet_type = ?");
        params_vec.push(facet_type.clone());
    }
    if let Some(label) = &query.label {
        sql.push_str(" AND label = ?");
        params_vec.push(label.clone());
    }
    let count: i64 = conn.query_row(&sql, rusqlite::params_from_iter(params_vec), |row| {
        row.get(0)
    })?;
    Ok(count > 0)
}

pub fn to_fts_query(text: &str) -> String {
    let terms: Vec<String> = text
        .split(|ch: char| !ch.is_alphanumeric())
        .map(str::trim)
        .filter(|term| term.len() > 2)
        .map(|term| format!("\"{}\"", term.replace('"', "")))
        .take(8)
        .collect();
    terms.join(" OR ")
}
