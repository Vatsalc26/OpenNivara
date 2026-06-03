use crate::workspace_map::db::{
    get_entries_for_run, get_latest_entry_by_path, get_latest_scan_run_id, search_latest_entries,
    MapEntry,
};
use rusqlite::Connection;

/// Queries the latest completed scan entries matching a search term.
pub fn search_map(conn: &Connection, query: &str) -> anyhow::Result<Vec<MapEntry>> {
    search_latest_entries(conn, query)
}

/// Inspects single node details by target path.
pub fn get_node_by_path(conn: &Connection, path: &str) -> anyhow::Result<Option<MapEntry>> {
    get_node_by_path_helper(conn, path)
}

fn get_node_by_path_helper(conn: &Connection, path: &str) -> anyhow::Result<Option<MapEntry>> {
    get_latest_entry_by_path(conn, path)
}

/// Retrieves list of nodes for drawing tree graphics, filtered optionally by maximum depth limits.
pub fn get_tree_entries(
    conn: &Connection,
    max_depth: Option<u32>,
) -> anyhow::Result<Vec<MapEntry>> {
    let run_id = match get_latest_scan_run_id(conn)? {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let entries = get_entries_for_run(conn, run_id)?;

    if let Some(limit) = max_depth {
        let filtered = entries
            .into_iter()
            .filter(|entry| entry.depth <= limit)
            .collect();
        Ok(filtered)
    } else {
        Ok(entries)
    }
}
