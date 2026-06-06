pub use crate::state::types::Surface;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

pub fn set_active_session(
    conn: &Connection,
    actor_id: &str,
    surface: Surface,
    session_id: &str,
) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO active_sessions (actor_id, surface, session_id, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(actor_id, surface)
         DO UPDATE SET session_id = excluded.session_id, updated_at = excluded.updated_at",
        params![actor_id, surface.as_str(), session_id, now],
    )?;
    Ok(())
}

pub fn get_active_session(
    conn: &Connection,
    actor_id: &str,
    surface: Surface,
) -> anyhow::Result<Option<String>> {
    conn.query_row(
        "SELECT session_id FROM active_sessions WHERE actor_id = ?1 AND surface = ?2",
        params![actor_id, surface.as_str()],
        |row| row.get(0),
    )
    .optional()
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use crate::state::db::open_state_db_at;
    use crate::state::sessions::{create_session, CreateSessionInput};
    use crate::state::types::Surface;
    use tempfile::tempdir;

    #[test]
    fn active_sessions_map_actor_and_surface_to_session() {
        let dir = tempdir().unwrap();
        let conn = open_state_db_at(dir.path().join("state.sqlite")).unwrap();
        let session = create_session(
            &conn,
            CreateSessionInput {
                title: Some("CLI".into()),
                surface_created: Surface::Cli,
                actor_id_created: Some("cli_owner".into()),
            },
        )
        .unwrap();

        super::set_active_session(&conn, "cli_owner", Surface::Cli, &session.id).unwrap();

        assert_eq!(
            super::get_active_session(&conn, "cli_owner", Surface::Cli).unwrap(),
            Some(session.id)
        );
        assert_eq!(
            super::get_active_session(&conn, "desktop_owner", Surface::Desktop).unwrap(),
            None
        );
    }
}
