# 판정 상세 — 테스트 하네스 축 (5파일)

- **판정일**: 2026-09-20
- **판정 범위**: `backend/tests/worker.rs` · `backend/tests/common/mod.rs` ·
  `backend/tests/analyses.rs` · `scripts/e2e.sh` ·
  `e2e/tests/sc02-02-acceptance-from-tests.spec.ts`
- **기준 트리**: 부모 **`7415771`** (main, 13차 패스 병합 직후)
- **reconciler task**: `tbm_feature-doc-comment-redundancy/rct_20260920-0007`
- **PR**: #96 (squash)

규칙은 [../README.md](../README.md), 판정 결과의 표면은 [../ledger.md](../ledger.md)에 있다.
이 파일은 이번 범위의 **근거**만 담는다.

## 범위를 이 5파일로 고른 이유 — 그리고 원장의 후보 ①을 쓰지 않은 이유

원장이 13차 패스에서 다음 축을 둘 지목해 뒀다: **① `tools/check-data-format-change.py` 58행**
(「단일 파일 최대. `#64` 가 들여놓았고 아직 아무도 안 봤다」)과 **② 테스트 하네스 축 3파일 58행**.

**①은 쓸 수 없다.** 아래 「무인 머지 경로가 없는 두 파일」 절에서 실측으로 보인다 — 그 파일은
판정기 자신의 **D6** 규칙에 경로로 걸려, 주석 한 줄만 고쳐도 필수 체크가 붙지 않는다. 원장이
D2 를 사람 게이트로 갈라 적으면서 **D6·D5 를 같은 성격으로 보지 않은 것**이 이 패스가 정정하는
지점이다.

그래서 ②를 집되, 같은 성격의 **비경합·게이트 무관** 파일 둘을 더해 5파일로 넓혔다 —
`scripts/e2e.sh`(e2e 러너 하네스, 어떤 D 규칙에도 안 걸린다)와
`e2e/tests/sc02-02-acceptance-from-tests.spec.ts`(인수 축에서 홀로 남은 미판정 spec).
`backend/tests/migrations.rs` 13행은 **섞지 않았다** — D2 다섯 중 하나라 섞으면 축 전체가 막힌다
(13차 패스가 `crypto.rs` 로 겪은 벽이고, 원장이 명시적으로 경고한 것이다).

## 무인 머지 경로가 없는 두 파일 — 원장 정정

`.github/workflows/data-format-review.yml`(`pull_request_target`)이 돌리는
`tools/check-data-format-change.py` 는 **경로 소속만으로** 판정하는 규칙을 셋 갖는다:

- **D1** `backend/migrations/**` 전부
- **D2** 저장 계층 핵심 5파일 (원장이 이미 사람 게이트로 등재)
- **D6 판정기 자신** — `tools/check-data-format-change.py` 와
  `.github/workflows/data-format-review.yml`
- **D5** 중 `deploy/**/pvc*.y*ml` (`is_pvc()` 매칭) 은 **모든** 변경

`classify()` 에서 이 넷은 `if f in SELF_PATHS` / `if f in CORE_FILES` / `if is_pvc(f)` 처럼
**파일 목록만** 보고 히트를 적는다. 줄 단위로 도는 **D3 에만** `if is_comment_only(text): continue`
예외가 있고, 경로 규칙에는 없다. 그래서 **주석 한 줄만 고쳐도** `needs_review=true` 가 되고,
워크플로는 commit status `review/data-format` 을 붙이지 않는다. 그 status 는 main 룰셋의 필수
체크라 없으면 `mergeable_state=blocked` 이 해소되지 않는다.

**음성 대조**(부모 `7415771` 에서 주석 줄만 지운 probe 커밋으로 판정기를 직접 실행):

