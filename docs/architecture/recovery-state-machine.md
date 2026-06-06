# Approval Recovery State Machine

This document defines approval recovery behavior for crashes, provider failures, duplicate approvals, and partial execution states.

The key invariant is: once an approved tool has run, never run it again during resume or retry. Retry only the model/provider continuation.

The first vertical slice that must prove this invariant is [MVP Vertical Slice](mvp-vertical-slice.md): CLI + `MockProvider` + `write_file` approval pause/resume. The completion gate for that slice is [MVP Completion Acceptance Gate](mvp-completion-acceptance-gate.md). The deterministic provider harness is defined in [MockProvider Test Harness](mock-provider-test-harness.md).

Surface actions must respect the same recovery state. Cross-surface action rendering is defined in [Surface Consistency Matrix](surface-consistency-matrix.md), [CLI Approval UX](cli-approval-ux.md), [Desktop Approval Card State Model](desktop-approval-card-state-model.md), and [Telegram Approval UX](telegram-approval-ux.md).

## Core Definitions

`executed` does not mean the whole turn is done.

- `executed`: the approved tool operation ran successfully.
- `completed`: the resumed model turn finished and the final assistant answer was stored.

Terminal approval statuses:

- `denied`
- `failed`
- `completed`

Intermediate statuses:

- `pending`
- `executing`
- `executed`

## Approval Statuses

Use:

- `pending`
- `denied`
- `executing`
- `executed`
- `failed`
- `completed`

`executed` is an intermediate recovery state. It means the tool ran and must not run again, but model/provider continuation may still need to be retried.

## Pending Turn Phases

Use:

- `awaiting_approval`
- `tool_executed_awaiting_model`
- `denied_awaiting_model`

The phase is stored both in `pending_turns.phase` and in `resume_payload_json`. The SQL column makes recovery queries possible without parsing every JSON payload.

## Schema

`pending_approvals`:

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
    operation_target TEXT,
    classification TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN (
        'pending',
        'denied',
        'executing',
        'executed',
        'failed',
        'completed'
    )),

    summary TEXT,
    reason TEXT,
    arguments_preview_json TEXT,

    result_summary TEXT,
    error_message TEXT,

    created_at TEXT NOT NULL,

    resolved_at TEXT,
    resolved_by_actor_id TEXT,

    execution_started_at TEXT,
    execution_finished_at TEXT,

    completed_at TEXT,

    resume_attempt_count INTEGER NOT NULL DEFAULT 0,
    last_resume_error TEXT,
    last_resume_attempt_at TEXT,

    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY(user_message_id) REFERENCES messages(id) ON DELETE CASCADE
);
```

Field rationale:

- `operation_target`: quick display in approval lists/cards without parsing preview JSON.
- `reason`: classifier reason, especially important for shell commands.
- `completed_at`: records when the full resumed model turn completed.
- `resume_attempt_count`: counts provider/model continuation retries.
- `last_resume_error`: stores latest provider/model continuation error.
- `last_resume_attempt_at`: records last retry time.

`pending_turns`:

```sql
CREATE TABLE pending_turns (
    approval_id TEXT PRIMARY KEY,

    session_id TEXT NOT NULL,
    request_id TEXT NOT NULL,
    user_message_id TEXT NOT NULL,

    provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,

    phase TEXT NOT NULL CHECK(phase IN (
        'awaiting_approval',
        'tool_executed_awaiting_model',
        'denied_awaiting_model'
    )),

    resume_payload_json TEXT NOT NULL,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    FOREIGN KEY(approval_id) REFERENCES pending_approvals(id) ON DELETE CASCADE,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY(user_message_id) REFERENCES messages(id) ON DELETE CASCADE
);
```

Indexes:

```sql
CREATE INDEX idx_pending_approvals_session_status
ON pending_approvals(session_id, status);

CREATE INDEX idx_pending_approvals_actor_status
ON pending_approvals(actor_id, status);

CREATE INDEX idx_pending_approvals_request_id
ON pending_approvals(request_id);

CREATE INDEX idx_pending_approvals_user_message_id
ON pending_approvals(user_message_id);

CREATE INDEX idx_pending_approvals_operation_target
ON pending_approvals(operation_target);

CREATE INDEX idx_pending_turns_session
ON pending_turns(session_id);

CREATE INDEX idx_pending_turns_request_id
ON pending_turns(request_id);

