# GitHub Connector V1

GitHub is the first authenticated connector after the connector/account/credential foundation exists.

Use two GitHub layers:

- GitHub V1A: read-only tools, automatic, no approval required
- GitHub V1B: low-risk mutation tools, approval required, only after approval flow is solid

Do not add GitHub tools before connector account metadata, connector credential metadata, `CredentialStore`, `ConnectorToolProvider`, account/scope checking, and approval flow for mutations exist.

Consider `octocrab` when implementation begins. Do not add it before the connector abstraction exists. If a narrow endpoint is easier with `reqwest`, handwritten `reqwest` is acceptable. Do not handwrite a large GitHub API by default.

## V1 Scope

GitHub V1A read-only:

- `github_list_repositories`
- `github_fetch_issue`
- `github_search_issues`
- `github_fetch_pr`
- `github_fetch_file`

Delayed from V1A:

- `github_search_code`

Reason: code search is useful but has extra API/result complexity. Repository, issue, PR, and file reads are enough for useful first workflows.

GitHub V1B low-risk mutations:

- `github_create_issue`
- `github_comment_issue`

Delayed from V1B:

- `github_add_issue_labels`

Reason: labels require extra validation/listing behavior and can come later.

Delayed high-risk tools:

- `github_update_file`
- `github_create_branch`
- `github_create_pull_request`
- `github_delete_file`
- `github_merge_pull_request`
- `github_update_ref`
- `github_rerun_workflow`
- workflow/action mutation tools

Reason: these need stronger previews, diffs, branch/ref safety, and recovery semantics.

## Internal Scopes

Keep scope names abstract internally first.

Suggested internal scopes:

- `github:repo:read`
- `github:issues:read`
- `github:issues:write`
- `github:pull_requests:read`
- `github:contents:read`
- `github:code:read`

Required scopes:

- `github_list_repositories`: `github:repo:read`
- `github_fetch_issue`: `github:issues:read`
- `github_search_issues`: `github:issues:read`
- `github_fetch_pr`: `github:pull_requests:read`
- `github_fetch_file`: `github:contents:read`
- `github_create_issue`: `github:issues:write`
- `github_comment_issue`: `github:issues:write`

## V1A Tools

### github_list_repositories

Capability:

- `github.repo.list`

Operation kind:

- `ExternalRead`

Approval:

- false

Args:

```json
{
  "account_id": "optional acct_<id>",
  "visibility": "all | public | private",
  "affiliation": "owner | collaborator | organization_member | all",
  "topn": 50
}
```

Defaults:

- `visibility = "all"`
- `affiliation = "all"`
- `topn = 50`

Result:

```json
{
  "repositories": [
    {
      "repository_full_name": "owner/repo",
      "description": "optional",
      "visibility": "public",
      "default_branch": "main",
      "archived": false,
      "updated_at": "..."
    }
  ],
  "truncated": false
}
```

Preview:

```json
{
  "schema_version": 1,
  "tool_name": "github_list_repositories",
  "preview_kind": "external_read",
  "operation_target": "github:repositories",
  "summary": "OpenNivara will list GitHub repositories visible to the connected account.",
  "details": {
    "connector_id": "github",
    "capability_id": "github.repo.list",
    "account_id": "acct_123",
    "account_display_name": "GitHub account",
    "visibility": "all",
    "topn": 50,
    "approval_required": false
  }
}
```

### github_fetch_issue

Capability:

- `github.issue.read`

Operation kind:

- `ExternalRead`

Approval:

- false

Args:

```json
{
  "account_id": "optional acct_<id>",
  "repository_full_name": "owner/repo",
  "issue_number": 123,
  "include_comments": false
}
```

Default:

- `include_comments = false`

Result:

```json
{
  "repository_full_name": "owner/repo",
  "issue_number": 123,
  "title": "Issue title",
  "state": "open",
  "body": "...",
  "author": "username",
  "labels": ["bug"],
  "assignees": [],
  "created_at": "...",
  "updated_at": "...",
  "url": "https://github.com/owner/repo/issues/123",
  "comments": []
}
```

