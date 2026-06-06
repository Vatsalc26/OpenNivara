# OpenNivara Implementation Sequencing Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement OpenNivara's agent architecture through small, testable, CI-green PR slices instead of one giant refactor.

**Architecture:** Build runtime IDs, state migrations, typed state APIs, shared DTOs, model gateway, operation policy, tool preview/result contracts, and engine foundations before approval resume and mutating tools. Prove the first vertical slice with CLI + MockProvider + `write_file` before Desktop, Telegram, memory tools, connectors, or shell commands.

**Tech Stack:** Rust 2021, SQLite/refinery, `rusqlite`, Specta, React/Tauri Desktop, CLI, Telegram, provider-neutral model gateway, Gemini adapter, `MockProvider`, tool operation policy.

---

## Existing Docs To Follow

**Files:**

- Read: `docs/STATUS.md`
- Read: `docs/architecture/implementation-roadmap.md`
- Read: `docs/architecture/mvp-vertical-slice.md`
- Read: `docs/architecture/write-file-v1.md`
- Read: `docs/architecture/mock-provider-test-harness.md`
- Read: `docs/architecture/approval-resume-state.md`
- Read: `docs/architecture/recovery-state-machine.md`
- Read: `docs/architecture/state-rust-api.md`
- Read: `docs/architecture/model-provider-gateway.md`
- Read: `docs/architecture/tool-operation-policy.md`
- Read: `docs/architecture/tool-preview-schema.md`
- Read: `docs/architecture/model-visible-tool-results.md`
- Read: `docs/architecture/cli-approval-ux.md`
- Read: `docs/architecture/desktop-approval-card-state-model.md`
- Read: `docs/architecture/telegram-approval-ux.md`

- [ ] **Step 1: Verify docs index**

Run:

```bash
bun run docs:check
```

Expected: markdown lint and internal links pass.

## PR 1: Docs Sync And Roadmap

**Files:**

- Modify: `docs/STATUS.md`
- Modify: `docs/architecture/implementation-roadmap.md`
- Modify: `docs/superpowers/plans/2026-06-06-implementation-sequencing.md`
- Create/modify: `docs/architecture/mvp-vertical-slice.md`
- Modify: `docs/architecture/github-connector-v1.md`

- [ ] **Step 1: Include current decisions**

Confirm these are covered:

- `completed` approval status
- pending turn phases
- `request_id` and `turn_id`
- `UserFacingError`
- `ModelVisibleToolResult`
- `PromptAssembly`
- memory proposal/tools
- connector foundation
- `http_get`
- GitHub V1A/V1B
- MVP vertical slice
- `write_file` V1 create_new/overwrite semantics
- scripted `MockProvider` test harness
- CLI approval command structure
- Desktop approval card state model
- Telegram approval commands

- [ ] **Step 2: Verify docs**

Run:

```bash
bun run docs:check
git diff --check
```

Expected: both commands exit 0.

## PR 2: Runtime IDs And Request/Turn Envelopes

**Files:**

- Create: `src/runtime/ids.rs`
- Modify: `src/runtime/mod.rs`
- Modify: engine request/response types where currently defined
- Test: runtime ID tests and request/source mapping tests

- [ ] **Step 1: Add ID helpers**

Add helpers for `req_`, `turn_`, `msg_`, `sess_`, `appr_`, and `toolcall_` IDs.

- [ ] **Step 2: Add surface/actor mapping**

Map Desktop to `Surface::Desktop`, CLI to `Surface::Cli`, and Telegram chat IDs to `Surface::Telegram` with `actor_id = "telegram_<chat_id>"`.

- [ ] **Step 3: Add turn envelope type**

Add `TurnEnvelope` with `request_id`, `turn_id`, `session_id`, `surface`, `actor_id`, and `created_at`.

- [ ] **Step 4: Verify**

Run targeted Rust tests for ID prefixes, uniqueness, mapping, and `EngineResponse` carrying `request_id`/`turn_id`.

## PR 3: State DB Migrations

**Files:**

- Create: `src/state/db.rs`
- Create: `src/state/migrations.rs`
- Create: `src/state/migrations/V1__initial_state_schema.sql`
- Create: `src/state/migrations/V2__approval_resume.sql`
- Test: state migration tests

- [ ] **Step 1: Add migration runner**

Implement embedded refinery migration startup in `open_state_db`.

- [ ] **Step 2: Add V1 schema**

Create sessions, messages, active sessions, pinned contexts, and pinned skills.

