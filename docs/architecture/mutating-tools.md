# Mutating And Opening Tools

This document defines the next OpenNivara tool catalog and how each tool maps to operation classification, approval, preview, and execution.

Current implemented tools are read-only/local workspace tools:

- `get_current_dir`
- `list_dir`
- `file_exists`
- `read_file`
- `map_summary`
- `map_tree`
- `map_search`
- `map_get_node`

The generated `tools.toml` has mentioned future tools such as `open_app`, `open_url`, `write_file`, and `run_command`, but those tools are not currently executable. Do not expose unimplemented tools through `ToolRegistry::definitions()`.

## Policy

OpenNivara is intentionally liberal:

- read/open/search/index/send-to-Gemini operations run automatically
- delete/modify/external mutation/mutating shell/deleting shell/unknown shell/unknown operations require approval

`ToolPreview` is defined in [Tool Preview Schema](tool-preview-schema.md). Preview is required before approval-required tools create approvals, and automatic tools may produce lightweight activity previews.

## Implementation Order

1. `open_url`
2. `open_file`
3. `open_app`
4. `write_file`
5. `write_binary_file`
6. `delete_file`
7. `run_command`

Opening tools are simple and automatic. Text/binary write and delete tools exercise approval previews. `run_command` is hardest and should come after operation classification and preview APIs are in place.

## open_url

- `OperationKind`: `Opening`
- `OperationClassification`: `Opening`
- Approval required: no
- Purpose: open URL in default browser
- v1 schemes: `http`, `https`

Parameters:

```json
{
  "url": "https://example.com"
}
```

Result:

```json
{
  "opened": true,
  "url": "https://example.com"
}
```

`file://` can come later if needed.

## open_file

- `OperationKind`: `Opening`
- `OperationClassification`: `Opening`
- Approval required: no
- Purpose: open a local file in the default app

Under the liberal policy, this can open any local file path by default. Still canonicalize/resolve the path so output and activity preview show the exact target.

Parameters:

```json
{
  "path": "some/file.pdf"
}
```

Result:

```json
{
  "opened": true,
  "path": "/absolute/path/some/file.pdf"
}
```

## open_app

- `OperationKind`: `Opening`
- `OperationClassification`: `Opening`
- Approval required: no
- Purpose: open a local app by name or path

Use OS-specific open mechanisms, not shell-like command execution. Do not use `open_app` for arbitrary shell commands. Use `run_command` for command execution.

Parameters:

```json
{
  "app": "Visual Studio Code"
}
```

Result:

```json
{
  "opened": true,
  "app": "Visual Studio Code"
}
```

## write_file

- `OperationKind`: `LocalModify`
- `OperationClassification`: `LocalModify`
- Approval required: yes
- Purpose: create, overwrite, or append UTF-8 text files

Use `write_file` for source code, Markdown, JSON, TOML, configs, scripts, and docs. It is intentionally text-only so approval previews can show meaningful diffs.

Parameters:

```json
{
  "path": "src/main.rs",
  "content": "... UTF-8 text ...",
  "mode": "create_new | overwrite | append"
}
```

Modes:

- `create_new`: fail if file exists
- `overwrite`: replace entire file
- `append`: append content to end

Result:

```json
{
  "written": true,
  "path": "/absolute/path/src/main.rs",
  "mode": "overwrite",
  "bytes_written": 1234
}
```

Approval preview should show target path, mode, line/byte stats, and either unified diff, new-file preview, or append preview.

## write_binary_file

- `OperationKind`: `LocalModify`
- `OperationClassification`: `LocalModify`
- Approval required: yes
- Purpose: write arbitrary bytes using base64 content

Use this for images, PDFs, archives, generated binary files, compiled artifacts, and other non-text content. Do not overload `write_file` with binary behavior.

Parameters:

```json
{
  "path": "image.png",
  "base64_content": "...",
  "mode": "create_new | overwrite | append",
  "mime_type": "image/png"
}
```

Modes:

- `create_new`: fail if file exists
- `overwrite`: replace entire file
- `append`: append decoded bytes to end

Result:

```json
{
  "written": true,
  "path": "/absolute/path/image.png",
  "mode": "overwrite",
  "bytes_written": 248122,
  "mime_type": "image/png"
}
```

