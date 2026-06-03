# Context Compiler

The context compiler is deterministic and auditable. It accepts a `ContextCompilerInput`, classifies the current user message, and builds a bounded `ContextCompilerOutput`.

Inclusion behavior:

- Normal greetings such as `hello` do not include memory.
- Memory lookup and task-planning prompts can include relevant local memory search results.
- Workspace context is included only for workspace-like requests.
- Runtime context is included for relative-time, task, route, memory lookup, and tool-workflow intents.
- Location context is included only for route/location-like requests, only when permission/privacy allow it, and only when the context is fresh or a saved place.
- Privacy mode `off`, paused memory, or private chat prevents memory context inclusion.

The compiler emits:

- `intent`: labels, confidence, and reason.
- `memory_brief`: compact memory facts selected for the prompt.
- `raw_prompt`: final assembled prompt text.
- `included_memory_ids`: exact item IDs included.
- `included_graph_edge_ids`: graph edges connected to included memories.
- `runtime_decision` and `location_decision`: explicit include/skip reasons.
- `token_budget_report`: estimated prompt tokens and budget notes.
- `audit`: a persisted `prompt_audits` row.

This design keeps the Store themes-only and keeps prompt contributions controlled by Settings plus explicit compiler decisions.
