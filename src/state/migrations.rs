use rusqlite::Connection;

mod embedded {
    refinery::embed_migrations!("src/state/migrations");
}

pub const STATE_MIGRATION_COUNT: usize = 2;

pub fn run_migrations(conn: &mut Connection) -> anyhow::Result<()> {
    embedded::migrations::runner()
        .run(conn)
        .map_err(|err| anyhow::anyhow!("failed to run state DB migrations: {err}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    fn table_exists(conn: &Connection, table_name: &str) -> bool {
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            [table_name],
            |row| row.get::<_, i64>(0),
        )
        .unwrap()
            == 1
    }

    #[test]
    fn fresh_db_runs_v1_and_v2_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();

        super::run_migrations(&mut conn).unwrap();

        assert!(table_exists(&conn, "sessions"));
        assert!(table_exists(&conn, "messages"));
        assert!(table_exists(&conn, "active_sessions"));
        assert!(table_exists(&conn, "session_pinned_contexts"));
        assert!(table_exists(&conn, "session_pinned_skills"));
        assert!(table_exists(&conn, "pending_approvals"));
        assert!(table_exists(&conn, "pending_turns"));
        assert!(table_exists(&conn, "refinery_schema_history"));
    }

    fn migrated_conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        super::run_migrations(&mut conn).unwrap();
        conn
    }

    fn column_names(conn: &Connection, table_name: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({table_name})"))
            .unwrap();
        stmt.query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn index_exists(conn: &Connection, index_name: &str) -> bool {
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'index' AND name = ?1)",
            [index_name],
            |row| row.get::<_, i64>(0),
        )
        .unwrap()
            == 1
    }

    fn seed_session_and_message(conn: &Connection) {
        conn.execute(
            "INSERT INTO sessions (
                id, title, created_at, updated_at, status, surface_created, actor_id_created, active
            ) VALUES (
                'sess_1', 'Test', '2026-06-07T00:00:00Z', '2026-06-07T00:00:00Z',
                'active', 'cli', 'cli_owner', 1
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (
                id, session_id, role, surface, actor_id, content, created_at
            ) VALUES (
                'msg_1', 'sess_1', 'user', 'cli', 'cli_owner', 'hello',
                '2026-06-07T00:00:00Z'
            )",
            [],
        )
        .unwrap();
    }

    #[test]
    fn v1_schema_uses_surface_and_actor_fields() {
        let conn = migrated_conn();

        let session_columns = column_names(&conn, "sessions");
        assert!(session_columns.contains(&"surface_created".to_string()));
        assert!(session_columns.contains(&"actor_id_created".to_string()));
        assert!(!session_columns.contains(&"source_created".to_string()));

        let message_columns = column_names(&conn, "messages");
        assert!(message_columns.contains(&"surface".to_string()));
        assert!(message_columns.contains(&"actor_id".to_string()));
        assert!(!message_columns.contains(&"source".to_string()));

        let active_columns = column_names(&conn, "active_sessions");
        assert!(active_columns.contains(&"actor_id".to_string()));
        assert!(active_columns.contains(&"surface".to_string()));
        assert!(!active_columns.contains(&"user_key".to_string()));
    }

    #[test]
    fn messages_role_check_rejects_invalid_role() {
        let conn = migrated_conn();
        seed_session_and_message(&conn);

        let result = conn.execute(
            "INSERT INTO messages (
                id, session_id, role, surface, actor_id, content, created_at
            ) VALUES (
                'msg_bad', 'sess_1', 'bad_role', 'cli', 'cli_owner', '{}',
                '2026-06-07T00:00:00Z'
            )",
            [],
        );

        assert!(result.is_err());
    }

    #[test]
    fn v2_schema_has_expected_approval_columns_and_indexes() {
        let conn = migrated_conn();

        let approval_columns = column_names(&conn, "pending_approvals");
        for expected in [
            "request_id",
            "turn_id",
            "user_message_id",
            "tool_call_id",
            "operation_target",
            "reason",
            "result_summary",
            "error_message",
            "completed_at",
        ] {
            assert!(approval_columns.contains(&expected.to_string()));
        }
        assert!(!approval_columns.contains(&"expires_at".to_string()));

        let turn_columns = column_names(&conn, "pending_turns");
        for expected in [
            "request_id",
            "turn_id",
            "provider_id",
            "model_id",
            "phase",
            "resume_payload_json",
        ] {
            assert!(turn_columns.contains(&expected.to_string()));
        }

        for expected in [
            "idx_pending_approvals_session_status",
            "idx_pending_approvals_actor_status",
            "idx_pending_approvals_request_id",
            "idx_pending_approvals_turn_id",
            "idx_pending_approvals_user_message_id",
            "idx_pending_approvals_operation_target",
            "idx_pending_turns_session",
            "idx_pending_turns_request_id",
            "idx_pending_turns_turn_id",
            "idx_pending_turns_phase",
        ] {
            assert!(index_exists(&conn, expected));
        }
    }

    #[test]
    fn pending_approval_status_check_rejects_invalid_status() {
        let conn = migrated_conn();
        seed_session_and_message(&conn);

        let result = conn.execute(
            "INSERT INTO pending_approvals (
                id, session_id, request_id, turn_id, user_message_id, tool_call_id,
                surface, actor_id, operation_name, classification, status, created_at
            ) VALUES (
                'appr_1', 'sess_1', 'req_1', 'turn_1', 'msg_1', 'toolcall_1',
                'cli', 'cli_owner', 'write_file', 'local_modify', 'approved',
                '2026-06-07T00:00:00Z'
            )",
            [],
        );

        assert!(result.is_err());
    }

    #[test]
    fn classification_accepts_evolving_free_text() {
        let conn = migrated_conn();
        seed_session_and_message(&conn);

        conn.execute(
            "INSERT INTO pending_approvals (
                id, session_id, request_id, turn_id, user_message_id, tool_call_id,
                surface, actor_id, operation_name, classification, status, created_at
            ) VALUES (
                'appr_1', 'sess_1', 'req_1', 'turn_1', 'msg_1', 'toolcall_1',
                'cli', 'cli_owner', 'tool', 'future_new_class', 'pending',
                '2026-06-07T00:00:00Z'
            )",
            [],
        )
        .unwrap();
    }

    #[test]
    fn pending_turn_phase_check_rejects_invalid_phase() {
        let conn = migrated_conn();
        seed_session_and_message(&conn);
        conn.execute(
            "INSERT INTO pending_approvals (
                id, session_id, request_id, turn_id, user_message_id, tool_call_id,
                surface, actor_id, operation_name, classification, status, created_at
            ) VALUES (
                'appr_1', 'sess_1', 'req_1', 'turn_1', 'msg_1', 'toolcall_1',
                'cli', 'cli_owner', 'tool', 'local_modify', 'pending',
                '2026-06-07T00:00:00Z'
            )",
            [],
        )
        .unwrap();

        let result = conn.execute(
            "INSERT INTO pending_turns (
                approval_id, session_id, request_id, turn_id, user_message_id,
                provider_id, model_id, phase, resume_payload_json, created_at, updated_at
            ) VALUES (
                'appr_1', 'sess_1', 'req_1', 'turn_1', 'msg_1',
                'gemini', 'gemini-2.5-flash', 'bad_phase', '{}',
                '2026-06-07T00:00:00Z', '2026-06-07T00:00:00Z'
            )",
            [],
        );

        assert!(result.is_err());
    }
}
