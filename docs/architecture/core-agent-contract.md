# Core Agent Contract

OpenNivara is a liberal local agent. It may inspect the user's computer, build context from local data, open local resources, and send relevant context to Gemini as part of normal operation. The approval boundary is narrow: deleting, modifying, external mutation, mutating shell commands, deleting shell commands, unknown shell commands, and unknown operations require explicit approval for that operation; automatic operations do not.

This document is the implementation contract for Desktop, CLI, and Telegram. Surface-specific code may authenticate differently and render approval UX differently, but it must not bypass shared engine policy. Stable request and turn IDs are documented in [Request And Turn Envelopes](request-turn-envelopes.md), the detailed model boundary is documented in [Model Provider Gateway](model-provider-gateway.md), the engine approval integration is documented in [Engine Approval Flow](engine-approval-flow.md), and the shared surface rendering/type contract is documented in [Surface Approval UX](surface-approval-ux.md) and [Shared Type Contract](shared-type-contract.md).

## Core Product Contract

OpenNivara is allowed to do anything on the computer, may access anything on the computer, and may send anything on the computer to Gemini when needed for a user request.

Local-first means OpenNivara primarily stores and operates locally. It does not mean data never leaves the computer. Model requests can include selected local context.

Desktop, CLI, and Telegram can all control the same agent capabilities. There is no broad safety system beyond high-risk operation approval. High risk means deleting, modifying, external mutation, mutating shell commands, deleting shell commands, unknown shell commands, and unknown operations. Approval is per operation, and the default mode is approve once for this operation only.

## Equal Surfaces Decision

Desktop, CLI, and Telegram are equal user surfaces over the same agent engine.

No surface is inherently more trusted or more restricted. Each surface may have different authentication and presentation code, but all three must use:

- the same request lifecycle
- the same context compiler
- the same skill selection path
- the same declared tool set
- the same operation classifier
- the same model provider gateway
- the same approval request store
- the same pending-turn resume store
- the same audit/event model

Surface adapters should stay thin. Their job is to identify the actor, normalize the request, render approval prompts, and return the answer. They must not run tools directly when that would bypass engine policy.

## Request Envelope

Every surface request normalizes into one envelope before it enters the engine:

```text
RequestEnvelope
- request_id
- surface: Desktop | CLI | Telegram
- actor_id
- session_id
- message
- selected_skill_id
- metadata
```

`actor_id` is the authenticated or identified user for that surface. Suggested owner actor IDs are `desktop_owner`, `cli_owner`, and `telegram_<chat_id>`. `metadata` may include surface-specific transport data such as Telegram chat IDs, CLI invocation details, or desktop window context. Engine policy must not depend on hidden surface state outside the envelope.

## Request Lifecycle

The canonical lifecycle is:

1. Receive request from Desktop, CLI, or Telegram.
2. Authenticate or identify the actor for that surface.
3. Normalize the input into a `RequestEnvelope`.
4. Resolve or create the session.
5. Store the user message.
6. Compile context.
7. Select skills.
8. Declare allowed tools.
9. Call the model provider.
10. If the model asks for a tool, classify the operation.
11. Execute automatic operations immediately.
12. Build a non-mutating tool preview or activity record.
13. For approval-required operations, create an approval record and pending-turn resume record.
14. Store an approval-required event message in the same chat with `role = 'event'` and JSON content.
15. Pause the same agent turn until the user approves or denies the pending operation.
16. If approved, atomically transition `pending` to `executing`, execute the operation once, and resume the same agent turn.
17. If denied, mark the approval `denied`, feed a denied-tool result back into the model, and let the agent continue or explain.
18. Store the assistant response.
19. Store the audit trail.
20. Delete the `pending_turns` row after terminal completion has been handled.
21. Return the answer to the originating surface.

The same lifecycle applies to all three surfaces. A surface may choose how to display pending approval or final output, but it does not get a separate tool execution path.

## Operation Classification Model

