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
- New architecture should land through incremental module boundaries, not one large refactor.
- Test strategy focuses existing infrastructure on approval, recovery, tools, provider, and surface scenarios.
- Chat-visible events, durable approval audit rows, and local developer logs are separate observability layers.
- Memory is local-first, dynamic, and has no templates.
- Time and location context are deterministic, permissioned, and audited.
- SQLite remains the source of truth.
- Mintlify docs live in `docs-site/`; internal engineering docs remain in `docs/`.
- Public Mintlify site: [https://story-0890af7b.mintlify.app/](https://story-0890af7b.mintlify.app/).
