-- LLM 이 쓰는 산출물 문장의 언어. 사용자가 한 번 고르고, 분석을 걸 때 그 분석에
-- 복사해 둔다.
--
-- 두 자리에 두는 이유는 두 사실이 다르기 때문이다. `users.llm_language` 는
-- **사용자가 지금 원하는 언어**이고, `analyses.llm_language` 는 **이 분석이 어떤
-- 언어로 쓰였는가**다. 한 분석의 단계들은 승인 게이트를 사이에 두고 따로따로 워커에
-- 넘어가므로, 사용자 설정을 매번 새로 읽으면 설정이 중간에 바뀐 분석이 절반은 한
-- 언어, 절반은 다른 언어로 남는다.
--
-- 사용자 쪽은 NOT NULL + 기본값 'ko' 다 — 이 컬럼이 생기기 전부터 있던 사용자 행도
-- 고른 적 없는 사용자와 같은 값을 읽는다. 분석 쪽은 NULL 을 허용한다. NULL 은 「언어
-- 지시 없이 시작된 분석」이라는 사실이고, 그런 분석은 시작할 때의 프롬프트 그대로
-- 끝까지 간다. 값의 어휘('ko' · 'en')는 코드가 판정하며 이 파일에 CHECK 로 박지
-- 않는다 — 어휘가 늘 때 테이블을 다시 만들지 않기 위해서다.
ALTER TABLE users ADD COLUMN llm_language TEXT NOT NULL DEFAULT 'ko';
ALTER TABLE analyses ADD COLUMN llm_language TEXT;
