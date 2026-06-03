use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashSet, VecDeque};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemoryGraphNode {
    pub id: String,
    pub node_type: String,
    pub source_table: String,
    pub source_id: String,
    pub label: String,
    pub properties_json: String,
    pub sensitivity: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemoryGraphEdge {
    pub id: String,
    pub from_node_id: String,
    pub edge_type: String,
    pub to_node_id: String,
    pub weight: f64,
    pub confidence: f64,
    pub source_memory_id: Option<String>,
    pub properties_json: String,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemoryGraphContext {
    pub focus_node_id: String,
    pub nodes: Vec<MemoryGraphNode>,
    pub edges: Vec<MemoryGraphEdge>,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MemoryGraphStatus {
    pub node_count: u32,
    pub edge_count: u32,
    pub index_count: u32,
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateGraphEdge {
    pub from_node_id: String,
    pub edge_type: String,
    pub to_node_id: String,
    pub weight: f64,
    pub confidence: f64,
    pub source_memory_id: Option<String>,
    pub properties_json: String,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
}

pub fn graph_node_id(source_table: &str, source_id: &str) -> String {
    format!("node_{source_table}_{}", source_id.replace('-', "_"))
}

pub fn graph_rebuild_from_sqlite(conn: &Connection) -> anyhow::Result<()> {
    conn.execute("DELETE FROM memory_graph_edge_index", [])?;
    conn.execute("DELETE FROM memory_graph_edges", [])?;
    conn.execute("DELETE FROM memory_graph_nodes", [])?;

    let now = Utc::now().to_rfc3339();
    insert_memory_nodes(conn, &now)?;
    insert_source_nodes(conn, &now)?;
    insert_entity_nodes(conn, &now)?;
    insert_facet_nodes(conn, &now)?;
    insert_task_nodes(conn, &now)?;
    insert_relationship_edges(conn, &now)?;
    graph_rebuild_edge_index(conn)?;
    Ok(())
}

pub fn graph_upsert_memory_item(conn: &Connection, _memory_id: &str) -> anyhow::Result<()> {
    graph_rebuild_from_sqlite(conn)
}

pub fn graph_upsert_entity(conn: &Connection, _entity_id: &str) -> anyhow::Result<()> {
    graph_rebuild_from_sqlite(conn)
}

pub fn graph_upsert_facet(conn: &Connection, _facet_id: &str) -> anyhow::Result<()> {
    graph_rebuild_from_sqlite(conn)
}

pub fn graph_validate_consistency(conn: &Connection) -> anyhow::Result<Vec<String>> {
    let mut errors = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT e.id, e.from_node_id, e.to_node_id
         FROM memory_graph_edges e
         LEFT JOIN memory_graph_nodes from_node ON from_node.id = e.from_node_id
         LEFT JOIN memory_graph_nodes to_node ON to_node.id = e.to_node_id
         WHERE from_node.id IS NULL OR to_node.id IS NULL",
    )?;
    let missing_edges = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    for edge in missing_edges {
        let (edge_id, from_node_id, to_node_id) = edge?;
        errors.push(format!(
            "edge {edge_id} references missing node {from_node_id} or {to_node_id}"
        ));
    }

    let mut stmt = conn.prepare(
        "SELECT id, source_table, source_id FROM memory_graph_nodes ORDER BY source_table, source_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    for row in rows {
        let (node_id, source_table, source_id) = row?;
        if !source_exists(conn, &source_table, &source_id)? {
            errors.push(format!(
                "node {node_id} points to missing {source_table}:{source_id}"
            ));
        }
    }

    Ok(errors)
}

pub fn graph_status(conn: &Connection) -> anyhow::Result<MemoryGraphStatus> {
    Ok(MemoryGraphStatus {
        node_count: count_table(conn, "memory_graph_nodes")?,
        edge_count: count_table(conn, "memory_graph_edges")?,
        index_count: count_table(conn, "memory_graph_edge_index")?,
        validation_errors: graph_validate_consistency(conn)?,
    })
}

pub fn graph_find_related_memories(
    conn: &Connection,
    memory_id: &str,
    limit: u32,
) -> anyhow::Result<Vec<MemoryGraphNode>> {
    let node_id = graph_node_id("memory_items", memory_id);
    graph_neighbors(conn, &node_id, limit.max(1))
}

pub fn graph_memory_context(
    conn: &Connection,
    memory_id: &str,
    max_depth: u32,
) -> anyhow::Result<MemoryGraphContext> {
    graph_context(conn, &graph_node_id("memory_items", memory_id), max_depth)
}

pub fn graph_entity_context(
    conn: &Connection,
    entity_id: &str,
    max_depth: u32,
) -> anyhow::Result<MemoryGraphContext> {
    graph_context(conn, &graph_node_id("entities", entity_id), max_depth)
}

pub fn graph_neighbors(
    conn: &Connection,
    node_id: &str,
    limit: u32,
) -> anyhow::Result<Vec<MemoryGraphNode>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT n.id, n.node_type, n.source_table, n.source_id, n.label,
            n.properties_json, n.sensitivity, n.updated_at
         FROM memory_graph_edges e
         JOIN memory_graph_nodes n
           ON n.id = CASE WHEN e.from_node_id = ?1 THEN e.to_node_id ELSE e.from_node_id END
         WHERE e.from_node_id = ?1 OR e.to_node_id = ?1
         ORDER BY e.confidence DESC, e.weight DESC, n.label ASC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![node_id, limit], graph_node_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn graph_shortest_path(
    conn: &Connection,
    from_node_id: &str,
    to_node_id: &str,
    max_depth: u32,
) -> anyhow::Result<MemoryGraphContext> {
    if from_node_id == to_node_id {
        return graph_context(conn, from_node_id, 0);
    }

    let mut queue = VecDeque::from([(from_node_id.to_string(), Vec::<MemoryGraphEdge>::new())]);
    let mut seen = HashSet::from([from_node_id.to_string()]);

    while let Some((node_id, path)) = queue.pop_front() {
        if path.len() as u32 >= max_depth {
            continue;
        }
        for edge in edges_for_node(conn, &node_id)? {
            let next = if edge.from_node_id == node_id {
                edge.to_node_id.clone()
            } else {
                edge.from_node_id.clone()
            };
            if !seen.insert(next.clone()) {
                continue;
            }
            let mut next_path = path.clone();
            next_path.push(edge);
            if next == to_node_id {
                let mut node_ids = HashSet::new();
                for edge in &next_path {
                    node_ids.insert(edge.from_node_id.clone());
                    node_ids.insert(edge.to_node_id.clone());
                }
                let nodes = load_nodes(conn, &node_ids)?;
                return Ok(MemoryGraphContext {
                    focus_node_id: from_node_id.to_string(),
                    nodes,
                    edges: next_path,
                    depth: max_depth,
                });
            }
            queue.push_back((next, next_path));
        }
    }

    Ok(MemoryGraphContext {
        focus_node_id: from_node_id.to_string(),
        nodes: vec![],
        edges: vec![],
        depth: max_depth,
    })
}

pub fn graph_find_entity_mentions(
    conn: &Connection,
    mention: &str,
    limit: u32,
) -> anyhow::Result<Vec<MemoryGraphNode>> {
    let needle = format!("%{}%", mention.to_lowercase());
    let mut stmt = conn.prepare(
        "SELECT id, node_type, source_table, source_id, label, properties_json, sensitivity, updated_at
         FROM memory_graph_nodes
         WHERE node_type = 'entity' AND lower(label) LIKE ?1
         ORDER BY label ASC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![needle, limit.max(1)], graph_node_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn graph_add_edge(
    conn: &Connection,
    input: &CreateGraphEdge,
) -> anyhow::Result<MemoryGraphEdge> {
    let now = Utc::now().to_rfc3339();
    let edge = MemoryGraphEdge {
        id: format!("edge_{}", Uuid::new_v4()),
        from_node_id: input.from_node_id.clone(),
        edge_type: input.edge_type.clone(),
        to_node_id: input.to_node_id.clone(),
        weight: input.weight,
        confidence: input.confidence,
        source_memory_id: input.source_memory_id.clone(),
        properties_json: input.properties_json.clone(),
        valid_from: input.valid_from.clone(),
        valid_until: input.valid_until.clone(),
        created_at: now.clone(),
        updated_at: now,
    };
    insert_edge(conn, &edge)?;
    graph_rebuild_edge_index(conn)?;
    Ok(edge)
}

fn graph_context(
    conn: &Connection,
    focus_node_id: &str,
    max_depth: u32,
) -> anyhow::Result<MemoryGraphContext> {
    let mut seen_nodes = HashSet::from([focus_node_id.to_string()]);
    let mut seen_edges = HashSet::new();
    let mut frontier = VecDeque::from([(focus_node_id.to_string(), 0_u32)]);
    let mut edges = Vec::new();

    while let Some((node_id, depth)) = frontier.pop_front() {
        if depth >= max_depth {
            continue;
        }
        for edge in edges_for_node(conn, &node_id)? {
            if seen_edges.insert(edge.id.clone()) {
                let next = if edge.from_node_id == node_id {
                    edge.to_node_id.clone()
                } else {
                    edge.from_node_id.clone()
                };
                if seen_nodes.insert(next.clone()) {
                    frontier.push_back((next, depth + 1));
                }
                edges.push(edge);
            }
        }
    }

    let nodes = load_nodes(conn, &seen_nodes)?;
    Ok(MemoryGraphContext {
        focus_node_id: focus_node_id.to_string(),
        nodes,
        edges,
        depth: max_depth,
    })
}

fn insert_memory_nodes(conn: &Connection, now: &str) -> anyhow::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, memory_type, title, summary, sensitivity, updated_at
         FROM memory_items WHERE deleted_at IS NULL",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(MemoryGraphNode {
            id: graph_node_id("memory_items", &row.get::<_, String>(0)?),
            node_type: "memory".into(),
            source_table: "memory_items".into(),
            source_id: row.get(0)?,
            label: row.get(2)?,
            properties_json: serde_json::json!({
                "memory_type": row.get::<_, String>(1)?,
                "summary": row.get::<_, String>(3)?,
            })
            .to_string(),
            sensitivity: row.get(4)?,
            updated_at: row.get::<_, String>(5).unwrap_or_else(|_| now.to_string()),
        })
    })?;
    for node in rows {
        insert_node(conn, &node?)?;
    }
    Ok(())
}