| probe | 판정기 출력 |
|---|---|
| `tools/check-data-format-change.py` 의 주석 1행 제거 | `⚠️ 사람 리뷰 필요 (status 미부여)` · **D6 판정기 자신 — 1건** |
| `deploy/k8s/pvc.yaml` 의 주석 제거 | `⚠️ 사람 리뷰 필요 (status 미부여)` · **D5 배포 저장소 — 1건** |
| 이 패스의 5파일 | `✅ 변경 없음 (review/data-format = success)` |

따라서 **자유 풀은 12파일 185행이 아니라 10파일 126행이었다.** 원장의 잔여 셋을 넷으로 고쳐
등재한다. `deploy/k8s/kustomization.yaml`(15행)은 **자유 풀에 남는다** — `is_pvc()` 가 아니고,
비-pvc `deploy/**/*.yaml` 은 줄 단위 D5 규칙인데 그 코드가
`if not text.strip().startswith("#")` 로 주석 줄을 먼저 거르기 때문이다.

## 집계

| 파일 | 주석 행(부모 → 이 패스) | 순 제거 |
|---|---|---|
| `backend/tests/worker.rs` | 35 → 6 | 29 |
| `e2e/tests/sc02-02-acceptance-from-tests.spec.ts` | 23 → 3 | 20 |
| `backend/tests/common/mod.rs` | 16 → 9 | 7 |
| `backend/tests/analyses.rs` | 7 → 0 | 7 |
| `scripts/e2e.sh` | 7 → 2 | 5 |
| **합계** | **88 → 20** | **68** |

범위 지문 `ce6ef17f0ae2afb615060a460a1941f8d9364b4aee9628debd2597a90443a2f5` →
`8b8764df56b28c69c1419f82a0f65c23c0316815315a37e6b0e9d09af544fc9c`.
**`backend/tests/analyses.rs` 는 주석 0행이 되어 지문의 파일 집합에서 빠진다**(범위 `files=5` →
`4`, 전역 `files=110` → `109`) — `format.ts`·`backend/tests/crypto.rs` 에 이은 세 번째다.
판정 파일 수를 지문의 `files` 로 세면 어긋나므로, 원장 합계는 **행의 목록 길이**로 센다.

## 제거한 것 — 복원 경로별

### ① 코드 자체가 이미 말하는 것 (가장 큰 몫)

- **테스트 이름이 그대로 말하는 `///` 10행**(`worker.rs`) — 주석과 그 아래 `async fn` 이름이 같은
  명제다. `내부 라우트는 워커 토큰이 없으면 닫힌다`(`internal_routes_are_closed_when_no_worker_token_is_configured`) ·
  `두 워커가 한 잡을 두고 경합하면 승자는 정확히 하나`(`two_workers_racing_for_one_job_produce_exactly_one_winner`) ·
  `두 잡·두 워커는 서로 다른 것을 집는다`(`concurrent_workers_take_disjoint_jobs`) ·
  `임대가 끝나면 잡이 큐로 돌아온다`(`an_expired_lease_returns_the_job_to_the_queue`) ·
  `임대 없는 워커는 보고할 수 없다`(`a_worker_without_the_lease_cannot_report`) ·
  `워커가 없어도 API 는 답하고 큐는 유지된다`(`the_api_serves_and_the_queue_holds_while_no_worker_claims`).
- **절 제목 7행** — `worker.rs` 의 `// ── authentication ──`·`// ── enqueue seeds the pipeline ──`·
  `// ── claim ──`·`// ── progress reporting ──` 넷은 바로 아래 테스트 이름이 그대로 말한다.
  `sc02-02` 의 `// ── 보강: 두 패스가 모두 기여했다 ──`·`// ── 보강의 근거는 테스트 코드다 ──` 둘은
  바로 아래 `expect(...)` 의 실패 메시지가 같은 문장이다.
