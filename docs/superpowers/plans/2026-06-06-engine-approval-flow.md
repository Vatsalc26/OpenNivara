# Engine Approval Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the engine so all surfaces use the same tool declaration, operation classification, approval pause, and same-turn resume flow.

**Architecture:** The engine derives `Surface` and `actor_id` from `RequestSource`, stores chat state through `state` APIs, talks to models through `ModelProvider`, and gates tool calls through `OperationClassification`. Automatic tools may create non-blocking activity previews and then execute immediately. Approval-required tools create `ToolPreview`, pending approval, pending turn, and event message atomically, then return `EngineResponseKind::ApprovalRequired` with `ApprovalView`.

**Tech Stack:** Rust 2021, existing `OpenNivaraEngine`, `state` module, `tools` operation policy, `model` provider gateway, `serde_json`, `tokio`.

---

## File Structure

- Modify `src/engine.rs`: surface/actor normalization, state API usage, provider-neutral loop, classification, approval pause/resume.
- Modify `src/state/types.rs`: `PendingTurnState` with native model types if not already added.
- Modify `src/state/approvals.rs`: high-level approval API calls consumed by engine.
- Modify `src/tools.rs`: classifier and tool declaration data consumed by engine.
- Modify `src/model/*`: provider boundary consumed by engine.
- Update tests in `src/engine.rs` or split into `src/engine/tests.rs` if the file becomes too large.

## Task 1: Surface And Actor Normalization

**Files:**

- Modify: `src/engine.rs`

- [ ] **Step 1: Add normalization tests**

Add:

```rust
#[test]
fn request_source_normalizes_to_surface_and_actor() {
    assert_eq!(
        surface_actor_from_source(&RequestSource::Desktop),
        (Surface::Desktop, "desktop_owner".to_string())
    );
    assert_eq!(
        surface_actor_from_source(&RequestSource::Cli),
        (Surface::Cli, "cli_owner".to_string())
    );
    assert_eq!(
        surface_actor_from_source(&RequestSource::Telegram {
            chat_id: 42,
            username: None,
        }),
        (Surface::Telegram, "telegram_42".to_string())
    );
}
```

- [ ] **Step 2: Run test and confirm failure**

Run: `cargo test request_source_normalizes_to_surface_and_actor`

Expected: fail because helper does not exist.

- [ ] **Step 3: Implement helper**

Add:

```rust
fn surface_actor_from_source(source: &RequestSource) -> (crate::state::types::Surface, String) {
    match source {
        RequestSource::Desktop => (crate::state::types::Surface::Desktop, "desktop_owner".to_string()),
        RequestSource::Cli => (crate::state::types::Surface::Cli, "cli_owner".to_string()),
        RequestSource::Telegram { chat_id, .. } => (
            crate::state::types::Surface::Telegram,
            format!("telegram_{chat_id}"),
        ),
    }
}
```

- [ ] **Step 4: Run test**

Run: `cargo test request_source_normalizes_to_surface_and_actor`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/engine.rs
git commit -m "feat(engine): normalize request surface actor"
```

## Task 2: State API Message Storage

**Files:**

- Modify: `src/engine.rs`
- Modify: `src/state/sessions.rs`
- Modify: `src/state/messages.rs`
- Modify: `src/state/active_sessions.rs`

- [ ] **Step 1: Add storage tests**

Use a temp state DB and assert:

```rust
assert_eq!(stored_user.role, "user");
assert_eq!(stored_user.surface, "telegram");
assert_eq!(stored_user.actor_id.as_deref(), Some("telegram_42"));
assert_eq!(stored_assistant.role, "assistant");
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test engine_stores_messages_with_surface_actor`

Expected: fail until engine uses state APIs and assistant role.

- [ ] **Step 3: Replace session/message writes**

Change engine session handling to use:

- `state::db::open_state_db`
- `state::sessions::create_session`
- `state::messages::store_message`
- `state::active_sessions::set_active_session`
- `state::active_sessions::get_active_session`

Store assistant final answers with `MessageRole::Assistant`, not raw `"model"`.

- [ ] **Step 4: Run storage tests**

Run: `cargo test engine_stores_messages_with_surface_actor`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/engine.rs src/state
git commit -m "refactor(engine): store chat through state API"
```

## Task 3: Equal Tool Declaration

**Files:**

- Modify: `src/engine.rs`

- [ ] **Step 1: Add declaration tests**

Add tests proving Desktop, CLI, and Telegram see the same enabled tools when selected skill policy is the same:

```rust
let desktop = declared_tool_names_for_source(RequestSource::Desktop, &tools_config, None);
let cli = declared_tool_names_for_source(RequestSource::Cli, &tools_config, None);
let telegram = declared_tool_names_for_source(
    RequestSource::Telegram { chat_id: 1, username: None },
    &tools_config,
    None,
);
assert_eq!(desktop, cli);
assert_eq!(cli, telegram);
```

Add a selected skill allowlist test proving a skill can still narrow tools.

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test equal_surface_tool_declaration`

Expected: fail because engine filters by Telegram policy and Desktop risk/confirmation.

- [ ] **Step 3: Simplify declarations**

Declare tools only by:

- global tools enabled
- individual tool enabled
- selected skill allowlist

Remove declaration filtering by Telegram remote policy, Desktop low-risk-only policy, `requires_confirmation`, and `ToolRisk`.

- [ ] **Step 4: Run declaration tests**

Run: `cargo test equal_surface_tool_declaration`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/engine.rs
git commit -m "refactor(engine): declare tools equally across surfaces"
```

## Task 4: Operation Classification In Tool Loop

