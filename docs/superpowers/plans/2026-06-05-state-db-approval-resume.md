# State DB Approval Resume Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the inline alpha state DB schema with embedded refinery migrations and add durable approval-resume storage, lifecycle helpers, and tests.

**Architecture:** Add a focused `src/state` module that owns DB opening, migration execution, legacy alpha reset, typed session/message/active-session APIs, approval persistence, and pending-turn storage. Keep `src/sessions.rs` as the compatibility layer during the migration, then move callers to the new state helpers in later PRs. Store approval chat events as `messages.role = 'event'` with JSON `content`.

**Tech Stack:** Rust 2021, `rusqlite`, `refinery`, `chrono`, `serde`, `serde_json`, `uuid`, `tempfile`, `serial_test`.

---

## File Structure

- Create `src/state/mod.rs`: module exports for state DB, migrations, approval helpers, and state types.
- Create `src/state/db.rs`: state DB path resolution, `open_state_db()`, injectable `open_state_db_at()`, foreign-key setup, legacy alpha backup/reset.
- Create `src/state/migrations.rs`: embedded refinery migration runner using `embed_migrations!("src/state/migrations")`.
- Create `src/state/types.rs`: `Surface`, `ApprovalStatus`, `PendingApprovalInput`, `PendingApprovalRecord`, `PendingTurnInput`, `PendingTurnRecord`, `DeniedToolResult`.
- Create `src/state/sessions.rs`: typed session CRUD over `&Connection`.
- Create `src/state/messages.rs`: typed message storage, event message storage, and session history loading.
- Create `src/state/active_sessions.rs`: active session mapping by `(actor_id, surface)`.
- Create `src/state/approvals.rs`: approval creation, pending turn persistence, same-session validation, hardcoded actor permission, atomic execution guard, terminal cleanup helpers.
- Create `src/state/migrations/V1__initial_state_schema.sql`: clean session/message/active-session/pinned schema with `surface` naming.
- Create `src/state/migrations/V2__approval_resume.sql`: approval metadata and pending-turn schema.
- Modify `src/lib.rs`: expose `pub mod state;`.
- Modify `src/sessions.rs`: switch `init_db()` to call `state::db::open_state_db()` in PR 1, then progressively delegate approval helpers in PR 2.
- Add `src/state/tests.rs` or inline `#[cfg(test)]` modules under `src/state`: migration, legacy reset, schema, approval lifecycle, and event-message tests.
- Modify `Cargo.toml` only if refinery embedded migrations require an additional feature after compiling.

## PR 1: State Migration Foundation

### Task 1: Add State Module Skeleton

**Files:**

- Create: `src/state/mod.rs`
- Create: `src/state/db.rs`
- Create: `src/state/migrations.rs`
- Create: `src/state/types.rs`
- Create: `src/state/sessions.rs`
- Create: `src/state/messages.rs`
- Create: `src/state/active_sessions.rs`
- Create: `src/state/approvals.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Add a failing module compile test**

Add this to `src/state/mod.rs`:

```rust
pub mod approvals;
pub mod active_sessions;
pub mod db;
pub mod messages;
pub mod migrations;
pub mod sessions;
pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn state_module_exports_expected_submodules() {
        let _ = super::db::STATE_DB_FILE_NAME;
        let _ = super::migrations::STATE_MIGRATION_COUNT;
    }
}
```

- [ ] **Step 2: Run the focused compile check**

Run: `cargo test state_module_exports_expected_submodules`

Expected: fail because `src/state/db.rs` and `src/state/migrations.rs` do not exist.

- [ ] **Step 3: Add minimal files**

Create `src/state/db.rs`:

```rust
pub const STATE_DB_FILE_NAME: &str = "opennivara_state.sqlite";
```

Create `src/state/migrations.rs`:

```rust
pub const STATE_MIGRATION_COUNT: usize = 2;
```

Create `src/state/types.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Surface {
    Desktop,
    Cli,
    Telegram,
}
```

Create `src/state/approvals.rs`:

```rust
pub fn actor_can_approve(actor_id: &str) -> bool {
    actor_id == "desktop_owner" || actor_id == "cli_owner" || actor_id.starts_with("telegram_")
}
```

Create `src/state/sessions.rs`:

```rust
pub use crate::state::types::{CreateSessionInput, Session};
```

Create `src/state/messages.rs`:

```rust
pub use crate::state::types::{DbMessage, StoreEventMessageInput, StoreMessageInput};
```

Create `src/state/active_sessions.rs`:

```rust
pub use crate::state::types::Surface;
```

Add to `src/lib.rs`:

```rust
pub mod state;
```

- [ ] **Step 4: Verify the skeleton compiles**

Run: `cargo test state_module_exports_expected_submodules`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/state
git commit -m "feat(state): add state module skeleton"
```

### Task 2: Add Embedded V1/V2 Migrations

**Files:**

- Create: `src/state/migrations/V1__initial_state_schema.sql`
- Create: `src/state/migrations/V2__approval_resume.sql`
- Modify: `src/state/migrations.rs`

