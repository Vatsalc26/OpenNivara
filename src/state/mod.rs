pub mod active_sessions;
pub mod approvals;
pub mod db;
pub mod messages;
pub mod migrations;
pub mod recovery;
pub mod sessions;
pub mod types;
pub mod views;

#[cfg(test)]
mod tests {
    #[test]
    fn state_module_exports_expected_submodules() {
        assert_eq!(super::db::STATE_DB_FILE_NAME, "opennivara_state.sqlite");
        assert_eq!(super::migrations::STATE_MIGRATION_COUNT, 2);
        assert_eq!(super::types::MessageRole::Event.as_str(), "event");
        assert!(super::approvals::actor_can_approve("cli_owner"));
    }
}