- **인라인 단정 재진술 5행** — `// Nothing else is claimable while the lease holds.`(다음 줄이
  `assert_eq!(claim(...), NO_CONTENT)`) · `// The queued job is visible on the home list.`(다음 블록이
  `GET /api/analyses` + `assert_eq!(jobs.len(), 1)`) · `// Nothing was queued.`(다음 줄이
  `SELECT COUNT(*) FROM analyses` = 0) · `// Only the executed stage moved …`(다음 줄이
  `assert_eq!(pending, STAGES.len() - 1)`).
- **시그니처·이름 재진술 5행** — `enqueue` 의 `/// Enqueues one analysis for `repo` and returns its id.` ·
  `real_state` 2행 · `cookie_value` 1행 · `WORKER_TOKEN` 1행(상수의 이름과 값이 그대로 말하고,
  꼬리의 `(see tests/worker.rs)` 는 **링크만의 교차 참조**라 README 가 제거 대상으로 적는다).
- **파일 머리 1행**(`scripts/e2e.sh`) — `# End-to-end: build → kind load → apply → port-forward →
  smoke + playwright → cleanup.` 는 아래 `[1/7]`~`[7/7]` echo 일곱 줄의 요약이다.
- **SKIP_BUILD 2행**(`scripts/e2e.sh`) — 로컬 동작은 바로 아래 `echo "… skipped (SKIP_BUILD=1,
  using prebuilt ${IMAGE})"` 와 에러 메시지가 그대로 말하고, 「CI 가 buildx 캐시로 미리 빌드한다」는
  워크플로의 자리다.
- **spec 설계 근거 3행**(`sc02-02`) — 「고정 개수를 세지 않고 두 출처의 관계를 본다」는
  `filter((s) => s.source === 'logic')` / `toBeGreaterThan(0)` / `toBeGreaterThanOrEqual(...)` 가
  보여 주는 것이고, 두 `expect` 의 실패 메시지가 의도를 적어 둔다.

### ② 저장소 문서 재진술 — AC 조항 · 시나리오 원문 · 테스트 이름 열거

- **`worker.rs` 모듈 머리 8행 전건.** `The worker queue protocol (AC4.5): authentication, atomic
  claim, lease reclaim, and stage reporting.` 는 AC 꼬리표 + 아래 절 제목 넷의 열거다. 이어지는
  「in-process 라 클러스터가 필요 없다」는 `build_router(state).oneshot(...)` 호출과 파일 위치가
  말하고, 「클러스터 쪽 절반은 `sc04-07`·`sc04-08` 이 단정한다」는 **AC↔spec 매핑**이라
  `docs/doc-tracker.md` 의 자리다(3차 패스가 같은 이유로 걷었다). `docs/prd/04-platform.md` AC4.5
  「검증 방법: 워커가 다운되어도 API는 응답 가능. 워커는 수평 확장 가능」이 원본이다.
- **`analyses.rs` 모듈 머리 3행 전건.** `Analysis enqueue surface (AC1.1): list accessible repos,
  pre-flight estimate, trigger an analysis (queued), and reject out-of-scope targets without queuing.`
  는 AC 꼬리표 + 아래 테스트 다섯 개의 이름을 그대로 푼 것이다. 12·13차 패스가 `tests/auth.rs` ·
  `tests/crypto.rs` 에서 **이 유형이 낡아 거짓이 된 사례를 둘 적발**했다.
- **`sc02-02` 의 시나리오 2 기대 결과 축자 인용 — 두 벌 4행.** 파일 머리 2행과 본문 2행이
  `docs/test/02-feature-representation.md` 「시나리오 2」의 기대 결과 문장을 그대로 옮긴다. 그 파일은
  이미 **기계 판독 `// 검증 시나리오: 02-feature-representation.md#시나리오 2`** 로 자신을 가리키고
  있으므로, 인용은 복원 경로가 둘이나 있는 세 번째 벌이다.
- **`sc02-02` 의 AC2.2 제목 축자 1행** — `docs/prd/02-feature-representation.md` 의 제목이자, 같은
  파일 `test.describe('AC2.2: 테스트 코드가 인수 기준을 보강한다')` 의 문면이다.