- [ ] **Step 1: Write the failing migration test**

Add this test to `src/state/migrations.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::run_migrations;
    use rusqlite::Connection;

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
    fn fresh_db_runs_v1_and_v2_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();

        run_migrations(&mut conn).unwrap();

        assert!(table_exists(&conn, "sessions"));
        assert!(table_exists(&conn, "messages"));
        assert!(table_exists(&conn, "active_sessions"));
        assert!(table_exists(&conn, "session_pinned_contexts"));
        assert!(table_exists(&conn, "session_pinned_skills"));
        assert!(table_exists(&conn, "pending_approvals"));
        assert!(table_exists(&conn, "pending_turns"));
        assert!(table_exists(&conn, "refinery_schema_history"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test state::migrations::tests::fresh_db_runs_v1_and_v2_migrations`

Expected: fail because `run_migrations` is not implemented and migration files are missing.

- [ ] **Step 3: Add V1 migration SQL**

Create `src/state/migrations/V1__initial_state_schema.sql`:

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    title TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    status TEXT NOT NULL,
    surface_created TEXT NOT NULL,
    actor_id_created TEXT,
    active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'tool', 'event', 'system')),
    surface TEXT NOT NULL,
    actor_id TEXT,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    metadata_json TEXT,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TABLE active_sessions (
    actor_id TEXT NOT NULL,
    surface TEXT NOT NULL,
    session_id TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY(actor_id, surface),
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TABLE session_pinned_contexts (
    session_id TEXT NOT NULL,
    context_id TEXT NOT NULL,
    pinned_at TEXT NOT NULL,
    PRIMARY KEY(session_id, context_id),
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TABLE session_pinned_skills (
    session_id TEXT NOT NULL,
    skill_id TEXT NOT NULL,
    pinned_at TEXT NOT NULL,
    PRIMARY KEY(session_id, skill_id),
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_messages_session_id ON messages(session_id);
CREATE INDEX idx_sessions_updated_at ON sessions(updated_at);
CREATE INDEX idx_active_sessions_session_id ON active_sessions(session_id);
```

- [ ] **Step 4: Add V2 migration SQL**

Create `src/state/migrations/V2__approval_resume.sql`:

```sql
CREATE TABLE pending_approvals (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    request_id TEXT NOT NULL,
    user_message_id TEXT NOT NULL,
    tool_call_id TEXT NOT NULL,
    surface TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    operation_name TEXT NOT NULL,
    classification TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'denied', 'executing', 'executed', 'failed')),
    summary TEXT,
    arguments_preview_json TEXT,
    result_summary TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL,
    resolved_at TEXT,
    resolved_by_actor_id TEXT,
    execution_started_at TEXT,
    execution_finished_at TEXT,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY(user_message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE TABLE pending_turns (
    approval_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    request_id TEXT NOT NULL,
    user_message_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    resume_payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(approval_id) REFERENCES pending_approvals(id) ON DELETE CASCADE,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY(user_message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX idx_pending_approvals_session_status ON pending_approvals(session_id, status);
CREATE INDEX idx_pending_approvals_actor_status ON pending_approvals(actor_id, status);
CREATE INDEX idx_pending_approvals_request_id ON pending_approvals(request_id);
CREATE INDEX idx_pending_approvals_user_message_id ON pending_approvals(user_message_id);
CREATE INDEX idx_pending_turns_session ON pending_turns(session_id);
CREATE INDEX idx_pending_turns_request_id ON pending_turns(request_id);
```

- [ ] **Step 5: Implement embedded migration runner**

Replace `src/state/migrations.rs` with:

```rust
use rusqlite::Connection;

mod embedded {
    refinery::embed_migrations!("src/state/migrations");
}

pub const STATE_MIGRATION_COUNT: usize = 2;

pub fn run_migrations(conn: &mut Connection) -> anyhow::Result<()> {
    embedded::migrations::runner()
        .run(conn)
        .map_err(|err| anyhow::anyhow!("failed to run state DB migrations: {err}"))?;
    Ok(())
}
```

Keep the test module from Step 1 below this implementation.

- [ ] **Step 6: Verify V1/V2 run**

Run: `cargo test state::migrations::tests::fresh_db_runs_v1_and_v2_migrations`

Expected: pass.

- [ ] **Step 7: Commit**

```bash
git add src/state/migrations.rs src/state/migrations
git commit -m "feat(state): add embedded state migrations"
```

### Task 3: Add Schema Contract Tests

**Files:**

- Modify: `src/state/migrations.rs`

- [ ] **Step 1: Add column and constraint tests**

Append these tests inside `src/state/migrations.rs` test module:

```rust
fn column_names(conn: &Connection, table_name: &str) -> Vec<String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .unwrap();
    stmt.query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn index_exists(conn: &Connection, index_name: &str) -> bool {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'index' AND name = ?1)",
        [index_name],
        |row| row.get::<_, i64>(0),
    )
    .unwrap()
        == 1
}

#[test]
fn v1_schema_uses_surface_and_actor_fields() {
    let mut conn = Connection::open_in_memory().unwrap();
    run_migrations(&mut conn).unwrap();

    let session_columns = column_names(&conn, "sessions");
    assert!(session_columns.contains(&"surface_created".to_string()));
    assert!(session_columns.contains(&"actor_id_created".to_string()));
    assert!(!session_columns.contains(&"source_created".to_string()));

    let message_columns = column_names(&conn, "messages");
    assert!(message_columns.contains(&"surface".to_string()));
    assert!(message_columns.contains(&"actor_id".to_string()));
    assert!(!message_columns.contains(&"source".to_string()));
}

#[test]
fn messages_role_check_rejects_invalid_role() {
    let mut conn = Connection::open_in_memory().unwrap();
    run_migrations(&mut conn).unwrap();

    conn.execute(
        "INSERT INTO sessions (id, title, created_at, updated_at, status, surface_created, actor_id_created, active)
         VALUES ('s1', 'Test', '2026-06-05T00:00:00Z', '2026-06-05T00:00:00Z', 'active', 'CLI', 'cli_owner', 1)",
        [],
    )
    .unwrap();

    let result = conn.execute(
        "INSERT INTO messages (id, session_id, role, surface, actor_id, content, created_at)
         VALUES ('m1', 's1', 'bad_role', 'CLI', 'cli_owner', '{}', '2026-06-05T00:00:00Z')",
        [],
    );

    assert!(result.is_err());
}

#[test]
fn v2_schema_has_expected_approval_columns_and_indexes() {
    let mut conn = Connection::open_in_memory().unwrap();
    run_migrations(&mut conn).unwrap();

    let approval_columns = column_names(&conn, "pending_approvals");
    for expected in [
        "request_id",
        "user_message_id",
        "tool_call_id",
        "result_summary",
        "error_message",
    ] {
        assert!(approval_columns.contains(&expected.to_string()));
    }
    assert!(!approval_columns.contains(&"expires_at".to_string()));

    let turn_columns = column_names(&conn, "pending_turns");
    for expected in ["provider_id", "model_id", "resume_payload_json"] {
        assert!(turn_columns.contains(&expected.to_string()));
    }

    for expected in [
        "idx_pending_approvals_session_status",
        "idx_pending_approvals_actor_status",
        "idx_pending_approvals_request_id",
        "idx_pending_approvals_user_message_id",
        "idx_pending_turns_session",
        "idx_pending_turns_request_id",
    ] {
        assert!(index_exists(&conn, expected));
    }
}

#[test]
fn pending_approval_status_check_rejects_invalid_status() {
    let mut conn = Connection::open_in_memory().unwrap();
    run_migrations(&mut conn).unwrap();
    seed_session_and_message(&conn);

    let result = conn.execute(
        "INSERT INTO pending_approvals (
            id, session_id, request_id, user_message_id, tool_call_id, surface, actor_id,
            operation_name, classification, status, created_at
        ) VALUES (
            'a1', 's1', 'r1', 'm1', 'tc1', 'CLI', 'cli_owner',
            'write_file', 'future_classification', 'approved', '2026-06-05T00:00:00Z'
        )",
        [],
    );

    assert!(result.is_err());
}

#[test]
fn classification_accepts_evolving_free_text() {
    let mut conn = Connection::open_in_memory().unwrap();
    run_migrations(&mut conn).unwrap();
    seed_session_and_message(&conn);

    conn.execute(
        "INSERT INTO pending_approvals (
            id, session_id, request_id, user_message_id, tool_call_id, surface, actor_id,
            operation_name, classification, status, created_at
        ) VALUES (
            'a1', 's1', 'r1', 'm1', 'tc1', 'CLI', 'cli_owner',
            'tool', 'new_future_class', 'pending', '2026-06-05T00:00:00Z'
        )",
        [],
    )
    .unwrap();
}

fn seed_session_and_message(conn: &Connection) {
    conn.execute(
        "INSERT INTO sessions (id, title, created_at, updated_at, status, surface_created, actor_id_created, active)
         VALUES ('s1', 'Test', '2026-06-05T00:00:00Z', '2026-06-05T00:00:00Z', 'active', 'CLI', 'cli_owner', 1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO messages (id, session_id, role, surface, actor_id, content, created_at)
         VALUES ('m1', 's1', 'user', 'CLI', 'cli_owner', 'hello', '2026-06-05T00:00:00Z')",
        [],
    )
    .unwrap();
}
```

- [ ] **Step 2: Run schema contract tests**

Run: `cargo test state::migrations::tests`

Expected: pass.

- [ ] **Step 3: Commit**

```bash
git add src/state/migrations.rs
git commit -m "test(state): lock state migration schema contracts"
```

### Task 4: Implement `open_state_db()` And Legacy Alpha Reset

**Files:**

- Modify: `src/state/db.rs`

- [ ] **Step 1: Write DB open and legacy reset tests**

Replace `src/state/db.rs` with this test-first scaffold:

```rust
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

    let mut conn = Connection::open(path)
        .map_err(|err| anyhow::anyhow!("Failed to open state SQLite database: {err}"))?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    migrations::run_migrations(&mut conn)?;
    Ok(conn)
}

fn reset_legacy_alpha_db_if_needed(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let conn = Connection::open(path)?;
    let has_refinery = table_exists(&conn, "refinery_schema_history")?;
    let has_legacy_sessions = column_exists(&conn, "sessions", "source_created")?;
    drop(conn);

    if has_legacy_sessions && !has_refinery {
        let backup_path = legacy_backup_path(path);
        fs::rename(path, backup_path)?;
    }

    Ok(())
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
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column_name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn legacy_backup_path(path: &Path) -> PathBuf {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    path.with_file_name(format!("opennivara_state.legacy-reset-{timestamp}.sqlite"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn open_state_db_at_creates_parent_and_runs_migrations() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("nested").join(STATE_DB_FILE_NAME);

        let conn = open_state_db_at(&db_path).unwrap();

        assert!(db_path.exists());
        assert!(table_exists(&conn, "pending_turns").unwrap());
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
                "INSERT INTO sessions (id, title, created_at, updated_at, status, source_created, active)
                 VALUES ('old', 'Old', '2026-06-05T00:00:00Z', '2026-06-05T00:00:00Z', 'active', 'cli', 1)",
                [],
            )
            .unwrap();
        }

        let conn = open_state_db_at(&db_path).unwrap();

        assert!(table_exists(&conn, "refinery_schema_history").unwrap());
        assert!(column_exists(&conn, "sessions", "surface_created").unwrap());
        assert!(!column_exists(&conn, "sessions", "source_created").unwrap());

        let backups = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("opennivara_state.legacy-reset-")
            })
            .count();
        assert_eq!(backups, 1);
    }
}
```

- [ ] **Step 2: Run DB tests**

Run: `cargo test state::db::tests`

Expected: pass.

- [ ] **Step 3: Commit**

```bash
git add src/state/db.rs
git commit -m "feat(state): open migrated state database"
```

## PR 2: Approval Storage And Lifecycle

### Task 5: Add Approval And Pending Turn Types

**Files:**

- Modify: `src/state/types.rs`

- [ ] **Step 1: Add type tests**

Replace `src/state/types.rs` with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Surface {
    Desktop,
    Cli,
    Telegram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Denied,
    Executing,
    Executed,
    Failed,
}

impl ApprovalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Denied => "denied",
            Self::Executing => "executing",
            Self::Executed => "executed",
            Self::Failed => "failed",
        }
    }
}

pub struct PendingApprovalInput<'a> {
    pub id: &'a str,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub user_message_id: &'a str,
    pub tool_call_id: &'a str,
    pub surface: &'a str,
    pub actor_id: &'a str,
    pub operation_name: &'a str,
    pub classification: &'a str,
    pub summary: Option<&'a str>,
    pub arguments_preview_json: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingApprovalRecord {
    pub id: String,
    pub session_id: String,
    pub request_id: String,
    pub user_message_id: String,
    pub tool_call_id: String,
    pub surface: String,
    pub actor_id: String,
    pub operation_name: String,
    pub classification: String,
    pub status: String,
    pub summary: Option<String>,
    pub arguments_preview_json: Option<String>,
}

pub struct PendingTurnInput<'a> {
    pub approval_id: &'a str,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub user_message_id: &'a str,
    pub provider_id: &'a str,
    pub model_id: &'a str,
    pub resume_payload_json: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTurnRecord {
    pub approval_id: String,
    pub session_id: String,
    pub request_id: String,
    pub user_message_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub resume_payload_json: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DeniedToolResult {
    pub tool_call_id: String,
    pub denied: bool,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_status_strings_match_schema() {
        assert_eq!(ApprovalStatus::Pending.as_str(), "pending");
        assert_eq!(ApprovalStatus::Denied.as_str(), "denied");
        assert_eq!(ApprovalStatus::Executing.as_str(), "executing");
        assert_eq!(ApprovalStatus::Executed.as_str(), "executed");
        assert_eq!(ApprovalStatus::Failed.as_str(), "failed");
    }
}
```

