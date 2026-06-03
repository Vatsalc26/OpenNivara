# Privacy And Data Flow

OpenNivara intentionally processes private personal information so it can act as a local-first personal AI agent.

Sensitive data can include profiles, preferences, contexts, goals, sessions, conversation history, memories, memory graph data, saved locations, Telegram messages, selected local file contents, prompts, queries and tool arguments.

## Local State

OpenNivara stores local state in TOML configuration files, SQLite databases and logs under the OpenNivara local data namespace. Users must treat these files as sensitive local data. OpenNivara does not claim local-state encryption unless encryption is explicitly implemented in the future.

OpenNivara uses a new local data namespace and does not automatically import local data from earlier private Jarvis development builds.

Users can locate local state with commands such as:

```bash
opennivara profile-path
opennivara preferences-path
opennivara style-path
opennivara contexts-path
opennivara telegram-path
opennivara state-db-path
opennivara memory-db-path
```

To delete local state, quit OpenNivara and remove the relevant files or the OpenNivara config directory returned by these commands.

## Gemini Data Flow

When answering a request, OpenNivara may send selected context to Gemini. Selected context can include profile fields, preferences, style instructions, contexts, goals, skill instructions, memory retrieval results, runtime context, location context, conversation history and selected local tool results.

Telegram-based requests pass through Telegram. A Telegram request may cause selected OpenNivara context to be sent to Gemini.

## Logs

Telegram tool-execution logs may contain private context, including file paths, queries and tool arguments. Users must treat logs as sensitive local data and must never commit them to source control.
