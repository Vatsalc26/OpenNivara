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

Do not mix tool approvals and memory proposals into the same DB table, same command namespace, or same UX concept. In particular, do not reuse operation approval `/approve` for memory proposals.

Implementation boundaries are defined in [Memory Tool Boundaries](memory-tool-boundaries.md). Removal semantics are defined in [Memory Retention Semantics](memory-retention-semantics.md).

## Memory Proposal UX

Desktop:

- show a non-blocking memory suggestion card after the assistant answer
- later add a `Memory -> Suggestions` inbox

Desktop card copy:

```text
Memory suggestion
OpenNivara suggests remembering:
"The user prefers concise answers."
[Save memory] [Dismiss] [Edit]
```

CLI:

- after final answer, print a short notification
- use a memory-specific command group

CLI copy:

```text
Memory suggestion created: "The user prefers concise answers."
Run `opennivara memory proposals list` to review.
```

CLI commands:

```text
opennivara memory proposals list
opennivara memory proposals show <proposal_id>
opennivara memory proposals approve <proposal_id>
opennivara memory proposals reject <proposal_id>
```

Telegram:

- do not reuse `/approve`
- use memory-specific commands

Telegram commands:

```text
/memory_proposals
/save_memory <proposal_id>
/reject_memory <proposal_id>
```

`/save_memory` approves a memory proposal. It does not approve an operation in `pending_approvals`.

## Proposal Payload

Standardize `proposal_json` with a versioned payload:

```rust
pub struct MemoryProposalPayload {
    pub schema_version: u32,
    pub proposed_memories: Vec<ProposedMemory>,
    pub proposed_tasks: Vec<ProposedTask>,
    pub ambiguities: Vec<String>,
    pub confidence: f64,
    pub reason: String,
}

pub struct ProposedMemory {
    pub memory_type: String,
    pub title: String,
    pub summary: String,
    pub details_json: serde_json::Value,
    pub sensitivity: String,
    pub visibility: String,
    pub confidence: f64,
    pub tags: Vec<String>,
}

pub struct ProposedTask {
    pub title: String,
    pub summary: Option<String>,
    pub status: String,
    pub due_at: Option<String>,
    pub natural_time_phrase: Option<String>,
    pub source_quote: Option<String>,
}
```

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
  "sensitivity": "normal",
  "tags": ["communication"],
  "source_message_id": "msg_123"
}
```

Behavior:

- `Off`: return `memory_disabled`
- `AskBeforeSaving`: create memory proposal
- `AutoSaveLowRisk`: auto-save if `sensitivity = normal` and `confidence >= 0.8`; otherwise create proposal
- `FullLifeJournal`: auto-save more broadly unless paused memory, private chat, or sensitive settings block it

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
  "visibility": "private",
  "confidence": 0.9,
  "tags": ["communication"]
}
```

Uses the same `MemoryMode` behavior as `remember_this`.

### update_memory

Edits an existing memory item and requires operation approval.

Example args:

```json
{
  "memory_id": "mem_...",
  "patch": {
    "summary": "The user now prefers detailed answers for architecture discussions.",
    "confidence": 0.95,
    "tags": ["communication", "architecture"]
  },
  "reason": "User corrected their preference."
}
```

### forget_memory

Retracts/stops using a memory and requires operation approval. Prefer this over hard deletion for normal "forget this" behavior.

Example args:

```json
{
  "memory_id": "mem_...",
  "reason": "User asked to forget this."
}
```

Query-based forgetting can come later.

Initial behavior: use `retract_memory_item`.

### delete_memory

Hard-deletes an exact memory record and always requires approval.

Example args:

```json
{
  "memory_id": "mem_...",
  "reason": "User requested permanent deletion."
}
```

Delay implementation until hard-delete semantics are explicit and implemented. `delete_memory` must not be exposed as a soft delete.

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

- `LocalModify`
- approval required

