# Approval Resume And State DB

This document defines the state database architecture for approval-required operations, same-turn pause/resume, and the migration path from the current inline alpha schema.

It builds on the [Core Agent Contract](core-agent-contract.md), [State Rust API](state-rust-api.md), [Tool Operation Policy](tool-operation-policy.md), [Tool Preview Schema](tool-preview-schema.md), [Model Provider Gateway](model-provider-gateway.md), [Engine Approval Flow](engine-approval-flow.md), and [Approval Recovery State Machine](recovery-state-machine.md). OpenNivara remains intentionally liberal: read-only work, workspace indexing, opening URLs/apps/files, external read/search, send-to-Gemini, and clearly read-only shell commands run automatically. Deleting, modifying, external mutation, mutating shell commands, deleting shell commands, unknown shell commands, and unknown operations require explicit per-operation approval.

## Decisions

Approval pauses and resumes the same agent turn. Approval does not stop the task and ask the user to retry.

Approval never expires. It is approve-once only, per operation, and single-use. One approval equals one operation and one execution. Approval cannot be replayed, and duplicate approvals must not execute the operation twice.

`executed` means the approved tool operation ran successfully. It does not mean the whole resumed turn is done. `completed` means the provider/model continuation finished and the final assistant answer was stored. The recovery state machine is defined in [Approval Recovery State Machine](recovery-state-machine.md).

Pending approvals survive app restart and can resume later. Pending approvals appear in the same chat history. Approval is tied to the particular chat/session where it was requested. Same-chat approval is required unless a future explicit cross-surface approval design is created.

Do not allow any surface to approve any pending operation globally. The approving actor must have approve permission and must approve within the correct originating chat/session approval context.

Denial is represented as a model-visible tool result so the model can continue or explain.

## Current State DB Context

The current state DB is `opennivara_state.sqlite`. It is initialized inline in `src/sessions.rs` through `init_db()` with `CREATE TABLE IF NOT EXISTS` statements.

Current inline tables are:

- `sessions`
- `messages`
- `active_sessions`
- `pending_approvals`
- `session_pinned_contexts`
- `session_pinned_skills`

The current `pending_approvals` table is shallow:

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

The current helpers only create and update pending approvals. This design cannot support full same-turn resume because it does not store pending turn state, does not model the execution lifecycle, and has no robust state DB migration system.

## Migration System

Use embedded refinery multi-file migrations immediately for the state DB.

If embedded refinery migrations are the target, do not first add a temporary `user_version` migration layer. Approval/resume is core state architecture, so this is the right moment to establish the proper state DB migration pattern.

Target structure:

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

The Rust API for these modules is defined in [State Rust API](state-rust-api.md). Create a new state DB entry point:

```rust
open_state_db() -> anyhow::Result<rusqlite::Connection>
```

`open_state_db()` must:

1. Resolve the `opennivara_state.sqlite` path.
2. Create the parent directory.
3. Detect and reset a legacy inline alpha DB when needed.
4. Open the SQLite connection.
5. Enable `PRAGMA foreign_keys = ON`.
6. Run embedded refinery migrations.
7. Return the connection.

## Legacy Alpha DB Handling

There are no active users yet. Existing alpha sessions and messages do not need to be preserved in the new clean schema.

If an old inline `opennivara_state.sqlite` exists without refinery metadata:

1. Close any connection to it.
2. Rename it to a legacy backup path such as `opennivara_state.legacy-reset-YYYYMMDD-HHMMSS.sqlite`.
3. Create a fresh `opennivara_state.sqlite`.
4. Run embedded refinery migrations V1 and V2.

Do not hard-delete the old DB without backup.

## Naming

Use `surface` language in the clean schema.

Desktop, CLI, and Telegram are equal surfaces over the same agent engine. The old inline state DB uses `source` and `source_created`, but the V1 migration should use `surface` and `surface_created` from day one.

Do not keep `source` or `source_created` in the clean migration schema unless a narrow compatibility wrapper needs to translate old Rust call sites while the code is being refactored.

## V1 Initial State Schema

`V1__initial_state_schema.sql` creates the clean baseline state schema:

- `sessions`
- `messages`
- `active_sessions`
- `session_pinned_contexts`
- `session_pinned_skills`
- core indexes

`sessions`:

```text
- id TEXT PRIMARY KEY
- title TEXT
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL
- status TEXT NOT NULL
- surface_created TEXT NOT NULL
- actor_id_created TEXT
- active INTEGER NOT NULL DEFAULT 1
```

