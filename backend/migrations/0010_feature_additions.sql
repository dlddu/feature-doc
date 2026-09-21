-- 자동 추출이 놓친 feature 를 사람이 한 문장으로 적으면, 모델이 저장소 트리 안에서
-- 근거를 찾아 인수 기준 초안과 의존성 후보를 내고, 사람이 확정한 것만 feature 가 되는
-- 흐름의 저장소.
--
-- 한 행이 추가 시도 하나의 일생이다 — 무엇을 적었고(`request`), 모델이 근거를
-- 찾았는지(`evidence_found`), 무엇을 초안으로 냈고(`draft_json`), 사람이 그것을
-- 확정했는지 취소했는지(`status`). 근거를 찾지 못한 시도도 행으로 남는다: 그때
-- 사람은 「근거 없음」을 단 채로 추가하거나 취소할 수 있고, 어느 쪽이었는지가
-- 기록이기 때문이다.
--
-- 왜 `feature_candidates` 에 행을 더하지 않는가. 그 표의 행은 파이프라인 4단계가
-- 낸 **자동 후보**이고, 승인이 5단계(자동 인수 기준 생성)를 다시 큐에 넣는 계약을
-- 진다. 사람이 직접 더한 feature 는 이미 초안을 들고 오므로 그 단계를 타지 않아야
-- 하고, 같은 표에 섞이면 「자동으로 발견된 것」과 「사람이 더한 것」의 출처가
-- 행의 모양으로 구분되지 않는다.
--
-- 왜 인수 문서(`analysis_documents`)를 직접 고치지 않는가. 그 행의 `content_hash` 는
-- 재분석 재현성과 두 분석의 비교가 함께 읽는 값이다 — 사람이 feature 를 더했다는
-- 이유로 「재분석 결과가 달라졌다」가 되면 두 판정이 동시에 거짓이 된다. 확정된
-- 추가는 문서를 내보내는 시점에 그 끝에 겹쳐 읽는다.
--
-- `key` 는 이 feature 의 정체성이다. 자동 후보의 키가 발견 위치에서 파생되는 것과
-- 달리 사람이 더한 feature 는 위치가 없을 수 있으므로(근거 없음) 행 고유값에서
-- 파생한다. 자동 후보의 키 공간과 겹치지 않도록 접두사를 둔다.
--
-- `source` 는 그 feature 를 누가 만들었는가다. 어휘는 편집 이력의 표와 같다 —
-- 모델이 근거를 찾아 초안을 냈으면 사람이 모델의 도움을 받은 것이고, 근거 없이
-- 사람이 그대로 더했으면 사람이 직접 한 것이다. 확정 전에는 아직 어느 쪽도 아니므로
-- NULL 이고, CHECK 가 확정된 행에만 값을 요구한다.
CREATE TABLE feature_additions (
    id             TEXT    PRIMARY KEY,
    analysis_id    TEXT    NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    key            TEXT    NOT NULL,
    name           TEXT    NOT NULL,
    request        TEXT    NOT NULL,
    evidence_found INTEGER NOT NULL DEFAULT 0,
    draft_json     TEXT    NOT NULL,
    status         TEXT    NOT NULL DEFAULT 'drafted',
    source         TEXT,
    model          TEXT,
    input_tokens   INTEGER NOT NULL DEFAULT 0,
    output_tokens  INTEGER NOT NULL DEFAULT 0,
    created_at     INTEGER NOT NULL,
    decided_at     INTEGER,
    UNIQUE(analysis_id, key),
    CHECK (evidence_found IN (0, 1)),
    CHECK (status IN ('drafted', 'confirmed', 'cancelled')),
    CHECK (source IS NULL OR source IN ('auto', 'user_llm', 'user_direct')),
    CHECK (status <> 'confirmed' OR source IS NOT NULL),
    CHECK (status = 'drafted' OR decided_at IS NOT NULL)
);

-- 겹쳐 읽기(확정분)와 확정될 목록의 수가 이 축으로 고른다.
CREATE INDEX idx_feature_additions_analysis
    ON feature_additions(analysis_id, status);
