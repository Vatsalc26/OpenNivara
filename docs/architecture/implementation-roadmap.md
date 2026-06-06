# Implementation Roadmap

This roadmap sequences the architecture work without creating one massive refactor PR.

Use small, testable, CI-green PR slices. Do not build connectors before approval/model/tool/state foundations. Do not expose mutating tools before preview plus approval pause/resume exists. Do not expose `delete_memory` until hard-delete semantics are honest.

## PR Status

PR-0 Rust workspace hygiene is complete. The repository now has an explicit root workspace containing the root `opennivara` package and `desktop/src-tauri`, uses resolver 2, keeps the Tauri crate depending on `opennivara` by path, and uses the root `Cargo.lock` as the workspace lockfile.

## First Alpha Approval Vertical Slice

The first complete proof is the alpha approval vertical slice:

```text
CLI + MockProvider + write_file create_new/overwrite + approval pause/resume
```

The existing [MVP Vertical Slice](mvp-vertical-slice.md) and [MVP Completion Acceptance Gate](mvp-completion-acceptance-gate.md) document names are historical. For implementation, treat them as the first alpha approval vertical slice and alpha approval acceptance gate. This slice proves request/turn IDs, raw message storage, model tool calls, `LocalModify` classification, `ToolPreview`, pending approval/turn storage, CLI approval, exactly-once execution, `ModelVisibleToolResult`, provider continuation, completion cleanup, and durable audit rows.

`write_file` V1 semantics are defined in [write_file V1](write-file-v1.md). The deterministic provider/test harness is defined in [MockProvider Test Harness](mock-provider-test-harness.md).

Do not use `run_command`, Desktop, Telegram, connectors, or memory tools as the first vertical slice.

## PR Sequence

### PR 0: Rust Workspace Hygiene

Status: complete.

Scope completed:

- root `[workspace]` includes `"."` and `"desktop/src-tauri"`
- workspace uses resolver 2
- root package remains `opennivara`
- Tauri crate depends on `opennivara` by path
- root `Cargo.lock` is authoritative
- CI/docs use workspace-aware Cargo commands where appropriate

### PR 1: Docs Sync And Roadmap

Scope:

- update [Docs Status](../STATUS.md)
- update this roadmap
- update [Implementation Sequencing Plan](../superpowers/plans/2026-06-06-implementation-sequencing.md)
- record PR-0 completion
- clarify that MVP-named docs mean the first alpha approval vertical slice
- include newer design decisions: `completed` status, pending turn phases, `request_id`/`turn_id`, `UserFacingError`, `ModelVisibleToolResult`, `PromptAssembly`, memory tools/proposals, connector foundation, `http_get`, and GitHub V1A/V1B

Do not include:

- code behavior changes

Acceptance:

- docs index links remain valid
- docs checks pass
- no code behavior changes

### PR 2: Runtime IDs And Request/Turn Envelopes

Scope:

- `src/runtime/ids.rs`
- `EngineRequest` gets `request_id` and `created_at`
- `RequestSource -> Surface` and `actor_id` helpers
- `TurnEnvelope`
- prepare `EngineResponse` shape with `request_id` and `turn_id`

Do not include:

- state migrations
- approval resume
- Desktop UI
- connectors

Acceptance:

- ID helpers use correct prefixes
- IDs are unique
- Desktop/CLI/Telegram source maps correctly
- `EngineResponse` can carry `request_id` and `turn_id`

### PR 3: State DB Migrations

Scope:

- `src/state/db.rs`
- `src/state/migrations.rs`
- `src/state/migrations/V1__initial_state_schema.sql`
- `src/state/migrations/V2__approval_resume.sql`
- legacy alpha backup/reset

Do not include:

- engine refactor
- approval execution
- surface UX

Acceptance:

- fresh DB migrates
- legacy inline DB is backed up/reset
- schema includes sessions, messages, active sessions, pending approvals, and pending turns
- constraints/indexes pass tests

### PR 4: Typed State API

Scope:

- `state::sessions`
- `state::messages`
- `state::active_sessions`
- `state::approvals`
- `state::recovery`
- `state::views`

Include APIs:

- `create_pending_approval_with_turn`
- `begin_execution_once`
- `deny_approval_and_update_turn`
- `mark_tool_executed_and_update_turn`
- `mark_tool_failed`
- `mark_resume_failed`
- `mark_approval_completed`
- `complete_denied_turn`
- `recover_stale_executing_approvals`
- `cleanup_completed_pending_turns`

