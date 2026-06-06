# Memory Proposals And Tools

OpenNivara already has a memory subsystem with memory settings, memory modes, memory sources, memory proposals, memory items/tasks, and proposal approval/rejection flow.

Keep memory proposals and tool approvals distinct.

## Mental Model

A memory proposal is a draft/suggested memory, not committed long-term memory yet.

Simple flow:

1. Chat message.
2. Memory proposal.
3. Saved memory, only if approved or auto-saved by memory mode.

Proposal means: "I think this might be worth remembering. Should I save it?"

Tool approval:

- blocks the same agent turn
- used for operations like `write_file`, `delete_file`, `run_command`, `update_memory`, and `delete_memory`
- approval means "execute this operation once"

Memory proposal:

- does not need to block the same turn
- used for long-term memory review
- approval means "save this proposed memory"

Do not mix tool approvals and memory proposals into the same DB table or same UX concept.

## Memory Modes

Off:

- no extraction
- no proposals
- no saved memories

AskBeforeSaving:

- extraction can create proposals
- user must approve before memory items/tasks are created

AutoSaveLowRisk:

- low-risk/high-confidence memories may be auto-saved
- sensitive/uncertain memories become proposals

FullLifeJournal:

- broader automatic memory saving
- still respects paused memory, private chat, and sensitive settings

Default recommendation: keep `AskBeforeSaving` for now. The rest of the agent is becoming liberal, but long-term memory writes should avoid surprising users during alpha.

## Timing Rule

Run memory extraction only after a turn has a final assistant answer/explanation.

Normal answer:

```text
handle_message -> final assistant answer stored -> memory extraction runs
```

Approval required:

```text
handle_message -> approval_required returned -> do not run final memory extraction yet
```

Approved and completed:

```text
approve -> tool executes -> final assistant answer stored -> memory extraction runs
```

Denied and explanation completed:

```text
deny -> final denial explanation stored -> memory extraction runs
```

Provider continuation failed:

- do not extract yet, because there is no final assistant answer.

Extraction should inspect:

- raw user message
- final assistant answer
- optional outcome summaries

Extraction should not inspect:

- compiled prompt
- hidden context
- hidden memory/profile sections
- full tool outputs
- pending-turn JSON
- approval event JSON

Memory should learn from the user-facing completed exchange, not hidden prompt scaffolding.

## DB Boundary

Keep memory DB separate. Do not merge memory DB into state DB during this architecture change.

Improve cross-links by populating memory source fields:

- session ID
- message ID
- source ref or turn-ID-style reference when possible

## Explicit Memory Tools

Add first-class model-callable memory tools:

- `remember_this`
- `create_memory`
- `update_memory`
- `forget_memory`
- `delete_memory`

These tools are separate from automatic extraction.

### remember_this

Natural-language memory creation. The model uses this when the user says "remember this".

Creates a proposal or saves depending on `MemoryMode`.

Example args:

```json
{
  "content": "The user prefers concise answers.",
  "memory_type": "preference",
  "confidence": 0.9,
  "source_message_id": "msg_..."
}
```

### create_memory

Structured memory creation.

Example args:

```json
{
  "memory_type": "preference",
  "title": "Prefers concise answers",
  "summary": "The user prefers concise answers.",
  "details_json": {
    "preference": "concise answers"
  },
  "sensitivity": "normal",
  "confidence": 0.9
}
```

### update_memory

Edits an existing memory item and requires operation approval.

Example args:

```json
{
  "memory_id": "mem_...",
  "patch": {
    "summary": "The user now prefers detailed answers for architecture discussions.",
    "confidence": 0.95
  },
  "reason": "User corrected their preference."
}
```

### forget_memory

Soft-deletes/suppresses memory and requires operation approval. Prefer this over hard deletion for normal "forget this" behavior.

Example args:

```json
{
  "memory_id": "mem_...",
  "reason": "User asked to forget this."
}
```

Query-based forgetting can come later.

### delete_memory

Hard-deletes an exact memory record and always requires approval.

Example args:

```json
{
  "memory_id": "mem_...",
  "reason": "User requested permanent deletion."
}
```

## Classification

