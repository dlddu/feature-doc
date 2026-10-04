-- 재요청의 시각은 이 표의 어느 열에도 없고, 열거된 열에서 유도되지도 않는다. 그 값을
-- 읽는 쪽이 생기면 열을 더하는 마이그레이션이 필요하다 — 이 표만 보고 재요청 시각을
-- 계산하려는 코드는 조용히 틀린 값을 만든다.
--
-- 소유자는 이 표에 적지 않는다. 요청을 받는 시점에 소유자를 해석하면 그 값이
-- 요청자의 경로 위에 놓이게 되고, 「소유자의 신원은 요청자의 화면·응답 어디에도
-- 나타나지 않는다」는 약속을 지키기 어려워진다.
CREATE TABLE access_requests (
    id                TEXT    PRIMARY KEY,
    analysis_id       TEXT    NOT NULL,
    requester_user_id TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at        INTEGER NOT NULL,
    UNIQUE(analysis_id, requester_user_id)
);

-- 통지가 서는 시점에 「이 분석에 걸린 요청들」을 고르는 축이다.
CREATE INDEX idx_access_requests_analysis ON access_requests(analysis_id);
