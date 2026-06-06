CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    title TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    status TEXT NOT NULL,
    surface_created TEXT NOT NULL,
    actor_id_created TEXT,
    active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'tool', 'event', 'system')),
    surface TEXT NOT NULL,
    actor_id TEXT,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    metadata_json TEXT,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TABLE active_sessions (
    actor_id TEXT NOT NULL,
    surface TEXT NOT NULL,
    session_id TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY(actor_id, surface),
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TABLE session_pinned_contexts (
    session_id TEXT NOT NULL,
    context_id TEXT NOT NULL,
    pinned_at TEXT NOT NULL,
    PRIMARY KEY(session_id, context_id),
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TABLE session_pinned_skills (
    session_id TEXT NOT NULL,
    skill_id TEXT NOT NULL,
    pinned_at TEXT NOT NULL,
    PRIMARY KEY(session_id, skill_id),
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_messages_session_id ON messages(session_id);
CREATE INDEX idx_sessions_updated_at ON sessions(updated_at);
CREATE INDEX idx_active_sessions_session_id ON active_sessions(session_id);
