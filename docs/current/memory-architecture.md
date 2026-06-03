# Memory Architecture

OpenNivara memory is local-first. SQLite is the source of truth. FTS5, the graph index, and optional sqlite-vec/fastembed are derived retrieval aids.

Memory has no templates. There are no Student, Freelancer, Health, Travel, or occupation-specific schemas. Dynamic facets attach domain and facet type labels to memories without restricting shape.

Core tables include sources, memory items, facets, entities, relationships, corrections, tasks, saved places, location observations, graph nodes/edges, prompt audits, proposals, FTS, summaries, and rollups.

Privacy rules:

- Stored memory is not automatically prompt context.
- ContextCompiler must decide what to include.
- Sensitive or location memories obey Settings gates.
- Retracted and deleted memories are excluded from normal retrieval.

Corrections preserve old/new relationships and can update graph context through rebuild or upsert helpers.