- [ ] **Step 3: Add V2 approval schema**

Create pending approvals and pending turns with `completed` status and pending turn phases.

- [ ] **Step 4: Add legacy backup/reset**

Back up old inline alpha DBs before creating the migrated DB.

- [ ] **Step 5: Verify**

Run state migration tests for fresh DB, legacy reset, constraints, and indexes.

## PR 4: Typed State API

**Files:**

- Create: `src/state/sessions.rs`
- Create: `src/state/messages.rs`
- Create: `src/state/active_sessions.rs`
- Create: `src/state/approvals.rs`
- Create: `src/state/recovery.rs`
- Create: `src/state/views.rs`
- Test: state API tests

- [ ] **Step 1: Add sessions/messages/active sessions APIs**

Implement typed helpers that use the migrated schema and return typed records.

- [ ] **Step 2: Add approval transition APIs**

Implement `create_pending_approval_with_turn`, `begin_execution_once`, `deny_approval_and_update_turn`, `mark_tool_executed_and_update_turn`, `mark_tool_failed`, `mark_resume_failed`, `mark_approval_completed`, and `complete_denied_turn`.

- [ ] **Step 3: Add recovery APIs**

Implement `recover_stale_executing_approvals` and `cleanup_completed_pending_turns`.

- [ ] **Step 4: Verify**

Run state tests proving duplicate approval cannot execute twice, wrong sessions reject, `executed` does not mean `completed`, phases are enforced, and completed cleanup keeps the audit row.

## PR 5: Shared Types And Specta Contract

**Files:**

- Modify: shared backend type definitions
- Modify: `desktop/src/generated/backendTypes.ts`
- Test: bindings and frontend typecheck

- [ ] **Step 1: Add shared DTOs**

Add `EngineResponseKind`, `EngineResponse`, `ApprovalView`, `ApprovalActionResponse`, `ApprovalStatus`, `PendingTurnPhase`, `ToolPreviewEnvelope`, `ToolExecutionResult`, `ModelVisibleToolResult`, `UserFacingError`, `ErrorKind`, and `Surface`.

- [ ] **Step 2: Regenerate bindings**

Run the existing Specta generation command used by the repo.

- [ ] **Step 3: Verify**

Run `cargo test bindings_are_current` and frontend typecheck.

## PR 6: Model Gateway And MockProvider

**Files:**

- Create: `src/model/types.rs`
- Create: `src/model/provider.rs`
- Create: `src/model/gemini.rs`
- Create: `src/model/mock.rs`
- Modify: `src/engine.rs`
- Test: model tests

- [ ] **Step 1: Add native model types**

Define `ModelMessage`, `ModelPart`, `ModelToolCall`, `ModelToolDeclaration`, `ModelRequest`, `ModelResponse`, and `GenerationConfig`.

- [ ] **Step 2: Move Gemini code**

Move Gemini-native structs and HTTP calls out of engine code.

- [ ] **Step 3: Add MockProvider**

Support scripted plain text, tool call, tool-result continuation, and provider failure.

Follow [MockProvider Test Harness](../../architecture/mock-provider-test-harness.md): record every `ModelRequest`, provide call-count assertions, and support assertion steps for model-visible tool results.

- [ ] **Step 4: Verify**

Run model tests for round-trip, Gemini conversion, mock scripts, and generated `tool_call_id`.

## PR 7: Tool Operation Policy And Config Defaults

**Files:**

- Create/modify: `src/tools/operation_policy.rs`
- Modify: tool definition/config code
- Test: tool policy tests

- [ ] **Step 1: Add operation policy types**

Add `OperationKind`, `OperationClassification`, `OperationDecision`, and `requires_approval`.

- [ ] **Step 2: Add shell classifier**

Classify known read-only, mutating, deleting, and unknown commands.

- [ ] **Step 3: Apply liberal defaults**

Set `read_file` enabled, `allowed_roots = []` unrestricted, `blocked_patterns = []`, and remove unimplemented tools from default `tools.toml`.

- [ ] **Step 4: Verify**

Run tool tests proving implemented tools are read-only, unknown operations require approval, shell classifier table passes, and `requires_confirmation` no longer decides approval.

## PR 8: ToolPreview, ToolExecutionResult, And Model-Visible Results

**Files:**

- Create/modify: tool preview/result modules
- Modify: `ToolRegistry`
- Test: preview/result tests

- [ ] **Step 1: Add preview support**

Implement `ToolPreview`, `ToolPreviewEnvelope`, and `ToolRegistry::preview`.

