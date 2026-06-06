# Feature Implementation Contract

This document defines the minimum contract for building OpenNivara desktop features without regressing existing chat, settings, store, and theme behavior.

---

## Before A Big Feature

Define these contracts before implementation starts:

1. Data model contract.
2. Backend command contract.
3. Frontend API schema.
4. UI routes, pages, and components.
5. Unit and component tests.
6. E2E smoke test.
7. Storybook stories for visible states.
8. Migration behavior.
9. Manual or automated Tauri smoke checklist.

Do not add a large user-facing feature until its regression surface is described here or in the feature spec.

---

## Architecture Rules

- Keep the current UI stack: React, TypeScript, Vite, Tauri, Tailwind, shadcn/ui, Radix primitives, TanStack Query, Zod, React Hook Form, cmdk, sonner, and lucide-react.
- Do not add Redux or a second UI component framework.
- Do not add a remote marketplace, marketplace SaaS framework, plugin runtime, WASM runtime, executable packs, or dangerous tool permissions.
- Store remains safe and non-executing. It may list and install data-only Skill Packs, but installation must not activate prompt behavior.
- Assistant behavior belongs in Settings -> Skills. Store items must not directly affect prompt assembly, tools, modes, response style, preferences, contexts, command snippets, or workspace rules.
- Skill Packs are installable bundles; Skills are toggleable behavior units inside packs. Installed skills are inactive until explicitly enabled in Settings -> Skills.
- Skill Packs must be data-only in v1. Do not add executable plugin runtimes, WASM runtimes, native scripts, Python/JavaScript skill runtimes, remote skill execution, or arbitrary code execution.
- Skills may request allowlists of existing safe tools through the Tool Registry, but no skill may modify tool permissions directly.
- Use TanStack Router for app navigation instead of growing local `activeView` switches.
- Use Zustand only for local UI state. Do not store Tauri/backend data there.
- Use XState for fragile async workflows that can loop, race, or time out.
- Keep browser preview fixtures explicit and separated from the real Tauri bridge.

---

## Required Gates

Run these before marking a feature complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

```bash
cd desktop
bun run typecheck
bun run check
bun run test:coverage
bun run e2e
bun run storybook:build
bun run tauri:e2e:smoke
bun run build
bun run knip
bun run quality
```

`bun run quality` currently executes `typecheck`, `check`, `test:coverage`, `knip`, and `build`.

---

## Coverage Standards

Initial Vitest thresholds:

- Statements: 70%.
- Branches: 60%.
- Functions: 70%.
- Lines: 70%.

Future targets:

- 85%+ general coverage.
- 90%+ coverage for critical modules: settings, marketplace/store, Context Inspector, theme, API clients, and app shell.

Tests must assert behavior and contracts. Do not fake coverage with shallow snapshot-only tests.

---

## Accessibility Standards

Playwright axe checks should cover the important user surfaces:

- Chat.
- Settings.
- Store and Store Details.
- Context Inspector.

The current acceptance bar is no critical axe violations. Serious color-contrast findings should be fixed as the design palette is tightened.

---

## Manual Tauri Smoke

Run `cd desktop && bun run tauri:dev` for a manual desktop smoke before release or before marking a major feature fully complete.

Checklist:

1. Chat opens.
2. Sending a test message works.
3. Context Inspector opens.
4. Prompt `hello.` finishes and does not loop.
5. Settings -> Preferences shows strength controls.
6. Preferences can edit likes, dislikes, notes, triggers, and `min_score`.
7. Settings -> Response Style shows all style groups.
8. Settings -> Appearance applies a theme visibly.
9. Store opens.
10. Store Details shows visual theme metadata and safety badges.
11. Store does not show behavior-pack, add-on, or quick-prompt controls.
12. Settings does not show behavior-pack Store tabs.
13. Browser preview warning does not appear in real Tauri.
14. Safe Shell/API status appears once.

If a manual smoke step fails, do not mark the feature complete.

---

## Storybook Standards

Stories must cover meaningful UI states, not generated placeholder demos. At minimum, keep stories for:

- App shell, Sidebar, and TitleBar.
- Chat and Context Inspector.
- Settings profile/style/preferences/appearance surfaces.
- Store themes/details/installed-theme states.
- Primitive UI components used across those surfaces.

Stories that depend on Tauri commands must use explicit Storybook mocks so they never call the real desktop bridge.
