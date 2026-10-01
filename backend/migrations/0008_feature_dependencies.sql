-- 두 테이블인 이유는 두 사실이 다르기 때문이다. `feature_dependency_requests` 는
-- **사람이 이 feature 의 의존성을 보자고 했다**는 사실과 그 실행 상태이고,
-- `feature_dependencies` 는 그 실행이 만든 **데이터**다. 요청이 먼저 남아야
-- 「분석 중」과 「분석했더니 아무것도 없었다」가 구분되고, 워커가 죽어도 무엇이
-- 남았는지 큐가 안다.
--
-- `feature_key` 는 `feature_candidates.key` 와 같은 값이다(발견 위치에서 파생된,
-- 분석을 가로지르는 feature 의 정체성). 외래키를 걸지 않는 이유는 그 키가
-- `(analysis_id, key)` 로만 유일하고 이 테이블도 같은 쌍을 들고 있어, 제약이
-- 더해 주는 것 없이 재추출 시 삭제 순서만 묶기 때문이다.
--
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
