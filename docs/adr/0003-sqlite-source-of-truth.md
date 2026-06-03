# ADR 0003: SQLite Source Of Truth

Status: accepted.

OpenNivara memory source of truth is local SQLite. FTS5 and optional sqlite-vec/fastembed can support retrieval, and the graph index can be rebuilt from SQLite.

Cloud memory services, Neo4j, Postgres, DuckDB, external vector databases, LangChain, and LlamaIndex are not core runtime dependencies.