Every tool request is classified before execution. The detailed tool classification design is defined in [Tool Operation Policy](tool-operation-policy.md). The classifier should use declared tool effects, exact arguments, and shell-command analysis where applicable.

Operation classes:

- `read_only`
- `opening`
- `workspace_index`
- `external_read`
- `external_mutation`
- `local_modify`
- `local_delete`
- `shell_read_only`
- `shell_mutating`
- `shell_deleting`
- `shell_unknown`
- `unknown`

`OperationClassification` is the source of truth for approval. `ToolRisk` is only a UI/display/severity hint, and `requires_confirmation` is deprecated compatibility state rather than the primary approval decision.

Automatic operations run without approval. Automatic includes:

- reading files
- listing directories
- searching the filesystem
- indexing a workspace
- reading OpenNivara state, memory, or configuration
- searching memory
- compiling context
- sending selected context, files, or data to Gemini
- opening URLs
- opening apps
- opening files in the default app, unless the operation itself modifies or deletes
- external read or search requests
- clearly read-only shell commands

Approval-required operations must create an approval request. Approval-required includes:

- deleting files or folders
- deleting memory, state, configuration, or installed packs/data
- modifying, editing, or overwriting files
- moving or renaming files
- changing file permissions
- changing OpenNivara settings
- modifying memory
- modifying profile, preferences, style, or contexts
- installing or uninstalling packs
- enabling or disabling tools when that changes future behavior
- external create, update, delete, post, send, or other state-changing actions
- mutating shell commands
- deleting shell commands
- unknown shell commands
- unknown operations

External read, external search, and send-to-Gemini operations are automatic. External create, update, delete, post, send, or remote API mutation requires approval. Sending a message, email, or API request is external mutation when it creates, updates, deletes, posts, sends, or changes external state.

Opening a URL, app, or file is automatic unless the operation writes, deletes, or modifies state.

Shell commands have their own policy:

- Clearly read-only commands run automatically.
- Mutating commands require approval.
- Deleting commands require approval.
- Unknown commands require approval.

Ambiguous operations should be classified as approval-required when they can delete, overwrite, edit, move, rename, change permissions, change settings, change memory, install, uninstall, mutate external state, or alter future behavior.

## Approval System

Approval is always per operation. Approval does not automatically approve future operations. Approve-for-session and remember-permanently are not part of the default design. Approval never expires in the current logical model.

Approval pauses and resumes the same agent turn. It must not stop the task and ask the user to retry. Pending approvals survive app restart and can resume later from stored turn state. The detailed persistence design is documented in [Approval Resume And State DB](approval-resume-state.md). The internal Rust API shape is documented in [State Rust API](state-rust-api.md). Pending turn model history uses OpenNivara-native model types from [Model Provider Gateway](model-provider-gateway.md), not provider-native Gemini structs.

An approval record must include:

```text
ApprovalRequest
- id
- request_id
- session_id
- user_message_id
- tool_call_id
- surface
- actor_id
- operation/tool name
- classification
- summary
- arguments_preview_json
- status: pending | denied | executing | executed | failed | completed
- result_summary
- error_message
- created_at
- resolved_at
- resolved_by_actor_id
- execution_started_at
- execution_finished_at
```

The pending turn record must include:

```text
PendingTurnState
- approval_id
- session_id
- request_id
- user_message_id
- provider_id
- model_id
- resume_payload_json
- created_at
```

The full pending turn payload should preserve the request envelope, OpenNivara-native model messages so far, declared model tools, pending tool call, compiled context audit ID, selected skills, pinned contexts, generation config, current tool-loop round, max rounds, and any provider/model state needed to continue the same turn.

Tool preview and approval view details are defined in [Tool Preview Schema](tool-preview-schema.md). Automatic read-only/opening operations may generate preview/activity records for transparency, but only approval-required classifications block for approval.

