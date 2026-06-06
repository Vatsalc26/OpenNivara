# Alpha Approval Completion Acceptance Gate

This document was originally named “MVP Completion Acceptance Gate.” For implementation, treat it as the acceptance gate for the first alpha approval vertical slice, not the entire product.

The first alpha approval vertical slice is:

```text
CLI + MockProvider + write_file + approval pause/resume
```

The alpha approval slice is complete when a CLI user can ask:

```text
Create notes.txt with hello world.
```

Then:

1. `MockProvider` requests `write_file`.
2. OpenNivara pauses before mutation.
3. OpenNivara shows preview.
4. OpenNivara stores pending approval and pending turn.
5. User approves.
6. OpenNivara writes the file exactly once.
7. OpenNivara resumes the same model turn.
8. Final assistant answer is stored.
9. Approval is marked completed.
10. Pending turn is cleaned up.
11. Pending approval remains as audit history.

## Required Alpha Flows

### 1. Happy Path

Flow:

```text
ask -> approval required -> approve -> write_file executes once -> final answer -> completed
```

Required proof:

- file is not created before approval
- file is created after approval
- tool execution count is 1
- pending turn is deleted after final answer
- pending approval remains as completed audit row

### 2. Denial Path

Flow:

```text
ask -> approval required -> deny -> model receives approval_denied -> final explanation
```

Required proof:

- file is never created
- tool execution count is 0
- model receives `ok = false` `approval_denied` tool result
- final denial explanation is stored
- pending turn is deleted after explanation

### 3. Provider Failure After Tool Execution

Flow:

```text
approve -> write_file executes -> provider fails -> status remains executed -> continue -> final answer
```

Required proof:

- file is created once
- status is `executed` while final answer is pending
- `last_resume_error` is stored
- continue retries provider only
- tool execution count remains 1
- status becomes `completed` after final answer

This is the most important recovery invariant.

### 4. Duplicate Approval

Flow:

```text
approve once -> second approve attempt rejected
```

Required proof:

- second approve does not run tool again
- response says `already_executing`, `already_executed`, or `already_completed`
- tool execution count is 1

### 5. Preview Does Not Mutate

Flow:

```text
preview write_file create_new -> filesystem unchanged
```

Required proof:

- preview returns `ToolPreviewEnvelope`
- file does not exist after preview
- approval is created only after valid preview/classification path

## Backend/State Acceptance

The alpha approval slice is accepted when:

- runtime IDs exist and are used
- `request_id` and `turn_id` exist in engine responses
- `pending_approvals` stores `request_id`, `turn_id`, `user_message_id`, and `tool_call_id`
- `pending_turn` stores frozen model history
- approval statuses support `pending`, `executing`, `executed`, `denied`, `failed`, and `completed`
- pending turn phases support recovery/continuation
- duplicate execution is blocked transactionally

## Model/Provider Acceptance

The alpha approval slice is accepted when:

- `MockProvider` scripts tool call -> final answer
- `MockProvider` can fail after tool execution
- `MockProvider` records `ModelRequest` history
- `tool_call_id` is stable
- tool result is sent back as `ModelVisibleToolResult`
- Gemini is not required for alpha approval tests

## Tooling Acceptance

The alpha approval slice is accepted when:

- `write_file` supports `create_new` and `overwrite`
- `write_file` preview never mutates
- `create_new` fails if file exists
- `overwrite` fails if file is missing
- overwrite preview includes diff
- `write_file` execution revalidates assumptions after approval
- model-visible result uses `ok/result/error` envelope

## Approval Engine Acceptance

The alpha approval slice is accepted when:

- `LocalModify` operation pauses before execution
- `ApprovalView` is returned
- approve executes exactly once
- deny never executes the tool
- continue never executes the tool
- provider failure after execution does not lose pending turn
- completed cleanup deletes pending turn only after final answer/explanation

## CLI Acceptance

The alpha approval slice is accepted when:

- interactive CLI shows approval prompt
- Enter defaults to no/deny
- details shows preview/full args
- approve works
- deny works
- continue works
- non-interactive mode never auto-approves
- approvals list/show/approve/deny/continue commands work

## Required Tests

- `approval_write_file_happy_path_executes_once`
- `approval_write_file_denial_does_not_execute_tool`
- `approval_write_file_provider_failure_can_continue_without_rerun`
- `approval_write_file_duplicate_approve_does_not_rerun`
- `write_file_preview_create_new_does_not_create_file`
- `write_file_overwrite_preview_includes_diff`
- `bindings_are_current`

## Not Required For The Alpha Approval Slice

- Desktop approval card
- Telegram approval commands
- `run_command`
- `delete_file`
- `write_binary_file`
- append mode
- memory tools
- `http_get`
- connectors
- GitHub
- OAuth/account store
- real Gemini approval-flow test
- hard-delete memory

## Done Statement

The alpha approval slice is done when a CLI user can approve or deny a `write_file` operation, the engine can resume the same model turn, and tests prove the tool executes at most once even across duplicate approvals and provider failure/retry.