`delete_memory`:

- `LocalDelete`
- approval required
- exact-ID only
- unavailable until true hard delete is implemented

`AutoSaveLowRisk` and `FullLifeJournal` can allow some memory creation without same-turn tool approval because the user has configured memory behavior. Explicit update/forget/delete should require approval.

## Preview Examples

Memory proposal preview:

```json
{
  "schema_version": 1,
  "preview_kind": "memory_proposal",
  "operation_target": "proposal:prop_123",
  "summary": "OpenNivara suggests saving a memory.",
  "details": {
    "proposal_id": "prop_123",
    "title": "Prefers concise answers",
    "summary": "The user prefers concise answers.",
    "confidence": 0.9,
    "sensitivity": "normal"
  }
}
```

`remember_this` proposal-creation preview:

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
    "delete_mode": "retract"
  }
}
```

`delete_memory` preview:

```json
{
  "schema_version": 1,
  "preview_kind": "memory_delete",
  "operation_target": "mem_123",
  "summary": "OpenNivara wants to permanently delete a memory.",
  "details": {
    "memory_id": "mem_123",
    "title": "Prefers concise answers",
    "current_summary": "The user prefers concise answers.",
    "delete_mode": "hard_delete",
    "exact_id_only": true
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

`forget_memory` retracted:

```json
{
  "ok": true,
  "tool_name": "forget_memory",
  "tool_call_id": "toolcall_456",
  "summary": "Retracted memory: Prefers concise answers.",
  "result": {
    "status": "memory_retracted",
    "memory_id": "mem_123"
  },
  "error": null,
  "metadata": null
}
```

`delete_memory` unavailable:

```json
{
  "ok": false,
  "tool_name": "delete_memory",
  "tool_call_id": "toolcall_789",
  "summary": "delete_memory is not available yet.",
  "result": null,
  "error": {
    "code": "memory_hard_delete_not_implemented",
    "message": "Permanent memory deletion is not implemented yet. Use forget_memory to stop using the memory.",
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
6. `forget_memory` retracts/stops using memory.
7. `delete_memory` hard-deletes exact memory records and stays disabled until true hard delete exists.
8. Memory creation respects `MemoryMode`.
9. Memory update/forget/delete require approval.
10. Memory tools use `ToolPreview` and `ModelVisibleToolResult` like other tools.
11. Memory proposals remain separate from tool approvals.
12. Memory extraction runs only after a completed turn, not while approval is pending.
13. Desktop uses non-blocking memory suggestion cards and later a Memory Suggestions inbox.
14. CLI uses `opennivara memory proposals ...` commands.
15. Telegram uses `/save_memory` and `/reject_memory`, not `/approve`.
16. Memory domain logic lives in `memory::tools`; model-callable bridge logic lives in `tools::memory`.
17. Do not declare memory write tools when `MemoryMode` is off.

## Tests

Required tests:

1. Memory proposal creation does not create committed memory in `AskBeforeSaving`.
2. Approving proposal creates memory item/task.
3. Rejecting proposal does not create memory.
4. `remember_this` creates proposal in `AskBeforeSaving`.
5. `remember_this` auto-saves low-risk memory in `AutoSaveLowRisk`.
6. `remember_this` returns `memory_disabled` when memory mode is off.
7. `update_memory` requires approval.
8. `forget_memory` requires approval and uses retract/soft-stop behavior.
9. `delete_memory` is not declared until hard-delete implementation exists.
10. Memory extraction does not run while approval is pending.
11. Memory extraction runs after approved turn completes.
12. Memory extraction runs after denied turn explanation completes.
13. Memory extraction does not inspect compiled prompt or approval event JSON.
14. Memory source stores session/message/turn reference where available.
15. Memory proposal commands are separate from operation approval commands.
16. Desktop memory suggestion card does not block chat.
17. Telegram `/save_memory` approves memory proposal, not operation approval.