The engine must never execute delete, modify, external-mutation, mutating-shell, deleting-shell, unknown-shell, or unknown operations silently. The engine may execute read-only, opening, indexing, external-read, send-to-Gemini, and clearly read-only shell operations silently.

Approval UX is surface-specific:

- Desktop renders a modal/dialog attached to the same chat.
- CLI renders a terminal prompt in the same turn/session.
- Telegram accepts `/approve <id>` or `/deny <id>` in the same chat.

After approval, the operation executes once. If the model requests another mutating or deleting operation, the engine creates another approval request.

Approval is single-use and cannot be replayed. Duplicate approvals must not execute the same operation twice. Denial should be represented as a tool result so the model can continue safely or explain why it cannot continue.

Approval is tied to the particular chat/session where it was requested. Do not allow any surface to approve any pending operation globally. Same-chat approval is required unless a future explicit cross-surface approval design is created.

Approval details should show the operation/tool name, classification, target or summary, preview, expandable full arguments, and classifier reason. Shell command approvals must show the command, classification, classifier reason, and whether it is read-only, mutating, deleting, or unknown. File modification approvals should eventually show a diff.

Before executing an approved operation, the shared approval store must use an atomic execution guard:

```sql
UPDATE pending_approvals
SET status = 'executing', execution_started_at = ?
WHERE id = ? AND status = 'pending';
```

Execute only if exactly one row changed.

## Actor Permission Model

Do not add a full actor or permission table yet.

Use plain `actor_id` values and hardcode approval permission for valid owner actors:

- `desktop_owner`
- `cli_owner`
- `telegram_<chat_id>`

Approval permission is necessary but not sufficient. The approval must also belong to the same originating session/chat context.

## State Events

Approval-related chat history is stored in `messages` with `role = 'event'`.

Event message `content` is JSON. Example:

```json
{
  "event_type": "approval_denied",
  "approval_id": "appr_123",
  "operation_name": "write_file",
  "classification": "local_modify",
  "summary": "User denied file modification"
}
```

Use event types such as:

- `approval_required`
- `approval_approved`
- `approval_denied`
- `approval_executed`
- `approval_failed`

## Audit Model

Every request should leave an audit trail sufficient to reconstruct what the agent did and why. The audit trail should record:

- request envelope fields
- actor and surface
- session ID
- stored user message ID
- context compilation summary
- selected skill ID
- declared tools
- model tool requests
- operation classification
- classifier reason
- approval request IDs and status changes
- executed tool name and exact arguments
- tool result summary
- assistant response ID
- timestamps for each major lifecycle step

Audit records should be shared across surfaces. A Telegram request that modifies a file after approval should be auditable in the same model as a Desktop or CLI request.

## Initial Implementation Milestones

1. Define shared Rust types for `RequestEnvelope`, `Surface`, `Actor`, operation risk classification, `ApprovalRequest`, and `PendingTurnState`.
2. Add one engine entry point that accepts `RequestEnvelope` and owns session resolution, message storage, context compilation, skill selection, provider calls, tool classification, approval handling, execution, response storage, and audit writes.
3. Refactor Desktop, CLI, and Telegram request paths into thin adapters that call the shared engine entry point.
4. Define tool metadata with declared effects: read-only, mutating, deleting, or external-state-mutating.
5. Implement the central operation classifier and require approval for every deleting, modifying, external-mutating, mutating-shell, deleting-shell, unknown-shell, or unknown operation.
6. Implement the state DB migration system, typed state API, model provider gateway, operation classifier, approval request store, pending turn state store, and per-surface approval UX.
7. Add same-turn approval pause/resume behavior using provider-neutral pending turn state.
8. Add audit records for request lifecycle steps, tool requests, classification decisions, approval decisions, and executions.
9. Add tests proving that automatic operations run without approval across all surfaces.
10. Add tests proving that approval-required operations require per-operation approval across all surfaces.
11. Add tests proving duplicate approval cannot execute the same operation twice.
12. Add tests proving no surface can bypass shared engine policy.
