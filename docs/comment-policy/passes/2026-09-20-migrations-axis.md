# 판정 상세 — 적용된 마이그레이션 축 (8파일)

- **판정일**: 2026-09-20
- **판정 범위**: `backend/migrations/0001_init.sql` · `0002_github_tokens.sql` ·
  `0003_analyses.sql` · `0004_analysis_stages.sql` · `0005_analysis_documents.sql` ·
  `0006_discovery_strategies.sql` · `0007_feature_candidates.sql` ·
  `0008_feature_dependencies.sql`
- **기준 트리**: 부모 **`7724b46`** (main, 15차 패스 병합 직후)
- **reconciler task**: `tbm_feature-doc-comment-redundancy/rct_20260920-0009`
- **PR**: #PRNUM (squash)
- ⚠️ **이 패스는 무인 머지 경로가 없다.** 운영 DB의 `_sqlx_migrations` 체크섬을 사람이 손으로
  고친 **뒤에만** 머지한다 — 아래 [repair 명세](#repair-명세--사람이-하는-일).

규칙은 [../README.md](../README.md), 판정 결과의 표면은 [../ledger.md](../ledger.md)에 있다.
이 파일은 이번 범위의 **근거**만 담는다. 이 축에만 붙는 절차는
[`backend/migrations/README.md`](../../../backend/migrations/README.md)가 SSOT다.

## 왜 이 축인가 — 자유 풀이 0이라 대체 슬라이스가 없다

15차 패스가 남아 있던 무인 자유 풀 5파일 38행을 통째로 가져가 **0으로 만들었고**, 원장이
「자유 풀이 0으로 남아 있는 동안 진짜로 남은 일은 마이그레이션 축의 사람 게이트 패스다(139행)」
라고 다음 일을 스스로 지목했다.

잔여를 원장의 자기 집계와 별개로 다시 셌고 **세 방향이 일치**한다: ⑴ 전역 지문의 파일 집합에서
판정 98파일을 뺀 집합 차 **14파일 / 242행** ⑵ 뺄셈 `2,310 − 2,068 = 242` ⑶ 몫의 합
`139 + 44 + 59 = 242` · `8 + 4 + 2 = 14`. 이 패스는 그중 **첫 몫 8파일 / 139행**이다.

남은 두 몫(D2 4파일 44행 · D6·D5 2파일 59행)은 **섞지 않았다**. 정의가 「다른 주석 정리와
섞지 않고 별도 PR 하나」를 못박기 때문이고, 섞으면 그쪽 정리까지 운영 DB repair 라는 훨씬 무거운
게이트에 묶인다. 예외는 `backend/tests/migrations.rs` 하나인데 — 아래 —
**주석 판정이 아니라 고정값 갱신** 때문에 불가피하게 이 PR에 들어온다.

## 데이터 저장 형식 변경 판정: ⚠️ 사람 리뷰 필요 (status 미부여)

원장이 못박은 「슬라이스 전 필수 절차」대로 판정기를 먼저 돌렸다. 이 축에서는 **`✅ 변경 없음`이
나오지 않는 것이 기대값**이다 — D1이 `backend/migrations/**`의 *모든* 변경을 잡기 때문이다.

```
$ python3 tools/check-data-format-change.py --base 7724b46 --head <이 PR head> --verbose
## 데이터 저장 형식 변경 판정: ⚠️ 사람 리뷰 필요 (status 미부여)

변경 파일 8개 · 범위 `7724b46...`

### D1 마이그레이션 — 7건
- backend/migrations/0001_init.sql · 0003 · 0004 · 0005 · 0006 · 0007 · 0008

### D2 저장 계층 핵심 — 1건
- backend/tests/migrations.rs
```

`main`의 필수 체크는 **`ci-gate`와 `review/data-format` 둘**이고(`/rules/branches/main` 실측),
`review/data-format`은 판정기가 `needs_review=false`를 낼 때만 워크플로가 붙인다. 즉 이 PR은
**사람이 그 status를 직접 올려 주기 전에는 구조적으로 머지되지 않는다.** 이것은 사고가 아니라
이 축에 설계된 게이트다 — 그 게이트가 가리는 진짜 위험은 status가 아니라 **운영 DB의 체크섬**이다.

## 집계

| 파일 | 부모(`7724b46`) | 이 패스 뒤 | 순 제거 |
| --- | ---: | ---: | ---: |
| `0001_init.sql` | 12 | 11 | 1 |
| `0002_github_tokens.sql` | 4 | 4 | **0 (파일 무변경)** |
| `0003_analyses.sql` | 6 | 5 | 1 |
| `0004_analysis_stages.sql` | 13 | 9 | 4 |
| `0005_analysis_documents.sql` | 16 | 14 | 2 |
| `0006_discovery_strategies.sql` | 21 | 19 | 2 |
| `0007_feature_candidates.sql` | 32 | 29 | 3 |
| `0008_feature_dependencies.sql` | 35 | 32 | 3 |
| **합** | **139** | **123** | **16** |

- 범위 지문: `b11c4d16de9686c65f99a206a14db13970dec55beee11a33f2a90054321cef7a`(부모) →
  **`1b35c33a821701c748cc127e43169b26f3b1b7f586c29f75e2fa7b1f0d3fe878`** (`lines=123 files=8`).
- 전역 지문: `lines=2310 files=108` → **`lines=2294 files=108`** /
  `7d885d272ff61b93038071007bcfe93a39eca0b8df7d1d30156fc13f04654c12`.
  **파일 수가 안 줄었다** — 이 축은 전건 제거된 파일이 없다.
- **SQL문 변경 0.** 여덟 파일 전부, 주석·빈 줄을 걷어낸 나머지가 부모와 **바이트 동일**하다.

## 판정 기준 — 이 파일들에만 붙는 규칙이 있다

이 축은 본문 「유지 대상」에 더해 `backend/migrations/README.md` §「주석에 무엇을 쓰나」가
**금지 목록을 직접 정해 둔 유일한 범위**다. 그 문서가 금지하는 것:

- **문서에 사는 식별자** — AC 번호, 여정·단계 ID(`JRN-*`/`STP-*`), 목업 이름.
- **파일 경로 · 모듈/함수 경로** — 언제든 리네임·이동된다.
- **시점 서술** — 「지금은 / 아직 / 나중 슬라이스에」.

그리고 그 금지가 옳다는 증거를 문서가 자기 이력으로 들고 있다: 금지 예시로 들었던 화면 ID `S04`는
체계 자체가 폐지돼 사라졌고, `AC4.1`은 번호가 그대로인 채 **내용이 통째로 갈렸다**(PAT → GitHub
App). `0004`의 "Only stage 1 executes today"는 실제로 거짓이 됐다.

그래서 이 패스의 제거 유형은 셋뿐이다 — **R1 절 제목**(바로 아래 선언이 이미 말한다) ·
**R2 금지된 참조**(문서 식별자 · 모듈/파일 경로 · 시점 서술) · **R3 선언 재진술**(CHECK·타입·
UNIQUE가 그대로 말하는 문장). 나머지, 즉 **「그때 왜 이렇게 했는가」·불변식·함정·저장 형식 규약**은
정의가 이 파일의 자리라고 못박은 내용이라 **전건 유지**다(`0002`가 한 줄도 안 바뀐 이유).

## 제거한 것 — 16행

### R1 절 제목 — 12행 (제목 7행 + 딸린 빈 주석 행 5행)

| 지운 주석 | 복원 경로 |
| --- | --- |
| `0001` `-- Credentials and identity schema.` | ① 바로 아래 `CREATE TABLE users/sessions/installations/llm_keys/audit_log` 다섯 선언이 그대로 말한다 |
| `0004` `-- Worker workload separation: the queue gains a lease so a *separate* worker process can claim work, and each analysis gains one row per pipeline stage.` (2행) | ① 바로 아래 `ALTER TABLE analyses ADD COLUMN claimed_by/claimed_at/lease_expires_at…` 여섯 줄과 `CREATE TABLE analysis_stages` — **무엇을 더했는지**의 재진술이다. 이 마이그레이션의 「왜」는 다음 문단(워커가 이 DB를 열지 않는다)이 따로 들고 있고 그쪽은 유지했다 |
| `0005` `-- Pipeline stage outputs: the documents the analysis produces.` | ① 테이블 이름 `analysis_documents` + `analysis_id`·`kind`·`content` 선언 |
| `0006` `-- Discovery strategy review and approval.` | ① 테이블 이름 `discovery_strategies` + `approved_at` 컬럼 |
| `0007` `-- Feature candidate extraction and review.` | ① 테이블 이름 `feature_candidates` + `decision`·`reject_reason` 컬럼 |
| `0008` `-- Feature 단위 종단 의존성 (AC2.4) 과 그 구조화 적재 (AC2.5).` | ① 두 테이블 이름 · ③④ **그리고 AC 번호는 이 파일이 쓰면 안 되는 것**(R2) |

제목 줄을 지우면 바로 뒤의 `--`(빈 주석 행)가 머리로 남으므로 함께 걷었다 — `0004`·`0005`·
`0006`·`0007`·`0008` 각 1행. **유형별 합은 12 + 3 + 1 = 16행**이고, diff 실측(제거 41행 ·
추가 25행 · 순 16행)과 일치한다.

### R3 선언 재진술 — 3행

| 지운 주석 | 바로 아래 선언 |
| --- | --- |
| `0003` `Owned by a user and always queried under that scope.` | `user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE` + `CREATE INDEX idx_analyses_user`. **같은 명제의 두 벌째**이기도 하다 — `0001` 머리가 이미 "Multi-user: every credential row is owned by a user and queried under that scope"를 말하고, 그 벌은 유지했다 |
| `0004` `Lease + lifecycle columns.` | 바로 아래 여섯 개 `ADD COLUMN` — 절 제목이자 선언 재진술. 같은 문단의 `claimed_by`가 pod 이름이라는 것과 `lease_expires_at`이 회수를 가능하게 한다는 것은 선언에서 안 나오므로 유지 |
| `0007` `` `decision` is `undecided` \| `approved` \| `rejected`. `reject_reason` is NOT NULL whenever `decision = 'rejected'` `` (2행 중 열거·제약 재진술 몫) | `CHECK (decision IN ('undecided','approved','rejected'))`와 `CHECK (decision <> 'rejected' OR reject_reason IS NOT NULL)` **두 줄이 글자 그대로** 같은 말을 한다. 남긴 것은 제약이 아니라 **왜 두 번째 방어선인가**(라우트가 먼저 거른다) |

### R2 금지된 참조 — 순 제거 1행 (나머지는 문단 재작성으로 흡수)

아래 아홉 곳은 문장을 **지운 게 아니라 금지 참조만 걷어내 다시 썼다**. 담긴 지식(왜·불변식)은
복원 경로가 없어 유지 대상이고, 참조만이 낡는 부분이기 때문이다.

| 위치 | 걷어낸 참조 | 대체 |
| --- | --- | --- |
| `0004` 워커 문단 | `the SQLite invariant in db.rs` | `the SQLite invariant that rests on it` — 파일 경로 제거 |
| `0004` `analysis_stages` 문단 | `seeded at enqueue from pipeline::STAGES (the code SSOT)` | `seeded at enqueue from the stage list in code (the SSOT)` — 모듈 경로 제거 |
| `0006` `approved_at` 문단 | `enforced … in worker_api::claim` | `the claim path enforces that against this column` — 모듈·함수 경로 제거 |
| `0007` `key` 문단 | `` That derivation is `feature_candidates::candidate_key` — one place `` | `The derivation lives in one place in code` — 모듈·함수 경로 제거 |
| `0008` `category` 문단 | `AC2.4 가 열거한 7종` · `` `backend/src/dependencies.rs` 의 `CATEGORIES` `` | `「아래 열거가 전부」` · `코드의 카테고리 상수` — AC 번호와 파일 경로 제거. **「7종」이라는 수 자체도 지웠다** — CHECK가 늘면 조용히 거짓이 되는 숫자다 |
| `0008` `evidence` 문단 | 여정 `JRN-review-feature` 의 예외 표 인용 | 요구사항을 **자체 완결로** 서술(「근거 없음으로 명시하고 임의로 채우지 않는다」) — README가 권하는 바로 그 형태 |
| `0008` `model` 문단 | `(AC4.6 의 실측 비용 회계는 아직 이 값을 읽지 않는다)` | 삭제 — AC 번호이면서 **시점 서술**(「아직」)이다. 읽는 쪽이 생기면 이 괄호는 아무도 안 고친 채 거짓이 된다 |
| `0008` 역방향 인덱스 | `AC2.5 의 역방향 질의` | `역방향 질의` — AC 번호 제거, 인용 문구는 유지 |
| `0008` 머리 문단 | `AC2.5 의 검증 방법이 …` · `「단순 텍스트 문서가 아니라 구조화된 데이터」라는 AC2.5 의 문면 그대로다` | 「답해야 하는 것이 **행 선택 질의**여서다」로 자체 완결 서술. 마지막 문장은 **AC 문면의 인용 그 자체**라 제거 |

`0008`의 `0007` 참조(「0007 이 후보를 행으로 고른 것과 같은 이유」)는 **남겼다** — 마이그레이션
버전 번호는 이 디렉터리 안에서 불변이고, README가 허용하는 「불변인 것」에 해당한다.

## 유지한 것 — 123행

전건 유지의 근거는 하나다: **이 파일의 주석은 「그 시점에 왜 이 변경을 했는가」의 기록**이고, 그
지식은 코드·문서·PR·커밋 어디서도 복원되지 않는다. 특히:

- **저장 형식 규약** — `0001`의 "Timestamps are unix epoch seconds (INTEGER). Ids are opaque TEXT
  (uuid v4)": 컬럼 타입은 `INTEGER`까지만 말하고 **밀리초가 아니라 초**라는 것은 말하지 않는다.
- **봉투 암호화 형식**(`0001` 4행) — 어떤 바이트가 무엇의 암호문인지는 컬럼 이름이 아니라 이
  문단만 말한다. `The plaintext key is NEVER stored`는 **없는 컬럼에 대한 주장**이라 코드에서
  복원되지 않는다. 같은 성격으로 `installations`의 "tokens are minted on demand and never
  stored"(3행)와 `audit_log`의 "Append-only"(1행)도 유지 — 셋 다 **제약이 아니라 약속**이다.
- **상류 API의 함정** — `0002`의 "the Setup URL's installation_id is spoofable (GitHub docs)".
  이 패스가 `0002`를 한 글자도 안 바꾼 이유다(12차 패스가 이 명제의 정본을 `github_app.rs`로
  정했지만, 이 벌은 **왜 이 테이블이 존재하는가**를 말하는 자리라 중복으로 보지 않았다).
- **단일 writer 불변식**(`0004` 3행) — 워커가 이 DB를 열지 않는다는 것은 이 스키마 어디에도
  없다. 「one pod, one mount, `Recreate`」는 배포 매니페스트 쪽 사실이고 여기 기록이 그 연결이다.
- **설계 선택의 이유**(`0005` 14행 · `0006` 19행 · `0007` 29행 · `0008` 32행) — 왜 버전 축을
  두지 않았나, 왜 문서와 전략을 두 행으로 갈랐나, 왜 후보는 행이고 전략은 JSON인가, 왜 요청과
  결과가 두 테이블인가. 전부 **선택하지 않은 쪽**을 함께 담고 있어 결과 코드에서 복원되지 않는다.

## 판단이 갈려 남긴 것 — 2건

1. **`0003`의 "Pre-flight estimates … (display only, never a hard cap)"** — "display only"는
   라우트 코드에서도 읽히지만, **한도로 쓰지 말라는 금지**는 코드에 없다(그 코드가 생기면 늦다).
   유지.
2. **`0006`·`0007`의 "Stage 3 writes …" / "Stage 4 writes …"** — 파이프라인 단계 *번호*는
   순서가 바뀌면 낡는다는 점에서 R2의 이웃이다. 다만 README의 금지 목록은 **문서에 사는 식별자**
   (AC·여정·화면)를 든 것이고 단계 번호는 이 저장소의 코드가 정하는 순서라 그 목록에 없다.
   **애매하면 남긴다**에 따라 유지하고 여기 적어 둔다 — 뒤집으려면 두 파일의 증분 재판정으로
   한 번에 한다(그때도 repair 게이트를 다시 통과해야 한다).

## `backend/tests/migrations.rs`가 이 PR에 들어온 이유 — 주석 판정이 아니다

`applied_migrations_are_never_edited`가 여덟 파일의 **SHA-256을 고정**하고 있어, `.sql`을 고치면
그 고정값을 같은 PR에서 갱신하지 않는 한 `ci-gate`가 빨갛다. 그래서 이 파일에서 바꾼 것은 **딱
둘**이다:

- `APPLIED` 배열의 해시 7개(0002는 불변).
- 그 바로 위 출처 주석 — "Pins the file bytes accepted by main at PR #44"가 거짓이 되므로
  이 PR을 가리키게 고쳤다.

**이 파일의 주석 13행은 판정하지 않았다.** D2 몫으로 잔여에 그대로 남는다(아래 「범위 밖」).
`migrations/README.md`가 경고하는 「파일과 고정값을 동시에 바꾸면 검사를 우회한다」가 바로 이
PR이며, 그 우회를 메우는 것이 아래 사람 게이트다.

## repair 명세 — 사람이 하는 일

sqlx는 `.sql` **파일 전체 바이트**의 SHA-384를 `_sqlx_migrations.checksum`에 들고 부팅마다
대조한다. 주석 한 글자만 달라도 `VersionMismatch`로 **부팅이 실패**한다. CI는 매번 새 DB라
이 사고를 못 잡고, main 머지는 `pin` → Flux 롤아웃으로 사람 개입 없이 운영에 닿는다.

**표의 「새 SHA-384」가 운영 DB에 들어가야 할 값이다.** 절차는
[`backend/migrations/README.md`](../../../backend/migrations/README.md) §「그래도 파일을 고쳐야
한다면 (repair)」 1~6단계를 **그대로** 따른다 — 여기 요약은 그 문서를 대체하지 않는다.

| version | 파일 | 부모(`7724b46`)의 SHA-384 | **새 SHA-384 (UPDATE 할 값)** |
| ---: | --- | --- | --- |
| 1 | `0001_init.sql` | `3e74cffa305814e63c635a8bc503789e806b7045079a1a7c3bb2f665624bb04791996fb00f61fb905e6ddb8debc803cf` | `7cba87314ff5a4b69d38fcb4b189afc289e22bf8d4ed9405a90ec16894f18e9932f49933b8ac92dfccbcb8020fb5f9aa` |
| 3 | `0003_analyses.sql` | `f1e43a3533eee5595b63ee023f250556ef77d7bc631b58b6a9981346f9376bfadf1b843ce842c9066bd0d14b69e567ed` | `07ff762860cbc01ea489bc041a270d410b0185b2ea4bd24d96677744b8a2269680817587b3c4d930d9a8e5553c7d88b9` |
| 4 | `0004_analysis_stages.sql` | `cb15b0eb442cc4162dd6088a7fc3900a664fb4b0a1af99b99806ef264e884ce40a66ec32d85e4831304fc93174035a61` | `c2fe2aa225c78fb365b790fe1b50ec642a1bb24c5c6fb9953711854b268ca84bed886a652c2a05c26e1e80269d5b3179` |
| 5 | `0005_analysis_documents.sql` | `d4ebc3ea3b8dea4a57283c623bdc041fdf4104e4e8279ae6418c5498927734111a892fbd093b40b30d25c7c9f85753c5` | `83152a959fd441d66ef22338b9ea3ac3e7717867fbf9eccefe708f6c33da63725b3d0c215801b4761ae87106b6bbdb23` |
| 6 | `0006_discovery_strategies.sql` | `55a09367daf36d176a3c0ee6fe08f52310160548e9c55047199b624bdc7853aebaf9260dfc39c08704b40768a919e790` | `8b18f22e0bf97ec7c3f4023382214d8c1a6447ea3b6132161427fdf9f1781d7cacbb2fd8228b8b98c730907588105004` |
| 7 | `0007_feature_candidates.sql` | `536fd820b4935d367f664781dee1a3d506d772c8d3a98a89749e3b7d1356b4777293de14fc4597098bed58216f37cf35` | `ee23f4af9fcc5b33bae8056fd282db5060a9b41ceedf1421f8c2f8cb1cf2b61e11e680a8830572927589e93f24747bc2` |
| 8 | `0008_feature_dependencies.sql` | `0e9b371c61529c49bb529564a2d1d2a8b29aa8099717fa7c81b798b8e93915c52478bb6722058ecba369894e8121ea29` | `93ec2fb97de48ef7e1188f6a74391df93f8527dd293be8c8062b0fe8f4dd6d5b245c0aa4e2020a32fc9eb25bdbc36498` |

**version 2(`0002_github_tokens.sql`)는 바뀌지 않았다 — 건드리지 않는다.**

> **「부모의 SHA-384」는 소스에서 계산한 값이지 운영 DB를 관측한 값이 아니다.** README가 같은
> 경고를 한다. 4단계에서 `SELECT version, description, hex(checksum) FROM _sqlx_migrations`로
> **실제 값을 먼저 읽어** 이 열과 같은지 확인하고, 다르면 UPDATE하지 말고 멈춘다 — 배포된 바이트가
> 여기 가정과 다르다는 뜻이다.

순서 요약(자세한 명령은 README):

1. **DB를 먼저 고치고 머지는 나중에.** 머지가 먼저면 새 파드는 무조건 `VersionMismatch`로 죽는다.
2. `flux suspend` → `kubectl scale deploy/featuredoc --replicas=0` → 파드가 완전히 내려간 것 확인
   (RWO 볼륨을 정비 파드가 잡아야 한다).
3. 정비 파드에서 `.bak` 백업 → 현재 `_sqlx_migrations` 확인 → **위 표의 7개 version만** UPDATE.
4. `flux resume` → **곧바로** 머지(resume과 머지 사이가 길면 구 이미지 파드가 새 체크섬과
   어긋난 채 재기동될 수 있다).
5. 롤아웃 완료 · 부팅 로그에 `VersionMismatch` 없음 · `/hello` 응답 확인.
6. 실패하면 `.bak` 복원 + 이미지 태그를 직전 SHA로 되돌린다(재빌드 불필요).

**머지 직전 대조**: PR head의 각 `.sql` SHA-384를 다시 계산해 위 표와 **정확히 같은지** 확인한다.
승인 뒤 rebase·수정으로 바이트가 달라졌으면 머지하지 않는다 — 운영 DB는 이미 표의 값으로 고쳐져
있다.

```bash
for f in backend/migrations/000{1,3,4,5,6,7,8}_*.sql; do
  python3 -c "import hashlib,pathlib,sys; p=pathlib.Path(sys.argv[1]); print(p.name, hashlib.sha384(p.read_bytes()).hexdigest())" "$f"
done
```

## 검증 (판정 시점 로컬 실측, 부모 `7724b46`)

| 검사 | 결과 |
| --- | --- |
| **SQL문 무변경** | 8파일 전건 **SAME** — 주석(`--`)과 빈 줄을 걷어낸 나머지가 부모와 바이트 동일. `.sql`에 블록 주석이 0건이라 줄 단위 stripper로 충분하다 |
| 전역 주석 지문 | `lines=2310 files=108` → `lines=2294 files=108` (−16, 파일 수 불변) |
| 범위 지문 | `b11c4d16…`(139행) → `1b35c33a8217…`(123행) |
| `python3 tools/check-data-format-change.py --base 7724b46 --head <head> --verbose` | **`⚠️ 사람 리뷰 필요`** — D1 7건 · D2 1건. **기대값**(이 축은 무인 경로가 없다) |
| `APPLIED` 고정값 | 7개 갱신 · `0002`는 그대로. `every_migration_file_is_pinned`의 목록 8개는 불변 |
| `PRE_CLEANUP` | **건드리지 않았다.** 그 여섯 값은 PR #44 부모의 SHA-384이고, 이 패스가 같은 파일을 더 바꿨으므로 `pre_cleanup_checksums_are_rejected_without_repair`의 「현재 파일과 달라야 한다」 단정은 계속 참이다 |
| `cargo test --test migrations` | **CI(`ci-gate`)가 돌린다** — 이 계획을 세운 호스트에 Rust 빌드를 올릴 디스크가 없어 로컬 실행을 하지 않았다. 고정값이 틀리면 `applied_migrations_are_never_edited`가 빨갛게 잡는다 |
| 문서 게이트 | `check-journey-mockup.py` rc=0 — 허브 `Documents` 41 → **42**(이 문서 등재) |

## 범위 밖 (후속)

- **D2 「저장 계층 핵심」 4파일 / 44행** — `backend/src/db.rs` 17 · `backend/src/crypto.rs` 13
  (13차 패스가 판정까지 마쳐 둠) · `backend/tests/migrations.rs` 13 · `backend/src/models.rs` 1.
- **D6 · D5 경로 규칙 2파일 / 59행** — `tools/check-data-format-change.py` 58 ·
  `deploy/k8s/pvc.yaml` 1.
- 이 패스가 머지되면 잔여는 **6파일 / 103행**이 되고, **전부 D2·D6·D5 경로 벽**이다. 무인
  자유 풀은 여전히 0이며, 열린 PR이 in-scope 파일에 주석을 더하면 그때 다시 열린다.
- **`0006`·`0007`의 단계 번호**(위 「판단이 갈려 남긴 것」 2번)는 뒤집으려면 증분 재판정이
  필요하고, 그때도 이 축의 repair 게이트를 다시 통과해야 한다.
