-- 두 테이블인 이유는 두 사실이 다르기 때문이다. `feature_dependency_requests` 는
-- **사람이 이 feature 의 의존성을 보자고 했다**는 사실과 그 실행 상태이고,
-- `feature_dependencies` 는 그 실행이 만든 **데이터**다. 요청이 먼저 남아야
-- 「분석 중」과 「분석했더니 아무것도 없었다」가 구분되고, 워커가 죽어도 무엇이
-- 남았는지 큐가 안다.
--
-- 왜 `analysis_documents` 에 한 덩어리로 넣지 않는가. 답해야 하는 것이
-- 「데이터 모델 X 를 사용하는 feature 전부」라는 **행 선택 질의**여서다. JSON blob 으로
-- 두면 그 질의가 모든 분석의 blob 을 파싱해야 답이 나온다 — 0007 이 후보를 행으로
-- 고른 것과 같은 이유다.
--
-- `feature_key` 는 `feature_candidates.key` 와 같은 값이다(발견 위치에서 파생된,
-- 분석을 가로지르는 feature 의 정체성). 외래키를 걸지 않는 이유는 그 키가
-- `(analysis_id, key)` 로만 유일하고 이 테이블도 같은 쌍을 들고 있어, 제약이
-- 더해 주는 것 없이 재추출 시 삭제 순서만 묶기 때문이다. 분석이 지워지면 두
-- 테이블 모두 `analyses` 의 ON DELETE CASCADE 로 함께 사라진다.
--
-- `category` 를 CHECK 로 못박는 이유는 이 목록이 화면의 분류 칩·프롬프트·역방향
-- 질의의 축을 동시에 이루기 때문이다 — 오타 하나가 조용히 분류를 하나 더 만들면
-- 「아래 열거가 전부」라는 주장이 거짓이 된다. 값의 단일 정의는 코드의 카테고리
-- 상수이고 이 제약은 그 두 번째 방어선이다.
--
-- `evidence` 가 NULL 인 것은 결함이 아니라 **기록된 사실**이다. 근거가 비어 있으면
-- 「근거 없음」으로 명시하고 임의로 채우지 않는다 — 지어내 채우는 것보다 비어 있는
-- 편이 옳다.
--
-- `model` 과 토큰 수를 요청 행에 남기는 것은 다른 단계가 `analysis_documents` 에
-- 남기는 것과 같은 습관이다.
-- 상태는 `queued` 에서 시작해 `succeeded` 또는 `failed` 로 끝난다 — 중간 상태를
-- 두지 않는 이유는 claim 이 분석 단위로 배타적이라, 리스가 끊긴 요청이 `queued` 로
-- 남아 다음 claim 에 그대로 다시 제안되는 편이 회수 경로가 짧기 때문이다.
CREATE TABLE feature_dependency_requests (
    id            TEXT    PRIMARY KEY,
    analysis_id   TEXT    NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    feature_key   TEXT    NOT NULL,
    status        TEXT    NOT NULL DEFAULT 'queued',
    error         TEXT,
    model         TEXT,
    input_tokens  INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    requested_at  INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    UNIQUE(analysis_id, feature_key),
    CHECK (status IN ('queued', 'succeeded', 'failed')),
    CHECK (status <> 'failed' OR error IS NOT NULL)
);

CREATE INDEX idx_feature_dependency_requests_analysis
    ON feature_dependency_requests(analysis_id, status);

CREATE TABLE feature_dependencies (
    id          TEXT    PRIMARY KEY,
    analysis_id TEXT    NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    feature_key TEXT    NOT NULL,
    seq         INTEGER NOT NULL,
    category    TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    evidence    TEXT,
    created_at  INTEGER NOT NULL,
    UNIQUE(analysis_id, feature_key, category, name),
    CHECK (category IN (
        'infrastructure', 'data', 'architecture',
        'framework', 'middleware', 'logic', 'interface'
    ))
);

CREATE INDEX idx_feature_dependencies_feature
    ON feature_dependencies(analysis_id, feature_key);

-- 역방향 질의(「이 데이터 모델을 쓰는 feature 전부」)가 고르는 축.
CREATE INDEX idx_feature_dependencies_reverse
    ON feature_dependencies(category, name);