- [ ] **Step 2: Run type tests**

Run: `cargo test state::types::tests`

Expected: pass.

- [ ] **Step 3: Commit**

```bash
git add src/state/types.rs
git commit -m "feat(state): add approval resume types"
```

### Task 6: Add Approval Storage Helpers

**Files:**

- Modify: `src/state/approvals.rs`

- [ ] **Step 1: Write storage helper tests and implementation**

Replace `src/state/approvals.rs` with:

```rust
use crate::state::types::{
    ApprovalStatus, DeniedToolResult, PendingApprovalInput, PendingApprovalRecord,
    PendingTurnInput, PendingTurnRecord,
};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

pub fn actor_can_approve(actor_id: &str) -> bool {
    actor_id == "desktop_owner" || actor_id == "cli_owner" || actor_id.starts_with("telegram_")
}

pub fn create_pending_approval(
    conn: &Connection,
    input: &PendingApprovalInput<'_>,
) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO pending_approvals (
            id, session_id, request_id, user_message_id, tool_call_id, surface, actor_id,
            operation_name, classification, status, summary, arguments_preview_json, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'pending', ?10, ?11, ?12)",
        params![
            input.id,
            input.session_id,
            input.request_id,
            input.user_message_id,
            input.tool_call_id,
            input.surface,
            input.actor_id,
            input.operation_name,
            input.classification,
            input.summary,
            input.arguments_preview_json,
            now,
        ],
    )?;
    Ok(())
}

pub fn create_pending_turn(conn: &Connection, input: &PendingTurnInput<'_>) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO pending_turns (
            approval_id, session_id, request_id, user_message_id, provider_id, model_id,
            resume_payload_json, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            input.approval_id,
            input.session_id,
            input.request_id,
            input.user_message_id,
            input.provider_id,
            input.model_id,
            input.resume_payload_json,
            now,
        ],
    )?;
    Ok(())
}

pub fn get_pending_approval(
    conn: &Connection,
    approval_id: &str,
) -> anyhow::Result<Option<PendingApprovalRecord>> {
    conn.query_row(
        "SELECT id, session_id, request_id, user_message_id, tool_call_id, surface, actor_id,
            operation_name, classification, status, summary, arguments_preview_json
         FROM pending_approvals WHERE id = ?1",
        [approval_id],
        |row| {
            Ok(PendingApprovalRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                request_id: row.get(2)?,
                user_message_id: row.get(3)?,
                tool_call_id: row.get(4)?,
                surface: row.get(5)?,
                actor_id: row.get(6)?,
                operation_name: row.get(7)?,
                classification: row.get(8)?,
                status: row.get(9)?,
                summary: row.get(10)?,
                arguments_preview_json: row.get(11)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

pub fn get_pending_turn(
    conn: &Connection,
    approval_id: &str,
) -> anyhow::Result<Option<PendingTurnRecord>> {
    conn.query_row(
        "SELECT approval_id, session_id, request_id, user_message_id, provider_id, model_id,
            resume_payload_json
         FROM pending_turns WHERE approval_id = ?1",
        [approval_id],
        |row| {
            Ok(PendingTurnRecord {
                approval_id: row.get(0)?,
                session_id: row.get(1)?,
                request_id: row.get(2)?,
                user_message_id: row.get(3)?,
                provider_id: row.get(4)?,
                model_id: row.get(5)?,
                resume_payload_json: row.get(6)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

pub fn validate_approval_context(
    approval: &PendingApprovalRecord,
    session_id: &str,
    actor_id: &str,
) -> anyhow::Result<()> {
    if approval.session_id != session_id {
        anyhow::bail!("approval belongs to a different session");
    }
    if !actor_can_approve(actor_id) {
        anyhow::bail!("actor is not allowed to approve operations");
    }
    Ok(())
}

pub fn mark_executing_once(conn: &Connection, approval_id: &str) -> anyhow::Result<bool> {
    let changed = conn.execute(
        "UPDATE pending_approvals
         SET status = 'executing', execution_started_at = ?1
         WHERE id = ?2 AND status = 'pending'",
        params![Utc::now().to_rfc3339(), approval_id],
    )?;
    Ok(changed == 1)
}

pub fn mark_terminal(
    conn: &Connection,
    approval_id: &str,
    status: ApprovalStatus,
    result_summary: Option<&str>,
    error_message: Option<&str>,
) -> anyhow::Result<()> {
    if !matches!(
        status,
        ApprovalStatus::Denied | ApprovalStatus::Executed | ApprovalStatus::Failed
    ) {
        anyhow::bail!("status is not terminal");
    }
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE pending_approvals
         SET status = ?1, execution_finished_at = ?2, result_summary = ?3, error_message = ?4
         WHERE id = ?5",
        params![status.as_str(), now, result_summary, error_message, approval_id],
    )?;
    conn.execute("DELETE FROM pending_turns WHERE approval_id = ?1", [approval_id])?;
    Ok(())
}

pub fn denied_tool_result(tool_call_id: &str, reason: &str) -> DeniedToolResult {
    DeniedToolResult {
        tool_call_id: tool_call_id.to_string(),
        denied: true,
        reason: reason.to_string(),
    }
}
```

