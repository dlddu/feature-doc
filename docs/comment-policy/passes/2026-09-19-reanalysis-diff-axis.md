# 판정 상세 — 재분석 diff 축 비경합 7파일 (2026-09-19)

판정 범위: `backend/src/diff.rs` · `backend/tests/diff.rs` · `frontend/src/AnalysisDiff.tsx` ·
`e2e/tests/sc02-08-reanalysis-diff.spec.ts` · `backend/src/repo_scan.rs` ·
`frontend/src/AnalysisProgress.tsx` · `backend/src/lib.rs`

판정 시점 트리: `f298ecb`(main tip). 범위의 주석 **300행 → 132행**, 순 제거 **168행**.

규칙은 [README.md](../README.md), 결과 표면은 [ledger.md](../ledger.md)에 있다.

## 이 범위를 고른 이유

직전 패스(의존성 축)가 「`#75` 경합」으로 넘긴 것들이 그 사이 전부 풀렸다. `#75`(슬라이스 5b-2,
AC2.6 재분석 diff)가 머지되면서 신규 4파일과 그 커밋이 건드린 3파일이 동시에 자유 풀로 들어왔고,
같은 커밋이 원장 1행(`analysis.rs`)에 얹은 증분도 판정 가능해졌다. 한 기능의 전 층(순수 판정
로직 · 통합 테스트 · 화면 · e2e · 스캔 더블 · 라이브러리 머리)이 한 범위에 들어오므로, **같은
문장이 층마다 몇 벌로 복제됐는지**를 한 번에 볼 수 있다.

## 제거한 것 — 복원 경로별

### ① 저장소 문서 재진술 (docs/prd · docs/test · 여정 · 목업 · doc-tracker) — 74행

가장 큰 덩어리는 **화면 두 개의 머리 주석**이었다.

- `AnalysisDiff.tsx` 머리 34행 중 **30행**. 「이 화면이 왜 Feature Acceptance 의 한 절이 아니라
  독립 화면인가」(2026-09-18 「문서 권위 순서」 결정), 「비교 대상은 사용자가 고르지 않는다」
  (AC2.6 조항 재진술), 여정 `STP-scan-diff` 이탈 위험 문장의 통째 인용, 그리고 **「이 슬라이스가
  그리지 않는 목업 요소 셋」**. 마지막 것은 주석 자신이 "all registered in `docs/doc-tracker.md`
  «알려진 목업↔구현 편차» with 해소 시점 「슬라이스 6」"라고 적는다 — 원장이 이미 들고 있다고
  주석이 스스로 밝히는 경우다.
- `AnalysisProgress.tsx` 머리 24행 중 **20행**. 「2026-09-02 에 대조 보류에서 활성으로 올라왔다」는
  경위와, 뒤이어 나열한 편차 셋(측정 spend · 단계 제목 언어 · 링 vs 막대). 이 역시 주석이
  "every remaining difference is a row in «알려진 목업↔구현 편차» rather than something this
  comment holds"라고 적고는 세 개를 다시 적고 있었다.
- `sc02-08` 머리에서 **시나리오 8 의 사전 조건·기대 결과 통째 인용 12행**. 그 spec 이 어느
  시나리오를 검증하는지는 1행의 기계 판독 선언 `// 검증 시나리오:` 이 이미 말한다.
- `repo_scan.rs` 의 `REVISION` doc 이 인용하던 `docs/test/02#시나리오 8` 사전 조건,
  `e2e-mocking-policy.md` EXT-03 등재 위치.

### ② 선언·시그니처 재진술 — 41행

- `pub const ADDED: &str = "+"` 위의 「줄 한 개의 방향」, `changed_line_count` 위의 「목업의
  「변경 줄」 계량」처럼 바로 아래 선언이 이미 말하는 줄.
- `lib.rs` 의 「The binary (`main.rs`) is a thin wrapper that loads [`config::Config`], connects
  the database, and serves [`build_router`]」 — rustdoc 링크 셋으로만 이루어진 교차 참조다.
  `build_router` 의 「the `/hello` probe, the (future) `/api/*` surface」는 본문의 라우트 나열을
  옮긴 것이고 **`(future)` 는 이미 낡아 거짓**이었다(그 라우트들은 실재한다).
- `repo_scan.rs::scan` doc 의 `Mode::Stub`/`Mode::Real` 설명 — 바로 아래 `match` 두 갈래가
  같은 말을 한다.