Acceptance:

- duplicate approval cannot execute twice
- wrong session is rejected
- `executed` does not mean `completed`
- pending turn phase transitions are enforced
- stale executing recovery marks failed/interrupted
- completed cleanup keeps audit row

### PR 5: Shared Types And Specta Contract

Scope:

- `EngineResponseKind`
- `EngineResponse`
- `ApprovalView`
- `ApprovalActionResponse`
- `ApprovalStatus`
- `PendingTurnPhase`
- `ToolPreviewEnvelope`
- `ToolExecutionResult`
- `ModelVisibleToolResult`
- `UserFacingError`
- `ErrorKind`
- `Surface`

Regenerate:

- `desktop/src/generated/backendTypes.ts`

Acceptance:

- `cargo test bindings_are_current` passes
- frontend typecheck passes
- old Desktop `AskResponse` is replaced or aligned

### PR 6: Model Gateway And MockProvider

Scope:

- `src/model/types.rs`
- `src/model/provider.rs`
- `src/model/gemini.rs`
- `src/model/mock.rs`
- move Gemini-native request/response structs out of `engine.rs`

Acceptance:

- native `ModelMessage`/`ModelPart` round-trip
- Gemini conversion tests pass
- `MockProvider` can script plain text, tool call, tool-result continuation, and provider failure
- `tool_call_id` is generated when provider lacks one

### PR 7: Tool Operation Policy And Liberal Config Defaults

Scope:

- `OperationKind`
- `OperationClassification`
- `OperationDecision`
- `requires_approval`
- shell classifier
- `ToolsConfig` v2 defaults
- `allowed_roots = []` unrestricted
- `blocked_patterns = []`
- `read_file` enabled
- remove unimplemented tools from default `tools.toml`

Acceptance:

- implemented tools classify `ReadOnly`
- unknown operations require approval
- shell classifier table tests pass
- old `requires_confirmation` parses but does not decide approval

### PR 8: ToolPreview, ToolExecutionResult, And Model-Visible Results

Scope:

- `ToolPreview`
- `ToolPreviewEnvelope`
- `ToolRegistry::preview`
- activity previews for read tools
- `write_file` V1 preview for `create_new` and `overwrite`
- `ToolExecutionResult`
- `ModelVisibleToolResult`
- truncation helpers
- redaction helper if not already added

Acceptance:

- preview never mutates
- automatic read tools get activity preview
- model-visible tool result uses `ok/result/error` envelope
- `write_file` preview never mutates and execution revalidates after approval
- large output truncation is explicit
- invalid preview creates no approval

### PR 9: Engine Foundation Refactor

Scope:

- engine uses state APIs
- engine uses `ModelProvider`
- engine creates `TurnEnvelope`
- engine stores raw user message and receives DB message ID
- engine assembles `PromptAssembly`
- automatic read tools still work

Do not include:

- approval pause/resume

Acceptance:

- plain answer still works
- automatic tool flow still works
- Desktop/CLI/Telegram source maps to surface/actor ID
- prior history uses user/assistant only
- context is compiled once per turn

### PR 10: Approval Pause Storage

Scope:

- `EngineResponseKind::ApprovalRequired`
- create pending approval
- create pending turn
- store approval-required event
- return `ApprovalView`

Acceptance:

- approval-required tool does not execute
- pending approval row exists
- pending turn has frozen model history
- `ApprovalView` has preview/action booleans

### PR 11: Approval Resume, Denial, And Recovery

Scope:

- `approve_pending_operation`
- `deny_pending_operation`
- `resume_pending_continuation`
- exactly-once tool execution
- `executed -> completed` distinction
- `mark_resume_failed`
- stale executing recovery

Acceptance:

- approve executes once
- duplicate approve is blocked
- provider failure after tool success does not rerun tool
- continue retries provider only
- deny creates model-visible `approval_denied` tool result
- pending turn is deleted only after final answer/explanation

### PR 12: CLI Approval UX

Scope:

- CLI approval commands/prompts
- approval lists/details

Docs:

- [CLI Approval UX](cli-approval-ux.md)
- [Surface Consistency Matrix](surface-consistency-matrix.md)

Acceptance:

- same chat/session can approve
- executed approval says to use continue
- completed approvals are hidden by default
- CLI non-interactive mode never auto-approves
- CLI `--json` emits shared DTOs rather than ad hoc JSON
- interactive TTY prompt supports approve, deny, details, and quit
- Enter defaults to no/deny

### PR 13: Opening And Mutating Local Tools

Recommended split:

- PR 13A opening tools
- PR 13B write/delete previews and execution
- PR 13C `run_command`

Scope:

- `open_url`
- `open_file`
- `open_app`
- `write_file`
- `write_binary_file`
- `delete_file`
- `run_command`

Acceptance:

- open tools are automatic
- write/delete require approval
- `write_file` has diff preview
- `write_binary_file` has hash/bytes preview
- `delete_file` is file-only, no directories/globs
- `run_command` classifier controls approval

### PR 14: Desktop Approval Card

Scope:

- Desktop approval card
- Desktop continue card
- generated shared approval types in frontend tests

Docs:

- [Desktop Approval Card State Model](desktop-approval-card-state-model.md)
- [Surface Consistency Matrix](surface-consistency-matrix.md)

Acceptance:

- Desktop derives UI from backend `ApprovalView`
- Desktop executed state shows Continue response only
- completed approvals are hidden by default
- no retry operation is exposed in the alpha slice

### PR 15: Telegram Approval Commands

Scope:

- Telegram `/approvals`
- Telegram `/approval <id>`
- Telegram `/approve <id>`
- Telegram `/deny <id>`
- Telegram `/continue <id>`

Docs:

- [Telegram Approval UX](telegram-approval-ux.md)
- [Surface Consistency Matrix](surface-consistency-matrix.md)

Acceptance:

- same-chat approval only
- no special Telegram tool policy layer
- previews are concise by default
- completed approvals are hidden by default

### PR 16: Memory Tools And Memory Proposal UX

Scope:

- `remember_this`
- `create_memory`
- `update_memory`
- `forget_memory`
- memory proposal cards/commands
- `memory::tools` and `tools::memory` bridge

Do not expose:

- `delete_memory`

Reason: hard-delete scope has unresolved prompt-audit/session-summary cleanup blockers.

Acceptance:

- `remember_this` creates proposal in `AskBeforeSaving`
- `AutoSaveLowRisk` can autosave normal high-confidence memory
- update/forget require approval
- forget uses retraction
- memory proposal approval commands are separate from operation approval

### PR 17: http_get

Scope:

- `http_get` external read
- URL validation
- timeout/max bytes/content type
- external-read preview
- `ModelVisibleToolResult`

Acceptance:

- HTTP/HTTPS only
- automatic `ExternalRead`
- truncation metadata
- no credentials
- logs redacted

### PR 18: Connector Foundation

Scope:

- `src/connectors`
- `ConnectorDefinition`
- `ConnectorAccount`
- `CredentialMetadata`
- `CredentialStore`
- `MockCredentialStore`
- state DB tables for connector metadata
- no real SDK yet

Acceptance:

- metadata is stored without secrets
- mock credential store works
- scopes round-trip
- tools are not exposed without connected account/scope

### PR 19: GitHub Read-Only Connector

Scope:

- GitHub connector definition
- `github_list_repositories`
- `github_fetch_issue`
- `github_search_issues`
- `github_fetch_pr`
- `github_fetch_file`

SDK:

- consider `octocrab` here if useful
- do not add `octocrab` before connector abstraction exists

Acceptance:

- `ExternalRead` automatic
- tool exposure depends on connected account/scopes
- previews show account/repo/target
- results use `ModelVisibleToolResult`

### PR 20: GitHub Low-Risk Mutations

Scope:

- `github_create_issue`
- `github_comment_issue`

Acceptance:

- `ExternalMutation` requires approval
- preview shows account/repo/title/body/scopes
- denial model-visible result works
- approve executes once

## Locked Decisions

1. Use existing roadmap/status/sequencing docs; do not create a second roadmap system.
2. Start code with runtime IDs and state migrations, not UI.
3. Keep PRs small and independently testable.
4. Build approval infrastructure before mutating tools.
5. Build connector foundation before GitHub connector.
6. Add SDKs only when connector implementation needs them.
7. Keep first GitHub connector read-only before mutations.
8. Keep first alpha approval vertical slice to CLI + MockProvider + `write_file`.