- [ ] **Step 2: Add tests for helper behavior**

Append this test module to `src/state/approvals.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::migrations::run_migrations;
    use rusqlite::Connection;

    fn setup() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        run_migrations(&mut conn).unwrap();
        conn.execute(
            "INSERT INTO sessions (id, title, created_at, updated_at, status, surface_created, actor_id_created, active)
             VALUES ('s1', 'Test', '2026-06-05T00:00:00Z', '2026-06-05T00:00:00Z', 'active', 'CLI', 'cli_owner', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO sessions (id, title, created_at, updated_at, status, surface_created, actor_id_created, active)
             VALUES ('s2', 'Other', '2026-06-05T00:00:00Z', '2026-06-05T00:00:00Z', 'active', 'CLI', 'cli_owner', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (id, session_id, role, surface, actor_id, content, created_at)
             VALUES ('m1', 's1', 'user', 'CLI', 'cli_owner', 'hello', '2026-06-05T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    fn insert_approval_and_turn(conn: &Connection) {
        create_pending_approval(
            conn,
            &PendingApprovalInput {
                id: "a1",
                session_id: "s1",
                request_id: "r1",
                user_message_id: "m1",
                tool_call_id: "tc1",
                surface: "CLI",
                actor_id: "cli_owner",
                operation_name: "write_file",
                classification: "local_modify",
                summary: Some("modify file"),
                arguments_preview_json: Some(r#"{"path":"src/main.rs"}"#),
            },
        )
        .unwrap();
        create_pending_turn(
            conn,
            &PendingTurnInput {
                approval_id: "a1",
                session_id: "s1",
                request_id: "r1",
                user_message_id: "m1",
                provider_id: "gemini",
                model_id: "gemini-pro",
                resume_payload_json: r#"{"model_messages_so_far":[]}"#,
            },
        )
        .unwrap();
    }

    #[test]
    fn approval_and_pending_turn_can_be_inserted_and_loaded() {
        let conn = setup();
        insert_approval_and_turn(&conn);

        let approval = get_pending_approval(&conn, "a1").unwrap().unwrap();
        let turn = get_pending_turn(&conn, "a1").unwrap().unwrap();

        assert_eq!(approval.status, "pending");
        assert_eq!(approval.classification, "local_modify");
        assert_eq!(turn.provider_id, "gemini");
        assert_eq!(turn.resume_payload_json, r#"{"model_messages_so_far":[]}"#);
    }

    #[test]
    fn same_session_validation_rejects_wrong_session() {
        let conn = setup();
        insert_approval_and_turn(&conn);
        let approval = get_pending_approval(&conn, "a1").unwrap().unwrap();

        let result = validate_approval_context(&approval, "s2", "cli_owner");

        assert!(result.is_err());
    }

    #[test]
    fn actor_without_permission_cannot_approve() {
        let conn = setup();
        insert_approval_and_turn(&conn);
        let approval = get_pending_approval(&conn, "a1").unwrap().unwrap();

        let result = validate_approval_context(&approval, "s1", "guest");

        assert!(result.is_err());
    }

    #[test]
    fn atomic_pending_to_executing_allows_one_execution() {
        let conn = setup();
        insert_approval_and_turn(&conn);

        assert!(mark_executing_once(&conn, "a1").unwrap());
        assert!(!mark_executing_once(&conn, "a1").unwrap());
    }

    #[test]
    fn terminal_status_deletes_pending_turn_but_keeps_approval() {
        let conn = setup();
        insert_approval_and_turn(&conn);

        mark_terminal(&conn, "a1", ApprovalStatus::Executed, Some("done"), None).unwrap();

        assert!(get_pending_approval(&conn, "a1").unwrap().is_some());
        assert!(get_pending_turn(&conn, "a1").unwrap().is_none());
    }

    #[test]
    fn denial_produces_model_visible_tool_result() {
        let result = denied_tool_result("tc1", "User denied file modification");

        assert_eq!(result.tool_call_id, "tc1");
        assert!(result.denied);
        assert_eq!(result.reason, "User denied file modification");
    }
}
```

