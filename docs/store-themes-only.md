# Store Themes-Only Contract

OpenNivara Store v1 is a visual theme store only.

## Scope

- Store items may change UI colors, density, glow, and other visual appearance tokens.
- Store items must be data-only `theme.toml` manifests.
- Store items must not change assistant identity, response style, preferences, contexts, prompt assembly, tools, shell permissions, network behavior, or command snippets.
- Settings remains the only place where assistant behavior is edited.

## Backend Boundary

- Theme install copies only `theme.toml` into the local theme store.
- Applying a theme writes `marketplace/appearance.toml`.
- Prompt assembly must read Settings files only. It may expose an active theme preview for UI diagnostics, but the active theme is not sent to the model.
- Adding a new Store item type requires an explicit product decision and a prompt-impact model before implementation starts.

## Frontend Boundary

- Store routes expose `Themes`, `Installed Themes`, and theme detail views.
- Store UI must not show pack, add-on, mode, quick prompt, response style, preference, context, command snippet, or workspace rule controls.
- Context Inspector may show the active visual theme only as UI-only metadata.
