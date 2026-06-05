# Tool Preview And Mutating Tools Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a first-class `ToolPreview` contract, store compact approval previews, and implement the next opening/mutating tool catalog with safe preview-before-approval behavior.

**Architecture:** `OperationClassification` remains the approval source of truth. `ToolRegistry::preview` is read-only and produces stable `ToolPreview` / activity data before execution or approval creation. Automatic tools can produce lightweight activity previews without blocking. Approval-required tools must preview successfully before the engine creates pending approval, pending turn, and approval-required event rows.

**Tech Stack:** Rust 2021, `serde`, `serde_json`, `rusqlite`, `sha2`, `base64` if needed, existing `ToolRegistry`, state approvals, engine approval flow.

---

## File Structure

- Modify `src/tools.rs`: add `ToolPreview`, `ToolRegistry::preview`, preview helpers, opening/mutating tool definitions, and executors as they become implemented.
- Modify or create `src/tools/diff.rs` if `src/tools.rs` becomes too large: unified text diff generation and truncation.
- Modify `src/state/migrations/V2__approval_resume.sql`: add `operation_target TEXT` and `reason TEXT` to `pending_approvals`.
- Modify `src/state/types.rs`: add `operation_target`, `reason`, `preview_json`, `full_arguments_json`, and `ApprovalView` fields where appropriate.
- Modify `src/state/approvals.rs`: store compact preview fields in `pending_approvals`, keep full arguments in pending turn.
- Modify `src/engine.rs`: build preview after classification, before approval creation; activity preview for automatic tools remains non-blocking.
- Modify Desktop/CLI/Telegram response DTOs after `ApprovalView` is added.

## Task 1: Add ToolPreview Type

**Files:**

- Modify: `src/tools.rs`

- [ ] **Step 1: Add serialization tests**

Add:

```rust
#[test]
fn tool_preview_serializes_with_schema_version() {
    let preview = ToolPreview {
        tool_name: "read_file".to_string(),
        operation_name: "read_file".to_string(),
        classification: OperationClassification::ReadOnly,
        operation_target: Some("/tmp/Cargo.toml".to_string()),
        summary: "OpenNivara read Cargo.toml.".to_string(),
        reason: "Tool declares read_only.".to_string(),
        preview_json: serde_json::json!({
            "schema_version": 1,
            "tool_name": "read_file",
            "preview_kind": "read_file",
            "operation_target": "/tmp/Cargo.toml",
            "summary": "OpenNivara read Cargo.toml.",
            "details": {
                "path": "/tmp/Cargo.toml",
                "max_bytes": 20000
            }
        }),
        full_arguments_json: serde_json::json!({"path": "Cargo.toml"}),
    };

    let json = serde_json::to_value(&preview).unwrap();

    assert_eq!(json["preview_json"]["schema_version"], 1);
    assert_eq!(json["classification"], "read_only");
}
```

- [ ] **Step 2: Run test and confirm failure**

Run: `cargo test tool_preview_serializes_with_schema_version`

Expected: fail because `ToolPreview` is not defined.

- [ ] **Step 3: Add `ToolPreview`**

Add:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

- [ ] **Step 4: Run serialization test**

Run: `cargo test tool_preview_serializes_with_schema_version`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/tools.rs
git commit -m "feat(tools): add tool preview type"
```

## Task 2: Add Read-Only Preview Support

**Files:**

- Modify: `src/tools.rs`

- [ ] **Step 1: Add read-only preview tests**

Add tests:

```rust
#[test]
fn read_file_preview_is_activity_not_approval() {
    let registry = ToolRegistry::new(false);
    let config = config_with_tool_enabled("read_file", true);
    let definition = registry.definition("read_file").unwrap();
    let decision = classify_tool_call(
        "read_file",
        &serde_json::json!({"path": "Cargo.toml"}),
        Some(&definition),
    );

    let preview = registry
        .preview("read_file", &serde_json::json!({"path": "Cargo.toml"}), &config, &decision)
        .unwrap();

    assert!(!decision.approval_required);
    assert_eq!(preview.preview_json["schema_version"], 1);
    assert_eq!(preview.preview_json["preview_kind"], "read_file");
}

