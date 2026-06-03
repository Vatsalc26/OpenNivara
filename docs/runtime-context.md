# Runtime Context

`src/runtime/clock.rs` builds `RuntimeContext` from the current UTC time, a requested timezone, and a `LocationContext`.

The compiler uses runtime context only when the prompt needs time grounding: relative dates, task planning, memory lookup, routes, location questions, reminders, or tool workflows. Normal chat skips runtime context.

Runtime context includes:

- UTC and local timestamps.
- local date, weekday, timezone, and calendar week.
- deterministic relative ranges for today, tomorrow, yesterday, this week, next week, this month, and next month.
- the current `LocationContext`, which still has separate prompt-inclusion privacy gates.
