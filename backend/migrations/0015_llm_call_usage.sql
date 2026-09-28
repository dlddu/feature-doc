-- 기본값 1 은 이 열이 생기기 전에 적힌 행을 위한 것이다(합쳐진 행은 그때도 1행이었다).
ALTER TABLE analysis_documents ADD COLUMN calls INTEGER NOT NULL DEFAULT 1;
