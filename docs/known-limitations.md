# Known Limitations

OpenNivara `v0.1.0-alpha` is an early public release.

Interactive approval enforcement for some local-tool actions is under development. Until it is complete, users should keep remote high-risk local-tool permissions disabled unless they understand the risk.

Known limitation: allowed filesystem directories must not contain untrusted symbolic links. Canonical-path enforcement is planned for a future security hardening pass.

Planned hardening: move Gemini API-key transport from URL query parameters to request headers.

Telegram tool-execution logs may contain private context, including file paths, queries and tool arguments. Users must treat logs as sensitive local data and must never commit them to source control.

OpenNivara uses a new local data namespace and does not automatically import local data from earlier private Jarvis development builds.
