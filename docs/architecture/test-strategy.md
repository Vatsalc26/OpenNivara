# Architecture Test Strategy

OpenNivara should use the existing test infrastructure heavily. Do not add a new Rust test framework or frontend test framework.

The goal is rigorous scenario coverage for approval, recovery, tools, provider, request/turn envelopes, shared types, and surfaces.

## Existing Tooling

Root Rust crate dev-dependencies already include:

- `tempfile`
- `assert_cmd`
- `predicates`
- `insta`
- `serial_test`
- `pretty_assertions`
- `proptest`

Desktop already has:

- Vitest
- Testing Library
- MSW
- Playwright
- Storybook
- WebdriverIO/Tauri E2E
- coverage tooling
- Biome
- Knip

CI already runs the main Rust, frontend, docs, Storybook, E2E, and skill-pack checks. Keep using those gates first.

## Test Layers

Rust unit tests cover pure logic:

- runtime ID helpers
- `RequestSource` to `Surface` and `actor_id`
- `OperationClassification.requires_approval()`
- shell classifier
- config parsing/defaults
- redaction helper
- `ToolPreview` schema builders
- `ToolExecutionResult` helpers
- `PendingTurnState` serialization
- `ModelMessage` serialization

Rust state integration tests use temp config dirs and real SQLite:

```text
lock TEST_CONFIG_ENV_MUTEX
create tempfile::tempdir()
set OPENNIVARA_TEST_CONFIG_DIR
open_state_db()
```

Use state integration tests for embedded refinery migrations, legacy alpha backup/reset, schema introspection, approval state transitions, atomic duplicate approval guard, pending-turn cleanup, and stale executing recovery. These are P0.

Rust engine tests should use `MockProvider`. Do not hit Gemini in tests.

Use `MockProvider` for:

- plain model answer
- model tool call to automatic read tool to final answer
- model tool call to approval-required response
- approve flow success
- provider failure after tool success
- resume continuation retry
- deny flow
- duplicate approval attempt

Tool tests use temp dirs/files for path and preview behavior. Actual `run_command` execution tests should be rare, harmless, capped, and timeout-limited; most command coverage belongs in classifier/preview tests.

CLI tests use `assert_cmd` and `predicates`. Start with approval subcommands rather than interactive chat prompts.

Telegram tests should not start a real Telegram bot. Factor command handling into pure functions where possible:

```rust
handle_telegram_command(command, chat_id, engine) -> TelegramReply
```

Desktop unit tests use Vitest/jsdom and generated Specta TypeScript types.

Storybook covers approval UI states. E2E remains small and high-value.

## Critical Matrix

P0:

- V1/V2 migrations
- legacy alpha backup/reset
- approval status transitions
- duplicate approval prevention
- pending-turn phase recovery
- provider abstraction with `MockProvider`
- engine approval-required response
- approve executes exactly once
- executed continuation retry never reruns tool
- deny feeds model-visible denial
- operation classification
- shell classifier
- liberal `tools.toml` defaults
- unrestricted read/open path behavior
- Specta bindings current

P1:

- `write_file` diff preview
- `write_binary_file` preview
- `run_command` output limits
- Desktop `ApprovalCard`
- Telegram `/approve`, `/deny`, `/continue` pure handler
- CLI approval subcommands

P2:

- Desktop E2E approval card flow

## Test Helpers

Backend helpers:

- `with_temp_config_dir(...)`
- `open_test_state_db()`
- `insert_test_session(...)`
- `insert_test_user_message(...)`
- `make_test_turn_envelope(...)`
- `make_test_pending_turn_state(...)`
- `make_test_tool_preview(...)`
- `make_test_approval(...)`

Engine helpers:

- `MockProviderScript`
- test tool registry / fake tool executor

Frontend fixtures:

- `approvalViewFixtures.ts`
- `engineResponseFixtures.ts`

## Coverage Strategy

Do not chase arbitrary global coverage first. Focus on scenario completeness for critical backend approval/recovery code.

Modules needing high coverage:

- state migrations/API
- operation policy
- provider gateway
- engine approval flow
- recovery transitions

## Required Decisions

1. Do not add new test frameworks.
2. Use existing Rust dev-dependencies heavily.
3. Use temp config dirs for DB/config integration tests.
4. Add `MockProvider` before engine approval tests.
5. Test recovery transitions at DB helper level first.
6. Test engine approval flow with `MockProvider`, not Gemini.
7. Factor Telegram command handling into pure functions.
8. Use generated Specta types in frontend tests.
9. Use Storybook for approval card state coverage.
10. Keep E2E small and high-value.
