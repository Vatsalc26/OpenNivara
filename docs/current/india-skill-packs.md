# India Skill Packs

OpenNivara ships five built-in India education skill packs:

- India Student Essentials: 8 skills
- India Engineering Exams: 19 skills
- India Medical Exams: 12 skills
- India UPSC CSE: 10 skills
- India Management and Law Exams: 13 skills

Together they provide 62 disabled-by-default skills for common Indian study and
exam workflows including JEE Main, JEE Advanced, GATE, NEET UG, NEET PG, UPSC
CSE, CAT, CLAT, and general student planning.

## Student Essentials

India Student Essentials is the curated general-study pack for common student
workflows that do not require current official data:

- weekly and multi-week study plans
- daily timetables and routine resets
- active-recall revision cycles
- mock-test mistake analysis
- weak-topic diagnosis
- notes-to-recall conversion from supplied text
- concept and doubt explanation
- no-shame exam stress and consistency support

All eight skills are data-only and tool-free (`tools.allow = []`). They must not
read local files, write files, open URLs, open apps, run commands, promise marks
or ranks, or invent official contacts. Notes and doubt skills are
`suggest_only`; the user selects them manually because they depend on supplied
content or a clear concept question.

## Fresh Information

Administrative workflows such as applications, admit cards, counselling, official
notices, and current affairs are marked freshness-sensitive. Those skills include
official source labels so OpenNivara can tell the user when current official data is
needed.

## Direct Curation

Built-in skill manifests are maintained directly in their pack directories under
`packs/builtin`. Do not use a generator to rewrite skill manifests.

Pack upgrades are performed individually with prompt review, routing review,
freshness/source-boundary review, and deterministic evaluation fixtures where the
pack has been upgraded for them.

India Student Essentials is the first curated high-coverage pack. Other built-in
India packs may still contain alpha-quality content awaiting individual upgrade;
India Engineering Exams is the next planned pack for comprehensive review.

After changing a built-in India skill pack, run:

```bash
cargo run -- skillctl validate-pack <pack_id>
cargo run -- skillctl report <pack_id>
```

For packs with deterministic fixtures, also run `cargo run -- skillctl eval
<pack_id>`.
