pub mod classifier;
pub mod config;
pub mod db;
pub mod formatter;
pub mod scanner;
pub mod search;

use std::path::PathBuf;

/// Gets the absolute path of the SQLite database by reading map.toml first.
pub fn get_db_path() -> anyhow::Result<PathBuf> {
    let map_config = config::read_map()?;
    db::get_db_path(&map_config.database.filename)
}

/// Triggers recursive workspace scanning and writes the indexed metadata to SQLite.
pub fn scan_workspace() -> anyhow::Result<String> {
    let map_config = config::read_map()?;
    let db_filename = &map_config.database.filename;

    let mut conn = db::establish_connection(db_filename)?;

    // Choose the first allowed root as the primary scan anchor path
    let root_path = map_config
        .roots
        .allowed_roots
        .first()
        .cloned()
        .unwrap_or_else(|| ".".to_string());

    let run_id = db::start_scan_run(&mut conn, &root_path)?;

    // Scan allowed roots recursively in pure Rust (offline scan, no LLM calls)
    let scanned = scanner::scan_allowed_roots(&map_config)?;

    let mut total_entries = 0;
    let mut total_files = 0;
    let mut total_dirs = 0;
    let mut total_blocked = 0;
    let mut total_ignored = 0;

    for entry in &scanned {
        total_entries += 1;
        if entry.kind == "dir" {
            total_dirs += 1;
        } else {
            total_files += 1;
        }
        if entry.is_blocked {
            total_blocked += 1;
        }
        if entry.is_ignored {
            total_ignored += 1;
        }
    }

    db::insert_map_entries(&mut conn, run_id, &scanned)?;
    db::finish_scan_run(
        &mut conn,
        run_id,
        total_entries,
        total_files,
        total_dirs,
        total_blocked,
        total_ignored,
    )?;

    let db_path = db::get_db_path(db_filename)?;

    Ok(format!(
        "Workspace map scan complete.\n\n\
         Root:     {}\n\
         Files:    {}\n\
         Folders:  {}\n\
         Blocked:  {}\n\
         Ignored:  {}\n\
         Database: {}",
        root_path,
        total_files,
        total_dirs,
        total_blocked,
        total_ignored,
        db_path.display()
    ))
}

/// Generates a readable CLI text block summarizing categories and scan run stats.
pub fn render_summary() -> anyhow::Result<String> {
    let map_config = config::read_map()?;
    let conn = db::establish_connection(&map_config.database.filename)?;

    let summary =
        match db::get_latest_scan_summary(&conn)? {
            Some(s) => s,
            None => return Ok(
                "No workspace scan run has completed yet. Please run `opennivara map-scan` first."
                    .to_string(),
            ),
        };

    let run_id = db::get_latest_scan_run_id(&conn)?.unwrap();
    let cat_counts = db::get_category_counts(&conn, run_id)?;

    let mut cat_breakdown = String::new();
    let mut sorted_cats: Vec<&String> = cat_counts.keys().collect();
    sorted_cats.sort();
    for cat in sorted_cats {
        let count = cat_counts.get(cat).unwrap();
        let icon = formatter::get_icon_for_category(cat, &map_config);
        cat_breakdown.push_str(&format!("  {} {}: {}\n", icon, cat, count));
    }

    // Identify standard landmarks (essential files like Cargo.toml, README.md)
    let mut landmarks = Vec::new();
    if db::get_latest_entry_by_path(&conn, "Cargo.toml")?.is_some() {
        landmarks.push("Cargo.toml");
    }
    if db::get_latest_entry_by_path(&conn, "README.md")?.is_some() {
        landmarks.push("README.md");
    }
    if db::get_latest_entry_by_path(&conn, "src/main.rs")?.is_some() {
        landmarks.push("src/main.rs");
    }

    Ok(format!(
        "=== Workspace Map Summary ===\n\
         Root Directory:  {}\n\
         Last Scan Time:  {}\n\
         Total Files:     {}\n\
         Total Folders:   {}\n\
         Blocked Items:   {}\n\
         Ignored Items:   {}\n\n\
         Category Breakdown:\n\
         {}\n\
         Key Landmarks Identified: {:?}\n\
         =============================",
        summary.root_path,
        summary.finished_at.unwrap_or_else(|| "Unknown".to_string()),
        summary.total_files,
        summary.total_dirs,
        summary.total_blocked,
        summary.total_ignored,
        cat_breakdown,
        landmarks
    ))
}

/// Generates tree display for CLI commands.
pub fn render_tree(max_depth: Option<u32>) -> anyhow::Result<String> {
    let map_config = config::read_map()?;
    let conn = db::establish_connection(&map_config.database.filename)?;

    let entries = search::get_tree_entries(&conn, max_depth)?;
    Ok(formatter::format_tree_string(&entries, &map_config))
}

/// Run search keyword and formats text results for CLI.
pub fn search_entries(query: &str) -> anyhow::Result<String> {
    let map_config = config::read_map()?;
    let conn = db::establish_connection(&map_config.database.filename)?;

    let matches = search::search_map(&conn, query)?;
    if matches.is_empty() {
        return Ok(format!("No entries found matching: '{}'", query));
    }

    let mut output = format!("Matches for '{}':\n", query);
    for entry in matches {
        let icon = formatter::get_icon_for_category(&entry.category, &map_config);
        output.push_str(&format!("  {} {}\n", icon, entry.path));
    }
    Ok(output)
}

