# 데이터 모델 (Data Model)

`dlddu/feature-doc`의 데이터 모델 지도다. 문서는 세 장이고, 각 장이 서로 다른 실재와 짝을 이룬다.
문서는 설계서가 아니라 **실재를 따라가는 지도**다 — 어긋나면 문서를 고친다.

| 문서 | 짝을 이루는 실재 | 충돌 시 이기는 쪽 |
|---|---|---|
| [erd.md](erd.md) | `backend/migrations/`를 빈 DB에 전부 적용한 스키마 | **스키마** |
| [query-patterns.md](query-patterns.md)의 패턴·호출 지점 | `backend/src`의 쿼리 지점 | 존재 여부는 **코드** |
| [query-patterns.md](query-patterns.md)의 지원 인덱스 칸 | 빈 DB에서 엔진이 낸 쿼리 플랜 | **엔진** |

판정 규칙의 원본은 reconciler `templates/data-model.tbm.md`의 고정부(불변식 · 형태 · 인덱스 라벨 ·
문서 배치 · 체커 계약)다. 이 문서는 그 고정부를 다시 쓰지 않고, 이 레포에 고유한 것만 적는다.

## 형태 정의

쿼리 형태는 템플릿 「형태」 절을 그대로 따른다 — `(연산, 대상 테이블, 등치 조건 컬럼, 범위 조건, 정렬)`을
한 줄 `<연산> <테이블> | eq(…) | range(…) | order(…)`로 적는다. 이 레포 고유의 정규화는 다음뿐이다.

