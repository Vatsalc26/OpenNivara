# Implementation Roadmap

This roadmap sequences the architecture work without creating one massive refactor PR.

## Order

1. Add `runtime/ids.rs`.
2. Add the `state/` module with embedded migrations and typed APIs.
3. Add tools submodules while keeping `src/tools.rs`.
4. Add the `model/` module and `MockProvider`.
5. Convert `engine.rs` to use state/model/tool-policy foundations.
6. Move `engine.rs` to `engine/mod.rs` only when internal pieces are covered by tests.
7. Keep `telegram.rs` and `bindings.rs` as single files at first.
8. Remove or shrink `sessions.rs` last.

Do not start by moving `engine.rs` and `tools.rs` into folders immediately. Add new modules first, integrate them, then split large files when behavior has tests.

## Phase 1: Runtime IDs

- Add prefixed ID helpers in `src/runtime/ids.rs`.
- Export `runtime::ids`.
- Add tests for prefixes and uniqueness.
- Start threading `request_id` and `turn_id` through new code paths.

## Phase 2: State Module

- Add `src/state`.
- Add embedded refinery migration runner.
- Add `V1__initial_state_schema.sql`.
- Add `V2__approval_resume.sql`.
- Add legacy alpha DB backup/reset.
- Add state DB tests with temp config dirs.
- Keep `src/sessions.rs` as compatibility wrapper where needed.

## Phase 3: Tools Foundations

- Add operation classification.
- Add shell classifier.
- Add `ToolPreview`.
- Add `ToolExecutionResult`.
- Add mutating/opening tool preview behavior.
- Keep old `src/tools.rs` until tests cover behavior.

## Phase 4: Model Gateway

- Add provider-neutral model types.
- Move Gemini structs and HTTP call into `model/gemini.rs`.
- Add `ModelProvider` trait.
- Add `MockProvider` for tests.
- Guarantee every model tool call has `toolcall_<uuid>`.

## Phase 5: Engine Approval Integration

- Normalize `RequestSource` to `Surface` and `actor_id`.
- Store user messages through state APIs and capture `user_message_id`.
- Create `TurnEnvelope`.
- Use provider-neutral model/tool loop.
- Execute automatic tools immediately.
- Pause approval-required operations with pending approval and pending turn.
- Add approve, deny, and continue APIs.
- Add recovery-safe transitions.

## Phase 6: Surface UX

- Desktop returns generated `EngineResponse`.
- Desktop renders approval and continuation cards.
- CLI adds approval subcommands.
- Telegram wires `/approve`, `/deny`, and `/continue` into engine APIs.
- Completed approvals are hidden by default.

## Phase 7: Shared Types And Bindings

- Export runtime, engine, approval, tool, and recovery DTOs with Specta.
- Regenerate `desktop/src/generated/backendTypes.ts`.
- Remove hand-written duplicate response types where possible.
- Keep `bindings_are_current` as the contract gate.

## Phase 8: Cleanup

- Remove old source-specific policy code.
- Remove Telegram-only file truncation or generalize it.
- Shrink or remove `sessions.rs`.
- Split `engine.rs` and `tools.rs` after test coverage is in place.

## Cross-Cutting Rules

- Do not add an ORM.
- Do not add a service framework.
- Do not add a new typegen framework.
- Do not add a new test framework.
- Keep state/tools/model independent of engine and surfaces.
- Use the existing Rust module system and Specta workflow.

## Required Integration Tests

1. New modules compile with no circular dependencies.
2. `runtime::ids` helpers are usable by state/model/engine.
3. State module can run migrations independently of engine.
4. Model module can convert/provider-test independently of engine.
5. Tools operation-policy tests run without state/engine.
6. Engine tests can use `MockProvider` and temp state DB.
7. Old session callers either migrate or compatibility wrapper works.
8. `bindings_are_current` passes after new DTO exports.
9. Telegram approval command helper can be tested without running bot.
10. No module imports the desktop crate into the root library.
