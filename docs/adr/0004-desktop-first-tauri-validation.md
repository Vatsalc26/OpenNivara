# ADR 0004: Desktop-First Tauri Validation

Status: accepted.

Desktop/Tauri is the product. Browser/Vite preview is useful for fast UI iteration, but Rust-backed memory, runtime, location, and config features require desktop validation.

Tauri E2E smoke coverage must protect the Memory route, Settings privacy controls, graph status, saved places, and prompt audit behavior.