- [ ] **Step 3: Run approval tests**

Run: `cargo test state::approvals::tests`

Expected: pass.

- [ ] **Step 4: Commit**

```bash
git add src/state/approvals.rs
git commit -m "feat(state): add approval lifecycle helpers"
```

### Task 7: Add Reopen Persistence Test

**Files:**

- Modify: `src/state/approvals.rs`

- [ ] **Step 1: Add persistence test**

Append this test inside `src/state/approvals.rs` test module:

```rust
#[test]
fn pending_approval_survives_database_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("state.sqlite");
    {
        let conn = crate::state::db::open_state_db_at(&db_path).unwrap();
        conn.execute(
            "INSERT INTO sessions (id, title, created_at, updated_at, status, surface_created, actor_id_created, active)
             VALUES ('s1', 'Test', '2026-06-05T00:00:00Z', '2026-06-05T00:00:00Z', 'active', 'CLI', 'cli_owner', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (id, session_id, role, surface, actor_id, content, created_at)
             VALUES ('m1', 's1', 'user', 'CLI', 'cli_owner', 'hello', '2026-06-05T00:00:00Z')",
            [],
        )
        .unwrap();
        insert_approval_and_turn(&conn);
    }

    let reopened = crate::state::db::open_state_db_at(&db_path).unwrap();

    assert!(get_pending_approval(&reopened, "a1").unwrap().is_some());
    assert!(get_pending_turn(&reopened, "a1").unwrap().is_some());
}
```

