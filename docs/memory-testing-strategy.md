# Memory Testing Strategy

Memory tests isolate config storage with `OPENNIVARA_TEST_CONFIG_DIR` and a shared env mutex. This prevents tests from touching the real user memory database.

Current Rust coverage:

- migration idempotence and required schema objects.
- memory source/item CRUD plus FTS search.
- planned versus completed answerability.
- compiler exclusion for ordinary chat.
- compiler inclusion for relevant memory lookup.
- privacy-off prompt exclusion and audit notes.
- dynamic memory facets and facet-filtered retrieval.
- SQLite graph rebuild and consistency validation.
- runtime clock relative-date resolution.
- saved places and disabled exact-location privacy behavior.
- compiler runtime/location relevance decisions.
- generated TypeScript bindings remain current.

Recommended verification before shipping memory changes:

```powershell
cargo test memory:: -- --nocapture
cargo test bindings_are_current -- --nocapture
cargo check --manifest-path desktop\src-tauri\Cargo.toml
Set-Location desktop
bun run typecheck
bun run check
```

Feature flags should also be checked when vector or embedding code changes:

```powershell
cargo check --features memory-vector
cargo check --features local-embeddings
```
