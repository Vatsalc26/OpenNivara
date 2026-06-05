# Architecture

OpenNivara is a local-first personal AI agent with three equal user surfaces: Desktop, CLI, and Telegram. All three surfaces must normalize requests into the same engine lifecycle and use the same tool policy, approval system, and audit model. The implementation contract is defined in [Core Agent Contract](../architecture/core-agent-contract.md). The same-turn approval persistence design is defined in [Approval Resume And State DB](../architecture/approval-resume-state.md).

Desktop is the React/Tauri app surface. Browser preview is only a fast UI development surface and is not a separate agent surface.

Settings owns assistant behavior. Store content is limited to visual themes. Memory is local-first and stored in SQLite under the user config directory.

Key foundations:

- Rust backend modules own memory, runtime, config, marketplace, and workspace behavior.
- Desktop, CLI, and Telegram adapters authenticate actors and render responses, then delegate request handling to the shared Rust engine.
- Tauri commands expose typed desktop APIs without bypassing shared engine policy.
- Specta exports frontend DTOs.
- React Query coordinates desktop UI data fetching.
- SQLite remains the source of truth for memory, with derived FTS, graph, and optional vector indexes.
