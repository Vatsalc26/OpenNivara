# External Operations Policy

OpenNivara treats external operations by effect, not by surface.

External read operations run automatically. External mutation operations require per-operation approval.

## Classifications

`ExternalRead`:

- reads from a remote service
- does not mutate remote state
- automatic

Examples:

- `http_get`
- read an issue
- search repositories
- read mail
- read messages

`ExternalMutation`:

- creates, updates, sends, deletes, or triggers remote side effects
- approval required

Examples:

- send email
- send chat message
- create issue
- comment on issue
- POST webhook
- PUT/PATCH/DELETE API request

`Unknown`:

- effect cannot be confidently classified
- approval required

Configured model-provider calls are a special case. Sending prompt/context to the configured model provider remains automatic and is not treated like arbitrary external mutation.

## Foundation Before Mutations

Authenticated external side-effect tools must wait for:

- connector registry
- account metadata
- credential metadata
- `CredentialStore`
- redaction policy
- scoped account/capability checks
- external mutation approval previews

Unauthenticated external read tools can land earlier. The first one should be `http_get`, defined in [First Connector Scope](first-connector-scope.md).

## External Mutation Preview

Every external mutation approval preview must show:

- connector
- account display name
- external identity if available
- capability
- target/destination
- method/action
- body/comment/message preview
- required scopes
- classification reason

Example:

```json
{
  "schema_version": 1,
  "tool_name": "github_comment_issue",
  "preview_kind": "external_mutation",
  "operation_target": "github:owner/repo#123",
  "summary": "OpenNivara wants to comment on GitHub issue #123.",
  "details": {
    "connector_id": "github",
    "capability_id": "github.issue.comment",
    "account_id": "acct_123",
    "account_display_name": "GitHub account",
    "repository_full_name": "owner/repo",
    "issue_number": 123,
    "body_preview": "Here is the proposed comment...",
    "body_truncated": false,
    "required_scopes": ["issues:write"],
    "classification": "external_mutation",
    "classification_reason": "Capability declares external_mutation."
  }
}
```

Do not include access tokens, refresh tokens, API keys, bot tokens, authorization headers, cookies, or raw credential material in previews.

## External Read Activity Preview

External reads do not require approval, but they can create lightweight activity previews.

Example:

```json
{
  "schema_version": 1,
  "tool_name": "github_fetch_issue",
  "preview_kind": "external_read",
  "operation_target": "github:owner/repo#123",
  "summary": "OpenNivara will read GitHub issue #123.",
  "details": {
    "connector_id": "github",
    "capability_id": "github.issue.read",
    "account_id": "acct_123",
    "account_display_name": "GitHub account",
    "required_scopes": ["issues:read"],
    "approval_required": false
  }
}
```

## Generic API Request

Do not add a generic authenticated `api_request` in v1.

Reasons:

- arbitrary action/method bodies are harder to preview safely
- redaction is connector-specific
- account/scope policy needs maturity first
- model tool schemas are clearer when actions are explicit

Unauthenticated `http_get` is acceptable as the first external read tool. Authenticated generic API requests should wait until connector/auth/redaction policy is mature.

## Tests

Required tests:

1. `ExternalRead` does not require approval.
2. `ExternalMutation` requires approval.
3. configured model-provider calls remain automatic.
4. external mutation previews include connector, account, capability, scopes, target, and body/destination.
5. external read activity previews include connector/capability details where applicable.
6. approval previews redact all credentials.
7. unauthenticated `http_get` is allowed before connector foundation.
8. authenticated external mutation tools are not exposed before connector foundation.
