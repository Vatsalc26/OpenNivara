# Memory Architecture

OpenNivara memory is local-first. The source of truth is `opennivara_memory.sqlite` under the normal config directory, opened through `src/memory/db.rs`. Migrations live in `src/memory/migrations.rs` and create sources, items, facets, graph nodes/edges, saved places, location observations, entities, corrections, tasks, summaries, rollups, prompt audits, proposals, and an FTS5 index.

The core rule is separation of storage from prompt inclusion. Stored memory is not automatically prompt context. `src/memory/compiler.rs` classifies the user message, retrieves relevant memories only for memory/task/correction/route/location intents, applies runtime/location relevance gates, then writes a prompt audit explaining what was included.

Main modules:

- `db`: SQLite open, migration, settings, CRUD, validation, repair.
- `retrieval`: FTS and structured search with answerability labels such as `planned_only` and `confirmed`.
- `facets`: dynamic domain labels attached to memory items. Facets are free-form rows, not templates.
- `graph`: a SQLite-derived graph index over memory rows, facets, entities, tasks, and sources.
- `compiler`: deterministic context assembly and prompt audit creation.
- `privacy`: memory inclusion/saving gates.
- `extraction`: proposal creation and approval flow.
- `entities`, `tasks`, `reminders`, `corrections`, `maps`: operational memory surfaces.
- `prompt/context_compiler.rs`: stable re-export path for prompt compiler integration.
- `runtime`: clock and location context services used by the compiler.

The graph is derived from SQLite, so rebuilding it never changes the source-of-truth memory rows. No cloud memory, template packs, external graph database, or vector database is required for the default path.

Optional features:

- `memory-vector`: enables the vector-table migration path.
- `local-embeddings`: enables the local embedding feature flag and depends on `memory-vector`.

The desktop app exposes memory through Tauri commands in `desktop/src-tauri/src/lib.rs` and a typed frontend client in `desktop/src/api/memoryClient.ts`.
