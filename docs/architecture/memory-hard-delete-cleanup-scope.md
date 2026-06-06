# Memory Hard-Delete Cleanup Scope

This document defines the exact cleanup scope required before OpenNivara can honestly expose `delete_memory`.

Locked distinction:

- `forget_memory` = retract/stop using memory
- `delete_memory` = true permanent deletion

Current delete-style memory behavior is soft-delete-like because it sets `deleted_at` and removes search/FTS visibility. That is not enough for `delete_memory`.

## Hard-Delete API

Add a new function later:

```rust
pub fn hard_delete_memory_item(
    conn: &mut Connection,
    memory_id: &str,
    reason: &str,
) -> anyhow::Result<MemoryHardDeleteResult>
```

It must run in one transaction.

Suggested result:

```rust
pub struct MemoryHardDeleteResult {
    pub memory_id: String,
    pub source_id: Option<String>,
    pub deleted_facets: u32,
    pub deleted_graph_edges: u32,
    pub deleted_graph_nodes: u32,
    pub deleted_tasks: u32,
    pub deleted_embeddings: u32,
    pub scrubbed_proposals: u32,
    pub scrubbed_corrections: u32,
    pub tombstone_id: String,
}
```

V1 restriction: `delete_memory` accepts exact `memory_id` only.

Do not support:

- delete by query
- delete by tag
- bulk hard delete
- delete all memories about a topic
- glob-like deletion

Bulk deletion needs a much stronger preview showing every affected item and derived object.

## Table Cleanup Map

`memory_items`:

- delete the row
- before deletion, load `memory_id`, `source_id`, title, and summary only for approval preview
- do not retain title or summary in tombstone

`memory_fts`:

- `DELETE FROM memory_fts WHERE memory_id = ?`
- mandatory because FTS stores searchable content copies

`tasks`:

- `DELETE FROM tasks WHERE memory_id = ?`
- tasks derived from the deleted memory must be removed

`memory_facets`:

- `DELETE FROM memory_facets WHERE memory_id = ?`
- facets may contain memory-derived content in labels or `details_json`

`memory_entities`:

- delete only the memory/entity link
- do not delete entity rows automatically because other memories may use them

`memory_graph_edges`:

- `DELETE FROM memory_graph_edges WHERE source_memory_id = ?`
- edges derived from the deleted memory should not remain

`memory_graph_edge_index`:

- rebuild or clear after deleting graph edges
- for v1, clearing/rebuilding the derived index is acceptable

`memory_graph_nodes`:

```sql
DELETE FROM memory_graph_nodes
WHERE source_table = 'memory_items'
  AND source_id = ?;
```

Do not delete unrelated entity/source nodes.

`memory_embeddings`:

- if memory-vector feature is enabled, `DELETE FROM memory_embeddings WHERE memory_id = ?`
- mandatory because embedding rows may store plaintext text
- gate with `cfg(feature = "memory-vector")` if needed

`memory_corrections`:

- do not leave foreign keys pointing to a hard-deleted memory
- `old_memory_id` and `new_memory_id` are nullable, so scrub references instead of keeping broken links

Suggested behavior:

```sql
UPDATE memory_corrections
SET old_memory_id = CASE WHEN old_memory_id = ? THEN NULL ELSE old_memory_id END,
    new_memory_id = CASE WHEN new_memory_id = ? THEN NULL ELSE new_memory_id END,
    reason = '[scrubbed: memory hard-deleted]'
WHERE old_memory_id = ? OR new_memory_id = ?;
```

`memory_sources`:

- source rows may contain `source_text` and `source_quote`
- a source may be shared by multiple memories
- if no other non-deleted `memory_items` reference this `source_id`, scrub `source_text` and `source_quote`
- otherwise leave the source row unchanged

Suggested scrub:

```sql
UPDATE memory_sources
SET source_text = '[scrubbed: source memory hard-deleted]',
    source_quote = NULL
WHERE id = ?;
```

Do not delete `memory_sources` rows if proposals still reference them. Scrubbing is safer than deleting.

`memory_proposals`:

- `proposal_json` may contain the same memory content
- scrub proposals tied to a scrubbed source

Suggested behavior:

```sql
UPDATE memory_proposals
SET proposal_json = '{"schema_version":1,"scrubbed":true,"reason":"source memory hard-deleted"}',
    status = 'scrubbed',
    updated_at = ?
WHERE source_id = ?;
```

