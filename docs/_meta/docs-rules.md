# Docs Rules

- `docs/README.md` is the canonical internal docs index.
- A doc is current only when linked from the Current section of `docs/README.md`.
- ADRs record product decisions that future work must preserve.
- Stale or conflicting docs must be updated, moved to `docs/stale/`, moved to `docs/archive/`, or listed in the stale register.
- Do not delete historical docs unless they are clearly junk.
- `docs/` is internal engineering documentation.
- `docs-site/` is public-facing Mintlify documentation.
- Do not put app or desktop dependencies in the docs tooling path.
- Do not commit Mintlify tokens, deployment credentials, or secrets.
