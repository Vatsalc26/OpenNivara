# Request And Turn Envelopes

Stable IDs are the backbone for approval resume, recovery, logs, and Desktop/CLI/Telegram UX.

OpenNivara should introduce stable IDs and envelopes across engine, state, tools, provider, approval, and logs.

## Current Context

Current `EngineRequest` has:

- `source`
- `session_id`
- `message`
- `ui_selected_skill_id`
- `pin_selected_skill`

It does not have:

- `request_id`
- `turn_id`
- `actor_id`
- `surface`
- `client_message_id`
- `created_at`

Current `EngineResponse` has:

- `session_id`
- `answer`

It cannot represent:

- `request_id`
- `turn_id`
- approval-required response
- `approval_id`
- recovery/continuation state

Current session/message helpers generate IDs internally. Approval resume needs `user_message_id`, so `store_message` must return `DbMessage`.

## ID Types

Use prefixed IDs:

- `request_id`: `req_<uuid>`
- `turn_id`: `turn_<uuid>`
- `session_id`: `sess_<uuid>`
- `message_id`: `msg_<uuid>`
- `tool_call_id`: `toolcall_<uuid>`
- `approval_id`: `appr_<uuid>`
- `resume_attempt_id`: `resume_<uuid>`

Meanings:

- `request_id`: one external call into the engine, such as Desktop ask click, CLI Enter, Telegram `/ask`, or Telegram `/continue`.
- `turn_id`: one model/tool-loop reasoning turn, grouping provider calls, model messages, tool calls, approvals, and final answer.
- `session_id`: chat/session ID. New schema should prefer `sess_<uuid>`, while old raw UUIDs can be accepted during transition.
- `message_id`: DB message ID. `user_message_id` is required in `pending_approvals` and `pending_turns`.
- `tool_call_id`: stable ID for a model tool call. Provider adapters must generate one if the provider does not supply one.
- `approval_id`: one approval, one operation, one execution.
- `resume_attempt_id`: one provider/model continuation retry. No DB table is required yet, but include it in logs/event metadata when useful.

Prefixed IDs make logs, Telegram commands, and DB inspection easier.

## Runtime ID Helper

Add:

```text
src/runtime/ids.rs
```

Functions:

```rust
new_request_id() -> String
new_turn_id() -> String
new_session_id() -> String
new_message_id() -> String
new_tool_call_id() -> String
new_approval_id() -> String
new_resume_attempt_id() -> String
```

Export `runtime::ids` from `src/runtime/mod.rs` or through the root crate path.

No new dependency is needed. `uuid` is already present.

## Request Source Mapping

Keep `RequestSource` as the canonical source/surface input for now.

Add helpers:

```rust
RequestSource::surface() -> Surface
RequestSource::actor_id() -> String
```

Mappings:

- Desktop: `surface = desktop`, `actor_id = desktop_owner`
- CLI: `surface = cli`, `actor_id = cli_owner`
- Telegram: `surface = telegram`, `actor_id = telegram_<chat_id>`

Do not duplicate `surface` and `actor_id` inside `EngineRequest` if they can be derived from `RequestSource`. This avoids mismatch bugs.

## EngineRequest Target

```rust
pub struct EngineRequest {
    pub request_id: String,
    pub source: RequestSource,
    pub session_id: Option<String>,
    pub message: String,
    pub ui_selected_skill_id: Option<String>,
    pub pin_selected_skill: bool,
    pub client_message_id: Option<String>,
    pub created_at: String,
}
```

Add:

```rust
EngineRequest::new(source, session_id, message)
```

The constructor should fill:

- `request_id`
- `created_at`
- `client_message_id = None`

`ui_selected_skill_id` and `pin_selected_skill` can be filled through a builder or explicit constructor args.

## TurnEnvelope

Create `TurnEnvelope` inside `handle_message` after session and user message are known:

```rust
pub struct TurnEnvelope {
    pub request_id: String,
    pub turn_id: String,
    pub session_id: String,
    pub surface: Surface,
    pub actor_id: String,
    pub user_message_id: String,
    pub created_at: String,
}
```

