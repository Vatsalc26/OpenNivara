use rusqlite::Connection;

pub const SCHEMA_VERSION: u32 = 2;

pub fn run_migrations(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS memory_sources (
            id TEXT PRIMARY KEY,
            source_type TEXT NOT NULL,
            source_ref TEXT NULL,
            source_text TEXT NOT NULL,
            source_quote TEXT NULL,
            session_id TEXT NULL,
            message_id TEXT NULL,
            created_at TEXT NOT NULL,
            observed_at TEXT NOT NULL,
            timezone TEXT NOT NULL,
            sensitivity TEXT NOT NULL,
            privacy_scope TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS memory_items (
            id TEXT PRIMARY KEY,
            memory_type TEXT NOT NULL,
            title TEXT NOT NULL,
            summary TEXT NOT NULL,
            details_json TEXT NOT NULL,
            status TEXT NOT NULL,
            confidence REAL NOT NULL,
            user_verified INTEGER NOT NULL DEFAULT 0,
            sensitivity TEXT NOT NULL,
            visibility TEXT NOT NULL,
            source_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            observed_at TEXT NOT NULL,
            valid_from TEXT NULL,
            valid_until TEXT NULL,
            happened_at TEXT NULL,
            starts_at TEXT NULL,
            ends_at TEXT NULL,
            due_at TEXT NULL,
            completed_at TEXT NULL,
            timezone TEXT NOT NULL,
            time_precision TEXT NOT NULL,
            natural_time_phrase TEXT NULL,
            recurrence_rule TEXT NULL,
            superseded_by TEXT NULL,
            deleted_at TEXT NULL,
            FOREIGN KEY(source_id) REFERENCES memory_sources(id)
        );

        CREATE TABLE IF NOT EXISTS entities (
            id TEXT PRIMARY KEY,
            entity_type TEXT NOT NULL,
            canonical_name TEXT NOT NULL,
            display_name TEXT NOT NULL,
            details_json TEXT NOT NULL,
            confidence REAL NOT NULL,
            user_verified INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT NULL
        );

        CREATE TABLE IF NOT EXISTS entity_aliases (
            id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL,
            alias TEXT NOT NULL,
            source_id TEXT NULL,
            confidence REAL NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(entity_id) REFERENCES entities(id),
            FOREIGN KEY(source_id) REFERENCES memory_sources(id)
        );

        CREATE TABLE IF NOT EXISTS entity_relationships (
            id TEXT PRIMARY KEY,
            from_entity_id TEXT NOT NULL,
            relationship_type TEXT NOT NULL,
            to_entity_id TEXT NOT NULL,
            details_json TEXT NOT NULL,
            confidence REAL NOT NULL,
            valid_from TEXT NULL,
            valid_until TEXT NULL,
            source_id TEXT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            superseded_by TEXT NULL,
            FOREIGN KEY(from_entity_id) REFERENCES entities(id),
            FOREIGN KEY(to_entity_id) REFERENCES entities(id),
            FOREIGN KEY(source_id) REFERENCES memory_sources(id)
        );

        CREATE TABLE IF NOT EXISTS memory_entities (
            memory_id TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            role TEXT NOT NULL,
            confidence REAL NOT NULL,
            resolution_status TEXT NOT NULL,
            PRIMARY KEY(memory_id, entity_id, role),
            FOREIGN KEY(memory_id) REFERENCES memory_items(id),
            FOREIGN KEY(entity_id) REFERENCES entities(id)
        );

        CREATE TABLE IF NOT EXISTS memory_corrections (
            id TEXT PRIMARY KEY,
            correction_source_id TEXT NOT NULL,
            old_memory_id TEXT NULL,
            new_memory_id TEXT NULL,
            correction_type TEXT NOT NULL,
            reason TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(correction_source_id) REFERENCES memory_sources(id),
            FOREIGN KEY(old_memory_id) REFERENCES memory_items(id),
            FOREIGN KEY(new_memory_id) REFERENCES memory_items(id)
        );

        CREATE TABLE IF NOT EXISTS tasks (
            memory_id TEXT PRIMARY KEY,
            task_type TEXT NOT NULL,
            priority INTEGER NOT NULL,
            status TEXT NOT NULL,
            due_at TEXT NULL,
            reminder_at TEXT NULL,
            completed_at TEXT NULL,
            checklist_json TEXT NOT NULL,
            FOREIGN KEY(memory_id) REFERENCES memory_items(id)
        );

        CREATE TABLE IF NOT EXISTS session_summaries (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            summary TEXT NOT NULL,
            open_questions_json TEXT NOT NULL,
            decisions_json TEXT NOT NULL,
            tasks_json TEXT NOT NULL,
            memory_candidates_json TEXT NOT NULL,
            covered_message_start TEXT NULL,
            covered_message_end TEXT NULL,
            token_estimate INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS rollups (
            id TEXT PRIMARY KEY,
            rollup_type TEXT NOT NULL,
            subject_entity_id TEXT NULL,
            period_start TEXT NULL,
            period_end TEXT NULL,
            timezone TEXT NOT NULL,
            summary TEXT NOT NULL,
            source_memory_ids_json TEXT NOT NULL,
            confidence REAL NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(subject_entity_id) REFERENCES entities(id)
        );

        CREATE TABLE IF NOT EXISTS prompt_audits (
            id TEXT PRIMARY KEY,
            session_id TEXT NULL,
            message_id TEXT NULL,
            user_message TEXT NOT NULL,
            compiled_context_json TEXT NOT NULL,
            included_memory_ids_json TEXT NOT NULL,
            included_task_ids_json TEXT NOT NULL,
            included_workspace_refs_json TEXT NOT NULL,
            token_budget_json TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS memory_proposals (
            id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL,
            proposal_json TEXT NOT NULL,
            sensitivity TEXT NOT NULL,
            confidence REAL NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(source_id) REFERENCES memory_sources(id)
        );

        CREATE TABLE IF NOT EXISTS memory_settings (
            singleton_id INTEGER PRIMARY KEY CHECK (singleton_id = 1),
            settings_json TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS memory_facets (
            id TEXT PRIMARY KEY,
            memory_id TEXT NOT NULL,
            domain TEXT NOT NULL,
            facet_type TEXT NOT NULL,
            label TEXT NOT NULL,
            details_json TEXT NOT NULL,
            sensitivity TEXT NOT NULL,
            confidence REAL NOT NULL,
            source TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(memory_id) REFERENCES memory_items(id)
        );

        CREATE TABLE IF NOT EXISTS memory_graph_nodes (
            id TEXT PRIMARY KEY,
            node_type TEXT NOT NULL,
            source_table TEXT NOT NULL,
            source_id TEXT NOT NULL,
            label TEXT NOT NULL,
            properties_json TEXT NOT NULL,
            sensitivity TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS memory_graph_edges (
            id TEXT PRIMARY KEY,
            from_node_id TEXT NOT NULL,
            edge_type TEXT NOT NULL,
            to_node_id TEXT NOT NULL,
            weight REAL NOT NULL,
            confidence REAL NOT NULL,
            source_memory_id TEXT NULL,
            properties_json TEXT NOT NULL,
            valid_from TEXT NULL,
            valid_until TEXT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS memory_graph_edge_index (
            from_node_id TEXT NOT NULL,
            to_node_id TEXT NOT NULL,
            min_depth INTEGER NOT NULL,
            path_count INTEGER NOT NULL,
            strongest_weight REAL NOT NULL,
            updated_at TEXT NOT NULL,
            PRIMARY KEY(from_node_id, to_node_id)
        );

        CREATE TABLE IF NOT EXISTS saved_places (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            place_type TEXT NOT NULL,
            latitude REAL NULL,
            longitude REAL NULL,
            address TEXT NULL,
            city TEXT NULL,
            region TEXT NULL,
            country TEXT NULL,
            timezone TEXT NULL,
            details_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT NULL
        );

        CREATE TABLE IF NOT EXISTS location_observations (
            id TEXT PRIMARY KEY,
            status TEXT NOT NULL,
            latitude REAL NULL,
            longitude REAL NULL,
            accuracy_meters REAL NULL,
            source TEXT NOT NULL,
            captured_at TEXT NOT NULL,
            freshness_seconds INTEGER NULL,
            label TEXT NULL,
            details_json TEXT NOT NULL,
            privacy_level TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
            memory_id UNINDEXED,
            title,
            summary,
            source_text,
            entity_names,
            tags
        );

        CREATE INDEX IF NOT EXISTS idx_memory_items_memory_type ON memory_items(memory_type);
        CREATE INDEX IF NOT EXISTS idx_memory_items_status ON memory_items(status);
        CREATE INDEX IF NOT EXISTS idx_memory_items_happened_at ON memory_items(happened_at);
        CREATE INDEX IF NOT EXISTS idx_memory_items_valid_range ON memory_items(valid_from, valid_until);
        CREATE INDEX IF NOT EXISTS idx_memory_items_due_at ON memory_items(due_at);
        CREATE INDEX IF NOT EXISTS idx_memory_items_confidence ON memory_items(confidence);
        CREATE INDEX IF NOT EXISTS idx_memory_items_user_verified ON memory_items(user_verified);
        CREATE INDEX IF NOT EXISTS idx_entities_entity_type ON entities(entity_type);
        CREATE INDEX IF NOT EXISTS idx_entity_aliases_alias ON entity_aliases(alias);
        CREATE INDEX IF NOT EXISTS idx_memory_entities_entity_id ON memory_entities(entity_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_status_due_at ON tasks(status, due_at);
        CREATE INDEX IF NOT EXISTS idx_memory_facets_memory_id ON memory_facets(memory_id);
        CREATE INDEX IF NOT EXISTS idx_memory_facets_domain ON memory_facets(domain);
        CREATE INDEX IF NOT EXISTS idx_memory_facets_facet_type ON memory_facets(facet_type);
        CREATE INDEX IF NOT EXISTS idx_memory_facets_label ON memory_facets(label);
        CREATE INDEX IF NOT EXISTS idx_memory_facets_sensitivity ON memory_facets(sensitivity);
        CREATE INDEX IF NOT EXISTS idx_memory_facets_confidence ON memory_facets(confidence);
        CREATE INDEX IF NOT EXISTS idx_graph_nodes_node_type ON memory_graph_nodes(node_type);
        CREATE INDEX IF NOT EXISTS idx_graph_nodes_source ON memory_graph_nodes(source_table, source_id);
        CREATE INDEX IF NOT EXISTS idx_graph_edges_from ON memory_graph_edges(from_node_id);
        CREATE INDEX IF NOT EXISTS idx_graph_edges_to ON memory_graph_edges(to_node_id);
        CREATE INDEX IF NOT EXISTS idx_graph_edges_type ON memory_graph_edges(edge_type);
        CREATE INDEX IF NOT EXISTS idx_graph_edges_confidence ON memory_graph_edges(confidence);
        CREATE INDEX IF NOT EXISTS idx_graph_edges_source_memory ON memory_graph_edges(source_memory_id);
        "#,
    )?;

    #[cfg(feature = "memory-vector")]
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS memory_embeddings (
            memory_id TEXT NOT NULL,
            chunk_id TEXT NOT NULL,
            embedding BLOB NOT NULL,
            text TEXT NOT NULL,
            created_at TEXT NOT NULL,
            PRIMARY KEY(memory_id, chunk_id),
            FOREIGN KEY(memory_id) REFERENCES memory_items(id)
        );
        "#,
    )?;

    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}
