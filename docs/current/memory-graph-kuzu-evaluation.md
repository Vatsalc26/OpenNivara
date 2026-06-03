# Kuzu Evaluation

Status: not added.

Kuzu remains optional and is not a production dependency in this pass.

What was evaluated:

- The current graph requirements are satisfied by SQLite-derived graph tables.
- The graph must remain rebuildable from SQLite.
- Tauri packaging should not gain a new native graph dependency until there is a measured need.

Blocker:

- No Windows/Tauri/CI packaging proof has been completed for a Kuzu Rust dependency in this repo.

Decision:

- Keep SQLite graph index as default.
- Revisit Kuzu only behind a `memory-graph-kuzu` feature after Windows, Tauri, and CI builds are proven.
