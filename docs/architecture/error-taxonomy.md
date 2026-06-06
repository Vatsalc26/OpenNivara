# Error Taxonomy

OpenNivara needs consistent typed errors without overengineering. Do not expose raw `anyhow` strings directly to Desktop, CLI, or Telegram long-term.

Use two layers:

1. internal Rust errors
2. user-facing error DTOs

## Current Context

The backend mostly uses `anyhow::Result`.

Desktop Tauri commands currently map backend errors to raw strings with `e.to_string()`.

The tool registry often returns JSON error payloads such as `{"error": "..."}` instead of typed errors.

The Gemini/provider error sanitizer currently redacts only the API key in provider errors.

The repo already has `thiserror`, `serde`, `serde_json`, and `specta`.

## Internal Error Type

Add:

```text
src/error.rs
```

Target internal error:

```rust
#[derive(Debug, thiserror::Error)]
pub enum OpenNivaraError {
    State { message: String },
    Config { message: String },
    Provider { message: String },
    Tool { message: String },
    Approval { message: String },
    InvalidRequest { message: String },
    PermissionDenied { message: String },
    NotFound { message: String },
    Internal { message: String },
}
```

Use typed errors in new modules. Keep `anyhow` temporarily in old compatibility code until migrated.

## User-Facing Error DTO

Export user-facing errors through Specta.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
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
```

Surfaces render `UserFacingError.message`, not raw internal error text.

## Stable Error Codes

Request/config/provider:

- `missing_api_key`
- `invalid_request`
- `config_parse_failed`
- `state_db_open_failed`
- `state_migration_failed`
- `provider_request_failed`
- `provider_response_invalid`
- `provider_rate_limited`
- `provider_auth_failed`
- `provider_continuation_failed`

Tool:

- `tool_not_found`
- `tool_disabled`
- `tool_invalid_args`
- `tool_preview_failed`
- `tool_execution_failed`
- `tool_output_truncated`
- `path_not_found`
- `path_is_directory`
- `invalid_base64`
- `command_timed_out`
- `command_failed`
- `memory_disabled`

Approval/recovery:

- `approval_not_found`
- `approval_wrong_session`
- `approval_actor_not_allowed`
- `approval_already_denied`
- `approval_already_executing`
- `approval_already_executed`
- `approval_already_failed`
- `approval_already_completed`
- `approval_invalid_phase`
- `approval_missing_pending_turn`
- `approval_resume_not_available`
- `approval_execution_interrupted`
- `approval_duplicate_execution_blocked`
- `approval_pending_turn_corrupt`

Surface:

- `telegram_wrong_chat`
- `telegram_unauthorized_chat`
- `cli_missing_session`
- `desktop_command_failed`

## Approval Transition Mapping

Keep precise internal transition enums such as `BeginExecutionResult` and `DenyApprovalResult`.

Map them to `UserFacingError` at the engine/surface boundary.

Examples:

Wrong session:

- `kind = WrongSession`
- `code = approval_wrong_session`
- `message = "This approval belongs to another chat."`

Already executed:

- `kind = AlreadyResolved`
- `code = approval_already_executed`
- `message = "This operation already executed. Continue the final response instead."`

Missing pending turn:

- `kind = Approval`
- `code = approval_missing_pending_turn`
- `message = "OpenNivara could not resume this approval because its pending turn data is missing."`

## Tool Errors

Add typed `ToolError` later:

```rust
pub enum ToolError {
    NotFound { name: String },
    Disabled { name: String },
    InvalidArgs { tool_name: String, message: String },
    PreviewFailed { tool_name: String, message: String },
    ExecutionFailed { tool_name: String, message: String },
    CommandTimedOut { timeout_seconds: u64 },
}
```

Internally, tools should move toward:

```rust
Result<ToolExecutionResult, ToolError>
```

For model-visible tool results, convert tool errors to structured JSON as defined in [Model-Visible Tool Results](model-visible-tool-results.md).

## Provider Errors

Provider errors should classify:

- Auth
- RateLimited
- Timeout
- Network
- InvalidResponse
- Server
- Unknown

Provider error fields:

- provider ID
- model ID
- kind
- sanitized message
- retryable

Provider adapters must sanitize errors before returning. Broaden the current Gemini sanitizer beyond API-key replacement by reusing redaction helpers from [Observability And Logging](observability-logging.md).

Provider failure after approved tool execution should:

- call `mark_resume_failed`
- keep approval status `executed`
- return a recoverable user-facing error or `ApprovalActionResponse` saying final response can be continued later
- never mark the approval failed only because continuation failed

## State Errors

State/database errors should distinguish:

- not found
- wrong session
- invalid phase/status
- constraint violation
- migration failure
- corrupt pending turn JSON

## Surface Rendering

Desktop:

- show `UserFacingError.message`
- optional details disclosure
- no raw stack traces

CLI:

- show `Error [code]: message`
- optional future `--debug-errors` can show details JSON

Telegram:

- short friendly message
- no JSON dumps by default

Tauri command target:

```rust
Result<T, UserFacingError>
```

Temporary acceptable bridge:

```rust
Result<T, String>
```

The string should be serialized `UserFacingError` JSON, not raw `e.to_string()`.

## Tests

Tests should assert on:

- error kind
- error code
- recoverable flag

Do not assert on full prose message text.

Required decisions:

1. Add `UserFacingError` DTO and export it with Specta.
2. Add `ErrorKind` enum and stable error code strings.
3. Stop returning raw `anyhow` strings to Desktop long-term.
4. Keep `anyhow` temporarily in old code, but new modules use typed errors.
5. Tool execution results use structured ok/error JSON for model-visible results.
6. Provider errors are sanitized and classified.
7. Approval transition enums map to `UserFacingError` at boundary.
8. Tests assert on error code/kind, not full message text.
9. Tauri commands eventually return `Result<T, UserFacingError>`.
10. CLI/Telegram render `UserFacingError` into concise text.
