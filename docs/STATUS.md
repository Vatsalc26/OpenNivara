# Docs Status

Current as of 2026-06-06.

The canonical current docs are linked from [README.md](README.md). Root-level legacy docs are retained as historical references until they are moved to `docs/archive/` or `docs/stale/`.

Product decisions:

- Desktop, CLI, and Telegram are equal user surfaces over the same agent engine.
- Browser preview is for fast React iteration only.
- Store discovers themes and skill packs; Settings owns assistant behavior.
- Read-only, opening, indexing, external-read, send-to-Gemini, and clearly read-only shell operations run without approval.
- Deleting, modifying, external mutation, mutating shell commands, deleting shell commands, unknown shell commands, and unknown operations require per-operation approval.
- Approval pauses and resumes the same agent turn, never expires, and cannot be replayed.
- Approved tool execution is exactly once. After status reaches `executed`, retry only provider/model continuation.
- Request IDs and turn IDs are stable cross-surface envelopes for recovery, logs, approvals, and provider calls.
- Specta remains the shared Desktop/frontend type-generation contract.
- User-facing errors use stable typed DTOs; surfaces should not render raw internal error strings.
- Model-visible tool results use one stable `{ ok, tool_name, tool_call_id, summary, result/error, metadata }` envelope.
- Internal tool execution results, model-visible results, UI errors, previews, pending turns, and audit rows are separate contracts.
- Pending turns freeze assembled model history; approval resume must not recompute context, skills, tools, or history.
- Memory proposals stay separate from tool approvals; memory extraction runs only after a completed turn.
- Memory proposal UX uses separate Desktop cards, CLI `memory proposals` commands, and Telegram memory commands; it does not reuse operation approval commands.
- `forget_memory` retracts/stops using memory; `delete_memory` means true permanent hard delete and remains unavailable until cleanup scope is honest.
- Connector/account/credential design must come before authenticated external mutation tools.
- Connector credential metadata belongs in SQLite; raw tokens/API keys/secrets belong in OS keychain, never directly in SQLite.
- First external tool is unauthenticated `http_get`; first authenticated connector is GitHub, starting read-only.
- Connector tools are explicit typed tools exposed dynamically by connected account, credential status, scopes, and tool config.
- Implementation should use small CI-green PR slices, starting with docs sync, runtime IDs, state migrations, typed state APIs, shared DTOs, model gateway, tool policy, previews/results, engine foundation, then approval pause/resume.
- First end-to-end MVP slice is CLI + `MockProvider` + `write_file` create/overwrite + approval pause/resume.
- `write_file` V1 supports only UTF-8 `create_new` and `overwrite`; preview is required, never mutates, and execution revalidates after approval.
- MVP approval/resume tests should use scripted `MockProvider` and a tool execution counter to prove `write_file` executes exactly once.
- CLI is the first approval UX proof surface, with interactive TTY approval plus `opennivara approvals ...` subcommands.
- Desktop renders backend `ApprovalView`; frontend must not invent approval transition logic.
- Telegram uses the same backend approval APIs with same-chat command-based approval UX and no special tool permission layer.
- Surface consistency requires Desktop, CLI, and Telegram to use the same backend allowed-action booleans and hide completed approvals by default.
- MVP completion requires happy path, denial, provider-failure continue, duplicate approval, and non-mutating preview proof for CLI + `MockProvider` + `write_file`.
- GitHub V1A is read-only (`github_list_repositories`, `github_fetch_issue`, `github_search_issues`, `github_fetch_pr`, `github_fetch_file`); GitHub V1B is low-risk issue creation/comment mutation with approval.
- New architecture should land through incremental module boundaries, not one large refactor.
- Test strategy focuses existing infrastructure on approval, recovery, tools, provider, and surface scenarios.
- Chat-visible events, durable approval audit rows, and local developer logs are separate observability layers.
- Memory is local-first, dynamic, and has no templates.
- Time and location context are deterministic, permissioned, and audited.
- SQLite remains the source of truth.
- Mintlify docs live in `docs-site/`; internal engineering docs remain in `docs/`.
- Public Mintlify site: [https://story-0890af7b.mintlify.app/](https://story-0890af7b.mintlify.app/).
