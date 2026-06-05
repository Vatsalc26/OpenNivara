# State Rust API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split `src/sessions.rs` into a typed, transaction-safe `src/state` module while preserving compatibility during the transition.

**Architecture:** Keep the current simple Rust style: free functions over `rusqlite::Connection`, typed input structs, typed records, and tests against temp SQLite DBs. `state::db::open_state_db()` owns DB opening and migrations; focused modules own sessions, messages, active sessions, and approvals. Multi-step approval creation is a single transaction that writes the approval row, pending-turn row, and event message together.

**Tech Stack:** Rust 2021, `rusqlite`, `refinery`, `chrono`, `serde`, `serde_json`, `uuid`, `tempfile`, `serial_test`.

---

## File Structure

- Create `src/state/mod.rs`: exports `db`, `migrations`, `types`, `sessions`, `messages`, `active_sessions`, and `approvals`.
- Create `src/state/types.rs`: shared typed records and input structs.
- Create `src/state/db.rs`: state DB path, `open_state_db()`, and legacy alpha backup/reset.
- Create `src/state/migrations.rs`: embedded refinery runner.
- Create `src/state/sessions.rs`: typed session CRUD.
- Create `src/state/messages.rs`: typed message and event-message APIs.
- Create `src/state/active_sessions.rs`: active session lookup by `(actor_id, surface)`.
- Create `src/state/approvals.rs`: approval transaction creation, execution guard, denial, terminal helpers, pending-turn cleanup.
- Modify `src/lib.rs`: expose `pub mod state;`.
- Modify `src/sessions.rs`: compatibility wrapper over new `state` modules where needed.
- Add migration SQL under `src/state/migrations/`.

## Task 1: Module Skeleton And Shared Types

**Files:**

- Create: `src/state/mod.rs`
- Create: `src/state/types.rs`
- Create: `src/state/db.rs`
- Create: `src/state/migrations.rs`
- Create: `src/state/sessions.rs`
- Create: `src/state/messages.rs`
- Create: `src/state/active_sessions.rs`
- Create: `src/state/approvals.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write the failing compile test**

Add this to `src/state/mod.rs`:

```rust
pub mod active_sessions;
pub mod approvals;
pub mod db;
pub mod messages;
pub mod migrations;
pub mod sessions;
pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn state_modules_are_available() {
        let _ = super::db::STATE_DB_FILE_NAME;
        let _ = super::migrations::STATE_MIGRATION_COUNT;
        let _ = super::types::MessageRole::Event.as_str();
    }
}
```

- [ ] **Step 2: Run the test and confirm failure**

Run: `cargo test state_modules_are_available`

Expected: fail because `src/state` modules are not wired.

- [ ] **Step 3: Add the shared types**

Create `src/state/types.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Surface {
    Desktop,
    Cli,
    Telegram,
}

impl Surface {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Cli => "cli",
            Self::Telegram => "telegram",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    Tool,
    Event,
    System,
}

