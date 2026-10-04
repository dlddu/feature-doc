-- A partial re-run of one stage overwrites its own row rather than accumulating
-- history: the analysis *is* the unit of history, and keeping a second axis of
-- versioning here would make "which document do we show" ambiguous.
CREATE TABLE analysis_documents (
    id            TEXT    PRIMARY KEY,
    analysis_id   TEXT    NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    kind          TEXT    NOT NULL,
    content       TEXT    NOT NULL,
    content_hash  TEXT    NOT NULL,
    model         TEXT    NOT NULL,
    input_tokens  INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    created_at    INTEGER NOT NULL,
    UNIQUE(analysis_id, kind)
);
CREATE INDEX idx_analysis_documents_analysis ON analysis_documents(analysis_id);