### ③ 테스트 이름을 다시 쓴 doc — 27행

`src/diff.rs` 단위 테스트 9건과 `tests/diff.rs` 통합 테스트 5건, `repo_scan.rs` 3건의 doc 주석이
함수 이름을 한국어·영어로 옮긴 것이었다. 예: `an_unchanged_feature_produces_no_diff` 위의
「같은 답을 두 번 받은 재분석은 아무 줄도 만들지 않는다」.

### ④ 세 벌 이상 복제된 명제 중 잉여분 — 18행

이 축은 같은 명제가 **최대 네 곳**에 있었다. 각각 한 벌만 남겼다.

| 명제 | 있던 곳 | 남긴 곳 |
|---|---|---|
| 「견줄 상대가 없다」≠「바뀐 게 없다」 | `diff.rs` 머리 · `analysis.rs` 두 곳 · `tests/diff.rs` · `AnalysisDiff.tsx` · `sc02-08` | `AnalysisDiff.tsx`(화면이 두 배너를 가르는 자리) + `analysis.rs` 의 `compared_to` 필드 1줄 |
| 묻지 않은 것 ≠ 제거됨 | `diff.rs::feature_diff` doc · `analysis.rs::traced_dependencies` · `tests/diff.rs` · `sc02-08` | `diff.rs::feature_diff` doc(규칙이 사는 자리) |
| 리비전은 더하기만 한다 | `repo_scan.rs::REVISION` doc · 같은 파일 테스트 doc · `stub_scan_at` 인라인 | `REVISION` doc |
| 변경 0줄 feature 는 목록에 없다 | `diff.rs` 머리 규칙 3 · `tests/diff.rs` 머리 · `AnalysisDiff.tsx` 머리 · `sc02-08` | `AnalysisDiff.tsx` 머리 1줄 |

### ⑤ 구분선 · 절 제목 — 8행

`sc02-08` 의 `// ── 첫 번째 분석 ──`·`// ── 코드가 바뀐다 ──`·`// ── diff ──` 등 6개와
`analysis.rs` 의 `// ── 종단 의존성 (AC2.4 · AC2.5) ──`. 바로 아래 코드가 이미 그 절이다.

### ⑥ AC 꼬리표 — 문장은 남기고 괄호만

`(AC2.6)`·`(AC2.4 · AC2.5)`·`(AC1.5)`·`(AC4.3)`·`(AC2.1)`·`(AC4.7)`·`(AC3.3)` 같은 꼬리표는
번호가 `docs/prd/` 에서 복원되므로 뗐다. 문장 자체가 지식이면 문장은 남겼다 — 예:
`repo_scan.rs` 의 「Never interpolate the response or the token into the message」는 그대로 두고
`(AC4.3)` 만 뗐다.

## 유지한 것 — 복원 불가능한 지식

- **기계가 읽는 주석**(정책 대상 밖): `sc02-08:1` 의 `// 검증 시나리오:`,
  `repo_scan.rs:62` 의 `// mock-exception: EXT-03`.
- **`frontend/src/*.tsx` 머리의 목업 매핑 두 줄** — 아래 「발견」 참조. 기계가 읽는다.
- **상류·인프라 함정**: `lib.rs` 의 ANSI 색상 분기(파이프에서는 `kubectl logs` 가 읽히지 않는다),
  `repo_scan.rs` 의 `truncated` 트리 조용한 과소계수, 「없는 ref 에 측정값을 돌려주면 더블이
  실 API 보다 관대해져 fetch 실패가 프로덕션 밖에서 도달 불가가 된다」.
- **동시성·임대 계약**: `sc02-08` 머리의 Isolation 블록(워커 replica 수와 워커 env 두 가지를
  배포 전역에서 **임대**하고 `finally` 로 돌려준다, `workers: 1` 핀).
- **테스트 순서 함정**: `tests/diff.rs` 의 「읽는 순간 검토 가능한 전략이 심어진다(lazy seed).
  이 GET 없이 approve 를 치면 `load_strategy` 가 아직 없는 행을 찾아 404 다」.
- **env 경합 함정**: `stub_scan_at` 이 `stub_scan` 과 갈라져 있는 이유(프로세스 전역 env 를
  건드리지 않고 「두 번째 리비전은 더하기만 한다」를 시험하기 위해 — 같은 바이너리의 다른
  테스트와 경합한다).