fn insert_source_nodes(conn: &Connection, now: &str) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("SELECT id, source_type, sensitivity FROM memory_sources")?;
    let rows = stmt.query_map([], |row| {
        let source_type: String = row.get(1)?;
        Ok(MemoryGraphNode {
            id: graph_node_id("memory_sources", &row.get::<_, String>(0)?),
            node_type: "source".into(),
            source_table: "memory_sources".into(),
            source_id: row.get(0)?,
            label: source_type.clone(),
            properties_json: serde_json::json!({ "source_type": source_type }).to_string(),
            sensitivity: row.get(2)?,
            updated_at: now.to_string(),
        })
    })?;
    for node in rows {
        insert_node(conn, &node?)?;
    }
    Ok(())
}

fn insert_entity_nodes(conn: &Connection, now: &str) -> anyhow::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, entity_type, display_name, details_json, updated_at
         FROM entities WHERE deleted_at IS NULL",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(MemoryGraphNode {
            id: graph_node_id("entities", &row.get::<_, String>(0)?),
            node_type: "entity".into(),
            source_table: "entities".into(),
            source_id: row.get(0)?,
            label: row.get(2)?,
            properties_json: serde_json::json!({
                "entity_type": row.get::<_, String>(1)?,
                "details": row.get::<_, String>(3)?,
            })
            .to_string(),
            sensitivity: "normal".into(),
            updated_at: row.get::<_, String>(4).unwrap_or_else(|_| now.to_string()),
        })
    })?;
    for node in rows {
        insert_node(conn, &node?)?;
    }
    Ok(())
}

