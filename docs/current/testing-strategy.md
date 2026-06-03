# Testing Strategy

OpenNivara uses layered validation:

- Rust tests for backend behavior, memory, runtime, graph, and config isolation.
- Tauri `cargo check` and Tauri E2E for desktop command coverage.
- Bun typecheck, Biome, Vitest, Knip, and Vite build for the desktop React app.
- Playwright and WebDriver tests for visible workflows.
- Docs lint and link checks for governance.

Browser preview tests cannot be the only proof for Rust-backed features. Memory, runtime, location, and SQLite behavior need isolated config tests and Tauri validation.
