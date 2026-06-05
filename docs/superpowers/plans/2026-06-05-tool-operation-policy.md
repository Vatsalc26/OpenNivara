# Tool Operation Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a centralized operation classification layer so approval decisions come from `OperationClassification::requires_approval()` instead of `ToolRisk` or `requires_confirmation`.

**Architecture:** Start inside `src/tools/operation_policy.rs` because the first consumer is tool-call classification. Add `operation_kind` to `ToolDefinition`, classify each tool call into `OperationDecision`, keep `ToolRisk` display-only, and make engine approval flow call the classifier. Redesign generated `tools.toml` defaults to match the liberal local-agent contract.

**Tech Stack:** Rust 2021, `serde`, `serde_json`, `toml`, existing `ToolRegistry`, existing engine tests.

---

## File Structure

- Modify `src/tools.rs`: add `OperationKind`, expose `operation_policy`, add `operation_kind` to `ToolDefinition`, update implemented tool definitions, update generated `tools.toml` defaults, adjust path resolver behavior.
- Create `src/tools/operation_policy.rs` or split `src/tools.rs` into `src/tools/mod.rs` plus `src/tools/operation_policy.rs` if the repository is ready for that mechanical move.
- Modify `src/engine.rs`: replace `requires_confirmation` and `ToolRisk != Low` approval decisions with centralized classification.
- Modify tests in `src/tools.rs` and `src/engine.rs`: add classification, shell, defaults, and engine integration tests.
- Update `src/bindings.rs` only if generated bindings require the new `ToolDefinition.operation_kind` field.

## Task 1: Classification Types

**Files:**

- Modify: `src/tools.rs`

- [ ] **Step 1: Add failing tests for approval rules**

Add tests:

```rust
#[test]
fn operation_classification_requires_approval_only_for_mutating_or_unknown_work() {
    use OperationClassification::*;

    for class in [ReadOnly, Opening, WorkspaceIndex, ExternalRead, ShellReadOnly] {
        assert!(!class.requires_approval(), "{class:?} should be automatic");
    }

    for class in [
        ExternalMutation,
        LocalModify,
        LocalDelete,
        ShellMutating,
        ShellDeleting,
        ShellUnknown,
        Unknown,
    ] {
        assert!(class.requires_approval(), "{class:?} should require approval");
    }
}

#[test]
fn operation_classification_serializes_to_snake_case() {
    assert_eq!(
        serde_json::to_value(OperationClassification::LocalModify).unwrap(),
        serde_json::json!("local_modify")
    );
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test operation_classification`

Expected: fail because operation classification types do not exist.

- [ ] **Step 3: Add classification types**

Add to `src/tools.rs`:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

