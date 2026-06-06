# MVP Vertical Slice

The first complete working vertical slice is:

```text
CLI + MockProvider + write_file create_new/overwrite + approval pause/resume
```

This is the smallest useful proof that the architecture works end to end.

Detailed contracts:

- `write_file` semantics are defined in [write_file V1](write-file-v1.md).
- `MockProvider` and the test harness are defined in [MockProvider Test Harness](mock-provider-test-harness.md).

## Demo Flow

User asks from CLI:

```text
Create notes.txt with the text "hello world".
```

Expected flow:

1. User sends request from CLI.
2. Engine creates `request_id` and `turn_id`.
3. Raw user message is stored.
4. `MockProvider` returns a `write_file` tool call.
5. `write_file` is classified as `LocalModify`.
6. Engine builds `ToolPreview`.
7. Engine creates pending approval.
8. Engine creates pending turn with frozen model history.
9. Engine returns `EngineResponseKind::ApprovalRequired`.
10. CLI displays approval prompt/details.
11. User approves once.
12. `begin_execution_once` transitions `pending -> executing`.
13. `write_file` executes exactly once.
14. `mark_tool_executed_and_update_turn` transitions `executing -> executed`.
15. `ModelVisibleToolResult` is appended to model history.
16. `MockProvider` receives tool result and returns final assistant answer.
17. Final assistant answer is stored.
18. `mark_approval_completed` transitions `executed -> completed`.
19. Pending turn is deleted.
20. Pending approval audit row remains.

## First Tool

Use `write_file`.

Reason:

- deterministic
- easy to test
- clear preview semantics
- proves `LocalModify` approval
- proves exactly-once execution
- less OS-dependent than `run_command`

Do not use `run_command` as the first vertical slice. It has shell classification, timeouts, stdout/stderr caps, OS differences, and unknown-command behavior.

## First Surface

Use CLI first.

Reason:

- easiest to test
- no React approval card required yet
- no Telegram chat/session edge cases yet
- proves backend behavior before UI polish

Surface order:

1. CLI
2. Desktop
3. Telegram

## First Provider

Use `MockProvider` first.

Reason:

- deterministic tests
- no API key required
- no flaky network/provider behavior
- can simulate tool call, final answer, denial, duplicate approval, and provider failure

MockProvider happy-path script:

Call 1:

- assistant tool call `write_file(path = "notes.txt", mode = "create_new", content = "hello world")`

Call 2 after approval/tool result:

- assistant text: `Created notes.txt with hello world.`

## write_file MVP Schema

```json
{
  "path": "notes.txt",
  "mode": "create_new",
  "content": "hello world"
}
```

MVP write modes:

- `create_new`
- `overwrite`

Rules:

- `create_new` fails if file exists
- `create_new` fails if parent directory is missing
- `overwrite` replaces an existing file
- `overwrite` fails if file is missing
- content is UTF-8 text
- append is out of scope
- binary write is out of scope
- parent directory creation is out of scope

## MVP Preview

New file preview:

```json
{
  "schema_version": 1,
  "tool_name": "write_file",
  "preview_kind": "text_write",
  "operation_target": "/absolute/path/notes.txt",
  "summary": "OpenNivara wants to create notes.txt.",
  "details": {
    "path": "/absolute/path/notes.txt",
    "mode": "create_new",
    "exists": false,
    "new_bytes": 11,
    "new_lines": 1,
    "new_file_preview": "hello world",
    "preview_truncated": false
  }
}
```

Preview must not mutate the filesystem.

## MVP Model-Visible Results

Successful execution:

```json
{
  "ok": true,
  "tool_name": "write_file",
  "tool_call_id": "toolcall_123",
  "summary": "Created notes.txt, writing 11 bytes.",
  "result": {
    "written": true,
    "path": "/absolute/path/notes.txt",
    "mode": "create_new",
    "bytes_written": 11,
    "sha256": "..."
  },
  "error": null,
  "metadata": null
}
```

Denied:

```json
{
  "ok": false,
  "tool_name": "write_file",
  "tool_call_id": "toolcall_123",
  "summary": "The user denied approval for write_file.",
  "result": null,
  "error": {
    "code": "approval_denied",
    "message": "The user denied approval for this operation. Do not try to perform the same operation again unless the user asks.",
    "recoverable": false
  },
  "metadata": null
}
```

## Required Paths

Happy path:

1. User asks to create file.
2. Engine returns `ApprovalRequired`.
3. CLI shows approval.
4. User approves.
5. File is written once.
6. Final answer is stored.
7. Approval status becomes `completed`.
8. Pending turn is deleted.
9. Pending approval audit row remains.

Denial path:

1. User denies.
2. Status becomes `denied`.
3. Pending turn phase becomes `denied_awaiting_model`.
4. Model receives `approval_denied` tool result.
5. Final denial explanation is stored.
6. Pending turn is deleted.
7. Tool never executes.

Provider failure after execution:

1. User approves.
2. Tool executes once.
3. Provider fails before final answer.
4. Status remains `executed`.
5. Pending turn remains.
6. Phase remains `tool_executed_awaiting_model`.
7. `last_resume_error` is stored.
8. `resume_attempt_count` increments.
9. User runs continue.
10. Provider resumes from stored tool result.
11. Tool is not executed again.
12. Final answer is stored.
13. Status becomes `completed`.
14. Pending turn is deleted.

Duplicate approval:

1. First approve starts execution.
2. Second approve is rejected as `already_executing` or `already_executed`.
3. Tool does not run twice.

## Out Of Scope

- Desktop approval card
- Telegram `/approve`
- `run_command`
- `delete_file`
- `write_binary_file`
- append mode
- connectors
- GitHub
- memory tools
- `http_get`
- hard-delete memory
- OAuth/account store

## Locked MVP

The first vertical slice must prove:

- approval required
- preview generated
- pending state stored
- approve executes once
- denial returns model-visible denial
- provider failure can continue without rerunning tool
- pending turn cleanup happens only after final answer/explanation

## Tests

Required tests:

1. `write_file create_new` preview does not mutate filesystem.
2. `write_file create_new` executes after approval.
3. `write_file create_new` fails if file exists.
4. overwrite preview includes enough information to understand replacement.
5. approval-required tool does not execute before approval.
6. approve executes tool exactly once.
7. duplicate approve does not execute twice.
8. deny does not execute tool.
9. deny appends `approval_denied` `ModelVisibleToolResult`.
10. provider failure after tool success leaves status `executed`.
11. continue retries provider only, not tool execution.
12. completed approval deletes pending turn and keeps audit row.
13. CLI can approve from same session/chat context.
14. wrong session approval is rejected.
15. MVP tests use a counting tool executor and assert `tool_execution_count("write_file")`.
16. Provider failure/retry tests assert provider call count and stored tool-result shape.
