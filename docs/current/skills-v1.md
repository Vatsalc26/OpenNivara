# Skills v1

OpenNivara skills are data-only routing profiles. A skill adds metadata, trigger terms,
prompt guidance, safety limits, and Store preview copy. Installing a pack copies
the skill manifests into the local marketplace; it does not enable the skill or
change assistant behavior.

## Product Boundary

- Store discovers and installs skill packs.
- Settings enables, disables, filters, and inspects installed skills.
- Skill manifests cannot grant runtime tools by default.
- Freshness-sensitive skills must label their official sources and require fresh
  information in the safety metadata.

## Manifest Preview Fields

`metadata` captures India-first exam routing fields such as country, exam, exam
stage, freshness sensitivity, source labels, and review date.

`store_preview` captures user-facing inspection fields: best use cases, sample
prompts, what the skill will do, and what it will not do.

## Safety Defaults

Built-in education packs use an empty allow-list and deny executable or external
actions such as `write_file`, `run_command`, and `open_url`. These deny entries
are reserved policy labels, not runtime tools.

## Authoring Workflow

Use `skillctl` from the repository root when creating or changing built-in skill
packs:

```bash
cargo run -- skillctl validate-pack india_student_essentials
cargo run -- skillctl eval india_student_essentials
cargo run -- skillctl report india_student_essentials
cargo run -- skillctl schema
```

`validate-pack` checks pack and skill manifests. `eval` runs deterministic
routing, prompt-contract, collision, language-variant, and safety fixtures.
`schema` regenerates JSON schemas under `schemas/`; commit schema changes only
when the Rust manifest structs changed.

The repo also includes `.taplo.toml` schema associations for pack manifests,
skill manifests, and eval fixtures. Use a Taplo-compatible editor or CLI to
lint TOML against the generated schemas during authoring.

## Routing Policies

`auto` skills can be selected from message content when their score meets the
skill's `min_score`. `suggest_only` skills can appear as candidates but are not
chosen automatically; the user must select them manually. `manual_only` skills
are selected only through explicit command/UI/session pin state.

Desktop chat supports a one-message manual skill selection and a session pin.
Pinned skills are session-local state and do not enable unavailable skills.
