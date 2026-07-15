-- Analysis jobs (AC1.1): an explicit, user-triggered request to analyze one
-- repository at one branch. This slice persists the *enqueue* only — a row lands in
-- status 'queued' after the target is confirmed within the App's granted access.
-- Draining the queue (the async worker) and per-stage progress arrive with the
-- analysis pipeline (AC1.5 / AC4.5, later slices). Owned by a user and always
-- queried under that scope (AC4.7).
CREATE TABLE analyses (
    id              TEXT    PRIMARY KEY,
    user_id         TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    installation_id INTEGER NOT NULL,
    repo_owner      TEXT    NOT NULL,
    repo_name       TEXT    NOT NULL,
    branch          TEXT    NOT NULL,
    status          TEXT    NOT NULL DEFAULT 'queued',
    -- Pre-flight estimates shown to the user before triggering (display only,
    -- never a hard cap). cost is stored in integer cents to avoid float columns.
    est_llm_calls   INTEGER NOT NULL,
    est_cost_cents  INTEGER NOT NULL,
    created_at      INTEGER NOT NULL
);
CREATE INDEX idx_analyses_user ON analyses(user_id);
