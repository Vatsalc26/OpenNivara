use chrono::Duration;
use rusqlite::Connection;

pub fn cleanup_completed_pending_turns(conn: &Connection) -> anyhow::Result<usize> {
    crate::state::approvals::cleanup_completed_pending_turns(conn)
}

pub fn recover_stale_executing_approvals(
    conn: &mut Connection,
    stale_after: Duration,
) -> anyhow::Result<usize> {
    crate::state::approvals::recover_stale_executing_approvals(conn, stale_after)
}
