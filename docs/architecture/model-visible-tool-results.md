# Model-Visible Tool Results

Model-visible tool results should use one stable envelope for every tool outcome: success, tool error, approval denial, timeout, truncation, and post-approval tool failure.

Do not feed raw internal errors or arbitrary `anyhow` strings back to the model. UI, audit, and debug details belong in `ApprovalView`, `UserFacingError`, `pending_approvals`, `ToolPreview`, `ToolExecutionResult`, logs, or pending-turn state. The layer boundary is defined in [Tool Result Schema](tool-result-schema.md).

## Envelope

Every model-visible result uses:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVisibleToolResult {
    pub ok: bool,
    pub tool_name: String,
    pub tool_call_id: String,
    pub summary: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<ModelVisibleToolError>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVisibleToolError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}
```

Recommended location:

```text
src/tools/results.rs
```

Model-visible results are created from tool execution, approval denial, or tool failure outcomes, then wrapped into `ModelPart::ToolResult`.

## Success Shape

```json
{
  "ok": true,
  "tool_name": "read_file",
  "tool_call_id": "toolcall_123",
  "summary": "Read 8,421 bytes from src/main.rs.",
  "result": {
    "path": "/absolute/path/src/main.rs",
    "content": "...",
    "bytes_read": 8421,
    "truncated": false
  },
  "error": null,
  "metadata": null
}
```

## Error Shape

```json
{
  "ok": false,
  "tool_name": "write_file",
  "tool_call_id": "toolcall_123",
  "summary": "write_file could not run.",
  "result": null,
  "error": {
    "code": "tool_invalid_args",
    "message": "write_file requires path, content, and mode.",
    "recoverable": true
  },
  "metadata": null
}
```

## Include And Exclude Rules

Include:

- `ok`
- `tool_name`
- `tool_call_id`
- `summary`
- `result` or `error`
- explicit truncation metadata where applicable

Do not include by default:

- full `ApprovalView`
- full approval preview JSON
- full pending-turn JSON
- raw provider errors
- stack traces
- secret-looking values
- full audit metadata

## Success Examples

`read_file`:

```json
{
  "ok": true,
  "tool_name": "read_file",
  "tool_call_id": "toolcall_1",
  "summary": "Read 8,421 bytes from src/main.rs.",
  "result": {
    "path": "/absolute/path/src/main.rs",
    "content": "...",
    "bytes_read": 8421,
    "truncated": false,
    "max_bytes": 20000
  },
  "error": null,
  "metadata": null
}
```

`write_file` after approval:

```json
{
  "ok": true,
  "tool_name": "write_file",
  "tool_call_id": "toolcall_2",
  "summary": "Overwrote src/main.rs, writing 1,234 bytes.",
  "result": {
    "written": true,
    "path": "/absolute/path/src/main.rs",
    "mode": "overwrite",
    "bytes_written": 1234
  },
  "error": null,
  "metadata": null
}
```

`run_command`:

```json
{
  "ok": true,
  "tool_name": "run_command",
  "tool_call_id": "toolcall_3",
  "summary": "Command completed with exit code 0: cargo test",
  "result": {
    "command": "cargo test",
    "cwd": "/absolute/project",
    "exit_code": 0,
    "stdout": "...",
    "stderr": "",
    "stdout_truncated": false,
    "stderr_truncated": false,
    "timed_out": false
  },
  "error": null,
  "metadata": null
}
```

## Failure Examples

Preview/input failure:

```json
{
  "ok": false,
  "tool_name": "write_binary_file",
  "tool_call_id": "toolcall_4",
  "summary": "write_binary_file could not be previewed.",
  "result": null,
  "error": {
    "code": "invalid_base64",
    "message": "The provided base64_content is not valid base64.",
    "recoverable": true
  },
  "metadata": null
}
```

Tool failed after approval:

```json
{
  "ok": false,
  "tool_name": "delete_file",
  "tool_call_id": "toolcall_5",
  "summary": "delete_file failed.",
  "result": null,
  "error": {
    "code": "tool_execution_failed",
    "message": "Failed to delete file: permission denied.",
    "recoverable": true
  },
  "metadata": null
}
```

Command timed out:

```json
{
  "ok": false,
  "tool_name": "run_command",
  "tool_call_id": "toolcall_6",
  "summary": "Command timed out after 30 seconds: npm test",
  "result": null,
  "error": {
    "code": "command_timed_out",
    "message": "The command timed out after 30 seconds.",
    "recoverable": true
  },
  "metadata": {
    "command": "npm test",
    "cwd": "/absolute/project",
    "timeout_seconds": 30
  }
}
```

Approval denial:

```json
{
  "ok": false,
  "tool_name": "write_file",
  "tool_call_id": "toolcall_7",
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

Approval denial is a model-visible tool result so the model can continue naturally: "I did not make that file change because you denied approval."

## Provider Failures

Provider failure is not a tool result.

If the provider fails after the tool result was already appended:

- do not create a fake tool result
- keep the stored pending turn with the already-appended tool result
- call `mark_resume_failed`
- allow resume/continue later

## Memory Tool Results

Explicit memory tools use the same envelope. `remember_this` and `create_memory` can return a pending proposal, saved memory, or `memory_disabled` error depending on `MemoryMode`. `update_memory` and `forget_memory` use the same approval, denial, success, and failure shapes as other mutating tools. `delete_memory` should not be declared until true hard-delete cleanup is implemented; until then a direct invocation should return `memory_hard_delete_not_implemented`. See [Memory Proposals And Tools](memory-proposals-and-tools.md), [Memory Retention Semantics](memory-retention-semantics.md), and [Memory Hard-Delete Cleanup Scope](memory-hard-delete-cleanup-scope.md).

## Truncation

Every large tool result must include explicit truncation metadata.

Example:

```json
{
  "stdout": "...",
  "stdout_truncated": true,
  "stdout_omitted_bytes": 120000
}
```

The model should never need to infer whether output is complete.

## ModelPart Integration

Tool result should be wrapped as:

```rust
ModelPart::ToolResult {
    tool_call_id,
    name,
    result: serde_json::to_value(ModelVisibleToolResult),
}
```

The provider adapter converts that into the provider-specific function response shape.

## Relationship To ToolExecutionResult

`ToolExecutionResult` is the internal/post-execution envelope for UI, audit, and logging.

`ModelVisibleToolResult` is the compact model-facing version.

Do not blindly send full `ToolExecutionResult` to the model if it contains timestamps, audit metadata, or internal fields.

## Locked Decisions

1. Every model-visible tool result uses `{ ok, tool_name, tool_call_id, summary, result/error, metadata }`.
2. Denied approvals are represented as `ok=false` tool results.
3. Tool execution failures are represented as `ok=false` tool results.
4. Provider failures are not represented as fake tool results.
5. Large outputs include explicit truncation metadata.
6. Model-visible result is separate from `ApprovalView`, `UserFacingError`, and audit rows.
7. Tests assert JSON shape for success, failure, denial, timeout, and truncation.

## Tests

Required tests:

1. `read_file` success result uses `ok=true` envelope.
2. `write_file` success after approval uses `ok=true` envelope.
3. tool invalid args uses `ok=false` envelope.
4. invalid base64 uses `ok=false` envelope with code `invalid_base64`.
5. approval denial uses `ok=false` envelope with code `approval_denied`.
6. command timeout uses `ok=false` envelope with code `command_timed_out`.
7. stdout/stderr truncation metadata is explicit.
8. provider failure after tool result does not append a fake tool result.
9. `ModelPart::ToolResult` contains `ModelVisibleToolResult` JSON.
10. Gemini adapter converts `ModelPart::ToolResult` to provider `function_response` correctly.
