# GitHub Connector V1

GitHub is the first authenticated connector after the connector/account/credential foundation exists.

Do not add GitHub tools before:

- connector account metadata exists
- connector credential metadata exists
- `CredentialStore` exists
- `ConnectorToolProvider` can expose capabilities by account/scope
- approval flow is ready for mutations

Consider `octocrab` when implementation begins. Do not add it before the connector abstraction exists. If `octocrab` is not suitable for a narrow endpoint, handwritten `reqwest` is acceptable. Do not handwrite the entire GitHub API by default.

## Capability Map

Read capabilities:

- `github.issue.read` -> `github_fetch_issue`
- `github.issue.search` -> `github_search_issues`
- `github.pr.read` -> `github_fetch_pr`
- `github.file.read` -> `github_fetch_file`
- `github.code.search` -> `github_search_code`
- `github.repo.list` -> `github_list_repositories`

Low-risk mutation capabilities:

- `github.issue.create` -> `github_create_issue`
- `github.issue.comment` -> `github_comment_issue`
- `github.issue.label.add` -> `github_add_issue_labels`

Stronger mutation capabilities:

- `github.file.update` -> `github_update_file`
- `github.branch.create` -> `github_create_branch`
- `github.pr.create` -> `github_create_pull_request`

Delayed high-risk capabilities:

- `github.file.delete` -> `github_delete_file`
- `github.pr.merge` -> `github_merge_pull_request`
- `github.ref.update` -> `github_update_ref`
- `github.workflow.rerun` -> `github_rerun_workflow`

## Phase 1: Read-Only Tools

Tools:

- `github_fetch_issue`
- `github_search_issues`
- `github_fetch_pr`
- `github_fetch_file`
- `github_search_code`
- `github_list_repositories`

All Phase 1 tools:

- `OperationKind = ExternalRead`
- automatic
- activity preview only
- not exposed without connected account/scope when authenticated access is needed

## Phase 2: Low-Risk Mutations

Tools:

- `github_create_issue`
- `github_comment_issue`
- `github_add_issue_labels`

All Phase 2 tools:

- `OperationKind = ExternalMutation`
- approval required
- preview shows account, repo, target, title/body/comment/labels, scopes, and classification reason

## Phase 3: Stronger Mutations

Tools:

- `github_update_file`
- `github_create_branch`
- `github_create_pull_request`

Additional preview requirements:

- file diffs
- branch/base/head preview
- commit message preview
- repository/branch target
- exact affected paths

## Delayed

Delay:

- `github_delete_file`
- `github_merge_pull_request`
- `github_update_ref`
- `github_rerun_workflow`
- `github_enable_auto_merge`
- `github_lock_issue_conversation`

Reason: these are higher-risk external mutations and need stronger preview/recovery semantics.

## Tool Schemas

`github_fetch_issue`:

```json
{
  "repository_full_name": "owner/repo",
  "issue_number": 123,
  "account_id": "optional acct_<id>"
}
```

`github_search_issues`:

```json
{
  "query": "bug label:triage",
  "repository_full_name": "owner/repo",
  "state": "open",
  "topn": 20,
  "account_id": "optional acct_<id>"
}
```

`github_fetch_pr`:

```json
{
  "repository_full_name": "owner/repo",
  "pr_number": 123,
  "include_diff": false,
  "include_comments": false,
  "account_id": "optional acct_<id>"
}
```

`github_fetch_file`:

```json
{
  "repository_full_name": "owner/repo",
  "path": "src/main.rs",
  "ref": "optional branch/tag/sha",
  "encoding": "utf-8",
  "start_line": 1,
  "end_line": 200,
  "account_id": "optional acct_<id>"
}
```

`github_create_issue`:

```json
{
  "repository_full_name": "owner/repo",
  "title": "Issue title",
  "body": "Issue body",
  "labels": ["optional"],
  "assignees": ["optional"],
  "milestone": null,
  "account_id": "optional acct_<id>"
}
```

`github_comment_issue`:

```json
{
  "repository_full_name": "owner/repo",
  "issue_number": 123,
  "body": "Comment body",
  "account_id": "optional acct_<id>"
}
```

`github_add_issue_labels`:

```json
{
  "repository_full_name": "owner/repo",
  "issue_number": 123,
  "labels": ["bug", "triage"],
  "account_id": "optional acct_<id>"
}
```

## Multiple Accounts

Tool args may include optional `account_id`.

If omitted:

- use the default account when configured
- use the only suitable account when exactly one exists
- fail preview with `account_selection_required` when multiple suitable accounts exist and no default is configured

Preview must always resolve and show the actual account before execution/approval.

## Preview Examples

GitHub read:

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
    "approval_required": false
  }
}
```

GitHub comment:

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
    "classification": "external_mutation"
  }
}
```

GitHub create issue:

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
    "labels": [],
    "assignees": [],
    "required_scopes": ["issues:write"],
    "classification": "external_mutation"
  }
}
```

## Model-Visible Results

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

## Exposure Rules

Do not expose GitHub tools when:

- GitHub connector is disabled
- no connected account exists
- credential is missing, expired, or revoked
- required scopes are missing
- tool is disabled in `tools.toml`

Do not expose mutation tools until approval flow is implemented.

Do not add unimplemented GitHub tools to default `tools.toml`.

## Tests

Required tests:

1. GitHub read capability maps to `ExternalRead`.
2. GitHub mutation capability maps to `ExternalMutation`.
3. GitHub tools are not exposed without connected account.
4. GitHub tools are not exposed with missing scopes.
5. `github_fetch_issue` preview is `external_read`.
6. `github_comment_issue` preview is `external_mutation`.
7. `github_comment_issue` requires approval.
8. denied `github_comment_issue` returns `approval_denied` model-visible result.
9. multiple GitHub accounts without default produce `account_selection_required`.
10. GitHub tool results use `ModelVisibleToolResult` shape.