**Files:**

- Modify: `src/engine.rs`
- Modify: `src/tools.rs`

- [ ] **Step 1: Add classification tests**

Add tests proving:

- `requires_confirmation = true` does not block a read-only tool.
- `ToolRisk::Medium` does not block `read_file`.
- a read-only tool executes immediately.
- an approval-required classification does not execute immediately.

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test engine_uses_operation_classification`

Expected: fail because the engine still uses old policy checks.

- [ ] **Step 3: Replace policy error path**

Replace `tool_execution_policy_error` with a precheck that only handles:

- selected skill allowlist block
- unknown tool
- disabled tool

Then classify the actual tool call:

```rust
let definition = registry.definition(&call.name);
let decision = crate::tools::classify_tool_call(
    &call.name,
    &call.args,
    definition.as_ref(),
);
```

- [ ] **Step 4: Execute automatic operations**

If `!decision.approval_required`, execute through `ToolRegistry::execute`, append model-native `ToolResult`, increment the round, and continue.

- [ ] **Step 5: Run classification tests**

Run: `cargo test engine_uses_operation_classification`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/engine.rs
git commit -m "refactor(engine): gate tool calls by classification"
```

## Task 5: Approval Pause Response

**Files:**

- Modify: `src/engine.rs`
- Modify: `src/state/types.rs`
- Modify: `src/state/approvals.rs`

- [ ] **Step 1: Add response type tests**

Add:

```rust
assert_eq!(response.kind, EngineResponseKind::ApprovalRequired);
assert!(response.approval.is_some());
assert!(response.answer.contains("approval"));
```

Also assert the DB contains one approval, one pending turn, and one `approval_required` event.

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test approval_required_tool_pauses_turn`

Expected: fail because `EngineResponseKind`, `ApprovalView`, and approval pause are missing.

- [ ] **Step 3: Add response shape**

Add:

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EngineResponseKind {
    Answer,
    ApprovalRequired,
}

pub struct EngineResponse {
    pub session_id: String,
    pub kind: EngineResponseKind,
    pub answer: String,
    pub approval: Option<ApprovalView>,
}
```

- [ ] **Step 4: Store pending turn, preview, and event**

When `decision.approval_required` is true, build `ToolPreview`, then call `state::approvals::create_pending_approval_with_turn` with native `PendingTurnState` containing messages so far, tools declaration, pending tool call, classification, reason, provider/model IDs, generation config, `current_round`, `max_rounds`, and full tool arguments. Return `ApprovalView` built from the stored compact preview and full arguments.

- [ ] **Step 5: Run approval pause tests**

Run: `cargo test approval_required_tool_pauses_turn`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/engine.rs src/state
git commit -m "feat(engine): pause approval-required tool calls"
```

## Task 6: Resume Approved Operation

**Files:**

- Modify: `src/engine.rs`
- Modify: `src/state/approvals.rs`

- [ ] **Step 1: Add resume tests**

Add tests proving:

- approved operation executes exactly once
- duplicate approval does not execute twice
- final answer is stored in the same session
- pending turn is deleted after terminal completion
- pending approval audit row remains

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test resume_approved_operation`

Expected: fail because resume method does not exist.

- [ ] **Step 3: Add resume input and method**

Add:

```rust
pub struct ResumeApprovedOperationInput {
    pub approval_id: String,
    pub session_id: String,
    pub surface: crate::state::types::Surface,
    pub actor_id: String,
}
```

Implement `resume_approved_operation(input) -> anyhow::Result<EngineResponse>`.

- [ ] **Step 4: Continue provider loop from pending state**

Load `PendingTurnState`, execute the stored tool once, mark executed or failed, append tool result, continue provider loop with stored messages/tools/generation, store final assistant answer, delete pending turn.

- [ ] **Step 5: Run resume tests**

Run: `cargo test resume_approved_operation`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/engine.rs src/state
git commit -m "feat(engine): resume approved operations once"
```

## Task 7: Denial Resume Flow

**Files:**

- Modify: `src/engine.rs`
- Modify: `src/state/approvals.rs`

- [ ] **Step 1: Add denial tests**

Add tests proving denial:

- marks approval denied
- appends a denial tool result
- calls provider again
- stores final assistant explanation
- deletes pending turn

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test deny_approval_resumes_model_turn`

Expected: fail because denial resume is missing.

- [ ] **Step 3: Implement denial path**

Call `state::approvals::deny_approval`, load pending turn, append:

```json
{ "error": "User denied approval." }
```

as the tool result, continue provider loop, store final assistant answer, and delete pending turn.

- [ ] **Step 4: Run denial tests**

Run: `cargo test deny_approval_resumes_model_turn`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/engine.rs src/state
git commit -m "feat(engine): resume model turn after denial"
```

## Final Verification

- [ ] **Step 1: Run engine tests**

Run: `cargo test engine`

Expected: all engine tests pass.

- [ ] **Step 2: Run state/model/tool tests**

Run: `cargo test state model tools`

Expected: state, model, and tool tests pass.

- [ ] **Step 3: Run full Rust tests**

Run: `cargo test`

Expected: all Rust tests pass.

- [ ] **Step 4: Run docs checks**

Run: `bun run docs:check`

Expected: markdown and internal docs links pass.

- [ ] **Step 5: Search for old engine policy**

Run: `rg -n "remote_policy|requires_confirmation|ToolRisk|role: \\\"model\\\"|max_file_preview_chars|tool_execution_policy_error" src/engine.rs`

Expected: no source-specific approval gates remain; any remaining references are compatibility tests scheduled for removal in the same PR.
