# Tool Preview And Diff Schema

The canonical preview contract is [Tool Preview Schema](tool-preview-schema.md). This companion document records the approval-specific preview and diff expectations for recovery-safe implementation planning.

Tool previews give users enough detail to approve or deny mutating operations without dumping sensitive data into chat history or logs.

The first mutating preview to implement should be the MVP `write_file` create-new preview from [MVP Vertical Slice](mvp-vertical-slice.md). It proves that preview generation is read-only before adding richer diff, binary, delete, or shell previews.

## Preview Principles

Tool previews should be compact, structured, and expandable.

Show:

- operation/tool name
- classification
- target or summary
- reason for approval
- safe preview
- expandable full arguments
- classifier reason, especially for shell commands
- result summary after execution
- error message when execution fails

Do not put huge file contents, binary blobs, raw stdout/stderr, provider prompts, provider responses, API keys, or environment variables into chat-visible preview text.

## Approval Preview Shape

`ToolPreview` should support:

- operation name
- classification
- operation target
- summary
- reason
- classifier reason
- preview JSON
- full arguments JSON
- diff preview
- content omissions
- safety notes

`preview_json` is safe for cards and lists. Full arguments are expandable detail and should still pass redaction/summarization before display or logging.

## File Modification Diff

For file modification approvals, the UI should eventually show a diff.

Use the approved `similar = "2"` dependency for text diff previews.

Diff previews should include:

- target path
- operation kind
- changed line count
- added line count
- removed line count
- truncation flag
- text diff snippet when safe

Binary changes should not inline content. Show byte counts, hashes where useful, and a clear binary-change summary.

MVP `write_file` create-new preview:

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

## Shell Command Preview

For shell commands, show:

- command
- classification
- classifier reason
- whether it is read-only, mutating, deleting, or unknown

Unknown shell commands require approval.

## Result Summary

After execution, store and show a compact result summary:

- target changed
- bytes/lines changed where relevant
- command exit status where relevant
- truncated output indicators
- sanitized error summary if failed

Do not copy raw stdout/stderr into developer logs by default.

## Tests

Required tests:

1. file edit preview includes target path and changed-line counts.
2. binary preview omits raw binary content.
3. shell command preview includes classifier reason.
4. full arguments display is redacted/summarized before rendering.
5. result summary omits huge stdout/stderr.
6. MVP `write_file create_new` preview does not mutate the filesystem.
7. MVP overwrite preview includes enough information to understand replacement.
