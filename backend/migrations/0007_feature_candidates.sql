-- Feature candidate extraction and review.
--
-- Stage 4 writes its *generated* candidate list to `analysis_documents` like every
-- other stage output, so reproducibility (content_hash) keeps working the same way.
-- This table is the other half: the copy the **user** decides on —
-- 승인 / 거부(+사유) / 병합 / 이름 변경.
--
-- Row per candidate, not one JSON blob like `discovery_strategies` (0006). The two
-- shapes answer two different questions. A strategy is always read and written as
-- one list (the client PUTs what it is showing), so JSON is the honest unit there.
-- A candidate is decided **one at a time**, and the carry-over query — "did the
-- reviewer reject this same candidate in an earlier analysis of the same target?"
-- — selects *rows* by `(repo, branch, key)`. Encoding that as JSON
-- would mean parsing every earlier analysis's blob to answer it.
--
-- `key` is the candidate's identity **across analyses**: derived from where it was
-- found (path + symbol), not from a row id, because the next analysis of the same
-- repository produces new rows for the same places. That derivation is
-- `feature_candidates::candidate_key` — one place, so the extractor and the
-- carry-over query cannot disagree about what "the same candidate" means.
--
-- `decision` is `undecided` | `approved` | `rejected`. `reject_reason` is NOT NULL
-- whenever `decision = 'rejected'` — a rejection must record its reason, and
-- the CHECK is what keeps a reason-less rejection from ever reaching the table
-- (the route rejects it first; this is the second line).
--
-- `merged_into` points at the surviving candidate's `key` when the reviewer merged
-- this one into another. The row is kept rather than deleted so the merge is
-- visible and reversible, and so a later analysis can still see that this place was
-- looked at.
CREATE TABLE feature_candidates (
    id            TEXT    PRIMARY KEY,
    analysis_id   TEXT    NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    key           TEXT    NOT NULL,
    seq           INTEGER NOT NULL,
    name          TEXT    NOT NULL,
    location      TEXT    NOT NULL,
    symbol        TEXT,
    rationale     TEXT    NOT NULL,
    decision      TEXT    NOT NULL DEFAULT 'undecided',
    reject_reason TEXT,
    merged_into   TEXT,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    UNIQUE(analysis_id, key),
    CHECK (decision IN ('undecided', 'approved', 'rejected')),
    CHECK (decision <> 'rejected' OR reject_reason IS NOT NULL)
);

CREATE INDEX idx_feature_candidates_analysis ON feature_candidates(analysis_id);

-- The carry-over query looks up every earlier rejection of the same
-- key, then filters to the analyses of this repository+branch.
CREATE INDEX idx_feature_candidates_key ON feature_candidates(key, decision);