- **쿼리 레이어**: `backend/src`의 `sqlx::query`·`query_as`·`query_scalar` **런타임 함수**(매크로가
  아니다)와 그 터보피시(`query_as::<_, T>`)에 넘긴 SQL. SQL은 문자열 리터럴(`"…"`·`r#"…"#`, 줄 끝
  `\` 연속 포함)이거나 같은 파일의 `const` SQL(`ACTIVE_KEY_SQL`·`CALL_ROWS` 등)이다.
- **`format!` 조립**: 보간마다 셋 중 하나로 푼다. ① 같은 파일의 `const` SQL이면(`{CALL_ROWS}`) 그
  상수를 그 자리에 편다. ② 같은 파일의 인자 없는 함수가 `format!` 하나를 돌려주면(`analysis_columns()`)
  그 함수를 같은 규칙으로 펴서 넣는다 — 선택 목록 안의 스칼라 서브쿼리도 형태에 합쳐진다. ③ SQL 문자열
  리터럴(`'…'`) 안의 보간(`'{done}'`)은 값일 뿐이라 형태에 영향이 없다. 셋 다 아니면 추출 불가 — 수동
  형태로 간다.
- **연산**: `INSERT … ON CONFLICT`와 `REPLACE`·`INSERT OR REPLACE`는 `upsert`다(`eq`는 충돌 대상 컬럼).
  `INSERT OR IGNORE`는 충돌 대상이 없으므로 `insert`다.
- **서브쿼리**: 선택 목록·`WHERE`·`SET` 어디의 서브쿼리든 바깥 쿼리와 한 형태로 합친다(테이블·조건
  합집합). `ORDER BY`는 서브쿼리 것이 먼저, 바깥 것이 뒤에 붙는다. `FROM`/`JOIN`의 파생 테이블(괄호
  서브쿼리에 별칭)은 바깥 조건의 컬럼이 한 테이블에 귀속되지 않으므로 추출 불가다.
- **컬럼 귀속**: 별칭은 테이블명으로 되돌린다 — 자기 조인(`analyses cur JOIN analyses prev`)의 조인
  등치는 `analyses.user_id=analyses.user_id`가 된다. 한정자 없는 컬럼은 그 쿼리의 테이블 중 그 컬럼을
  가진 하나(마이그레이션을 적용한 스키마 기준)에 귀속한다.
- **식 정규화**: 조건의 왼쪽이 식이면 식 텍스트를 소문자·공백 1칸으로 정규화한다(`eq(lower(login))`).
  `ORDER BY`의 식도 같다(`order((provider = 'openai') desc, created_at desc)`).
- **표 안의 `|`**: `query-patterns.md` 표 칸 안에서는 형태·SQL의 `|`를 `\|`로 쓴다(GFM 표 규칙). 체커는
  `\|`를 `|`로 읽는다.

## 풀스캔 허용 기준

**사람이 소유한다.** 기준을 새로 만들거나 넓히는 것은 사람의 결정이고, 자동화(data plane)는 기준을
발명하지 않는다 — 필요해 보이면 PR 설명에 제안만 남긴다.

| ID | 기준 | 관측 가능한 근거 |
|---|---|---|
| F1 | 사용자 한 명의 사용량 화면처럼 사람이 직접 여는 조회에서, 파생 테이블(`UNION ALL`) 구체화 때문에 스캔하는 경우 | 플랜의 `MATERIALIZE` 아래 `SCAN`, 호출 지점이 사용자 요청 핸들러 하나 |

지원 인덱스가 없는 패턴은 인덱스 추가 마이그레이션 PR로 가거나, 이 표의 기준 하나에 해당할 때 그
기준으로 등재된다. 표에 없는 기준이 필요하면 사람이 먼저 이 표에 더한다.

F1은 `Q-31`(`usage.rs::read`, `GET /api/usage` 사용량 화면)을 계기로 더한 기준이다. 바깥 `analyses`는
`idx_analyses_user`로 찾지만, `LEFT JOIN`한 파생 테이블(`{CALL_ROWS}` — 네 테이블의 `UNION ALL`)을 엔진이
통째로 구체화하면서 네 테이블을 풀스캔한다. 네 테이블에는 이미 `analysis_id` 인덱스가 있고, 덮개
인덱스를 더해도 플랜은 `SCAN … USING COVERING INDEX`(풀스캔)로 남는다 — 인덱스로는 닫히지 않는다.
F1을 받아들이지 않았다면 남는 길은 쿼리를 바꾸는 것(이 지도의 범위 밖)이었다.

## 추출 제외 범위

제외는 **쿼리가 아닌 것**만 뺀다. 제품 경로의 쿼리를 체커가 어려워한다는 이유로 빼지 않는다 — 그건
수동 형태로 간다.

- **범위 밖 경로**: `backend/tests/`(통합 테스트)와 `backend/migrations/`(마이그레이션 자체)는
  `scope`가 `backend/src`라서 처음부터 들지 않는다. 그래서 `exclude` 키는 두지 않는다.
- **`#[cfg(test)]` 모듈**: `backend/src` 파일 안의 테스트 모듈 쿼리는 경로로 뺄 수 없어 버전 지문에는
  들어가지만, 체커는 지점에서 제외한다. 주석·문자열 안에서 `site`에 걸린 텍스트도 호출이 아니므로
  제외한다. 제외한 후보도 리포트의 `sites`에 사유와 함께 남는다 — 조용히 건너뛰는 후보는 없다.
- **스키마 관리 테이블**: `_sqlx_migrations`(sqlx 적용 이력)와 `sqlite_sequence`(SQLite 내부)는 불변식
  1에서 뺀다(`schema-exclude`).

아래 블록은 체커와 reconciler 감지 스크립트가 **함께** 읽는 범위 선언이다. 범위는 이 블록 한 곳에만
둔다 — 체커 코드에 따로 적지 않는다.

```data-model-scope
migrations: backend/migrations
checker: tools/check-data-model.py
scope: backend/src
site: sqlx::query(_as|_scalar)?(::<[^>]*>)?!?\(
schema-exclude: ^(_sqlx_migrations|sqlite_sequence)$
sql: \b(FROM|JOIN|INTO|DELETE FROM) [a-z_]+\b|\bUPDATE [a-z_]+( +SET\b| *\\?$)|\bWHERE\b|\b(AND|OR) \(?[a-z_.()]+ *(=|<|>|!=|IN |IS |LIKE|BETWEEN)|\bORDER BY\b|\bON CONFLICT\b
```

## ERD의 `의미` 링크

ERD의 엔티티마다 `의미:` 한 줄은 그 테이블을 소유한 **모듈의 rustdoc 머리(`//!`)** 를 가리킨다 —
`` 의미: [`session`](../../backend/src/session.rs) ``. 행 구조체(`#[derive(FromRow)]`)는 테이블과 1:1이
아니라 조회마다 다른 투영이라 테이블의 의미를 말하지 않는다. 테이블이 무엇을 위해 있는지는 그 테이블을
쓰는 흐름을 소유한 모듈의 머리 주석이 말한다. 의미 문장은 ERD에 쓰지 않는다 — 코드 옆에 있어야 코드와
함께 바뀐다.

링크 텍스트가 대상 파일 이름(확장자 제외)과 같으면 모듈 링크이고, 체커는 그 파일의 첫 줄이 `//!`인지
확인한다. 다르면 항목 링크이고, 그 파일에 `///` doc 주석이 달린 그 이름의 항목이 있는지 확인한다.

## 체커

```sh
python3 tools/check-data-model.py          # 사람이 읽는 리포트
python3 tools/check-data-model.py --json   # 기계 판독형
```

- **엔진**: Python 표준 `sqlite3`(러너의 SQLite). 운영은 `libsqlite3-sys` 0.30.1 bundled SQLite
  3.46.0이다(`backend/Cargo.lock`). 리포트 첫 줄에 체커가 쓴 엔진 버전이 찍힌다. 플랜 판정을 들일 때
  bundled 3.46.0 소스를 그대로 빌드해 같은 체커를 돌려 대조했다 — 3.45.1과 81개 패턴의 판정·미사용
  인덱스 8개가 모두 같았다. 엔진 버전이 바뀌면 이 대조를 다시 한다.
- **스키마 관측**: 빈 인메모리 DB에 `migrations` 디렉터리의 `<버전>_<설명>.sql`을 버전 순으로 전부
  적용한다(sqlx `migrate!`와 같은 순서, `.down.sql` 제외). 그 뒤 `PRAGMA table_info`·`foreign_key_list`·
  `index_list`·`index_xinfo`로 읽는다. 마이그레이션 SQL 텍스트를 해석하지 않는다(부분 인덱스 조건과 식
  인덱스의 식 텍스트만 엔진이 보관한 `CREATE INDEX` 문에서 읽는다).
- **종료 코드**: `0` 정합 · `1` 위반 · `2` 판정 불가(블록·문서 파싱 실패, 마이그레이션 적용 실패, 아직
  구현되지 않은 불변식). 판정 불가를 정합으로 뭉개지 않는다.
- **지점 추출**: `scope` 파일에서 `site`에 걸리는 줄을 모두 후보로 보고(버전 지문의 `site_lines`와 같은
  수), 각각을 (지점, 형태, SQL) · (지점, 추출 불가 사유) · (제외 사유) 중 하나로 낸다. 지점 이름은
  둘러싼 함수에서, SQL은 호출의 첫 인자(리터럴·`const`·`format!`)에서, 형태는 SQL을 읽어 계산한다.
- **출력**: 위반마다 기대하는 행(ERD 컬럼 표 행 · 인덱스 표 행 · mermaid 관계 줄 · 빠진 엔티티 절 전체 ·
  카탈로그 패턴 행 · 수동 형태 행)을 함께 낸다. 체커는 문서를 고치지 않는다 — 수정 PR이 그 행을 옮겨
  적는다. `--json`의 `sites`는 후보마다 추출 결과를 담는다.
- **플랜 판정(불변식 3)**: 패턴마다 그 지점들의 SQL(수동 형태는 대표 SQL)을 같은 빈 DB에서
  `ANALYZE` 없이 `EXPLAIN QUERY PLAN`으로 돌린다(자리표시자는 `NULL`). 테이블 접근마다 `SEARCH … USING
  [COVERING] INDEX`·`USING INTEGER PRIMARY KEY`·`USING PRIMARY KEY`는 그 인덱스의 지원, 그 밖의
  `SCAN`과 `AUTOMATIC` 인덱스는 풀스캔이다. 별칭은 SQL의 `FROM`·`JOIN`·`UPDATE`에서 테이블로 되돌리고,
  파생 테이블 자체의 접근은 세지 않는다(그 안의 테이블 접근은 센다). 같은 패턴의 지점끼리 판정이
  다르면 위반(`split-plan`)이다. 별도 정렬 단계(`USE TEMP B-TREE FOR ORDER BY`)는 `--json`의 `plans`에
  표시만 하고 판정에 넣지 않는다. 어떤 플랜도 쓰지 않는 인덱스(PK 제외)가 「미사용 인덱스」 표와
  양방향으로 같은지도 본다. 불변식 2가 참이 아니면 불변식 3은 판정 불가다.
- **현재 범위**: 세 불변식을 모두 판정한다. 수동 형태의 대표 SQL은 빈 DB에서 준비(`EXPLAIN`)되는지도
  본다.
- **CI**: `report` 모드(머지 비차단)로 돈다 — `.github/workflows/docs-data-model.yml`이 모든 PR과 `main`
  push에서 체커를 돌려 리포트를 job 요약에 남긴다. 종료 코드 `1`(위반)은 경고 주석만 달고 통과하고,
  `2`(판정 불가)는 job을 실패로 표시한다. 이 job은 `ci-gate`의 `needs`에 없어서 어느 쪽이든 머지를
  막지 않는다. 세 불변식이 처음 함께 참이 된 뒤(`main`에서 종료 코드 `0`) `gate` 모드(머지 차단)로
  올린다 — 그때는 다른 `docs-*.yml`처럼 `workflow_call`로 바꿔 `ci.yml`이 부르고 `ci-gate`의 `needs`에
  넣으며, 종료 코드 `1`도 실패로 친다.