- **저장 계약과의 결합**: `diff.rs` 머리에 남긴 「의존성의 정체성 축은 0008 의
  `UNIQUE(analysis_id, feature_key, category, name)` 과 같다」 — 둘이 어긋나면 저장에서 한 행인
  것이 diff 에서 두 항목이 된다. 코드 어느 쪽에서도 읽히지 않는 결합이다.
- **React 함정**: `AnalysisProgress.tsx` 의 「interval 안에서 읽되 의존성으로 만들지 않는다
  (매 tick 마다 타이머가 재시작된다)」.

## 판단이 갈려 남긴 것 (5건)

정책의 「애매하면 남긴다」를 적용했다.

1. `AnalysisDiff.tsx::markOf` 의 「`-` on the wire, `−` (U+2212) on screen」 — 본문이 같은 말을
   하지만, **어느 글리프가 목업의 것인가**는 목업 대조 게이트가 보는 축이라 남겼다.
2. `AnalysisDiff.tsx::Appbar` 의 「three-slot appbar … (M7 counts the slots)」 — 자리표시자
   `icon-btn ghost` 가 **왜 있는지**가 M7 게이트의 입력이다.
3. `diff.rs::Scenario::text` 의 「관측 가능한 결과인 `then` 을 쓴다」 — 세 필드 중 `then` 을 고른
   이유는 이름에서 복원되지 않는다.
4. `repo_scan.rs::ScanResult::paths` 의 「2단계가 증거원으로 쓰므로 다시 가져오지 않고 실어
   보낸다」 — 단계 간 결합이다.
5. `AnalysisProgress.tsx` 의 `Props.onBack` 주석(「Run in background — the job keeps running」) —
   뒤로 가도 잡이 계속 돈다는 것은 핸들러 이름에 없다.

## 발견 — 지문에 보이는 **기계 판독** 주석이 하나 더 있다

`tools/check-mockup-render.py` 의 `discover_screens()` 는 각 `frontend/src/*.tsx` 의 **앞 2,000자**를
읽어 `docs/mockups/<파일>.html#STP-<앵커>` 패턴을 찾고, 그것이 있는 파일만 「화면」으로 본다
(M1: 대조 범위 표의 화면 집합 == 발견된 화면 집합, 누락·유령 0). 즉 화면 머리의 목업 매핑 줄은
**사람이 읽는 주석이 아니라 게이트의 입력**이다. 지우면 그 화면이 M1 의 발견 집합에서 빠져
게이트가 붉어진다.

그런데 이 패턴은 정책 본문 「기계가 읽는 주석」 목록과 모델의 `DIRECTIVE` 제외 패턴에 **없다** —
`// 검증 시나리오:`·`mock-exception:` 과 달리 지문에 그대로 들어와 있고, 판정자가 모르면 지울 수
있다. 이번 패스는 두 화면(`AnalysisDiff.tsx`·`AnalysisProgress.tsx`)의 매핑 줄을 **그대로 두고**
그 아래 산문만 걷어냈다. 현재 매핑을 가진 tsx 는 10개다.

**다음 판정으로 넘긴다**: 제외 패턴에 `docs/mockups/.*\.html#STP-` 를 더할지 여부. 더하면 10개
화면의 매핑 줄 10행이 지문에서 빠지므로(값·행수 이동) 모델 정의를 바꾸는 control plane 작업이고,
data plane 패스가 단독으로 할 수 없다. 그때까지는 **화면 머리의 목업 매핑 줄을 제거 후보로 보지
않는다**는 것을 이 문서가 기록으로 남긴다.

## 증분 재판정 ④ — 원장 1행 `analysis.rs` (2026-09-19)

원장 1행이 「미판정 증분 36행 — `#75` 경합」으로 열어 둔 항목을, 그 경합이 풀려 닫는다. 판정
대상은 4차 패스(증분 재판정 ②) 이후 `analysis.rs` 에 더해진 **60행**이다 — `#71` 의 36행(종단
의존성 라우트·핸들러·뷰)과 `#75` 의 24행(재분석 diff 라우트·핸들러·뷰). 순 제거 **29행**,
`analysis.rs` 270행 → 241행, 1행 전체 616행 → **587행**.

제거 근거는 위와 같은 분류다.

