# Approval Resume And State DB

This document defines how OpenNivara stores approval-required tool calls, pauses the same agent turn, and resumes it after approval or denial.

It builds on the [Core Agent Contract](core-agent-contract.md). The product remains intentionally liberal: automatic operations run without approval, while deleting, modifying, external mutation, mutating shell commands, deleting shell commands, unknown shell commands, and unknown operations require per-operation approval.

## Decisions

Approval pauses and resumes the same agent turn. Approval does not stop the task and ask the user to retry.

Approval never expires. It is approve-once only, per operation, and single-use. One approval equals one operation and one execution. Approval cannot be replayed, and duplicate approvals must not execute the operation twice.

Pending approvals survive app restart and can resume later. Pending approvals should appear in the same chat history. Approval is tied to the particular chat/session where it was requested. Same-chat approval is required unless a future explicit cross-surface approval design is created.

Do not allow any surface to approve any pending operation globally. The approving actor must have approve permission and must approve within the correct chat/session approval context.

Denial is represented as a tool result so the model can continue or explain.

## Existing State DB Context

The current state DB is initialized inline in `src/sessions.rs` with `CREATE TABLE IF NOT EXISTS`. The current `pending_approvals` table is small and metadata-focused:

```text
pending_approvals
- id
- session_id
- source
- request_json
- status
- created_at
- expires_at
```

The current helper functions only create a pending approval and update its status. This table is not sufficient for full same-turn resume.

The memory DB already uses a migration style through `migrations::run_migrations`, and the root `Cargo.toml` already includes `refinery`. State DB schema changes should move to a proper migration system instead of continuing to evolve the schema only through inline `CREATE TABLE` statements.

## Pending Turn State

Use Option A: store the full pending turn state.

The full pending turn state is stored only for the particular chat/session where the approval was requested. Approving the operation resumes that same chat turn only. It must not resume another session or act as a global approval.

`PendingTurnState` should store enough to resume without asking the model to recreate the task:

- request envelope
- session ID
- user message ID
- model messages so far
- declared tools
- pending tool call
- compiled context audit ID
- selected skill IDs
- pinned context IDs
- provider ID
- model ID
- generation config
- any other model or provider state needed to continue the same turn

## Database Design

Use two tables, not one overloaded table.

`pending_approvals` stores searchable metadata and status:

```text
pending_approvals
- id TEXT PRIMARY KEY
- session_id TEXT NOT NULL
- surface TEXT NOT NULL
- actor_id TEXT NOT NULL
- operation_name TEXT NOT NULL
- classification TEXT NOT NULL
- status TEXT NOT NULL
- summary TEXT
- arguments_preview_json TEXT
- created_at TEXT NOT NULL
- resolved_at TEXT
- resolved_by_actor_id TEXT
- execution_started_at TEXT
- execution_finished_at TEXT
```

`pending_turns` stores the large resumable state payload linked by approval ID:

```text
pending_turns
- approval_id TEXT PRIMARY KEY
- session_id TEXT NOT NULL
- request_id TEXT NOT NULL
- user_message_id TEXT NOT NULL
- resume_payload_json TEXT NOT NULL
- created_at TEXT NOT NULL
```

Relationships:

- `pending_turns.approval_id` references `pending_approvals.id`.
- Both records must reference the same `session_id`.
- Approval resume must verify that the approval belongs to the same chat/session context.

## Compatibility With `expires_at`

The existing `pending_approvals` table has `expires_at NOT NULL`. Since approvals now never expire, keep `expires_at` temporarily for backward compatibility.

Set it to a far-future value or ignore it. Do not base approval validity on `expires_at`.

After state DB migrations are implemented, remove or relax `expires_at` properly.

## State DB Migration System

Add a full state DB migration system.

Rationale:

- Existing inline `CREATE TABLE IF NOT EXISTS` is no longer enough.
- Approval resume requires schema changes that need versioned migrations.
- The memory DB already uses migrations.
- The project already depends on `refinery`.
- The state DB should get a migration module similar to the memory DB.
- `init_db` should eventually open the DB, enable foreign keys, run migrations, then return the connection.
- Avoid ad-hoc `ALTER TABLE` logic scattered through `sessions.rs`.

Recommended migration milestones:

1. Create a `src/state` or `src/sessions/migrations` module.
2. Move the current state schema into migration 001.
3. Add migration 002 for enhanced approvals.
4. Add the `pending_turns` table.
5. Add approval status and metadata indexes.
6. Keep compatibility with existing alpha DBs.
7. Add tests for fresh DB creation and migration from the old `pending_approvals` schema.

## Approval Resume Flow

The canonical resume flow is:

1. The model requests an approval-required operation.
2. The engine classifies the operation.
3. The engine creates a `pending_approvals` metadata row.
4. The engine creates a `pending_turns` resume-state row.
5. The engine inserts a visible approval-required event into chat history.
6. The engine returns an approval prompt to the originating surface/chat.
7. The user approves or denies in that same chat/session approval context.
8. The engine checks that the actor has approve permission.
9. The engine atomically changes status from `pending` to `executing`.
10. If the update affects zero rows, the engine does not execute; this prevents duplicate execution.
11. The engine loads `pending_turns.resume_payload_json`.
12. The engine executes the approved operation once.
13. The engine marks approval as `executed` or `failed`.
14. The engine appends the tool result to the stored model messages.
15. The engine calls the model again.
16. The engine stores the final assistant response in the same session.
17. The engine returns the final answer to the originating chat/surface.

Atomic execution guard:

```sql
UPDATE pending_approvals
SET status = 'executing', execution_started_at = ?
WHERE id = ? AND status = 'pending';
```

Only execute the operation if exactly one row was updated.

## Statuses

Use these approval statuses:

- `pending`
- `approved`
- `denied`
- `executing`
- `executed`
- `failed`

Optional future status:

- `cancelled`

Do not use `expired` in the new logical model because approvals never expire.

## Chat History

Pending approvals should appear in chat history as event/system messages so the user can see what happened in that conversation.

Suggested visible event:

```text
Approval required: OpenNivara wants to perform <operation>.
Preview: <summary>.
Approve once or deny.
```

If denied, store the denial event, feed a denied-tool result back into the model, and allow the agent to continue or explain.

## Approval Details UX

Show:

- operation/tool name
- classification
- target or summary
- preview
- expandable full arguments
- classifier reason, especially for shell commands
- diff for file modifications when available
- approve once
- deny

For shell commands, show:

- command
- classification
- classifier reason
- whether it is read-only, mutating, deleting, or unknown

Surface-specific approval UX:

- Desktop: modal/dialog attached to the same chat
- CLI: terminal prompt in the same turn/session
- Telegram: `/approve <id>` or `/deny <id>` in the same chat

## Implementation Milestones

1. Add state DB migrations for the existing schema.
2. Add enhanced `pending_approvals` metadata fields.
3. Add `pending_turns`.
4. Implement pending-turn serialization and deserialization.
5. Implement same-chat approval context validation.
6. Implement the atomic `pending` to `executing` transition.
7. Implement denial as a model-visible tool result.
8. Add visible approval events to chat history.
9. Add restart/resume tests.
10. Add duplicate approval tests proving one operation executes at most once.
