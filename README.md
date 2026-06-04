# OpenNivara

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

OpenNivara is an alpha-stage, desktop-first personal AI agent with a Rust CLI, local state, memory, skills, marketplace packs, and controlled local tools.

OpenNivara is currently `v0.1.0-alpha`. Expect rough edges, incomplete hardening, and changing interfaces.

## What OpenNivara Is

OpenNivara is a local-first personal AI agent. The main product surface is a Tauri + React desktop app backed by a Rust backend and SQLite/TOML local state. The `opennivara` CLI is an additional surface for setup, diagnostics, chat, workspace indexing, marketplace operations, skills, sessions, and Telegram configuration.

Local-first does not mean that data never leaves your device. OpenNivara uses the Gemini API for model responses, and optional Telegram integration routes messages through Telegram. Selected context can be sent to Gemini when answering a request.

## Features

- Desktop app built with Tauri and React.
- Rust CLI binary named `opennivara`.
- Personal profile, preferences, response style, reusable contexts, and goals.
- SQLite-backed sessions, conversation history, memory, and memory graph data.
- Workspace map and controlled local tools.
- Optional Telegram bot integration with allow-list authentication.
- Marketplace themes for visual customization only.
- Data-only skill packs that can be previewed and installed from Store, then enabled or disabled in Settings -> Skills.

## Safety Notice

OpenNivara is alpha software that can process sensitive local data. Local state may include private profile data, preferences, sessions, memories, saved locations, selected file contents, prompts, tool arguments, Telegram metadata, and logs.

Important current limitations:

- Interactive approval enforcement for some local-tool actions is still under development.
- Canonical-path and symlink hardening for allowed filesystem roots is planned, not complete.
- Moving Gemini API-key transport from URL query parameters to request headers is planned hardening, not complete.
- Telegram tool-execution logs may contain sensitive data.

Read the repository docs for [privacy and data flow](docs/privacy-and-data-flow.md), [security model](docs/security-model.md), and [known limitations](docs/known-limitations.md) before using OpenNivara with private data.

## Prerequisites

- Rust and Cargo from [rustup.rs](https://rustup.rs/).
- Bun for desktop, frontend, and documentation scripts.
- A Gemini API key.
- Optional: a Telegram bot token for Telegram access.
- Platform prerequisites for Tauri 2 development. See the official [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/).

## Environment

Copy `.env.example` to `.env` and replace the placeholders:

```env
GEMINI_API_KEY=replace_this_with_your_key
TELEGRAM_BOT_TOKEN=replace_this_with_your_bot_token
```

Never commit `.env`, local TOML state, SQLite databases, logs, generated artifacts, or files containing private prompts, memory data, tool arguments, or credentials.

## Desktop Quickstart

Run the actual desktop application in development:

```bash
cd desktop
bun install --frozen-lockfile
bun run tauri:dev
```

`bun run dev` starts Vite for browser/frontend iteration only. It does not run the full desktop app and should not be used to validate Rust-backed desktop behavior.

For a production frontend build:

```bash
cd desktop
bun run build
```

## CLI Quickstart

Initialize the basic local configuration:

```bash
cargo run -- init-profile
cargo run -- init-preferences
cargo run -- init-style
cargo run -- init-tools
```

Ask one question through the CLI:

```bash
cargo run -- ask "Summarize my active context."
```

Build the release binary:

```bash
cargo build --release
```

The executable is `target/release/opennivara` on Unix-like systems and `target/release/opennivara.exe` on Windows. See the public docs source in [docs-site/cli/commands.mdx](docs-site/cli/commands.mdx) for a compact command reference.

## Store, Themes, And Skills

Themes are visual only. They must not change prompts, memory, preferences, contexts, tools, or assistant behavior.

Store can preview and install data-only skill packs. Installing a skill pack does not activate prompt behavior. Settings -> Skills controls enabling, disabling, and inspecting skill behavior. Skill packs do not add executable code, install-time activation, network tools, or tool-permission changes.

## Data And Privacy Summary

OpenNivara stores user-owned state locally in TOML files, SQLite databases, and logs under the OpenNivara local data namespace. Local state is not claimed to be encrypted.

When answering a request, OpenNivara may send selected context to Gemini. Selected context can include profile fields, preferences, style instructions, contexts, goals, skill instructions, memory retrieval results, runtime context, location context, conversation history, and selected local tool results. Telegram-based requests pass through Telegram and may cause selected context to be sent to Gemini.

## Development And Testing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and quality expectations.

Common checks:

```bash
cargo fmt --all -- --check
cargo test
cd desktop
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

## Documentation, Support, Security, And License

- Public docs source: [docs-site/](docs-site/).
- Bugs and feature requests: [GitHub Issues](https://github.com/Vatsalc26/OpenNivara/issues).
- Vulnerability reporting: read [SECURITY.md](SECURITY.md). Do not open public issues for sensitive vulnerabilities. The current verified private contact is [@choco_chip2m on X](https://x.com/choco_chip2m).
- License: [MIT](LICENSE).

First-party bundled packs are distributed under this repository's MIT License unless otherwise noted.
