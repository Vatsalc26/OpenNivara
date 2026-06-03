# ContextCompiler Main Chat Unification

Current state:

- `OpenNivaraEngine::preview_context_for_message()` still builds the main chat prompt context manually.
- `memory::compiler::compile_context()` already owns richer context assembly for runtime, location, memory, graph context, token budgeting, and audit output.
- Skills v1 were added to the older engine preview path in commit `7f2d24015907fcac430797d26425214aecc6d25c`.

Next step:

- Make ContextCompiler a superset of the old preview builder before routing main chat through it.
- Keep the existing preview API for now, but have it call the compiler internally once the compiler can emit every field the UI and engine currently need.
- Route both `preview_context_for_message()` and `handle_message()` through the same compiler output so prompt assembly, selected skills, warnings, token budget decisions, and audit records share one source of truth.

Regression requirements for the unification patch:

- Preserve profile privacy behavior.
- Preserve Settings style behavior.
- Keep Store themes UI-only.
- Preserve triggered preferences, triggered contexts, pinned contexts, and workspace map hints.
- Include Skills as a compiler section.
- Apply token budgeting across all prompt sections.
- Emit audit information for main chat prompts.
