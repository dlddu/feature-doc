# 판정 상세 — 후보·전략 축 비경합 8파일 (2026-09-19)

판정 범위: `backend/src/discovery_strategy.rs` · `backend/src/feature_candidates.rs` ·
`backend/tests/strategy.rs` · `backend/tests/candidates.rs` ·
`frontend/src/DiscoveryStrategy.tsx` · `frontend/src/FeatureCandidates.tsx` ·
`e2e/tests/sc01-04-strategy-edit-and-approve.spec.ts` ·
`e2e/tests/sc01-07-candidate-rejection-carryover.spec.ts`

판정 시점 트리: `e963a5f`(main tip). 범위의 주석 **362행 → 140행**, 순 제거 **222행**
(diff 기준 290행 삭제 · 65행 재작성).

규칙은 [README.md](../README.md), 결과 표면은 [ledger.md](../ledger.md)에 있다.

## 이 범위를 고른 이유

직전 패스(재분석 diff 축)가 닫히면서 원장 안의 미판정 증분이 0이 됐고, 잔여는 순수한 **슬라이스
잔량**만 남았다. 자유 풀에서 다음 축을 고를 때 기준은 앞 패스들과 같다 — **한 기능의 전 층을 한
범위에 넣어 같은 문장이 층마다 몇 벌로 복제됐는지 한 번에 본다.**

3단계(탐색 전략 제안)와 4단계(후보 추출)는 한 사용자 동선이다: 전략을 승인해야 후보가 열리고,
후보의 신원은 전략이 고른 경로다. 그래서 두 단계의 **순수 로직 2 · 통합 테스트 2 · 화면 2 ·
e2e 2** 를 함께 집었다. 실측 결과 같은 명제가 **최대 여섯 벌**까지 복제돼 있었다(아래 ④).

대안으로 「자유 풀에서 가장 큰 파일부터」(`frontend/src/api.ts` 96행)를 저울질했으나 버렸다.
`api.ts` 의 축 동료인 `App.tsx`(53행)가 **`#79` 경합**이라, `api.ts` 만 먼저 집으면 프런트 데이터
계층이 두 패스로 쪼개져 이 축의 핵심 관측(복제 벌수를 층 전체에서 한 번에 세기)을 잃는다.
`#79` 가 머지되면 둘을 함께 집는 것이 다음 패스의 첫 후보다.

### 경합 집합은 감지 브리프 이후 바뀌었다

detector 가 넘긴 브리프는 경합을 **5파일 / 212행**으로 적었는데, 계획 시점에 `/pulls` 를 다시
전수 실측하니 **7파일 / 296행**이었다. 그 사이 **`#79`(목업↔구현 수렴 슬라이스 ⑥ — 홈·연결 화면
병합)가 열렸기 때문**이다. `#79` 는 `sc01-02`(31행) · `App.tsx`(53행)를 새로 잡았고,
`ConnectRepository.tsx`·`HomeRepositories.tsx`·`index.css` 의 경합을 `#17` 과 겹쳐 늘렸다.
이 패스의 8파일은 `#79`·`#64`·`#26`·`#17` 네 PR 의 파일 목록과 **한 파일도 겹치지 않는다**
(`#64` 는 in-scope 주석 파일이 0건이라 애초에 경합이 아니다).

`#79` 는 `sc01-01`(원장 3행)도 건드린다 — 머지되면 원장 3행에 **미판정 증분이 새로 열린다**.
이 패스의 범위 밖이고, 다음 감지가 그 이동을 발화시킨다.

## 제거한 것 — 복원 경로별

### ① AC 조항·검증 방법 재진술 (docs/prd · docs/test) — 파일 머리 넷이 가장 큼

네 파일의 머리 주석이 **AC1.3 / AC1.4 의 검증 방법을 절 단위로 옮겨 적고** 있었다.

- `backend/tests/strategy.rs` 머리 17행 → **4행**. 「AC1.3's verification method has three
  clauses, and each has a test here」 밑에 세 조항(검토 · 수정 · 승인된 전략만 입력)을 나열했는데,
  그 셋은 바로 아래 테스트 **이름 자체**다(`the_reviewer_deletes_and_adds_and_the_provenance_survives`
  등).
- `backend/tests/candidates.rs` 머리 20행 → **4행**. 같은 형태로 AC1.4 의 네 동작(제시 · 승인/거부 ·
  병합/이름 변경 · 사유 기록)을 나열했다.
- `discovery_strategy.rs` 머리 17행 → **6행**. AC1.3 의 열거 조항을 **한국어 원문 그대로 인용**
  (「진입점 파일 패턴, 라우팅 정의 위치, UI 컴포넌트 경로, 외부에 노출된 API 엔드포인트, CLI 명령 등」).