Keep `sessions.status` as free text for now. Do not add a SQL `CHECK` constraint yet.

`messages`:

```text
- id TEXT PRIMARY KEY
- session_id TEXT NOT NULL
- role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'tool', 'event', 'system'))
- surface TEXT NOT NULL
- actor_id TEXT
- content TEXT NOT NULL
- created_at TEXT NOT NULL
- metadata_json TEXT
- FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
```

Approval-related chat events are stored as normal messages with `role = 'event'`. Event message `content` is JSON, for example:

```json
{
  "event_type": "approval_required",
  "approval_id": "appr_123",
  "operation_name": "write_file",
  "classification": "local_modify",
  "summary": "OpenNivara wants to modify src/main.rs"
}
```

`metadata_json` remains available for auxiliary UI/debug metadata, but the event payload itself belongs in `content`.

`active_sessions`:

```text
- actor_id TEXT NOT NULL
- surface TEXT NOT NULL
- session_id TEXT NOT NULL
- updated_at TEXT NOT NULL
- PRIMARY KEY(actor_id, surface)
- FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
```

`session_pinned_contexts`:

```text
- session_id TEXT NOT NULL
- context_id TEXT NOT NULL
- pinned_at TEXT NOT NULL
- PRIMARY KEY(session_id, context_id)
- FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
```

`session_pinned_skills`:

```text
- session_id TEXT NOT NULL
- skill_id TEXT NOT NULL
- pinned_at TEXT NOT NULL
- PRIMARY KEY(session_id, skill_id)
- FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
```

V1 indexes:

```sql
CREATE INDEX idx_messages_session_id ON messages(session_id);
CREATE INDEX idx_sessions_updated_at ON sessions(updated_at);
CREATE INDEX idx_active_sessions_session_id ON active_sessions(session_id);
```

## V2 Approval Resume Schema

`V2__approval_resume.sql` creates the approval-resume schema:

- `pending_approvals`
- `pending_turns`
- approval indexes

`pending_approvals` is the small metadata, status, and audit table:

```text
- id TEXT PRIMARY KEY
- session_id TEXT NOT NULL
- request_id TEXT NOT NULL
- turn_id TEXT NOT NULL
- user_message_id TEXT NOT NULL
- tool_call_id TEXT NOT NULL
- surface TEXT NOT NULL
- actor_id TEXT NOT NULL
- operation_name TEXT NOT NULL
- classification TEXT NOT NULL
- status TEXT NOT NULL CHECK(status IN ('pending', 'denied', 'executing', 'executed', 'failed', 'completed'))
- summary TEXT
- operation_target TEXT
- reason TEXT
- arguments_preview_json TEXT
- result_summary TEXT
- error_message TEXT
- created_at TEXT NOT NULL
- resolved_at TEXT
- resolved_by_actor_id TEXT
- execution_started_at TEXT
- execution_finished_at TEXT
- completed_at TEXT
- resume_attempt_count INTEGER NOT NULL DEFAULT 0
- last_resume_error TEXT
- last_resume_attempt_at TEXT
- FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
- FOREIGN KEY(user_message_id) REFERENCES messages(id) ON DELETE CASCADE
```

Do not include `expires_at`; approvals never expire.

Keep `classification` as free text in SQL. Enforce valid classification names in Rust because classification names may evolve. The stored classification value must come from serialized `OperationClassification` as defined in [Tool Operation Policy](tool-operation-policy.md).

`operation_target` is a compact text field for approval lists and UI cards. It avoids forcing surfaces to parse preview JSON just to show the target.

`arguments_preview_json` stores compact `ToolPreview.preview_json`. Full arguments and full pending tool-call state live in `pending_turns.resume_payload_json`.

`completed_at`, `resume_attempt_count`, `last_resume_error`, and `last_resume_attempt_at` support crash/provider-failure recovery after the tool has already run.

`pending_turns` stores the operational resume payload:

```text
- approval_id TEXT PRIMARY KEY
- session_id TEXT NOT NULL
- request_id TEXT NOT NULL
- turn_id TEXT NOT NULL
- user_message_id TEXT NOT NULL
- provider_id TEXT NOT NULL
- model_id TEXT NOT NULL
- phase TEXT NOT NULL CHECK(phase IN ('awaiting_approval', 'tool_executed_awaiting_model', 'denied_awaiting_model'))
- resume_payload_json TEXT NOT NULL
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL
- FOREIGN KEY(approval_id) REFERENCES pending_approvals(id) ON DELETE CASCADE
- FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
- FOREIGN KEY(user_message_id) REFERENCES messages(id) ON DELETE CASCADE
```

