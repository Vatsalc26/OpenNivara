# OpenNivara

A local-first personal AI agent by Vatsal Chavda.

OpenNivara provides a desktop interface and CLI with personal context, persistent memory, skills, Telegram access and controlled local-tool integration. It is designed to work with private user data while keeping local state on the user's device by default.

OpenNivara is an alpha-stage project. The public release version is `v0.1.0-alpha`.

## Overview

OpenNivara combines a Rust backend, a Tauri desktop application, a CLI, SQLite-backed local state, configurable personal context, bundled skills and packs, Telegram access and controlled local tools. It is intended for users who want a personal assistant that can work with local context while keeping configuration, memories, sessions and runtime state local by default.

Public repository: [Vatsalc26/OpenNivara](https://github.com/Vatsalc26/OpenNivara)

## Features

- Desktop application built with Tauri and React.
- CLI executable named `opennivara`.
- Personal profiles, preferences, contexts and goals.
- Persistent memory, memory graph and retrieval functionality.
- Sessions and conversation history.
- Runtime and location context, including saved places.
- Skills and bundled first-party packs.
- Marketplace/store functionality for packs and themes.
- Telegram bot integration with allow-list authentication.
- Controlled local-file, workspace-map and other local tools.
- Intentional Telegram tool-argument logging for diagnostics.

## Installation Prerequisites

- Rust and Cargo from [rustup.rs](https://rustup.rs/).
- Bun for desktop and documentation tooling.
- A Gemini API key.
- Optional: a Telegram bot token if you want Telegram access.

## Environment Variables

Copy `.env.example` to `.env` and replace the placeholders:

```env
GEMINI_API_KEY=replace_this_with_your_key
TELEGRAM_BOT_TOKEN=replace_this_with_your_bot_token
```

Never commit `.env` or any real credential.

## CLI Usage

Initialize local configuration:

```bash
cargo run -- init-profile
cargo run -- init-preferences
cargo run -- init-style
cargo run -- init-tools
cargo run -- init-map
cargo run -- init-telegram
```

Ask a question:

```bash
cargo run -- ask "Summarize my active context."
```

Build the release binary:

```bash
cargo build --release
```

The executable is `target/release/opennivara` on Unix-like systems and `target/release/opennivara.exe` on Windows.

## Desktop App

Run the desktop app in development:

```bash
cd desktop
bun install --frozen-lockfile
bun run dev
```

Build the desktop frontend:

```bash
cd desktop
bun run build
```

## Local Private Data Storage

OpenNivara stores user-owned state locally, including TOML configuration files, SQLite databases and logs. These may contain private profile data, preferences, sessions, memories, saved locations, Telegram metadata, selected local file contents, tool arguments, prompts and queries.

OpenNivara uses a new local data namespace and does not automatically import local data from earlier private Jarvis development builds.

Treat all local state as sensitive. See [Privacy and Data Flow](docs/privacy-and-data-flow.md), [Security Model](docs/security-model.md) and [Known Limitations](docs/known-limitations.md).

## Security Notes

Interactive approval enforcement for some local-tool actions is under development. Until it is complete, users should keep remote high-risk local-tool permissions disabled unless they understand the risk.

Known limitation: allowed filesystem directories must not contain untrusted symbolic links. Canonical-path enforcement is planned for a future security hardening pass.

Planned hardening: move Gemini API-key transport from URL query parameters to request headers.

Telegram tool-execution logs may contain private context, including file paths, queries and tool arguments. Users must treat logs as sensitive local data and must never commit them to source control.

## Development And Testing

Rust checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
```

Desktop checks:

```bash
cd desktop
bun install --frozen-lockfile
bun run typecheck
bun run check
bun run test:run
bun run build
```

Documentation checks:

```bash
bun run docs:check
bun run docs:site:check
```

## License

OpenNivara is released under the MIT License. See [LICENSE](LICENSE).

First-party bundled packs are distributed under this repository's MIT License unless otherwise noted.

## Author

Vatsal Chavda
