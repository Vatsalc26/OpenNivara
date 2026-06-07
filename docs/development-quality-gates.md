# Development Quality Gates Guide

Welcome to the OpenNivara development environment. To maintain code quality, prevent regressions, and ensure that Settings-owned assistant behavior and Store-owned visual themes remain fully functional, we enforce a strict quality pipeline.

This document explains the locally-executable quality gates and validation steps required before pushing code or submitting pull requests.

---

## 1. Quality Gates Summary

Every change must satisfy the following checks:
1. **Rust Backend Consistency**: Code is formatted correctly, has no compiler warnings/lints, and passes all backend tests.
2. **React Frontend Compliance**: No code style issues (Biome), zero TypeScript compilation errors (`tsc`), passes Vitest unit tests under JSDom, and compiles to a production bundle cleanly.
3. **E2E Stability**: Browser smoke tests verify routing and accessibility. Tauri WebDriver tests verify the packaged desktop shell, Rust-backed settings, Store, Context Inspector, theme activation, and chat bridge.
4. **Storybook Coverage**: Stories cover real app components with mocked Tauri commands and build successfully.

---

## 2. Rust Backend Quality Checks

Run these commands from the root directory:

### A. Formatting Checks
Keep code clean and formatted according to Rust style conventions:
```bash
cargo fmt --all -- --check
```
*To auto-format files:* `cargo fmt --all`

### B. Clippy Lints
We treat lints as compiler warnings and fail on any active violations:
```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### C. Backend Unit and Integration Tests
Verify settings persistence, theme-store directories, data-only theme installation, prompt assembly boundaries, and migrations:
```bash
cargo test --workspace
```

---

## 3. React Frontend Quality Checks

Install desktop dependencies and start the real Tauri desktop app from the repo root:

```bash
bun install --cwd desktop
bun run --cwd desktop tauri:dev
```

The equivalent workflow from inside `desktop/` is:

```bash
bun install
bun run tauri:dev
```

On Windows, if Bun prints many `Failed to link ... EUNKNOWN` errors during install, stop any running desktop dev server, delete `desktop/node_modules`, and retry:

```powershell
Remove-Item -Recurse -Force desktop\node_modules
bun install --cwd desktop
```

Run the following frontend quality commands from the `desktop/` directory:

### A. Code Style and Lints
We use **Biome** for lightning-fast linting and formatting verification:
```bash
bun run check
```
*To auto-fix lints and format files:* `bun run format`

### B. TypeScript Compilation Check
Ensure zero type errors exist across all tsx/ts modules:
```bash
bun run typecheck
```

### C. Frontend Unit and Integration Tests
Verify component states, details modals, settings sidebar tab-switching, and dynamic theme applications:
```bash
bun run test:run --pool=forks
```
*To run tests interactively:* `bun run test`

### D. Build Production Bundle
Ensure that standard Vite asset bundles package cleanly without bundler warnings:
```bash
bun run build
```

### E. Storybook Build
Verify component stories compile with the same app providers and mocks used by tests:
```bash
bun run storybook:build
```

---

## 4. End-to-End (E2E) Browser Tests

We use **Playwright** to run browser-level integration smoke tests.

To execute the E2E suite locally:
```bash
bun run e2e
```
*To view tests executing interactively:* `bun run e2e:ui`

---

## 5. Tauri Desktop WebDriver Tests

The real desktop suite uses `tauri-driver`, EdgeDriver, and WebdriverIO. It builds the Tauri debug binary and launches the actual WebView2 app.

Prerequisites on Windows:

```powershell
cargo install tauri-driver --locked
```

EdgeDriver must match the installed Microsoft Edge version. In this repo, a local driver can be placed at `desktop/.tools/msedgedriver.exe`, or you can set `EDGE_DRIVER_PATH`.

Run the smoke test:

```bash
cd desktop
bun run tauri:e2e:smoke
```

Run the full desktop suite:

```bash
cd desktop
bun run tauri:e2e
```

Failure artifacts are written to `desktop/test-results/tauri-e2e`. Set `OPENNIVARA_KEEP_TAURI_E2E_CONFIG=1` to preserve the isolated test config directory printed on failure.

---

## 6. Pre-Commit Verification Script

For a one-stop validation of the entire frontend codebase, run the unified quality command inside the `desktop/` directory:
```bash
bun run quality
```
This executes typecheck, Biome checks, Vitest coverage, Knip, and standard Vite compilation in sequence. Run Storybook, Playwright, and Tauri E2E separately when a change touches UI surfaces or desktop integration.
