# ADR 0001: Store Themes Only

Status: accepted.

The Store is limited to visual themes. Themes never alter prompts, tools, preferences, contexts, memory behavior, or assistant personality.

Settings owns assistant behavior. Any feature that changes prompt assembly must live in Settings or explicit runtime/compiler code, not Store content.
