# First Connector Scope

Separate the first external tool from the first authenticated connector.

First external tool:

- `http_get`

First authenticated connector:

- GitHub

Reason: `http_get` validates the external read path without account/auth complexity. GitHub is the best first authenticated connector because OpenNivara is a developer/codebase agent and GitHub read/mutation actions map naturally to codebase workflows.

## Phase 0: http_get

Tool name:

- `http_get`

Operation kind:

- `ExternalRead`

Approval:

- automatic

Auth:

- none

Dependencies:

- existing `reqwest`
- `url` crate for validation

Purpose:

- test `ExternalRead` classification
- test external activity preview
- test response caps/timeouts
- test redaction/logging basics
- avoid OAuth/account/keychain complexity

`http_get` should support:

- HTTP/HTTPS only
- URL validation
- timeout
- max response bytes
- content-type capture
- final URL after redirects
- truncation metadata

Do not support in v1:

- authentication headers
- arbitrary request bodies
- non-GET methods
- cookies
- generic API mutation

## Phase 1: Connector Foundation

Before authenticated connectors:

- connector account metadata
- connector credential metadata
- `CredentialStore` trait
- `MockCredentialStore`
- account/scope checking
- `ConnectorToolProvider` capability exposure

See [Connectors, Accounts, And Credentials](connectors-accounts-credentials.md) and [Connector Tool Registry](connector-tool-registry.md).

## Phase 2: GitHub Read-Only

Implement GitHub read-only connector tools first:

- `github_fetch_issue`
- `github_search_issues`
- `github_fetch_pr`
- `github_fetch_file`
- `github_search_code`
- `github_list_repositories`

All Phase 2 tools:

- `OperationKind = ExternalRead`
- `approval_required = false`
- activity preview only
- require connected GitHub account/scopes if authenticated access is needed
- do not expose if no connected account/scope is available

## Phase 3: GitHub Low-Risk Mutations

Add after approval flow is solid:

- `github_create_issue`
- `github_comment_issue`
- `github_add_issue_labels`

All Phase 3 tools:

- `OperationKind = ExternalMutation`
- `approval_required = true`
- approval preview must show account, repo, target, title/body/comment/labels, scopes, and classification reason

## Phase 4: Stronger GitHub Mutations

Delay until stronger previews exist:

- `github_update_file`
- `github_create_branch`
- `github_create_pull_request`

These require:

- file diffs
- branch/base/head preview
- commit message preview
- repository/branch target
- exact affected paths

## Later

Delay higher-risk GitHub mutations:

- `github_delete_file`
- `github_merge_pull_request`
- `github_update_ref`
- `github_rerun_workflow`
- `github_enable_auto_merge`
- `github_lock_issue_conversation`

Then consider:

- Gmail draft/send design, starting with create draft rather than send email
- Slack message design
- generic authenticated `api_request` only if still needed

## Locked Decisions

1. First external tool is `http_get`.
2. First authenticated connector is GitHub.
3. `http_get` is unauthenticated `ExternalRead`.
4. GitHub starts read-only.
5. GitHub low-risk mutations come after approval flow is solid.
6. High-risk GitHub mutations are delayed.
7. GitHub connector should consider `octocrab` when implementation begins.
8. Do not add `octocrab` until connector abstraction exists.
9. Do not expose unavailable/missing-scope GitHub tools.
10. GitHub mutation previews must show account/repo/target/body/scopes.
11. GitHub tools use `ModelVisibleToolResult`.

## Tests

Required tests:

1. `http_get` is `ExternalRead` and automatic.
2. `http_get` rejects non-HTTP/HTTPS URLs.
3. `http_get` records final URL after redirects.
4. `http_get` enforces max response bytes.
5. `http_get` includes truncation metadata.
6. GitHub read capability maps to `ExternalRead`.
7. GitHub mutation capability maps to `ExternalMutation`.
8. GitHub tools are not exposed without connected account.
9. GitHub tools are not exposed with missing scopes.
10. GitHub tool results use `ModelVisibleToolResult` shape.