Preview:

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
    "repository_full_name": "owner/repo",
    "issue_number": 123,
    "include_comments": false,
    "approval_required": false
  }
}
```

### github_search_issues

Capability:

- `github.issue.search`

Operation kind:

- `ExternalRead`

Approval:

- false

Args:

```json
{
  "account_id": "optional acct_<id>",
  "repository_full_name": "optional owner/repo",
  "query": "approval state machine",
  "state": "open | closed | all",
  "include_pull_requests": false,
  "topn": 20
}
```

Rules:

- if `repository_full_name` is provided, constrain query to that repo
- if `include_pull_requests = false`, search issues only
- `topn` should be capped, recommended max 50

Result:

```json
{
  "items": [
    {
      "repository_full_name": "owner/repo",
      "number": 123,
      "kind": "issue",
      "title": "Issue title",
      "state": "open",
      "url": "https://github.com/owner/repo/issues/123",
      "updated_at": "..."
    }
  ],
  "truncated": false
}
```

### github_fetch_pr

Capability:

- `github.pr.read`

Operation kind:

- `ExternalRead`

Approval:

- false

Args:

```json
{
  "account_id": "optional acct_<id>",
  "repository_full_name": "owner/repo",
  "pr_number": 123,
  "include_files": false,
  "include_comments": false,
  "include_diff": false
}
```

Defaults:

- `include_files = false`
- `include_comments = false`
- `include_diff = false`

Rules:

- if `include_diff = true`, cap diff size
- large diffs must include truncation metadata

Result:

```json
{
  "repository_full_name": "owner/repo",
  "pr_number": 123,
  "title": "PR title",
  "state": "open",
  "body": "...",
  "author": "username",
  "base_branch": "main",
  "head_branch": "feature",
  "mergeable": true,
  "url": "https://github.com/owner/repo/pull/123",
  "files": [],
  "comments": [],
  "diff": null,
  "diff_truncated": false
}
```

### github_fetch_file

Capability:

- `github.file.read`

Operation kind:

- `ExternalRead`

Approval:

- false

Args:

```json
{
  "account_id": "optional acct_<id>",
  "repository_full_name": "owner/repo",
  "path": "src/main.rs",
  "ref": "main",
  "start_line": 1,
  "end_line": 200,
  "max_bytes": 20000
}
```

Defaults:

- `ref = default branch` if omitted
- `start_line` and `end_line` optional
- `max_bytes = tool limit`

Rules:

- do not fetch huge binary files as text
- if file is binary or too large, return metadata plus truncation/error
- always include `bytes_read` and `truncated` metadata

Result:

```json
{
  "repository_full_name": "owner/repo",
  "path": "src/main.rs",
  "ref": "main",
  "sha": "...",
  "content": "...",
  "encoding": "utf-8",
  "bytes_read": 8421,
  "truncated": false
}
```

## V1B Tools

### github_create_issue

Capability:

- `github.issue.create`

Operation kind:

- `ExternalMutation`

Approval:

- true

Args:

```json
{
  "account_id": "optional acct_<id>",
  "repository_full_name": "owner/repo",
  "title": "Issue title",
  "body": "Issue body",
  "labels": [],
  "assignees": []
}
```

Delay milestone support for v1 unless needed.

Approval preview:

```json
{
  "schema_version": 1,
  "tool_name": "github_create_issue",
  "preview_kind": "external_mutation",
  "operation_target": "github:owner/repo/issues",
  "summary": "OpenNivara wants to create a GitHub issue.",
  "details": {
    "connector_id": "github",
    "capability_id": "github.issue.create",
    "account_id": "acct_123",
    "account_display_name": "GitHub account",
    "repository_full_name": "owner/repo",
    "title": "Issue title",
    "body_preview": "Issue body...",
    "body_truncated": false,
    "labels": [],
    "assignees": [],
    "required_scopes": ["github:issues:write"],
    "classification": "external_mutation"
  }
}
```

Result:

```json
{
  "repository_full_name": "owner/repo",
  "issue_number": 123,
  "title": "Issue title",
  "url": "https://github.com/owner/repo/issues/123",
  "state": "open"
}
```

### github_comment_issue

Capability:

- `github.issue.comment`

Operation kind:

- `ExternalMutation`

Approval:

- true

Args:

```json
{
  "account_id": "optional acct_<id>",
  "repository_full_name": "owner/repo",
  "issue_number": 123,
  "body": "Comment body"
}
```

Approval preview:

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
    "body_preview": "Comment body...",
    "body_truncated": false,
    "required_scopes": ["github:issues:write"],
    "classification": "external_mutation"
  }
}
```

