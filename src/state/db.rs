use crate::state::migrations;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

pub const STATE_DB_FILE_NAME: &str = "opennivara_state.sqlite";

pub fn state_db_path() -> anyhow::Result<PathBuf> {
    Ok(crate::config_paths::config_dir()?.join(STATE_DB_FILE_NAME))
}

pub fn open_state_db() -> anyhow::Result<Connection> {
    open_state_db_at(state_db_path()?)
}

pub fn open_state_db_at(path: impl AsRef<Path>) -> anyhow::Result<Connection> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    reset_legacy_alpha_db_if_needed(path)?;

    let mut conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrations::run_migrations(&mut conn)?;
    Ok(conn)
}

pub fn reset_legacy_alpha_db_if_needed(path: &Path) -> anyhow::Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }

    let conn = Connection::open(path)?;
    let is_migrated = table_exists(&conn, "refinery_schema_history")?;
    let has_legacy_sessions = column_exists(&conn, "sessions", "source_created")?;
    drop(conn);

    if is_migrated || !has_legacy_sessions {
        return Ok(None);
    }

    let backup_path = next_legacy_backup_path(path);
    fs::rename(path, &backup_path)?;
    Ok(Some(backup_path))
}

fn table_exists(conn: &Connection, table_name: &str) -> anyhow::Result<bool> {
    let exists = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [table_name],
        |row| row.get::<_, i64>(0),
    )?;
    Ok(exists == 1)
}

fn column_exists(conn: &Connection, table_name: &str, column_name: &str) -> anyhow::Result<bool> {
    if !table_exists(conn, table_name)? {
        return Ok(false);
    }

    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for column in columns {
        if column? == column_name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn next_legacy_backup_path(path: &Path) -> PathBuf {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S-%f").to_string();

    for suffix in 0.. {
        let suffix = if suffix == 0 {
            String::new()
        } else {
            format!("-{suffix}")
        };
        let candidate = path.with_file_name(format!(
            "opennivara_state.legacy-reset-{timestamp}{suffix}.sqlite"
        ));
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("unbounded backup suffix search should always return")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::tempdir;

    fn table_exists(conn: &Connection, table_name: &str) -> bool {
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            [table_name],
            |row| row.get::<_, i64>(0),
        )
        .unwrap()
            == 1
    }

    #[test]
    fn open_state_db_at_creates_parent_enables_foreign_keys_and_runs_migrations() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("nested").join(STATE_DB_FILE_NAME);

        let conn = open_state_db_at(&db_path).unwrap();

        assert!(db_path.exists());
        assert!(table_exists(&conn, "pending_turns"));
        let foreign_keys: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn legacy_inline_alpha_db_is_backed_up_and_reset() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join(STATE_DB_FILE_NAME);
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute(
                "CREATE TABLE sessions (
                    id TEXT PRIMARY KEY,
                    title TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    status TEXT NOT NULL,
                    source_created TEXT NOT NULL,
                    active INTEGER NOT NULL DEFAULT 1
                )",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO sessions (
                    id, title, created_at, updated_at, status, source_created, active
                ) VALUES (
                    'legacy_session', 'Old', '2026-06-07T00:00:00Z',
                    '2026-06-07T00:00:00Z', 'active', 'CLI', 1
                )",
                [],
            )
            .unwrap();
        }

        let backup = reset_legacy_alpha_db_if_needed(&db_path).unwrap();
        assert!(backup.as_ref().is_some_and(|path| path.exists()));

        let conn = open_state_db_at(&db_path).unwrap();
        assert!(table_exists(&conn, "pending_turns"));
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn migrated_db_is_not_backed_up_again() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join(STATE_DB_FILE_NAME);
        let _ = open_state_db_at(&db_path).unwrap();

        let backup = reset_legacy_alpha_db_if_needed(&db_path).unwrap();

        assert!(backup.is_none());
    }
}
