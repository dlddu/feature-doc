-- 되돌린 행도 지우지 않는다: 「지웠다가 되돌렸다」는 그 자체가 이력이고, 같은
-- feature 를 다시 지우면 새 행이 선다.
--
-- 왜 인수 문서(`analysis_documents`)에서 feature 를 빼지 않는가. 그 행의 `content_hash`
-- 는 재분석 재현성과 두 분석의 비교가 함께 읽는 값이다(0009 · 0010 의 주석과 같은 이유).
-- 자동 산출물은 그대로 두고, 열린 삭제 행이 있는 feature 를 문서를 내보내는 시점에
-- 걸러 낸다. 그래서 「즉시 영구 제거되지 않는다」는 저장 방식의 성질이다 — 문서에는
-- 여전히 있고, 읽는 자리가 가릴 뿐이다.
--
-- `restore_until` 을 행에 적는 이유는 「일정 기간」이 지운 시점의 사실이어야 하기
-- 때문이다 — 나중에 기간 설정이 바뀌어도 이미 지운 것의 기한은 그때 약속한 값이다.
--
-- `reason` 은 비어 있을 수 있다. 후보 거부(0007)와 달리 요구가 「사유를 함께 기록할 수
-- 있고」라 사유는 선택이다 — 있으면 재발견 표시에 함께 실린다.
CREATE TABLE feature_deletions (
    id            TEXT    PRIMARY KEY,
    analysis_id   TEXT    NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    feature_key   TEXT    NOT NULL,
    name          TEXT    NOT NULL,
    reason        TEXT,
    deleted_at    INTEGER NOT NULL,
    restore_until INTEGER NOT NULL,
    restored_at   INTEGER,
    CHECK (restore_until > deleted_at),
    CHECK (restored_at IS NULL OR restored_at >= deleted_at)
);

-- 겹쳐 읽기(가림)와 보관소 목록이 이 축으로 고른다.
CREATE INDEX idx_feature_deletions_analysis
    ON feature_deletions(analysis_id, restored_at);

CREATE UNIQUE INDEX idx_feature_deletions_open
    ON feature_deletions(analysis_id, feature_key)
    WHERE restored_at IS NULL;

-- 재발견 표시는 같은 대상의 앞선 분석들에서 이 키의 열린 삭제를 찾는다.
CREATE INDEX idx_feature_deletions_key
    ON feature_deletions(feature_key, restored_at);