#[test]
fn list_dir_and_file_exists_have_activity_previews() {
    let registry = ToolRegistry::new(false);
    let config = config_with_tool_enabled("list_dir", true);

    for (name, args, kind) in [
        ("list_dir", serde_json::json!({"path": "src"}), "list_dir"),
        ("file_exists", serde_json::json!({"path": "Cargo.toml"}), "file_exists"),
    ] {
        let definition = registry.definition(name).unwrap();
        let decision = classify_tool_call(name, &args, Some(&definition));
        let preview = registry.preview(name, &args, &config, &decision).unwrap();

        assert!(!decision.approval_required);
        assert_eq!(preview.preview_json["preview_kind"], kind);
    }
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test read_file_preview_is_activity_not_approval list_dir_and_file_exists_have_activity_previews`

Expected: fail because `ToolRegistry::preview` is not implemented.

- [ ] **Step 3: Implement preview method**

Add:

```rust
impl ToolRegistry {
    pub fn preview(
        &self,
        name: &str,
        args: &serde_json::Value,
        config: &ToolsConfig,
        decision: &OperationDecision,
    ) -> anyhow::Result<ToolPreview> {
        match name {
            "read_file" => preview_read_file(args, config, decision),
            "list_dir" => preview_list_dir(args, config, decision),
            "file_exists" => preview_file_exists(args, config, decision),
            _ => preview_unknown(name, args, decision),
        }
    }
}
```

Each preview resolves the path and returns the common envelope. It must not execute the tool.

- [ ] **Step 4: Run read-only preview tests**

Run: `cargo test read_file_preview_is_activity_not_approval list_dir_and_file_exists_have_activity_previews`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/tools.rs
git commit -m "feat(tools): preview read-only tool activity"
```

## Task 3: Add Diff Preview Helpers

**Files:**

- Modify: `src/tools.rs`
- Optional create: `src/tools/diff.rs`

- [ ] **Step 1: Add diff tests**

Add tests:

```rust
#[test]
fn text_overwrite_preview_includes_unified_diff() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("main.rs");
    std::fs::write(&path, "fn main() {\n    println!(\"old\");\n}\n").unwrap();

    let preview = preview_write_file_for_test(
        &path,
        "fn main() {\n    println!(\"new\");\n}\n",
        "overwrite",
    )
    .unwrap();

    let diff = preview.preview_json["details"]["diff"]["text"].as_str().unwrap();
    assert!(diff.contains("---"));
    assert!(diff.contains("+++"));
    assert!(diff.contains("-    println!(\"old\");"));
    assert!(diff.contains("+    println!(\"new\");"));
}

#[test]
fn huge_text_diff_is_truncated() {
    let old = "old\n".repeat(30_000);
    let new = "new\n".repeat(30_000);
    let diff = unified_diff_with_caps("old", &old, "new", &new, 40_000, 300);

    assert!(diff.truncated);
    assert!(diff.omitted_bytes > 0);
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test text_overwrite_preview_includes_unified_diff huge_text_diff_is_truncated`

Expected: fail because diff helpers are not implemented.

- [ ] **Step 3: Implement diff helpers**

Implement a small unified diff generator. It can be simple line-based v1, as long as output is unified-like and includes labels, stats, truncation, and changed lines. Enforce:

```rust
const MAX_DIFF_BYTES: usize = 40_000;
const MAX_PREVIEW_LINES: usize = 300;
```

- [ ] **Step 4: Run diff tests**

Run: `cargo test text_overwrite_preview_includes_unified_diff huge_text_diff_is_truncated`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/tools.rs
git commit -m "feat(tools): add text diff preview helpers"
```

## Task 4: Add Mutating Tool Definitions And Previews

**Files:**

- Modify: `src/tools.rs`

- [ ] **Step 1: Add definition and preview tests**

Add tests:

```rust
#[test]
fn mutating_tools_have_approval_classifications() {
    let registry = ToolRegistry::new(false);
    for (name, expected) in [
        ("write_file", OperationClassification::LocalModify),
        ("write_binary_file", OperationClassification::LocalModify),
        ("delete_file", OperationClassification::LocalDelete),
    ] {
        let definition = registry.definition(name).unwrap();
        let decision = classify_tool_call(name, &serde_json::json!({}), Some(&definition));
        assert_eq!(decision.classification, expected);
        assert!(decision.approval_required);
    }
}

#[test]
fn invalid_base64_preview_fails_before_approval() {
    let result = preview_write_binary_file_args(&serde_json::json!({
        "path": "image.png",
        "base64_content": "not base64",
        "mode": "overwrite",
        "mime_type": "image/png"
    }));

    assert!(result.is_err());
}

#[test]
fn delete_file_directory_preview_fails_before_approval() {
    let dir = tempfile::tempdir().unwrap();
    let result = preview_delete_file_path(dir.path());

    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test mutating_tools_have_approval_classifications invalid_base64_preview_fails_before_approval delete_file_directory_preview_fails_before_approval`

Expected: fail because mutating definitions/previews do not exist.

- [ ] **Step 3: Add definitions**

Add `write_file`, `write_binary_file`, and `delete_file` definitions with:

- `operation_kind: OperationKind::LocalModify` for write tools
- `operation_kind: OperationKind::LocalDelete` for delete
- `risk_level: ToolRisk::High`

Do not expose executors until approval resume is in place unless the engine guarantees approval before execution.

- [ ] **Step 4: Implement previews**

Implement:

- `write_file` create-new/overwrite/append previews
- `write_binary_file` base64 validation, byte count, MIME type, SHA-256 previews
- `delete_file` file-only stat/hash preview

Previews must not write, delete, or execute.

- [ ] **Step 5: Run mutating preview tests**

Run: `cargo test mutating_tools_have_approval_classifications invalid_base64_preview_fails_before_approval delete_file_directory_preview_fails_before_approval`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/tools.rs
git commit -m "feat(tools): preview mutating file tools"
```

## Task 5: Add Opening And Command Previews

**Files:**

- Modify: `src/tools.rs`

- [ ] **Step 1: Add tests**

Add tests:

```rust
#[test]
fn opening_tools_are_automatic_with_activity_previews() {
    let registry = ToolRegistry::new(false);
    for name in ["open_url", "open_file", "open_app"] {
        let definition = registry.definition(name).unwrap();
        let decision = classify_tool_call(name, &serde_json::json!({}), Some(&definition));
        assert_eq!(decision.classification, OperationClassification::Opening);
        assert!(!decision.approval_required);
    }
}

#[test]
fn run_command_preview_does_not_execute() {
    let preview = preview_run_command_args(&serde_json::json!({
        "command": "touch should_not_exist",
        "cwd": ".",
        "timeout_seconds": 30
    }))
    .unwrap();

    assert_eq!(preview.preview_json["preview_kind"], "shell_command");
    assert_eq!(preview.preview_json["details"]["will_execute_after_approval"], true);
    assert!(!std::path::Path::new("should_not_exist").exists());
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test opening_tools_are_automatic_with_activity_previews run_command_preview_does_not_execute`

Expected: fail until opening/command previews exist.

- [ ] **Step 3: Add definitions and previews**

Add:

- `open_url`: `Opening`
- `open_file`: `Opening`
- `open_app`: `Opening`
- `run_command`: `ShellCommand`

Implement preview-only behavior for `run_command` using shell classification reason. Do not execute command in preview.

- [ ] **Step 4: Run tests**

Run: `cargo test opening_tools_are_automatic_with_activity_previews run_command_preview_does_not_execute`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/tools.rs
git commit -m "feat(tools): preview opening and shell tools"
```

## Task 6: Update State Schema And ApprovalView

**Files:**

- Modify: `src/state/migrations/V2__approval_resume.sql`
- Modify: `src/state/types.rs`
- Modify: `src/state/approvals.rs`

- [ ] **Step 1: Add schema/API tests**

Add tests asserting:

```rust
assert!(column_names(&conn, "pending_approvals").contains(&"operation_target".to_string()));
assert!(column_names(&conn, "pending_approvals").contains(&"reason".to_string()));
```

Add an `ApprovalView` construction test:

```rust
let view = approval_view_from_records(...).unwrap();
assert_eq!(view.operation_target.as_deref(), Some("src/main.rs"));
assert_eq!(view.preview_json["schema_version"], 1);
assert!(view.full_arguments_json.get("path").is_some());
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test pending_approvals_stores_operation_target approval_view`

Expected: fail until schema/types are updated.

- [ ] **Step 3: Update V2 schema**

Add to `pending_approvals`:

```sql
operation_target TEXT,
reason TEXT,
```

Keep `arguments_preview_json TEXT` as compact preview JSON.

- [ ] **Step 4: Add ApprovalView**

Add:

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

- [ ] **Step 5: Store preview fields**

Update `create_pending_approval_with_turn` to store:

- `summary`
- `operation_target`
- `reason`
- `classification`
- compact `preview_json` in `arguments_preview_json`

Keep full args in pending turn.

- [ ] **Step 6: Run schema/API tests**

Run: `cargo test pending_approvals_stores_operation_target approval_view`

Expected: pass.

- [ ] **Step 7: Commit**

```bash
git add src/state
git commit -m "feat(state): store compact approval previews"
```

## Task 7: Engine Preview Integration

**Files:**

- Modify: `src/engine.rs`

- [ ] **Step 1: Add engine preview tests**

Add tests proving:

- read-only tool preview does not block execution
- preview failure creates no approval
- approval-required tool stores preview and full args
- approval response contains `ApprovalView`

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test engine_tool_preview`

Expected: fail until engine calls preview.

- [ ] **Step 3: Integrate preview**

After classification:

```rust
let preview = registry.preview(&call.name, &call.args, config, &decision)?;
```

If `decision.approval_required` is false, execution continues immediately and preview may be recorded as activity.

If `decision.approval_required` is true, store preview through state approvals and return `EngineResponseKind::ApprovalRequired`.

- [ ] **Step 4: Run engine preview tests**

Run: `cargo test engine_tool_preview`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/engine.rs
git commit -m "feat(engine): build previews before tool execution"
```

## Task 8: Surface Approval UX Contract

**Files:**

- Modify: `desktop/src/api/opennivaraClient.ts`
- Modify: `desktop/src/generated/backendTypes.ts`
- Modify: `desktop/src/features/chat/ChatView.tsx`
- Modify: `src/main.rs`
- Modify: `src/telegram.rs`
- Modify: `desktop/src-tauri/src/lib.rs`

- [ ] **Step 1: Add contract tests**

Add tests proving:

- Desktop ask response can carry approval data.
- CLI ask/chat handles `y`, `n`, and `details`.
- Telegram `/approve` and `/deny` call engine resume paths.
- Wrong session/chat is rejected.
- Surfaces do not execute tools directly.

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test approval_command`

Expected: fail until surface commands are wired.

- [ ] **Step 3: Add backend commands**

Add shared backend functions:

- `list_pending_approvals_for_session`
- `get_pending_approval_details`
- `approve_pending_operation`
- `deny_pending_operation`

Desktop Tauri commands should call these functions and return updated `AskResponse`.

- [ ] **Step 4: Add CLI commands**

Add:

```text
opennivara approvals list --session <session_id optional>
opennivara approvals show <approval_id>
opennivara approvals approve <approval_id> --session <session_id optional>
opennivara approvals deny <approval_id> --session <session_id optional>
```

- [ ] **Step 5: Replace Telegram scaffolding**

Make `/approve <id>` and `/deny <id>` call engine resume/denial methods with `actor_id = telegram_<chat_id>` and same-session validation.

- [ ] **Step 6: Run surface tests**

Run: `cargo test approval_command`

Expected: pass.

- [ ] **Step 7: Commit**

```bash
git add src/main.rs src/telegram.rs desktop/src desktop/src-tauri/src
git commit -m "feat: add shared approval surface commands"
```

## Final Verification

- [ ] **Step 1: Run tool tests**

Run: `cargo test tools`

Expected: tool classification, preview, diff, and mutating/opening tests pass.

- [ ] **Step 2: Run state tests**

Run: `cargo test state`

Expected: preview storage and approval view tests pass.

- [ ] **Step 3: Run engine tests**

Run: `cargo test engine`

Expected: preview integration and approval pause tests pass.

- [ ] **Step 4: Run full Rust tests**

Run: `cargo test`

Expected: all Rust tests pass.

- [ ] **Step 5: Run docs checks**

Run: `bun run docs:check`

Expected: markdown and internal docs links pass.

- [ ] **Step 6: Search for unsafe preview behavior**

Run:

```bash
rg -n "preview_.*(write|delete|execute|Command::new|spawn|remove_file|write\\()" src/tools.rs src/tools
```

Expected: preview helpers do not mutate, execute commands, delete files, or write files.
