use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::tools::{
    ModelVisibleToolError, ModelVisibleToolResult, ToolExecutionResult, ToolExecutionStatus,
    ToolPreviewEnvelope,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WriteFilePreviewMode {
    CreateNew,
    Overwrite,
}

impl WriteFilePreviewMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::CreateNew => "create_new",
            Self::Overwrite => "overwrite",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WriteFilePreviewInput {
    pub path: String,
    pub mode: WriteFilePreviewMode,
    pub content: String,
    pub allowed_roots: Vec<String>,
    pub blocked_patterns: Vec<String>,
}

pub fn preview_write_file(input: WriteFilePreviewInput) -> anyhow::Result<ToolPreviewEnvelope> {
    let resolved_path = crate::tools::validate_and_resolve_path(
        &input.path,
        &input.allowed_roots,
        &input.blocked_patterns,
    )?;
    let target = resolved_path.to_string_lossy().to_string();
    let content_bytes = input.content.len();

    if resolved_path.is_dir() {
        anyhow::bail!("target_is_directory: write_file target is a directory");
    }
    let parent = resolved_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("path_not_found: target has no parent directory"))?;
    if !parent.exists() {
        anyhow::bail!("path_not_found: parent directory does not exist");
    }

    match input.mode {
        WriteFilePreviewMode::CreateNew => {
            if resolved_path.exists() {
                anyhow::bail!("file_already_exists: create_new target already exists");
            }

            Ok(ToolPreviewEnvelope {
                schema_version: 1,
                tool_name: "write_file".to_string(),
                preview_kind: "local_modify".to_string(),
                operation_target: Some(target.clone()),
                summary: format!("Create {} with {} bytes.", target, content_bytes),
                details: json!({
                    "mode": input.mode.as_str(),
                    "content_bytes": content_bytes,
                    "content_preview": preview_text(&input.content),
                    "approval_required": true
                }),
            })
        }
        WriteFilePreviewMode::Overwrite => {
            if !resolved_path.exists() {
                anyhow::bail!("file_missing_for_overwrite: overwrite target does not exist");
            }
            if !resolved_path.is_file() {
                anyhow::bail!("target_is_not_file: overwrite target is not a file");
            }
            let before = std::fs::read_to_string(&resolved_path).map_err(|err| {
                anyhow::anyhow!("read_existing_failed: failed to read overwrite target: {err}")
            })?;
            let diff = unified_diff(&before, &input.content);

            Ok(ToolPreviewEnvelope {
                schema_version: 1,
                tool_name: "write_file".to_string(),
                preview_kind: "local_modify".to_string(),
                operation_target: Some(target.clone()),
                summary: format!("Overwrite {} with {} bytes.", target, content_bytes),
                details: json!({
                    "mode": input.mode.as_str(),
                    "content_bytes": content_bytes,
                    "content_preview": preview_text(&input.content),
                    "diff": diff,
                    "approval_required": true
                }),
            })
        }
    }
}

pub fn preview_read_only_tool(
    tool_name: impl Into<String>,
    operation_target: Option<&str>,
    summary: impl Into<String>,
) -> ToolPreviewEnvelope {
    ToolPreviewEnvelope {
        schema_version: 1,
        tool_name: tool_name.into(),
        preview_kind: "read_only".to_string(),
        operation_target: operation_target.map(str::to_string),
        summary: summary.into(),
        details: json!({ "approval_required": false }),
    }
}

pub fn model_visible_tool_result_from_execution(
    result: &ToolExecutionResult,
) -> ModelVisibleToolResult {
    match result.status {
        ToolExecutionStatus::Succeeded => ModelVisibleToolResult {
            ok: true,
            tool_name: result.tool_name.clone(),
            tool_call_id: result.tool_call_id.clone(),
            summary: result
                .result_summary
                .clone()
                .unwrap_or_else(|| "Tool executed successfully.".to_string()),
            result: result.result_json.clone(),
            error: None,
            metadata: result.truncation.as_ref().map(|truncation| {
                json!({
                    "truncation": truncation
                })
            }),
        },
        ToolExecutionStatus::Denied => ModelVisibleToolResult {
            ok: false,
            tool_name: result.tool_name.clone(),
            tool_call_id: result.tool_call_id.clone(),
            summary: result
                .result_summary
                .clone()
                .unwrap_or_else(|| "Tool execution was denied.".to_string()),
            result: None,
            error: Some(ModelVisibleToolError {
                code: "approval_denied".to_string(),
                message: result
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "The user denied approval for this tool call.".to_string()),
                recoverable: false,
            }),
            metadata: None,
        },
        ToolExecutionStatus::Failed => ModelVisibleToolResult {
            ok: false,
            tool_name: result.tool_name.clone(),
            tool_call_id: result.tool_call_id.clone(),
            summary: result
                .result_summary
                .clone()
                .or_else(|| result.error_message.clone())
                .unwrap_or_else(|| "Tool execution failed.".to_string()),
            result: None,
            error: Some(ModelVisibleToolError {
                code: result
                    .error_code
                    .clone()
                    .unwrap_or_else(|| "tool_failed".to_string()),
                message: result
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "Tool execution failed.".to_string()),
                recoverable: true,
            }),
            metadata: None,
        },
    }
}

fn preview_text(text: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 500;
    if text.chars().count() <= MAX_PREVIEW_CHARS {
        return text.to_string();
    }

    let mut preview: String = text.chars().take(MAX_PREVIEW_CHARS).collect();
    preview.push_str("\n[preview truncated]");
    preview
}

fn unified_diff(before: &str, after: &str) -> String {
    let before_lines: Vec<&str> = before.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();
    let mut diff = String::from("--- before\n+++ after\n");
    let max_len = before_lines.len().max(after_lines.len());

    for index in 0..max_len {
        match (before_lines.get(index), after_lines.get(index)) {
            (Some(old), Some(new)) if old == new => {
                diff.push(' ');
                diff.push_str(old);
                diff.push('\n');
            }
            (Some(old), Some(new)) => {
                diff.push('-');
                diff.push_str(old);
                diff.push('\n');
                diff.push('+');
                diff.push_str(new);
                diff.push('\n');
            }
            (Some(old), None) => {
                diff.push('-');
                diff.push_str(old);
                diff.push('\n');
            }
            (None, Some(new)) => {
                diff.push('+');
                diff.push_str(new);
                diff.push('\n');
            }
            (None, None) => {}
        }
    }

    diff
}
