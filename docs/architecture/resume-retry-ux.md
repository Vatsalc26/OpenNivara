# Resume Retry UX

Approval resume distinguishes two user actions:

1. Approve or deny an operation that has not run yet.
2. Continue the model response after an approved operation already ran.

Core invariant: if status is `executed`, the approved tool already ran. Never run it again. The only safe action is provider/model continuation retry.

## Shared API Decisions

Add engine-level methods:

```text
approve_pending_operation(input) -> ApprovalActionResponse
deny_pending_operation(input) -> ApprovalActionResponse
resume_pending_continuation(input) -> ApprovalActionResponse
```

`ApprovalActionInput`:

- `approval_id: String`
- `session_id: String`
- `surface: Surface`
- `actor_id: String`

`ResumeContinuationInput`:

- `approval_id: String`
- `session_id: String`
- `surface: Surface`
- `actor_id: String`

`ApprovalActionResponse`:

- `approval_id: String`
- `session_id: String`
- `status: String`
- `message: String`
- `engine_response: Option<EngineResponse>`
- `approval: Option<ApprovalView>`

`approve_pending_operation` may return a final assistant answer if execution and continuation succeed. It may also return an operation-executed-but-final-response-pending message if provider continuation fails.

`deny_pending_operation` may return a final denial explanation if continuation succeeds. It may also return a denial-recorded-but-final-explanation-pending message if provider continuation fails.

`resume_pending_continuation` never runs the tool. It only retries provider/model continuation.

## ApprovalView Recovery Fields

`ApprovalView` should include:

- approval ID
- session ID
- status
- phase
- operation name
- classification
- summary
- operation target
- reason
- preview JSON
- full arguments JSON
- can approve
- can deny
- can resume continuation
- can retry tool execution
- result summary
- error message
- last resume error
- resume attempt count

`can_retry_tool_execution` should always be false for existing approvals. If a tool failed or was interrupted, the user/model should start a new operation rather than reuse the old approval.

## Status Display Rules

`pending` plus `awaiting_approval`:

- Meaning: waiting for approval.
- Actions: approve, deny, details.

`executing` plus `awaiting_approval`:

- Meaning: operation started.
- Actions: no duplicate approve. Show running/recovery state.

`executed` plus `tool_executed_awaiting_model`:

- Meaning: operation ran; final assistant response pending.
- Actions: resume continuation only.

`denied` plus `denied_awaiting_model`:

- Meaning: denial recorded; final denial explanation pending.
- Actions: resume continuation only.

`failed`:

- Meaning: operation failed or was interrupted.
- Actions: show error. No resume by default.

`completed`:

- Meaning: done.
- Actions: hide from default pending list.

## Wording Decisions

Use "continue" for provider/model continuation retry in CLI and Telegram. Telegram already uses `/resume <session_id>` for session resume.

Use "Resume final response" for Desktop.

Locked decisions:

1. Add `resume_pending_continuation` as a separate engine API.
2. Do not let approve rerun or retry executed approvals.
3. Use "continue" wording for provider/model continuation retry.
4. CLI command: `opennivara approvals continue <id>`.
5. Telegram command: `/continue <id>`.
6. Desktop button: `Resume final response`.
7. `ApprovalView` includes status, phase, and action booleans.
8. Default approval lists hide completed rows.
9. Failed/interrupted approvals do not silently retry tool execution.
10. Provider continuation retry is allowed only for executed/denied pending-turn phases.

## Tests

Required tests:

1. `/approve` pending approval approves and resumes.
2. `/approve` executed approval does not rerun tool and tells user to use `/continue`.
3. `/continue` executed approval retries provider continuation only.
4. `/continue` pending approval tells user to approve/deny first.
5. CLI approvals continue calls `resume_pending_continuation`.
6. Desktop Resume final response calls `resume_pending_continuation`.
7. `ApprovalView` action booleans match status/phase.
8. Completed approvals are hidden from default list.
9. Failed approvals do not offer retry tool execution.
10. Wrong session/chat cannot continue.
11. Continuation retry increments `resume_attempt_count` on provider failure.
12. Continuation success stores final assistant answer and cleans `pending_turn`.
