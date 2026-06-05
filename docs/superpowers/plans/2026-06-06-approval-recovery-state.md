# Approval Recovery State Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make approval resume safe across crashes, provider failures, duplicate approvals, and partial execution states.

**Architecture:** Treat `executed` as an intermediate state meaning the tool already ran, and `completed` as the terminal success state meaning the resumed model turn finished and final assistant message was stored. Store recovery phase in `pending_turns.phase` for SQL recovery scans. Retry only provider/model continuation after tool success; never re-execute a tool once status reaches `executed`.

**Tech Stack:** Rust 2021, SQLite/refinery, `rusqlite`, `chrono`, `serde_json`, existing `state::approvals`, engine resume flow, model provider gateway.

---

## File Structure

- Modify `src/state/migrations/V2__approval_resume.sql`: add `completed` status, recovery columns, `pending_turns.phase`, `updated_at`, and recovery indexes.
- Modify `src/state/types.rs`: add `ApprovalStatus::Completed`, pending turn phase enum, resume failure fields, and recovery inputs/outputs.
- Modify `src/state/approvals.rs`: add recovery transition helpers.
- Modify `src/engine.rs`: call recovery helpers during approved/denied resume and provider failure handling.
- Add tests under `src/state` and `src/engine.rs`.

## Task 1: Migration Schema Recovery Fields

**Files:**

- Modify: `src/state/migrations/V2__approval_resume.sql`
- Modify: state migration tests

- [ ] **Step 1: Add failing schema tests**

Add tests:

```rust
#[test]
fn approval_status_accepts_completed_and_rejects_invalid() {
    let mut conn = Connection::open_in_memory().unwrap();
    run_migrations(&mut conn).unwrap();
    seed_session_and_message(&conn);

    for status in ["pending", "denied", "executing", "executed", "failed", "completed"] {
        conn.execute(
            "INSERT INTO pending_approvals (
                id, session_id, request_id, user_message_id, tool_call_id, surface, actor_id,
                operation_name, classification, status, created_at
            ) VALUES (?1, 's1', 'r1', 'm1', ?1, 'cli', 'cli_owner', 'tool', 'local_modify', ?2, '2026-06-06T00:00:00Z')",
            rusqlite::params![format!("a_{status}"), status],
        )
        .unwrap();
    }

    let invalid = conn.execute(
        "INSERT INTO pending_approvals (
            id, session_id, request_id, user_message_id, tool_call_id, surface, actor_id,
            operation_name, classification, status, created_at
        ) VALUES ('bad', 's1', 'r1', 'm1', 'tc_bad', 'cli', 'cli_owner', 'tool', 'local_modify', 'done', '2026-06-06T00:00:00Z')",
        [],
    );
    assert!(invalid.is_err());
}

#[test]
fn pending_turn_phase_check_accepts_recovery_phases() {
    let mut conn = Connection::open_in_memory().unwrap();
    run_migrations(&mut conn).unwrap();
    seed_session_message_and_approval(&conn);

    for phase in ["awaiting_approval", "tool_executed_awaiting_model", "denied_awaiting_model"] {
        conn.execute(
            "INSERT INTO pending_turns (
                approval_id, session_id, request_id, user_message_id, provider_id, model_id,
                phase, resume_payload_json, created_at, updated_at
            ) VALUES (?1, 's1', 'r1', 'm1', 'gemini', 'gemini-2.5-flash', ?2, '{}', '2026-06-06T00:00:00Z', '2026-06-06T00:00:00Z')",
            rusqlite::params![format!("a_{phase}"), phase],
        )
        .unwrap();
    }
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test approval_status_accepts_completed pending_turn_phase_check_accepts_recovery_phases`

Expected: fail because schema does not include `completed` and phase.

- [ ] **Step 3: Update V2 schema**

Change `pending_approvals.status` check to include:

```sql
'completed'
```

Add:

```sql
completed_at TEXT,
resume_attempt_count INTEGER NOT NULL DEFAULT 0,
last_resume_error TEXT,
last_resume_attempt_at TEXT,
```

