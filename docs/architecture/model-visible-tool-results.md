# Model-Visible Tool Results

Model-visible tool results should be structured and stable. Do not feed raw internal errors or arbitrary `anyhow` strings back to the model.

This contract supports automatic tools, approval denial, failed tools, and recovery-safe continuation.

## Shape

Successful tool result:

```json
{
  "ok": true,
  "result": {},
  "summary": "Read Cargo.toml."
}
```

Failed tool result:

```json
{
  "ok": false,
  "error": {
    "code": "tool_invalid_args",
    "message": "write_file requires path, content, and mode."
  }
}
```

Denied tool result:

```json
{
  "ok": false,
  "error": {
    "code": "approval_denied",
    "message": "The user denied approval for this operation."
  }
}
```

## Rules

- Use stable error codes from [Error Taxonomy](error-taxonomy.md).
- Keep messages short and model-readable.
- Do not include raw stack traces.
- Do not include API keys, environment variables, secrets, or provider prompts.
- Cap or summarize large stdout/stderr and file content.
- Include operation summaries when useful.
- Keep `tool_call_id` in the surrounding `ModelPart::ToolResult`, not duplicated in every JSON payload unless needed for debugging.

## ToolExecutionResult Target

`ToolExecutionResult` should carry:

- status
- result JSON
- result summary
- output truncation metadata
- sanitized error code/message when failed

The model receives the structured `ok/result/error` payload. Surfaces can render richer detail from `ToolExecutionResult` and `ApprovalView`.

## Approval And Recovery

If a user denies approval, append a denied tool result to pending turn model history and continue the provider/model response.

If a tool fails before status reaches `executed`, append a failure result if possible, mark approval failed, and ask the provider for an explanation if possible.

If provider continuation fails after status reaches `executed`, do not append another tool result and do not rerun the tool. Retry continuation from the stored model history that already contains the successful tool result.

## Tests

Required tests:

1. tool invalid args maps to `ok = false` with `tool_invalid_args`.
2. approval denial maps to `ok = false` with `approval_denied`.
3. command timeout maps to `command_timed_out`.
4. large stdout/stderr is summarized/truncated.
5. model-visible errors do not include raw secrets.
6. executed continuation retry reuses stored successful tool result.