If the source is shared by other active memory items, leave proposals unchanged unless a more precise content match exists.

`rollups`:

- rollups contain derived summaries and `source_memory_ids_json`
- if `source_memory_ids_json` contains `memory_id`, delete or mark stale
- recommendation: delete affected rollups because they are derived and can be regenerated

`entities`:

- leave entity rows alone
- entities may be referenced by other memories

`entity_aliases`:

- if `source_id` equals the scrubbed source ID, set `source_id = NULL`
- do not delete alias text unless provenance proves it only came from this memory

`entity_relationships`:

- if `source_id` equals the scrubbed source ID, set `source_id = NULL`
- do not delete relationship rows unless provenance proves they only came from this memory

`saved_places`:

- leave unchanged unless a direct memory/source link is added later

`location_observations`:

- leave unchanged unless a direct memory/source link is added later

## Unresolved Historical Content

`prompt_audits` may contain user messages, compiled context JSON, included memory IDs, and historical compiled prompt content. Per-memory scrubbing is difficult because `compiled_context_json` may contain free-form memory content.

Decision: do not enable `delete_memory` until one of these is true:

- prompt-audit scrub strategy is implemented
- product policy clearly states prompt audits are historical audit records outside per-memory deletion scope

Recommended target:

- V1: remove memory ID references from `included_memory_ids_json` if easy
- V2: scrub `compiled_context_json` sections containing the deleted memory

`session_summaries` and memory-candidate JSON may contain free-text content. Per-memory scrubbing is also difficult.

Decision: do not enable `delete_memory` until one of these is true:

- session summary scrub/regeneration strategy is implemented
- product policy clearly states historical conversation summaries are outside per-memory deletion scope

If strict permanent deletion is required, delay `delete_memory` until summaries can be scrubbed or regenerated.

## Tombstone Table

Add a tombstone table later:

```sql
CREATE TABLE IF NOT EXISTS memory_deletion_tombstones (
    id TEXT PRIMARY KEY,
    memory_id TEXT NOT NULL,
    delete_mode TEXT NOT NULL,
    reason TEXT NOT NULL,
    deleted_at TEXT NOT NULL,
    deleted_by_actor_id TEXT NULL,
    session_id TEXT NULL,
    source_id TEXT NULL
);
```

Allowed tombstone fields:

- tombstone ID
- memory ID
- delete mode
- reason
- deleted timestamp
- deleting actor ID
- session ID
- source ID

Tombstone must not contain:

- original title
- original summary
- `details_json`
- source quote
- source text
- embedding text
- sensitive content
- full proposal JSON
- full compiled prompt

## Implementation Phases

Phase 1:

- implement `forget_memory` only, using `retract_memory_item`

Phase 2:

- add hard-delete migration with `memory_deletion_tombstones`
- add needed indexes for memory/source references

Phase 3:

- implement `hard_delete_memory_item` for structured tables and derived indexes

Phase 4:

- decide prompt-audit/session-summary scrub policy

Phase 5:

- expose `delete_memory` only after Phase 4 is complete

## Critical Honesty Rule

Do not expose `delete_memory` until OpenNivara either:

1. removes or scrubs all direct content copies, including historical prompt/session summary content, or
2. clearly documents that certain historical audit records are outside per-memory deletion scope.

## Tests

Required tests:

1. hard delete removes `memory_items` row.
2. hard delete removes `memory_fts` row.
3. hard delete removes tasks row.
4. hard delete removes `memory_facets`.
5. hard delete removes `memory_entities` links.
6. hard delete removes graph edges sourced from memory.
7. hard delete removes graph nodes sourced from memory item.
8. hard delete rebuilds or clears graph edge index.
9. hard delete removes `memory_embeddings` when feature enabled.
10. hard delete nulls correction references and scrubs reason.
11. hard delete scrubs memory sources only when no other active memory uses the source.
12. hard delete does not scrub a shared source still used by another memory.
13. hard delete scrubs proposals tied to a scrubbed source.
14. hard delete deletes affected rollups.
15. tombstone contains no original title, summary, details, or source text.
16. `delete_memory` is not declared while prompt-audit/session-summary scope is unresolved.
17. `forget_memory` remains available and uses retraction.
18. `delete_memory` rejects non-exact-ID requests.
