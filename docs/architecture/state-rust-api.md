# State Rust API

This document defines the internal Rust API shape for OpenNivara state storage. It is not an HTTP API and should stay close to the current code style: focused free functions over `rusqlite::Connection`, typed inputs, typed records, and direct unit tests against temporary SQLite databases.

The current state/session code is concentrated in `src/sessions.rs`. That file owns state DB path resolution, inline schema creation, sessions, messages, active sessions, pending approvals, pinned contexts, and pinned skills. The target is a proper `src/state` module backed by embedded refinery migrations, while keeping the API simple enough for engine, desktop, CLI, and Telegram code to call directly.

## Principles

Use free functions in focused modules. Do not introduce a large `StateStore` singleton or service object yet.

State functions should accept `&Connection` or `&mut Connection`. They should not hide `open_state_db()` inside every helper, because callers often need multiple state operations in one turn or transaction.

Use typed input structs and typed output structs. Return typed records where useful instead of only raw IDs.

Multi-step writes must use transactions. Approval creation is the clearest example: the approval row, pending-turn row, and chat event message must commit or roll back together.

New engine, desktop, CLI, and Telegram code should call `src/state` functions. They should not manually insert or update state rows.

Keep `src/sessions.rs` as a compatibility wrapper if needed during the transition, but shrink it over time.

## Target Layout

```text
src/state/
  mod.rs
  db.rs
  migrations.rs
  types.rs
  sessions.rs
  messages.rs
  active_sessions.rs
  approvals.rs
  migrations/
    V1__initial_state_schema.sql
    V2__approval_resume.sql
```

Responsibilities:

- `db.rs`: path resolution, DB open, foreign keys, legacy alpha backup/reset.
- `migrations.rs`: embedded refinery migration runner.
- `types.rs`: shared state types and input structs.
- `sessions.rs`: session CRUD.
- `messages.rs`: user, assistant, tool, event, and system message storage.
- `active_sessions.rs`: active session lookup by actor and surface.
- `approvals.rs`: approval creation, pending-turn storage, execution guard, denial, terminal status helpers, and pending-turn cleanup.

## DB Entry Point

The state DB entry point is:

```rust
state::db::open_state_db() -> anyhow::Result<rusqlite::Connection>
```

`open_state_db()` must:

1. Resolve `opennivara_state.sqlite`.
2. Create the parent directory.
3. Detect an old inline alpha DB without refinery metadata.
4. Backup/reset the old alpha DB.
5. Open the SQLite connection.
6. Enable `PRAGMA foreign_keys = ON`.
7. Run embedded refinery migrations.
8. Return the connection.

`state::db` API:

```rust
pub fn state_db_path() -> anyhow::Result<PathBuf>;
pub fn open_state_db() -> anyhow::Result<rusqlite::Connection>;
pub fn reset_legacy_alpha_db_if_needed(path: &Path) -> anyhow::Result<Option<PathBuf>>;
```

`reset_legacy_alpha_db_if_needed` returns the backup path when it resets an old inline alpha DB and `None` when no reset was needed.

## Shared Types

`Surface` is the shared surface enum:

```rust
pub enum Surface {
    Desktop,
    Cli,
    Telegram,
}
```

The serialized DB value should be stable and human-readable, such as `desktop`, `cli`, and `telegram`.

`MessageRole` mirrors the SQL role constraint:

```rust
pub enum MessageRole {
    User,
    Assistant,
    Tool,
    Event,
    System,
}
```

SQL must enforce:

```sql
CHECK(role IN ('user', 'assistant', 'tool', 'event', 'system'))
```

## Sessions API

Module: `state::sessions`

Functions:

```rust
pub fn create_session(conn: &Connection, input: CreateSessionInput) -> anyhow::Result<Session>;
pub fn get_session(conn: &Connection, session_id: &str) -> anyhow::Result<Option<Session>>;
pub fn list_sessions(conn: &Connection) -> anyhow::Result<Vec<Session>>;
pub fn close_session(conn: &Connection, session_id: &str) -> anyhow::Result<()>;
pub fn rename_session(conn: &Connection, session_id: &str, title: &str) -> anyhow::Result<()>;
```

Input:

```rust
pub struct CreateSessionInput {
    pub title: Option<String>,
    pub surface_created: Surface,
    pub actor_id_created: Option<String>,
}
```

`create_session` returns the inserted `Session`, not only the ID.

## Messages API

Module: `state::messages`

Functions:

```rust
pub fn store_message(conn: &Connection, input: StoreMessageInput) -> anyhow::Result<DbMessage>;
pub fn get_session_messages(conn: &Connection, session_id: &str) -> anyhow::Result<Vec<DbMessage>>;
pub fn store_event_message(conn: &Connection, input: StoreEventMessageInput) -> anyhow::Result<DbMessage>;
```

Input:

```rust
pub struct StoreMessageInput {
    pub session_id: String,
    pub role: MessageRole,
    pub surface: Surface,
    pub actor_id: Option<String>,
    pub content: String,
    pub metadata_json: Option<String>,
}
```

Approval events are normal messages with `role = MessageRole::Event`. Event message `content` is JSON. `metadata_json` may still hold auxiliary UI or debug metadata.

## Active Sessions API

Module: `state::active_sessions`

Functions:

```rust
pub fn set_active_session(
    conn: &Connection,
    actor_id: &str,
    surface: Surface,
    session_id: &str,
) -> anyhow::Result<()>;

pub fn get_active_session(
    conn: &Connection,
    actor_id: &str,
    surface: Surface,
) -> anyhow::Result<Option<String>>;
```

The schema uses `(actor_id, surface)` as the primary key. Do not continue the old `user_key` model in the clean schema.

## Approvals API

Module: `state::approvals`

The engine should not manually insert into `pending_approvals`, `pending_turns`, and `messages` separately. Approval creation should be one high-level atomic operation:

```rust
pub fn create_pending_approval_with_turn(
    conn: &mut Connection,
    input: CreatePendingApprovalInput,
    turn: PendingTurnState,
) -> anyhow::Result<PendingApproval>;
```

This function creates, in one transaction:

1. `pending_approvals` row.
2. `pending_turns` row.
3. `role = 'event'` approval-required message.

If any write fails, none of the three writes are committed.

Input:

```rust
pub struct CreatePendingApprovalInput {
    pub session_id: String,
    pub request_id: String,
    pub user_message_id: String,
    pub tool_call_id: String,
    pub surface: Surface,
    pub actor_id: String,
    pub operation_name: String,
    pub classification: String,
    pub summary: Option<String>,
    pub operation_target: Option<String>,
    pub reason: String,
    pub arguments_preview_json: Option<String>,
}
```

`PendingTurnState` is stored as JSON in `pending_turns.resume_payload_json` and should include the request envelope, session ID, user message ID, model messages so far, declared tools, pending tool call, compiled context audit ID, selected skill IDs, pinned context IDs, provider ID, model ID, generation config, and provider/model state required to resume the same turn.

## Execution And Denial API

Approval execution starts with an atomic guard:

```rust
pub fn begin_execution_once(
    conn: &Connection,
    approval_id: &str,
    session_id: &str,
    approving_actor_id: &str,
) -> anyhow::Result<BeginExecutionResult>;
```

Responsibilities:

- Validate same session/chat context.
- Validate the approving actor has approval permission.
- Atomically transition `pending` to `executing`.
- Load and return `PendingTurnState` only if the transition succeeds.
- Prevent duplicate execution.

SQL transition:

```sql
UPDATE pending_approvals
SET status = 'executing', execution_started_at = ?
WHERE id = ?
  AND session_id = ?
  AND status = 'pending';
```

Result enum:

```rust
pub enum BeginExecutionResult {
    Started(PendingTurnState),
    NotFound,
    WrongSession,
    AlreadyResolved,
    ActorNotAllowed,
}
```

Terminal helpers:

```rust
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

`deny_approval` marks status `denied`, stores `resolved_at` and `resolved_by_actor_id`, inserts an event message, and returns enough data to create a model-visible denied tool result.

Approval details shown to surfaces should be returned as `ApprovalView`:

```rust
pub struct ApprovalView {
    pub approval_id: String,
    pub session_id: String,
    pub operation_name: String,
    pub classification: String,
    pub summary: String,
    pub operation_target: Option<String>,
    pub reason: String,
    pub preview_json: serde_json::Value,
    pub full_arguments_json: serde_json::Value,
}
```

`preview_json` comes from `ToolPreview.preview_json`; `full_arguments_json` comes from pending turn state while the approval is pending.

Delete `pending_turns` after terminal completion/resume is fully handled. Keep `pending_approvals` as the smaller audit/status record.

## Actor Permission

Do not add an actors table yet.

Approval permission can be hardcoded around valid owner actor IDs for now:

- `desktop_owner`
- `cli_owner`
- `telegram_<chat_id>`

Approval permission is not global authorization. The approval must also match the same session/chat context.

## Event Messages

Approval lifecycle helpers should insert event messages for:

- `approval_required`
- `approval_approved`
- `approval_denied`
- `approval_executed`
- `approval_failed`

Event messages use `role = 'event'` and JSON `content`.

## Required API Tests

Add tests for:

1. `state::db` opens a fresh migrated DB.
2. Legacy inline alpha DB is backed up/reset.
3. `create_session` returns a typed `Session`.
4. `store_message` returns a typed `DbMessage`.
5. Invalid message roles are rejected by SQL.
6. `active_sessions` maps `(actor_id, surface)` to session.
7. `create_pending_approval_with_turn` inserts approval, pending turn, and event message atomically.
8. Failed approval creation rolls back all three writes.
9. `begin_execution_once` transitions `pending` to `executing` and returns `PendingTurnState`.
10. Duplicate `begin_execution_once` does not execute twice.
11. Wrong session cannot approve.
12. Actor without approval permission cannot approve.
13. `deny_approval` marks denied and inserts an event.
14. `mark_executed` stores `result_summary` and allows pending-turn cleanup.
15. `mark_failed` stores `error_message` and allows pending-turn cleanup.
16. `delete_pending_turn` removes the resume blob while preserving the approval row.
17. Pending approval and pending turn survive DB reopen until terminal cleanup.

## Recovery-Safe Transition API Update

Do not implement approval status as generic free-form updates. Use explicit transition functions.

The core invariant is: once an approved tool has executed, OpenNivara must never execute that same tool call again.

Required functions:

```rust
pub fn create_pending_approval_with_turn(
    conn: &mut Connection,
    input: CreatePendingApprovalInput,
    turn: PendingTurnState,
    preview: ToolPreview,
) -> anyhow::Result<PendingApproval>;