impl MessageRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
            Self::Event => "event",
            Self::System => "system",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub id: String,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub status: String,
    pub surface_created: String,
    pub actor_id_created: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSessionInput {
    pub title: Option<String>,
    pub surface_created: Surface,
    pub actor_id_created: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DbMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub surface: String,
    pub actor_id: Option<String>,
    pub content: String,
    pub created_at: String,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreMessageInput {
    pub session_id: String,
    pub role: MessageRole,
    pub surface: Surface,
    pub actor_id: Option<String>,
    pub content: String,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreEventMessageInput {
    pub session_id: String,
    pub surface: Surface,
    pub actor_id: Option<String>,
    pub event_json: String,
    pub metadata_json: Option<String>,
}
```

- [ ] **Step 4: Add minimal module files**

Create `src/state/db.rs`:

```rust
pub const STATE_DB_FILE_NAME: &str = "opennivara_state.sqlite";
```

Create `src/state/migrations.rs`:

```rust
pub const STATE_MIGRATION_COUNT: usize = 2;
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

Create `src/state/approvals.rs`:

```rust
pub fn actor_can_approve(actor_id: &str) -> bool {
    actor_id == "desktop_owner" || actor_id == "cli_owner" || actor_id.starts_with("telegram_")
}
```

Add to `src/lib.rs`:

```rust
pub mod state;
```

- [ ] **Step 5: Run the test**

Run: `cargo test state_modules_are_available`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/lib.rs src/state
git commit -m "feat(state): add typed state module skeleton"
```

## Task 2: Embedded Migrations And DB Open

**Files:**

- Create: `src/state/migrations/V1__initial_state_schema.sql`
- Create: `src/state/migrations/V2__approval_resume.sql`
- Modify: `src/state/migrations.rs`
- Modify: `src/state/db.rs`

- [ ] **Step 1: Add migration test**

Add to `src/state/migrations.rs`:

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_db_runs_state_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();
        run_migrations(&mut conn).unwrap();
        for table in [
            "sessions",
            "messages",
            "active_sessions",
            "pending_approvals",
            "pending_turns",
            "refinery_schema_history",
        ] {
            let exists: i64 = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(exists, 1, "{table} should exist");
        }
    }
}
```

- [ ] **Step 2: Run the migration test and confirm failure**

Run: `cargo test fresh_db_runs_state_migrations`

Expected: fail until SQL migration files exist.

- [ ] **Step 3: Add V1 SQL**

Create `src/state/migrations/V1__initial_state_schema.sql` with the schema from [Approval Resume And State DB](../../architecture/approval-resume-state.md), including `sessions`, `messages`, `active_sessions`, pinned tables, role check, `surface` naming, and V1 indexes.

- [ ] **Step 4: Add V2 SQL**

Create `src/state/migrations/V2__approval_resume.sql` with `pending_approvals`, `pending_turns`, status check, foreign keys, and V2 indexes from [Approval Resume And State DB](../../architecture/approval-resume-state.md).

- [ ] **Step 5: Implement DB open and legacy reset**

Replace `src/state/db.rs` with:

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
    open_state_db_at(&state_db_path()?)
}

pub fn open_state_db_at(path: &Path) -> anyhow::Result<Connection> {
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

pub fn reset_legacy_alpha_db_if_needed(path: &Path) -> anyhow::Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }
    let conn = Connection::open(path)?;
    let has_refinery = table_exists(&conn, "refinery_schema_history")?;
    let has_old_source = column_exists(&conn, "sessions", "source_created")?;
    drop(conn);
    if has_old_source && !has_refinery {
        let backup = path.with_file_name(format!(
            "opennivara_state.legacy-reset-{}.sqlite",
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        ));
        fs::rename(path, &backup)?;
        Ok(Some(backup))
    } else {
        Ok(None)
    }
}

fn table_exists(conn: &Connection, table_name: &str) -> anyhow::Result<bool> {
    let exists: i64 = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [table_name],
        |row| row.get(0),
    )?;
    Ok(exists == 1)
}

fn column_exists(conn: &Connection, table_name: &str, column_name: &str) -> anyhow::Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for column in columns {
        if column? == column_name {
            return Ok(true);
        }
    }
    Ok(false)
}
```

- [ ] **Step 6: Add DB tests**

Add tests for `open_state_db_at_creates_parent_and_runs_migrations` and `legacy_inline_alpha_db_is_backed_up_and_reset` using `tempfile::tempdir()`.

- [ ] **Step 7: Run DB tests**

Run: `cargo test state::db state::migrations`

Expected: pass.

- [ ] **Step 8: Commit**

```bash
git add src/state
git commit -m "feat(state): add migrated state database entry point"
```

## Task 3: Sessions, Messages, And Active Sessions

**Files:**

- Modify: `src/state/sessions.rs`
- Modify: `src/state/messages.rs`
- Modify: `src/state/active_sessions.rs`
- Modify: `src/state/types.rs`

- [ ] **Step 1: Write API tests**

Add tests proving:

```rust
let session = create_session(&conn, CreateSessionInput {
    title: Some("Planning".into()),
    surface_created: Surface::Cli,
    actor_id_created: Some("cli_owner".into()),
}).unwrap();
assert_eq!(session.title.as_deref(), Some("Planning"));

