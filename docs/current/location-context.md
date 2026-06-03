# Location Context

Location context is permissioned and freshness-aware. Exact location must never be hidden tracking.

Location sources include manual observations, saved places, OS location, coarse IP hints, map queries, and unknown.

Prompt rules:

- Normal chat does not include location.
- Denied or disabled location does not leak.
- Stale location must not be called current.
- Route, travel, local task, weather/location, nearby place, and leave-time planning prompts can include compact location context if allowed.

Saved places provide label, city, region, country, timezone, and optional coordinates without requiring live tracking.