CREATE INDEX idx_pending_turns_phase
ON pending_turns(phase);
```

## State Transitions

Approval required:

```text
pending_approvals.status = pending
pending_turns.phase = awaiting_approval
```

User approves:

```text
pending -> executing
phase remains awaiting_approval
execution_started_at set
resolved_at set
resolved_by_actor_id set
```

Tool succeeds:

```text
executing -> executed
phase -> tool_executed_awaiting_model
execution_finished_at set
result_summary set
resume_payload_json updated with tool result already appended
```

This update must be atomic. Once status reaches `executed`, the tool must never run again.

Provider/model continuation succeeds:

```text
executed -> completed
completed_at set
final assistant message stored
pending_turn deleted
```

User denies:

```text
pending -> denied
phase -> denied_awaiting_model
resolved_at set
resolved_by_actor_id set
resume_payload_json updated with denied tool result
```

After the model responds to denial:

```text
pending_turn deleted
pending_approvals row remains denied
```

Tool fails:

```text
executing -> failed
execution_finished_at set
error_message set
approval_failed event stored
```

After the model is informed of failure:

```text
pending_turn deleted
pending_approvals row remains failed
```

## Crash Recovery

Crash while executing:

- status is `executing`
- unsafe to retry automatically because the operation may or may not have happened
- do not auto-retry tool execution
- after stale threshold, mark failed with `Execution was interrupted before completion could be confirmed.`

Recommended stale executing threshold:

- 10 minutes

Crash after tool success but before provider continuation:

- status is `executed`
- phase is `tool_executed_awaiting_model`
- safe to retry provider/model continuation
- never re-execute the tool

Provider fails after tool success:

- keep status `executed`
- keep phase `tool_executed_awaiting_model`
- increment `resume_attempt_count`
- set `last_resume_error`
- set `last_resume_attempt_at`
- allow retry of provider continuation only

Crash after final answer but before cleanup:

- status may be `completed`
- pending turn may still exist
- cleanup should delete pending turn

## State API

Needed functions:

```rust
begin_execution_once(...)
mark_tool_executed_and_update_turn(...)
mark_tool_failed(...)
mark_approval_completed(...)
mark_resume_failed(...)
mark_stale_executing_as_failed(...)
cleanup_completed_pending_turns(...)
```

Critical atomic function:

```rust
mark_tool_executed_and_update_turn(...)
```

It must atomically:

1. Set `pending_approvals.status = 'executed'`.
2. Set `execution_finished_at`.
3. Set `result_summary`.
4. Set `pending_turns.phase = 'tool_executed_awaiting_model'`.
5. Update `pending_turns.resume_payload_json` with tool result already appended.
6. Update `pending_turns.updated_at`.

This atomic update prevents duplicate execution after provider failure or app crash.

## Events

Minimum visible/stored approval events:

- `approval_required`
- `approval_approved`
- `approval_denied`
- `approval_executed`
- `approval_failed`

Additional recommended event:

- `approval_completed`

Optional recovery event:

- `approval_resume_failed`

`approval_completed` can be stored but rendered quietly or hidden by default.

## Retry Policy

Allowed:

- retry provider/model continuation when status is `executed` and phase is `tool_executed_awaiting_model`

Not allowed automatically:

- retry tool execution after crash while executing

If an operation failed, the model/user should create a new operation rather than silently reusing the old approval.

## Forbidden Transitions

Reject these transitions:

- `denied` to `executing`
- `executed` to `executing`
- `completed` to `executing`
- `failed` to `executing`
- `executing` to `denied`
- `completed` to `failed`
- `failed` to `completed`

Do not implement status as generic free-form updates. Use explicit transition functions.

## Required Tests

Add tests for:

1. Status check accepts `pending`, `denied`, `executing`, `executed`, `failed`, and `completed`.
2. Status check rejects invalid status.
3. Pending turn phase check accepts `awaiting_approval`, `tool_executed_awaiting_model`, and `denied_awaiting_model`.
4. Phase check rejects invalid phase.
5. `begin_execution_once` transitions `pending` to `executing` exactly once.
6. Duplicate approve cannot transition twice.
7. `mark_tool_executed_and_update_turn` atomically sets status `executed` and phase `tool_executed_awaiting_model`.
8. Executed approval does not re-execute tool on resume retry.
9. Provider failure after tool success increments `resume_attempt_count` and stores `last_resume_error`.
10. `mark_approval_completed` sets `completed_at` and status `completed`.
11. `cleanup_completed_pending_turns` deletes pending turn but keeps pending approval.
12. Stale executing approval is marked failed after threshold.
13. Denied approval phase becomes `denied_awaiting_model` before model continuation.
14. Failed approval stores `error_message`.
15. Completed approval remains as audit row.
16. MVP provider failure after `write_file` success leaves status `executed` and phase `tool_executed_awaiting_model`.
17. MVP continuation retries provider only and does not rerun `write_file`.