- **`sc02-03` 교차 참조 2행** — 「모순 케이스는 이 파일의 몫이 아니다」는 `검증 시나리오:` 선언의
  1:1 매핑이 말하고, `tools/check-scenario-e2e.py` 가 그 1:1 을 강제한다.
- **`scripts/e2e.sh` 의 spec 11개 열거 4행 중 3행** — `(sc04-07, sc01-05, … sc01-03 … sc01-07,
  sc02-01 … sc02-04)` 는 임대 규약을 쓰는 spec 목록이라 **자매가 하나 늘 때마다 낡는다**. 정본은
  `e2e/support/cluster.ts` 의 임대 규약과 각 spec 의 `scaleWorkers` 호출이다.

### ③ 작업 흔적 — 슬라이스 번호

- **`worker.rs` 의 `// Widened again by slice 4b-1: …` 4행 → 2행.** 「slice 4b-1 이 또 넓혔다」는
  커밋 메시지·PR 의 자리이고, 「stage 3 이 구현돼 큐가 `fetch`·`cross_cutting` 옆에 그것도 내준다」는
  **바로 아래 `assert_eq!` 가 리스트를 그대로 적는다**. 남긴 것은 아래 「유지한 것」에.

### ④ 복제된 명제 — 정본을 한 곳으로

- **워커 임대 문단 4행 → 1행**(`sc02-02`). 정본은 `e2e/support/cluster.ts` 머리 11행이고
  (「*leased, not owned*」·`playwright.config.ts` 가 `workers: 1` 을 고정한다는 문장까지 거기 있다),
  같은 4행짜리 벌이 `sc01-01`·`sc01-06`·`sc02-01` 에도 있다. **이미 판정된 자매들이 쓰는 1줄 형태**
  (`sc01-03`·`sc01-05`·`sc02-03` 의 `// Leases the analysis worker — lease rules in
  `e2e/support/cluster.ts`.`)가 있으므로 미판정인 `sc02-02` 를 그 형태로 맞췄다.
  **앞 패스의 「유지」를 뒤집지 않는다** — 4행 형태가 남은 세 파일은 이미 판정된 범위라 건드리지
  않았고, 그쪽 압축은 증분 재판정의 몫이다(아래 「범위 밖」).

## 유지한 것 — 복원 불가능한 지식 (20행)

### `#![allow(dead_code)]` 가 그 자리에 있는 이유 (`common/mod.rs` 4행)

「통합 테스트 바이너리마다 이 모듈을 **inline 하지만 각자 필요한 헬퍼만 쓴다**」 — Rust 통합 테스트의
컴파일 단위 규약이라 코드에서 안 읽히고, 이 문장이 없으면 다음 사람이 `allow` 를 지우고 경고에
파묻힌다.

### 임시 DB 경로 충돌의 함정 (`common/mod.rs` 5행, 6행에서 압축)

`SystemTime::now().as_nanos()` 가 **유일하지 않다**는 것 — 벽시계가 나노초보다 굵게 움직여 병렬
테스트가 같은 값을 뽑고 한 DB 파일을 공유했다. 전형적인 실패 모드의 함정이라 유지한다. 다만
`docs/doc-tracker/2026-08.md` 가 「병렬 테스트가 같은 임시 DB 경로를 뽑아 간헐 실패하던
`tests/common` 결함 수정」으로 **증상 자체는 이미 기록**하므로, 증상 서술(「한쪽이 아직 마이그레이션
중인데 다른 쪽이 질의해 `no such table: users` 로 보였다」)만 걷고 **원인과 대책**을 남겼다.

### 동시성 계약 — 협조적 인터리빙 ≠ 스레드 경합 (`worker.rs` 2행, 3행에서 압축)

