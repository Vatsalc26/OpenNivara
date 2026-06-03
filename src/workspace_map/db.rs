use rusqlite::{params, Connection};
use std::path::PathBuf;

/// Struct representing metadata details of a single mapped folder or landmark file.
#[derive(Debug, Clone)]
pub struct MapEntry {
    pub path: String,
    pub name: String,
    pub parent_path: Option<String>,
    pub kind: String,     // "file" or "dir"
    pub category: String, // e.g. "directory", "rust", "config", etc.
    pub extension: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<String>,
    pub depth: u32,
    pub is_hidden: bool,
    pub is_blocked: bool,
    pub is_ignored: bool,
    pub is_symlink: bool,
    pub hash: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScanSummary {
    pub root_path: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub total_entries: u32,
    pub total_files: u32,
    pub total_dirs: u32,
    pub total_blocked: u32,
    pub total_ignored: u32,
    pub status: String,
}

/// Resolves standard folder location for the SQLite database.
pub fn get_db_path(filename: &str) -> anyhow::Result<PathBuf> {
    Ok(crate::config_paths::config_dir()?.join(filename))
}

/// Establishes connection to the SQLite database and executes table setups.
pub fn establish_connection(filename: &str) -> anyhow::Result<Connection> {
    let db_path = get_db_path(filename)?;

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open SQLite database: {}", e))?;

    // Enable WAL journal mode for performance and concurrent read safety
    let _: String = conn.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;

    init_db(&conn)?;

    Ok(conn)
}

fn init_db(conn: &Connection) -> anyhow::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scan_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            root_path TEXT NOT NULL,
            started_at TEXT NOT NULL,
            finished_at TEXT,
            total_entries INTEGER DEFAULT 0,
            total_files INTEGER DEFAULT 0,
            total_dirs INTEGER DEFAULT 0,
            total_blocked INTEGER DEFAULT 0,
            total_ignored INTEGER DEFAULT 0,
            status TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| anyhow::anyhow!("Failed to create scan_runs table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS map_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_run_id INTEGER NOT NULL,
            path TEXT NOT NULL,
            name TEXT NOT NULL,
            parent_path TEXT,
            kind TEXT NOT NULL,
            category TEXT NOT NULL,
            extension TEXT,
            mime_type TEXT,
            size_bytes INTEGER,
            modified_at TEXT,
            depth INTEGER NOT NULL,
            is_hidden INTEGER NOT NULL DEFAULT 0,
            is_blocked INTEGER NOT NULL DEFAULT 0,
            is_ignored INTEGER NOT NULL DEFAULT 0,
            is_symlink INTEGER NOT NULL DEFAULT 0,
            hash TEXT,
            FOREIGN KEY(scan_run_id) REFERENCES scan_runs(id)
        )",
        [],
    )
    .map_err(|e| anyhow::anyhow!("Failed to create map_entries table: {}", e))?;

    // Create indices to speed up common query lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_map_entries_path ON map_entries(path)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_map_entries_name ON map_entries(name)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_map_entries_category ON map_entries(category)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_map_entries_extension ON map_entries(extension)",
        [],
    )?;

    Ok(())
}

/// Inserts a new scan run record to SQLite and returns its ID.
pub fn start_scan_run(conn: &mut Connection, root_path: &str) -> anyhow::Result<i64> {
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "INSERT INTO scan_runs (root_path, started_at, status) VALUES (?1, ?2, 'RUNNING')",
        params![root_path, now],
    )
    .map_err(|e| anyhow::anyhow!("Failed to start scan run: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// Transactional batch inserts of discovered files/directories metadata records into SQLite.
pub fn insert_map_entries(
    conn: &mut Connection,
    run_id: i64,
    entries: &[MapEntry],
) -> anyhow::Result<()> {
    let tx = conn
        .transaction()
        .map_err(|e| anyhow::anyhow!("Failed to start database transaction: {}", e))?;

    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO map_entries (
                scan_run_id, path, name, parent_path, kind, category, 
                extension, mime_type, size_bytes, modified_at, depth, 
                is_hidden, is_blocked, is_ignored, is_symlink, hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            )
            .map_err(|e| anyhow::anyhow!("Failed to prepare map entry insert statement: {}", e))?;

        for entry in entries {
            stmt.execute(params![
                run_id,
                entry.path,
                entry.name,
                entry.parent_path,
                entry.kind,
                entry.category,
                entry.extension,
                entry.mime_type,
                entry.size_bytes,
                entry.modified_at,
                entry.depth,
                entry.is_hidden as i32,
                entry.is_blocked as i32,
                entry.is_ignored as i32,
                entry.is_symlink as i32,
                entry.hash
            ])?;
        }
    }

    tx.commit()
        .map_err(|e| anyhow::anyhow!("Failed to commit database transaction: {}", e))?;

    Ok(())
}

/// Marks a scan run as completed and saves total counts.
pub fn finish_scan_run(
    conn: &mut Connection,
    run_id: i64,
    total_entries: usize,
    total_files: usize,
    total_dirs: usize,
    total_blocked: usize,
    total_ignored: usize,
) -> anyhow::Result<()> {
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "UPDATE scan_runs SET finished_at = ?1, total_entries = ?2, total_files = ?3, \
         total_dirs = ?4, total_blocked = ?5, total_ignored = ?6, status = 'COMPLETED' WHERE id = ?7",
        params![
            now,
            total_entries as i32,
            total_files as i32,
            total_dirs as i32,
            total_blocked as i32,
            total_ignored as i32,
            run_id
        ],
    ).map_err(|e| anyhow::anyhow!("Failed to update scan run summary: {}", e))?;

    Ok(())
}

