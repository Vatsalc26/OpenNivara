# Contributing

Thank you for helping improve OpenNivara.

## Development Setup

Install Rust, Cargo and Bun. Then install desktop dependencies:

```bash
cd desktop
bun install --frozen-lockfile
```

## Rust Checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
```

## Desktop Checks

```bash
cd desktop
bun run typecheck
bun run check
bun run test:run
bun run build
```

## Documentation

Keep public documentation honest about current behavior. Do not claim encryption, complete approval enforcement, symlink escape protection or Gemini header-based API-key transport until those features are implemented and verified.

Run documentation checks when changing docs:

```bash
bun run docs:check
bun run docs:site:check
```

## Privacy Rule

Never commit `.env`, personal TOML state, SQLite databases, logs, generated artifacts or any files containing private profile data, Telegram configuration, memory data, saved locations, prompts, queries, tool arguments or credentials.

## Security Issues

Report security issues privately to Vatsal Chavda rather than through public issues.