let message = store_message(&conn, StoreMessageInput {
    session_id: session.id.clone(),
    role: MessageRole::User,
    surface: Surface::Cli,
    actor_id: Some("cli_owner".into()),
    content: "hello".into(),
    metadata_json: None,
}).unwrap();
assert_eq!(message.role, "user");

set_active_session(&conn, "cli_owner", Surface::Cli, &session.id).unwrap();
assert_eq!(
    get_active_session(&conn, "cli_owner", Surface::Cli).unwrap(),
    Some(session.id)
);
```

Also test invalid roles by inserting a bad raw SQL row and asserting SQLite rejects it.

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test state::sessions state::messages state::active_sessions`

Expected: fail because helpers are not implemented.

- [ ] **Step 3: Implement sessions helpers**

Implement `create_session`, `get_session`, `list_sessions`, `close_session`, and `rename_session`. `create_session` should generate a UUID, set `created_at` and `updated_at`, insert `status = 'active'`, and return the typed row loaded by `get_session`.

- [ ] **Step 4: Implement messages helpers**

Implement `store_message`, `get_session_messages`, and `store_event_message`. `store_event_message` must set `role = 'event'`, store JSON in `content`, and return the typed row.

- [ ] **Step 5: Implement active session helpers**

Implement `set_active_session` and `get_active_session` using `actor_id`, `surface`, and `session_id`.

- [ ] **Step 6: Run API tests**

Run: `cargo test state::sessions state::messages state::active_sessions`

Expected: pass.

- [ ] **Step 7: Commit**

```bash
git add src/state/sessions.rs src/state/messages.rs src/state/active_sessions.rs src/state/types.rs
git commit -m "feat(state): add typed session message APIs"
```

## Task 4: Transactional Approval API

**Files:**

- Modify: `src/state/approvals.rs`
- Modify: `src/state/types.rs`

- [ ] **Step 1: Add approval types**

Add `CreatePendingApprovalInput`, `PendingTurnState`, `PendingApproval`, `DeniedApproval`, and `BeginExecutionResult` to `src/state/types.rs`. `PendingTurnState` should derive `Serialize` and `Deserialize` and include:

```rust
pub request_envelope: serde_json::Value,
pub session_id: String,
pub user_message_id: String,
pub model_messages_so_far: serde_json::Value,
pub declared_tools: serde_json::Value,
pub pending_tool_call: serde_json::Value,
pub compiled_context_audit_id: Option<String>,
pub selected_skill_ids: Vec<String>,
pub pinned_context_ids: Vec<String>,
pub provider_id: String,
pub model_id: String,
pub generation_config: serde_json::Value,
pub provider_state_json: serde_json::Value,
```

- [ ] **Step 2: Write transactional creation test**

Assert one call creates all three rows:

```rust
let approval = create_pending_approval_with_turn(&mut conn, input, turn).unwrap();
assert_eq!(approval.status, "pending");
assert_eq!(count_rows(&conn, "pending_approvals"), 1);
assert_eq!(count_rows(&conn, "pending_turns"), 1);
assert_eq!(count_event_messages(&conn, "approval_required"), 1);
```

Add rollback test by passing invalid `arguments_preview_json` only after implementing JSON validation or by passing a missing `user_message_id` so the foreign key fails:

```rust
let result = create_pending_approval_with_turn(&mut conn, bad_input, turn);
assert!(result.is_err());
assert_eq!(count_rows(&conn, "pending_approvals"), 0);
assert_eq!(count_rows(&conn, "pending_turns"), 0);
assert_eq!(count_rows(&conn, "messages"), 1);
```

- [ ] **Step 3: Run tests and confirm failure**

Run: `cargo test create_pending_approval_with_turn`

Expected: fail because approval API is not implemented.

- [ ] **Step 4: Implement transactional creation**

