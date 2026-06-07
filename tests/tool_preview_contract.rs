use opennivara::tool_preview::{
    model_visible_tool_result_from_execution, preview_read_only_tool, preview_write_file,
    WriteFilePreviewInput, WriteFilePreviewMode,
};
use opennivara::tools::{ToolExecutionResult, ToolExecutionStatus};
use serde_json::json;

#[test]
fn write_file_preview_create_new_does_not_create_file() {
    let temp = tempfile::tempdir().expect("temp dir");
    let target = temp.path().join("notes.txt");

    let preview = preview_write_file(WriteFilePreviewInput {
        path: target.to_string_lossy().to_string(),
        mode: WriteFilePreviewMode::CreateNew,
        content: "hello world".to_string(),
        allowed_roots: vec![],
        blocked_patterns: vec![],
    })
    .expect("create_new preview");

    assert!(!target.exists());
    assert_eq!(preview.tool_name, "write_file");
    assert_eq!(preview.preview_kind, "local_modify");
    let operation_target = preview
        .operation_target
        .as_deref()
        .expect("operation target");
    assert!(operation_target.ends_with("notes.txt"));
    assert!(preview.summary.contains("Create"));
    assert_eq!(preview.details["mode"], "create_new");
}

#[test]
fn write_file_overwrite_preview_includes_unified_diff() {
    let temp = tempfile::tempdir().expect("temp dir");
    let target = temp.path().join("notes.txt");
    std::fs::write(&target, "old line\nsame\n").expect("seed file");

    let preview = preview_write_file(WriteFilePreviewInput {
        path: target.to_string_lossy().to_string(),
        mode: WriteFilePreviewMode::Overwrite,
        content: "new line\nsame\n".to_string(),
        allowed_roots: vec![],
        blocked_patterns: vec![],
    })
    .expect("overwrite preview");

    let diff = preview.details["diff"].as_str().expect("diff");
    assert!(diff.contains("--- before"));
    assert!(diff.contains("+++ after"));
    assert!(diff.contains("-old line"));
    assert!(diff.contains("+new line"));
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "old line\nsame\n"
    );
}

#[test]
fn write_file_preview_rejects_invalid_targets_before_approval() {
    let temp = tempfile::tempdir().expect("temp dir");
    let existing = temp.path().join("notes.txt");
    std::fs::write(&existing, "old").expect("seed file");

    let create_existing = preview_write_file(WriteFilePreviewInput {
        path: existing.to_string_lossy().to_string(),
        mode: WriteFilePreviewMode::CreateNew,
        content: "new".to_string(),
        allowed_roots: vec![],
        blocked_patterns: vec![],
    })
    .expect_err("create_new existing target fails");
    assert!(create_existing.to_string().contains("file_already_exists"));

    let missing = temp.path().join("missing.txt");
    let overwrite_missing = preview_write_file(WriteFilePreviewInput {
        path: missing.to_string_lossy().to_string(),
        mode: WriteFilePreviewMode::Overwrite,
        content: "new".to_string(),
        allowed_roots: vec![],
        blocked_patterns: vec![],
    })
    .expect_err("overwrite missing target fails");
    assert!(overwrite_missing
        .to_string()
        .contains("file_missing_for_overwrite"));
}

#[test]
fn read_only_previews_are_activity_records_not_approvals() {
    let preview = preview_read_only_tool("read_file", Some("Cargo.toml"), "Read Cargo.toml");

    assert_eq!(preview.preview_kind, "read_only");
    assert_eq!(preview.details["approval_required"], false);
}

#[test]
fn model_visible_results_are_compact_and_denials_use_approval_denied() {
    let denied = model_visible_tool_result_from_execution(&ToolExecutionResult {
        tool_name: "write_file".to_string(),
        tool_call_id: "toolcall_123".to_string(),
        status: ToolExecutionStatus::Denied,
        result_json: None,
        result_summary: None,
        error_code: None,
        error_message: None,
        truncation: None,
        started_at: "2026-06-07T00:00:00Z".to_string(),
        finished_at: "2026-06-07T00:00:01Z".to_string(),
    });

    assert!(!denied.ok);
    assert_eq!(denied.error.as_ref().unwrap().code, "approval_denied");
    assert_eq!(denied.result, None);

    let succeeded = model_visible_tool_result_from_execution(&ToolExecutionResult {
        tool_name: "read_file".to_string(),
        tool_call_id: "toolcall_456".to_string(),
        status: ToolExecutionStatus::Succeeded,
        result_json: Some(json!({ "content": "hello" })),
        result_summary: Some("Read file.".to_string()),
        error_code: None,
        error_message: None,
        truncation: None,
        started_at: "2026-06-07T00:00:00Z".to_string(),
        finished_at: "2026-06-07T00:00:01Z".to_string(),
    });

    assert!(succeeded.ok);
    assert_eq!(succeeded.summary, "Read file.");
    assert_eq!(succeeded.result, Some(json!({ "content": "hello" })));
    assert_eq!(succeeded.error, None);
}
