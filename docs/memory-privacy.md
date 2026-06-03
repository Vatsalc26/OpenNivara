# Memory Privacy

Memory privacy is enforced before retrieval context can enter a prompt.

Supported modes:

- `off`: no memory inclusion or saving.
- `ask_before_saving`: proposals wait for user review.
- `auto_save_low_risk`: low-risk memories may be saved automatically.
- `full_life_journal`: maximum local capture mode, still local-first.

Additional gates:

- `pause_memory`: stops memory activity.
- `private_chat`: prevents memory inclusion and saving.
- `allow_location_memories`: reserved for explicit location-memory consent.
- `sensitive_approval_required`: requires review for sensitive categories.

Sensitive handling is intentionally conservative. The first implementation provides category gates and review workflow rather than automatic cloud classification.
