# Memory Tool Boundaries

This document defines where explicit model-callable memory tools live and how they connect to memory proposals, tool previews, tool results, and operation approvals.

Memory proposals and operation approvals stay separate:

- memory proposal: non-blocking review item that suggests saving memory
- operation approval: same-turn gate that approves one mutating/deleting operation

Do not reuse `pending_approvals`, `pending_turns`, `/approve`, or the approval queue for memory proposals.

## Module Boundary

Use this dependency direction:

```text
engine
  -> tools registry
      -> tools::memory
          -> memory::tools
              -> memory::db
```

Avoid:

- `memory -> engine`
- `memory -> model provider`
- `memory -> approval state`
- `state -> memory`

## Domain Module

Memory domain logic lives in:

```text
src/memory/tools.rs
```

`src/memory/tools.rs` owns:

- memory-specific parameter structs
- memory policy decisions
- proposal creation
- autosave behavior
- update/forget/delete domain operations
- calls into `memory::db`

Suggested policy helper:

```rust
pub enum MemoryToolDecision {
    Disabled,
    CreateProposal,
    AutoSave,
    RequiresApproval,
}

pub fn decide_memory_create_behavior(
    settings: &MemorySettings,
    sensitivity: &str,
    confidence: f64,
) -> MemoryToolDecision
```

Decision behavior:

- `Off`: `Disabled`
- `AskBeforeSaving`: `CreateProposal`
- `AutoSaveLowRisk`: `AutoSave` if `sensitivity == "normal"` and `confidence >= 0.8`; otherwise `CreateProposal`
- `FullLifeJournal`: `AutoSave` unless paused memory, private chat, or sensitive settings block it

## Tool Bridge Module

Model-callable bridge code lives in:

```text
src/tools/memory.rs
```

`src/tools/memory.rs` owns:

- `ToolDefinition` schemas
- `OperationKind` mapping
- tool declaration gating
- `ToolPreview` building by delegating to `memory::tools`
- `ToolExecutionResult` and `ModelVisibleToolResult` conversion
- bridge registration with `ToolRegistry`

The bridge should not reimplement memory policy. It asks `memory::tools` what should happen, then presents the result through the shared tool system.

## Declaration Rule

Do not declare memory write tools when `MemoryMode` is `Off`.

When memory is off, avoid letting the model repeatedly call memory tools that will predictably return `memory_disabled`. If a direct user request says "remember this" while memory is off, the surface or engine can explain that memory is disabled.

Do not add memory tools to `tools.toml` until they are implemented.

Initial tool config after implementation:

```toml
[tools.remember_this]
enabled = true

[tools.create_memory]
enabled = true

[tools.update_memory]
enabled = true

[tools.forget_memory]
enabled = true
```

Delay:

```toml
[tools.delete_memory]
enabled = true
```

until true hard-delete behavior is implemented honestly.

## Implementation Order

1. Add `memory::tools` module with domain functions and schemas.
2. Add `tools::memory` bridge.
3. Implement `remember_this` and `create_memory` first.
4. Implement `update_memory` and `forget_memory` after approval flow exists.
5. Delay `delete_memory` until hard-delete behavior is explicit and implemented.

## Tests

Required tests:

1. `decide_memory_create_behavior` returns `Disabled` when memory mode is off.
2. `decide_memory_create_behavior` returns `CreateProposal` in `AskBeforeSaving`.
3. `decide_memory_create_behavior` returns `AutoSave` for normal high-confidence `AutoSaveLowRisk`.
4. sensitive `AutoSaveLowRisk` memory creates a proposal.
5. memory tools are not declared when `MemoryMode` is off.
6. `remember_this` and `create_memory` delegate proposal/autosave decisions to `memory::tools`.
7. `update_memory` and `forget_memory` are approval-required.
8. `delete_memory` is not declared until hard-delete is implemented.
9. `tools::memory` does not depend on provider-specific model code.
10. `memory::tools` does not depend on engine, provider, approval state, or surface modules.