Use `let tx = conn.transaction()?;`, insert `pending_approvals`, insert `pending_turns`, insert an approval-required event message, then `tx.commit()?`.

- [ ] **Step 5: Run transactional tests**

Run: `cargo test create_pending_approval_with_turn`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/state/approvals.rs src/state/types.rs
git commit -m "feat(state): create approval turns atomically"
```

## Task 5: Execution Guard, Denial, And Cleanup

**Files:**

- Modify: `src/state/approvals.rs`
- Modify: `src/state/types.rs`

- [ ] **Step 1: Write lifecycle tests**

Add tests proving:

- `begin_execution_once` returns `Started(PendingTurnState)` for the correct session and allowed actor.
- A duplicate call returns `AlreadyResolved`.
- A wrong session returns `WrongSession`.
- A disallowed actor returns `ActorNotAllowed`.
- `deny_approval` marks denied, sets `resolved_by_actor_id`, inserts `approval_denied`, and returns denied-tool-result data.
- `mark_executed` stores `result_summary`.
- `mark_failed` stores `error_message`.
- `delete_pending_turn` removes the resume blob and preserves `pending_approvals`.
- Approval and pending turn survive DB reopen until cleanup.

- [ ] **Step 2: Run lifecycle tests and confirm failure**

Run: `cargo test state::approvals`

Expected: fail for missing lifecycle helpers.

- [ ] **Step 3: Implement lifecycle helpers**

Implement:

```rust
pub fn begin_execution_once(
    conn: &Connection,
    approval_id: &str,
    session_id: &str,
    approving_actor_id: &str,
) -> anyhow::Result<BeginExecutionResult>;

pub fn mark_executed(
    conn: &Connection,
    approval_id: &str,
    result_summary: Option<&str>,
) -> anyhow::Result<()>;

pub fn mark_failed(
    conn: &Connection,
    approval_id: &str,
    error_message: &str,
) -> anyhow::Result<()>;

pub fn deny_approval(
    conn: &Connection,
    approval_id: &str,
    session_id: &str,
    actor_id: &str,
) -> anyhow::Result<DeniedApproval>;

pub fn delete_pending_turn(conn: &Connection, approval_id: &str) -> anyhow::Result<()>;
```

- [ ] **Step 4: Run lifecycle tests**

Run: `cargo test state::approvals`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/state/approvals.rs src/state/types.rs
git commit -m "feat(state): add approval execution lifecycle"
```

## Task 6: Compatibility Wrapper

**Files:**

- Modify: `src/sessions.rs`

- [ ] **Step 1: Add compatibility tests**

Keep existing callers passing by testing:

```rust
let conn = init_db().unwrap();
let session_id = create_session(&conn, "cli", Some("Compat")).unwrap();
store_message(&conn, &session_id, "user", "cli", "hello", None).unwrap();
assert_eq!(get_session_messages(&conn, &session_id).unwrap().len(), 1);
```

- [ ] **Step 2: Run tests and confirm current behavior**

Run: `cargo test sessions`

Expected: existing tests pass or reveal call sites that need compatibility.

- [ ] **Step 3: Delegate where practical**

Make `init_db()` call `state::db::open_state_db()`. Convert compatibility inputs like `"cli"` into `Surface::Cli` where direct mapping exists. Keep narrow legacy wrappers only where old callers still expect raw strings.

- [ ] **Step 4: Run full Rust tests**

Run: `cargo test`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/sessions.rs src/state
git commit -m "refactor(state): route session compatibility through state module"
```

## Final Verification

- [ ] **Step 1: Run Rust tests**

Run: `cargo test`

Expected: all Rust tests pass.

- [ ] **Step 2: Run docs checks**

Run: `bun run docs:check`

Expected: markdown and internal docs links pass.

- [ ] **Step 3: Inspect state call sites**

Run: `rg -n "INSERT INTO (sessions|messages|pending_approvals|pending_turns)|UPDATE pending_approvals|source_created|user_key" src`

Expected: remaining hits are in migrations, state modules, tests, or intentional compatibility wrappers.
