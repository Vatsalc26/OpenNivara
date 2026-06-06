pub use crate::state::types::{CreateSessionInput, Session};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

pub fn create_session(conn: &Connection, input: CreateSessionInput) -> anyhow::Result<Session> {
    let session_id = crate::runtime::ids::new_session_id();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO sessions (
            id, title, created_at, updated_at, status, surface_created, actor_id_created, active
        ) VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6, 1)",
        params![
            session_id,
            input.title,
            now,
            now,
            input.surface_created.as_str(),
            input.actor_id_created
        ],
    )?;

    get_session(conn, &session_id)?.ok_or_else(|| anyhow::anyhow!("created session was not found"))
}

pub fn get_session(conn: &Connection, session_id: &str) -> anyhow::Result<Option<Session>> {
    conn.query_row(
        "SELECT id, title, created_at, updated_at, status, surface_created, actor_id_created, active
         FROM sessions
         WHERE id = ?1",
        [session_id],
        session_from_row,
    )
    .optional()
    .map_err(Into::into)
}

pub fn list_sessions(conn: &Connection) -> anyhow::Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, updated_at, status, surface_created, actor_id_created, active
         FROM sessions
         ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], session_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn close_session(conn: &Connection, session_id: &str) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE sessions SET active = 0, status = 'closed', updated_at = ?1 WHERE id = ?2",
        params![now, session_id],
    )?;
    Ok(())
}

pub fn rename_session(conn: &Connection, session_id: &str, title: &str) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE sessions SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, session_id],
    )?;
    Ok(())
}

fn session_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
    let active: i64 = row.get(7)?;
    Ok(Session {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
        status: row.get(4)?,
        surface_created: row.get(5)?,
        actor_id_created: row.get(6)?,
        active: active != 0,
    })
}

#[cfg(test)]
mod tests {
    use crate::state::db::open_state_db_at;
    use crate::state::types::Surface;
    use tempfile::tempdir;

    #[test]
    fn create_session_returns_typed_session_and_supports_lifecycle() {
        let dir = tempdir().unwrap();
        let conn = open_state_db_at(dir.path().join("state.sqlite")).unwrap();

        let session = super::create_session(
            &conn,
            super::CreateSessionInput {
                title: Some("Planning".into()),
                surface_created: Surface::Cli,
                actor_id_created: Some("cli_owner".into()),
            },
        )
        .unwrap();

        assert!(session.id.starts_with("sess_"));
        assert_eq!(session.title.as_deref(), Some("Planning"));
        assert_eq!(session.surface_created, "cli");
        assert_eq!(session.actor_id_created.as_deref(), Some("cli_owner"));
        assert!(session.active);

        super::rename_session(&conn, &session.id, "Renamed").unwrap();
        let renamed = super::get_session(&conn, &session.id).unwrap().unwrap();
        assert_eq!(renamed.title.as_deref(), Some("Renamed"));

        super::close_session(&conn, &session.id).unwrap();
        let closed = super::get_session(&conn, &session.id).unwrap().unwrap();
        assert_eq!(closed.status, "closed");
        assert!(!closed.active);

        assert_eq!(super::list_sessions(&conn).unwrap().len(), 1);
    }
}
