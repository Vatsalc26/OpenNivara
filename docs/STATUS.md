# Docs Status

Current as of 2026-06-03.

The canonical current docs are linked from [README.md](README.md). Root-level legacy docs are retained as historical references until they are moved to `docs/archive/` or `docs/stale/`.

Product decisions:

- Desktop/Tauri is the primary app.
- Browser preview is for fast React iteration only.
- Store discovers themes and skill packs; Settings owns assistant behavior.
- Settings owns assistant behavior.
- Memory is local-first, dynamic, and has no templates.
- Time and location context are deterministic, permissioned, and audited.
- SQLite remains the source of truth.
- Mintlify docs live in `docs-site/`; internal engineering docs remain in `docs/`.
- Public Mintlify site: [https://story-0890af7b.mintlify.app/](https://story-0890af7b.mintlify.app/).