impl OperationClassification {
    pub fn requires_approval(&self) -> bool {
        matches!(
            self,
            Self::ExternalMutation
                | Self::LocalModify
                | Self::LocalDelete
                | Self::ShellMutating
                | Self::ShellDeleting
                | Self::ShellUnknown
                | Self::Unknown
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct OperationDecision {
    pub classification: OperationClassification,
    pub approval_required: bool,
    pub reason: String,
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test operation_classification`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/tools.rs
git commit -m "feat(tools): add operation classification types"
```

## Task 2: Add `operation_kind` To Tool Definitions

**Files:**

- Modify: `src/tools.rs`
- Modify: `src/bindings.rs` if generated/exported types require it

- [ ] **Step 1: Add tests for current tool declarations**

Add:

```rust
#[test]
fn every_implemented_tool_declares_operation_kind() {
    let registry = ToolRegistry::new(true);
    for definition in registry.definitions() {
        assert_eq!(
            definition.operation_kind,
            OperationKind::ReadOnly,
            "{} should be read-only today",
            definition.name
        );
    }
}
```

- [ ] **Step 2: Run test and confirm failure**

Run: `cargo test every_implemented_tool_declares_operation_kind`

Expected: fail because `ToolDefinition` lacks `operation_kind`.

- [ ] **Step 3: Extend `ToolDefinition`**

Change `ToolDefinition`:

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub risk_level: ToolRisk,
    pub operation_kind: OperationKind,
}
```

Add `operation_kind: OperationKind::ReadOnly` to every currently implemented definition:

- `get_current_dir`
- `list_dir`
- `file_exists`
- `read_file`
- `map_summary`
- `map_tree`
- `map_search`
- `map_get_node`

- [ ] **Step 4: Run declaration tests**

Run: `cargo test every_implemented_tool_declares_operation_kind`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/tools.rs src/bindings.rs
git commit -m "feat(tools): declare operation kind for tools"
```

## Task 3: Basic Tool Classifier

**Files:**

- Modify: `src/tools.rs`

- [ ] **Step 1: Add classifier tests**

Add:

```rust
#[test]
fn read_only_tool_does_not_require_approval_even_when_risk_is_medium() {
    let definition = ToolRegistry::new(false).definition("read_file").unwrap();
    let decision = classify_tool_call("read_file", &serde_json::json!({"path": ".env"}), Some(&definition));

    assert_eq!(decision.classification, OperationClassification::ReadOnly);
    assert!(!decision.approval_required);
    assert!(!decision.reason.is_empty());
}

#[test]
fn unknown_tool_requires_approval() {
    let decision = classify_tool_call("missing", &serde_json::json!({}), None);

    assert_eq!(decision.classification, OperationClassification::Unknown);
    assert!(decision.approval_required);
    assert!(!decision.reason.is_empty());
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test classify_tool_call`

Expected: fail because the classifier does not exist.

- [ ] **Step 3: Implement basic classifier**

Add:

```rust
pub fn classify_tool_call(
    tool_name: &str,
    args: &serde_json::Value,
    definition: Option<&ToolDefinition>,
) -> OperationDecision {
    let Some(definition) = definition else {
        return decision(OperationClassification::Unknown, "Tool is not recognized.");
    };

    let classification = match definition.operation_kind {
        OperationKind::ReadOnly => OperationClassification::ReadOnly,
        OperationKind::Opening => OperationClassification::Opening,
        OperationKind::WorkspaceIndex => OperationClassification::WorkspaceIndex,
        OperationKind::ExternalRead => OperationClassification::ExternalRead,
        OperationKind::ExternalMutation => OperationClassification::ExternalMutation,
        OperationKind::LocalModify => OperationClassification::LocalModify,
        OperationKind::LocalDelete => OperationClassification::LocalDelete,
        OperationKind::ShellCommand => classify_shell_args(args),
        OperationKind::Unknown => OperationClassification::Unknown,
    };

    decision(
        classification,
        &format!("Tool '{tool_name}' declares {:?}.", definition.operation_kind),
    )
}

fn decision(classification: OperationClassification, reason: &str) -> OperationDecision {
    OperationDecision {
        approval_required: classification.requires_approval(),
        classification,
        reason: reason.to_string(),
    }
}

fn classify_shell_args(_args: &serde_json::Value) -> OperationClassification {
    OperationClassification::ShellUnknown
}
```

- [ ] **Step 4: Run classifier tests**

Run: `cargo test classify_tool_call`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/tools.rs
git commit -m "feat(tools): classify tool calls by operation kind"
```

## Task 4: Shell Classifier

**Files:**

- Modify: `src/tools.rs`

- [ ] **Step 1: Add shell tests**

Add table-driven tests:

```rust
#[test]
fn shell_classifier_marks_known_commands() {
    for command in ["pwd", "ls", "git status", "cargo check", "cargo test", "cargo build", "bun test"] {
        assert_eq!(classify_shell_command(command).classification, OperationClassification::ShellReadOnly);
    }

    for command in ["touch file", "mkdir dir", "git add .", "cargo fmt", "cargo fix", "npm install", "bun add vite", "cargo add serde"] {
        assert_eq!(classify_shell_command(command).classification, OperationClassification::ShellMutating);
    }

    for command in ["rm file", "rmdir dir", "del file", "Remove-Item file", "git clean -fd"] {
        assert_eq!(classify_shell_command(command).classification, OperationClassification::ShellDeleting);
    }

    for command in ["unknown-binary", "bash -c \"echo hi\"", "cat a > b", "ls | sort"] {
        assert_eq!(classify_shell_command(command).classification, OperationClassification::ShellUnknown);
    }
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test shell_classifier_marks_known_commands`

Expected: fail because shell classifier is not implemented.

- [ ] **Step 3: Implement conservative shell classifier**

Add `classify_shell_command(command: &str) -> OperationDecision`. Treat redirection, pipes, `&&`, `||`, `;`, `bash -c`, `sh -c`, and `powershell -Command` as `ShellUnknown`. Match simple command prefixes and exact program/subcommand pairs for read-only, mutating, and deleting examples from [Tool Operation Policy](../../architecture/tool-operation-policy.md).

- [ ] **Step 4: Run shell tests**

Run: `cargo test shell_classifier_marks_known_commands`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/tools.rs
git commit -m "feat(tools): classify shell commands conservatively"
```

## Task 5: Liberal Tool Defaults And Path Semantics

**Files:**

- Modify: `src/tools.rs`

- [ ] **Step 1: Add defaults tests**

Add tests:

```rust
#[test]
fn generated_tools_toml_uses_liberal_read_defaults() {
    let text = default_tools_toml();
    assert!(text.contains("[tools.read_file]"));
    assert!(text.contains("enabled = true"));
    assert!(text.contains("requires_confirmation = false"));
    assert!(text.contains("allowed_roots = []"));
    assert!(text.contains("blocked_patterns = []"));
    assert!(!text.contains("[tools.open_url]"));
    assert!(!text.contains("[tools.open_app]"));
    assert!(!text.contains("[tools.write_file]"));
    assert!(!text.contains("[tools.run_command]"));
}

#[test]
fn empty_allowed_roots_means_unrestricted_for_read_only_resolution() {
    let resolved = validate_and_resolve_path("Cargo.toml", &[], &[]).unwrap();
    assert!(resolved.ends_with("Cargo.toml"));
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test generated_tools_toml_uses_liberal_read_defaults empty_allowed_roots_means_unrestricted_for_read_only_resolution`

Expected: fail because defaults and path resolver still use old safety semantics.

- [ ] **Step 3: Extract default TOML helper**

Move the generated TOML string into:

```rust
pub fn default_tools_toml() -> &'static str {
    r#"[general]
enabled = true
max_tool_rounds = 3
show_tool_activity = true

[paths]
allowed_roots = []
blocked_patterns = []

[tools.get_current_dir]
enabled = true
requires_confirmation = false

[tools.list_dir]
enabled = true
requires_confirmation = false

[tools.file_exists]
enabled = true
requires_confirmation = false

[tools.read_file]
enabled = true
requires_confirmation = false
max_bytes = 20000
"#
}
```

Make `init_tools()` write `default_tools_toml()`.

- [ ] **Step 4: Update path resolver**

Change `validate_and_resolve_path` so empty `allowed_roots` means unrestricted after path normalization and blocked pattern checks:

```rust
if allowed_roots.is_empty() {
    return Ok(cleaned_path);
}
```

Keep `blocked_patterns` checks, but generated defaults should pass an empty list.

- [ ] **Step 5: Run defaults tests**

Run: `cargo test generated_tools_toml_uses_liberal_read_defaults empty_allowed_roots_means_unrestricted_for_read_only_resolution`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/tools.rs
git commit -m "feat(tools): use liberal tool defaults"
```

## Task 6: Engine Integration

**Files:**

- Modify: `src/engine.rs`
- Modify: `src/tools.rs`

- [ ] **Step 1: Add engine tests**

Add tests proving:

```rust
// read_file has ToolRisk::Medium but does not require approval.
assert_eq!(decision.classification, OperationClassification::ReadOnly);
assert!(!decision.approval_required);

// Unknown or local modify classifications create approval instead of executing.
assert!(approval_required_outcome.approval_id.len() > 0);
```

Also add a regression test that `requires_confirmation = true` does not force approval when the classification is read-only.

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test engine_uses_operation_classification`

Expected: fail because engine still uses `requires_confirmation` and `ToolRisk`.

- [ ] **Step 3: Replace ad hoc approval decisions**

Replace logic like:

```rust
let requires_confirmation = settings.requires_confirmation;
let is_risky = definition.risk_level != ToolRisk::Low;
```

with:

```rust
let decision = crate::tools::classify_tool_call(
    &call.name,
    &call.args,
    registry.definition(&call.name).as_ref(),
);
if decision.approval_required {
    // create pending approval with decision.classification and decision.reason
}
```

- [ ] **Step 4: Store classification and reason**

When creating approval metadata, store:

```rust
serde_json::to_string(&decision.classification)?;
decision.reason.clone();
```

Use the serialized classification string for `pending_approvals.classification`.

- [ ] **Step 5: Run engine tests**

Run: `cargo test engine_uses_operation_classification`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/engine.rs src/tools.rs
git commit -m "feat(engine): gate tools with operation classification"
```

## Final Verification

- [ ] **Step 1: Run tool tests**

Run: `cargo test tools`

Expected: all tool tests pass.

- [ ] **Step 2: Run engine tests**

Run: `cargo test engine`

Expected: all engine tests pass.

- [ ] **Step 3: Run full Rust tests**

Run: `cargo test`

Expected: all Rust tests pass.

- [ ] **Step 4: Run docs checks**

Run: `bun run docs:check`

Expected: markdown and internal docs links pass.

- [ ] **Step 5: Search for obsolete approval logic**

Run: `rg -n "requires_confirmation|risk_level !=|ToolRisk::Low|desktop approval and was not executed" src`

Expected: remaining hits are config compatibility, UI display, tests intentionally covering deprecated compatibility, or code scheduled for removal in a follow-up commit.