- **절 제목** `// ── 종단 의존성 (AC2.4 · AC2.5) ──` 와 그 아래 「인수 시나리오가 파이프라인
  5단계인 것과 달리…(`docs/test/02` 시나리오 5 인용)」 7행 → 3행으로. 남긴 것은 「요청 하나가
  행을 남기고 분석을 재큐잉한다」는 큐 계약이다.
- **AC 조항 재진술**: `analysis_diff` doc 의 「AC2.6」·「삭제·복구는 AC3.3 의 몫」,
  `dependencies` 핸들러들의 「AC2.5's reverse query」·「AC2.5's export」·「the same reading AC2.1
  gets」·「an id that is not theirs does not exist (AC4.7)」.
- **여정 인용**: `evidence` 필드의 「the journey's exception table forbids filling it in」,
  `categories` 필드의 「the screen draws 「전체 7종」」.
- **선언 재진술**: `candidate_locations` 의 「이 분석의 후보가 발견된 자리 (`key` → `location`)」,
  `AnalysisDiffView` 의 「이번 재분석이 만든 차이」, `changed_line_count` 의 「목업의 「변경 줄」
  계량」, `key` 필드의 「in the body for the same reason the decision routes take it there」.
- **중복**: 라우트 테이블 위 3행이 바로 아래 핸들러 doc 과 같은 말을 하고 있었다.

유지한 것: `(created_at, rowid)` 정렬 이유(같은 초의 형제 분석), 「묻지 않은 것 ≠ 없던 것」과
0008 의 두 테이블 분리, 「요청이 재큐잉 없이 살아남으면 아무도 돌리지 않을 요청이 된다」는
한 트랜잭션 계약, 실패 후 재요청이 행을 `queued` 로 되돌린다는 것, feature key 가 경로 구분자를
품어 본문·쿼리로 받는다는 것.

## 이 패스가 병합되면

- 범위 7파일 지문: **132행** / `a0354a08cf044961cf4a88f1e07188fc2f8029091e09d889ab12c68bae3f234c`
- 원장 1행 지문: **587행** / `6b77b15a4cb16a13e9acdddfc86e94a935f3d4d5b02cd7e5f1c0452eb8850ba9`
- 전역(모델 versionScript, 후행 개행 제외): 부모 `f298ecb` 의 `lines=3464 files=110` /
  `0f8cdcef…` 에서 **순 제거 197행** → `lines=3267 files=110` /
  `fdb14462ceb06069dd97062ff9095b6338e03fe63af165ade0111c4c91750a78`.

전역 절대값은 열린 PR 이 먼저 머지되면 그만큼 움직인다 — 그때도 **순 제거 197행**은 그대로다.

## 범위 밖 (후속)

- `backend/migrations/*.sql` **8파일 / 139행** — 본문 「적용된 마이그레이션」 절의 전용 PR ·
  수동 repair · 사람 승인 게이트를 거치는 별도 패스.
- **열린 PR 접촉 8파일 / 369행** — `#77`·`#64`·`#26`·`#17` 의 `/pulls/<n>/files` 전수 실측
  (2026-09-19T02:5xZ).
- **자유 풀 나머지 56파일 / 1,315행**.
- 위 「발견」의 제외 패턴 확장 판단(control plane).

## 증분 재판정 ① — `frontend/src/AnalysisProgress.tsx` +4행 (2026-09-21 · `rct_20260921-0003`)

`#99`(`899800e`, 자매 모델의 수렴 슬라이스 ⑩)가 실패 안내를 정적 문장으로 바꾸고 3단계 카드의
진입 버튼을 걷으면서 쓴 주석을 판정했다. 지문에 잡힌 4행 중 **제거 3 · 유지 1**, 지문 밖 JSX 블록
연속행 3행은 함께 제거.

- **제거 — `subOf` 머리 3행** 「The server's reason is not drawn here: the mockup answers a failed
  stage with one standing sentence (below the list), and 시나리오 6 only asks that the stage can be
  re-run」. ② doc-tracker 변경 이력 ⑿ 「실패 단계의 서버 사유 대신 목업의 정적 안내 문장을 그린다」
  · ② `docs/test/01-analysis-pipeline.md` 시나리오 6 기대 결과 · ③ PR #99 계획 3 · ① 바로 아래
  `return '실패했어요'` 와 아래쪽 `data-testid="stage-failed"` 의 한 문장 — 삼중 복원.
