# Architecture

OpenNivara is a Rust backend with a React/Tauri desktop frontend. Desktop is the primary app. Browser preview is only a fast UI development surface.

Settings owns assistant behavior. Store content is limited to visual themes. Memory is local-first and stored in SQLite under the user config directory.

Key foundations:

- Rust backend modules own memory, runtime, config, marketplace, and workspace behavior.
- Tauri commands expose typed desktop APIs.
- Specta exports frontend DTOs.
- React Query coordinates desktop UI data fetching.
- SQLite remains the source of truth for memory, with derived FTS, graph, and optional vector indexes.
