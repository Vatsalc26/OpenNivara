# Surface Approval UX

This document defines the shared approval response and command contract for Desktop, CLI, and Telegram.

All surfaces are equal in policy. They use the same engine, state DB, approval model, and same-turn resume flow. They differ only in presentation.

Surfaces should render typed `UserFacingError` values from [Error Taxonomy](error-taxonomy.md), not raw internal error strings.

Memory proposal review is not operation approval. Memory proposal UX and commands are defined in [Memory Proposals And Tools](memory-proposals-and-tools.md). Do not route memory proposal save/reject actions through `pending_approvals` or `/approve`.

External mutation approvals must render connector/account/capability/scope details from [External Operations Policy](external-operations-policy.md). Surfaces must not show raw tokens, API keys, authorization headers, cookies, or credential material.

## Shared Response Contract

Replace answer-only engine responses with a response that supports approval pause:

```rust
pub enum EngineResponseKind {
    Answer,
    ApprovalRequired,
}

pub struct EngineResponse {
    pub request_id: String,
    pub turn_id: String,
    pub session_id: String,
    pub kind: EngineResponseKind,
    pub answer: String,
    pub approval: Option<ApprovalView>,
}
```

`ApprovalView` is the common renderable object for every surface:

```rust
pub struct ApprovalView {
    pub approval_id: String,
    pub session_id: String,
    pub request_id: String,
    pub turn_id: String,
    pub status: ApprovalStatus,
    pub phase: Option<PendingTurnPhase>,
    pub operation_name: String,
    pub classification: OperationClassification,
    pub summary: String,
    pub operation_target: Option<String>,
    pub reason: String,
    pub preview: ToolPreviewEnvelope,
    pub full_arguments_json: Option<serde_json::Value>,
    pub can_approve: bool,
    pub can_deny: bool,
    pub can_resume_continuation: bool,
    pub can_retry_tool_execution: bool,
    pub result_summary: Option<String>,
    pub error_message: Option<String>,
    pub last_resume_error: Option<String>,
    pub resume_attempt_count: u32,
}
```

`ApprovalView` is built from `ToolPreview`, pending approval metadata, and pending turn arguments. See [Tool Preview Schema](tool-preview-schema.md) and [Shared Type Contract](shared-type-contract.md).

When the engine returns `EngineResponseKind::ApprovalRequired`:

- `answer` contains a human-readable approval prompt
- `approval` contains structured details
- no tool has been executed
- `pending_approvals` row exists
- `pending_turns` row exists
- `role = 'event'` approval-required message exists in chat history

## Backend Approval Commands

Surfaces call shared engine/state APIs. They never execute approval-required tools directly.

Required backend functions:

```rust
list_pending_approvals_for_session(session_id)
get_pending_approval_details(approval_id, session_id)
approve_pending_operation(approval_id, session_id, actor_id, surface)
deny_pending_operation(approval_id, session_id, actor_id, surface)
resume_pending_continuation(input) -> ApprovalActionResponse
```

Approval is same-chat/session only. An approval can only be approved or denied from the originating session/chat context.

Default actor IDs:

- Desktop: `desktop_owner`
- CLI: `cli_owner`
- Telegram: `telegram_<chat_id>`

## Desktop UX

Desktop should use an inline approval card inside the same chat, not a global modal.

The card should show:

- operation/tool name
- classification
- target/summary
- reason
- preview
- expandable full arguments
- Approve once button
- Deny button

For file modifications, show diff inside expandable full details.

For external mutations, show connector, account display name, target/destination, method/action, body/comment/message preview, required scopes, and classification reason.

Desktop Tauri commands to add:

```rust
approve_pending_operation(approval_id: String, session_id: String) -> Result<AskResponse, String>
deny_pending_operation(approval_id: String, session_id: String) -> Result<AskResponse, String>
get_pending_approval_details(approval_id: String, session_id: String) -> Result<ApprovalView, String>
list_pending_approvals_for_session(session_id: String) -> Result<Vec<ApprovalView>, String>
```

Update Desktop `AskResponse` to include:

- `session_id`
- `kind`
- `answer`
- `approval`

## CLI UX

For `opennivara ask` and `opennivara chat` v1, use inline blocking approval.

Prompt shape:

```text
Approval required:
Operation: write_file
Classification: local_modify
Target: src/main.rs
Reason: Tool declares local_modify.

Approve once? [y/N/details]:
```

Input behavior:

- `y`: approve once and resume immediately
- `n`: deny and resume with denied tool result
- `details`: print full arguments JSON, then ask again

Add a top-level approval queue command group:

```text
opennivara approvals list --session <session_id optional>
opennivara approvals show <approval_id>
opennivara approvals approve <approval_id> --session <session_id optional>
opennivara approvals deny <approval_id> --session <session_id optional>
```