- `feature_candidates.rs` 머리 22행 → **6행**. AC1.4 의 「각 후보에는 발견된 위치(파일·심볼)와
  추정 근거가 함께 기록된다」 인용.

두 e2e spec 의 머리에서는 **시나리오 제목 + AC 검증 방법 인용 5~7행**을 걷었다. 그 spec 이 어느
시나리오를 검증하는지는 1행의 기계 판독 선언 `// 검증 시나리오:` 이 이미 말한다 — 원장 3행
(e2e 하네스 축)이 같은 판정을 이미 했다.

본문에 남은 **AC 꼬리표 44줄**(`(AC1.3)`·`(AC1.4)`·`(AC4.7)`·`(AC1.5)`·「AC1.3's lazy seed」·
「AC1.3's gate」 등)도 번호가 `docs/prd/` 에서 복원되므로 뗐다. 앞 패스들과 같이 **문장이 지식이면
문장은 남기고 괄호만** 뗐다 — 예: `candidates.rs` 의 「404, not 403 — the API never confirms that
someone else's id exists」는 그대로 두고 `(AC4.7)` 만.

### ② 목업 · 여정 · doc-tracker 인용 — 27행

두 화면의 머리 주석이 다시 큰 덩어리였다.

- `DiscoveryStrategy.tsx` 머리 16행 → **5행**. 「The mockup is the SSOT for what this screen
  says」 선언, AC1.3 재진술, 그리고 **다섯 예시 패턴이 `data-sample` 이라 카피 게이트가 건너뛴다**는
  `docs/mockups/README.md` 「예시값 표기 규약」 인용 7행. 마지막 문장은 「it used to be carried as
  a row in `docs/doc-tracker.md` … which is no longer needed」 — **없어진 원장 행의 경위**까지
  적고 있었다. 경위는 커밋 메시지의 자리다.
- `FeatureCandidates.tsx` 머리 17행 → **5행**. 「Two kinds of difference from the prototype, both
  registered in `docs/doc-tracker.md` 「알려진 목업↔구현 편차」」 — 주석이 **원장에 등재돼 있다고
  스스로 밝히고는 그 두 건을 다시 적는** 형태다. 원장 4·7행이 같은 유형을 이미 판정했다.
- 두 화면의 `LOADING`/`APPROVED`/`NOT_EXTRACTED`/`MERGE_*` 상수 위 doc 블록 **9행** — 「Copy the
  static prototype has no counterpart for … All registered in `docs/doc-tracker.md`」. 이미 판정된
  `AnalysisDiff.tsx`·`FeatureAcceptance.tsx` 는 같은 상수들을 **doc 없이** 두고 있어, 이 축만
  남아 있던 형태다.
- `sc01-07` 의 「탐색 전략 화면의 승인 버튼이 곧 기능 후보 화면으로 가는 길이다(목업의
  `data-goto`)」, `DiscoveryStrategy.tsx::Props.onOpenCandidates` 의 `data-goto="STP-sift-candidates"`
  인용.

### ③ 선언·시그니처 재진술 — doc 주석 다수

- Rust 비공개 항목의 doc: `schema()` 위의 「The JSON shape the answer is constrained to」(두 파일
  모두), `landscape()` 위의 3행, `glob_match()` 위의 2행, `MAX_ENTRIES`/`MAX_CANDIDATES` 위의
  상한 설명(→ ④로 합침).
- `pub` 항목의 요약 중 **이름 밖의 정보를 한 비트도 더하지 않는 것**: `KINDS`(바로 아래가
  `["route","ui","api","job","cli"]`), `propose`/`extract`(「Runs stage 3/4 and returns the
  document to persist」), `patterns`/`candidates`(「…, in document order」), `Candidate` 구조체
  (「One extracted candidate as the review half stores it」).
- TS 쪽: `Props.onBack` 의 「Discovery Strategy → Analysis Progress」 화살표 주석 2건,
  `useState` 세 개 위의 「The candidate whose rejection panel is open, and the reason being
  written」 류 3행 — 변수 이름이 같은 말을 한다.
- `offered_stages` 위의 「Claiming is what a worker does; the assertion is about what the queue
  *offers*」.

> **판정 규칙(이 패스가 적용한 것)**: 정책 본문은 「Rust 는 `pub` 항목의 요약 1줄 `///` 을
> 유지한다」이고 동시에 「doc 주석 본문이 시그니처·타입을 되풀이하는 부분은 제거 대상」이다.
> 이 패스는 둘을 **「이름 밖의 정보를 1비트라도 더하면 남기고, 이름·시그니처를 영어로 옮긴
> 것뿐이면 제거」**로 갈랐다. 그래서 `detail()` 의 요약은 두 파일 모두 **남겼고**(「detail」 이라는
> 이름만으로는 그것이 Analysis Progress 의 한 줄이라는 것이 복원되지 않는다), `propose()` 의
> 요약은 제거했다(모듈 머리가 이미 「Stage 3」 이라고 말한다).

### ④ 같은 명제의 복제 — 최대 여섯 벌

이 축의 가장 큰 덩어리다. 각각 **한 벌만** 남겼다.

| 명제 | 있던 곳 (벌수) | 남긴 곳 |
|---|---|---|
| 화면이 아니라 서버가 기억한다 — 새로고침이 그 증거 | 두 화면 머리 2 · 두 화면 `mutate` doc 2 · `sc01-04` 인라인 · `sc01-07` 인라인 (**6**) | 두 화면 머리 각 1줄 |
| 후보의 신원은 **위치**다 (행 id 도 이름도 아니다) | `feature_candidates.rs` 머리 · `candidate_key` doc · 같은 파일 테스트 인라인 · `backend/tests/candidates.rs` rename 인라인 (**4**) | `candidate_key` doc |
| 승인 전에는 큐가 4단계를 내주지 않는다 | `discovery_strategy.rs` 머리 · `strategy.rs` 머리 · `strategy.rs` 테스트 doc · `candidates.rs` 머리 · `sc01-04` 머리 · `sc01-07` 인라인 (**6**) | `strategy.rs` 테스트 doc + `sc01-07` 인라인(사용자 쪽에서 본 모습) |
| 워커는 **임대**다 — `finally` 로 돌려준다 | 두 spec 머리 Isolation 2 · 두 spec `finally` 인라인 2 (**4**) | 머리 Isolation 블록 각 1 |
| 상한(cap)은 사람이 검토하기 때문이다 | `MAX_ENTRIES` doc · `propose` 인라인 · 캡 테스트 doc · `MAX_CANDIDATES` doc · `extract` 인라인 (**5**) | 각 파일의 인라인 1벌씩(집행 자리) |
| 사용자가 보탠 항목은 다음 분석으로 이어진다(화면 카피 인용) | `strategy.rs` 머리 · `strategy.rs` 테스트 doc · `sc01-04` 인라인 (**3**) | `strategy.rs` 테스트 doc |
| 「아직 제안 전」 ≠ 「제안했는데 비었다」 (404 대 빈 목록) | `sc01-04` 인라인 · `strategy.rs` 인라인 (**2**) | `strategy.rs`(집행 자리) |
| 전략 행은 첫 읽기에 심어진다(lazy seed) | `candidates.rs::approve` 인라인 · `sc01-07::runToCandidates` 인라인 (**2**) | 둘 다 1줄로 축약해 유지 — 두 하네스가 독립적으로 이 순서를 지켜야 한다 |

### ⑤ 테스트 이름을 다시 쓴 doc

`discovery_strategy.rs` 단위 테스트 3건(`every_entry_cites_a_path_from_the_tree` 위의 「AC1.3's
proposal is only reviewable if it points at things that exist」 등), `strategy.rs`·`candidates.rs`
의 테스트 doc 일부. 이름이 문장인 테스트에 문장을 다시 붙인 형태다.

### ⑥ 구분선 · 절 제목 — 21행

두 e2e spec 의 `// ── setup: this spec's own user … ──` · `// ── 승인 ──` · `// ── 검토: 지운다 ──`
류 **21개**. 바로 아래 코드가 이미 그 절이다. 이미 판정된 e2e spec 넷(`sc01-01`·`sc01-06`·
`sc02-05`·`sc02-08`)에는 이런 구분선이 **0개**라 이 축만 남아 있던 형태다.

## 유지한 것 — 복원 불가능한 지식

- **기계가 읽는 주석**(정책 대상 밖): `sc01-04:1`·`sc01-07:1` 의 `// 검증 시나리오:` 2건,
  `discovery_strategy.rs` 의 `mock-exception: LLM-01` 2건, `feature_candidates.rs` 1건.
- **`frontend/src/*.tsx` 머리의 목업 매핑 2줄** — `tools/check-mockup-render.py::discover_screens()`
  가 앞 2,000자에서 읽는 M1 의 입력이다(직전 패스의 「발견」). `(Discovery Strategy, AC1.3)` 같은
  **꼬리표만 떼고 매핑 자체는 그대로** 뒀다. 판정 뒤 `discover_screens()` 를 직접 호출해 두 화면이
  발견 집합에 그대로 있음을 확인했다.
- **카피 추출기(M3B) 함정 3건** — 이 축에서만 나오는 도구 제약이다.
  - `DiscoveryStrategy.tsx::mutate` 가 thunk 가 아니라 **이미 시작된 요청**을 받는 이유:
    `() => Promise<Strategy>` 라는 타입 문자열이 M3B 카피 추출기에 **제품 카피로 읽힌다**.
  - `quotedRejection`·`locationOf` 가 템플릿 리터럴 대신 **문자열 연결**을 쓰는 이유: 추출기가
    문자열 리터럴을 통째로 읽어 보간 리터럴을 카피로 등재해 버린다.
  둘 다 코드만 보면 「왜 이렇게 썼지」로 읽히고, 되돌리면 카피 원장에 영원히 참이 되지 않는 행이
  생긴다.
- **Playwright 함정**: `sc01-04` 의 「다중 요소 로케이터에 `.not.toContainText` 를 걸면 strict
  mode 위반이고, 부분 문자열 대조는 `payments-api/**` 처럼 서로 접두사인 패턴에서 공허해진다」.
- **테스트 격리 함정**: 두 spec 의 「오버레이가 이미 0 이지만 전제를 명시해 큐에 있던 잡이 「아직
  아무것도 안 돌았다」 단정 전에 드레인되지 않게 한다」.
- **동시성·임대 계약**: `strategy.rs::requeue` 의 「쥐고 있는 임대가 다음 단계를 가린다」,
  `propose` 의 「`/internal` 쓰기는 임대로 보호되므로 먼저 claim 한다」, `candidates.rs` 의
  「`approve_strategy` 는 살아 있는 임대 아래의 잡을 재큐잉하지 않으므로 `finish` 가 알아채야
  한다」와 「승인이 워커가 임대를 쥔 채 도착한다」.
- **저장 계약 함정**: `candidates.rs` 의 「`created_at` 은 unix **초**라 같이 트리거된 두 분석이
  동률이 된다 — 이월 쿼리는 rowid 로 동률을 깨고, 여기서 첫 분석을 소급해 **동률이 아니라 정렬
  조항**을 시험한다」. 이 주석이 없으면 소급 UPDATE 가 군더더기로 읽힌다.
- **stub 충실도 경계**: 두 `stub_answer` 가 트리·승인 패턴에서 **파생**되는 이유 — 고정 문자열로
  두면 배선이 끊겨도 e2e 가 통과한다.
- **상류 모델 동작**: 「instruction 을 믿지 않고 여기서 자른다 — 「at most N」 을 무시하는 모델이
  있다」, 「모델은 자기가 지어낸 위치를 인용한다」.
- **픽스처 독립 자세**: 두 spec 머리의 「기대값을 상수로 박지 않는다 … 픽스처를 바꿔도 이 테스트는
  여전히 옳고, 3·4단계가 근거 없는 패턴·후보를 만들어내는 순간에만 깨진다」. 원장 6행
  (`sc02-05`)이 같은 문단을 유지로 판정한 전례를 따랐다.
- **계층 분업**: 「큐가 다음 단계를 내주는지는 `/internal` 이라 브라우저에서 못 보므로
  `backend/tests/*.rs` 가 대신 지킨다」 — 원장 3행이 `sc01-01` 에서 같은 유형을 유지로 판정했다.
- **하네스 불변식**: 두 통합 테스트 머리의 「문서는 워커 자신의 `/internal` 경로로 쓴다 — 손으로
  넣은 행은 워커가 실제로 보내는 것과 어긋날 수 있다」.
- **CSS 함정**: 두 화면 `Appbar` 의 「오른쪽 `icon-btn ghost` 는 컨트롤이 아니라 **스페이서**다 —
  없으면 `.appbar` 의 `space-between` 이 제목을 오른쪽 끝으로 민다」. 이미 판정된
  `FeatureAcceptance.tsx`·`AnalysisDiff.tsx` 가 같은 문장을 유지하고 있어 형태를 맞췄다.

## 판단이 갈려 남긴 것 (6건)

정책의 「애매하면 남긴다」를 적용했다.

1. **두 `detail()` 의 요약 1줄** — 호출자를 grep 하면 복원되지만, `detail` 이라는 이름이 너무
   얇아 「무엇의 detail 인가」가 선언에서 읽히지 않는다.
2. `strategy.rs` 의 「Blank and duplicate entries are not a list the reviewer meant to make」 —
   바로 아래 단정이 정규화 결과를 보이지만, **왜 서버가 정규화하는지**(사용자 의도)는 단정에 없다.
3. `sc01-07` 의 「Not a 404 — 「아직 추출 전」은 화면이 그릴 수 있는 상태」 — `sc01-04` 의 404
   인라인과 짝처럼 보이지만 **판정이 정반대**(여기는 404 가 아니어야 한다)라 한 벌로 합칠 수 없다.
4. `candidates.rs` 의 「Kept, not deleted — the merge stays visible and reversible」 — 단정이
   `mergedInto` 를 확인하므로 앞 절은 복원 가능하지만, **되돌릴 수 있다**는 성질은 어느 단정에도
   없다.
5. `sc01-04` 의 「새 분석의 제안은 그 분석의 것이다 — 직전에 지운 항목이 여기서 다시 나타나도
   이상하지 않다」 — 이 주석이 없으면 「지운 항목의 부재를 왜 단정하지 않나」가 결함으로 읽힌다.
6. `feature_candidates.rs::matching` 의 패턴 방언 3줄(`**` 는 구분자를 넘고 `*` 는 못 넘으며,
   와일드카드 없는 패턴은 접두사) — `matches_pattern`/`glob_match` 를 읽으면 복원되지만, **손으로
   진입점을 보태는 사용자**에게는 계약이다(화면의 `cmd/admin-cli` 플레이스홀더가 그 경로다).

## 이 패스가 병합되면

- 범위 8파일 지문: **140행** / `d46c92626168878bf48e2c9605bb8636d3eead6b3bbd6165fcc23fedfaeb911f`
- 전역(모델 versionScript, 후행 개행 제외): 부모 `e963a5f` 의 `lines=3289 files=111` /
  `bc68c231…` 에서 **순 제거 222행** → `lines=3067 files=111` /
  `df0f9019d9423520ff0c4189e5fd43a5b2e5a9d37de905f6d86c7c0b0728c403`.

전역 절대값은 열린 PR(`#79`·`#64`·`#26`·`#17`)이 먼저 머지되면 그만큼 움직인다 — 그때도
**순 제거 222행**은 그대로다. 완료 기준은 절대 지문이 아니라 이 순 제거다.

## 검증 (판정 시점 로컬 실측)

- **비주석 변경 0줄** — `git diff -U0` 의 모든 `+`/`-` 줄이 주석 줄이거나 빈 줄임을 프로그램으로
  전수 대조했다(눈대중 diff 가 아니다). 코드·문자열·단정은 한 글자도 바뀌지 않았다.
- **문서 게이트 3종 rc=0** — `check-scenario-e2e.py` · `check-mockup-render.py`(M1~M7) ·
  `check-journey-mockup.py`(R0~R10).
- **기계 판독 주석 보존** — `// 검증 시나리오:` 27건 · `mock-exception:` 18건 · 목업 매핑 11건
  (전역 계수, 부모와 동일).
- `cargo test` 와 `check-journey-prototype.js`(node)는 호스트에 cargo·node 가 없어 로컬에서 돌릴
  수 없다 — CI 의 ci-gate 가 유일한 집행자다.

## 범위 밖 (후속)

- `backend/migrations/*.sql` **8파일 / 139행** — 본문 「적용된 마이그레이션」 절의 전용 PR ·
  수동 repair · 사람 승인 게이트를 거치는 별도 패스.
- **열린 PR 접촉(경합) 7파일 / 296행** — `#79`·`#26`·`#17` 의 `/pulls/<n>/files` 전수 실측
  (2026-09-19T04:5xZ). `#64` 는 in-scope 주석 파일 0건이라 경합이 아니다.
- **자유 풀 나머지 50파일 / 1,048행**. 큰 후보: `frontend/src/api.ts` 96 ·
  `tools/check-journey-mockup.py` 55 · `e2e/tests/sc01-03-cross-cutting-determinism.spec.ts` 45 ·
  `tools/check-scenario-e2e.py` 44 · `backend/src/pipeline.rs` 40 ·
  `e2e/tests/sc01-05-resume-after-app-exit.spec.ts` 39 · `frontend/src/CrossCuttingConcerns.tsx` 38 ·
  `backend/tests/progress.rs` 38.
- **`#79` 가 머지되면 원장 3행(e2e 하네스 축)에 미판정 증분이 열린다** — `#79` 가 `sc01-01` 을
  `+6 −4` 로 건드린다. 다음 감지가 그 이동을 발화시킨다.
- 직전 패스의 「발견」이 남긴 제외 패턴 확장 판단(`docs/mockups/.*\.html#STP-`)은 여전히
  control plane 몫이다. 그때까지 **화면 머리의 목업 매핑 줄은 제거 후보로 보지 않는다.**
