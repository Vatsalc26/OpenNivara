# Security Model

OpenNivara is local-first software. User configuration, memories, sessions, workspace-map data and runtime state are intended to remain on the user's device by default.

## Trust Model

Users are trusted to configure local tools, workspace roots, Telegram access and model credentials carefully. Gemini and Telegram are external services; any context sent to them leaves the local device.

## Telegram Access

Telegram access uses allow-list authentication. Users must add trusted chat IDs before remote requests are accepted.

Remote Telegram access to high-risk local tools must remain disabled by default unless the user deliberately enables it and understands the risk.

## Tool Permissions

OpenNivara includes controlled local tools and workspace-map tools. Tool availability is constrained by local configuration, source of request and skill/tool policy.

Interactive approval enforcement for some local-tool actions is under development. Until it is complete, users should keep remote high-risk local-tool permissions disabled unless they understand the risk.

## Logging

Telegram tool arguments are intentionally logged for diagnostics and observability.

Telegram tool-execution logs may contain private context, including file paths, queries and tool arguments. Users must treat logs as sensitive local data and must never commit them to source control.

## Known Limitations

Known limitation: allowed filesystem directories must not contain untrusted symbolic links. Canonical-path enforcement is planned for a future security hardening pass.

Planned hardening: move Gemini API-key transport from URL query parameters to request headers.

## Operational Guidance

- Keep `.env`, Telegram configuration, TOML state, SQLite databases and logs private.
- Keep high-risk local tools disabled for Telegram unless needed.
- Do not enable local-file tools on directories containing untrusted symbolic links.
- Review logs before sharing diagnostics.
