-- 복원은 문서를 다시 쓰는 일이 아니라 **재생 구간을 자르는 일**이고, 이 표의 한 행은
-- 그 자르는 지점 하나다.
--
-- 왜 보상 편집(되돌릴 문장을 0009 의 새 승인 행으로 써 넣기)이 아닌가. 그러면 자동
-- 분석이 쓴 문장과 똑같은 문장이 「사용자가 고친 것」으로 이력에 서게 되어, AC3.4 가
-- 요구하는 바로 그 출처 어휘(자동 / 사용자 by LLM / 사용자 직접)가 거짓이 된다.
-- 되돌리기는 문장을 새로 쓰는 행위가 아니라 **어느 시점을 고르는 행위**다.
--
-- 왜 그 시점의 문서를 행에 적어 두지 않는가. 0009 가 `analysis_documents` 를 고치지
-- 않는 것과 같은 이유다 — 재생으로 도출되는 값을 따로 저장하면 두 값이 갈라질 수 있고,
-- 갈라진 순간 어느 쪽이 사실인지 말해 줄 것이 없다. 자른 지점만 적고 상태는 매번
-- 재생으로 얻는다.
--
-- `seq` 는 그 feature 안에서 이 복원이 몇 번째인가다. 시각이 아니라 순번으로 잇는
-- 이유는 재생 순서가 초 단위 시각에 기대면 안 되기 때문이다 — 복원과 그 직후의 편집이
-- 같은 초에 들어오면 시각만으로는 둘의 앞뒤를 가릴 수 없고, 그 한 번의 모호함이
-- 「복원 뒤에 한 편집이 조용히 사라진다」로 나타난다.
--
-- 편집 쪽에도 같은 이유로 한 칸이 붙는다(`after_restore`) — 그 편집이 **어느 복원
-- 뒤에** 선 것인지. NULL 이면 그 feature 에 아직 복원이 없던 때의 편집이다. 이 두 칸이
-- 편집과 복원을 하나의 순서로 엮어, 재생이 시각을 읽지 않아도 되게 한다.
CREATE TABLE feature_doc_restores (
    id          TEXT    PRIMARY KEY,
    analysis_id TEXT    NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    feature_key TEXT    NOT NULL,
    seq         INTEGER NOT NULL,
    target_kind TEXT    NOT NULL,
    target_id   TEXT,
    created_at  INTEGER NOT NULL,
    UNIQUE(analysis_id, feature_key, seq),
    CHECK (seq > 0),
    CHECK (target_kind IN ('auto', 'edit', 'restore')),
    CHECK ((target_kind = 'auto') = (target_id IS NULL))
);

-- 재생도 이력 화면도 이 축으로 고른다.
CREATE INDEX idx_feature_doc_restores_feature
    ON feature_doc_restores(analysis_id, feature_key, seq);

ALTER TABLE feature_doc_edits ADD COLUMN after_restore TEXT REFERENCES feature_doc_restores(id);