fn insert_facet_nodes(conn: &Connection, _now: &str) -> anyhow::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, domain, facet_type, label, details_json, sensitivity, updated_at
         FROM memory_facets",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(MemoryGraphNode {
            id: graph_node_id("memory_facets", &row.get::<_, String>(0)?),
            node_type: "facet".into(),
            source_table: "memory_facets".into(),
            source_id: row.get(0)?,
            label: row.get(3)?,
            properties_json: serde_json::json!({
                "domain": row.get::<_, String>(1)?,
                "facet_type": row.get::<_, String>(2)?,
                "details": row.get::<_, String>(4)?,
            })
            .to_string(),
            sensitivity: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    for node in rows {
        insert_node(conn, &node?)?;
    }
    Ok(())
}

fn insert_task_nodes(conn: &Connection, now: &str) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("SELECT memory_id, task_type, status FROM tasks")?;
    let rows = stmt.query_map([], |row| {
        let memory_id: String = row.get(0)?;
        Ok(MemoryGraphNode {
            id: graph_node_id("tasks", &memory_id),
            node_type: "task".into(),
            source_table: "tasks".into(),
            source_id: memory_id,
            label: row.get::<_, String>(1)?,
            properties_json: serde_json::json!({ "status": row.get::<_, String>(2)? }).to_string(),
            sensitivity: "normal".into(),
            updated_at: now.to_string(),
        })
    })?;
    for node in rows {
        insert_node(conn, &node?)?;
    }
    Ok(())
}