Pass `TurnEnvelope` into:

- `PendingTurnState`
- approval creation
- tool preview
- provider request metadata
- tracing spans

## EngineResponse Target

```rust
pub enum EngineResponseKind {
    Answer,
    ApprovalRequired,
}

pub struct EngineResponse {
    pub request_id: String,
    pub turn_id: String,
    pub session_id: String,
    pub kind: EngineResponseKind,
    pub answer: String,
    pub approval: Option<ApprovalView>,
}
```

For early validation errors before a turn starts, return an error rather than an `EngineResponse` with a missing `turn_id`.

## PendingTurnState Target

```rust
pub struct PendingTurnState {
    pub schema_version: u32,

    pub request_id: String,
    pub turn_id: String,
    pub session_id: String,
    pub user_message_id: String,

    pub surface: Surface,
    pub actor_id: String,

    pub provider_id: String,
    pub model_id: String,

    pub phase: PendingTurnPhase,

    pub messages_so_far: Vec<ModelMessage>,
    pub tools: Vec<ModelToolDeclaration>,
    pub pending_tool_call: ModelToolCall,

    pub generation: GenerationConfig,
    pub current_round: u32,
    pub max_rounds: u32,

    pub compiled_context_audit_id: Option<String>,
    pub selected_skill_ids: Vec<String>,
    pub pinned_context_ids: Vec<String>,
    pub pinned_skill_ids: Vec<String>,

    pub created_at: String,
    pub updated_at: String,
}
```

## Approval Action Inputs

```rust
pub struct ApprovalActionInput {
    pub request_id: String,
    pub approval_id: String,
    pub session_id: String,
    pub source: RequestSource,
    pub created_at: String,
}

pub struct ResumeContinuationInput {
    pub request_id: String,
    pub approval_id: String,
    pub session_id: String,
    pub source: RequestSource,
    pub created_at: String,
}
```

Derive `surface` and `actor_id` from `source`.

## State DB Updates

Add `turn_id TEXT NOT NULL` to:

- `pending_approvals`
- `pending_turns`

Add indexes:

```sql
CREATE INDEX idx_pending_approvals_turn_id ON pending_approvals(turn_id);
CREATE INDEX idx_pending_turns_turn_id ON pending_turns(turn_id);
```

Reason:

- `request_id` identifies the external surface call.
- `turn_id` identifies the model/tool loop that owns the approval.

`pending_approvals` must include:

- `request_id`
- `turn_id`
- `user_message_id`
- `tool_call_id`

`pending_turns` must include:

- `request_id`
- `turn_id`
- `user_message_id`

## State API Changes

`store_message` must return `DbMessage`, not `()`.

```rust
state::messages::store_message(...) -> anyhow::Result<DbMessage>
```

This is required so the engine can store `user_message_id` in:

- `TurnEnvelope`
- `pending_approvals`
- `pending_turns`
- `PendingTurnState`

Migrate state APIs to accept IDs where needed instead of generating them too late.

## Provider Tool Call IDs

The model provider adapter must guarantee every `ModelToolCall` has an ID.

If the provider lacks tool-call IDs, generate `toolcall_<uuid>`.

Gemini currently needs generated `tool_call_id`.

## Tests

Required tests:

1. ID helpers return correct prefixes.
2. ID helpers return unique IDs.
3. `RequestSource` maps to correct `Surface` and `actor_id`.
4. `EngineRequest::new` fills `request_id` and `created_at`.
5. `store_message` returns `DbMessage` with ID.
6. `TurnEnvelope` contains `request_id`, `turn_id`, `session_id`, `user_message_id`, `surface`, and `actor_id`.
7. `EngineResponse` includes `request_id` and `turn_id`.
8. `PendingTurnState` JSON round-trips with `request_id` and `turn_id`.
9. Provider adapter generates `tool_call_id` if missing.
10. `pending_approvals` includes `turn_id`.
11. `pending_turns` includes `turn_id`.
12. approval events include `approval_id`, `request_id`, and `turn_id` where useful.
13. logs/tracing fields can include `request_id` and `turn_id`.
