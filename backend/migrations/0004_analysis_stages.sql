-- Worker workload separation (AC4.5): the queue gains a lease so a *separate*
-- worker process can claim work, and each analysis gains the per-stage rows the
-- S04 pipeline is made of.
--
-- The worker never opens this database. It claims and reports over the API's
-- `/internal/*` routes, so the API stays the single writer and the SQLite
-- invariant in db.rs (one pod, one mount, `Recreate`) survives untouched.

-- Lease + lifecycle columns. `claimed_by` is a worker identity (pod name), kept
-- for operator visibility; `lease_expires_at` is what makes a dead worker's job
-- reclaimable by the next claim.
ALTER TABLE analyses ADD COLUMN claimed_by       TEXT;
ALTER TABLE analyses ADD COLUMN claimed_at       INTEGER;
ALTER TABLE analyses ADD COLUMN lease_expires_at INTEGER;
ALTER TABLE analyses ADD COLUMN started_at       INTEGER;
ALTER TABLE analyses ADD COLUMN finished_at      INTEGER;
ALTER TABLE analyses ADD COLUMN error            TEXT;

-- The claim selector scans by (status, created_at) — FIFO within the queue.
CREATE INDEX idx_analyses_queue ON analyses(status, created_at);

-- One row per pipeline stage, seeded at enqueue from pipeline::STAGES (the code
-- SSOT mirroring docs/mockups/s04-analysis-progress.html). Only stage 1 (`fetch`)
-- executes today; stages 2-5 stay 'pending' until the LLM pipeline lands
-- (AC1.2~1.4). `detail` carries the operator/user-facing one-liner the S04 step
-- renders ("847 files · 2.3 MB").
CREATE TABLE analysis_stages (
    id           TEXT    PRIMARY KEY,
    analysis_id  TEXT    NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    seq          INTEGER NOT NULL,
    key          TEXT    NOT NULL,
    title        TEXT    NOT NULL,
    status       TEXT    NOT NULL DEFAULT 'pending',
    detail       TEXT,
    error        TEXT,
    started_at   INTEGER,
    finished_at  INTEGER,
    UNIQUE(analysis_id, seq)
);
CREATE INDEX idx_analysis_stages_analysis ON analysis_stages(analysis_id);