- **제거 — JSX 블록 연속행 3행** 「Stage 3 is deliberately not one of them: the mockup walks this
  journey 진행 → 횡단 관심사 → 탐색 전략, so the way in is the cross-cutting screen's own CTA」.
  ② doc-tracker ⒃ · ③ PR 계획 2 「대체 경로는 이미 구현돼 있다(`open-cross-cutting` →
  `to-discovery-strategy`)」 축자. 줄머리가 기호가 아니라 지문에 없던 줄이라 **행 수에는 세지
  않는다**(원장 「지문과 사각지대」). 블록의 원래 2행(「A stage that produced a document gets a way
  into it …」)은 1차 판정이 유지한 것이라 그대로다.
- **유지 — 「At most one stage is failed at a time — the pipeline stops there」 1행.** 바로 아래
  `stages.find(…failed)` 가 첫 실패만 집는 것의 **전제**인 백엔드 불변식이다. `docs/prd` ·
  `docs/test` · `backend/src/pipeline.rs` 에 「한 번에 하나만 실패한다」를 문장으로 적은 곳이
  **0히트**(시나리오 6 사전 조건과 `FAILED` 상태 모델에서 유추는 가능하나 명시는 없다). 복원 경로가
  추론뿐이면 **애매하면 남긴다**(본문 「충돌 시 기본 방향」) — 판단이 갈려 남긴 것 1건으로 등재.
  지우려면 먼저 `pipeline.rs` 의 상태 모델 doc 이나 PRD 에 그 불변식을 적어야 한다(「원본을
  고친다」).

**건드리지 않은 것**: 화면 머리의 목업 매핑(M1 이 읽는다) · 1차 판정 유지분 · 동작 코드. stripper
잔여가 부모와 **바이트 동일**(9,908 == 9,908). `tsc -b && vite build` 통과.