Use plain JSON text for `resume_payload_json`. Do not compress or encrypt it yet.

`phase` is duplicated from the payload into SQL so recovery scans can find turn state without parsing every JSON blob.

`pending_turns` is operational resume state, not permanent audit storage. Delete the pending turn after the terminal outcome has been handled and the original agent turn has completed. Keep the smaller `pending_approvals` row as audit/status history.

Terminal statuses are:

- `denied`
- `failed`
- `completed`

Intermediate statuses are:

- `pending`
- `executing`
- `executed`

V2 indexes:

```sql
CREATE INDEX idx_pending_approvals_session_status ON pending_approvals(session_id, status);
CREATE INDEX idx_pending_approvals_actor_status ON pending_approvals(actor_id, status);
CREATE INDEX idx_pending_approvals_request_id ON pending_approvals(request_id);
CREATE INDEX idx_pending_approvals_turn_id ON pending_approvals(turn_id);
CREATE INDEX idx_pending_approvals_user_message_id ON pending_approvals(user_message_id);
CREATE INDEX idx_pending_approvals_operation_target ON pending_approvals(operation_target);
CREATE INDEX idx_pending_turns_session ON pending_turns(session_id);
CREATE INDEX idx_pending_turns_request_id ON pending_turns(request_id);
CREATE INDEX idx_pending_turns_turn_id ON pending_turns(turn_id);
CREATE INDEX idx_pending_turns_phase ON pending_turns(phase);
```

## Pending Turn State

Use Option A: store full pending turn state.

Store it only for the chat/session where the approval was requested. Approving the operation resumes that same chat turn only. It must not resume another session or act as a global approval.

Pending turn state must use OpenNivara-native model types from [Model Provider Gateway](model-provider-gateway.md). Do not store Gemini-native `Content`, `Part`, or `function_call` structs in `resume_payload_json`.

Pending turn state stores the exact assembled model history from [Prompt Context Assembly](prompt-context-assembly.md). Do not recompute context, skill selection, tool declarations, memory retrieval, workspace map brief, or conversation history on approval resume.

`PendingTurnState` should include:

- request envelope
- request ID
- turn ID
- session ID
- user message ID
- OpenNivara-native model messages so far
- declared model tools
- pending tool call
- operation classification
- operation reason
- compiled context audit ID
- selected skill IDs
- pinned context IDs
- provider ID
- model ID
- generation config
- current round
- max rounds
- any other model or provider state needed to continue the same turn

## Actor And Permission Model

Add `actor_id` as plain text now. Do not add a full `actors` table yet.

Default actor IDs:

- `desktop_owner`
- `cli_owner`
- `telegram_<chat_id>`

Hardcode approval permission for valid owner actors for now. A full actor permission config/table can come later.

Approval permission is still same-chat/session only: an actor with approve permission may approve only inside the correct originating approval context.

## Approval Resume Flow

The canonical resume flow is:

1. The model requests an approval-required operation.
2. The engine classifies the operation.
3. The engine builds a read-only `ToolPreview`.
4. If preview fails because arguments are invalid, the engine returns a tool error and creates no approval.
5. The engine creates a `pending_approvals` metadata row with compact preview fields.
6. The engine creates a `pending_turns` resume-state row with full arguments and model state.
7. The engine inserts a visible approval-required event into chat history with `role = 'event'` and JSON `content`.
8. The engine returns an approval prompt to the originating surface/chat.
9. The user approves or denies in that same chat/session approval context.
10. The engine checks that the actor has approve permission.
11. For approval, the engine atomically changes status from `pending` to `executing`.
12. If the update affects zero rows, the engine does not execute; this prevents duplicate execution.
13. The engine loads `pending_turns.resume_payload_json`.
14. The engine executes the approved operation once.
15. If the tool succeeds, the engine atomically marks approval `executed`, changes pending turn phase to `tool_executed_awaiting_model`, and updates the pending turn payload with the tool result already appended.
16. If provider/model continuation fails, the engine keeps status `executed`, increments resume failure metadata, and never re-executes the tool.
17. The engine calls the model again.
18. The engine stores the final assistant response in the same session.
19. The engine marks approval `completed` and sets `completed_at`.
20. The engine deletes the `pending_turns` row after terminal completion is handled.
21. The engine returns the final answer to the originating chat/surface.

