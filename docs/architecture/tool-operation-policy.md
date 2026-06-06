# Tool Operation Policy

Tool tests should follow [Architecture Test Strategy](test-strategy.md). Tool errors should map into [Error Taxonomy](error-taxonomy.md) and model-visible payloads from [Model-Visible Tool Results](model-visible-tool-results.md).

This document defines the operation classification layer for OpenNivara tools. The product policy is intentionally liberal: read/open/search/send/index operations run automatically, while delete, modify, external mutation, mutating/deleting shell commands, unknown shell commands, and unknown operations require per-operation approval.

The engine should not make ad hoc approval decisions. It should call a centralized operation classification API.

## Current Tool Context

The current tool system lives mostly in `src/tools.rs`.

Current `ToolDefinition` fields:

- `name`
- `description`
- `parameters`
- `risk_level`

`ToolDefinition` does not yet have `operation_kind`.

Current executable tools:

- `get_current_dir`
- `list_dir`
- `file_exists`
- `read_file`
- `map_summary`
- `map_tree`
- `map_search`
- `map_get_node`

Current generated `tools.toml` includes future tools such as `open_app`, `open_url`, `write_file`, and `run_command`, but those tools are not executable in `ToolRegistry::execute`. Do not declare unimplemented tools to the model until their executors exist. The future opening and mutating tool catalog is documented in [Mutating And Opening Tools](mutating-tools.md).

## Source Of Truth

`OperationClassification` is the source of truth for approval.

`ToolRisk` remains useful as a UI/display/severity hint, but it must not decide approval.

Examples:

- `read_file`: `ToolRisk::Medium`, `OperationClassification::ReadOnly`, approval not required.
- `write_file`: `ToolRisk::High`, `OperationClassification::LocalModify`, approval required.

`requires_confirmation` is deprecated as the approval source of truth. Keep it temporarily for config compatibility, but do not let it override the classification policy in a contradictory way.

## Recommended Module

Start with:

```text
src/tools/operation_policy.rs
```

The first use case is classifying tool calls. If the policy grows beyond tools later, it can move to:

```text
src/operation_policy/
```

The engine should call:

```rust
classify_tool_call(&tool_call, &tool_definition) -> OperationDecision
```

Tool preview/activity is separate from classification. See [Tool Preview Schema](tool-preview-schema.md). Automatic read-only/opening tools can produce lightweight previews for transparency without requiring approval.

## Tool Definition Shape

