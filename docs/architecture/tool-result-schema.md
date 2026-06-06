# Tool Result Schema

This document defines the boundary between internal tool execution results, model-visible tool results, UI errors, previews, and audit rows.

Use this schema boundary to avoid leaking internal state into the model or reducing audit records to the compact payload that the model sees.

## Result Layers

OpenNivara should keep these layers separate:

- `ToolPreview`: read-only pre-execution summary used for approval UI and audit setup.
- `ToolExecutionResult`: internal post-execution result used by engine, UI, logs, and audit.
- `ModelVisibleToolResult`: compact provider-facing result appended as `ModelPart::ToolResult`.
- `UserFacingError`: surface-facing error DTO for Desktop, CLI, and Telegram.
- `pending_approvals`: durable approval audit/status row.
- `PendingTurnState`: operational resume payload for an unfinished model turn.

Do not send `ToolExecutionResult`, `ApprovalView`, `UserFacingError`, full preview JSON, or pending-turn JSON directly to the model.

## Internal ToolExecutionResult

Recommended shape:

```rust
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub tool_call_id: String,
    pub status: ToolExecutionStatus,
    pub result_json: Option<serde_json::Value>,
    pub result_summary: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub truncation: Option<ToolOutputTruncation>,
    pub started_at: String,
    pub finished_at: String,
}

#[serde(rename_all = "snake_case")]
pub enum ToolExecutionStatus {
    Succeeded,
    Failed,
    Denied,
}
```

Internal result objects may include timestamps, detailed truncation metadata, audit references, and richer debug fields. Convert them before appending to model history.

## Model-Visible Conversion

The model-facing envelope is defined in [Model-Visible Tool Results](model-visible-tool-results.md):

```rust
ModelPart::ToolResult {
    tool_call_id,
    name,
    result: serde_json::to_value(ModelVisibleToolResult),
}
```

Conversion rules:

- success maps to `ok = true`, `result = Some(...)`, `error = None`
- denied maps to `ok = false`, `error.code = approval_denied`
- invalid preview/input maps to `ok = false` and a stable validation/tool code
- failed execution maps to `ok = false` and a stable tool error code
- truncation must be explicit in `result` or `metadata`
- provider failure does not create a tool result

## Stable Codes

Tool result error codes should come from [Error Taxonomy](error-taxonomy.md). Add new codes there before using them in model-visible payloads or surface DTOs.

Memory tools should use the same result envelope. For example, `remember_this` can return a saved memory ID, a pending proposal ID, or `memory_disabled`; `update_memory`, `forget_memory`, and `delete_memory` use approval and result semantics from [Memory Proposals And Tools](memory-proposals-and-tools.md).

## Tests

Required tests:

1. `ToolExecutionResult` success converts to `ModelVisibleToolResult` success.
2. `ToolExecutionResult` failure converts to stable `ok=false` JSON.
3. approval denial converts to `approval_denied`.
4. invalid preview/input converts to stable validation/tool code.
5. truncation metadata survives conversion.
6. provider errors do not convert into fake tool results.
7. internal timestamps and audit-only fields are not present in model-visible JSON.
8. memory tool outcomes use the same envelope.