Memory extraction is not part of approval resume. Run memory extraction only after the final assistant answer or denial explanation is stored, as defined in [Memory Proposals And Tools](memory-proposals-and-tools.md).

Memory proposal approval is not part of this table or lifecycle. Proposal review stays in the memory subsystem and memory proposal UX. Do not reuse `pending_approvals`, `pending_turns`, or operation approval commands for memory proposals.

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
- `denied`
- `executing`
- `executed`
- `failed`
- `completed`

Do not use `approved`; approval transitions directly from `pending` to `executing`. Do not use `expired` because approvals never expire. `cancelled` is a possible future status, but it is not part of V2.

`executed` is not terminal. It means the tool ran and must never run again. The provider/model continuation can be retried from pending turn state. `completed` is terminal success.

## Approval Details UX

Show:

- operation/tool name
- classification
- operation target and summary
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

## Test Coverage Requirements

Add strong coverage for the state DB and approval lifecycle:

1. Fresh DB runs embedded V1/V2 migrations.
2. Legacy inline alpha DB is backed up/reset.
3. V1 tables exist.
4. V1 uses `surface` and `surface_created`, not `source` and `source_created`.
5. `sessions` includes `actor_id_created`.
6. `messages` includes `actor_id`.
7. `messages.role` rejects invalid roles.
8. `active_sessions` uses `(actor_id, surface)` as its primary key.
9. V2 `pending_approvals` exists with expected fields.
10. `pending_approvals.status` rejects invalid statuses.
11. `classification` accepts evolving free text.
12. `pending_approvals` includes `request_id`, `turn_id`, `user_message_id`, and `tool_call_id`.
13. `pending_approvals` includes `operation_target`, `reason`, `result_summary`, and `error_message`.
14. `pending_turns` includes `request_id`, `turn_id`, `provider_id`, `model_id`, and `resume_payload_json`.
15. `pending_turns` uses plain JSON text.
16. `pending_turns` can be inserted and loaded for a pending approval.
17. Pending approval survives DB reopen.
18. Same-chat/session validation rejects wrong-session approval.
19. Actor without approve permission cannot approve.
20. Atomic `pending` to `executing` transition allows only one execution.
21. Duplicate approval attempt does not execute twice.
22. Denial can produce a model-visible denied tool result.
23. Event messages are stored with `role = 'event'`.
24. Event messages use JSON content.
25. `pending_turns` is deleted after terminal completion while `pending_approvals` remains.
26. Preview failure creates no approval.
27. `pending_approvals.arguments_preview_json` stores compact `ToolPreview.preview_json`.
28. `pending_turns` stores full arguments while pending.
29. `pending_approvals.status` accepts `completed`.
30. `pending_turns.phase` rejects invalid phases.
31. `mark_tool_executed_and_update_turn` atomically sets status `executed` and phase `tool_executed_awaiting_model`.
32. Provider failure after tool success increments `resume_attempt_count`.
33. `mark_approval_completed` sets `completed_at` and preserves the audit row.
34. Stale `executing` approvals are marked failed without retrying tool execution.

## Implementation Milestones

PR 1 establishes the state DB migration and module foundation:

- Add `src/state` module.
- Add embedded refinery migration runner.
- Add `V1__initial_state_schema.sql`.
- Add `V2__approval_resume.sql`.
- Add legacy alpha DB backup/reset.
- Add migration/schema tests.

PR 2 adds typed state APIs for sessions, messages, and active sessions:

- Add shared state types.
- Add `state::sessions` helpers.
- Add `state::messages` helpers.
- Add `state::active_sessions` helpers.
- Keep `src/sessions.rs` as a compatibility wrapper where needed.
- Add typed API tests.

PR 3 adds approval storage and lifecycle primitives:

- Add high-level `create_pending_approval_with_turn`.
- Insert approval, pending turn, and event message in one transaction.
- Add same-chat/session validation.
- Add hardcoded owner actor approval permission.
- Add atomic execution guard.
- Add recovery state helpers.
- Add approval lifecycle tests.

PR 4 integrates same-turn approval resume into the engine:

- Integrate approval-required tool handling.
- Store pending approvals and pending turn state.
- Add approval event messages to chat.
- Resume the same turn after approval.
- Feed denial as a model-visible tool result.
- Delete `pending_turns` after terminal completion.

PR 5 adds approval UX:

- Add Desktop approval dialog attached to the same chat.
- Add CLI approval prompt in the same session.
- Add Telegram `/approve <id>` and `/deny <id>` handling in the same chat.
