# Tool Preview Schema

This document defines `ToolPreview`, the stable preview/activity contract used before tool execution and before approval creation.

OpenNivara is intentionally liberal: read-only, opening, searching, indexing, and send-to-provider operations run automatically. Those automatic operations may still produce lightweight preview/activity records for transparency. They must not require approval. Delete, modify, external mutation, mutating shell commands, deleting shell commands, unknown shell commands, and unknown operations require approval.

## ToolPreview Type

Add a first-class preview type:

```rust
pub struct ToolPreview {
    pub tool_name: String,
    pub operation_name: String,
    pub classification: OperationClassification,
    pub operation_target: Option<String>,
    pub summary: String,
    pub reason: String,
    pub preview_json: serde_json::Value,
    pub full_arguments_json: serde_json::Value,
}
```

`ToolRegistry` should eventually support both preview and execution:

```rust
preview(name, args, config, decision) -> anyhow::Result<ToolPreview>
execute(name, args, config) -> serde_json::Value
```

Preview must never mutate state.

Preview may:

- resolve or canonicalize paths
- read files for diff generation
- stat files
- compute hashes
- inspect command metadata
- validate base64
- validate URL, app, and path arguments

Preview must not:

- write files
- delete files
- run commands
- send external mutation
- install packages
- modify config, memory, profile, or state

## Common Envelope

Every `preview_json` uses a stable versioned envelope:

```json
{
  "schema_version": 1,
  "tool_name": "write_file",
  "preview_kind": "text_write",
  "operation_target": "/absolute/path/src/main.rs",
  "summary": "OpenNivara wants to overwrite src/main.rs.",
  "details": {}
}
```

Recommended `preview_kind` values:

- `read_file`
- `list_dir`
- `file_exists`
- `opening`
- `text_write`
- `binary_write`
- `file_delete`
- `shell_command`
- `external_read`
- `external_mutation`
- `unknown`

## Engine Flow

Automatic operation flow:

1. Classify tool call.
2. Build lightweight `ToolPreview` or activity record if useful.
3. Execute immediately.
4. Optionally store/show activity.
5. Push tool result into model history.

Approval-required operation flow:

1. Classify tool call.
2. Build `ToolPreview`.
3. If preview fails because arguments are invalid, return a tool error and do not create approval.
4. Create pending approval.
5. Create pending turn.
6. Insert `role = 'event'` approval-required message.
7. Return `EngineResponseKind::ApprovalRequired`.

## Read-Only Activity Previews

Read-only previews are optional activity records, not approval cards.

`read_file`:

```json
{
  "schema_version": 1,
  "tool_name": "read_file",
  "preview_kind": "read_file",
  "operation_target": "/absolute/path/Cargo.toml",
  "summary": "OpenNivara read Cargo.toml.",
  "details": {
    "path": "/absolute/path/Cargo.toml",
    "max_bytes": 20000
  }
}
```

`list_dir`:

```json
{
  "schema_version": 1,
  "tool_name": "list_dir",
  "preview_kind": "list_dir",
  "operation_target": "/absolute/path/src",
  "summary": "OpenNivara listed src.",
  "details": {
    "path": "/absolute/path/src",
    "recursive": false
  }
}
```

`file_exists`:

```json
{
  "schema_version": 1,
  "tool_name": "file_exists",
  "preview_kind": "file_exists",
  "operation_target": "/absolute/path/file.txt",
  "summary": "OpenNivara checked whether file.txt exists.",
  "details": {
    "path": "/absolute/path/file.txt"
  }
}
```

## Text Write Previews

`write_file` is UTF-8 text only. Use it for source code, Markdown, JSON, TOML, configs, scripts, and docs.

Overwrite preview:

```json
{
  "schema_version": 1,
  "tool_name": "write_file",
  "preview_kind": "text_write",
  "operation_target": "/absolute/path/src/main.rs",
  "summary": "OpenNivara wants to overwrite src/main.rs.",
  "details": {
    "path": "/absolute/path/src/main.rs",
    "mode": "overwrite",
    "exists": true,
    "old_bytes": 1200,
    "new_bytes": 1450,
    "old_lines": 80,
    "new_lines": 94,
    "bytes_delta": 250,
    "lines_delta": 14,
    "diff": {
      "format": "unified",
      "truncated": false,
      "old_label": "src/main.rs (current)",
      "new_label": "src/main.rs (proposed)",
      "text": "--- src/main.rs (current)\n+++ src/main.rs (proposed)\n@@ ..."
    }
  }
}
```

Create-new preview:

```json
{
  "schema_version": 1,
  "tool_name": "write_file",
  "preview_kind": "text_write",
  "operation_target": "/absolute/path/new_file.rs",
  "summary": "OpenNivara wants to create new_file.rs.",
  "details": {
    "path": "/absolute/path/new_file.rs",
    "mode": "create_new",
    "exists": false,
    "new_bytes": 800,
    "new_lines": 40,
    "new_file_preview": "first lines here...",
    "preview_truncated": false
  }
}
```

Append preview:

```json
{
  "schema_version": 1,
  "tool_name": "write_file",
  "preview_kind": "text_write",
  "operation_target": "/absolute/path/log.md",
  "summary": "OpenNivara wants to append to log.md.",
  "details": {
    "path": "/absolute/path/log.md",
    "mode": "append",
    "exists": true,
    "old_bytes": 3000,
    "append_bytes": 250,
    "append_lines": 8,
    "append_preview": "text being appended...",
    "preview_truncated": false
  }
}
```

## Diff Rules

Use unified diff text and structured stats.

Recommended caps:

- `max_diff_bytes = 40000`
- `max_preview_lines = 300`

For huge diffs or previews, truncate and report omission:

```json
{
  "diff": {
    "format": "unified",
    "truncated": true,
    "omitted_bytes": 50000,
    "text": "--- truncated diff ..."
  }
}
```

Full tool arguments live in pending turn state while pending. Compact preview and audit fields live in `pending_approvals`.

## Binary Write Previews

`write_binary_file` writes arbitrary bytes from base64. Do not show raw base64 by default.

```json
{
  "schema_version": 1,
  "tool_name": "write_binary_file",
  "preview_kind": "binary_write",
  "operation_target": "/absolute/path/image.png",
  "summary": "OpenNivara wants to overwrite image.png with 248122 bytes.",
  "details": {
    "path": "/absolute/path/image.png",
    "mode": "overwrite",
    "mime_type": "image/png",
    "exists": true,
    "old_bytes": 190301,
    "new_bytes": 248122,
    "old_sha256": "abc123...",
    "new_sha256": "def456...",
    "base64_preview_omitted": true
  }
}
```

Invalid base64 fails preview before approval creation. Do not create an approval for invalid base64.

## Delete File Preview

`delete_file` v1 deletes files only.

```json
{
  "schema_version": 1,
  "tool_name": "delete_file",
  "preview_kind": "file_delete",
  "operation_target": "/absolute/path/file.txt",
  "summary": "OpenNivara wants to delete file.txt.",
  "details": {
    "path": "/absolute/path/file.txt",
    "exists": true,
    "is_file": true,
    "is_dir": false,
    "bytes": 1820,
    "sha256": "abc123...",
    "will_delete": true
  }
}
```

If the target is a directory, return a tool error and do not create approval. Directory deletion is a separate future tool.

## Run Command Preview

`run_command` preview must not execute the command.

```json
{
  "schema_version": 1,
  "tool_name": "run_command",
  "preview_kind": "shell_command",
  "operation_target": "npm install",
  "summary": "OpenNivara wants to run a mutating shell command.",
  "details": {
    "command": "npm install",
    "cwd": "/absolute/project",
    "timeout_seconds": 30,
    "max_output_bytes": 20000,
    "shell_classification": "shell_mutating",
    "classifier_reason": "npm install modifies dependencies and lockfiles.",
    "will_execute_after_approval": true
  }
}
```

## Opening Preview

Opening operations are automatic, but may still produce activity previews.

```json
{
  "schema_version": 1,
  "tool_name": "open_url",
  "preview_kind": "opening",
  "operation_target": "https://example.com",
  "summary": "Open https://example.com in the default browser.",
  "details": {
    "url": "https://example.com"
  }
}
```

## External Operation Previews

External read previews are automatic activity records. External mutation previews become approval cards.

External read:

```json
{
  "schema_version": 1,
  "tool_name": "github_fetch_issue",
  "preview_kind": "external_read",
  "operation_target": "github:owner/repo#123",
  "summary": "OpenNivara will read GitHub issue #123.",
  "details": {
    "connector_id": "github",
    "capability_id": "github.issue.read",
    "account_display_name": "GitHub account",
    "required_scopes": ["issues:read"],
    "approval_required": false
  }
}
```

External mutation:

```json
{
  "schema_version": 1,
  "tool_name": "github_comment_issue",
  "preview_kind": "external_mutation",
  "operation_target": "github:owner/repo#123",
  "summary": "OpenNivara wants to comment on GitHub issue #123.",
  "details": {
    "connector_id": "github",
    "capability_id": "github.issue.comment",
    "account_display_name": "GitHub account",
    "body_preview": "Here is the proposed comment...",
    "required_scopes": ["issues:write"],
    "classification_reason": "Capability declares external_mutation."
  }
}
```

External previews must not include tokens, API keys, authorization headers, cookies, or other credential material.

## ApprovalView Mapping

`ApprovalView` is built from `ToolPreview`:

```rust
pub struct ApprovalView {
    pub approval_id: String,
    pub session_id: String,
    pub operation_name: String,
    pub classification: String,
    pub summary: String,
    pub operation_target: Option<String>,
    pub reason: String,
    pub preview_json: serde_json::Value,
    pub full_arguments_json: serde_json::Value,
}
```

Approval UI should render from `ApprovalView` without direct tool access.

## Storage

`pending_approvals` stores compact preview/audit fields:

- `summary`
- `operation_target`
- `reason`
- `arguments_preview_json`
- `classification`

`pending_turns` stores temporary full resume payload and full tool arguments while pending.

After execution, denial, or failure:

- delete `pending_turns`
- keep `pending_approvals` audit/status row

## Required Tests

Add tests for:

1. `ToolPreview` serializes.
2. `preview_json` includes `schema_version`.
3. `read_file` preview is generated but does not require approval.
4. `list_dir` preview is generated but does not require approval.
5. `write_file` overwrite preview includes unified diff.
6. `write_file` create-new preview includes new file stats.
7. `write_file` append preview includes append stats.
8. Huge diff is truncated.
9. `write_binary_file` preview omits raw base64.
10. `write_binary_file` invalid base64 fails preview and creates no approval.
11. `delete_file` preview includes file size/hash.
12. `delete_file` directory target returns error and creates no approval.
13. `run_command` preview does not execute command.
14. Shell classifier reason appears in `preview_json`.
15. Opening preview is optional activity and automatic.
16. `pending_approvals` stores `operation_target`.
17. `pending_approvals` stores compact preview.
18. `pending_turns` stores full args while pending.
19. `pending_turn` deletion preserves compact approval audit.
20. Approval UI can render from `ApprovalView` without direct tool access.
21. External read preview is automatic and includes connector/capability metadata.
22. External mutation preview includes connector/account/scopes/target/body.
23. External previews redact credential material.