- [ ] **Step 2: Run persistence test**

Run: `cargo test pending_approval_survives_database_reopen`

Expected: pass.

- [ ] **Step 3: Commit**

```bash
git add src/state/approvals.rs
git commit -m "test(state): persist pending approvals across reopen"
```

### Task 8: Store Approval Event Messages

**Files:**

- Modify: `src/state/approvals.rs`

- [ ] **Step 1: Add event helper**

Add this function to `src/state/approvals.rs`:

```rust
pub fn store_approval_event_message(
    conn: &Connection,
    message_id: &str,
    session_id: &str,
    surface: &str,
    actor_id: Option<&str>,
    event_json: &str,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO messages (id, session_id, role, surface, actor_id, content, created_at)
         VALUES (?1, ?2, 'event', ?3, ?4, ?5, ?6)",
        params![
            message_id,
            session_id,
            surface,
            actor_id,
            event_json,
            Utc::now().to_rfc3339(),
        ],
    )?;
    Ok(())
}
```

- [ ] **Step 2: Add event-message test**

Append this test inside `src/state/approvals.rs` test module:

```rust
#[test]
fn approval_event_messages_are_stored_as_json_content() {
    let conn = setup();
    let event_json = r#"{"event_type":"approval_required","approval_id":"a1"}"#;

    store_approval_event_message(&conn, "event1", "s1", "CLI", Some("cli_owner"), event_json)
        .unwrap();

    let (role, content): (String, String) = conn
        .query_row(
            "SELECT role, content FROM messages WHERE id = 'event1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(role, "event");
    assert_eq!(content, event_json);
    serde_json::from_str::<serde_json::Value>(&content).unwrap();
}
```

