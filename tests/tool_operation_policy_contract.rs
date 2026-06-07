use opennivara::tool_operation_policy::{
    classify_external_http_method, classify_model_provider_send, classify_opening,
    classify_shell_command, OperationClassification, OperationKind,
};
use opennivara::tools::{ToolRegistry, ToolsConfig};

#[test]
fn implemented_tools_are_read_only_and_do_not_require_approval() {
    let registry = ToolRegistry::new(true);

    for name in [
        "get_current_dir",
        "list_dir",
        "file_exists",
        "read_file",
        "map_summary",
        "map_tree",
        "map_search",
        "map_get_node",
    ] {
        let definition = registry.definition(name).expect("implemented tool");
        assert_eq!(definition.operation_kind, OperationKind::ReadOnly);

        let decision = registry.classify_tool_call(name, &serde_json::json!({}));
        assert_eq!(decision.classification, OperationClassification::ReadOnly);
        assert!(!decision.approval_required);
        assert!(!decision.reason.trim().is_empty());
    }
}

#[test]
fn unknown_tools_classify_as_unknown_and_require_approval() {
    let decision =
        ToolRegistry::new(false).classify_tool_call("missing_tool", &serde_json::json!({}));

    assert_eq!(decision.classification, OperationClassification::Unknown);
    assert!(decision.approval_required);
    assert!(!decision.reason.trim().is_empty());
}

#[test]
fn approval_required_matches_operation_classification_contract() {
    let approval_required = [
        OperationClassification::ExternalMutation,
        OperationClassification::LocalModify,
        OperationClassification::LocalDelete,
        OperationClassification::ShellMutating,
        OperationClassification::ShellDeleting,
        OperationClassification::ShellUnknown,
        OperationClassification::Unknown,
    ];
    for classification in approval_required {
        assert!(classification.requires_approval(), "{classification:?}");
    }

    let automatic = [
        OperationClassification::ReadOnly,
        OperationClassification::Opening,
        OperationClassification::WorkspaceIndex,
        OperationClassification::ExternalRead,
        OperationClassification::ShellReadOnly,
    ];
    for classification in automatic {
        assert!(!classification.requires_approval(), "{classification:?}");
    }
}

#[test]
fn shell_classifier_separates_read_only_mutating_deleting_and_unknown_commands() {
    for command in [
        "pwd",
        "ls src",
        "git status",
        "git diff",
        "cargo check",
        "cargo test",
        "cargo build",
        "npm test",
        "npm run build",
        "bun test",
        "bun run build",
        "python --version",
        "node --version",
    ] {
        let decision = classify_shell_command(command);
        assert_eq!(
            decision.classification,
            OperationClassification::ShellReadOnly,
            "{command}"
        );
        assert!(!decision.approval_required, "{command}");
    }

    for command in [
        "cargo fmt",
        "cargo fix",
        "cargo add serde",
        "npm install",
        "bun add vite",
        "git add src",
    ] {
        let decision = classify_shell_command(command);
        assert_eq!(
            decision.classification,
            OperationClassification::ShellMutating,
            "{command}"
        );
        assert!(decision.approval_required, "{command}");
    }

    for command in [
        "rm target",
        "rmdir build",
        "del notes.txt",
        "Remove-Item notes.txt",
        "git clean -fd",
    ] {
        let decision = classify_shell_command(command);
        assert_eq!(
            decision.classification,
            OperationClassification::ShellDeleting,
            "{command}"
        );
        assert!(decision.approval_required, "{command}");
    }

    for command in [
        "custom-script --do-it",
        "bash -c \"echo hi\"",
        "echo hi > out.txt",
        "rg foo | sort",
    ] {
        let decision = classify_shell_command(command);
        assert_eq!(
            decision.classification,
            OperationClassification::ShellUnknown,
            "{command}"
        );
        assert!(decision.approval_required, "{command}");
    }
}

#[test]
fn opening_http_and_model_provider_classifiers_match_policy() {
    let opening = classify_opening("https://example.com");
    assert_eq!(opening.classification, OperationClassification::Opening);
    assert!(!opening.approval_required);

    for method in ["GET", "head", "OPTIONS"] {
        let decision = classify_external_http_method(method);
        assert_eq!(
            decision.classification,
            OperationClassification::ExternalRead
        );
        assert!(!decision.approval_required);
    }

    for method in ["POST", "put", "PATCH", "DELETE"] {
        let decision = classify_external_http_method(method);
        assert_eq!(
            decision.classification,
            OperationClassification::ExternalMutation
        );
        assert!(decision.approval_required);
    }

    let provider = classify_model_provider_send("gemini");
    assert_eq!(
        provider.classification,
        OperationClassification::ExternalRead
    );
    assert!(!provider.approval_required);
}

#[test]
fn generated_tools_toml_uses_liberal_defaults_only_for_implemented_tools() {
    let config: ToolsConfig =
        toml::from_str(opennivara::tools::DEFAULT_TOOLS_TOML).expect("default tools config parses");

    assert!(config.paths.allowed_roots.is_empty());
    assert!(config.paths.blocked_patterns.is_empty());
    assert!(config.tools["read_file"].enabled);
    assert!(!config.tools["read_file"].requires_confirmation);

    for future_tool in ["open_app", "open_url", "write_file", "run_command"] {
        assert!(!config.tools.contains_key(future_tool));
    }
}

#[test]
fn empty_allowed_roots_is_unrestricted_and_blocked_patterns_default_empty() {
    let resolved = opennivara::tools::validate_and_resolve_path(".env", &[], &[])
        .expect("empty allowed_roots is unrestricted");

    assert!(resolved.ends_with(".env"));
}
