pub mod clock;
pub mod context;
pub mod ids;
pub mod location;
pub mod model_registry;

#[cfg(test)]
mod id_contract_tests {
    use super::ids;

    #[test]
    fn runtime_ids_have_expected_prefixes_and_are_unique() {
        let request_a = ids::new_request_id();
        let request_b = ids::new_request_id();

        assert!(request_a.starts_with("req_"));
        assert!(request_b.starts_with("req_"));
        assert_ne!(request_a, request_b);

        assert!(ids::new_turn_id().starts_with("turn_"));
        assert!(ids::new_session_id().starts_with("sess_"));
        assert!(ids::new_message_id().starts_with("msg_"));
        assert!(ids::new_tool_call_id().starts_with("toolcall_"));
        assert!(ids::new_approval_id().starts_with("appr_"));
        assert!(ids::new_resume_attempt_id().starts_with("resume_"));
    }
}
