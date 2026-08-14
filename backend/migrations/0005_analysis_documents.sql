-- Pipeline stage outputs (AC1.2~AC1.4): the documents the analysis produces.
--
-- One row per (analysis, kind) — the latest content for that document on that
-- analysis. A stage re-run (AC1.5's partial retry) overwrites its own row rather
-- than accumulating history: the analysis *is* the unit of history, and keeping a
-- second axis of versioning here would make "which document does S05 show" ambiguous.
--
-- `content_hash` is what makes AC1.2's determinism clause observable. Re-analyzing
-- the same repository writes a new analysis with its own row; comparing this hash
-- against the previous analysis's row for the same target answers "did the result
-- reproduce, or did it change" without diffing the documents in the client.
--
-- `model`, `input_tokens`, `output_tokens` are the per-call cost accounting AC4.6
-- eventually surfaces. Recorded now so the numbers exist when the screen for them
-- lands; nothing reads them yet.
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