/// Formats detailed node attributes for CLI.
pub fn get_entry_info(path_str: &str) -> anyhow::Result<String> {
    let map_config = config::read_map()?;
    let conn = db::establish_connection(&map_config.database.filename)?;

    let node = match search::get_node_by_path(&conn, path_str)? {
        Some(n) => n,
        None => {
            return Ok(format!(
                "Path '{}' not found in workspace map database.",
                path_str
            ))
        }
    };

    Ok(format!(
        "Path:      {}\n\
         Name:      {}\n\
         Kind:      {}\n\
         Category:  {}\n\
         Size:      {} bytes\n\
         Modified:  {}\n\
         Blocked:   {}\n\
         Ignored:   {}\n\
         Symlink:   {}",
        node.path,
        node.name,
        node.kind,
        node.category,
        node.size_bytes.unwrap_or(0),
        node.modified_at.unwrap_or_else(|| "Unknown".to_string()),
        node.is_blocked,
        node.is_ignored,
        node.is_symlink
    ))
}

// ==========================================
// Gemini Tool Handlers (Returns JSON values)
// ==========================================

pub fn tool_map_summary(config_filename: &str) -> serde_json::Value {
    let conn = match db::establish_connection(config_filename) {
        Ok(c) => c,
        Err(e) => {
            return serde_json::json!({ "error": format!("Database connection error: {}", e) })
        }
    };

    let summary = match db::get_latest_scan_summary(&conn) {
        Ok(Some(s)) => s,
        Ok(None) => {
            return serde_json::json!({ "error": "No scan summary found. Workspace map has not been scanned yet." })
        }
        Err(e) => return serde_json::json!({ "error": format!("Database query error: {}", e) }),
    };

    let run_id = db::get_latest_scan_run_id(&conn).unwrap().unwrap();
    let cat_counts = db::get_category_counts(&conn, run_id).unwrap_or_default();

    let mut landmarks = Vec::new();
    if db::get_latest_entry_by_path(&conn, "Cargo.toml")
        .unwrap_or(None)
        .is_some()
    {
        landmarks.push("Cargo.toml");
    }
    if db::get_latest_entry_by_path(&conn, "README.md")
        .unwrap_or(None)
        .is_some()
    {
        landmarks.push("README.md");
    }
    if db::get_latest_entry_by_path(&conn, "src/main.rs")
        .unwrap_or(None)
        .is_some()
    {
        landmarks.push("src/main.rs");
    }

    serde_json::json!({
        "status": "success",
        "last_scan": summary.finished_at,
        "root_paths": [summary.root_path],
        "total_files": summary.total_files,
        "total_dirs": summary.total_dirs,
        "total_blocked": summary.total_blocked,
        "total_ignored": summary.total_ignored,
        "categories": cat_counts,
        "landmarks": landmarks
    })
}

pub fn tool_map_tree(config_filename: &str, depth: Option<u32>) -> serde_json::Value {
    let map_config = match config::read_map() {
        Ok(c) => c,
        Err(e) => return serde_json::json!({ "error": format!("Config error: {}", e) }),
    };

    let conn = match db::establish_connection(config_filename) {
        Ok(c) => c,
        Err(e) => {
            return serde_json::json!({ "error": format!("Database connection error: {}", e) })
        }
    };

    let entries = match search::get_tree_entries(&conn, depth) {
        Ok(v) => v,
        Err(e) => return serde_json::json!({ "error": format!("Database query error: {}", e) }),
    };

    let tree_txt = formatter::format_tree_string(&entries, &map_config);
    serde_json::json!({
        "status": "success",
        "tree": tree_txt
    })
}

pub fn tool_map_search(config_filename: &str, query: &str) -> serde_json::Value {
    let conn = match db::establish_connection(config_filename) {
        Ok(c) => c,
        Err(e) => {
            return serde_json::json!({ "error": format!("Database connection error: {}", e) })
        }
    };

    let matches = match search::search_map(&conn, query) {
        Ok(v) => v,
        Err(e) => return serde_json::json!({ "error": format!("Database search error: {}", e) }),
    };

    let results: Vec<serde_json::Value> = matches
        .iter()
        .map(|entry| {
            serde_json::json!({
                "path": entry.path,
                "name": entry.name,
                "kind": entry.kind,
                "category": entry.category,
                "size_bytes": entry.size_bytes,
                "is_blocked": entry.is_blocked,
                "is_ignored": entry.is_ignored
            })
        })
        .collect();

    serde_json::json!({
        "status": "success",
        "query": query,
        "matches": results
    })
}

pub fn tool_map_get_node(config_filename: &str, path_str: &str) -> serde_json::Value {
    let conn = match db::establish_connection(config_filename) {
        Ok(c) => c,
        Err(e) => {
            return serde_json::json!({ "error": format!("Database connection error: {}", e) })
        }
    };

    match search::get_node_by_path(&conn, path_str) {
        Ok(Some(entry)) => serde_json::json!({
            "status": "success",
            "path": entry.path,
            "name": entry.name,
            "kind": entry.kind,
            "category": entry.category,
            "extension": entry.extension,
            "mime_type": entry.mime_type,
            "size_bytes": entry.size_bytes,
            "modified_at": entry.modified_at,
            "depth": entry.depth,
            "is_blocked": entry.is_blocked,
            "is_ignored": entry.is_ignored,
            "is_symlink": entry.is_symlink,
            "hash": entry.hash
        }),
        Ok(None) => serde_json::json!({
            "status": "error",
            "message": format!("Path '{}' not found in workspace map.", path_str)
        }),
        Err(e) => serde_json::json!({ "error": format!("Database query error: {}", e) }),
    }
}