Target `ToolDefinition`:

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub risk_level: ToolRisk,
    pub operation_kind: OperationKind,
}
```

`OperationKind` is the default operation declared by the tool:

```rust
pub enum OperationKind {
    ReadOnly,
    Opening,
    WorkspaceIndex,
    ExternalRead,
    ExternalMutation,
    LocalModify,
    LocalDelete,
    ShellCommand,
    Unknown,
}
```

All implemented tools must declare `operation_kind`. Missing or unknown declarations classify as `Unknown` and require approval.

## Operation Classification

`OperationClassification` is the final decision after inspecting the tool definition and actual arguments:

```rust
pub enum OperationClassification {
    ReadOnly,
    Opening,
    WorkspaceIndex,
    ExternalRead,
    ExternalMutation,
    LocalModify,
    LocalDelete,
    ShellReadOnly,
    ShellMutating,
    ShellDeleting,
    ShellUnknown,
    Unknown,
}
```

Use snake_case when serializing:

- `read_only`
- `opening`
- `workspace_index`
- `external_read`
- `external_mutation`
- `local_modify`
- `local_delete`
- `shell_read_only`
- `shell_mutating`
- `shell_deleting`
- `shell_unknown`
- `unknown`

`OperationDecision`:

```rust
pub struct OperationDecision {
    pub classification: OperationClassification,
    pub approval_required: bool,
    pub reason: String,
}
```

`reason` must be non-empty because it is displayed in approval UI and stored with approval metadata.

## Approval Rules

`OperationClassification::requires_approval()` returns `true` for:

- `ExternalMutation`
- `LocalModify`
- `LocalDelete`
- `ShellMutating`
- `ShellDeleting`
- `ShellUnknown`
- `Unknown`

It returns `false` for:

- `ReadOnly`
- `Opening`
- `WorkspaceIndex`
- `ExternalRead`
- `ShellReadOnly`

## Current Tool Mapping

Current implemented tools are read-only:

- `get_current_dir` -> `OperationKind::ReadOnly`
- `list_dir` -> `OperationKind::ReadOnly`
- `file_exists` -> `OperationKind::ReadOnly`
- `read_file` -> `OperationKind::ReadOnly`
- `map_summary` -> `OperationKind::ReadOnly`
- `map_tree` -> `OperationKind::ReadOnly`
- `map_search` -> `OperationKind::ReadOnly`
- `map_get_node` -> `OperationKind::ReadOnly`

`read_file` may keep `ToolRisk::Medium` for UI severity, but it is still read-only and does not require approval.

The current map tools query an existing workspace map DB. A future tool that builds or rebuilds the workspace index should use `OperationKind::WorkspaceIndex` and remain automatic.

Future implemented tool mapping:

- `open_url` -> `OperationKind::Opening`
- `open_app` -> `OperationKind::Opening`
- `open_file` -> `OperationKind::Opening`, unless the operation modifies or deletes
- `write_file` -> `OperationKind::LocalModify`
- `write_binary_file` -> `OperationKind::LocalModify`
- `delete_file` -> `OperationKind::LocalDelete`
- `run_command` -> `OperationKind::ShellCommand`

Memory tool mapping is defined in [Memory Proposals And Tools](memory-proposals-and-tools.md):

- `remember_this` -> `OperationKind::LocalModify`, governed by `MemoryMode` for proposal/save behavior
- `create_memory` -> `OperationKind::LocalModify`, governed by `MemoryMode` for proposal/save behavior
- `update_memory` -> `OperationKind::LocalModify`, approval required
- `forget_memory` -> `OperationKind::LocalModify` or `OperationKind::LocalDelete`, approval required
- `delete_memory` -> `OperationKind::LocalDelete`, approval required

Memory proposals are not tool approvals. Proposal approval saves a suggested memory; tool approval executes one same-turn operation.

## Shell Command Policy

Use a simple conservative shell classifier first. Do not attempt to fully parse Bash, zsh, PowerShell, cmd.exe, substitutions, redirects, scripts, and arbitrary compound commands in v1.

Behavior:

1. Extract the first command/program when the command is simple.
2. If the command is known read-only, classify as `ShellReadOnly`.
3. If the command is known mutating, classify as `ShellMutating`.
4. If the command is known deleting, classify as `ShellDeleting`.
5. If the command is complex or unknown, classify as `ShellUnknown`.
6. Unknown shell commands require approval.

Read-only examples:

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
- `pnpm test`
- `python --version`
- `node --version`

Mutating examples:

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
- `cargo fmt`
- `cargo fix`
- `cargo add`
- `npm install`
- `npm update`
- `npm uninstall`
- `bun add`
- `bun remove`
- `pnpm add`
- `pip install`
- `brew install`
- `apt install`

Deleting examples:

- `rm`
- `rmdir`
- `del`
- `erase`
- `Remove-Item`
- `trash`
- `git clean`

Unknown examples requiring approval:

- arbitrary scripts
- unknown binaries
- `bash -c "..."`
- `sh -c "..."`
- `powershell -Command "..."`
- commands with redirection such as `>` or `>>`
- complex pipelines or compound shell syntax
- `curl` with unknown or mutating method

Build commands are automatic even if they write build artifacts, as long as they write generated/cache/build output and do not intentionally modify source or project configuration.

Automatic examples:

- `cargo check`
- `cargo test`
- `cargo build`
- `npm test`
- `npm run build`
- `bun test`
- `bun run build`

Approval-required examples:

- `cargo fmt`
- `cargo fix`
- `npm install`
- `npm update`
- `npm uninstall`
- `bun add`
- `bun remove`
- `cargo add`
- `pip install`
- `brew install`
- `apt install`

## Opening And External API Policy

Opening a URL, app, or file is `OperationKind::Opening` and `OperationClassification::Opening`. It runs automatically unless the operation writes, modifies, or deletes.

Classify external API operations by method when applicable:

- `GET`, `HEAD`, `OPTIONS` -> `ExternalRead`, automatic.
- `POST`, `PUT`, `PATCH`, `DELETE` -> `ExternalMutation`, approval.
- Unknown method -> `Unknown`, approval.

Sending selected context, files, or data to Gemini is automatic under the product contract. Do not classify normal Gemini generation as external mutation.

## Tool Config Semantics

Use liberal defaults:

- `read_file` enabled by default.
- `read_file.requires_confirmation = false`.
- `allowed_roots = []` means unrestricted.
- `blocked_patterns = []` by default.
- Do not block `.env`, `.ssh`, secrets, tokens, or credentials by default for read-only tools.
- Remove unimplemented `open_app`, `open_url`, `write_file`, and `run_command` from generated default `tools.toml` until they exist.

Config meanings:

- `enabled` decides whether a tool can be declared/executed.
- `operation_kind` and classification decide approval.
- `risk_level` is display-only.
- `requires_confirmation` is deprecated compatibility state.
- `allowed_roots = []` means unrestricted.
- `blocked_patterns = []` means no default pattern blocking.

Path normalization should remain, but its purpose changes. It resolves exact target paths, supports clear outputs/approvals, avoids confusing relative paths, and helps future modify/delete approvals show exactly what path will be modified or deleted. It is not a default read-only safety block.

## Classification Storage

The classification string stored in `pending_approvals.classification` must come from serialized `OperationClassification`.

Approval reason should be stored/displayed with the approval request, especially for shell commands.

Approval preview data should come from `ToolPreview`. `pending_approvals` stores compact preview/audit fields, including `summary`, `operation_target`, `reason`, `arguments_preview_json`, and `classification`. `pending_turns` stores full arguments and model resume state while pending.

Reason examples:

- `Tool declares local_modify.`
- `Command 'cargo fmt' modifies source formatting.`
- `Command contains shell redirection, which is treated as unknown.`
- `HTTP method POST mutates external state.`
- `Tool declares read_only.`

## Required Tests

Add tests for:

1. Every implemented tool has `operation_kind`.
2. `get_current_dir` is read-only and does not require approval.
3. `list_dir` is read-only and does not require approval.
4. `file_exists` is read-only and does not require approval.
5. `read_file` is read-only and does not require approval.
6. `map_summary`, `map_tree`, `map_search`, and `map_get_node` are read-only and do not require approval.
7. Unknown tools classify as `unknown` and require approval.
8. `requires_approval()` is true only for external mutation, local modify, local delete, shell mutating, shell deleting, shell unknown, and unknown.
9. Shell read-only commands do not require approval.
10. Shell mutating commands require approval.
11. Shell deleting commands require approval.
12. Shell unknown commands require approval.
13. `cargo build`, `cargo test`, and `cargo check` are automatic.
14. `cargo fmt` and `cargo fix` require approval.
15. `npm install`, `bun add`, and `cargo add` require approval.
16. Open URL/app/file is automatic.
17. HTTP `GET` is automatic.
18. HTTP `POST`, `PUT`, `PATCH`, and `DELETE` require approval.
19. Gemini generation/send-to-provider is automatic.
20. Classification reason is non-empty.
21. `pending_approvals.classification` uses serialized `OperationClassification`.
22. `read_file` is enabled by default in generated `tools.toml`.
23. `read_file` does not require confirmation in generated `tools.toml`.
24. `blocked_patterns` defaults to empty.
25. `allowed_roots` empty means unrestricted.
26. `.env`, `.ssh`, and token-looking paths are not blocked by default for read-only operations.
27. Unimplemented future tools are not in generated default `tools.toml`.
28. `ToolRisk` does not decide approval.
29. Automatic read-only tools can generate preview/activity records without approval.
30. Approval-required tool calls build `ToolPreview` before creating approval.

## Implementation Milestones

PR 1 creates the classification primitives:

- Add `OperationKind`.
- Add `OperationClassification`.
- Add `OperationDecision`.
- Add `requires_approval()`.
- Add `operation_kind` to `ToolDefinition`.
- Mark all currently implemented tools as `OperationKind::ReadOnly`.
- Add classifier tests for current tools and unknown tools.

PR 2 adds shell and external classifiers:

- Add simple shell classifier.
- Add known read-only, mutating, deleting, and unknown shell tests.
- Add HTTP method classification.
- Add opening and Gemini-send classifications.

PR 3 redesigns generated tool config defaults:

- Enable `read_file` by default.
- Set `read_file.requires_confirmation = false`.
- Set `allowed_roots = []`.
- Set `blocked_patterns = []`.
- Remove unimplemented `open_app`, `open_url`, `write_file`, and `run_command` from generated default `tools.toml`.
- Update path resolver so empty `allowed_roots` means unrestricted.
- Update tests.

PR 4 integrates classification into engine tool-call flow:

- Engine calls centralized classifier.
- Automatic operations execute immediately.
- Approval-required operations create pending approval and pending turn.
- Store classification and reason in approval metadata.

PR 5 adds future tools only when executors exist:

- Add `open_url` as `Opening`.
- Add `open_app` as `Opening`.
- Add `write_file` as `LocalModify`.
- Add `run_command` as `ShellCommand`.
