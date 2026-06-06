use uuid::Uuid;

fn prefixed_id(prefix: &str) -> String {
    format!("{prefix}_{}", Uuid::new_v4())
}

pub fn new_request_id() -> String {
    prefixed_id("req")
}

pub fn new_turn_id() -> String {
    prefixed_id("turn")
}

pub fn new_session_id() -> String {
    prefixed_id("sess")
}

pub fn new_message_id() -> String {
    prefixed_id("msg")
}

pub fn new_tool_call_id() -> String {
    prefixed_id("toolcall")
}

pub fn new_approval_id() -> String {
    prefixed_id("appr")
}

pub fn new_resume_attempt_id() -> String {
    prefixed_id("resume")
}
