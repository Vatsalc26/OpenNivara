# Engine Approval Flow

This document defines how the OpenNivara engine should use equal surfaces, state APIs, model provider abstractions, operation classification, and approval resume.

The engine should expose implemented enabled tools liberally, classify actual tool calls, build read-only tool previews, execute automatic operations immediately, and pause/resume the same model turn for approval-required operations.

Approval recovery semantics are defined in [Approval Recovery State Machine](recovery-state-machine.md). The most important invariant is that an approved tool must never run again after status reaches `executed`; provider/model continuation is the only retryable part.

Engine errors should map to the typed user-facing contract in [Error Taxonomy](error-taxonomy.md). Model-visible tool success, denial, and failure payloads should follow [Model-Visible Tool Results](model-visible-tool-results.md) and the layer boundaries in [Tool Result Schema](tool-result-schema.md).

Prompt/context assembly should follow [Prompt Context Assembly](prompt-context-assembly.md). Approval resume continues the stored model history; it must not recompute context, memory retrieval, skill selection, tool declarations, or conversation history.

Memory extraction runs only after a final assistant answer or denial explanation is stored. Memory proposal behavior and explicit memory tools are defined in [Memory Proposals And Tools](memory-proposals-and-tools.md).

## Current Engine Context

`src/engine.rs` currently owns a Gemini-specific tool-calling loop. It already:

- resolves or creates a session
- stores the user message
- loads tools config
- compiles context
- builds Gemini history
- declares tools
- calls Gemini
- detects `function_call` parts
- executes tools via `ToolRegistry`
- pushes `function_response` back into Gemini history
- stores the final model answer

Current behavior to replace:

- Telegram tool declaration is filtered through `remote_policy`.
- Desktop only gets low-risk tools that do not require confirmation.
- `tool_execution_policy_error` blocks based on selected skill allowlist, Telegram policy, and Desktop risk/confirmation logic.
- `read_file` has Telegram-specific truncation behavior.
- Assistant responses are stored as role `model`.
- Messages store source strings instead of `surface` and `actor_id`.

## Equal Surface Normalization

Keep `RequestSource` for now if useful, but derive `Surface` and `actor_id` before state writes or policy decisions.

Mapping:

```text
RequestSource::Desktop -> surface = Desktop, actor_id = "desktop_owner"
RequestSource::Cli -> surface = Cli, actor_id = "cli_owner"
RequestSource::Telegram { chat_id, .. } -> surface = Telegram, actor_id = "telegram_<chat_id>"
```

The engine must not apply special Telegram/Desktop approval logic. Approval is decided by `OperationClassification`, not by surface, `ToolRisk`, or `requires_confirmation`.

## State API Usage

Use the state module APIs:

- `state::db::open_state_db`
- `state::sessions`
- `state::messages`
- `state::active_sessions`
- `state::approvals`

Do not manually insert/update `pending_approvals`, `pending_turns`, and event messages separately in engine code. Use the high-level state approval API.

Store user messages with:

- `role = MessageRole::User`
- derived `surface`
- derived `actor_id`

Store assistant responses with:

- `role = MessageRole::Assistant`
- engine/system surface representation from the final state type design
- `actor_id = None` or `opennivara`

Do not store assistant responses as role `model` in the new schema.

## Tool Declaration Rules

Declare tools based on:

1. global tools enabled
2. individual tool enabled
3. selected skill allowlist

Do not filter tool declarations by:

- Telegram remote policy
- Desktop low-risk-only policy
- `requires_confirmation`
- `ToolRisk`

The model may see approval-required tools if they are implemented, enabled, and allowed by the selected skill policy. If the model calls one, the engine pauses and creates an approval request.

Skills can narrow available tools. Skills must not bypass approval requirements.

Replace `tool_execution_policy_error` with a precheck that only handles:

- unknown tool
- disabled tool
- selected skill allowlist block

## Operation Classification

After the provider returns a tool call:

1. Look up `ToolDefinition`.
2. Classify the call with `operation_policy::classify_tool_call`.
3. Read `OperationDecision`:
   - `classification`
   - `approval_required`
   - `reason`

`OperationClassification` is the approval source of truth.

## Automatic Operation Flow

If `approval_required` is false:

