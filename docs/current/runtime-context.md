# Runtime Context

Runtime time is not memory. It is available internally even when memory is off.

`RuntimeContext` includes UTC/local time, timezone, local date, weekday, calendar week, relative date ranges, location context, and model context info.

`ClockService` behavior:

- Build runtime context from UTC, timezone, location, and model defaults.
- Resolve today, tomorrow, yesterday, this week, next week, this month, and next month.
- Fall back to UTC when no trusted timezone is available.

Runtime is included in prompts only when useful: relative date questions, memory lookup, task planning, reminders, routes, location questions, and tool workflows.
