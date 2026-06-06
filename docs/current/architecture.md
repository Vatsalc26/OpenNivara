# Architecture

OpenNivara is a local-first personal AI agent with three equal user surfaces: Desktop, CLI, and Telegram. All three surfaces must normalize requests into the same engine lifecycle and use the same tool policy, approval system, and audit model. The implementation contract is defined in [Core Agent Contract](../architecture/core-agent-contract.md). Stable request and turn envelopes are defined in [Request And Turn Envelopes](../architecture/request-turn-envelopes.md). Prompt assembly is defined in [Prompt Context Assembly](../architecture/prompt-context-assembly.md). Tool result boundaries are defined in [Tool Result Schema](../architecture/tool-result-schema.md) and [Model-Visible Tool Results](../architecture/model-visible-tool-results.md). The same-turn approval persistence design is defined in [Approval Resume And State DB](../architecture/approval-resume-state.md), and the crash-safe lifecycle is defined in [Recovery State Machine](../architecture/recovery-state-machine.md). The first end-to-end implementation target is [MVP Vertical Slice](../architecture/mvp-vertical-slice.md), with `write_file` semantics in [write_file V1](../architecture/write-file-v1.md) and deterministic tests in [MockProvider Test Harness](../architecture/mock-provider-test-harness.md). Memory proposal and explicit memory tool behavior is defined in [Memory Proposals And Tools](../architecture/memory-proposals-and-tools.md), [Memory Tool Boundaries](../architecture/memory-tool-boundaries.md), [Memory Retention Semantics](../architecture/memory-retention-semantics.md), and [Memory Hard-Delete Cleanup Scope](../architecture/memory-hard-delete-cleanup-scope.md). Connector/account/credential foundation and external-operation policy are defined in [Connectors, Accounts, And Credentials](../architecture/connectors-accounts-credentials.md), [External Operations Policy](../architecture/external-operations-policy.md), [Connector Tool Registry](../architecture/connector-tool-registry.md), [First Connector Scope](../architecture/first-connector-scope.md), and [GitHub Connector V1](../architecture/github-connector-v1.md). Module boundaries and sequencing are defined in [Module Boundaries](../architecture/module-boundaries.md) and [Implementation Roadmap](../architecture/implementation-roadmap.md).

Desktop is the React/Tauri app surface. Browser preview is only a fast UI development surface and is not a separate agent surface.

Settings owns assistant behavior. Store content is limited to visual themes. Memory is local-first and stored in SQLite under the user config directory.

Key foundations:

- Rust backend modules own memory, runtime, config, marketplace, and workspace behavior.
- Desktop, CLI, and Telegram adapters authenticate actors and render responses, then delegate request handling to the shared Rust engine.
- Tauri commands expose typed desktop APIs without bypassing shared engine policy.
- Specta exports frontend DTOs.
- React Query coordinates desktop UI data fetching.
- SQLite remains the source of truth for memory, with derived FTS, graph, and optional vector indexes.
