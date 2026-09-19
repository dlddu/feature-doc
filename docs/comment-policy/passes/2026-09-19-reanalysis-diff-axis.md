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