1. Build a lightweight `ToolPreview` or activity record if useful.
2. Execute the tool immediately through `ToolRegistry::execute`.
3. Optionally store/show activity.
4. Push a `ToolResult` into in-memory model history.
5. Continue the tool loop.

`ToolRisk` remains a UI/display hint only. `requires_confirmation` is deprecated compatibility state and is not an engine approval source.

## Approval-Required Operation Flow

If `approval_required` is true:

1. Do not execute the tool.
2. Build read-only `ToolPreview`.
3. If preview fails because arguments are invalid, return a tool error and create no approval.
4. Create a `pending_approvals` row with compact preview fields.
5. Create a `pending_turns` row with full arguments and model resume state.
6. Insert a `role = 'event'` approval-required chat message.
7. Return `EngineResponseKind::ApprovalRequired`.
8. Do not push a policy-denied function response yet.

The same turn is paused until approval or denial.

Pending turn state must include:

- request envelope or equivalent request metadata
- request ID
- turn ID
- session ID
- user message ID
- model history so far, including the model tool-call message
- tool declarations
- pending tool call
- operation classification
- operation reason
- compiled context audit ID
- selected skill IDs
- pinned context IDs
- provider ID
- model ID
- generation config
- current round
- max rounds
- any provider/model state needed to resume the same turn

The current engine already pushes the model content containing a function call into history before executing tools. Store that history in `PendingTurnState` so resume is exact.

## Engine Response Shape

Change `EngineResponse` to support approval pause:

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

For `ApprovalRequired`:

- `kind = ApprovalRequired`
- `answer` is a human-readable approval prompt
- `approval = Some(ApprovalView)`

The prompt should summarize:

- operation/tool name
- classification
- target/summary
- reason
- how to approve or deny for the current surface

## Resume Approved Operation

Add:

```rust
resume_approved_operation(input) -> anyhow::Result<EngineResponse>
```

Suggested input:

```rust
pub struct ResumeApprovedOperationInput {
    pub request_id: String,
    pub approval_id: String,
    pub session_id: String,
    pub source: RequestSource,
    pub created_at: String,
}
```

Approved flow:

1. Call `state::approvals::begin_execution_once`.
2. Validate same session/chat context.
3. Validate actor approval permission.
4. Atomically transition `pending` to `executing`.
5. Load `PendingTurnState`.
6. Execute the stored pending tool call once.
7. If the tool succeeds, call `mark_tool_executed_and_update_turn` to atomically set status `executed`, set phase `tool_executed_awaiting_model`, and persist history with the tool result appended.
8. If the tool fails, call `mark_tool_failed`.
9. Continue the model loop using stored history and stored tool declarations.
10. If provider continuation fails after tool success, call `mark_resume_failed` and leave status `executed`.
11. Store final assistant answer in the same session.
12. Mark approval `completed`.
13. Delete pending turn after terminal completion.
14. Return `EngineResponseKind::Answer` to the original chat/surface.

## Denial Flow

Denial should resume the model turn, not merely stop.

Flow:

1. Call `state::approvals::deny_approval`.
2. Load `PendingTurnState` if still present.
3. Append a tool result with denial, such as `{ "error": "User denied approval." }`.
4. Call the provider again so denial is model-visible.
5. Store the final assistant answer or explanation.
6. Delete pending turn after terminal completion.
7. Return the answer to the same chat/surface.

Denied approvals are terminal from the approval status perspective, but the pending turn can temporarily use phase `denied_awaiting_model` while the denial tool result is fed back to the model.

## Resume Invariants

`PendingTurnState` should store `current_round` and `max_rounds` so resume keeps the same tool-loop budget.

`PendingTurnState` should store tool declarations. Resume should use the same declared tools rather than recompiling.

Do not necessarily store full compiled context separately if it is already inside model history. Store `compiled_context_audit_id` for traceability.

If status is `executed` and phase is `tool_executed_awaiting_model`, resume retries provider/model continuation only. It must not execute the tool again.

If status is `executing` and stale beyond the configured threshold, recovery should mark the approval failed with `Execution was interrupted before completion could be confirmed.`

Remove Telegram-only `read_file` truncation from the engine. If output truncation is needed, make it global/general through `max_bytes` or `max_chars` settings.

## Old Policies To Remove

Remove or replace:

- Telegram-specific tool gating
- Desktop low-risk-only declaration filtering
- `requires_confirmation`-based declaration blocking
- `ToolRisk`-based execution blocking
- Telegram-only file preview truncation

## Required Tests

Add tests for:

1. Desktop, CLI, and Telegram normalize to correct surface and actor ID.
2. User message is stored with surface and actor ID.
3. Assistant message is stored as role `assistant`.
4. Tool declarations are not filtered differently by surface.
5. Selected skill allowlist still blocks undeclared tools.
6. `requires_confirmation` does not block read-only tool declaration.
7. `ToolRisk` does not block execution.
8. Read-only tool calls execute immediately.
9. Read-only tool calls can produce activity previews without approval.
10. Approval-required tool calls build `ToolPreview` before approval.
11. Preview failure creates no approval.
12. Approval-required tool calls create pending approval, pending turn, and event message.
13. Approval-required response includes `ApprovalView`.
14. `PendingTurnState` includes history with the model tool call.
15. Resume approved operation executes exactly once.
16. Duplicate approve cannot execute twice.
17. Denial is fed back as a tool result and resumes the model turn.
18. `current_round` and `max_rounds` survive resume.
19. Tool declarations survive resume.
20. Telegram-specific file truncation is removed or generalized.
21. Final response after resume is stored in the same session.
22. Final response after resume returns to the original same chat/surface.
23. Executed approval retries do not re-execute tools.
24. Provider failure after tool success records resume failure metadata.
25. Completed approval cleanup deletes pending turn and preserves audit row.

## Implementation Milestones

PR 1 normalizes surface/actor and state writes:

- Add surface/actor normalization helpers.
- Store user messages through state APIs.
- Store assistant messages as `assistant`, not `model`.
- Keep behavior otherwise similar.

PR 2 simplifies tool declarations:

- Declare by global enabled, individual enabled, and selected skill allowlist.
- Remove Telegram/Desktop source-specific declaration filters.
- Keep selected skill allowlist.

PR 3 integrates operation classification:

- Classify provider tool calls.
- Execute automatic operations immediately.
- Return `ApprovalRequired` and store pending approval/turn/event for approval-required operations.

PR 4 adds resume and denial:

- Add `resume_approved_operation`.
- Add denial resume flow.
- Continue provider loop from `PendingTurnState`.

PR 5 removes old policy code:

- Remove or replace `tool_execution_policy_error`.
- Remove source-specific policy tests.
- Add equal-surface regression tests.

## Approval Action API Update

Expose these engine-level methods:

```text
approve_pending_operation(input) -> ApprovalActionResponse
deny_pending_operation(input) -> ApprovalActionResponse
resume_pending_continuation(input) -> ApprovalActionResponse
```

`resume_pending_continuation` is separate from approve. It never runs the tool; it only retries provider/model continuation.

`ApprovalActionInput`:

- `approval_id: String`
- `session_id: String`
- `surface: Surface`
- `actor_id: String`

`ResumeContinuationInput` has the same fields.

`ApprovalActionResponse`:

- approval ID
- session ID
- status
- message
- optional engine response
- optional approval view

Approve clicked:

1. call `begin_execution_once`
2. if `Started`, execute tool
3. otherwise return current state message
4. never execute unless `Started`

Tool execution success:

1. append tool result to pending turn model history
2. call `mark_tool_executed_and_update_turn`
3. call provider/model continuation
4. if provider succeeds, store assistant answer and call `mark_approval_completed`
5. if provider fails, call `mark_resume_failed`

Tool execution failure:

1. append tool failure result if possible
2. call `mark_tool_failed`
3. call provider for explanation if possible
4. delete pending turn after explanation

Deny clicked:

1. append denied tool result to pending turn model history
2. call `deny_approval_and_update_turn`
3. call provider/model continuation
4. if provider succeeds, store assistant answer and call `complete_denied_turn`
5. if provider fails, call `mark_resume_failed`

Continuation retry:

1. allow only `executed/tool_executed_awaiting_model` or `denied/denied_awaiting_model`
2. load stored pending turn
3. retry provider/model continuation only
4. never execute the tool

Startup/recovery:

1. run `recover_stale_executing_approvals(10 minutes)`
2. run `cleanup_completed_pending_turns()`
3. do not automatically retry provider continuations at startup
4. surface recoverable continuations with "This operation already executed, but the final response is pending. Resume?"
