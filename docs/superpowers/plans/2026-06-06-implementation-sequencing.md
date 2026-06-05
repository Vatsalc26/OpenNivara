# Approval Resume Implementation Sequencing Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Sequence OpenNivara's state migration, typed state API, operation policy, model gateway, engine approval flow, and surface UX work into small testable phases.

**Architecture:** Build stable foundations before approval resume: state migrations first, typed state APIs second, operation classification third, provider abstraction fourth, then engine refactor and approval pause/resume. Keep Desktop, CLI, and Telegram equal throughout. Use operation classification as the approval source of truth.

**Tech Stack:** Rust 2021, SQLite/refinery, `rusqlite`, model provider gateway, Gemini adapter, tool operation policy, Desktop/CLI/Telegram surfaces.

---

## Phase 0: Documentation

**Files:**

- Modify: `docs/architecture/core-agent-contract.md`
- Modify: `docs/architecture/approval-resume-state.md`
- Create/modify: `docs/architecture/tool-operation-policy.md`
- Create/modify: `docs/architecture/state-rust-api.md`
- Create: `docs/architecture/model-provider-gateway.md`
- Create: `docs/architecture/engine-approval-flow.md`

- [ ] **Step 1: Confirm docs exist**

Run:

```bash
rg --files docs/architecture | rg "core-agent-contract|approval-resume-state|tool-operation-policy|state-rust-api|model-provider-gateway|engine-approval-flow"
```

Expected: all six architecture docs are listed.

- [ ] **Step 2: Run docs checks**

Run: `bun run docs:check`

Expected: markdown lint and internal links pass.

## Phase 1: State DB Embedded Migrations

**Plan:** [State DB Approval Resume Implementation Plan](2026-06-05-state-db-approval-resume.md)

**Goal:** Replace inline state DB schema creation with embedded refinery migrations.

- [ ] **Step 1: Add `src/state` migration skeleton**

Implement `src/state/mod.rs`, `db.rs`, `migrations.rs`, and V1/V2 SQL migration files.

- [ ] **Step 2: Add legacy alpha backup/reset**

Detect old inline DBs without refinery metadata, rename to `opennivara_state.legacy-reset-YYYYMMDD-HHMMSS.sqlite`, and run fresh migrations.

- [ ] **Step 3: Verify migration tests**

Run: `cargo test state::migrations state::db`

Expected: fresh DB migration, legacy reset, tables, columns, indexes, and constraints pass.

## Phase 2: State Rust API

**Plan:** [State Rust API Implementation Plan](2026-06-05-state-rust-api.md)

**Goal:** Add typed, transaction-safe state APIs over the migrated schema.

- [ ] **Step 1: Add typed sessions/messages/active sessions**

Implement `state::sessions`, `state::messages`, and `state::active_sessions`.

- [ ] **Step 2: Add transactional approvals**

Implement `create_pending_approval_with_turn`, `begin_execution_once`, `deny_approval`, `mark_executed`, `mark_failed`, and `delete_pending_turn`.

- [ ] **Step 3: Verify state API tests**

Run: `cargo test state`

Expected: typed records, atomic approval creation, duplicate execution guard, wrong-session rejection, and pending-turn cleanup pass.

## Phase 3: Tool Operation Classification

**Plan:** [Tool Operation Policy Implementation Plan](2026-06-05-tool-operation-policy.md)

**Goal:** Make `OperationClassification` the source of truth for approval.

- [ ] **Step 1: Add operation policy types**

Implement `OperationKind`, `OperationClassification`, `OperationDecision`, and `requires_approval`.

- [ ] **Step 2: Mark tool definitions**

Add `operation_kind` to `ToolDefinition` and mark current implemented tools as read-only.

- [ ] **Step 3: Add liberal config defaults**

Enable `read_file`, default `blocked_patterns = []`, make `allowed_roots = []` unrestricted, and remove unimplemented future tools from generated defaults.

- [ ] **Step 4: Verify tool policy tests**

Run: `cargo test tools`

Expected: current tools are read-only, unknown tools require approval, shell classifier works, liberal defaults pass, and `ToolRisk` does not decide approval.

## Phase 4: Provider Abstraction / Model Gateway

**Plan:** [Model Provider Gateway Implementation Plan](2026-06-06-model-provider-gateway.md)

**Goal:** Move Gemini-native structs and HTTP calls out of `src/engine.rs` before pending turn serialization.

- [ ] **Step 1: Add native model types**

Implement `ModelRole`, `ModelMessage`, `ModelPart`, `ModelToolCall`, `ModelToolDeclaration`, `ModelRequest`, `ModelResponse`, and `GenerationConfig`.

- [ ] **Step 2: Add Gemini provider**

Move Gemini structs and HTTP logic into `src/model/gemini.rs`, with native conversion in both directions.

- [ ] **Step 3: Add mock provider**

Implement deterministic provider tests for text, tool call, tool result continuation, and provider error.

- [ ] **Step 4: Verify model tests**

Run: `cargo test model`

Expected: native type round-trips, Gemini conversion, mock provider, and pending-turn native serialization pass.

## Phase 5: ToolPreview And Mutating/Open Tools

**Plan:** [Tool Preview And Mutating Tools Implementation Plan](2026-06-06-tool-preview-mutating-tools.md)

**Goal:** Add read-only previews, approval previews, diff schemas, compact approval storage, and the next opening/mutating tool catalog.

- [ ] **Step 1: Add `ToolPreview`**

