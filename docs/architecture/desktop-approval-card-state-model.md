# Desktop Approval Card State Model

Desktop should not invent its own approval logic.

Cross-surface action consistency is defined in [Surface Consistency Matrix](surface-consistency-matrix.md).

Desktop renders backend `ApprovalView` and calls backend commands. Frontend state is derived from:

- `approval.status`
- `approval.phase`
- `approval.can_approve`
- `approval.can_deny`
- `approval.can_resume_continuation`
- `approval.can_retry_tool_execution`

Frontend must not guess transition rules.

## Approval Card States

### pending

Meaning:

- tool has not executed
- user must approve or deny

UI:

- title: `Approval required`
- operation
- target
- preview summary/diff
- buttons: `Approve once`, `Deny`
- secondary: `Details`

Actions:

- Approve once -> `approve_pending_operation`
- Deny -> `deny_pending_operation`
- Details -> expand preview/full arguments

### executing

Meaning:

- user approved
- operation is running

UI:

- title: `Executing approved operation`
- spinner/progress
- buttons disabled

No cancel in MVP unless cancellation exists.

### executed

Meaning:

- tool executed successfully
- final model response is not completed yet
- usually caused by provider failure/interruption after tool execution

UI:

- title: `Operation executed`
- subtitle: `Final response pending`
- show `result_summary`
- show `last_resume_error` if present
- button: `Continue response`

Do not show:

- Approve again
- Deny

### denied

Meaning:

- user denied operation

Subcases:

- denial explanation completed
- denial explanation still pending due to provider failure

If explanation pending:

- title: `Operation denied`
- subtitle: `Final explanation pending`
- button: `Continue response`

If completed:

- title: `Operation denied`
- no primary action

### failed

Meaning:

- approved operation failed during execution

UI:

- title: `Operation failed`
- show sanitized `error_message`
- show preview/target
- no retry operation button in MVP

Reason: one approval equals one operation and one execution.

### completed

Meaning:

- final model answer/explanation is done

UI:

- card collapses into history/audit row
- no action buttons
- hidden from active approvals by default

## Action Mapping

Approve once:

- command: `approve_pending_operation(approval_id, session_id)`
- enabled when `can_approve = true`

Deny:

- command: `deny_pending_operation(approval_id, session_id)`
- enabled when `can_deny = true`

Continue response:

- command: `continue_pending_approval(approval_id, session_id)`
- enabled when `can_resume_continuation = true`

Retry operation:

- not shown in MVP
- `can_retry_tool_execution` should be false

## Tauri Command Targets

```rust
#[tauri::command]
async fn approve_pending_operation(
    approval_id: String,
    session_id: String,
) -> Result<ApprovalActionResponse, UserFacingError>;

#[tauri::command]
async fn deny_pending_operation(
    approval_id: String,
    session_id: String,
) -> Result<ApprovalActionResponse, UserFacingError>;

#[tauri::command]
async fn continue_pending_approval(
    approval_id: String,
    session_id: String,
) -> Result<ApprovalActionResponse, UserFacingError>;

#[tauri::command]
async fn list_pending_approvals(
    session_id: Option<String>,
    include_completed: bool,
) -> Result<Vec<PendingApprovalSummary>, UserFacingError>;

#[tauri::command]
async fn get_approval_details(
    approval_id: String,
    session_id: Option<String>,
) -> Result<ApprovalView, UserFacingError>;
```

## Frontend Components

Components:

- `ApprovalCard`
- `ApprovalPreview`
- `ApprovalDetailsDisclosure`
- `ApprovalActions`
- `ApprovalStatusBadge`
- `PendingApprovalsPanel`

MVP components:

- `ApprovalCard`
- `ApprovalPreview`

Props:

```ts
type ApprovalCardProps = {
  approval: ApprovalView
  onApprove: (approvalId: string) => Promise<void>
  onDeny: (approvalId: string) => Promise<void>
  onContinue: (approvalId: string) => Promise<void>
}
```

Do not pass duplicated booleans separately if they already exist on `ApprovalView`.

## Preview Rendering

`ToolPreviewEnvelope.preview_kind` drives preview rendering.

MVP preview kinds:

- `text_write`
- `text_diff`
- generic fallback

Future preview kinds:

- `external_read`
- `external_mutation`
- `memory_proposal`
- `memory_update`
- `memory_forget`
- `memory_delete`

Fallback:

- show summary
- show JSON details
- do not crash

## Backend Response Handling

When user clicks Approve:

1. set local action state to approving
2. disable buttons
3. call backend
4. render returned `ApprovalActionResponse`

Rules:

- do not optimistically mark approved before backend confirms
- if `engine_response.kind = Answer`, append/display final assistant answer
- if `engine_response.kind = ApprovalRequired`, display next approval card
- if `approval.status = executed`, show Continue response if provider failed after execution
- if error, show `UserFacingError.message`

## Details Disclosure

Collapsed card shows:

- summary
- target
- short preview
- primary buttons

Expanded details shows:

- full preview details JSON
- full arguments JSON
- classification reason
- optional `request_id` / `turn_id` debug row

Default: full arguments collapsed.

## Placement

MVP:

- inline approval card in chat at the point approval is required

Later:

- pending approvals side panel/inbox

## Locked Decisions

1. Desktop renders `ApprovalView` directly.
2. Frontend derives UI from status/phase/can booleans.
3. Pending shows Approve once and Deny.
4. Executing disables actions.
5. Executed shows Continue response only.
6. Failed shows sanitized error and no retry operation.
7. Completed collapses/hides by default.
8. Retry operation is not in MVP.
9. Details disclosure shows full args and preview JSON.
10. Preview renderer supports `text_write`/`text_diff` first, with generic fallback.
11. Desktop does not implement independent approval state logic.
12. Backend remains source of truth for allowed actions.

## Tests

Required tests:

1. Pending card shows Approve once and Deny.
2. Executing card disables buttons.
3. Executed card shows Continue response only.
4. Denied pending-explanation card shows Continue response.
5. Failed card shows sanitized error and no retry button.
6. Completed card has no action buttons.
7. `text_write` preview renders.
8. `text_diff` preview renders.
9. Unknown preview kind falls back to JSON details.
10. Approve calls backend and disables buttons while pending.
11. Deny calls backend and disables buttons while pending.
12. Continue calls backend and does not rerun tool.
13. `UserFacingError.message` is shown on backend error.
14. Frontend imports generated Specta `ApprovalView` type.
