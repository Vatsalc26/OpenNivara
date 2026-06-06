pub mod db;
pub mod migrations;

#[cfg(test)]
mod tests {
    #[test]
    fn state_module_exports_expected_submodules() {
        assert_eq!(super::db::STATE_DB_FILE_NAME, "opennivara_state.sqlite");
        assert_eq!(super::migrations::STATE_MIGRATION_COUNT, 2);
    }
}
