# Core Agent Contract

OpenNivara is a liberal local agent. It may inspect the user's computer, build context from local data, open local resources, and send relevant context to Gemini as part of normal operation. The approval boundary is narrow: deleting, modifying, external mutation, and unknown shell commands require explicit approval for that operation; automatic operations do not.

This document is the implementation contract for Desktop, CLI, and Telegram. Surface-specific code may authenticate differently and render approval UX differently, but it must not bypass shared engine policy.

## Core Product Contract

OpenNivara is allowed to do anything on the computer, may access anything on the computer, and may send anything on the computer to Gemini when needed for a user request.

Local-first means OpenNivara primarily stores and operates locally. It does not mean data never leaves the computer. Model requests can include selected local context.

Desktop, CLI, and Telegram can all control the same agent capabilities. There is no broad safety model beyond high-risk operation approval. High risk means deleting, modifying, external mutation, mutating shell commands, deleting shell commands, and unknown shell commands. Approval is per operation, and the default mode is approve once for this operation only.

## Equal Surfaces Decision

Desktop, CLI, and Telegram are equal user surfaces over the same agent engine.

No surface is inherently more trusted or more restricted. Each surface may have different authentication and presentation code, but all three must use:

- the same request lifecycle
- the same context compiler
- the same skill selection path
- the same declared tool set
- the same operation classifier
- the same approval request store
- the same audit model

Surface adapters should stay thin. Their job is to identify the actor, normalize the request, render approval prompts, and return the answer. They must not run tools directly when that would bypass engine policy.

## Request Envelope

Every surface request normalizes into one envelope before it enters the engine:

```text
RequestEnvelope
- request_id
- surface: Desktop | CLI | Telegram
- actor
- session_id
- message
- selected_skill_id
- metadata
```

`actor` is the authenticated or identified user for that surface. `metadata` may include surface-specific transport data such as Telegram chat IDs, CLI invocation details, or desktop window context. Engine policy must not depend on hidden surface state outside the envelope.

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
12. For approval-required operations, create an `ApprovalRequest`.
13. Pause the same agent turn until the user approves or denies the pending operation.
14. If approved, execute the operation once and resume the same agent turn.
15. If denied, feed a denied-tool result back into the model and let the agent continue or explain.
16. Store the assistant response.
17. Store the audit trail.
18. Return the answer to the originating surface.

The same lifecycle applies to all three surfaces. A surface may choose how to display pending approval or final output, but it does not get a separate tool execution path.

## Operation Classification Model

Every tool request is classified before execution. The classifier should use declared tool effects, exact arguments, and shell-command analysis where applicable.

Operation classes:

- `read_only`
- `opening`
- `external_read`
- `external_mutation`
- `local_mutation`
- `local_deletion`
- `shell_read_only`
- `shell_mutating`
- `shell_deleting`
- `shell_unknown`
- `unknown`

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

Approval pauses and resumes the same agent turn. It must not stop the task and ask the user to retry. Pending approvals survive app restart and can resume later from stored turn state. The detailed persistence design is documented in [Approval Resume And State DB](approval-resume-state.md).

An approval request must include:

```text
ApprovalRequest
- id
- request_id
- session_id
- surface
- actor_id
- operation/tool name
- exact arguments
- reason
- classification
- status: pending | approved | denied | executing | executed | failed
- created_at
- resolved_at
- resolved_by_actor_id
- execution_started_at
- execution_finished_at
- resume_payload_json
```

The engine must never execute delete, modify, external-mutation, mutating-shell, deleting-shell, unknown-shell, or unknown operations silently. The engine may execute read-only, opening, indexing, external-read, send-to-Gemini, and clearly read-only shell operations silently.

Approval UX is surface-specific:

- Desktop renders a modal or dialog.
- CLI renders a terminal prompt.
- Telegram accepts `/approve <id>` or `/deny <id>`.

After approval, the operation executes once. If the model requests another mutating or deleting operation, the engine creates another approval request.

Approval is single-use and cannot be replayed. Duplicate approvals must not execute the same operation twice. Denial should be represented as a tool result so the model can continue safely or explain why it cannot continue.

Approval is tied to the particular chat/session where it was requested. Do not allow any surface to approve any pending operation globally. Same-chat approval is required unless a future explicit cross-surface approval design is created.

Approval details should show the operation/tool name, classification, target or summary, preview, expandable full arguments, and classifier reason. Shell command approvals must show the command, classification, classifier reason, and whether it is read-only, mutating, deleting, or unknown. File modification approvals should eventually show a diff.

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

1. Define shared Rust types for `RequestEnvelope`, `Surface`, `Actor`, operation risk classification, and `ApprovalRequest`.
2. Add one engine entry point that accepts `RequestEnvelope` and owns session resolution, message storage, context compilation, skill selection, provider calls, tool classification, approval handling, execution, response storage, and audit writes.
3. Refactor Desktop, CLI, and Telegram request paths into thin adapters that call the shared engine entry point.
4. Define tool metadata with declared effects: read-only, mutating, deleting, or external-state-mutating.
5. Implement the central operation classifier and require approval for every deleting, modifying, external-mutating, mutating-shell, deleting-shell, unknown-shell, or unknown operation.
6. Implement the approval request store, pending turn state store, and per-surface approval UX.
7. Add same-turn approval pause/resume behavior.
8. Add audit records for request lifecycle steps, tool requests, classification decisions, approval decisions, and executions.
9. Add tests proving that automatic operations run without approval across all surfaces.
10. Add tests proving that approval-required operations require per-operation approval across all surfaces.
11. Add tests proving duplicate approval cannot execute the same operation twice.
12. Add regression tests proving no surface can bypass shared engine policy.