Approval preview should show mode, MIME type, decoded byte count, existing file size, SHA-256 before and after, and omit raw base64 by default.

Invalid base64 fails preview before approval creation.

## delete_file

- `OperationKind`: `LocalDelete`
- `OperationClassification`: `LocalDelete`
- Approval required: yes
- Purpose: delete one exact local file

v1 deletes files only:

- no directory deletion
- no recursive deletion
- no glob expansion

Parameters:

```json
{
  "path": "file.txt"
}
```

If the path is a directory, return an error saying `delete_file` only deletes files and directory deletion is not supported yet.

Result:

```json
{
  "deleted": true,
  "path": "/absolute/path/file.txt"
}
```

Future `delete_dir` should have a stronger preview with file count, total size, sample of files, symlink behavior, and recursive confirmation.

## run_command

- `OperationKind`: `ShellCommand`
- `OperationClassification`: shell classifier decides
- Approval required:
  - known read-only shell command: no
  - known mutating shell command: yes
  - known deleting shell command: yes
  - unknown/complex shell command: yes

Parameters:

```json
{
  "command": "cargo test",
  "cwd": ".",
  "timeout_seconds": 30
}
```

Defaults:

- `cwd` defaults to current directory
- `timeout_seconds` defaults to `30`
- stdout/stderr output is capped
- recommended output cap is 20000 bytes per stream or shared cap

Result:

```json
{
  "exit_code": 0,
  "stdout": "...",
  "stderr": "...",
  "timed_out": false
}
```

Preview must not execute the command. Preview should show command, resolved cwd, classification, classifier reason, timeout, and output cap.

Automatic examples:

- `pwd`
- `ls`
- `dir`
- `cat`
- `type`
- `head`
- `tail`
- `grep`
- `rg`
- `find`
- `git status`
- `git diff`
- `cargo check`
- `cargo test`
- `cargo build`
- `npm test`
- `npm run build`
- `bun test`
- `bun run build`

Approval-required examples:

- `rm`
- `rmdir`
- `del`
- `erase`
- `Remove-Item`
- `trash`
- `touch`
- `mkdir`
- `cp`
- `mv`
- `chmod`
- `chown`
- `git add`
- `git commit`
- `git checkout`
- `git reset`
- `git clean`
- `cargo fmt`
- `cargo fix`
- `npm install`
- `npm update`
- `npm uninstall`
- `bun add`
- `bun remove`
- `pnpm add`
- `cargo add`
- `pip install`
- `brew install`
- `apt install`

Unknown/complex commands requiring approval:

- `bash -c "..."`
- `sh -c "..."`
- `powershell -Command "..."`
- command with `>` or `>>`
- complex pipelines
- arbitrary scripts
- unknown binaries
- `curl` with mutating/unknown method

## Future Tools

Do not implement these in v1:

- `apply_patch`
- `delete_dir`
- `move_file`
- `copy_file`
- `create_dir`

Future mappings:

- `apply_patch` -> `LocalModify`
- `delete_dir` -> `LocalDelete`
- `move_file` -> `LocalModify`
- `copy_file` -> `LocalModify`
- `create_dir` -> `LocalModify`

## Required Tests

Add tests for:

1. `open_url` is opening and automatic.
2. `open_file` is opening and automatic.
3. `open_app` is opening and automatic.
4. `write_file` is local modify and requires approval.
5. `write_binary_file` is local modify and requires approval.
6. `delete_file` is local delete and requires approval.
7. `run_command` classification decides approval.
8. `write_file` create-new fails if file exists.
9. `write_file` overwrite produces diff preview.
10. `write_file` append produces append preview.
11. `write_binary_file` rejects invalid base64.
12. `write_binary_file` preview shows byte count and hashes.
13. `delete_file` refuses directories.
14. `delete_file` does not expand globs.
15. `run_command` preview does not execute command.
16. Tool preview is read-only.
17. Approval view uses `preview_json` and `full_arguments_json`.
18. `ToolRegistry` does not expose unimplemented tools.
19. Implemented tools have `operation_kind`.
20. Mutating tools are never executed before approval.
