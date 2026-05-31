-- FeatureDoc S02 (Repositories home) schema.
-- A repository the user has connected for analysis. Owned by a user and queried
-- under that scope (AC4.7). Timestamps are unix epoch seconds (INTEGER); ids are
-- opaque TEXT (uuid v4), matching 0001.
--
-- The analysis-derived columns (status/feature_count/conflict_count/spend_cents/
-- progress/step/last_analyzed_at) are reserved here but not yet produced by any
-- pipeline: connect leaves them NULL / 'not_analyzed' so the home screen renders
-- the empty ("Not analyzed") state until the analysis slice lands.
CREATE TABLE repositories (
    id               TEXT    PRIMARY KEY,
    user_id          TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    owner            TEXT    NOT NULL,
    name             TEXT    NOT NULL,
    branch           TEXT    NOT NULL,
    status           TEXT    NOT NULL DEFAULT 'not_analyzed',
    feature_count    INTEGER,
    conflict_count   INTEGER,
    spend_cents      INTEGER,
    progress         INTEGER,
    step             TEXT,
    last_analyzed_at INTEGER,
    created_at       INTEGER NOT NULL,
    UNIQUE(user_id, owner, name)
);
CREATE INDEX idx_repositories_user ON repositories(user_id);