Initial implementation can start with inline approval in `ask` and `chat`, then add queue commands immediately after.

## Telegram UX

Keep:

- `/approve <id>`
- `/deny <id>`

These commands are only for operation approvals.

Approval message shape:

```text
Approval required

Operation: write_file
Classification: local_modify
Target: src/main.rs
Reason: Tool declares local_modify.

Approve:
/approve appr_123

Deny:
/deny appr_123
```

Telegram rules:

- only the same Telegram chat can approve
- wrong chat rejects
- `actor_id = telegram_<chat_id>`
- approval resumes the same session/chat

Optional later detail command:

- `/approval <id>`
- `/show_approval <id>`

Memory proposal Telegram commands are separate:

```text
/memory_proposals
/save_memory <proposal_id>
/reject_memory <proposal_id>
```

`/save_memory` approves a memory proposal, not a same-turn operation approval.

## Event Messages

Approval events are normal messages:

- `role = 'event'`
- `content = JSON`

Do not create a separate `chat_events` table yet.

`approval_required`:

```json
{
  "event_type": "approval_required",
  "approval_id": "appr_123",
  "session_id": "sess_123",
  "operation_name": "write_file",
  "classification": "local_modify",
  "operation_target": "src/main.rs",
  "summary": "OpenNivara wants to modify src/main.rs.",
  "reason": "Tool declares local_modify."
}
```

`approval_approved`:

```json
{
  "event_type": "approval_approved",
  "approval_id": "appr_123",
  "approved_by_actor_id": "desktop_owner"
}
```

`approval_denied`:

```json
{
  "event_type": "approval_denied",
  "approval_id": "appr_123",
  "denied_by_actor_id": "desktop_owner"
}
```

`approval_executed`:

```json
{
  "event_type": "approval_executed",
  "approval_id": "appr_123",
  "result_summary": "Modified src/main.rs."
}
```

`approval_failed`:

```json
{
  "event_type": "approval_failed",
  "approval_id": "appr_123",
  "error_message": "Failed to write file: permission denied."
}
```

## Required Tests

Add tests for:

1. `EngineResponse` supports `Answer` and `ApprovalRequired`.
2. `ApprovalRequired` response includes `ApprovalView`.
3. Desktop ask response can return approval data.
4. CLI ask/chat handles `ApprovalRequired` with `y`, `n`, and `details`.
5. Telegram `/approve` calls engine resume, not scaffolding.
6. Telegram `/deny` calls engine denial resume, not scaffolding.
7. Wrong Telegram chat cannot approve.
8. Wrong CLI/Desktop session cannot approve.
9. Approval events are stored with `role = 'event'`.
10. Event content is valid JSON.
11. `ApprovalView` contains preview and full arguments.
12. Surfaces never execute tools directly.
13. Approved operation resumes same turn.
14. Denied operation resumes same turn with denied tool result.
15. Pending approval remains visible in same chat history.
16. Memory proposal commands do not call operation approval handlers.
17. External mutation approval cards include connector, account, target, body/destination, scopes, and reason.
18. External mutation approval cards redact credential material.

## Continue UX Update

Approval and continuation are separate user actions:

1. Approve or deny an operation that has not run yet.
2. Continue the final model response after an approved operation already ran.

Desktop should add:

- `resume_pending_continuation(approval_id, session_id)`
- `Resume final response` button

The Desktop `Resume final response` button must call `resume_pending_continuation`, not `approve_pending_operation`.

CLI should add:

- `opennivara approvals continue <approval_id> --session <session_id optional>`

Use `continue`, not `resume`, because session resume already exists.

Telegram should add:

- `/continue <id>`

Do not use `/resume` for approval continuation because `/resume <session_id>` already exists.

Telegram behavior:

- `/approve` for a pending approval approves, executes, and continues model.
- `/approve` for an already executed approval must not rerun the tool. Reply: `This operation already executed. Use /continue <id> to resume the final response.`
- `/continue` for an executed approval retries provider/model continuation only.
- `/continue` for a pending approval replies: `This operation has not been approved yet. Use /approve <id> or /deny <id>.`
- wrong Telegram chat rejects.

`ApprovalView` should include status, phase, action booleans, result summary, error message, last resume error, and resume attempt count. `can_retry_tool_execution` should always be false for existing approvals.

Default approval lists should show pending, executed with continuation pending, denied with continuation pending, executing if not stale, and recent failed/interrupted rows. Hide completed rows by default.

Additional required tests:

1. `/continue` executed approval retries provider continuation only.
2. `/continue` pending approval tells the user to approve or deny first.
3. Desktop Resume final response calls `resume_pending_continuation`.
4. CLI approvals continue calls `resume_pending_continuation`.
5. completed approvals are hidden from default pending lists.