`many_workers_racing_never_claim_the_same_job_twice` 가 왜 앞 테스트와 **따로** 있어야 하는지.
`#[tokio::test]` 는 단일 스레드에서 협조적으로 인터리빙하므로 앞의 경합 테스트만으로는 진짜
스레드 경합에서의 안전성이 증명되지 않는다. 숫자(워커 6·스레드 4·잡 6)는 코드가 말하므로 걷고
**이 판별만** 남겼다.

### stub ↔ real 충실도 경계 (`worker.rs` 2행)

「stub 모드도 (합성) 단명 installation 토큰을 **발급한다** — 워커는 real 과 같은 모양의 것을
필요로 한다」. README 가 유지 대상으로 못박은 「stub 분기가 real 과 갈리는 지점」이다.

### 부재의 이유 — stage 4 는 두 가지 독립된 이유로 안 나온다 (`worker.rs` 2행, 4행에서 재작성)

`assert_eq!` 는 **있는 것**의 목록이라 **없는 것**을 설명하지 못한다. stage 4 가 큐에 안 나오는 것은
「미구현」과 「사용자 승인 게이트」 **둘 다**이므로, 구현만 해도 나오지 않는다. 슬라이스 번호와
AC 꼬리표, `tests/strategy.rs` 교차 참조는 걷고 이 판별만 남겼다.

### 백엔드와의 **복제 계약** (`sc02-02` 1행, 2행에서 압축)

`isTestPath()` 는 백엔드 `acceptance::is_test_path` 의 규약을 **복제한 것**이고 한쪽만 고치면
갈라진다 — 두 파일이 서로를 모르므로 코드에서 복원되지 않는다. 「문서가 주장하는 값이 아니라
경로로 판정한다」는 함수 본문이 그대로 보여 주므로 걷었다.

### 0 replicas 인데 rollout 을 기다리는 이유 (`scripts/e2e.sh` 2행, 4행에서 재작성)

오버레이가 워커를 0 으로 두는데도 `kubectl rollout status` 를 거는 것이 실수로 보인다. 완료 보고가
**Deployment 자체가 깨끗이 적용됐음**을 증명한다는 것이 남길 값이다. AC 꼬리표·spec 열거·「오버레이가
0 으로 둔다」(오버레이가 말한다)는 걷었다.

## 판단이 갈려 남긴 것

없다. 이 범위에서 애매해 보류한 주석은 없었다 — 제거한 68행은 전부 복원 경로를 축자로 짚을 수
있었고, 남긴 20행은 전부 위 여섯 범주 중 하나다.

## 검증 (판정 시점 로컬 실측, 부모 `7415771`)

- **주석 제거 후 부모와 동일 5/5.** 블록 주석(`/** */`)까지 걷어낸 소스의 해시가 부모와 같다.
  「비주석 diff 0행」이 아니라 이 방식으로 재는 것은, 머리 주석을 재작성할 때 **딸려 지워진 선언**
  (`mod common;` 같은)을 그 검사가 못 잡기 때문이다.
- `cargo test --tests` — **159 passed / 0 failed**, 3회 연속 동일.
- 문서 게이트 3종 rc=0 — `check-journey-mockup.py`(R0~R11) · `check-mockup-render.py` ·
  `check-scenario-e2e.py`. `sc02-02` 의 기계 판독 `// 검증 시나리오:` 선언 1건은 보존했다.
- `check-data-format-change.py --base 7415771 --head <head>` → **`✅ 변경 없음
  (review/data-format = success)`**.

