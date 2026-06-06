# Observability And Logging

OpenNivara needs the approval/tool/provider lifecycle to be debuggable without turning chat history into a noisy log stream.

Use the existing `tracing` and `tracing-subscriber` stack. Do not add a different logging framework. Do not introduce external telemetry.

## Dependency Updates

Update `tracing-subscriber` features:

```toml
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
```

Add:

```toml
tracing-appender = "0.2"
```

Optional later:

```toml
tracing-error = "0.2"
```

## Three Observability Layers

Use three layers:

1. Chat-visible events.
2. Durable approval audit.
3. Developer logs.

## Chat-Visible Events

Chat-visible events are messages with:

- `role = "event"`
- `content = JSON`

Use chat-visible events only for important user-relevant lifecycle events.

Recommended event types:

- `approval_required`
- `approval_approved`
- `approval_denied`
- `approval_executed`
- `approval_failed`
- `approval_completed`

Optional:

- `approval_resume_failed`
- `approval_interrupted`

Do not store every automatic read/open/list operation as a visible chat event by default. Automatic tool activity can be shown ephemerally in UI but should not make chat history noisy.

## Durable Approval Audit

Use the `pending_approvals` row as the durable audit record for approval-required operations.

`pending_approvals` should store compact audit fields:

- status
- phase indirectly via `pending_turns` while active
- operation name
- operation target
- classification
- summary
- reason
- result summary
- error message
- resume attempt count
- last resume error
- timestamps

Do not store full file content, full binary blobs, huge stdout/stderr, or full provider prompts in `pending_approvals`.

Automatic operations do not get durable audit rows in v1. Do not add a `tool_activity` table in v1.

## Developer Logs

Use tracing spans/events for detailed debugging.

Recommended tracing events/spans:

- `engine.request_started`
- `engine.request_finished`
- `context.compile_started`
- `context.compile_finished`
- `provider.generate_started`
- `provider.generate_finished`
- `provider.generate_failed`
- `tool.call_received`
- `tool.classified`
- `tool.preview_built`
- `tool.execution_started`
- `tool.execution_finished`
- `tool.execution_failed`
- `connector.account_resolved`
- `connector.credential_loaded`
- `connector.scope_check_failed`
- `connector.request_started`
- `connector.request_finished`
- `approval.created`
- `approval.approved`
- `approval.denied`
- `approval.executed`
- `approval.completed`
- `approval.resume_failed`
- `approval.recovery_stale_executing`

Recommended structured fields:

- `request_id`
- `turn_id`
- `session_id`
- `message_id`
- `approval_id`
- `resume_attempt_id`
- `tool_call_id`
- `tool_name`
- `classification`
- `approval_required`
- `connector_id`
- `capability_id`
- `account_id`
- `surface`
- `actor_id`
- `status`
- `phase`
- `duration_ms`
- `error_kind`

Use IDs, summaries, and counts in logs.

Do not log by default:

- file contents
- full tool arguments
- full base64 payloads
- provider prompt text
- provider response text
- API keys
- OAuth access tokens
- OAuth refresh tokens
- bot tokens
- authorization headers
- cookies
- environment variables
- secret-looking values
- raw command stdout/stderr
- `resume_payload_json`

Instead log:

- operation targets/paths
- byte counts
- line counts
- hashes where useful
- truncation flags
- operation summaries
- sanitized error strings

## Command And Provider Logging

The `run_command` command string may be logged because it is central to approval/debugging.

Do not log stdout/stderr by default. stdout/stderr should be capped in `ToolExecutionResult` and not copied into developer logs unless debug explicitly requests it.

Provider logging should include provider ID, model ID, request ID, duration, and error kind. Do not log full prompts, messages, or provider responses by default. Sanitize provider errors so API keys are never logged.

## Log Destinations

Default CLI/dev:

- human-readable stderr logs controlled by `RUST_LOG`

Desktop/daemon:

- optional file logs under config logs directory

Suggested paths:

- `<config_dir>/logs/opennivara.log`
- `<config_dir>/logs/opennivara-telegram.log`
- `<config_dir>/logs/opennivara-desktop.log`

Use `tracing-appender` for file logs later.

## Log Levels

`ERROR`:

- DB migration failure
- corrupted pending turn
- provider failure preventing response
- approved operation failed
- unrecoverable state transition failure

`WARN`:

- stale executing approval recovered
- duplicate approval attempt
- wrong-session approval attempt
- provider continuation retry failed
- invalid/forbidden state transition attempted

`INFO`:

- request started/finished
- approval created/approved/denied/completed
- approval-required tool executed summary
- recovery cleanup performed

`DEBUG`:

- tool preview details, redacted
- provider request metadata
- selected skill IDs
- context section counts
- output truncation details

`TRACE`:

- very detailed internal timing
- never enabled by default

## Redaction Helper

Add:

```rust
pub fn redact_for_log(value: &serde_json::Value) -> serde_json::Value
```

Redact values for keys containing:

- `api_key`
- `token`
- `access_token`
- `refresh_token`
- `secret`
- `password`
- `credential`
- `authorization`
- `cookie`

Summarize or omit keys:

- `content`
- `base64_content`
- `stdout`
- `stderr`
- `prompt`
- `messages`
- `resume_payload_json`

Example behavior:

- `content`: `<omitted: 12345 chars>`
- `base64_content`: `<omitted base64: 248122 bytes decoded>`
- `stdout`: `<omitted stdout: 20000 chars, truncated=true>`
- `messages`: `<omitted model messages: 12 messages>`
- `api_key`: `<redacted>`

Use `redact_for_log` before logging tool args, preview details, provider metadata, resume payload excerpts, and errors that may include secret-looking values.

## Chat History Policy

Approval lifecycle events are persisted as event messages.

Automatic read/open/list/search tool activities are not persisted as chat messages by default.

Automatic tool activity can be displayed ephemerally:

- Desktop: small inline activity row
- CLI: optional compact line
- Telegram: usually suppress to avoid noise

## Future Tool Activity Table

Only add later if full observability is needed.

Possible schema:

```text
tool_activity (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  request_id TEXT,
  tool_call_id TEXT,
  tool_name TEXT NOT NULL,
  classification TEXT NOT NULL,
  approval_id TEXT,
  operation_target TEXT,
  status TEXT NOT NULL,
  summary TEXT,
  preview_json TEXT,
  result_summary TEXT,
  started_at TEXT,
  finished_at TEXT,
  duration_ms INTEGER,
  error_message TEXT
)
```

Not for v1.

## Future Debug CLI Commands

Add later, not in the first implementation wave:

- `opennivara debug logs path`
- `opennivara debug approvals inspect <id>`
- `opennivara debug state status`
- `opennivara debug recovery run`

## Tests

Required tests:

1. `redact_for_log` redacts `api_key`, `token`, `secret`, `password`, `credential`, `authorization`, and `cookie` fields.
2. `redact_for_log` summarizes `content`, `base64_content`, `stdout`, `stderr`, `prompt`, `messages`, and `resume_payload_json`.
3. provider error sanitizer removes API keys.
4. approval lifecycle creates `role = "event"` messages.
5. automatic read/open activity does not create chat event messages by default.
6. `approval_required` event content is valid JSON.
7. `approval_failed` event stores sanitized error.
8. duplicate approval attempt logs `WARN` but does not execute.
9. stale executing recovery logs `WARN` and marks failed.
10. logging functions do not panic when given arbitrary JSON.
