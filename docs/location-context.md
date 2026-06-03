# Location Context

Location context is local-first and opt-in. Saved places live in `saved_places`; current observations live in `location_observations`.

Rules:

- The compiler does not include location for ordinary chat.
- Exact or approximate current location is included only when permission state is `granted`, privacy level is not `disabled`, and the request is location/route relevant.
- When exact location is not allowed, `get_location_context(..., false)` returns a disabled context with `permission_state = denied`.
- Saved places can provide city/timezone hints without live tracking.

The Store remains themes-only. Location behavior is controlled by Settings and compiler relevance decisions, not Store packs.