/// Returns the last scan run ID if available.
pub fn get_latest_scan_run_id(conn: &Connection) -> anyhow::Result<Option<i64>> {
    let mut stmt = conn
        .prepare("SELECT id FROM scan_runs WHERE status = 'COMPLETED' ORDER BY id DESC LIMIT 1")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        Ok(Some(id))
    } else {
        Ok(None)
    }
}

/// Retrieves the details of the latest complete scan run summary.
pub fn get_latest_scan_summary(conn: &Connection) -> anyhow::Result<Option<ScanSummary>> {
    let mut stmt = conn.prepare(
        "SELECT root_path, started_at, finished_at, total_entries, total_files, \
         total_dirs, total_blocked, total_ignored, status FROM scan_runs \
         WHERE status = 'COMPLETED' ORDER BY id DESC LIMIT 1",
    )?;

    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(ScanSummary {
            root_path: row.get(0)?,
            started_at: row.get(1)?,
            finished_at: row.get(2)?,
            total_entries: row.get(3)?,
            total_files: row.get(4)?,
            total_dirs: row.get(5)?,
            total_blocked: row.get(6)?,
            total_ignored: row.get(7)?,
            status: row.get(8)?,
        }))
    } else {
        Ok(None)
    }
}

/// Retrieves category counts for the latest scan.
pub fn get_category_counts(
    conn: &Connection,
    run_id: i64,
) -> anyhow::Result<std::collections::HashMap<String, u32>> {
    let mut stmt = conn.prepare(
        "SELECT category, COUNT(*) FROM map_entries WHERE scan_run_id = ?1 GROUP BY category",
    )?;
    let mut rows = stmt.query(params![run_id])?;
    let mut counts = std::collections::HashMap::new();
    while let Some(row) = rows.next()? {
        let cat: String = row.get(0)?;
        let count: u32 = row.get(1)?;
        counts.insert(cat, count);
    }
    Ok(counts)
}

/// Helper mapping to convert a rusqlite row back to a clean MapEntry structure.
fn row_to_map_entry(row: &rusqlite::Row) -> rusqlite::Result<MapEntry> {
    Ok(MapEntry {
        path: row.get(0)?,
        name: row.get(1)?,
        parent_path: row.get(2)?,
        kind: row.get(3)?,
        category: row.get(4)?,
        extension: row.get(5)?,
        mime_type: row.get(6)?,
        size_bytes: row.get(7)?,
        modified_at: row.get(8)?,
        depth: row.get(9)?,
        is_hidden: row.get::<_, i32>(10)? != 0,
        is_blocked: row.get::<_, i32>(11)? != 0,
        is_ignored: row.get::<_, i32>(12)? != 0,
        is_symlink: row.get::<_, i32>(13)? != 0,
        hash: row.get(14)?,
    })
}

/// Retrieves all files and folders index nodes for a specific scan run.
pub fn get_entries_for_run(conn: &Connection, run_id: i64) -> anyhow::Result<Vec<MapEntry>> {
    let mut stmt = conn.prepare(
        "SELECT path, name, parent_path, kind, category, extension, mime_type, \
         size_bytes, modified_at, depth, is_hidden, is_blocked, is_ignored, \
         is_symlink, hash FROM map_entries WHERE scan_run_id = ?1 ORDER BY path ASC",
    )?;
    let rows = stmt.query_map(params![run_id], row_to_map_entry)?;
    let mut results = Vec::new();
    for entry in rows {
        results.push(entry?);
    }
    Ok(results)
}

/// Searches the latest workspace map metadata matching a query on name, path, extension, or category.
pub fn search_latest_entries(conn: &Connection, query: &str) -> anyhow::Result<Vec<MapEntry>> {
    let run_id = match get_latest_scan_run_id(conn)? {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let sql_query = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT path, name, parent_path, kind, category, extension, mime_type, \
         size_bytes, modified_at, depth, is_hidden, is_blocked, is_ignored, \
         is_symlink, hash FROM map_entries \
         WHERE scan_run_id = ?1 AND ( \
             name LIKE ?2 OR \
             path LIKE ?2 OR \
             extension LIKE ?2 OR \
             category LIKE ?2 \
         ) ORDER BY path ASC LIMIT 100",
    )?;

    let rows = stmt.query_map(params![run_id, sql_query], row_to_map_entry)?;
    let mut results = Vec::new();
    for entry in rows {
        results.push(entry?);
    }
    Ok(results)
}

/// Retrieves a single node record from the latest complete scan run by path.
pub fn get_latest_entry_by_path(
    conn: &Connection,
    path_str: &str,
) -> anyhow::Result<Option<MapEntry>> {
    let run_id = match get_latest_scan_run_id(conn)? {
        Some(id) => id,
        None => return Ok(None),
    };

    // Normalize path separators for database query consistency
    let normalized = path_str.replace('\\', "/");

    let mut stmt = conn.prepare(
        "SELECT path, name, parent_path, kind, category, extension, mime_type, \
         size_bytes, modified_at, depth, is_hidden, is_blocked, is_ignored, \
         is_symlink, hash FROM map_entries \
         WHERE scan_run_id = ?1 AND (path = ?2 OR path = ?3) LIMIT 1",
    )?;

    let mut rows = stmt.query(params![run_id, normalized, format!("./{}", normalized)])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_map_entry(row)?))
    } else {
        Ok(None)
    }
}