- [ ] **Step 2: Add result envelopes**

Implement `ToolExecutionResult` and `ModelVisibleToolResult`.

- [ ] **Step 3: Add truncation/redaction helpers**

Make large output truncation explicit and keep secrets out of previews/results/logs.

- [ ] **Step 4: Verify**

Run tests proving previews never mutate, automatic read previews work, model-visible JSON uses `ok/result/error`, and invalid preview creates no approval.

## PR 9: Engine Foundation Refactor

**Files:**

- Modify: `src/engine.rs`
- Modify: state/model/tool integration call sites
- Test: engine foundation tests

- [ ] **Step 1: Use state APIs**

Store raw user messages through state APIs and capture DB message IDs.

- [ ] **Step 2: Use ModelProvider**

Replace direct provider calls with the provider abstraction.

- [ ] **Step 3: Assemble PromptAssembly**

Compile prompt/context once per turn and store selected history in turn state.

- [ ] **Step 4: Verify**

Run engine tests proving plain answer works, automatic tool flow works, source maps to surface/actor, prior history uses user/assistant only, and context compiles once.

## PR 10: Approval Pause Storage

**Files:**

- Modify: `src/engine.rs`
- Modify: state approval integration
- Test: approval pause tests

- [ ] **Step 1: Return ApprovalRequired**

Return `EngineResponseKind::ApprovalRequired` when a tool requires approval.

- [ ] **Step 2: Store pending state**

Create pending approval, pending turn, and approval-required event.

- [ ] **Step 3: Verify**

Run tests proving the tool does not execute, pending rows exist, frozen model history is stored, and `ApprovalView` has preview/action booleans.

## PR 11: Approval Resume, Denial, And Recovery

**Files:**

- Modify: engine approval action APIs
- Modify: state recovery integration
- Test: approval resume/recovery tests

- [ ] **Step 1: Implement approve**

Call `begin_execution_once`, execute once, append tool result, and continue provider.

- [ ] **Step 2: Implement deny**

Append model-visible `approval_denied`, continue provider, and keep approval status `denied`.

- [ ] **Step 3: Implement continue**

Retry provider continuation only for `executed/tool_executed_awaiting_model` or `denied/denied_awaiting_model`.

- [ ] **Step 4: Verify**

Run tests proving duplicate approve is blocked, provider failure after tool success does not rerun the tool, denial is model-visible, and pending turn cleanup waits for final answer/explanation.

## MVP Vertical Slice Checkpoint

**Files:**

- Use: `docs/architecture/mvp-vertical-slice.md`
- Use: `docs/architecture/write-file-v1.md`
- Use: `docs/architecture/mock-provider-test-harness.md`
- Surface: CLI only
- Provider: `MockProvider`
- Tool: `write_file` with `create_new` and `overwrite`

- [ ] **Step 1: Run happy path**

Ask from CLI to create `notes.txt` with `hello world`.

Expected: engine returns approval, CLI approves, file is written once, final answer is stored, approval becomes `completed`, pending turn is deleted, audit row remains.

- [ ] **Step 2: Run denial path**

Deny the same operation.

Expected: file is not written, model receives `approval_denied`, final denial explanation is stored, pending turn is deleted.

- [ ] **Step 3: Run provider failure/continue path**

Script `MockProvider` to fail after tool success, then continue.

Expected: status remains `executed`, continue retries provider only, tool is not executed again, final answer completes the approval.

- [ ] **Step 4: Run write_file preview checks**

Call `write_file` preview for `create_new` and `overwrite`.

Expected: preview never mutates, `create_new` fails if file exists, `overwrite` fails if file is missing, and overwrite preview includes unified diff/truncation metadata when needed.

## PR 12: Surface Approval UX

**Files:**

- Modify: Desktop approval UI
- Modify: CLI approval commands/prompts
- Modify: Telegram command handling
- Test: surface approval tests

- [ ] **Step 1: Add Desktop cards**

Render approval and continuation cards from generated types.

Follow `docs/architecture/desktop-approval-card-state-model.md`. The frontend derives actions from `ApprovalView` status/phase/can booleans and does not implement its own transition logic.

- [ ] **Step 2: Add CLI commands**

Add approval prompts and approval list/show/approve/deny/continue commands.

Follow `docs/architecture/cli-approval-ux.md`. Non-interactive approval-required ask must print commands and never auto-approve. `--json` must emit shared DTOs.

- [ ] **Step 3: Add Telegram commands**

