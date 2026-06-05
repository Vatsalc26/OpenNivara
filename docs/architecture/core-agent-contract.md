# Core Agent Contract

OpenNivara is a liberal local agent. It may inspect the user's computer, build context from local data, and send relevant context to Gemini as part of normal operation. The approval boundary is narrow: deleting or modifying operations require explicit approval for that operation; read-only operations do not.

This document is the implementation contract for Desktop, CLI, and Telegram. Surface-specific code may authenticate differently and render approval UX differently, but it must not bypass shared engine policy.

## Core Product Contract

OpenNivara is allowed to do anything on the computer, may access anything on the computer, and may send anything on the computer to Gemini when needed for a user request.

Local-first means OpenNivara primarily stores and operates locally. It does not mean data never leaves the computer. Model requests can include selected local context.

Desktop, CLI, and Telegram can all control the same agent capabilities. There is no broad safety model beyond high-risk operation approval. High risk means deleting or modifying. Approval is per operation, and the default mode is approve once for this operation only.

## Equal Surfaces Decision

Desktop, CLI, and Telegram are equal user surfaces over the same agent engine.

No surface is inherently more trusted or more restricted. Each surface may have different authentication and presentation code, but all three must use:

- the same request lifecycle
- the same context compiler
- the same skill selection path
- the same declared tool set
- the same read-only versus mutating/deleting classifier
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
10. If the model asks for a tool, classify the operation as read-only or mutating/deleting.
11. Run read-only operations automatically.
12. For deleting or modifying operations, create an approval request.
13. If approved, execute the operation once.
14. Store the assistant response.
15. Store the audit trail.
16. Return the answer to the originating surface.

The same lifecycle applies to all three surfaces. A surface may choose how to display pending approval or final output, but it does not get a separate tool execution path.

## Operation Model

Read-only operations run without approval. Read-only includes reading files, indexing files, searching the computer, opening local state for analysis, retrieving memory, compiling prompt context, sending selected context to Gemini, and using normal read-only tools.

Deleting or modifying operations require approval. Mutating/deleting includes:

- deleting files or folders
- deleting memory, state, configuration, or installed packs/data
- overwriting or editing files
- moving or renaming files
- changing permissions
- changing OpenNivara settings
- modifying memory
- modifying profile, preferences, style, or contexts
- installing or uninstalling packs
- enabling or disabling tools when that changes future behavior
- running commands that are explicitly destructive or mutating
- using any tool whose declared effect mutates local or external state

The classifier should be based on declared tool effects and exact arguments. Ambiguous operations should be treated as mutating when they can delete, overwrite, edit, move, rename, change permissions, change settings, change memory, install, uninstall, or alter future behavior.

## Approval System

Approval is always per operation. Approval does not automatically approve future operations. Approve-for-session and remember-permanently are not part of the default design.

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
- risk classification
- status: pending | approved | denied | expired | executed
- created_at
- expires_at
```

The engine must never execute delete or modify operations silently. The engine may execute read-only operations silently.

Approval UX is surface-specific:

- Desktop renders a modal or dialog.
- CLI renders a terminal prompt.
- Telegram accepts `/approve <id>` or `/deny <id>`.

After approval, the operation executes once. If the model requests another mutating or deleting operation, the engine creates another approval request.

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
5. Implement the central operation classifier and require approval for every mutating or deleting operation.
6. Implement the approval request store and per-surface approval UX.
7. Add audit records for request lifecycle steps, tool requests, classification decisions, approval decisions, and executions.
8. Add tests proving that read-only operations run without approval across all surfaces.
9. Add tests proving that deleting and modifying operations require per-operation approval across all surfaces.
10. Add regression tests proving no surface can bypass shared engine policy.