> **무관한 기존 flake 1건.** `llm::tests::stub_is_deterministic_for_the_same_ask` 가 확률적으로
> 실패한다. 이 패스가 만든 것이 아니다 — **부모 `7415771` 에서 12회 중 3회, 이 패스 트리에서 12회 중
> 2회**로 같은 비율이고, `backend/src/llm.rs` 는 이 범위 밖이다. CI 가 붉으면 재실행으로 가린다.
> 정리는 이 모델의 판정 표면이 아니므로 별도 task 의 몫이다.
> **— 해소됨(2026-09-22 · PR #123 `77158c2` · `rct_20260922-0001`)**: 트리거 needle 이 공유 픽스처와
> 겹치지 않는 고유 문자열이 되어 이 flake 는 재현되지 않는다.

## 경합 — 열린 PR 3건, 파일 겹침 0

`/pulls?state=open` 전수(#91 반응형 레이아웃 · #92 슬라이스 6a · #93 OpenAI strict fix, 셋 다 base
`f5a2937`)의 `/pulls/<n>/files` 를 이 5파일과 대조해 **겹침 0**. 다만 셋 다 in-scope 주석 파일을
건드리므로 먼저 머지되면 **전역** 지문의 절대값은 움직인다 — 그래서 완료 기준을 절대 지문이 아니라
**「부모 대비 순 제거 68행」**으로 잡는다. 위 범위 지문은 이 5파일만의 값이라 자매 머지에 무관하다.

## 범위 밖 (후속)

- **사람 게이트 14파일 / 242행** — 무인 슬라이스의 후보 풀이 아니다.
  - `backend/migrations/*.sql` **8파일 / 139행** — 전용 PR · 수동 repair · 사람 승인.
  - **D2 저장 계층 핵심 4파일 / 44행** — `db.rs` 17 · `crypto.rs` 13(13차 패스가 판정만 마쳐 뒀다) ·
    `tests/migrations.rs` 13 · `models.rs` 1.
  - **D6·D5 2파일 / 59행** — `tools/check-data-format-change.py` 58 · `deploy/k8s/pvc.yaml` 1.
    **이 패스가 새로 가른 몫이다**(위 「무인 머지 경로가 없는 두 파일」).
- **무인 자유 풀 5파일 / 38행** — 다음 패스의 후보다:
  `deploy/k8s/kustomization.yaml` 15 · `backend/src/util.rs` 12 · `backend/src/main.rs` 5 ·
  `backend/src/error.rs` 5 · `backend/src/state.rs` 1. 넷이 `backend/src/**/*.rs` 라 D3 에 걸릴
  수 있어 보이지만, D3 는 **줄 단위**이고 `is_comment_only()` 가 `//` 로 시작하는 줄을 먼저 거르므로
  주석만 지우는 한 걸리지 않는다(이 패스의 `backend/tests` 와 달리 **실측으로 확인한 뒤** 집을 것).
- **증분 재판정 후보** — 워커 임대 문단의 4행짜리 벌이 `sc01-01`(3행) · `sc01-06` · `sc02-01` 에
  남아 있다. 셋 다 이미 판정된 범위(원장 3·9행)라 이 패스가 건드리지 않았다. 한 번에 1줄 형태로
  모으는 증분 재판정이 세 행을 동시에 갱신해야 한다.
- **디렉터리 단위 종료를 선언하지 않는다.** `backend/tests` 에는 `migrations.rs`(D2)가 남고,
  `e2e/tests` 와 `backend/src` 는 열린 PR 이 계속 파일과 주석을 더하는 중이다.

## 슬라이스를 고를 때 반드시 돌릴 것 (이 패스가 배운 것)

원장이 다음 축을 **파일과 행수까지** 지목해 두더라도, 집기 전에 **후보 트리에서 판정기를 직접
돌려 본다**:

```
python3 tools/check-data-format-change.py --base <main tip> --head <probe> --verbose
```

`✅ 변경 없음` 이 아니면 그 축은 무인 슬라이스가 아니다. 13차 패스는 `crypto.rs` 로 이 벽에
부딪혔고 원장에 D2 를 적어 뒀지만, **경로 규칙은 D2 하나가 아니다**(D1·D2·D5-pvc·D6). 「D2 목록에
없다」로는 충분하지 않고, 판정기를 돌린 출력만이 충분하다.
