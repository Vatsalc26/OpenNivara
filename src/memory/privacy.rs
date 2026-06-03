use super::types::MemoryMode;

pub fn memory_inclusion_allowed(mode: &MemoryMode) -> bool {
    !matches!(mode, MemoryMode::Off)
}

pub fn memory_saving_allowed(mode: &MemoryMode, private_chat: bool, paused: bool) -> bool {
    !paused && !private_chat && !matches!(mode, MemoryMode::Off)
}

pub fn is_sensitive_category(category: &str) -> bool {
    matches!(
        category,
        "health"
            | "finance"
            | "relationships"
            | "location"
            | "identity"
            | "credentials"
            | "secrets"
            | "private_conversations"
            | "legal"
            | "work_confidential"
            | "minors"
            | "family"
    )
}