결과: 범위 지문 `bbca5248…`(136, #99 직후) → **`4aa7a8eb…`(133)**. 1차 판정 시점 값
`a0354a08…`(132) 로 돌아가지 않는 1행이 위 유지분이다.

⚠️ PR #101(슬라이스 ⑪, `3d147d6`)이 이 패스의 머지 직전에 먼저 머지돼 같은 파일에 `STAGE_TITLES` 머리 주석
**2행**이 들어왔다 — 병합 트리의 범위 지문은 **`47fdf929…`(135)** 이고 그 2행은 미판정으로 남아
이 행에 다시 증분이 열렸다(다음 판정 대상: 「서버 `title` 은 enqueue 시점에 고정돼 화면이 문안을
소유한다」 — 그 근거가 doc-tracker 「문자열 소유자가 범위 밖」 행에 이미 있는지 볼 것).

## 증분 재판정 ② — `frontend/src/AnalysisProgress.tsx` +2행 (2026-09-21 · `rct_20260921-0005`)

**창.** `#101`(`3d147d6`, 자매 모델의 수렴 슬라이스 ⑪ — 파이프라인 5단계 제목의 표시 소유자를 화면으로)이
증분 재판정 ①의 머지 직전에 먼저 착지해 같은 파일의 `STAGE_TITLES` 머리에 지문 기준 **순증 2행**을 얹었다
(7행 133 → 135, 지문 원본 diff 는 `>` 2줄뿐 · 파일 집합 불변). 위 ①의 꼬리가 예고한 대로 이 절이 그 2행을
판정한다 — 원장 규약대로 새 행·새 파일 없이 7행의 결과 칸을 갱신한다.

| 위치 (`3d147d6` 트리) | 명제 | 판정 | 근거 |
|---|---|---|---|
| `AnalysisProgress.tsx:28` | 「Display titles keyed by the wire `key`」 | **제거** | ① 바로 아래 `STAGE_TITLES: Record<string, string>` 과 `titleOf()` 의 `STAGE_TITLES[stage.key]` 가 그 문장 자체. ② doc-tracker 슬라이스 ⑪ 「`AnalysisProgress.tsx` 가 stage `key` 로 목업의 한국어 제목을 고르고」. |
| `AnalysisProgress.tsx:28-29` | 「The server's `title` is persisted per analysis at enqueue, so rows seeded before a copy change would keep the old text」 | **제거** | ② `docs/doc-tracker/2026-09.md` 슬라이스 ⑪ 문단이 **두 벌**로 문장째 담는다 — 「구현 수렴 대기」 절(「시드 `title` 은 분석마다 DB 에 영속되므로 시드만 바꾸면 이미 시드된 과거 분석은 영문으로 남는데, 화면이 고르면 과거·미래 행이 함께 수렴한다」)과 변경 이력 표 행(「시드 `title` 은 enqueue 때 분석마다 `analysis_stages.title` 에 영속되므로 …」). ③ PR #101 본문 「왜 시드가 아니라 화면인가」 절이 같은 문장. 유일하게 「저장소 제약의 함정」(유지 목록)에 닿는 명제인데, 복원이 추론이 아니라 **명시 문장 두 벌**이라 「애매하면 남긴다」의 조건(복원 경로가 추론뿐)에 들지 않는다 — 같은 파일에서 ①이 「한 번에 실패하는 단계는 하나」를 `docs/` **0히트**로 남긴 것과 기준은 같고 사실이 반대다. doc-tracker 는 나아가 「다음 감지가 「백엔드 시드를 한국어로」를 다시 후보로 올리지 말 것」까지 적어 두어, 이 함정을 다음 편집자에게 전하는 자리가 이미 정해져 있다. |
| `AnalysisProgress.tsx:29` | 「the screen owns the copy」 | **제거** | ② 같은 문단 「표시 제목의 소유자를 화면으로 옮겼다」 · ③ PR #101 제목 「표시 소유자를 화면으로」 · ④ 커밋 메시지 제목이 같은 문장. ① `?? stage.title` 이 「서버 title 은 fallback」을 말한다. |

**증분 2행 중 제거 2 · 유지 0.** 물리 diff 는 `−2 / +0`, 비주석 코드 무접촉 — `//` 행과 공백을 걷어 낸
스트립 잔여의 md5 가 부모와 **동일**(`ce76e50e…`). 한국어 제목 5개·`titleOf()`·단계 카드 라벨·실패 안내
렌더 자리는 그대로이므로 자매 게이트 `check-mockup-render.py`(M3A·M3B·M5)의 입력도 변하지 않는다.

### 7행은 증분 재판정 ① 시점 값으로 돌아간다

| 원장 행 | ① 판정 시점 (#101 이전) | #101 이후 (`3d147d6` = ① 병합 트리 `f62dcf2`) | 이 재판정 뒤 |
|---|---|---|---|
| 7행 | 133 / `4aa7a8eb…` | 135 / `47fdf929…` | **133 / `4aa7a8eb…`** |

유지 0행이라 줄 수·지문 둘 다 ① 시점 값으로 **바이트 복귀**했다(지문 규약: 범위 파일 경로 접두사 포함 ·
정규화·정렬 · 후행 개행 포함 sha256 — `47fdf929…`(135) 가 이 규약으로 `f62dcf2` 에서 재현됨을 먼저 확인했다).
전역 as-is 는 부모 대비 **차분 −2**(`files` 불변)다 — `f62dcf2` 기준 `lines=2322`/`014b9fc4…` →
`lines=2320`/`22b4013b…`, 자매 #102(`rct_20260921-0002`, 10행 −4)가 먼저 착지한 트리에서는 2318 → **2316**.
원장 합계 문단은 머지 시점 main 이 적은 값에 이 차분을 더한다(절대값을 완료 기준으로 쓰지 않는다).

### 검증

1. 스트립 잔여 md5 부모 == head(위) · `git diff --stat` 코드 1파일 `−2`.
2. `frontend`: `npm ci && tsc -b && vite build` 통과(node 22). `python3 tools/check-scenario-e2e.py` ·
   `check-mockup-render.py` rc=0 — `// 검증 시나리오:` 선언 무접촉.
3. `python3 tools/check-data-format-change.py --base <main tip> --head <branch> --verbose` →
   `✅ 변경 없음 (review/data-format = success)` · 검사 파일 1(`AnalysisProgress.tsx`) — 원장 「슬라이스 전
   필수 절차」를 집기 전(`f62dcf2` 프로브 `edf0970`)과 준비 브랜치에서 각각 돌렸다.

이 절로 **판정 완료 범위 안의 미판정 증분은 0** 이 된다(#93 9행 → `rct_20260921-0001` · #91 5행 → `-0002` ·
#99 25행 → `-0003` · #101 2행 → 이 절). 다음 증분은 자매 모델의 수렴 슬라이스가 e2e·프런트를 다시 쓸 때
같은 형태로 열린다. 증분 밖 잔여(14파일/242행)는 원장이 적은 사람 게이트 셋 그대로다.

## 증분 재판정 ③ — #114(슬라이스 6d)가 연 15행 증분 · 2026-09-22

reconciler task `rct_20260922-0005`. `repo_scan.rs` **+14 −5**(제자리 재작성 — 같은 명제 5행이 한국어에서
영어로 다시 쓰였다) · `AnalysisDiff.tsx` **+6**. **순 제거 3행 · 유지 12행.**

### 제거 3행

- `Revision::First` 의 `/// The tree every first analysis sees.` — variant 이름이 그대로 말하는 ① 선언 재진술.
- `/// Paths this revision adds on top of the previous one.` — 필드 이름(`adds`)과 반환 타입의 재진술.
- `AnalysisDiff.tsx` 의 `onResolve` prop JSDoc(`/** 「부딪힌 곳 정리하기」 → the first conflict still open. */`) —
  이름과 화면 카피의 재진술. 같은 패스가 `ResolveConflict.tsx` 의 같은 모양 2건을 함께 걷었다.

### 유지 12행

- **「리비전은 더하기만 한다」 4행**(`Each revision only **adds** paths on top of the one before it — removing or
  renaming a path would move the feature keys built on it, and «the same feature's representation was updated»
  could no longer be observed.`) — **스텁 충실도의 불변식**이다. *왜* 더하기만 해야 하는지(feature 키가 경로
  위에 세워져 있어 관측 대상 자체가 움직인다)는 코드에도 docs 에도 문장으로 없다.
- **리비전 2·3 의 variant doc 5행** — 이 축이 **리비전 어휘의 정본**이다. 원장 4행(`acceptance.rs`)에 있던
  같은 명제의 사본 2건을 이 판정이 걷었으므로, 여기가 유일한 벌이 된다.
- `REVISION_THIRD_PATH` doc 1행 · 환경변수 파싱의 하위 호환 2행(「값을 세지 않던 시절의 spec 이 넣던 `2` 가
  그대로 두 번째로 읽힌다」 — 낡은 spec 과의 계약).
- `AnalysisDiff.tsx` 의 「한 곳을 읽어야 넘어간다」 표시의 수명 4행(`sessionStorage` 를 고른 이유 — 화면을
  떠나도 살아남지만 탭을 넘기지는 않는다)과 「배너가 세는 것은 충돌 행이 아니라 **기능**이다 — 한 기능에 두
  자리가 부딪혀도 한 건」 1행. 둘 다 화면 불변식이고 doc-tracker 는 배너의 존재만 적는다.
  (앞의 4행은 `/**` 여는 줄까지 지문에 3행으로 들어온다 — 본문 「지문과 사각지대」.)

줄 수·지문: 133 / `4aa7a8eb…` → (트리거) 148 → **145 / `2e366521…`**.
맥락 [2026-09-22-conflict-axis.md](2026-09-22-conflict-axis.md).

## 증분 재판정 ④ — `#121` · `#137` 이 원장 7행에 연 15행 (2026-09-24 · `rct_20260924-0001`)

등재된 **#121 1행**(`frontend/src/AnalysisProgress.tsx` — 「자매 착지 재실측」 2026-09-22)과, 그 뒤 착지한
**#137**(`fd6cdad`)이 `backend/src/repo_scan.rs` 에 연 **14행**을 함께 판정한다.

### ② 저장소 문서에서 복원되는 것 — `stub_read` doc 3행 (제거)

```rust
/// Answers only for paths the stub tree of the same repository and branch holds —
/// a path outside it is skipped exactly as the real `404` is, so the double never
/// hands stage 2 a file the real API could not.
```

`docs/e2e-mocking-policy.md` 39~40행이 **세 절을 모두** 적는다:

> `repo_scan::stub_read`는 같은 저장소·브랜치의 스텁 트리에 있는 경로에만 본문을 답하고, 그 밖의
> 경로는 real이 `404`를 건너뛰듯 건너뛴다 — 실 API가 줄 수 없는 파일을 2단계에 건네지 않는다

그 문서가 **충실도 경계의 정본**이고(자매 모델 `tbm_feature-doc-e2e-mock-policy` 의 판정 표면),
`#137` 이 같은 PR 에서 그 줄을 직접 넣었다. 주석은 그 등재의 사본이다. 정책 본문이 「stub 이 real 과
갈리는 지점」을 유지 대상으로 드는 것은 **그것이 대개 어디에도 없기 때문**이고, 여기서는 있다.

### ③ PR 본문에서 복원되는 것 — `read_files` doc 본문 5행 (제거)

```rust
///
/// A path that cannot be read (removed since the tree was listed, not text, a
/// transient 5xx) is left out rather than failing the caller: the excerpts are
/// context on top of the path list, and the stage still has its evidence without
/// them. A missing token is still an error, as it is for [`scan`].
```

`#137` PR 본문이 한 문장으로 적는다 — 「개별 파일 실패(404·5xx)는 건너뛰고 경로만으로 진행, 토큰
없음은 기존 스캔과 같은 오류」(③). 코드도 같은 말을 한다: 실패는 `tracing::warn!` 뒤 `continue`,
토큰 부재는 `ok_or_else` 로 `Err`(①). 마지막 문장은 **rustdoc 링크만의 교차 참조**(`[`scan`]`)라
정책이 따로 막는 형태다. **요약 1줄은 남긴다** — 같은 파일의 `pub async fn scan` 이 이 패스 이후
`/// Counts the blobs in `owner/name@branch` and sums their sizes.` 한 줄로 살아남은 그 규약이다.

### ① 코드에서 복원되는 것 — `excerpt` doc 2행 (제거)

```rust
/// Cuts on a UTF-8 boundary so a multi-byte character is never split into
/// replacement noise at the end of an excerpt.
```

바로 아래 `while !text.is_char_boundary(end) { end -= 1; }` 가 앞 절 그대로다(①). 뒤 절은 **사실과도
어긋난다** — `String::from_utf8_lossy` 가 이미 돌아 `text` 는 유효한 `str` 이므로, 경계 아닌 곳을
자르면 대체문자가 생기는 게 아니라 **패닉**한다. 낡아 틀린 사본은 제거 근거가 강해진다(정책 본문
「충돌 시 기본 방향」). `#137` PR 본문도 「UTF-8 경계에서 자르고」로 적는다(③).

### 2행 → 1행 재작성 — `truncated` 필드 doc

```rust
/// The file continued past `max_bytes`; the model is told so it does not
/// mistake a cut-off head for the whole file.
```

앞 절은 필드 이름 `truncated` 와 그 대입 `truncated: end < text.len()` 이 말하고(①), 「the model is
told」는 `cross_cutting.rs:199` 의 `let cut = if e.truncated { " (truncated)" } else { "" };` 가 말한다(①).
남는 것은 **왜 알려야 하는가** 하나다 — 잘린 앞부분을 파일 전체로 읽는 LLM 경계의 함정.
결과: `/// Surfaced to the model so a cut-off head is not read as the whole file.`

### 유지 1행 — `FileExcerpt` 요약

`/// The head of one file, as read for stage 2's context.` 는 `pub` 항목의 요약 1줄로 남긴다.

### 전건 제거 — `AnalysisProgress.tsx` 3행 (#121)

```jsx
{/* AC1.5 covers finished stages too. Only that stage re-runs; the ones
    behind it keep their result. Hidden while the job is queued or
    running — the server refuses a reset under a live lease anyway. */}
```

네 절이 모두 복원된다 — 「AC1.5 covers finished stages too」는 **AC 조항 재진술**(②, 이 패스 「⑥ AC
꼬리표」와 같은 형태) · 「Only that stage re-runs; the ones behind it keep their result」는 #121 PR 본문
(「끝난 단계를 다시 돌려도 뒤 단계는 초기화하지 않는다」, ③) · 「Hidden while the job is queued or
running」은 **바로 다음 줄의 가드** `!ACTIVE.has(analysis.status)`(①) · 「the server refuses a reset under
a live lease」는 #121 PR 본문(「분석이 `running` 이면 여전히 409」, ③)이다.
(지문에는 여는 줄 1행만 들어온다 — 본문 「지문과 사각지대」.)

### 값

판정 15행 · **순 제거 12행 · 유지 3행**(`repo_scan.rs` −11 · `AnalysisProgress.tsx` −1).
160(#137 착지 후) → **148 / `d55bd63e1adc1672800636757f49c4ef9e406ea6e6f9b17b0117f743feac6481`**.
이 행에 **미판정 증분은 남지 않는다.**
