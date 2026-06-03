# Memory Correction Model

Memory items are corrected rather than silently overwritten.

Correction paths:

- `retract_memory_item`: marks an item `retracted` and removes it from active retrieval.
- `supersede_memory`: marks the old item retracted and links it to a replacement.
- `memory_corrections`: records old/new memory IDs, correction type, reason, and source.

Retrieval maps inactive or contradicted states to answerability labels:

- `completed`: `confirmed`.
- `planned` or `active`: `planned_only`.
- `uncertain`: `ambiguous`.
- `retracted`, `cancelled`, `missed`: `contradicted`.

This prevents planned intent, completed action, and corrected facts from collapsing into a single misleading memory.
