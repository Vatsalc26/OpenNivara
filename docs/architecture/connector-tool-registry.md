# Connector Tool Registry

Connector tools should feel like normal typed tools to the model, but their availability is dynamic based on connected accounts, granted scopes, connector status, credential status, and tool config.

Avoid two extremes:

1. one giant static `ToolRegistry` with hundreds of hardcoded connector tools
2. one generic `connector_action` tool where the model passes arbitrary action names and JSON

Use the middle path:

- connector definitions own capabilities
- connector accounts own granted scopes
- `ConnectorToolProvider` exposes currently available scoped capabilities as normal model-callable tools
- `ToolRegistry` aggregates built-in tools, memory tools, and connector tools

## Core Types

```rust
pub struct ConnectorDefinition {
    pub connector_id: String,
    pub display_name: String,
    pub capabilities: Vec<ConnectorCapability>,
}

pub struct ConnectorCapability {
    pub capability_id: String,
    pub display_name: String,
    pub operation_kind: OperationKind,
    pub required_scopes: Vec<String>,
    pub tool_name: String,
    pub input_schema: serde_json::Value,
}
```

Capability ID naming:

- use namespaced dotted IDs internally
- examples: `github.issue.read`, `github.issue.comment`, `gmail.message.send`

Model-facing tool naming:

- use provider-safe snake_case names
- examples: `github_fetch_issue`, `github_comment_issue`, `gmail_send_email`

Do not expose dotted capability IDs as model tool names unless provider compatibility is guaranteed.

## Why Explicit Tool Names

Explicit model tool names are preferred because:

- the model understands them better
- schemas are clearer
- approval previews are action-specific
- tests can assert exact tools
- operation classification is simpler

Avoid in v1:

- one generic `connector_action`
- arbitrary action strings from the model
- generic authenticated `api_request`

## ToolProvider

```rust
pub trait ToolProvider {
    fn definitions(&self, context: &ToolDefinitionContext) -> Vec<ToolDefinition>;

    fn preview(
        &self,
        tool_name: &str,
        args: serde_json::Value,
        context: &ToolExecutionContext,
    ) -> Result<ToolPreview, ToolError>;

    fn execute(
        &self,
        tool_name: &str,
        args: serde_json::Value,
        context: &ToolExecutionContext,
    ) -> Result<ToolExecutionResult, ToolError>;
}
```

Tool providers:

- `BuiltinToolProvider`
- `MemoryToolProvider`
- `ConnectorToolProvider`

`ToolRegistry` target:

```rust
pub struct ToolRegistry {
    providers: Vec<Box<dyn ToolProvider>>,
}
```

`ToolRegistry` responsibilities:

- aggregate definitions from providers
- route preview calls to the provider owning `tool_name`
- route execute calls to the provider owning `tool_name`
- prevent duplicate tool names
- expose only enabled and available tools to the model

## ConnectorToolProvider

Responsibilities:

- load connected accounts
- check connector enabled state
- check account status
- check credential status
- check granted scopes
- check tool config enablement
- expose eligible connector capabilities as `ToolDefinition`s
- resolve `account_id` before preview/execution
- delegate to connector implementation module

A connector capability becomes a model tool only if:

- connector is enabled
- account exists
- account status is `connected`
- credential status is `active`
- account has required scopes
- tool is enabled in config
- surface/session policy allows connected account use

If no account/scope is available:

- do not expose that tool
- avoid repeated model calls that only return `connector_not_connected` or `missing_scope`

Missing scope information belongs in connector settings, not repeated tool failures.

## Multiple Accounts

Tool args may include optional `account_id`.

Example:

```json
{
  "account_id": "acct_123",
  "repository_full_name": "owner/repo",
  "issue_number": 12,
  "body": "Proposed comment body"
}
```

If `account_id` is omitted:

- use the default connected account for that connector when configured
- if exactly one suitable account exists, use it
- if multiple suitable accounts exist and no default is configured, preview fails with `account_selection_required`

Preview must always resolve and show the actual account before execution or approval.

## Preview Examples

External read:

```json
{
  "schema_version": 1,
  "tool_name": "github_fetch_issue",
  "preview_kind": "external_read",
  "operation_target": "github:owner/repo#12",
  "summary": "OpenNivara will read GitHub issue #12.",
  "details": {
    "connector_id": "github",
    "capability_id": "github.issue.read",
    "account_display_name": "GitHub account",
    "required_scopes": ["issues:read"],
    "approval_required": false
  }
}
```

External mutation:

```json
{
  "schema_version": 1,
  "tool_name": "github_comment_issue",
  "preview_kind": "external_mutation",
  "operation_target": "github:owner/repo#12",
  "summary": "OpenNivara wants to comment on GitHub issue #12.",
  "details": {
    "connector_id": "github",
    "capability_id": "github.issue.comment",
    "account_id": "acct_123",
    "account_display_name": "GitHub account",
    "repository_full_name": "owner/repo",
    "issue_number": 12,
    "body_preview": "Here is the proposed comment...",
    "required_scopes": ["issues:write"],
    "classification": "external_mutation"
  }
}
```

## Tool Results

Connector tools use `ModelVisibleToolResult` like all other tools.

Success:

```json
{
  "ok": true,
  "tool_name": "github_fetch_issue",
  "tool_call_id": "toolcall_123",
  "summary": "Fetched GitHub issue #12.",
  "result": {
    "repository_full_name": "owner/repo",
    "issue_number": 12,
    "title": "Issue title",
    "state": "open"
  },
  "error": null,
  "metadata": null
}
```

Denied mutation:

```json
{
  "ok": false,
  "tool_name": "github_comment_issue",
  "tool_call_id": "toolcall_456",
  "summary": "The user denied approval for this GitHub comment.",
  "result": null,
  "error": {
    "code": "approval_denied",
    "message": "The user denied approval for this external mutation. Do not send it again unless the user asks.",
    "recoverable": false
  },
  "metadata": null
}
```

## Config

Connector tools should appear in `tools.toml` only after implemented.

Example later:

```toml
[tools.github_fetch_issue]
enabled = true

[tools.github_search_issues]
enabled = true

[tools.github_comment_issue]
enabled = true
```

Connector/account config lives separately from `tools.toml`:

- `connector_accounts` in state DB
- `connector_credentials` metadata in state DB
- secret material in keychain

Do not declare connector tools when:

- connector is disabled
- no connected account exists
- required scopes are missing
- credential is missing, revoked, or expired
- tool is disabled

## SDK Policy

Use SDKs when implementing real connector modules if the SDK is maintained, typed, and helpful.

Do not add large SDKs before the connector abstraction exists.

Likely future choices:

- GitHub: consider `octocrab`
- Slack: consider `slack-morphism`
- Gmail: evaluate a Gmail crate versus handwritten `reqwest`

## Locked Decisions

1. Use explicit connector tool names, not one generic `connector_action`.
2. Keep internal capability IDs namespaced with dots.
3. Model-facing tool names use snake_case.
4. `ConnectorToolProvider` dynamically exposes tools based on connected accounts/scopes.
5. Capability metadata carries `OperationKind`.
6. `ExternalRead` is automatic.
7. `ExternalMutation` requires approval.
8. If no account/scope is available, do not expose that tool.
9. Tool args may include optional `account_id`.
10. Preview must resolve and show actual account before execution or approval.
11. Multiple accounts require default account selection or explicit `account_id`.
12. `ToolRegistry` becomes an aggregator over built-in, memory, and connector providers.
13. SDKs are added connector-by-connector, not all upfront.

## Tests

Required tests:

1. Connector capability maps to `ToolDefinition`.
2. Dotted capability ID maps to snake_case tool name.
3. `ExternalRead` capability is automatic.
4. `ExternalMutation` capability requires approval.
5. Missing account means tool is not exposed.
6. Missing scope means tool is not exposed.
7. Multiple accounts without default cause `account_selection_required` at preview time.
8. Preview resolves actual account.
9. External mutation preview includes connector/account/scopes/target/body.
10. `ToolRegistry` rejects duplicate tool names.
11. `ConnectorToolProvider` does not expose disabled connector tools.
12. `ModelVisibleToolResult` shape is used for connector successes/errors.
13. Approval denial for connector mutation returns `approval_denied` model-visible result.
