CREATE TABLE pending_approvals (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    request_id TEXT NOT NULL,
    turn_id TEXT NOT NULL,
    user_message_id TEXT NOT NULL,
    tool_call_id TEXT NOT NULL,
    surface TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    operation_name TEXT NOT NULL,
    classification TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'denied', 'executing', 'executed', 'failed', 'completed')),
    summary TEXT,
    operation_target TEXT,
    reason TEXT,
    arguments_preview_json TEXT,
    result_summary TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL,
    resolved_at TEXT,
    resolved_by_actor_id TEXT,
    execution_started_at TEXT,
    execution_finished_at TEXT,
    completed_at TEXT,
    resume_attempt_count INTEGER NOT NULL DEFAULT 0,
    last_resume_error TEXT,
    last_resume_attempt_at TEXT,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY(user_message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE TABLE pending_turns (
    approval_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    request_id TEXT NOT NULL,
    turn_id TEXT NOT NULL,
    user_message_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    phase TEXT NOT NULL CHECK(phase IN ('awaiting_approval', 'tool_executed_awaiting_model', 'denied_awaiting_model')),
    resume_payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(approval_id) REFERENCES pending_approvals(id) ON DELETE CASCADE,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY(user_message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX idx_pending_approvals_session_status ON pending_approvals(session_id, status);
CREATE INDEX idx_pending_approvals_actor_status ON pending_approvals(actor_id, status);
CREATE INDEX idx_pending_approvals_request_id ON pending_approvals(request_id);
CREATE INDEX idx_pending_approvals_turn_id ON pending_approvals(turn_id);
CREATE INDEX idx_pending_approvals_user_message_id ON pending_approvals(user_message_id);
CREATE INDEX idx_pending_approvals_operation_target ON pending_approvals(operation_target);
CREATE INDEX idx_pending_turns_session ON pending_turns(session_id);
CREATE INDEX idx_pending_turns_request_id ON pending_turns(request_id);
CREATE INDEX idx_pending_turns_turn_id ON pending_turns(turn_id);
CREATE INDEX idx_pending_turns_phase ON pending_turns(phase);
