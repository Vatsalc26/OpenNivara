# Module Boundaries

OpenNivara should add new modules instead of expanding existing monoliths. Split by responsibility, not by surface, and avoid one giant rename-only PR.

The current high-risk files for growth are:

- `src/engine.rs`
- `src/tools.rs`
- `src/sessions.rs`
- `src/telegram.rs`
- `src/bindings.rs`

The repo already has good modular examples in `memory`, `skills`, and `runtime`.

## Principles

1. Add new modules instead of expanding old monoliths.
2. Split by responsibility, not by surface.
3. Avoid a giant rename-only PR.
4. Keep compatibility wrappers temporarily where needed.
5. Avoid circular dependencies.
6. Engine coordinates state, tools, and model.
7. State, tools, and model stay independent of engine and surfaces.

## Target Top-Level Modules

`src/lib.rs` should eventually expose:

- `bindings`
- `config_paths`
- `config_store`
- `context`
- `context_selector`
- `daemon`
- `engine`
- `first_run`
- `marketplace`
- `memory`
- `model`
- `output`
- `preferences`
- `profile`
- `remote_policy`
- `runtime`
- `secrets`
- `service`
- `state`
- `skills`
- `style`
- `telegram`
- `tools`
- `workspace_map`

Optional later:

- `surfaces`

Keep `sessions` temporarily while migrating:

```rust
pub mod sessions;
pub mod state;
```

`sessions.rs` should become a compatibility wrapper during migration or be removed after all callers move to `state`.

## Target File Tree

```text
src/
  lib.rs
  main.rs
  bindings.rs
  config_paths.rs
  config_store.rs

  engine/
    mod.rs
    types.rs
    request.rs
    turn.rs
    tool_loop.rs
    approvals.rs
    context.rs

  state/
    mod.rs
    db.rs
    migrations.rs
    types.rs
    sessions.rs
    messages.rs
    active_sessions.rs
    approvals.rs
    recovery.rs
    views.rs
    migrations/
      V1__initial_state_schema.sql
      V2__approval_resume.sql

  model/
    mod.rs
    types.rs
    provider.rs
    gemini.rs
    mock.rs

  tools/
    mod.rs
    types.rs
    registry.rs
    config.rs
    path.rs
    operation_policy.rs
    shell_classifier.rs
    preview.rs
    results.rs
    read.rs
    opening.rs
    write.rs
    command.rs

  runtime/
    mod.rs
    ids.rs
    clock.rs
    context.rs
    location.rs
    model_registry.rs

  surfaces/
    mod.rs
    types.rs
    approval_actions.rs
```

Do not create `surfaces/` immediately unless shared surface code grows.

## State Module

`state` owns SQLite state, migrations, typed state records, approval lifecycle primitives, recovery helpers, and approval views.

State may depend on:

- `rusqlite`
- `chrono`
- `serde` / `serde_json`
- `config_paths`
- runtime IDs

State must not depend on:

- engine
- model provider
- Desktop/Tauri
- Telegram UI

## Engine Module

Engine owns request handling, turn envelopes, provider-neutral model/tool loop, approval orchestration, and context compilation glue.

Engine may depend on:

- state
- model
- tools
- runtime
- skills
- context/memory

Engine should not define:

- Gemini API structs
- raw DB schema operations
- desktop-specific UI structs
- Telegram-specific policy

## Model Module

`model` owns provider-neutral model types, provider trait, Gemini adapter, and `MockProvider`.

Model may depend on:

- `reqwest` in `gemini.rs`
- `serde`
- runtime IDs for generated `tool_call_id`

Model must not depend on:

- tools registry
- state DB
- engine internals

## Tools Module

Do not immediately move all of `src/tools.rs` in a huge PR.

Stage 1:

- keep `src/tools.rs`
- add submodules for operation policy, preview, results, and shell classifier

Stage 2:

- move `src/tools.rs` to `src/tools/mod.rs`
- split registry/config/path/execution into focused files

Tools may depend on filesystem/std helpers, `serde`, `similar`, `base64`, `url`, `shell-words`, `open`, and `chrono` if needed.

Tools must not depend on:

- engine
- state DB
- model provider

## Surface And Binding Modules

Keep `telegram.rs` as one file initially. Replace approval scaffolding with engine approval API calls and factor pure approval command helpers for tests.

Keep `bindings.rs` as one file initially. Add grouping comments:

- Runtime
- Engine
- Approvals
- Tools
- Profile/Preferences/Context
- Memory
- Runtime/location

Do not split `bindings.rs` yet.

## Dependency Direction

```text
surfaces / CLI / Desktop / Telegram
  -> engine
engine
  -> state
  -> tools
  -> model
state
  -> SQLite
tools
  -> filesystem / OS
model
  -> provider APIs
```

Runtime/config/helpers are shared lower-level utilities. Bindings export DTOs only; they must not own business logic.

## Locked Decisions

1. Add new modules; do not keep expanding `engine.rs`, `tools.rs`, and `sessions.rs`.
2. Use `state/` as replacement for `sessions.rs`.
3. Use `model/` for provider gateway; Gemini structs leave engine.
4. Use `tools/` submodules for operation policy, preview, results, shell classifier, and future mutating tools.
5. Add `runtime/ids.rs`.
6. Keep `bindings.rs` single but grouped.
7. Keep `telegram.rs` single initially; factor approval command handling for tests.
8. Do not create `surfaces/` unless shared surface code grows.
9. Avoid circular dependencies.
10. Move files gradually to avoid one massive refactor PR.