Result:

```json
{
  "repository_full_name": "owner/repo",
  "issue_number": 123,
  "comment_id": 456,
  "url": "https://github.com/owner/repo/issues/123#issuecomment-456"
}
```

## Account Selection

All GitHub tools support optional `account_id`.

Resolution:

1. If `account_id` is provided, use it if it satisfies connector/scope/status.
2. Else if a default GitHub account satisfies requirements, use default.
3. Else if exactly one suitable GitHub account exists, use it.
4. Else preview fails with `account_selection_required`.

Never execute or approve an external operation until the account is resolved.

## Exposure Rules

Do not expose GitHub tools when:

- GitHub connector is disabled
- no connected account exists
- credential is missing, expired, or revoked
- required scopes are missing
- tool is disabled in `tools.toml`

Do not expose GitHub mutation tools until approval flow is implemented.

If GitHub is not connected:

- do not expose GitHub tools to the model
- answer normally: `GitHub is not connected yet. Connect a GitHub account from Desktop or CLI first.`
- this should not be a normal model tool call in V1

Add GitHub tools to `tools.toml` only when implemented.

Example later:

```toml
[tools.github_list_repositories]
enabled = true

[tools.github_fetch_issue]
enabled = true

[tools.github_search_issues]
enabled = true

[tools.github_fetch_pr]
enabled = true

[tools.github_fetch_file]
enabled = true

[tools.github_create_issue]
enabled = true

[tools.github_comment_issue]
enabled = true
```

Do not add unimplemented GitHub tools to default `tools.toml`.

## Model-Visible Results

All GitHub tools must return `ModelVisibleToolResult`.

Success:

```json
{
  "ok": true,
  "tool_name": "github_fetch_issue",
  "tool_call_id": "toolcall_123",
  "summary": "Fetched GitHub issue #123.",
  "result": {
    "repository_full_name": "owner/repo",
    "issue_number": 123,
    "title": "Issue title",
    "state": "open",
    "url": "https://github.com/owner/repo/issues/123"
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

## Locked V1 Scope

GitHub V1A read-only:

- `github_list_repositories`
- `github_fetch_issue`
- `github_search_issues`
- `github_fetch_pr`
- `github_fetch_file`

GitHub V1B low-risk mutations:

- `github_create_issue`
- `github_comment_issue`

Delay:

- `github_search_code`
- `github_add_issue_labels`
- `github_update_file`
- `github_create_branch`
- `github_create_pull_request`
- `github_delete_file`
- `github_merge_pull_request`
- workflow tools

## Tests

Required tests:

1. GitHub V1A tools classify as `ExternalRead`.
2. GitHub V1B tools classify as `ExternalMutation`.
3. GitHub V1B tools require approval.
4. GitHub tools are not exposed without a connected account.
5. GitHub tools are not exposed with missing scopes.
6. `github_fetch_issue` preview is `external_read`.
7. `github_fetch_file` respects `max_bytes`, `start_line`, and `end_line`.
8. `github_fetch_pr` caps diff output if `include_diff = true`.
9. `github_create_issue` preview includes account/repo/title/body/scopes.
10. `github_comment_issue` preview includes account/repo/issue/body/scopes.
11. denied `github_comment_issue` returns `approval_denied` `ModelVisibleToolResult`.
12. multiple GitHub accounts without default produce `account_selection_required`.
13. GitHub tool results use `ModelVisibleToolResult` shape.
14. Unimplemented GitHub tools are absent from default `tools.toml`.
