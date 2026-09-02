-- Discovery strategy review and approval (AC1.3).
--
-- Stage 3 writes its *generated* strategy to `analysis_documents` like every other
-- stage output, so reproducibility (content_hash) keeps working the same way. This
-- table is the other half AC1.3 asks for: the copy the **user** edits and approves.
--
-- They are deliberately two rows in two tables. The document is what the model
-- produced for this analysis and must not change when a person deletes a line;
-- the strategy below is what the pipeline will actually be told to scan. Keeping
-- them separate is what lets S06 show "이건 제안이고, 승인한 건 이것"과 재시도가
-- 사용자의 편집을 지우지 않는 것을 동시에 성립시킨다.
--
-- `entries` is a JSON array of `{ "pattern": …, "source": "generated" | "user" }`.
-- JSON rather than a child table because the whole list is always read and written
-- as one unit (the screen PUTs the list it is showing), and because `source` is the
-- only structure a query would ever filter on — carrying a user's own entries into
-- the next analysis of the same target, which is the one query that needs it.
--
-- `approved_at IS NULL` means draft. AC1.3's "승인된 전략만 다음 단계의 입력이
-- 된다" is enforced against this column in worker_api::claim, which withholds the
-- next stage from the queue until it is set.
CREATE TABLE discovery_strategies (
    analysis_id TEXT    PRIMARY KEY REFERENCES analyses(id) ON DELETE CASCADE,
    entries     TEXT    NOT NULL,
    approved_at INTEGER,
    updated_at  INTEGER NOT NULL
);