- [ ] **Step 3: Run event test**

Run: `cargo test approval_event_messages_are_stored_as_json_content`

Expected: pass.

- [ ] **Step 4: Commit**

```bash
git add src/state/approvals.rs
git commit -m "feat(state): store approval event messages"
```

## PR 3: Engine Integration

### Task 9: Route Approval-Required Tool Calls Through State Helpers

**Files:**

- Modify: `src/engine.rs`
- Modify: `src/tools.rs`
- Modify: `src/sessions.rs`
- Test: existing engine/tool tests or new focused engine tests

- [ ] **Step 1: Write an engine test for approval-required pause**

Add a test that simulates a mutating tool call and asserts:

```rust
assert_eq!(approval.status, "pending");
assert!(pending_turn.resume_payload_json.contains("model_messages_so_far"));
assert_eq!(event_message.role, "event");
```

Use a fake model/tool executor so no real file mutation occurs.

- [ ] **Step 2: Run the test and verify it fails**

Run: `cargo test approval_required_tool_call_creates_pending_turn`

Expected: fail because engine integration has not been implemented.

- [ ] **Step 3: Implement approval-required branch**

In the central tool-call handling path:

```rust
if classification.requires_approval() {
    state::approvals::create_pending_approval(conn, &approval_input)?;
    state::approvals::create_pending_turn(conn, &pending_turn_input)?;
    state::approvals::store_approval_event_message(
        conn,
        &event_message_id,
        &session_id,
        request.surface.as_str(),
        Some(&request.actor_id),
        &serde_json::to_string(&approval_event)?,
    )?;
    return Ok(EngineTurnOutcome::WaitingForApproval { approval_id });
}
```

- [ ] **Step 4: Run approval pause test**

Run: `cargo test approval_required_tool_call_creates_pending_turn`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/engine.rs src/tools.rs src/sessions.rs
git commit -m "feat(engine): persist approval-required turns"
```

### Task 10: Resume Same Turn After Approval Or Denial

**Files:**

- Modify: `src/engine.rs`
- Modify: `src/state/approvals.rs`
- Test: engine approval resume tests

- [ ] **Step 1: Write resume tests**

Add tests with these assertions:

```rust
assert!(mark_executing_once(conn, approval_id).unwrap());
assert_eq!(executed_tool_calls.len(), 1);
assert_eq!(final_response.session_id, original_session_id);
assert!(get_pending_turn(conn, approval_id).unwrap().is_none());
```

Add duplicate approval assertion:

```rust
assert!(!mark_executing_once(conn, approval_id).unwrap());
assert_eq!(executed_tool_calls.len(), 1);
```

Add denial assertion:

```rust
assert_eq!(tool_result.denied, true);
assert!(assistant_response.content.contains("denied") || assistant_response.content.len() > 0);
```

- [ ] **Step 2: Run tests and verify failure**

Run: `cargo test approval_resume`

Expected: fail because approval resume is not wired.

- [ ] **Step 3: Implement resume entry point**

Add a shared engine function shaped like:

```rust
pub async fn resume_pending_approval(
    request: ApprovalDecisionRequest,
) -> anyhow::Result<EngineTurnOutcome> {
    let approval = state::approvals::get_pending_approval(&conn, &request.approval_id)?
        .ok_or_else(|| anyhow::anyhow!("approval not found"))?;
    state::approvals::validate_approval_context(
        &approval,
        &request.session_id,
        &request.actor_id,
    )?;

    if request.decision == ApprovalDecision::Deny {
        let result = state::approvals::denied_tool_result(&approval.tool_call_id, "User denied");
        state::approvals::mark_terminal(
            &conn,
            &approval.id,
            ApprovalStatus::Denied,
            Some("denied"),
            None,
        )?;
        return continue_model_with_tool_result(result).await;
    }

    if !state::approvals::mark_executing_once(&conn, &approval.id)? {
        anyhow::bail!("approval is no longer pending");
    }

    let pending_turn = state::approvals::get_pending_turn(&conn, &approval.id)?
        .ok_or_else(|| anyhow::anyhow!("pending turn missing"))?;
    execute_approved_tool_once_and_continue(pending_turn).await
}
```

- [ ] **Step 4: Run resume tests**

Run: `cargo test approval_resume`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/engine.rs src/state/approvals.rs
git commit -m "feat(engine): resume approved turns once"
```

