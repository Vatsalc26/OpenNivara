# Testing Strategy

OpenNivara uses layered tests so browser-preview behavior cannot hide desktop regressions.

## Layers

- Rust unit/property tests: backend validation, config path isolation, trigger scoring, marketplace safety, and file persistence.
- Vitest unit/component tests: API schema contracts, Tauri client command mapping, UI state, and editor behavior.
- Storybook stories: stable visual states with mocked Tauri commands and deterministic data.
- Playwright E2E: browser-preview routing, accessibility, and smoke coverage.
- Tauri WebDriver E2E: real WebView2 desktop app, Rust-backed settings, Store, theme, Context Inspector, and chat bridge.

## Store Theme Regression Tests

Store regression tests must assert the themes-only contract:

- Store lists visual themes and installed themes.
- Theme details show visual metadata and data-only safety badges.
- Store UI does not expose behavior packs, add-ons, modes, quick prompts, command snippets, preferences, contexts, response style, or tool permissions.
- Installing or applying a theme does not change prompt assembly. Prompt previews must continue to come from Settings-owned profile, style, preferences, and contexts only.

## Failure Artifacts

- Playwright artifacts live in `desktop/test-results` and Playwright output folders.
- Tauri WebDriver artifacts live in `desktop/test-results/tauri-e2e`.
- Failed Tauri tests save screenshots and print the isolated config directory. Use `OPENNIVARA_KEEP_TAURI_E2E_CONFIG=1` when you need to inspect generated TOML files after a run.

## Contract Rule

Every new desktop feature should have at least one test at the lowest useful layer and one user-facing smoke path when it changes visible behavior. Prefer schema and command-contract tests for data boundaries, then add E2E only for workflows users actually perform.

## Current Memory Gates

Use Bun for desktop JavaScript commands:

```powershell
cargo test
cargo check --manifest-path desktop\src-tauri\Cargo.toml
Set-Location desktop
bun run typecheck
bun run check
bun run test:run
bun run build
```
