# Contributing

Thank you for helping improve OpenNivara.

Please read the [Code of Conduct](CODE_OF_CONDUCT.md) before participating.
Security vulnerabilities must be reported through the private path in
[SECURITY.md](SECURITY.md), not through public issues.

## Development Setup

Install Rust, Cargo and Bun. Then install desktop dependencies:

```bash
cd desktop
bun install --frozen-lockfile
```

## Rust Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

For faster local Rust test feedback, install and use nextest:

```bash
cargo install cargo-nextest --locked
cargo nextest run --workspace
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

Built-in skills are reviewed and edited directly in their pack directories under
`packs/builtin/`. Do not use a generator to rewrite skill manifests. Skills are
upgraded pack by pack so prompt boundaries, routing behavior, freshness labels,
and Store preview copy can be reviewed deliberately.

India Student Essentials is the first extensively curated and evaluated pack.
Other built-in India packs may still contain alpha-quality content awaiting
individual upgrade.

When changing a built-in pack, run validation for that pack:

```bash
cargo run -- skillctl validate-pack <pack_id>
cargo run -- skillctl report <pack_id>
cargo run -- skillctl schema
git diff --exit-code -- schemas
```

For packs with deterministic evaluation fixtures, also run:

```bash
cargo run -- skillctl eval <pack_id>
```

For Student Essentials specifically:

```bash
cargo run -- skillctl validate-pack india_student_essentials
cargo run -- skillctl eval india_student_essentials
cargo run -- skillctl report india_student_essentials
```

Use `.taplo.toml` with a Taplo-compatible editor or CLI to lint pack, skill, and
eval TOML against `schemas/`.

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

Report security issues privately through GitHub private vulnerability reporting
as described in [SECURITY.md](SECURITY.md). Do not open public issues for
vulnerabilities.
