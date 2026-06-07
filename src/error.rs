use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[specta(rename_all = "snake_case")]
pub enum ErrorKind {
    State,
    Config,
    Provider,
    Tool,
    Approval,
    InvalidRequest,
    PermissionDenied,
    NotFound,
    Conflict,
    AlreadyResolved,
    WrongSession,
    Validation,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct UserFacingError {
    pub kind: ErrorKind,
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub approval_id: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl UserFacingError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            code: "internal_error".to_string(),
            message: message.into(),
            recoverable: false,
            request_id: None,
            session_id: None,
            approval_id: None,
            details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_kind_serializes_as_snake_case() {
        let serialized = serde_json::to_string(&ErrorKind::PermissionDenied).unwrap();
        assert_eq!(serialized, "\"permission_denied\"");
    }
}
