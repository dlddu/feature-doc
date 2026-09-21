-- 사람이 한 줄로 부탁하면 모델이 고쳐 쓴 문장을 보여 주고, 사람이 승인한 것만 문서에
-- 반영하는 흐름의 저장소. 변경마다 그것을 누가 만들었는지도 함께 남긴다.
--
-- 한 행이 제안 하나의 일생이다 — 무엇을 고쳐 달라 했고(`request`), 모델이 무엇을
-- 내놓았고(`after_json`), 그 자리에 무엇이 있었고(`before_json`), 사람이 그것을
-- 승인했는지 거부했는지(`status`). 제안과 적용을 두 테이블로 나누지 않은 이유는
-- 거부된 제안도 보존해야 하는 기록이기 때문이다: 요구가 「거부된 제안은 기록되어 다음
-- 제안 시 회피된다」이므로 거부는 버려지는 실패가 아니라 다음 호출의 입력이다. 두
-- 테이블이면 그 입력을 두 곳에서 모아야 한다.
--
-- 왜 `analysis_documents` 의 행을 직접 고치지 않는가. 그 행의 `content_hash` 는 같은
-- 저장소를 다시 분석했을 때 결과가 재현됐는지 판정하는 쪽과, 두 분석을 견주어 달라진
-- 것을 내놓는 쪽이 함께 읽는 값이다 — 사람이 문장을 다듬었다는 이유로 「재분석 결과가
-- 달라졌다」가 되면 두 판정이 동시에 거짓이 된다. 자동 산출물은 자동 산출물대로 두고,
-- 사람이 승인한 변경은 문서를 내보내는 시점에 그 위로 겹쳐 읽는다. 자동 결과와 사용자
-- 편집이 같은 자리에서 갈릴 때 어느 쪽을 택할지는 사람에게 묻기로 한 문제이고, 그때
-- 묻는 쪽이 이 표를 입력으로 삼는다.
--
-- `scenario_index` 는 그 feature 문서 안에서의 자리(0-based)다. 시나리오에 안정
-- 식별자가 없기 때문인데, 자리로 잡아도 되는 이유는 겹쳐 읽기가 같은 분석 안에서만
-- 일어나고 한 분석의 자동 문서는 그 뒤로 다시 쓰이지 않기 때문이다. 분석을 가로지르는
-- 정체성은 `feature_key` 가 잇는다.
--
-- `source` 는 그 변경을 누가 만들었는가다. 어휘가 셋인 이유는 자동 분석이 쓴 것,
-- 사람이 모델의 도움을 받아 고친 것, 사람이 직접 고친 것이 **같은 이력의 다른 값**이지
-- 서로 다른 표가 아니기 때문이다. CHECK 가 그 어휘를 못박는다.
--
-- `reason` 은 거부 사유다. 거부에만 있으므로 CHECK 로 상태와 묶는다 — 사유 없는
-- 거부는 다음 제안이 무엇을 피해야 하는지 말해 주지 못해, 기록으로서 값이 없다.
CREATE TABLE feature_doc_edits (
    id             TEXT    PRIMARY KEY,
    analysis_id    TEXT    NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    feature_key    TEXT    NOT NULL,
    scenario_index INTEGER NOT NULL,
    request        TEXT    NOT NULL,
    before_json    TEXT    NOT NULL,
    after_json     TEXT    NOT NULL,
    status         TEXT    NOT NULL DEFAULT 'proposed',
    source         TEXT    NOT NULL DEFAULT 'user_llm',
    reason         TEXT,
    model          TEXT,
    input_tokens   INTEGER NOT NULL DEFAULT 0,
    output_tokens  INTEGER NOT NULL DEFAULT 0,
    created_at     INTEGER NOT NULL,
    decided_at     INTEGER,
    CHECK (status IN ('proposed', 'approved', 'rejected')),
    CHECK (source IN ('auto', 'user_llm', 'user_direct')),
    CHECK (status = 'rejected' OR reason IS NULL),
    CHECK (status <> 'rejected' OR reason IS NOT NULL),
    CHECK (status = 'proposed' OR decided_at IS NOT NULL)
);

-- 겹쳐 읽기(승인분)와 회피 컨텍스트(거부분) 둘 다 이 축으로 고른다.
CREATE INDEX idx_feature_doc_edits_feature
    ON feature_doc_edits(analysis_id, feature_key, status);