fn insert_relationship_edges(conn: &Connection, now: &str) -> anyhow::Result<()> {
    let mut stmt = conn
        .prepare("SELECT id, source_id, confidence FROM memory_items WHERE deleted_at IS NULL")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, f64>(2)?,
        ))
    })?;
    for row in rows {
        let (memory_id, source_id, confidence) = row?;
        insert_typed_edge(
            conn,
            EdgeSpec {
                from_node_id: graph_node_id("memory_items", &memory_id),
                edge_type: "derived_from".into(),
                to_node_id: graph_node_id("memory_sources", &source_id),
                weight: 0.7,
                confidence,
                source_memory_id: Some(memory_id),
                now: now.into(),
            },
        )?;
    }

    let mut stmt = conn.prepare("SELECT id, memory_id, confidence FROM memory_facets")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, f64>(2)?,
        ))
    })?;
    for row in rows {
        let (facet_id, memory_id, confidence) = row?;
        insert_typed_edge(
            conn,
            EdgeSpec {
                from_node_id: graph_node_id("memory_items", &memory_id),
                edge_type: "has_facet".into(),
                to_node_id: graph_node_id("memory_facets", &facet_id),
                weight: 1.0,
                confidence,
                source_memory_id: Some(memory_id),
                now: now.into(),
            },
        )?;
    }

    let mut stmt =
        conn.prepare("SELECT memory_id, entity_id, role, confidence FROM memory_entities")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, f64>(3)?,
        ))
    })?;
    for row in rows {
        let (memory_id, entity_id, role, confidence) = row?;
        insert_typed_edge(
            conn,
            EdgeSpec {
                from_node_id: graph_node_id("memory_items", &memory_id),
                edge_type: format!("mentions:{role}"),
                to_node_id: graph_node_id("entities", &entity_id),
                weight: 0.9,
                confidence,
                source_memory_id: Some(memory_id),
                now: now.into(),
            },
        )?;
    }

    let mut stmt = conn.prepare("SELECT memory_id FROM tasks")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    for memory_id in rows {
        let memory_id = memory_id?;
        insert_typed_edge(
            conn,
            EdgeSpec {
                from_node_id: graph_node_id("tasks", &memory_id),
                edge_type: "tracks_memory".into(),
                to_node_id: graph_node_id("memory_items", &memory_id),
                weight: 1.0,
                confidence: 1.0,
                source_memory_id: Some(memory_id),
                now: now.into(),
            },
        )?;
    }

    Ok(())
}

struct EdgeSpec {
    from_node_id: String,
    edge_type: String,
    to_node_id: String,
    weight: f64,
    confidence: f64,
    source_memory_id: Option<String>,
    now: String,
}

fn insert_typed_edge(conn: &Connection, spec: EdgeSpec) -> anyhow::Result<()> {
    insert_edge(
        conn,
        &MemoryGraphEdge {
            id: format!(
                "edge_{}_{}_{}",
                spec.from_node_id.replace('-', "_"),
                spec.edge_type.replace(':', "_"),
                spec.to_node_id.replace('-', "_")
            ),
            from_node_id: spec.from_node_id,
            edge_type: spec.edge_type,
            to_node_id: spec.to_node_id,
            weight: spec.weight,
            confidence: spec.confidence,
            source_memory_id: spec.source_memory_id,
            properties_json: "{}".into(),
            valid_from: None,
            valid_until: None,
            created_at: spec.now.clone(),
            updated_at: spec.now,
        },
    )
}

fn graph_rebuild_edge_index(conn: &Connection) -> anyhow::Result<()> {
    conn.execute("DELETE FROM memory_graph_edge_index", [])?;
    let now = Utc::now().to_rfc3339();
    let mut stmt = conn.prepare(
        "SELECT from_node_id, to_node_id, weight FROM memory_graph_edges ORDER BY confidence DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, f64>(2)?,
        ))
    })?;
    for row in rows {
        let (from_node_id, to_node_id, weight) = row?;
        conn.execute(
            "INSERT INTO memory_graph_edge_index (
                from_node_id, to_node_id, min_depth, path_count, strongest_weight, updated_at
             ) VALUES (?1, ?2, 1, 1, ?3, ?4)
             ON CONFLICT(from_node_id, to_node_id) DO UPDATE SET
                path_count = path_count + 1,
                strongest_weight = max(strongest_weight, excluded.strongest_weight),
                updated_at = excluded.updated_at",
            params![from_node_id, to_node_id, weight, now],
        )?;
    }
    Ok(())
}