Implement the serializable preview type and `ToolRegistry::preview`.

- [ ] **Step 2: Add read-only activity previews**

Generate optional non-blocking previews for `read_file`, `list_dir`, and `file_exists`.

- [ ] **Step 3: Add mutating previews**

Generate diff previews for `write_file`, metadata/hash previews for `write_binary_file`, and stat/hash previews for `delete_file`.

- [ ] **Step 4: Add compact approval storage**

Add `operation_target` and `reason` to `pending_approvals`, store compact preview JSON there, and keep full args in pending turn state.

- [ ] **Step 5: Verify preview tests**

Run: `cargo test tools state`

Expected: previews are read-only, invalid preview creates no approval, diff truncation works, and compact approval preview storage passes.

## Phase 6: Engine Refactor Onto Foundations

**Plan:** [Engine Approval Flow Implementation Plan](2026-06-06-engine-approval-flow.md)

**Goal:** Make the engine use state APIs, provider gateway, and operation classifier without full resume UX yet.

- [ ] **Step 1: Normalize surface and actor**

Map Desktop, CLI, and Telegram requests to `Surface` and owner `actor_id`.

- [ ] **Step 2: Use state APIs**

Store user messages with surface/actor and assistant messages as `assistant`.

- [ ] **Step 3: Use model provider**

Replace direct Gemini HTTP with `ModelProvider`.

- [ ] **Step 4: Simplify tool declaration**

Declare by global enabled, individual enabled, and selected skill allowlist only.

- [ ] **Step 5: Verify engine foundation tests**

Run: `cargo test engine`

Expected: equal surfaces, provider abstraction, selected skill allowlist, and existing automatic tool behavior pass.

## Phase 7: Approval Pause Storage

**Goal:** Pause when an approval-required tool is requested and persist the pending turn.

- [ ] **Step 1: Add `EngineResponseKind`**

Support `Answer` and `ApprovalRequired`.

- [ ] **Step 2: Store approval-required turn**

When classification requires approval, create pending approval, pending turn, and approval-required event through the state approval API.

- [ ] **Step 3: Verify pause tests**

Run: `cargo test approval_required_tool_pauses_turn`

Expected: DB rows exist, response includes approval ID, and pending turn contains model history with the tool call.

## Phase 8: Approval Recovery State

**Plan:** [Approval Recovery State Implementation Plan](2026-06-06-approval-recovery-state.md)

**Goal:** Make approval resume safe across crashes, provider failures, duplicate approvals, and partial execution states.

- [ ] **Step 1: Add completed status and pending turn phases**

Update schema/types so `executed` is intermediate and `completed` is terminal success.

- [ ] **Step 2: Add recovery transition helpers**

Implement `mark_tool_executed_and_update_turn`, `mark_resume_failed`, `mark_approval_completed`, stale executing recovery, and completed cleanup.

- [ ] **Step 3: Verify recovery tests**

Run: `cargo test state engine`

Expected: duplicate approvals do not re-execute tools, provider continuation retries are tracked, and completed cleanup preserves audit rows.

## Phase 9: Approval Resume And Denial

**Goal:** Resume the same model turn after approve or deny.

- [ ] **Step 1: Implement approved resume**

Call `begin_execution_once`, execute the stored pending tool once, append tool result, continue provider loop, store final answer, and delete pending turn.

- [ ] **Step 2: Implement denial resume**

Mark denied, append denial tool result, continue provider loop, store explanation, and delete pending turn.

- [ ] **Step 3: Verify resume tests**

Run: `cargo test resume_approved_operation deny_approval_resumes_model_turn`

Expected: approved operations execute once, duplicate approval is blocked, denial is model-visible, pending turn is cleaned up, and approval audit remains.

## Phase 10: Desktop/CLI/Telegram Approval UX

**Goal:** Expose the same approval model across all equal surfaces.

- [ ] **Step 1: Desktop UX**

Add a same-chat approval dialog with preview, expandable details, approve once, and deny.

- [ ] **Step 2: CLI UX**

Add same-session approval prompt and resume behavior.

- [ ] **Step 3: Telegram UX**

Add `/approve <id>` and `/deny <id>` in the same chat/session.

- [ ] **Step 4: Verify UX tests**

Run: `cargo test approval_command` and relevant desktop tests.

Expected: same-chat approval succeeds, wrong-session approval rejects, unauthorized actor rejects, and events appear in chat history.

## Phase 11: Hardening

**Goal:** Lock behavior and remove old policy remnants.

- [ ] **Step 1: Search for old policy gates**

Run:

```bash
rg -n "remote_policy|requires_confirmation|ToolRisk|source_created|role: \"model\"|tool_execution_policy_error" src
```

Expected: remaining hits are compatibility code, display-only UI, migrations/tests, or intentionally removed in this phase.

- [ ] **Step 2: Run full verification**

Run:

```bash
cargo test
bun run docs:check
```

Expected: Rust tests and docs checks pass.

## Recommended PR Order

1. Docs architecture decisions.
2. State DB migrations.
3. State Rust API.
4. Tool operation classification and config cleanup.
5. Provider abstraction/model gateway.
6. ToolPreview, compact approval previews, and mutating/open tool catalog.
7. Engine refactor onto state/model/tool policy foundations.
8. Approval pause storage.
9. Approval recovery state.
10. Approval resume/deny execution.
11. Desktop/CLI/Telegram UX.
12. Hardening/regression cleanup.
