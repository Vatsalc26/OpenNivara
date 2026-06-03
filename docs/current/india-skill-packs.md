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

## Fresh Information

Administrative workflows such as applications, admit cards, counselling, official
notices, and current affairs are marked freshness-sensitive. Those skills include
official source labels so OpenNivara can tell the user when current official data is
needed.

## Regeneration

The generated pack manifests live under `packs/builtin`. Run
`python scripts/generate_india_skill_packs.py` from the repository root to
regenerate them.
