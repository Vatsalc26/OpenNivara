pub use crate::state::types::{DbMessage, StoreEventMessageInput, StoreMessageInput};
use chrono::Utc;
use rusqlite::{params, Connection};

pub fn store_message(conn: &Connection, input: StoreMessageInput) -> anyhow::Result<DbMessage> {
    let message_id = crate::runtime::ids::new_message_id();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO messages (
            id, session_id, role, surface, actor_id, content, created_at, metadata_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            message_id,
            input.session_id,
            input.role.as_str(),
            input.surface.as_str(),
            input.actor_id,
            input.content,
            now,
            input.metadata_json
        ],
    )?;
    conn.execute(
        "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
        params![now, input.session_id],
    )?;

    Ok(DbMessage {
        id: message_id,
        session_id: input.session_id,
        role: input.role.as_str().to_string(),
        surface: input.surface.as_str().to_string(),
        actor_id: input.actor_id,
        content: input.content,
        created_at: now,
        metadata_json: input.metadata_json,
    })
}

pub fn store_event_message(
    conn: &Connection,
    input: StoreEventMessageInput,
) -> anyhow::Result<DbMessage> {
    store_message(
        conn,
        StoreMessageInput {
            session_id: input.session_id,
            role: crate::state::types::MessageRole::Event,
            surface: input.surface,
            actor_id: input.actor_id,
            content: input.event_json,
            metadata_json: input.metadata_json,
        },
    )
}

pub fn get_session_messages(conn: &Connection, session_id: &str) -> anyhow::Result<Vec<DbMessage>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, surface, actor_id, content, created_at, metadata_json
         FROM messages
         WHERE session_id = ?1
         ORDER BY created_at ASC, id ASC",
    )?;
    let rows = stmt.query_map([session_id], message_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn message_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DbMessage> {
    Ok(DbMessage {
        id: row.get(0)?,
        session_id: row.get(1)?,
        role: row.get(2)?,
        surface: row.get(3)?,
        actor_id: row.get(4)?,
        content: row.get(5)?,
        created_at: row.get(6)?,
        metadata_json: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::state::db::open_state_db_at;
    use crate::state::sessions::{create_session, CreateSessionInput};
    use crate::state::types::{MessageRole, StoreEventMessageInput, StoreMessageInput, Surface};
    use tempfile::tempdir;

    #[test]
    fn store_message_and_event_message_return_typed_records() {
        let dir = tempdir().unwrap();
        let conn = open_state_db_at(dir.path().join("state.sqlite")).unwrap();
        let session = create_session(
            &conn,
            CreateSessionInput {
                title: None,
                surface_created: Surface::Cli,
                actor_id_created: Some("cli_owner".into()),
            },
        )
        .unwrap();

        let message = super::store_message(
            &conn,
            StoreMessageInput {
                session_id: session.id.clone(),
                role: MessageRole::User,
                surface: Surface::Cli,
                actor_id: Some("cli_owner".into()),
                content: "hello".into(),
                metadata_json: Some(r#"{"client":"test"}"#.into()),
            },
        )
        .unwrap();

        assert!(message.id.starts_with("msg_"));
        assert_eq!(message.role, "user");
        assert_eq!(message.surface, "cli");
        assert_eq!(message.actor_id.as_deref(), Some("cli_owner"));

        let event = super::store_event_message(
            &conn,
            StoreEventMessageInput {
                session_id: session.id.clone(),
                surface: Surface::Cli,
                actor_id: Some("cli_owner".into()),
                event_json: r#"{"event_type":"approval_required"}"#.into(),
                metadata_json: None,
            },
        )
        .unwrap();

        assert_eq!(event.role, "event");
        assert_eq!(event.content, r#"{"event_type":"approval_required"}"#);

        let messages = super::get_session_messages(&conn, &session.id).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, message.id);
        assert_eq!(messages[1].id, event.id);
    }
}