Add to `pending_turns`:

```sql
phase TEXT NOT NULL CHECK(phase IN (
    'awaiting_approval',
    'tool_executed_awaiting_model',
    'denied_awaiting_model'
)),
updated_at TEXT NOT NULL,
```

Add indexes:

```sql
CREATE INDEX idx_pending_approvals_operation_target ON pending_approvals(operation_target);
CREATE INDEX idx_pending_turns_phase ON pending_turns(phase);
```

- [ ] **Step 4: Run schema tests**

Run: `cargo test approval_status_accepts_completed pending_turn_phase_check_accepts_recovery_phases`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/state/migrations src/state/migrations.rs
git commit -m "feat(state): add approval recovery schema"
```

## Task 2: Recovery State Types

**Files:**

- Modify: `src/state/types.rs`

- [ ] **Step 1: Add type tests**

Add:

```rust
#[test]
fn approval_status_completed_serializes_to_schema_value() {
    assert_eq!(ApprovalStatus::Completed.as_str(), "completed");
}

#[test]
fn pending_turn_phase_serializes_to_schema_values() {
    assert_eq!(PendingTurnPhase::AwaitingApproval.as_str(), "awaiting_approval");
    assert_eq!(
        PendingTurnPhase::ToolExecutedAwaitingModel.as_str(),
        "tool_executed_awaiting_model"
    );
    assert_eq!(
        PendingTurnPhase::DeniedAwaitingModel.as_str(),
        "denied_awaiting_model"
    );
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test approval_status_completed_serializes_to_schema_value pending_turn_phase_serializes_to_schema_values`

Expected: fail until types are updated.

- [ ] **Step 3: Add types**

Add:

```rust
pub enum ApprovalStatus {
    Pending,
    Denied,
    Executing,
    Executed,
    Failed,
    Completed,
}

pub enum PendingTurnPhase {
    AwaitingApproval,
    ToolExecutedAwaitingModel,
    DeniedAwaitingModel,
}
```

Add `as_str()` implementations matching SQL values.

- [ ] **Step 4: Run type tests**

Run: `cargo test approval_status_completed_serializes_to_schema_value pending_turn_phase_serializes_to_schema_values`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/state/types.rs
git commit -m "feat(state): add approval recovery types"
```

## Task 3: Atomic Tool Executed Transition

**Files:**

- Modify: `src/state/approvals.rs`

- [ ] **Step 1: Add atomic transition test**

Add:

```rust
#[test]
fn mark_tool_executed_and_update_turn_is_atomic() {
    let conn = setup_pending_executing_approval();
    mark_tool_executed_and_update_turn(
        &conn,
        "a1",
        r#"{"messages_so_far":["tool_result_appended"]}"#,
        Some("Wrote src/main.rs"),
    )
    .unwrap();

    let (status, result_summary): (String, Option<String>) = conn
        .query_row(
            "SELECT status, result_summary FROM pending_approvals WHERE id = 'a1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let phase: String = conn
        .query_row("SELECT phase FROM pending_turns WHERE approval_id = 'a1'", [], |row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(status, "executed");
    assert_eq!(phase, "tool_executed_awaiting_model");
    assert_eq!(result_summary.as_deref(), Some("Wrote src/main.rs"));
}
```

- [ ] **Step 2: Run test and confirm failure**

Run: `cargo test mark_tool_executed_and_update_turn_is_atomic`

Expected: fail because helper is missing.

- [ ] **Step 3: Implement helper**

Implement `mark_tool_executed_and_update_turn` using a transaction. It must update `pending_approvals` and `pending_turns` together. If either update fails, neither persists.

- [ ] **Step 4: Run transition test**

Run: `cargo test mark_tool_executed_and_update_turn_is_atomic`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/state/approvals.rs
git commit -m "feat(state): atomically mark tool executed"
```

## Task 4: Resume Failure And Completion Helpers

**Files:**

- Modify: `src/state/approvals.rs`

- [ ] **Step 1: Add helper tests**

Add tests for:

```rust
mark_resume_failed(&conn, "a1", "provider timed out").unwrap();
mark_approval_completed(&conn, "a1").unwrap();
cleanup_completed_pending_turns(&conn).unwrap();
```

Assert:

- `resume_attempt_count` increments.
- `last_resume_error` is stored.
- status remains `executed` after resume failure.
- `completed_at` is set.
- status becomes `completed`.
- pending turn is deleted.
- approval row remains.

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test mark_resume_failed mark_approval_completed cleanup_completed_pending_turns`

Expected: fail because helpers are missing.

- [ ] **Step 3: Implement helpers**

Implement:

```rust
mark_resume_failed(conn, approval_id, error_message)
mark_approval_completed(conn, approval_id)
cleanup_completed_pending_turns(conn)
```

- [ ] **Step 4: Run helper tests**

Run: `cargo test mark_resume_failed mark_approval_completed cleanup_completed_pending_turns`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/state/approvals.rs
git commit -m "feat(state): track approval resume recovery"
```

## Task 5: Stale Executing Recovery

**Files:**

- Modify: `src/state/approvals.rs`

- [ ] **Step 1: Add stale executing test**

Add:

```rust
#[test]
fn stale_executing_is_marked_failed_without_retry() {
    let conn = setup_stale_executing_approval("2026-06-06T00:00:00Z");
    let changed = mark_stale_executing_as_failed(
        &conn,
        chrono::DateTime::parse_from_rfc3339("2026-06-06T00:11:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
        chrono::Duration::minutes(10),
    )
    .unwrap();

    assert_eq!(changed, 1);
    let (status, error): (String, String) = conn
        .query_row(
            "SELECT status, error_message FROM pending_approvals WHERE id = 'a1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(status, "failed");
    assert!(error.contains("interrupted"));
}
```

- [ ] **Step 2: Run test and confirm failure**

Run: `cargo test stale_executing_is_marked_failed_without_retry`

Expected: fail until helper exists.

- [ ] **Step 3: Implement helper**

Implement `mark_stale_executing_as_failed(conn, now, threshold) -> anyhow::Result<usize>`.

It should mark stale `executing` approvals failed with:

```text
Execution was interrupted before completion could be confirmed.
```

It must not execute or retry any tool.

- [ ] **Step 4: Run stale recovery test**

Run: `cargo test stale_executing_is_marked_failed_without_retry`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/state/approvals.rs
git commit -m "feat(state): fail stale executing approvals"
```

## Task 6: Engine Resume Recovery

**Files:**

- Modify: `src/engine.rs`

- [ ] **Step 1: Add engine recovery tests**

Add tests proving:

- after tool success, provider failure leaves status `executed`
- retry from status `executed` does not execute tool again
- retry only calls provider/model continuation
- successful retry marks approval `completed`

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test approval_resume_recovery`

Expected: fail until engine uses recovery helpers.

- [ ] **Step 3: Update approved resume flow**

After approved tool execution succeeds, call `mark_tool_executed_and_update_turn` before provider continuation.

If provider continuation fails, call `mark_resume_failed` and return the provider error without changing status away from `executed`.

On retry where status is `executed` and phase is `tool_executed_awaiting_model`, skip tool execution and resume provider continuation from stored history.

After final assistant answer is stored, call `mark_approval_completed` and delete pending turn.

- [ ] **Step 4: Run engine recovery tests**

Run: `cargo test approval_resume_recovery`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/engine.rs
git commit -m "feat(engine): recover approval resume after tool execution"
```

## Final Verification

- [ ] **Step 1: Run state tests**

Run: `cargo test state`

Expected: state migration and approval recovery tests pass.

- [ ] **Step 2: Run engine tests**

Run: `cargo test engine`

Expected: engine recovery tests pass.

- [ ] **Step 3: Run full Rust tests**

Run: `cargo test`

Expected: all Rust tests pass.

- [ ] **Step 4: Run docs checks**

Run: `bun run docs:check`

Expected: markdown and internal docs links pass.
