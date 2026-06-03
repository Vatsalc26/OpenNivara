# Feature Implementation Contract

New features must preserve these contracts:

- Desktop/Tauri remains primary.
- Store remains safe and non-executing. It may list/install data-only Skill Packs, but installing a pack must not activate prompt behavior.
- Settings -> Skills owns assistant behavior activation and configuration.
- Skill Packs are data-only in v1: no executable plugin runtime, WASM runtime, native scripts, remote skill execution, or arbitrary code execution.
- Skills can expose only safe allowlists of existing Tool Registry tools and must not modify tool permissions directly.
- Memory remains local-first and dynamic.
- No memory templates or role templates.
- Time and location context are deterministic, permissioned, and audited.
- SQLite remains the memory source of truth.
- Dangerous tool permissions and hidden tracking are not allowed.

Every feature should include the lowest useful automated test and a user-facing smoke path when it changes visible behavior.
