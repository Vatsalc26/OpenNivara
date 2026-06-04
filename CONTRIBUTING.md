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

For faster local Rust test feedback, install and use nextest:

```bash
cargo install cargo-nextest --locked
cargo nextest run
```

For coverage work, install and run cargo-llvm-cov:

```bash
cargo install cargo-llvm-cov --locked
cargo llvm-cov --workspace --summary-only
```

## Desktop Checks

```bash
cd desktop
bun run typecheck
bun run check
bun run test:run
bun run build
```

## Skill Pack Authoring

Built-in Skills v1 packs are data-only. Installing a pack must not enable it, and
skills must not grant executable behavior or mutate tool permissions.

When changing `packs/builtin/india_student_essentials`, run:

```bash
cargo run -- skillctl validate-pack india_student_essentials
cargo run -- skillctl eval india_student_essentials
cargo run -- skillctl report india_student_essentials
cargo run -- skillctl schema
git diff --exit-code -- schemas
```

Use `.taplo.toml` with a Taplo-compatible editor or CLI to lint pack, skill, and
eval TOML against `schemas/`. The Student Essentials pack is curated by hand; the
generator skips it unless `OPENNIVARA_REGENERATE_CURATED_STUDENT_ESSENTIALS=1`
is set.

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
