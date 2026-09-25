-- 한 feature 문서의 변경 이력에서 **임의 시점으로 되돌리는** 흐름의 저장소(AC3.4).
--
-- 지금 사람이 보는 문서는 저장된 값이 아니라 재생의 결과다 — 자동 문서(0005) 위에
-- 확정된 추가(0010)가 겹치고, 열린 삭제(0011)가 가리고, 승인 편집(0009)이 만들어진
-- 순서대로 얹힌다. 그러면 「임의 시점 T 의 상태」는 그 재생을 T 에서 끊은 것과 같다.
-- 그래서 복원은 문서를 다시 쓰는 일이 아니라 **재생 구간을 자르는 일**이고, 이 표의
-- 한 행은 그 자르는 지점 하나다.
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
-- `target_kind`/`target_id` 는 사람이 고른 이력 항목이다. `auto` 는 편집이 하나도
-- 얹히지 않은 자동 기준선(그때는 `target_id` 가 NULL), `edit` 은 0009 의 승인 행,
-- `restore` 는 이 표의 앞선 행이다. 복원 자체가 이력의 한 항목이므로 그것을 다시
-- 고를 수 있어야 한다 — 여정 `JRN-restore-history` 의 「복원 자체가 이력에 남으므로
-- 다시 되돌리기 가능」이 산문이 아니라 저장 방식의 성질이 되는 자리다.
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
