-- `carried_from` 은 편집 행이 어느 분석의 어느 편집을 이어받은 것인지다. 이월된 행이
-- 원본과 같은 표에 서는 이유는 겹쳐 읽기(0009)가 분석 안에서만 일어나기 때문이다 —
-- 사람이 A 에서 승인한 문장이 B 의 문서에도 서려면 B 에도 행이 있어야 하고, 그 행이
-- 「이어받은 것」임을 잃지 않아야 이력이 한 편집을 두 번 센 것처럼 보이지 않는다.
--
-- 충돌 행은 갈린 두 문장을 **그 시점의 값으로** 들고 있다 — `mine_json` 은 사람이 승인한
-- 문장(들), `before_json` 은 그 편집이 고쳐 쓴 당시의 자동 문장, `auto_json` 은 이번
-- 자동 결과. 셋 다 행에 적는 이유는 사람이 결정하는 시점에 어느 쪽 분석의 문서가 다시
-- 쓰였어도 화면이 「그때 무엇과 무엇이 부딪혔는가」를 그대로 보여 주기 위해서다.
ALTER TABLE feature_doc_edits ADD COLUMN carried_from TEXT REFERENCES feature_doc_edits(id);

CREATE TABLE feature_doc_conflicts (
    id                   TEXT    PRIMARY KEY,
    analysis_id          TEXT    NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    previous_analysis_id TEXT    NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    edit_id              TEXT    NOT NULL REFERENCES feature_doc_edits(id) ON DELETE CASCADE,
    feature_key          TEXT    NOT NULL,
    scenario_index       INTEGER NOT NULL,
    request              TEXT    NOT NULL,
    source               TEXT    NOT NULL,
    mine_json            TEXT    NOT NULL,
    before_json          TEXT    NOT NULL,
    auto_json            TEXT    NOT NULL,
    mine_decided_at      INTEGER NOT NULL,
    status               TEXT    NOT NULL DEFAULT 'open',
    merge_edit_id        TEXT    REFERENCES feature_doc_edits(id) ON DELETE SET NULL,
    created_at           INTEGER NOT NULL,
    decided_at           INTEGER,
    CHECK (status IN ('open', 'auto', 'mine', 'merged')),
    CHECK (source IN ('auto', 'user_llm', 'user_direct')),
    CHECK (status = 'open' OR decided_at IS NOT NULL),
    CHECK (status <> 'merged' OR merge_edit_id IS NOT NULL)
);

-- 배너 집계·충돌 화면 목록이 이 축으로 고른다.
CREATE INDEX idx_feature_doc_conflicts_analysis
    ON feature_doc_conflicts(analysis_id, status, created_at);

CREATE UNIQUE INDEX idx_feature_doc_conflicts_open
    ON feature_doc_conflicts(analysis_id, feature_key, scenario_index)
    WHERE status = 'open';
