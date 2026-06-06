# Prompt Context Assembly

OpenNivara already has a rich context compiler and memory compiler. Do not rewrite them. The approval-resume system needs a stable model-history boundary so `PendingTurnState` can freeze the exact model turn and resume without recomputing context.

Target boundary:

```text
ContextCompilerOutput
-> PromptAssembly
-> Vec<ModelMessage>
-> PendingTurnState.messages_so_far
```

## Current Context

OpenNivara already supports:

- `contexts.toml` entries with `send_policy` values such as `always`, `manual`, `session_pinned`, `triggered_strict`, `never`, and `disabled`
- context selection through triggers, pinned IDs, and score thresholds
- memory compiler sections for system/style/profile/preferences/settings contexts/skills/runtime/location/route/task/memory/graph/workspace/current user message
- prompt audit records with selected context, memory, workspace metadata, and token budget information
- engine behavior that stores the raw user message in DB, then sends a compiled context-rich current user prompt to the model
- `build_history_with_compiled_current` behavior that avoids duplicating the raw current user message in model history

Current conflicts with the new design:

1. Existing system policy tells the model not to read secrets, `.env`, API keys, SSH keys, passwords, or credentials. This conflicts with the new liberal read policy.
2. Current history uses roles `user` and `model`; the new schema uses `user`, `assistant`, `tool`, `event`, and `system`.
3. Current engine owns too much prompt assembly inline.

## PromptAssembly

Suggested location:

```text
src/engine/context.rs
```

Types:

```rust
pub struct PromptAssembly {
    pub audit_id: Option<String>,
    pub messages: Vec<ModelMessage>,
    pub selected_skill_ids: Vec<String>,
    pub pinned_context_ids: Vec<String>,
    pub pinned_skill_ids: Vec<String>,
    pub warnings: Vec<String>,
}
```

Suggested function:

```rust
assemble_model_messages_for_turn(
    conn: &Connection,
    input: PromptAssemblyInput,
) -> anyhow::Result<PromptAssembly>
```

Responsibilities:

- load pinned context/skill IDs
- compile context
- load prior user/assistant history
- build `Vec<ModelMessage>`
- return audit ID and selected metadata

## Model-History Contract

For a normal user turn, model history should contain:

1. prior conversation window
2. current compiled user prompt
3. assistant message with `ToolCall` if provider asks for a tool
4. tool message with `ToolResult` if a tool executes, is denied, or fails
5. final assistant answer

In `ModelMessage` terms:

- prior user/assistant messages
- current user message with compiled context
- assistant message with `ToolCall`
- tool message with `ToolResult`
- assistant final text

For v1, preserve existing behavior where compiled context is sent as the current user message. Do not split into system/developer/user messages yet.

Later, if provider abstraction supports system/developer roles consistently, split:

- system policy to `ModelRole::System`
- profile/style/preferences/context to system/developer-style messages
- current user message to `ModelRole::User`

## Store Versus Recompute Rules

Store in `PendingTurnState`:

- `messages_so_far`
- tools declaration
- pending tool call
- provider ID/model ID
- generation config
- current round/max rounds
- compiled context audit ID
- selected skill IDs
- pinned context IDs
- pinned skill IDs
- request ID
- turn ID
- session ID
- user message ID
- surface
- actor ID

Do not recompute on approval resume:

- profile/style/preferences/context
- workspace map brief
- memory retrieval
- skill selection
- tool declarations
- conversation history window

Approval resume must continue the same turn. Files, memory, profile, or config may have changed while approval was pending; recomputation would change the prompt.

Store `compiled_context_audit_id` in `PendingTurnState`. Do not duplicate every audit detail there. The pending turn must still store the exact `ModelMessage` history required to resume.

## System Policy Update

Replace conservative "do not read secrets/.env/API keys/SSH keys/passwords/credentials" language.

New direction:

```text
You may use available tools when useful. Read-only, opening, workspace indexing, external read/search, and sending context to Gemini may run automatically. Operations that modify, delete, externally mutate, or are unknown will require user approval through OpenNivara. If a tool result says approval was denied, do not claim the operation happened.
```

Remove:

```text
Do not try to read secrets, API keys, SSH keys, .env files, passwords, or credentials.
```

Keep:

- Do not claim you read a file unless a tool result confirms it.
- Do not claim an operation happened unless a tool result confirms it.

## Conversation History Roles

New DB role values:

- `user`
- `assistant`
- `tool`
- `event`
- `system`

Normal model conversation window should include:

- `user`
- `assistant`

Exclude by default:

- `event`
- `tool`
- `system`

Current-turn tool calls/results are held in model history. Past tool/event messages can be noisy. If needed later, summarize past tool activity into session summaries or memory rather than replaying all events.

Approval events are stored in DB as `role = event` for chat/audit display. They should not be included in the normal model conversation window.

A denied or failed current pending tool should be represented to the model as `ModelPart::ToolResult` in `PendingTurnState`, not as a DB event message.

## Raw User Message Versus Compiled Prompt

Keep this pattern:

- DB stores raw human-readable user message.
- Model receives compiled context-rich current prompt.
- Prompt audit links raw `user_message_id` to compiled prompt/audit.

Keep a simple max message window for v1. Store the exact selected history in `PendingTurnState`.

Future config:

```toml
[generation]
max_conversation_history_messages = 20
```

## Skills And Workspace Map

Keep current skill rule:

- selected skills can narrow tool declarations
- selected skills cannot bypass operation approval

For resume:

- store selected skill IDs
- store actual tool declarations
- do not recompute skill selection

Keep workspace map tool-discoverable. Do not inject full map contents by default. The model can use:

- `map_summary`
- `map_tree`
- `map_search`
- `map_get_node`

These are read-only and automatic.

## Locked Decisions

1. Keep current memory/context compiler.
2. Preserve compiled prompt as current user model message for v1.
3. Store raw user message in DB; compiled prompt in model history.
4. Store full `ModelMessage` history in `PendingTurnState` when approval pauses.
5. Do not recompute context, skills, tools, or history on approval resume.
6. Store `compiled_context_audit_id` in `PendingTurnState`.
7. Prior model history includes user + assistant only, not event/tool messages.
8. Rename old DB/model role `model` to `assistant` in the new schema.
9. Update system policy to match liberal read/open/tool policy.
10. Selected skills narrow tool declarations only; approval remains operation-classification based.
11. Workspace map remains tool-discoverable, not fully injected.
12. Add `PromptAssembly` boundary/helper so engine does not own all context assembly inline.

## Tests

Required tests:

1. `PromptAssembly` includes prior user/assistant messages.
2. `PromptAssembly` excludes event/tool messages from prior history.
3. Current raw user message is stored in DB.
4. Current compiled prompt is sent as `ModelRole::User`.
5. `PendingTurnState` stores exact `ModelMessage` history.
6. Approval resume does not re-run context compiler.
7. selected skill IDs and tool declarations survive resume.
8. `compiled_context_audit_id` is present when audit exists.
9. System policy no longer blocks reading secrets, `.env`, or credentials.
10. Tool denial/failure is represented as `ModelPart::ToolResult`, not DB event history.
