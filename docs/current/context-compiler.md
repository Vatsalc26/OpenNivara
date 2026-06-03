# Context Compiler

The ContextCompiler is deterministic and audited. It receives the user message, RuntimeContext, privacy mode, available source hints, and context budget.

Routing labels include normal chat, memory lookup, correction, task planning, route, location, workspace, tool workflow, email, file, app control, settings, and sensitive-private intents.

Prompt sections are included only when relevant:

- Runtime: relative date, task, route, memory, or tool workflow prompts.
- Location: route/location/local planning prompts when permission and freshness allow.
- Memory: memory lookup, task, correction, route, or tool workflow prompts.
- Workspace: workspace/file/code prompts.
- Graph facts: only when connected to included memories and useful.

Audit output records included IDs, graph edges, runtime decision, location decision, skipped/trimmed sections, and token budget.
