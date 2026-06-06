# Memory Retention Semantics

OpenNivara uses three memory removal levels:

1. retract
2. soft delete
3. hard delete

Be honest in user-facing tool names:

- `forget_memory` means stop using this memory.
- `delete_memory` means permanently erase this memory.

Do not implement `delete_memory` as a soft delete while calling it permanent deletion.

## Level 1: Retract

Use retract when the memory is wrong, outdated, corrected, or no longer applies.

Examples:

- "That memory is wrong."
- "That no longer applies."
- "Forget that preference; it changed."

Effect:

- memory row remains
- status becomes `retracted`
- search/FTS entry is removed
- correction/audit row records why
- memory no longer appears in retrieval

Tool:

- `forget_memory`

Recommendation: `forget_memory` should use `retract_memory_item` initially.

Reason: retraction preserves correction history and helps prevent the system from relearning or reusing the same wrong fact.

## Level 2: Soft Delete

Use soft delete when the user wants the memory removed from active use but not necessarily physically erased.

Examples:

- "Remove this memory from active use."
- "I do not want this showing up anymore."

Effect:

- memory row remains
- `deleted_at` is set
- search/FTS entry is removed
- memory is not used in retrieval
- memory is not shown in normal memory lists
- memory may appear in a deleted/history/admin view

Do not expose soft delete as `delete_memory` if users would expect permanent erasure.

Recommendation: use `forget_memory`/retract for normal "forget this" behavior. Do not create a user-facing `soft_delete_memory` tool in v1 unless there is a clear UX need.

## Level 3: Hard Delete

Use hard delete when the user wants permanent erasure.

Examples:

- "Permanently delete this memory."
- "Remove it from the database."
- "Erase this stored personal data."

Effect:

- memory content is physically removed or scrubbed from all direct storage locations
- FTS/search rows are removed
- related facets, edges, tasks, and corrections are cleaned up or scrubbed
- proposal/source references containing the content are removed or scrubbed where feasible
- a minimal content-free tombstone may remain

Tool:

- `delete_memory`

Decision: `delete_memory` means true hard delete, exact memory ID only, always approval-required.

Do not expose `delete_memory` until hard-delete cleanup semantics are fully implemented and documented. The cleanup scope is defined in [Memory Hard-Delete Cleanup Scope](memory-hard-delete-cleanup-scope.md).

## Hard-Delete Preview

Before hard delete, preview must show:

- exact `memory_id`
- title
- summary
- sensitivity
- created/updated timestamps if useful
- linked derived objects if known
- clear warning that deletion is permanent

Example:

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

`forget_memory`:

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

`delete_memory` after true hard delete:

```json
{
  "ok": true,
  "tool_name": "delete_memory",
  "tool_call_id": "toolcall_789",
  "summary": "Permanently deleted memory mem_123.",
  "result": {
    "status": "memory_hard_deleted",
    "memory_id": "mem_123"
  },
  "error": null,
  "metadata": null
}
```

If hard delete is not implemented:

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

1. Use three removal levels: retract, soft delete, hard delete.
2. `forget_memory` means retract/stop using memory.
3. `delete_memory` means true permanent deletion.
4. `delete_memory` must be exact-ID only.
5. `delete_memory` always requires approval.
6. Do not expose `delete_memory` until hard-delete cleanup is implemented.
7. Do not implement `delete_memory` as soft delete.
8. Hard delete removes or scrubs memory content everywhere directly stored.
9. A minimal tombstone without content may remain.
10. No bulk/query/tag hard delete in v1.

## Tests

Required tests:

1. `forget_memory` marks memory retracted and removes it from retrieval.
2. `forget_memory` preserves correction/audit context.
3. soft-deleted memories do not appear in normal retrieval.
4. `delete_memory` is not declared until hard-delete is implemented.
5. hard delete removes memory item content.
6. hard delete removes FTS/search entry.
7. hard delete removes or scrubs facets linked to memory.
8. hard delete removes or scrubs graph links derived from memory.
9. hard delete removes or scrubs task links if derived from memory.
10. hard delete creates content-free tombstone.
11. tombstone does not contain title, summary, details, or source quote.
12. `delete_memory` rejects query, bulk, and tag deletion.
13. `delete_memory` approval preview shows exact target.
14. `delete_memory` model-visible result uses `memory_hard_deleted`.
