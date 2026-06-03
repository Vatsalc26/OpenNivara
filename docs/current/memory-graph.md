# Memory Graph

The Memory Graph is a derived SQLite index. Source-of-truth data remains in normal memory tables.

Graph tables:

- `memory_graph_nodes`
- `memory_graph_edges`
- `memory_graph_edge_index`

The graph can be rebuilt from SQLite at any time. If graph rows are stale or corrupted, user memory remains safe because source rows are unchanged.

Current graph APIs support rebuild, status, validation, neighbors, memory context, entity context, related memories, entity mention search, shortest path, and manual edge insertion.

Kuzu is not a production dependency. It can be evaluated behind an optional feature later, but SQLite graph remains default.
