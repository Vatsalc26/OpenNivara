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