## PR 4: Surface UX

### Task 11: Desktop Approval Dialog

**Files:**

- Modify: `desktop/src/features/tools/ToolApprovalDialog.tsx`
- Modify: `desktop/src/features/chat/ChatView.tsx`
- Modify: `desktop/src/api/opennivaraClient.ts`
- Test: `desktop/src/features/tools/ToolApprovalDialog.test.tsx`

- [ ] **Step 1: Add UI test**

Assert the dialog renders operation name, classification, summary, expandable arguments, approve, and deny controls:

```tsx
expect(screen.getByText("write_file")).toBeInTheDocument();
expect(screen.getByText("local_modify")).toBeInTheDocument();
expect(screen.getByRole("button", { name: /approve/i })).toBeInTheDocument();
expect(screen.getByRole("button", { name: /deny/i })).toBeInTheDocument();
```

- [ ] **Step 2: Run test and verify failure**

Run: `cd desktop && bun test ToolApprovalDialog`

Expected: fail until the dialog is wired to pending approval data.

- [ ] **Step 3: Wire approval decision calls**

Call backend commands equivalent to:

```ts
await client.approvePendingOperation({ approvalId, sessionId });
await client.denyPendingOperation({ approvalId, sessionId });
```

- [ ] **Step 4: Run desktop tests**

Run: `cd desktop && bun test ToolApprovalDialog ChatView`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add desktop/src/features/tools desktop/src/features/chat desktop/src/api
git commit -m "feat(desktop): attach approval dialog to chat"
```

### Task 12: CLI And Telegram Approval Commands

**Files:**

- Modify: `src/main.rs`
- Modify: `src/telegram.rs`
- Modify: `src/engine.rs`
- Test: CLI command tests and Telegram handler tests

- [ ] **Step 1: Add CLI tests**

Assert same-session CLI approval calls the shared resume entry point:

```rust
assert_eq!(decision.approval_id, "a1");
assert_eq!(decision.session_id, "s1");
assert_eq!(decision.actor_id, "cli_owner");
```

- [ ] **Step 2: Add Telegram tests**

Assert `/approve a1` and `/deny a1` normalize to the current chat session and `telegram_<chat_id>` actor:

```rust
assert_eq!(decision.actor_id, "telegram_12345");
assert_eq!(decision.session_id, active_chat_session_id);
```

- [ ] **Step 3: Run tests and verify failure**

Run: `cargo test approval_command`

Expected: fail until command handlers are wired.

- [ ] **Step 4: Implement handlers**

Add command handling that constructs:

```rust
ApprovalDecisionRequest {
    approval_id,
    session_id,
    actor_id,
    decision,
}
```

Then call the shared engine resume function.

- [ ] **Step 5: Run command tests**

Run: `cargo test approval_command`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/main.rs src/telegram.rs src/engine.rs
git commit -m "feat: add cli and telegram approval decisions"
```

## Final Verification

- [ ] **Step 1: Run Rust tests**

Run: `cargo test`

Expected: all Rust tests pass.

- [ ] **Step 2: Run desktop tests**

Run: `cd desktop && bun test`

Expected: all desktop tests pass.

- [ ] **Step 3: Run docs checks**

Run: `bun run docs:check`

Expected: markdown and docs link checks pass if the repo script is available.

- [ ] **Step 4: Manual approval smoke test**

Run a mutating file operation through CLI or Desktop and verify:

```text
approval event appears in the same chat
approval row is pending
pending_turn row exists
approval executes once
duplicate approval is rejected
pending_turn is deleted after terminal completion
pending_approval remains as audit/status
```

- [ ] **Step 5: Final commit**

```bash
git status --short
git commit -m "feat: complete approval resume state foundation"
```