`remember_this`:

- `LocalModify`
- approval/proposal behavior governed by `MemoryMode`

`create_memory`:

- `LocalModify`
- approval/proposal behavior governed by `MemoryMode`

`update_memory`:

- `LocalModify`
- approval required

`forget_memory`:

- `LocalModify` or `LocalDelete` depending implementation
- approval required

`delete_memory`:

- `LocalDelete`
- approval required

`AutoSaveLowRisk` and `FullLifeJournal` can allow some memory creation without same-turn tool approval because the user has configured memory behavior. Explicit update/forget/delete should require approval.

## Preview Examples

`remember_this` preview:

```json
{
  "schema_version": 1,
  "tool_name": "remember_this",
  "preview_kind": "memory_create",
  "operation_target": "memory:new",
  "summary": "OpenNivara wants to remember: The user prefers concise answers.",
  "details": {
    "memory_type": "preference",
    "title": "Prefers concise answers",
    "summary": "The user prefers concise answers.",
    "sensitivity": "normal",
    "confidence": 0.9,
    "mode": "ask_before_saving"
  }
}
```

`forget_memory` preview:

```json
{
  "schema_version": 1,
  "tool_name": "forget_memory",
  "preview_kind": "memory_forget",
  "operation_target": "mem_123",
  "summary": "OpenNivara wants to forget a stored memory.",
  "details": {
    "memory_id": "mem_123",
    "title": "Prefers concise answers",
    "current_summary": "The user prefers concise answers.",
    "delete_mode": "soft_delete"
  }
}
```

## Model-Visible Results

`remember_this` creates proposal:

```json
{
  "ok": true,
  "tool_name": "remember_this",
  "tool_call_id": "toolcall_123",
  "summary": "Created a memory proposal.",
  "result": {
    "proposal_id": "prop_123",
    "status": "pending",
    "memory_type": "preference"
  },
  "error": null,
  "metadata": null
}
```

`remember_this` auto-saves:

```json
{
  "ok": true,
  "tool_name": "remember_this",
  "tool_call_id": "toolcall_123",
  "summary": "Saved memory: Prefers concise answers.",
  "result": {
    "memory_id": "mem_123",
    "status": "saved"
  },
  "error": null,
  "metadata": null
}
```

Memory disabled:

```json
{
  "ok": false,
  "tool_name": "remember_this",
  "tool_call_id": "toolcall_123",
  "summary": "Memory is disabled.",
  "result": null,
  "error": {
    "code": "memory_disabled",
    "message": "Memory is currently disabled, so OpenNivara did not save this.",
    "recoverable": true
  },
  "metadata": null
}
```

## Locked Decisions

1. Keep automatic extraction/proposal system.
2. Add explicit model-callable memory tools.
3. `remember_this` is natural-language memory creation.
4. `create_memory` is structured memory creation.
5. `update_memory` edits existing memory.
6. `forget_memory` soft-deletes/suppresses memory.
7. `delete_memory` hard-deletes exact memory records.
8. Memory creation respects `MemoryMode`.
9. Memory update/forget/delete require approval.
10. Memory tools use `ToolPreview` and `ModelVisibleToolResult` like other tools.
11. Memory proposals remain separate from tool approvals.
12. Memory extraction runs only after a completed turn, not while approval is pending.

## Tests

Required tests:

1. Memory proposal creation does not create committed memory in `AskBeforeSaving`.
2. Approving proposal creates memory item/task.
3. Rejecting proposal does not create memory.
4. `remember_this` creates proposal in `AskBeforeSaving`.
5. `remember_this` auto-saves low-risk memory in `AutoSaveLowRisk`.
6. `remember_this` returns `memory_disabled` when memory mode is off.
7. `update_memory` requires approval.
8. `forget_memory` requires approval and soft-deletes/suppresses memory.
9. `delete_memory` requires approval and hard-deletes exact memory ID.
10. Memory extraction does not run while approval is pending.
11. Memory extraction runs after approved turn completes.
12. Memory extraction runs after denied turn explanation completes.
13. Memory extraction does not inspect compiled prompt or approval event JSON.
14. Memory source stores session/message/turn reference where available.
