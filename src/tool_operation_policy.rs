use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    ReadOnly,
    Opening,
    WorkspaceIndex,
    ExternalRead,
    ExternalMutation,
    LocalModify,
    LocalDelete,
    ShellCommand,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationClassification {
    ReadOnly,
    Opening,
    WorkspaceIndex,
    ExternalRead,
    ExternalMutation,
    LocalModify,
    LocalDelete,
    ShellReadOnly,
    ShellMutating,
    ShellDeleting,
    ShellUnknown,
    Unknown,
}

impl OperationClassification {
    pub fn requires_approval(self) -> bool {
        matches!(
            self,
            Self::ExternalMutation
                | Self::LocalModify
                | Self::LocalDelete
                | Self::ShellMutating
                | Self::ShellDeleting
                | Self::ShellUnknown
                | Self::Unknown
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationDecision {
    pub classification: OperationClassification,
    pub approval_required: bool,
    pub reason: String,
}

impl OperationDecision {
    fn new(classification: OperationClassification, reason: impl Into<String>) -> Self {
        Self {
            classification,
            approval_required: classification.requires_approval(),
            reason: reason.into(),
        }
    }
}

pub fn classify_tool_operation(
    tool_name: &str,
    operation_kind: OperationKind,
    args: &serde_json::Value,
) -> OperationDecision {
    match operation_kind {
        OperationKind::ReadOnly => decision(
            OperationClassification::ReadOnly,
            "Tool declares read_only.",
        ),
        OperationKind::Opening => {
            decision(OperationClassification::Opening, "Tool declares opening.")
        }
        OperationKind::WorkspaceIndex => decision(
            OperationClassification::WorkspaceIndex,
            "Tool declares workspace_index.",
        ),
        OperationKind::ExternalRead => decision(
            OperationClassification::ExternalRead,
            "Tool declares external_read.",
        ),
        OperationKind::ExternalMutation => decision(
            OperationClassification::ExternalMutation,
            "Tool declares external_mutation.",
        ),
        OperationKind::LocalModify => decision(
            OperationClassification::LocalModify,
            "Tool declares local_modify.",
        ),
        OperationKind::LocalDelete => decision(
            OperationClassification::LocalDelete,
            "Tool declares local_delete.",
        ),
        OperationKind::ShellCommand => args
            .get("command")
            .and_then(|value| value.as_str())
            .map(classify_shell_command)
            .unwrap_or_else(|| {
                decision(
                    OperationClassification::ShellUnknown,
                    format!("Tool '{tool_name}' did not provide a shell command string."),
                )
            }),
        OperationKind::Unknown => decision(
            OperationClassification::Unknown,
            format!("Tool '{tool_name}' declares an unknown operation."),
        ),
    }
}

pub fn classify_unknown_tool(tool_name: &str) -> OperationDecision {
    decision(
        OperationClassification::Unknown,
        format!("Tool '{tool_name}' is not recognized."),
    )
}

pub fn classify_shell_command(command: &str) -> OperationDecision {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return decision(
            OperationClassification::ShellUnknown,
            "Shell command is empty.",
        );
    }
    if is_complex_shell_command(trimmed) {
        return decision(
            OperationClassification::ShellUnknown,
            "Command contains compound shell syntax or redirection.",
        );
    }

    let tokens: Vec<String> = trimmed
        .split_whitespace()
        .map(|token| token.trim_matches('"').trim_matches('\'').to_lowercase())
        .collect();
    let Some(program) = tokens.first().map(String::as_str) else {
        return decision(
            OperationClassification::ShellUnknown,
            "Shell command has no program.",
        );
    };

    if is_deleting_command(program, &tokens) {
        return decision(
            OperationClassification::ShellDeleting,
            format!("Command '{trimmed}' deletes files or state."),
        );
    }
    if is_mutating_command(program, &tokens) {
        return decision(
            OperationClassification::ShellMutating,
            format!("Command '{trimmed}' modifies files, source, or project state."),
        );
    }
    if is_read_only_command(program, &tokens) {
        return decision(
            OperationClassification::ShellReadOnly,
            format!("Command '{trimmed}' is known read-only or build/test output only."),
        );
    }

    decision(
        OperationClassification::ShellUnknown,
        format!("Command '{trimmed}' is not in the shell classifier allowlist."),
    )
}

pub fn classify_opening(target: &str) -> OperationDecision {
    decision(
        OperationClassification::Opening,
        format!("Opening '{target}' does not modify or delete state."),
    )
}

pub fn classify_external_http_method(method: &str) -> OperationDecision {
    match method.trim().to_ascii_uppercase().as_str() {
        "GET" | "HEAD" | "OPTIONS" => decision(
            OperationClassification::ExternalRead,
            format!("HTTP method {} reads external state.", method.trim()),
        ),
        "POST" | "PUT" | "PATCH" | "DELETE" => decision(
            OperationClassification::ExternalMutation,
            format!("HTTP method {} mutates external state.", method.trim()),
        ),
        _ => decision(
            OperationClassification::Unknown,
            format!("HTTP method {} is unknown.", method.trim()),
        ),
    }
}

pub fn classify_model_provider_send(provider: &str) -> OperationDecision {
    decision(
        OperationClassification::ExternalRead,
        format!("Sending context to configured model provider '{provider}' is automatic."),
    )
}

fn decision(
    classification: OperationClassification,
    reason: impl Into<String>,
) -> OperationDecision {
    OperationDecision::new(classification, reason)
}

fn is_complex_shell_command(command: &str) -> bool {
    let lower = command.to_ascii_lowercase();
    command.contains('|')
        || command.contains('>')
        || command.contains('<')
        || command.contains(';')
        || command.contains('\n')
        || command.contains('\r')
        || lower.contains("&&")
        || lower.contains("||")
        || lower.starts_with("bash -c")
        || lower.starts_with("sh -c")
        || lower.starts_with("powershell -command")
        || lower.starts_with("pwsh -command")
}

fn is_deleting_command(program: &str, tokens: &[String]) -> bool {
    matches!(
        program,
        "rm" | "rmdir" | "del" | "erase" | "remove-item" | "trash"
    ) || (program == "git" && tokens.get(1).is_some_and(|arg| arg == "clean"))
}

fn is_mutating_command(program: &str, tokens: &[String]) -> bool {
    matches!(program, "touch" | "mkdir" | "cp" | "mv" | "chmod" | "chown")
        || (program == "git"
            && tokens
                .get(1)
                .is_some_and(|arg| matches!(arg.as_str(), "add" | "commit" | "checkout" | "reset")))
        || (program == "cargo"
            && tokens
                .get(1)
                .is_some_and(|arg| matches!(arg.as_str(), "fmt" | "fix" | "add")))
        || (program == "npm"
            && tokens
                .get(1)
                .is_some_and(|arg| matches!(arg.as_str(), "install" | "update" | "uninstall")))
        || (program == "bun"
            && tokens
                .get(1)
                .is_some_and(|arg| matches!(arg.as_str(), "add" | "remove")))
        || (program == "pnpm" && tokens.get(1).is_some_and(|arg| arg == "add"))
        || (program == "pip" && tokens.get(1).is_some_and(|arg| arg == "install"))
        || (program == "brew" && tokens.get(1).is_some_and(|arg| arg == "install"))
        || (program == "apt" && tokens.get(1).is_some_and(|arg| arg == "install"))
}

fn is_read_only_command(program: &str, tokens: &[String]) -> bool {
    matches!(
        program,
        "pwd" | "ls" | "dir" | "cat" | "type" | "head" | "tail" | "grep" | "rg" | "find"
    ) || (program == "git"
        && tokens
            .get(1)
            .is_some_and(|arg| matches!(arg.as_str(), "status" | "diff")))
        || (program == "cargo"
            && tokens
                .get(1)
                .is_some_and(|arg| matches!(arg.as_str(), "check" | "test" | "build")))
        || (program == "npm"
            && (tokens.get(1).is_some_and(|arg| arg == "test")
                || tokens.get(1).is_some_and(|arg| arg == "run")
                    && tokens.get(2).is_some_and(|arg| arg == "build")))
        || (program == "bun"
            && (tokens.get(1).is_some_and(|arg| arg == "test")
                || tokens.get(1).is_some_and(|arg| arg == "run")
                    && tokens.get(2).is_some_and(|arg| arg == "build")))
        || (program == "pnpm" && tokens.get(1).is_some_and(|arg| arg == "test"))
        || (matches!(program, "python" | "node")
            && tokens.get(1).is_some_and(|arg| arg == "--version"))
}