fn insert_node(conn: &Connection, node: &MemoryGraphNode) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO memory_graph_nodes (
            id, node_type, source_table, source_id, label, properties_json, sensitivity, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
            node_type = excluded.node_type,
            source_table = excluded.source_table,
            source_id = excluded.source_id,
            label = excluded.label,
            properties_json = excluded.properties_json,
            sensitivity = excluded.sensitivity,
            updated_at = excluded.updated_at",
        params![
            &node.id,
            &node.node_type,
            &node.source_table,
            &node.source_id,
            &node.label,
            &node.properties_json,
            &node.sensitivity,
            &node.updated_at,
        ],
    )?;
    Ok(())
}

fn insert_edge(conn: &Connection, edge: &MemoryGraphEdge) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO memory_graph_edges (
            id, from_node_id, edge_type, to_node_id, weight, confidence, source_memory_id,
            properties_json, valid_from, valid_until, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(id) DO UPDATE SET
            from_node_id = excluded.from_node_id,
            edge_type = excluded.edge_type,
            to_node_id = excluded.to_node_id,
            weight = excluded.weight,
            confidence = excluded.confidence,
            source_memory_id = excluded.source_memory_id,
            properties_json = excluded.properties_json,
            valid_from = excluded.valid_from,
            valid_until = excluded.valid_until,
            updated_at = excluded.updated_at",
        params![
            &edge.id,
            &edge.from_node_id,
            &edge.edge_type,
            &edge.to_node_id,
            edge.weight,
            edge.confidence,
            &edge.source_memory_id,
            &edge.properties_json,
            &edge.valid_from,
            &edge.valid_until,
            &edge.created_at,
            &edge.updated_at,
        ],
    )?;
    Ok(())
}

fn edges_for_node(conn: &Connection, node_id: &str) -> anyhow::Result<Vec<MemoryGraphEdge>> {
    let mut stmt = conn.prepare(
        "SELECT id, from_node_id, edge_type, to_node_id, weight, confidence, source_memory_id,
            properties_json, valid_from, valid_until, created_at, updated_at
         FROM memory_graph_edges
         WHERE from_node_id = ?1 OR to_node_id = ?1
         ORDER BY confidence DESC, weight DESC",
    )?;
    let rows = stmt.query_map([node_id], graph_edge_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn load_nodes(
    conn: &Connection,
    node_ids: &HashSet<String>,
) -> anyhow::Result<Vec<MemoryGraphNode>> {
    let mut nodes = Vec::new();
    for node_id in node_ids {
        if let Some(node) = conn
            .query_row(
                "SELECT id, node_type, source_table, source_id, label, properties_json, sensitivity, updated_at
                 FROM memory_graph_nodes WHERE id = ?1",
                [node_id],
                graph_node_from_row,
            )
            .optional()?
        {
            nodes.push(node);
        }
    }
    nodes.sort_by(|left, right| left.label.cmp(&right.label));
    Ok(nodes)
}

fn source_exists(conn: &Connection, source_table: &str, source_id: &str) -> anyhow::Result<bool> {
    let sql = match source_table {
        "memory_items" => "SELECT count(*) FROM memory_items WHERE id = ?1 AND deleted_at IS NULL",
        "memory_sources" => "SELECT count(*) FROM memory_sources WHERE id = ?1",
        "entities" => "SELECT count(*) FROM entities WHERE id = ?1 AND deleted_at IS NULL",
        "memory_facets" => "SELECT count(*) FROM memory_facets WHERE id = ?1",
        "tasks" => "SELECT count(*) FROM tasks WHERE memory_id = ?1",
        _ => return Ok(false),
    };
    let count: i64 = conn.query_row(sql, [source_id], |row| row.get(0))?;
    Ok(count > 0)
}

fn count_table(conn: &Connection, table: &str) -> anyhow::Result<u32> {
    let sql = format!("SELECT count(*) FROM {table}");
    conn.query_row(&sql, [], |row| row.get(0))
        .map_err(Into::into)
}

fn graph_node_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryGraphNode> {
    Ok(MemoryGraphNode {
        id: row.get(0)?,
        node_type: row.get(1)?,
        source_table: row.get(2)?,
        source_id: row.get(3)?,
        label: row.get(4)?,
        properties_json: row.get(5)?,
        sensitivity: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn graph_edge_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryGraphEdge> {
    Ok(MemoryGraphEdge {
        id: row.get(0)?,
        from_node_id: row.get(1)?,
        edge_type: row.get(2)?,
        to_node_id: row.get(3)?,
        weight: row.get(4)?,
        confidence: row.get(5)?,
        source_memory_id: row.get(6)?,
        properties_json: row.get(7)?,
        valid_from: row.get(8)?,
        valid_until: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}
