# write_file V1

`write_file` is the first approval-required local mutating tool for the MVP vertical slice.

The V1 goal is narrow: deterministic preview, deterministic execution, exactly-once approval behavior, and easy tests. It is not a full editor.

## Scope

Tool:

- `write_file`

Supported V1 modes:

- `create_new`
- `overwrite`

Delayed:

- `append`
- `patch`
- `replace_range`
- `write_binary_file`
- `create_parent_dirs`
- `create_or_overwrite`

Tool schema:

```json
{
  "path": "notes.txt",
  "mode": "create_new",
  "content": "hello world"
}
```

Required fields:

- `path`: string
- `mode`: enum `create_new | overwrite`
- `content`: UTF-8 string

Do not add optional `create_parent_dirs` or `expected_existing_sha256` in the MVP unless absolutely necessary.

## Mode Semantics

`create_new`:

- create the file only if it does not already exist
- fail if the file exists
- fail if the parent directory does not exist
- fail if target is a directory
- use atomic create-new behavior where possible

Rust behavior:

```rust
OpenOptions::new().write(true).create_new(true)
```

`overwrite`:

- replace the full file contents
- file must already exist
- fail if the file does not exist
- fail if target is a directory
- do not create missing files

Reason:

- `create_new` means create file, fail if exists
- `overwrite` means replace existing file, fail if missing

## Path Resolution

Rules:

- relative paths resolve against current workspace/current process cwd
- absolute paths are allowed
- no glob expansion
- no shell expansion
- no `~` expansion in MVP unless explicitly implemented
- `allowed_roots = []` means unrestricted
- `blocked_patterns = []` by default

Preview should display:

- original input path
- resolved absolute path

Symlinks:

- allowed under the liberal local policy
- preview should disclose symlink/resolved target if easy
- if symlink disclosure is not implemented in MVP, do not claim extra protection

## UTF-8 Behavior

`write_file` is text-only.

Rules:

- content arrives as a Rust/JSON string, so it is valid UTF-8
- written bytes are `content.as_bytes()`
- line endings are preserved exactly
- no automatic newline is added
- no automatic formatting is performed
- binary writes belong in future `write_binary_file`

## Preview Rules

Preview must never mutate.

Preview caps should use tool/runtime defaults:

- `max_preview_lines = 300`
- `max_diff_bytes = 40000`
- `max_tool_result_bytes = 20000`

If preview/diff is large:

- truncate preview
- include explicit metadata such as `diff_truncated = true` and `omitted_bytes`
- approval may still be allowed, but preview must clearly say it is incomplete

## create_new Preview

If the file does not exist, return:

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

If the file already exists:

- preview fails with `file_already_exists`
- no approval is created

## overwrite Preview

If the file exists and is not a directory:

- read current file contents
- build unified diff
- include old/new byte counts
- include old/new hashes

Example:

```json
{
  "schema_version": 1,
  "tool_name": "write_file",
  "preview_kind": "text_diff",
  "operation_target": "/absolute/path/notes.txt",
  "summary": "OpenNivara wants to overwrite notes.txt.",
  "details": {
    "path": "/absolute/path/notes.txt",
    "mode": "overwrite",
    "old_bytes": 18,
    "new_bytes": 11,
    "old_sha256": "...",
    "new_sha256": "...",
    "diff": "--- notes.txt\n+++ notes.txt\n@@ ...",
    "diff_truncated": false
  }
}
```

If the file does not exist:

- preview fails with `file_missing_for_overwrite` or `path_not_found`
- no approval is created

If target is a directory:

- preview fails with `path_is_directory`
- no approval is created

## Execution

Execution must revalidate assumptions after approval. Preview is informational; execution must not trust stale preview state blindly.

`create_new` execution:

1. Resolve path.
2. Re-check parent exists.
3. Re-check target is not directory.
4. Open with `create_new`.
5. Write UTF-8 bytes.
6. Flush.
7. Return `ToolExecutionResult`.

`overwrite` execution:

1. Resolve path.
2. Re-check file exists.
3. Re-check target is not directory.
4. Write content to temp file in same directory.
5. Preserve existing file permissions if practical.
6. Rename temp file over target.
7. Return `ToolExecutionResult`.

Atomicity:

- same-directory temp file avoids cross-device rename problems
- do not silently fall back to partial in-place write if atomic replace fails
- return `atomic_replace_failed` instead

## Model-Visible Results

Success:

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

Failure:

```json
{
  "ok": false,
  "tool_name": "write_file",
  "tool_call_id": "toolcall_123",
  "summary": "write_file failed.",
  "result": null,
  "error": {
    "code": "file_already_exists",
    "message": "The file already exists, so create_new did not overwrite it.",
    "recoverable": true
  },
  "metadata": null
}
```

## Stable Error Codes

Tests should assert error codes, not full prose.

Codes:

- `tool_invalid_args`
- `path_not_found`
- `parent_directory_not_found`
- `path_is_directory`
- `file_already_exists`
- `file_missing_for_overwrite`
- `file_not_utf8_for_diff`
- `write_failed`
- `atomic_replace_failed`
- `permission_denied`

## Locked V1 Decisions

1. `write_file` is UTF-8 text only.
2. Supported modes are `create_new` and `overwrite`.
3. `create_new` fails if file exists.
4. `overwrite` fails if file is missing.
5. No append in V1.
6. No binary write in V1.
7. No parent directory creation in V1.
8. Preview is required before approval.
9. `create_new` preview shows new file preview.
10. `overwrite` preview shows unified diff.
11. Execution revalidates assumptions after approval.
12. `overwrite` uses same-directory temp file plus atomic rename where practical.
13. Preview never mutates.
14. Model-visible result uses `ok/result/error` envelope.

## Tests

Required tests:

1. `create_new` preview does not create the file.
2. `create_new` preview fails if file exists.
3. `create_new` execution creates file after approval.
4. `create_new` execution fails if parent is missing.
5. `create_new` execution fails if target is directory.
6. `overwrite` preview fails if file is missing.
7. `overwrite` preview fails if target is directory.
8. `overwrite` preview includes unified diff.
9. `overwrite` execution replaces existing file.
10. `overwrite` execution uses atomic temp-file replacement where practical.
11. Large preview/diff includes truncation metadata.
12. `write_file` success returns `ModelVisibleToolResult` with `ok = true`.
13. `write_file` failure returns `ModelVisibleToolResult` with `ok = false` and stable code.
14. Approval flow executes `write_file` exactly once.
15. Duplicate approval does not write twice.
