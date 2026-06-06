# Surface Consistency Matrix

Desktop, CLI, and Telegram are equal approval surfaces.

Surfaces may look different, but they must render the same backend truth and must not invent different state transitions.

Backend source of truth:

- `ApprovalView`
- `ApprovalStatus`
- `PendingTurnPhase`
- `can_approve`
- `can_deny`
- `can_resume_continuation`
- `can_retry_tool_execution`
- `ApprovalActionResponse`
- `UserFacingError`

All surfaces render:

- `approval.status`
- `approval.phase`
- `approval.can_approve`
- `approval.can_deny`
- `approval.can_resume_continuation`
- `approval.result_summary`
- `approval.error_message`
- `approval.last_resume_error`

## State/Action Matrix

### pending

Meaning:

- tool has not executed
- user can approve or deny

Desktop:

- inline card
- Approve once button
- Deny button
- Details disclosure

CLI:

- interactive prompt in TTY `ask`/`chat`
- `opennivara approvals approve <id>`
- `opennivara approvals deny <id>`

Telegram:

- `/approve <id>`
- `/deny <id>`
- `/approval <id>` for details

### executing

Meaning:

- approval accepted
- operation is running

Desktop:

- spinner/progress
- actions disabled

CLI:

- "Already executing" / wait message

Telegram:

- "Already executing" / try again shortly

### executed

Meaning:

- tool executed successfully
- final model response is still pending

Desktop:

- Continue response button only

CLI:

- `opennivara approvals continue <id>`

Telegram:

- `/continue <id>`

### denied With Explanation Pending

Meaning:

- user denied the operation
- final model denial explanation is still pending

Desktop:

- Continue response button

CLI:

- `opennivara approvals continue <id>`

Telegram:

- `/continue <id>`

### denied Completed

Meaning:

- user denied operation
- final denial explanation is complete

Desktop:

- no action

CLI:

- already denied/completed

Telegram:

- already denied/completed

### failed

Meaning:

- approved operation failed during execution

Desktop:

- sanitized error
- no retry operation

CLI:

- sanitized error
- no retry operation

Telegram:

- sanitized error
- no retry operation

### completed

Meaning:

- final model answer/explanation is done

Desktop:

- hidden/collapsed by default

CLI:

- hidden from approvals list by default

Telegram:

- hidden from `/approvals` by default

## Action Consistency

Approve once:

- allowed only when `can_approve = true`
- normally status is `pending`
- Desktop: Approve once button
- CLI: `opennivara approvals approve <id>`
- Telegram: `/approve <id>`

Deny:

- allowed only when `can_deny = true`
- normally status is `pending`
- Desktop: Deny button
- CLI: `opennivara approvals deny <id>`
- Telegram: `/deny <id>`

Continue response:

- allowed only when `can_resume_continuation = true`
- normally status is `executed` or denied-with-explanation-pending
- Desktop: Continue response button
- CLI: `opennivara approvals continue <id>`
- Telegram: `/continue <id>`

Retry operation:

- not available in MVP
- `can_retry_tool_execution` should be false
- Desktop: not shown
- CLI: not available
- Telegram: not available

Show details:

- Desktop: Details disclosure
- CLI: `opennivara approvals show <id>`
- Telegram: `/approval <id>`

List active approvals:

- Desktop: inline cards, later side panel/inbox
- CLI: `opennivara approvals list`
- Telegram: `/approvals`

## Invariants

1. Approve never runs an already executed operation.
2. Continue never executes a tool.
3. Deny never affects an already executed operation.
4. Failed operation is not retried under the same approval.
5. Completed approvals are hidden by default but remain auditable.
6. Same-chat/session rules apply across surfaces.
7. Full arguments are expandable, not shown by default.
8. Preview is shown before approval.
9. One approval equals one operation and one execution.
10. Backend allowed-action booleans are the source of truth.

## Shared Response Handling

Every surface handles `ApprovalActionResponse` the same way.

If `engine_response.kind = Answer`:

- show final answer

If `engine_response.kind = ApprovalRequired`:

- render the next approval request

If `approval.status = executed` and no final answer exists:

- show Continue response

If `UserFacingError`:

- show `UserFacingError.message`

## Display Consistency

Operation name:

- visible on all surfaces

Target:

- visible on all surfaces

Preview summary:

- visible on all surfaces

Full preview details:

- Desktop: expandable
- CLI: `approvals show` / `--full-args`
- Telegram: usually not shown; concise preview only

Full arguments:

- Desktop: expandable
- CLI: `show --full-args`
- Telegram: not by default

Request/turn IDs:

- Desktop: debug/details only
- CLI: `--json` or verbose later
- Telegram: not shown

Result summary:

- visible when useful

Error message:

- sanitized and visible on all surfaces

## MVP Rollout

Implementation order:

1. CLI
2. Desktop
3. Telegram

This order does not mean CLI has more privileges. All surfaces are equal by policy. CLI is simply easiest to test first.

## Locked Decisions

1. Backend `ApprovalView` is source of truth.
2. All surfaces use the same allowed action booleans.
3. All surfaces use "continue", not "resume".
4. All surfaces hide completed approvals by default.
5. All surfaces show pending approvals as approve/deny.
6. All surfaces show executed approvals as continue-only.
7. No surface offers retry-operation in MVP.
8. Telegram uses commands only in MVP.
9. Desktop uses inline cards in MVP.
10. CLI uses interactive prompt plus approvals subcommands.
11. Full arguments are not shown by default on any surface.
12. `UserFacingError.message` is what surfaces show to users.

## Tests

Required tests:

1. Pending approval shows approve/deny on all surfaces.
2. Executed approval shows continue only on all surfaces.
3. Completed approvals are hidden by default on all surfaces.
4. Failed approval shows sanitized error and no retry action.
5. Approve on executed approval does not rerun tool.
6. Continue on executed approval does not rerun tool.
7. Deny on executed approval is rejected.
8. Same-session/same-chat validation is enforced.
9. Full arguments are collapsed or hidden by default.
10. `ApprovalActionResponse` is handled consistently.
