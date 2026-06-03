# Memory Graph

The Memory Graph is a local SQLite-derived index. It stores graph rows in `memory_graph_nodes`, `memory_graph_edges`, and `memory_graph_edge_index`, but source-of-truth data remains in the normal memory tables.

Indexed node sources:

- `memory_items`
- `memory_sources`
- `memory_facets`
- `entities`
- `tasks`

The graph rebuild command clears graph rows and derives them again from SQLite. This keeps graph repair deterministic and avoids hidden state. Current edges include memory-to-source, memory-to-facet, memory-to-entity, and task-to-memory relationships.

Kuzu was not added in this pass. The default architecture does not need an external graph engine yet, and keeping the graph inside SQLite avoids another native dependency, migration surface, and desktop packaging risk. If graph traversal becomes too slow or too expressive for SQLite, evaluate Kuzu behind an optional feature with clear desktop build gates.