Wire `/approve`, `/deny`, and `/continue` to engine APIs.

Follow `docs/architecture/telegram-approval-ux.md`. MVP uses commands only, same-chat approval, concise previews, and no full argument JSON dumps by default.

- [ ] **Step 4: Verify**

Run tests proving same chat/session can approve, wrong chat/session is rejected, executed approvals say use continue, completed is hidden by default, non-interactive CLI never auto-approves, Telegram previews are truncated, and frontend tests import generated types.

## PR 13: Opening And Mutating Local Tools

Recommended split:

- PR 13A: `open_url`, `open_file`, `open_app`
- PR 13B: `write_file`, `write_binary_file`, `delete_file`
- PR 13C: `run_command`

- [ ] **Step 1: Add opening tools**

Opening tools are automatic.

- [ ] **Step 2: Add write/delete tools**

Write/delete tools require approval and have previews/diffs.

- [ ] **Step 3: Add run_command**

Shell classifier controls approval.

- [ ] **Step 4: Verify**

Run tool tests for open automatic behavior, write/delete previews, file-only deletion, and shell classification.

## PR 14: Memory Tools And Proposal UX

**Files:**

- Create/modify: `src/memory/tools.rs`
- Create/modify: `src/tools/memory.rs`
- Modify: surface memory proposal UX
- Test: memory tool/proposal tests

- [ ] **Step 1: Add creation tools**

Implement `remember_this` and `create_memory`.

- [ ] **Step 2: Add update/forget tools**

Implement `update_memory` and `forget_memory`; forget uses retraction.

- [ ] **Step 3: Keep delete disabled**

Do not expose `delete_memory`.

- [ ] **Step 4: Verify**

Run tests proving proposals are separate from operation approvals, `AskBeforeSaving` creates proposals, `AutoSaveLowRisk` can autosave normal high-confidence memory, and update/forget require approval.

## PR 15: http_get

**Files:**

- Create/modify: external read tool implementation
- Test: `http_get` tests

- [ ] **Step 1: Add unauthenticated GET**

Support HTTP/HTTPS only, URL validation, timeout, max bytes, content type, final URL, and truncation metadata.

- [ ] **Step 2: Verify**

Run tests proving `http_get` is automatic `ExternalRead`, rejects unsupported schemes, includes truncation metadata, uses no credentials, and logs are redacted.

## PR 16: Connector Foundation

**Files:**

- Create: `src/connectors`
- Modify: state DB migrations for connector metadata
- Test: connector foundation tests

- [ ] **Step 1: Add connector types**

Add `ConnectorDefinition`, `ConnectorAccount`, `CredentialMetadata`, and scopes.

- [ ] **Step 2: Add credential store**

Add `CredentialStore` and `MockCredentialStore`.

- [ ] **Step 3: Add metadata storage**

Store account/credential metadata without secrets.

- [ ] **Step 4: Verify**

Run tests proving metadata stores without secrets, mock credential store works, scopes round-trip, and tools are not exposed without account/scope.

## PR 17: GitHub V1A Read-Only Connector

**Files:**

- Create/modify: GitHub connector implementation
- Test: GitHub read tests

- [ ] **Step 1: Add GitHub definition**

Expose V1A capabilities through `ConnectorToolProvider`.

- [ ] **Step 2: Add read tools**

Implement `github_list_repositories`, `github_fetch_issue`, `github_search_issues`, `github_fetch_pr`, and `github_fetch_file`.

- [ ] **Step 3: Verify**

Run tests proving `ExternalRead` automatic behavior, account/scope-gated exposure, previews show account/repo/target, and results use `ModelVisibleToolResult`.

## PR 18: GitHub V1B Low-Risk Mutations

**Files:**

- Modify: GitHub connector implementation
- Test: GitHub mutation approval tests

- [ ] **Step 1: Add issue creation/comment tools**

Implement `github_create_issue` and `github_comment_issue`.

- [ ] **Step 2: Verify approval behavior**

Run tests proving `ExternalMutation` requires approval, previews show account/repo/title/body/scopes, denial returns model-visible `approval_denied`, and approve executes once.

## Constraints

- Do not build connectors before approval/model/tool/state foundations.
- Do not expose mutating tools before preview plus approval pause/resume exists.
- Do not expose `delete_memory` until hard-delete semantics are honest.
- Do not add unimplemented tools to default `tools.toml`.
- Keep each PR independently testable and CI-green.
- Add SDKs only when the connector implementation needs them.
