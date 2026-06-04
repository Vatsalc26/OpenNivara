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

## Regeneration

The generated exam-pack manifests live under `packs/builtin`. Run
`python scripts/generate_india_skill_packs.py` from the repository root to
regenerate the generated India exam packs.

India Student Essentials is curated by hand and is skipped by the generator by
default. Only overwrite it intentionally by setting
`OPENNIVARA_REGENERATE_CURATED_STUDENT_ESSENTIALS=1`.

After any India skill-pack change, run:

```bash
cargo run -- skillctl validate-pack india_student_essentials
cargo run -- skillctl eval india_student_essentials
```
