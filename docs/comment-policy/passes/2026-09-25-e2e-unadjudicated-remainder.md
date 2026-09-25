# 판정 상세 — e2e 잔여 미판정 2파일 (App 설치 헬퍼 · 단계 재실행 spec) · 2026-09-25

reconciler task `rct_20260925-0001`(모델 `tbm_feature-doc-comment-redundancy`) · 25차 패스.
22차 패스(#130)가 합계 문단에서 **「판정 행 밖이라 잔여로 들어갔다」**로 두 번 명시 인계한
새 파일 2개가 이 행의 전부다 — `e2e/support/github-app.ts` 4행(#141 유입)과
`e2e/tests/sc01-08-succeeded-stage-rerun.spec.ts` 13행(#121 유입).

> **이 문서의 측정 기재값은 PR head 의 base 에 묶인다.** 아래 「검증」의 부모 SHA·게이트 결과는
> 그때의 base 에서 잰 값이다. 자매 착지로 base 를 올리면 **원장 행뿐 아니라 이 파일의 수치까지
> 같이 다시 잰다.** 완료 기준은 절대 지문이 아니라 **부모 대비 순 제거 14행**이다.

| 파일 | 전 | 후 | 순 제거 | 요지 |
|---|---|---|---|---|
| `e2e/support/github-app.ts` | 4 | **1** | 3 | JSDoc 3행 → 요약 1행(정책 유지 조항) · 지운 2행은 `docs/e2e-mocking-policy.md` 충실도 보증 절이 **이 파일을 이름으로 들어** 같은 내용을 적는다 · 인라인 1행은 `request.get(url)` 재진술 |
| `e2e/tests/sc01-08-succeeded-stage-rerun.spec.ts` | 13 | **2** | 11 | 머리 블록 9행(시나리오 8 원문 축자 인용 · 소유 매핑 · 세 번째 사본이 된 임대 규약) · 절 제목 2행 |
| 합 | **17** | **3** | **14** | |

## `e2e/support/github-app.ts` — 제거 3행

### ① 제품 문서가 이 파일을 이름으로 들어 적는 내용 2행

```
 /** Gives the signed-in user the App installation by walking the URL the product
-  *  itself hands out. The stub install flow issues one id per user and the Setup
-  *  callback now refuses any other, so the id cannot be chosen by the caller. */
+  *  itself hands out. */
```

지운 두 절은 `docs/e2e-mocking-policy.md` 「충실도 보증」의 `verify_user_owns_installation` 항목이
그대로 말한다 — *「Setup URL이 실어 온 `installation_id`가 **그 사용자의 스텁 설치 id**
(`stub_installation_id(github_id)`)일 때만 통과시키고, 다른 사용자의 것은 real과 같은 `Forbidden`으로
거부한다 … 설치를 선행 조건으로 두는 스펙 10곳은 제품이 `/api/github/install-url`로 내주는 URL을
그대로 따라가며(`e2e/support/github-app.ts`)」*. 복원 경로 ②이고, 문서 쪽이 **이 파일을 이름으로
가리키므로** 원본이 사라질 위험도 없다. 「now refuses」의 `now` 는 경위 서술이라 복원 경로 ④이기도 하다.

남긴 첫 줄은 **export 함수의 JSDoc 요약 1줄**로, 정책 본문이 이름으로 열거한 유지 조항이다.

### ② `request.get(url)` 이 그대로 말하는 1행

```
-  // The Setup URL callback is a GET that redirects back into the SPA.
   expect((await request.get(url)).ok(), 'App Setup URL callback').toBeTruthy();
```

`request.get` 이 GET 임을 말하고, 리다이렉트 뒤 SPA 로 돌아가는 것은 `backend/src/github.rs` 의
setup 핸들러가 소유한다. 복원 경로 ①.

## `e2e/tests/sc01-08-succeeded-stage-rerun.spec.ts` — 제거 11행

### ③ 시나리오 원문 축자 인용 + 셋업 코드 재진술 + 소유 매핑 (빈 `//` 2행 포함 6행)

```
 // 검증 시나리오: 01-analysis-pipeline.md#시나리오 8
-//
-// 사전 조건(1~4단계 완료, 분석 멈춤)은 제품 표면만으로 만든다: 분석을 걸고, 전략을
-// 읽어 실체화한 뒤 승인하면 4단계가 돈다 — sc01-06 의 두 번째 arc 와 같은 길이다.
-// 전략 검토 화면 자체의 검증은 sc01-04 가, 후보 화면은 sc01-07 이 소유하므로 여기서는
-// API 로 걷는다(setup).
-//
```

- **「사전 조건(1~4단계 완료, 분석 멈춤)」** — `docs/test/01-analysis-pipeline.md` 시나리오 8 의
  *「사전 조건: … (1~4단계 완료, 분석 멈춤 상태)」* 축자 인용이다. 복원 경로 ②.
- **「분석을 걸고, 전략을 읽어 실체화한 뒤 승인하면 4단계가 돈다」** — 바로 아래 셋업 블록이
  그 순서 그대로다(`analysisOf` 폴링 → `discovery-strategy/approve` POST → `feature_candidates`
  폴링). 복원 경로 ①.
- **「전략 검토 화면 자체의 검증은 sc01-04 가, 후보 화면은 sc01-07 이 소유」** — 각 spec 머리의
  `// 검증 시나리오:` 선언이 소유를 정하고 `tools/check-scenario-e2e.py` 가 그 1:1 을 게이트로
  강제한다. 복원 경로 ①.

바로 위 `// 검증 시나리오:` 한 줄이 기계 판독 주석으로 남아 시나리오 원문으로 가는 링크 노릇을 한다.

### ④ 세 번째 사본이 된 임대 규약 3행

```
-// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
-// It scales the Deployment to 1 inside its own block and returns it to 0 in
-// `finally`; every assertion is about the job this spec created.
```

임대 규약 자체는 **동시성 계약**이라 정책의 유지 대상이고, 3차 패스가 원장 3행에서
`e2e/support/cluster.ts` 의 규약 블록을 그 이유로 **유지**했다. 이 블록은 그 계약의 **세 번째 사본**이며
자기 입으로 `(see e2e/support/cluster.ts)` 라 출처를 가리킨다 — 원본이 살아 있고
`import { scaleWorkers } from '../support/cluster';` 가 그 원본으로 가는 길이다. 2차·11차 패스가
같은 유형(「같은 문구 5회 반복」·「형제 게이트 셋이 각자 한 벌씩 적은 SSOT 서술」)을 제거한 선례를 따른다.
뒷문장 「returns it to 0 in `finally`」 는 3차 패스 ⑤(「`finally` 가 이미 말하는 문장」)와 같은 자리다.

**유지 원본은 건드리지 않았다** — `e2e/support/cluster.ts` 의 임대 규약 블록은 이 패스에서 0줄 변화다.

### ⑤ 절 제목 2행

```
-      // 사전 조건: 전략 승인 → 4단계까지 완료.
       expect(await strategyApproved(page, id)).toBe(false);
```

바로 아래 세 줄이 승인 POST 와 4단계 폴링이고, 시나리오 문서가 같은 문장을 사전 조건으로 적는다(②).

```
-      // 앞(입력)도 뒤(후보·승인)도 건드리지 않았다.
       for (const k of ['cross_cutting', 'feature_candidates']) {
```

시나리오 8 의 **기대 결과** 축자 인용이다 — *「그 입력인 횡단 분석은 다시 돌지 않고, 뒤 단계인
feature 후보 목록과 승인된 전략·후보 결정은 그대로 유지된다」*. 바로 아래 단정 메시지
(`stays succeeded` · `must not re-run` · `keeps its result`)가 같은 말을 영어로 한 번 더 한다.

## 유지 3행

| 파일 | 줄 | 유지 사유 |
|---|---|---|
| `e2e/support/github-app.ts` | `/** Gives the signed-in user the App installation by walking the URL the product itself hands out. */` | export 함수 JSDoc 요약 1줄 — 정책 유지 조항 |
| `sc01-08` | `// A *new* attempt: startedAt is cleared by the reset, so only a value that is` / `// present and different from the old one means a worker really ran it again.` | **단정 설계의 근거**. `startedAt` 이 재실행 리셋으로 `null` 이 된다는 것은 백엔드(`pipeline.rs`)의 동작이고 시나리오 문서·테스트 문서 어디에도 없다. 이것이 없으면 `s.startedAt !== null && s.startedAt !== before.startedAt` 라는 두 겹 단정이 왜 필요한지 복원되지 않는다 — 「애매하면 남긴다」 이전에 **복원 불가능** 쪽이다 |

## 검증

- **비주석 0줄** — `git diff -U0 | grep -E '^[+-]' | grep -v '^[+-][+-]' | grep -vE '^[+-]\s*(//|/\*|\*( |$)|\*/)'` 가 빈 출력.
- **행 24 재현** — 두 파일의 주석 줄을 전역과 같은 패턴·DIRECTIVE 제외·공백 정규화·정렬로 모아
  `printf '%s\n' | sha256sum` → `07b7fb6e37e0014995b39a1a1dbec946d787583a0e32417d5fcbe68a72126ce6`, 3행.
- **산술** — 행 열의 합 2,476 + 잔여 159 = 전역 2,635(`lines=2635 files=136`).
- **게이트** — `docs-journey-mockup`(R0~R11) · `docs-mockup-render` · `docs-scenario-e2e` ·
  `cargo test --release` · `check-data-format-change.py`(D1·D6 무접촉 → `review/manual-approval` 부착).

## 인계

- **잔여 159행은 전부 범위 밖이다** — D1 사람 게이트(`backend/migrations/0009`~`0013` 123행, sqlx
  체크섬 repair 선행)와 D6 판정기 자신(`tools/check-data-format-change.py` 36행). 이 둘 말고
  **미판정은 0행**이며, 원장 23행 전부가 줄 수·지문 둘 다 재현된다(23/23).
- 다음 유입은 **열린 PR #126**(docs-impl 슬라이스 6e)이 들여올 새 파일
  `backend/src/doc_history.rs` · `backend/tests/doc_history.rs` · `frontend/src/FeatureHistory.tsx` ·
  `e2e/tests/sc03-08-history-and-point-in-time-restore.spec.ts` 와 `backend/migrations/0014_*.sql`,
  그리고 그 PR 이 원장 7·10·21행에 여는 증분이다. 이 패스는 그 어느 파일도 건드리지 않는다.
