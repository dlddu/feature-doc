CREATE TABLE analyses (
    id              TEXT    PRIMARY KEY,
    user_id         TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    installation_id INTEGER NOT NULL,
    repo_owner      TEXT    NOT NULL,
    repo_name       TEXT    NOT NULL,
    branch          TEXT    NOT NULL,
    status          TEXT    NOT NULL DEFAULT 'queued',
    -- Pre-flight estimates shown before triggering: display only, never a hard cap.
    est_llm_calls   INTEGER NOT NULL,
    est_cost_cents  INTEGER NOT NULL,
    created_at      INTEGER NOT NULL
);
CREATE INDEX idx_analyses_user ON analyses(user_id);