pub fn begin_execution_once(
    conn: &mut Connection,
    approval_id: &str,
    session_id: &str,
    approving_actor_id: &str,
) -> anyhow::Result<BeginExecutionResult>;

pub fn mark_tool_executed_and_update_turn(
    conn: &mut Connection,
    input: MarkToolExecutedInput,
) -> anyhow::Result<PendingTurnState>;

pub fn mark_tool_failed(
    conn: &mut Connection,
    input: MarkToolFailedInput,
) -> anyhow::Result<()>;

pub fn mark_approval_completed(
    conn: &mut Connection,
    approval_id: &str,
    final_assistant_message_id: &str,
) -> anyhow::Result<()>;

pub fn mark_resume_failed(
    conn: &Connection,
    approval_id: &str,
    error_message: &str,
) -> anyhow::Result<()>;

pub fn deny_approval_and_update_turn(
    conn: &mut Connection,
    input: DenyApprovalInput,
) -> anyhow::Result<DenyApprovalResult>;

pub fn complete_denied_turn(
    conn: &mut Connection,
    approval_id: &str,
    final_assistant_message_id: &str,
) -> anyhow::Result<()>;

pub fn cleanup_completed_pending_turns(conn: &Connection) -> anyhow::Result<usize>;

pub fn recover_stale_executing_approvals(
    conn: &mut Connection,
    stale_after: chrono::Duration,
) -> anyhow::Result<usize>;
```

`create_pending_approval_with_turn` inserts `pending_approvals(status = pending)`, `pending_turns(phase = awaiting_approval)`, and an `approval_required` event in one transaction.

`begin_execution_once` is the only function that can move `pending` to `executing`. Tool execution may start only if it returns `Started`.

`BeginExecutionResult` must distinguish:

- `Started { approval, turn }`
- `NotFound`
- `WrongSession`
- `ActorNotAllowed`
- `AlreadyDenied`
- `AlreadyExecuting`
- `AlreadyExecuted`
- `AlreadyFailed`
- `AlreadyCompleted`
- `MissingPendingTurn`
- `InvalidPhase`

`mark_tool_executed_and_update_turn` is the critical crash-safe transition. It is allowed only when approval status is `executing` and pending turn phase is `awaiting_approval`. It must atomically set status `executed`, set `execution_finished_at`, set `result_summary`, set phase `tool_executed_awaiting_model`, update `resume_payload_json` with the tool result already appended, update `updated_at`, and insert `approval_executed`.

`mark_resume_failed` is allowed for `executed/tool_executed_awaiting_model` and `denied/denied_awaiting_model`. It increments `resume_attempt_count`, stores `last_resume_error`, stores `last_resume_attempt_at`, and keeps the pending turn.

`deny_approval_and_update_turn` is allowed only for `pending/awaiting_approval`. It atomically sets status `denied`, sets `resolved_at`, sets `resolved_by_actor_id`, sets phase `denied_awaiting_model`, appends denied tool result into `resume_payload_json`, and inserts `approval_denied`.

`complete_denied_turn` deletes `pending_turn` and keeps `pending_approvals.status = denied`. Do not change denied to completed.

`recover_stale_executing_approvals` should use a 10 minute default threshold and mark interrupted `executing` approvals as `failed` with `Execution was interrupted before completion could be confirmed.` Do not retry tool execution.

Additional required tests:

1. `mark_tool_executed_and_update_turn` atomically sets status `executed` and phase `tool_executed_awaiting_model`.
2. once status is `executed`, tool execution cannot happen again.
3. `mark_resume_failed` does not change `executed` status.
4. provider continuation retry starts from stored tool result, not from tool execution.
5. `mark_approval_completed` sets `completed_at` and deletes pending turn.
6. denied remains denied after model explanation.
7. `complete_denied_turn` deletes pending turn and keeps denied status.
8. stale executing recovery marks approvals failed/interrupted.
9. forbidden transitions are rejected.
