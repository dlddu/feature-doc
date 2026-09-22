# 판정 상세 — 인수(acceptance) 축 (비경합 6파일)

- **판정일**: 2026-09-18
- **판정 범위**: `backend/src/acceptance.rs` · `backend/tests/acceptance.rs` ·
  `e2e/support/acceptance.ts` · `e2e/tests/sc02-01-acceptance-from-logic.spec.ts` ·
  `e2e/tests/sc02-04-user-facing-acceptance-doc.spec.ts` · `frontend/src/FeatureAcceptance.tsx`
- **기준 트리**: `ebe8657` (main, #60 머지 직후)
- **reconciler task**: `tbm_feature-doc-comment-redundancy/rct_20260918-0004`

규칙은 [../README.md](../README.md), 판정 결과의 표면은 [../ledger.md](../ledger.md)에 있다.
이 파일은 이번 범위의 **근거**만 담는다.

## 범위를 이 6파일로 고른 이유

**⑴ 이번 트리거가 들인 주석이 곧 이 범위다.** 잔여를 +337행 키운 것은 #43(`6484e21`,
슬라이스 5a)이 들인 **신규 7파일 / 383행**이고, 그 일곱이 곧 「인수 축」이다. 새로 들어온
주석은 PRD-2의 AC2.1·AC2.2·AC2.3 조항과 `docs/doc-tracker.md`의 편차 등재 항목을 **본문째**
되풀이한다 — 복원 경로 ②가 문면으로 증명되는 사례라, 판정이 갈릴 여지가 가장 작다.

**⑵ 그 일곱 중 여섯만 집었다 — 열린 PR 무접촉(기존 방침).** 일곱 번째
`e2e/tests/sc02-02-acceptance-from-tests.spec.ts`(31행)는 **열린 PR #67**
(「e2e 매핑 래칫 해제 — 02#시나리오 3 전용 spec 분할」, 2026-09-18T16:56Z 갱신)이 건드린다.
그 PR은 이 파일을 쪼개 `sc02-03`을 새로 내므로, 지금 같은 줄을 지우면 확실히 충돌한다.
다음 패스가 #67 머지 뒤에 집는다.

열린 PR 전수 대조(판정 시점 4건): **#67** — `sc02-02` 접촉(위). **#64** —
`tools/check-data-format-change.py`(신규 파일) 접촉. **#26**(draft, 08-31) —
`tools/check-mockup-render.py` 접촉. **#17**(draft, 08-30) — `frontend/src/ConnectRepository.tsx` ·
`CredentialsSetup.tsx` · `HomeRepositories.tsx` · `index.css` 접촉. 이번 6파일은 **넷 모두와
무접촉**이다.

**⑶ 직전 패스가 세운 경합 경계 하나가 그 사이 사라졌다.** 원장 ②가 「자매 모델
`tbm_feature-doc-e2e-mock-policy`의 PR(#59·#60) 뒤로 미룬다」고 적어 둔 12파일은, **#60이
`ebe8657`로 머지되면서**(2026-09-18T17:0xZ) 더 이상 경합이 아니다. 이번 범위는 그 축과
독립이지만, 이 사실은 다음 패스의 후보 풀을 넓힌다 — 아래 「범위 밖」에 적었다.

**⑷ 응집.** 여섯은 한 슬라이스가 한 번에 들인 파일이고, 백엔드 로직·백엔드 통합 테스트·
e2e 하네스·e2e spec 2개·프론트 화면으로 **한 기능의 전 층**을 이룬다. 같은 문장이 층마다
되풀이되는지를 한 번에 볼 수 있는 단위다.

## 집계

| 파일 | 판정 전(지문 기준) | 판정 후 | 지문 기준 감소 | diff |
|---|---|---|---|---|
| `backend/src/acceptance.rs` | 98 | 57 | 41 | 68행 삭제 · 27행 재작성 |
| `frontend/src/FeatureAcceptance.tsx` | 46 | 12 | 34 | 41행 삭제 · 6행 재작성 |
| `backend/tests/acceptance.rs` | 47 | 28 | 19 | 27행 삭제 · 8행 재작성 |
| `e2e/tests/sc02-01-acceptance-from-logic.spec.ts` | 28 | 11 | 17 | 17행 삭제 · 재작성 0 |
| `e2e/tests/sc02-04-user-facing-acceptance-doc.spec.ts` | 24 | 12 | 12 | 16행 삭제 · 4행 재작성 |
| `e2e/support/acceptance.ts` | 25 | 21 | 4 | 11행 삭제 · 7행 재작성 |
| **합계** | **268** | **141** | **127** | **180행 삭제 · 52행 재작성** |

재작성이 많은 것은 이 범위의 중복이 **한 블록 안에서 유지 문장과 섞여** 있기 때문이다 —
AC 조항 인용 한 줄만 들어내면 문단이 끊기므로 남는 지식을 한 문장으로 다시 적었다. diff
삭제(180)와 지문 감소(127)의 차는 재작성 52행과 블록 주석 닫는 줄(`*/` — 지문 패턴에 잡히지
않는다)이다.

## 지우지 않은 것부터 — 기계가 읽는 주석

- `// 검증 시나리오: 02-feature-representation.md#시나리오 1` (sc02-01) ·
  `#시나리오 4` (sc02-04) — **손대지 않았다.** `tools/check-scenario-e2e.py`가 파싱한다.
- `// mock-exception: LLM-01 …` 2건(`backend/src/acceptance.rs`의 두 `stub:` 인자) —
  `docs/e2e-mocking-policy.md`의 표기 규약이다. 그대로 뒀다.

둘 다 as-is 지문의 `DIRECTIVE` 패턴에 걸려 **지문에 보이지 않는다** — 실수로 지워도 수치가
알려주지 않으므로 아래 검증 절에 개수 대조를 따로 넣었다.

## `backend/src/acceptance.rs` — 41행 제거

### ① PRD-2 조항을 통째로 옮겨 적은 모듈 머리 (19행)

머리 28행 중 **AC2.1·AC2.2·AC2.3의 설명을 항목별로 다시 쓴 9행**이 핵심이다.

```
//!   * **AC2.1** — read the feature's *logic* and say what a user experiences, as
//!     인수 기준 ("주어진 ~ / ~ 했을 때 / ~ 해야 한다"). …
//!   * **AC2.2** — read the feature's *test* code as well and **보강**: …
//!   * **AC2.3** — hand back one document per feature, in end-user language.
```

`docs/prd/02-feature-representation.md`의 AC2.1~AC2.3 **설명 칸과 문장 단위로 대응**한다
(「주어진 ~ / ~ 했을 때 / ~ 해야 한다」는 AC2.1 원문의 인용부호까지 같다). 복원 경로 ②.

이어지는 「두 번 호출하는 이유」·「모순 규칙은 코드다」·「이 단계가 하지 않는 일」 세 문단은
**압축해 남겼다**: 두 번 호출이 보강을 관측 가능하게 한다는 설계 근거와, 모순 판정이 모델의
판단이 아니라 규칙이라는 사실은 코드에서 복원되지 않는다. 다만 그 안의
`merge`가 무엇을 모순으로 보는지를 서술한 3행(같은 `given`+`when`에 다른 `then`)은 **바로 그
`merge`의 본문**이라 지웠고, `test/02 시나리오 3` 인용도 지웠다.

`acceptance_dependencies`라는 와이어 키가 의존성 작업을 하지 않는다는 **함정 한 줄은 남겼다**.
그 경위(「AC2.1~AC2.6이 한 슬라이스였을 때 로드맵이 고른 키」·「AC2.4~AC2.6은 `docs/test/02`
시나리오 5의 feature 단위 액션」·「목업이 이 단계를 「인수 시나리오 생성」으로 그린다」)는
`docs/doc-tracker.md` 2026-09-02 5a 행이 **그대로 담고 있어** 지웠다 — 복원 경로 ②.

### ② 상수·함수 doc 안의 문서 인용 (8행)

- `MAX_FEATURES`: `JRN-review-feature`의 「12개 feature가 있으면 12번 반복된다」 인용 → 삭제.
  남긴 것은 「상한은 실행 비용의 문제이지 화면 길이의 문제가 아니다」 — 판단이다.
- `MAX_SCENARIOS`: 「which is AC2.3's whole point」 → 삭제.
- `is_test_path`: `AC2.2 says "동일 feature와 연관된 **테스트 코드**"` 인용 → 삭제. 「여기가
  그 질문에 답하는 유일한 자리라 프롬프트·검증·e2e가 어긋날 수 없다」는 불변식은 유지.
- `scenarios_for`: 「AC2.1's "근거가 된 코드 위치가 첨부된다"」 인용 → 삭제. 「지시를 믿는
  대신 답을 검사한다」는 근거는 유지.
- `stub_logic`·`stub_tests`: AC2.1/AC2.2 재진술과 `test/02 시나리오 3` 포인터 → 삭제.
  **stub이 real과 갈리는 지점**(고정 문자열이 아니라 트리에서 파생한다 / 테스트 파일이 없는
  저장소는 테스트 패스 자체가 없다)은 정책이 이름 붙인 유지 대상이라 그대로 뒀다.

### ③ 코드가 이미 말하는 문장 (6행)

- `situation`의 「같은 상황에 다른 결말이면 AC2.2가 드러내라는 모순이다」 → `merge` 본문.
- `merge`의 「순서는 승인된 feature, 그 로직 시나리오, 그다음 새로 더한 테스트 시나리오」
  4행 → 바로 아래 루프가 그 순서 그대로다. 「로직 문장은 항상 남는다」는 **이유**는 유지.
- `merge` doc 끝의 목업 카피 인용(「어느 쪽이 맞는지는 이 코드를 쓰신 분만 판단할 수 있어요」)
  → `frontend/src/FeatureAcceptance.tsx`에 **실제 렌더되는 문자열**이고 목업이 SSOT다.

### ④ AC 라벨만 붙은 단계 표지 (2행)

`// AC2.1 — the logic pass.` · `// AC2.2 — the test pass, skipped outright when …`.
앞은 바로 아래가 `SYSTEM_LOGIC`을 쓰는 `llm::ask`라 라벨이 없어도 같다. 뒤는 라벨만 걷고
「없는 입력으로 모델을 부르는 것은 사용자 돈을 쓰는 일이다」 2행을 남겼다 — 비용 판단이다.

### ⑤ 단언을 되풀이한 테스트 주석 (4행)

`// A directory that merely *starts* with "test" is not a test directory.`(바로 아래가
`assert!(!is_test_path("payments-api/testimonials/page.tsx"))`) ·
`// 보강: … (AC2.2's 검증 방법 — "시나리오 수가 같거나 많다")` ·
`// 분리: the disagreeing sentence goes to its own section and nowhere else.` ·
`// The test pass's *other* criterion is in the list, marked as its own source.`
— 넷 다 바로 아래 단언과 그 실패 메시지가 같은 말을 한다. 복원 경로 ①.

## `frontend/src/FeatureAcceptance.tsx` — 34행 제거

### ① doc-tracker에 등재된 편차를 그대로 옮겨 적은 10행

```
// Two things the mockup draws that this slice does not implement, both registered in
// docs/doc-tracker.md "알려진 목업↔구현 편차" with 해소 시점 = 슬라이스 5b:
//  · the resume banner and the 검수 상태 …
//  · `근거가 진짜인지 확인하기` — the entry to `STP-verify-evidence`. …
```

**주석 스스로 「등재돼 있다」고 적는다.** 실제로 `docs/doc-tracker.md` 446·447·448행이
같은 세 건을 사유까지 담아 들고 있고, 그 표는 `tools/check-mockup-render.py`의 M4·M5가
검사하는 **원장**이다. 원본이 고쳐질 때 이 사본만 조용히 거짓이 되는 형태의 교과서적
사례라 전부 지웠다. 같은 이유로 `LOADING`·`NOT_GENERATED` 위의 4행(원장 449행 재진술)도
지웠다.

### ② PRD-2 재진술 + 목업 SSOT 서술 (6행)

「What it *does* is PRD-2's first half: stage 5 read the confirmed feature's logic
(AC2.1) and its tests (AC2.2) and wrote one document per feature in end-user language
(AC2.3)」 — PRD 조항 셋을 한 문장에 다시 담았다. 「카피는 목업의 카피다」도 목업이 SSOT라는
규약 자체의 재진술이고 `check-mockup-render.py`가 매 PR 검사한다.

**남긴 것**: 「이 화면의 모든 문장은 서버에서 왔다 — 그래서 새로고침해도 같은 문서가 보이고,
잃어버릴 로컬 초안이 없다」. 상태 설계의 불변식이고 코드 어디에도 한 줄로 적혀 있지 않다.

### ③ 앱바 경위 (11행 → 3행)

`Appbar`의 doc 12행은 **#65(`6bc37f9`)의 커밋 메시지와 `docs/doc-tracker.md` 2026-09-18
슬라이스 ② 행이 같은 내용을 더 자세히** 담고 있다 — 3슬롯 구조, ✕의 목적지를 지어내지 않은
근거, 재개 지점이 5b라는 것까지. 복원 경로 ②④.

**남긴 3행**: 왼쪽 슬롯의 `icon-btn ghost`가 컨트롤이 아니라 `space-between` 아래에서 제목을
가운데 두는 **자리표시자**라는 것. 이건 지우면 「죽은 버튼」으로 보여 다음 사람이 없앨 위험이
있고, 없애는 순간 제목이 다시 오른쪽으로 붙는다 — 복원 불가능한 함정이다.

### ④ 이름 재진술 (3행)

`onBack`·`onOpenCandidates` prop doc 2행과 `notAFeature` state doc 1행. 셋 다 식별자가
그대로 말한다.

## `backend/tests/acceptance.rs` — 19행 제거

머리의 **게이트·커버리지 2항목 서술 7행**은 바로 아래 테스트 셋의 이름
(`stage_five_is_withheld_until_a_feature_is_confirmed` ·
`rejecting_a_candidate_does_not_requeue_the_analysis` ·
`confirming_a_second_feature_reopens_the_stage`)과 각 테스트의 doc이 다시 말한다 — 같은 파일
안에서의 자기 중복이다. 「단위 테스트는 `src/acceptance.rs`에, 여기는 앱 전체만 답할 수 있는
것」이라는 **경계**와 「픽스처 대신 워커의 `/internal` 경로로 쓴다」는 **근거**는 남겼다.

본문에서 지운 것은 전부 **바로 아래 단언의 되풀이**다: `try_claim`의 「`None`은 204」
(코드가 그 분기 그대로다) · 「Nothing decided yet: … 큐가 비어 있다」 · 「Confirming one
feature opens it」 · 「Everything confirmed is documented」 · 「…until a second feature is
confirmed」 · `// AC2.1:` · `// AC2.2:` · 「…and a disagreement is kept out of the
scenario list」. AC 라벨 둘은 그 아래 단언이 필드 존재·경로 소속·`source == "test"`를 직접
검사하므로 라벨이 정보를 더하지 않는다.

**남긴 것**: 「거부는 아무것도 열지 않으므로 재큐잉하지 않는다 — 없으면 전부 거부된 분석이
영원히 돌 수 있다」(트랩), 「문서는 분석당 하나라 두 번째 확정이 단계를 다시 열어야 한다 /
「단계가 성공했는가」로 잠그면 두 번째 feature는 영영 문서를 못 갖는다」(술어 함정),
「stage가 돌기 전은 404 — 「아직 안 돎」과 「찾은 게 없음」의 구분」, 「남의 분석 id는 403이
아니라 404 — API는 id의 존재를 확인해 주지 않는다」(보안 불변식), `SIFTED`·`tree()`의
「stub 트리와 같은 출처를 쓴다」(커플링).

## `e2e/tests/sc02-01` · `sc02-04` — 29행 제거

### ① 파일 제목 + AC 검증 방법 통째 인용 (13행)

```
// AC2.1 (로직 코드로부터 인수 기준 도출) 전용 spec.
//
// AC2.1 의 검증 방법 두 가지를 그대로 따라간다: … **"주어진 ~ / ~ 했을 때 / ~ 해야 한다"**
// 형태로 생성되고, **각 인수 기준에 근거가 된 코드 위치가 첨부**된다.
```

제목은 PRD의 AC 제목 그대로이고, 인용 두 줄은 **AC2.1 검증 방법 칸의 문장**이다. sc02-04도
같은 형태로 AC2.3의 요구 셋을 옮겨 적었다. 그 문서를 가리키는 포인터는 **바로 위
`// 검증 시나리오:` 선언**이 이미 갖고 있고(그 줄은 남는다), AC↔spec 매핑은
`docs/doc-tracker.md`의 자리이며 `check-scenario-e2e.py` S3가 그 표를 검사한다.
직전 패스가 sc01-01·sc01-06에서 지운 것과 **같은 형태**다.

sc02-01의 「이 spec 이 브라우저에서 관측하는 또 하나 …」 3행도 지웠다 — 테스트 이름
(「확정 전에는 문서가 없고, 확정하면 …」)과 `acceptanceOf`의 JSDoc(「404면 `null`」)이 같은
말을 한다.

### ② 절 제목(구분선) 4행 + 단언 재진술 6행

`// ── 확정 전: 쓸 문서가 없다 ──`(아래가 `expect(await acceptanceOf(page, early)).toBeNull()`) ·
`// ── 워커를 켜고 확정까지 걸어간다 ──`(아래가 `scaleWorkers(1)` · `runToAcceptance`) ·
`// ── 형태: 주어진 / 이럴 때 / 이렇게 됩니다 ──` · `// ── 인수 시나리오 화면이 그 문서를
그린다 ──` · sc02-04의 `// ── 인수 시나리오 화면: 기능을 고르면 그 기능의 문서만 그린다 ──`.
정책이 이름 붙인 중복 유형 ④.

단언 재진술은 「확정한 feature 의 문서다」 · 「근거: 각 인수 기준에 코드 위치가 첨부되고 …」 ·
「최종 사용자의 언어: 시나리오 본문에 개발자 어휘가 없다」(바로 아래가
`DEVELOPER_VOCABULARY.test(part)`와 실패 메시지 「개발자 용어가 남아 있다」) · 「모든
시나리오에 근거 위치가 붙는다」(실패 메시지가 「근거 없는 시나리오」) · 「다른 기능의 문장이
섞여 들지 않는다」(`not.toContainText`) · 「표현이 아니라 발견 자체가 틀렸다면 …」
(`getByTestId('not-a-feature')` → `back-to-candidates`).

### ③ 목업 페인포인트 인용 (2행)

`DEVELOPER_VOCABULARY`의 「`JRN-review-feature` 의 페인포인트: "개발자 용어가 남아 있으면
V3 가 무너진다"」. 목업이 SSOT다.

**남긴 것**: 두 파일의 **Isolation 블록 전체**(워커 임대·`workers: 1`·drain 계약 — 정책이
유지 대상으로 이름 붙인 동시성 계약이고, 직전 패스도 sc01-01·sc01-06에서 같은 블록을
남겼다), sc02-01의 「기대값을 상수로 박지 않는다 …」 4행(설계 근거), 「화면이 아니라 서버가
기억한다 — 새로고침이 곧 그 증거다」, sc02-04의 「사람 판정은 자동화 밖이고 기계로 지킬 수
있는 것은 필요조건뿐 / 근거 칸은 검사에서 뺀다」와 「기능 **둘**을 확정한다 — 하나만
확정해서는 관측할 수 없는 성질이다」.

## `e2e/support/acceptance.ts` — 4행 제거

「세 spec 이 서로 다른 성질을 단정하고, 같은 60행 걷기를 세 번 복사하면 드리프트한다」는
설명은 공유 헬퍼라는 **형태 자체**가 말한다. 「setup은 공유하고 verification은 공유하지
않는다」는 **규약 한 줄만 남겼다**. AC1.3·AC2.1 라벨 2건도 지웠다(문장의 나머지 —
lazy seed, 확정이 5단계를 연다 — 는 유지).

**남긴 것**: 「이 파일은 `testDir` 밖이라 테스트로 수집되지도, AC↔spec 매칭 단위로 세어지지도
않는다」(`check-scenario-e2e.py`의 매칭 규칙과 직결된 제약), `signInWithCredentials`의
「다른 AC의 화면을 걷는 것은 setup 이지 verification 이 아니다」(규약),
`runToAcceptance`의 「워커는 호출자가 임대한다 — lease 와 그 `finally` 가 소유한 spec 에
보이도록」(임대 계약).

## 판단이 갈려 남긴 것

정책의 「애매하면 남긴다」에 따라 **복원 경로가 있는데도 남긴 것** 넷:

1. `backend/src/acceptance.rs`의 「와이어 키는 `acceptance_dependencies` 지만 이 단계는
   의존성 작업을 하지 않는다」 — 경위는 doc-tracker에 있지만, **이름과 동작이 어긋난다**는
   사실 자체는 이 자리에서만 경고가 된다.
2. `frontend/src/FeatureAcceptance.tsx`의 ghost 자리표시자 3행 — 위 ③.
3. `backend/tests/acceptance.rs`의 「「단계가 성공했는가」는 틀린 술어」 — doc-tracker
   2026-09-02 행과 `worker_api::acceptance_pending`의 doc에도 있다(3중). 술어를 잘못
   고르면 조용히 문서 하나가 사라지는 함정이라 테스트 쪽 사본을 남겼다.
4. `e2e/support/acceptance.ts`의 `acceptanceOf` JSDoc 1줄(「404면 `null`」) — 코드가
   그대로지만 export 함수의 요약 1줄은 정책의 유지 대상이다.

## 검증 — 이 패스가 통과한 대조

1. **실행 코드 무변경.** 6파일에서 주석·빈 줄을 걷어낸 나머지를 부모 `ebe8657`과 대조해
   **바이트 동일**을 확인했다. diff 전체에서 주석 줄이 아닌 `+`/`-` 행은 **0건**이다.
   그래서 `cargo test`·e2e의 결과는 정의상 바뀔 수 없다.
2. **기계 판독 주석 보존.** `// 검증 시나리오:` 선언 sc02-01 · sc02-04 각 1건(합 2건) 유지,
   `mock-exception:` 2건 유지.
3. **레포 게이트 3종 통과** (판정 시점 실행):
   - `tools/check-scenario-e2e.py` → `✓ 시나리오 ↔ e2e 정합성 통과`
   - `tools/check-mockup-render.py` → 전 규칙 통과, **대조 카피 M3A 198 · M3B 125로 불변**
     (주석만 건드렸으므로 제품 카피는 하나도 움직이지 않았다)
   - `tools/check-journey-mockup.py` → R0~R10 이상 없음
4. **범위 지문 이동.** `957e2129…`(268행) → **`e38dcf20…`(141행)**.
5. **전역 지문 이동.** `lines=3192 files=97` / `f724aab9…` →
   **`lines=3055 files=97` / `b1f90262…`**. 감소 137 = 이 범위 127 + 증분 재판정 10.

> 지문 계산 규약: 범위 지문은 `echo "$HITS" | sha256sum`(후행 개행 **포함**), 전역 지문은
> 모델의 versionScript 그대로 `printf '%s'`(후행 개행 **제외**)다. 두 값은 같은 입력에도
> 다르게 나온다 — 원장 대조는 반드시 전자로 한다.

## 범위 밖으로 남긴 것 (다음 패스가 받는다)

- **`e2e/tests/sc02-02-acceptance-from-tests.spec.ts` (31행)** — 열린 PR #67 접촉. 인수 축의
  일곱 번째 파일이고, #67이 머지되면(`sc02-03` 분할 포함) 바로 다음 후보다.
- **#60 머지로 풀린 자유 풀** — 원장 ②가 자매 모델 뒤로 미뤄 둔 파일들이 이제 비경합이다.
  그중 `backend/src/config.rs`는 **원장 2행 안에서 +23행 늘었고 그 증분은 미판정**이다
  (아래 별도 항목).
- **원장 2행의 증분 재판정 (`config.rs` +23행)** — #60이 `Mode`·`ApiDoubles` doc으로 들인
  주석이다. `docs/e2e-mocking-policy.md`의 허용 목록·EXT-0N 식별자를 인용하는 부분이 제거
  후보로 보이고, 「경계별로 고른다」는 격리 근거는 유지 후보다. 이번 패스에서 판정하지
  않았다 — 원장 2행의 수치·지문은 tip 기준으로 갱신하되 **그 23행이 미판정임을 명시**했다.
- **밀도 높은 자유 풀 후보**(주석 줄 수, tip 기준): `backend/src/bin/worker.rs` 99 ·
  `tools/check-mockup-render.py` 71(#26 draft 접촉) · `frontend/src/api.ts` 72 ·
  `e2e/tests/sc04-08-…` 64 · `backend/src/feature_candidates.rs` 63 ·
  `tools/check-journey-mockup.py` 55 · `backend/src/repo_scan.rs` 49 ·
  `backend/src/discovery_strategy.rs` 46 · `tools/check-scenario-e2e.py` 44.
- **`backend/migrations/*.sql` 7파일 / 104행** — 본문 「적용된 마이그레이션」 절의 전용 PR ·
  수동 repair · 사람 승인 게이트를 거치는 **별도 패스**다. 이번 PR과 섞지 않았다.

## 증분 재판정 ① — `backend/src/acceptance.rs` +1행 (2026-09-21 · `rct_20260921-0001`)

`#93`(`8205b7a`)이 `schema()` 의 `"symbol"` 위에 더한
`// Required-but-nullable; see feature_candidates::schema.` 1행을 **제거**했다. 앞 절반은 두 줄 위
`required` 배열과 바로 아래 `"type": ["string", "null"]` 의 축자 재진술(①), 뒤 절반은 다른 파일
주석으로의 교차 참조뿐이며, 같은 문장이 `dependencies.rs` 에도 있어 두 벌이었다. 명제의 정본은
`llm.rs::assert_strict_schema` 옆이고 같은 파일의 테스트 `schema_is_accepted_by_openai_strict_mode` 가
그 자리를 이름으로 가리킨다. 근거 전체는
[2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 「증분 재판정 ⑤」.
결과: 이 범위의 줄 수·지문이 #93 이전 값 **141 / `e38dcf20…`** 으로 되돌아왔다. 비주석 코드 무접촉
(스트립 잔여 526 == 526).

## 증분 재판정 ② — `frontend/src/FeatureAcceptance.tsx` +5행 (2026-09-21 · `rct_20260921-0012`)

`#112`(`fc6d191`, 슬라이스 6c)가 이 화면에 삭제 탭·사유·보관소·되돌리기를 그리며 쓴 주석 5행을 판정해
**제거 4 · 유지 1**. 새 파일 쪽 판정은 [2026-09-21-feature-delete-axis.md](2026-09-21-feature-delete-axis.md).

| 줄 | 판정 · 복원 경로 |
|---|---|
| `// AC3.3 — 지운 feature 는 문서에서 가려질 뿐 보관소에 남는다. 문서와 함께 읽어` (머리 2행의 앞줄) | 제거 — ② doc-tracker 6c 행 · ① 0011 머리 「문서에는 여전히 있고, 읽는 자리가 가릴 뿐이다」 · ③ PR #112 — AC 꼬리표. 뒷줄 「두 목록이 같은 시점의 서버 상태를 그린다」는 앞줄 끝의 「문서와 함께 읽어」를 붙여 1행으로 재작성해 **유지** — `Promise.all([getAcceptance, listDeletions])` 을 두 effect 가 아니라 한 번의 동시 읽기로 고른 *이유*는 코드·문서·PR 어디에도 없다(**판단이 갈려 남긴 것 5건째**) |
| `/** 삭제·복구 뒤에는 서버를 다시 읽는다 — 화면이 들고 있는 값이 아니라 서버가 들고 있는 값이다. */` (`reload`) | 제거 — ① `setGeneration((g) => g + 1)` 과 effect 의존성 `[id, generation]` 이 그 자체 · 파일 머리의 화면 불변식 「Every sentence on this screen came from the server — which is why a reload shows the same document」(1차 판정이 유지한 정본) |
| `// 남은 기능이 없고 보관소만 있는 상태 — 되돌릴 자리는 있어야 한다.` | 제거 — ① 바로 위 `features.length === 0 && archive.length === 0` 분기와 이 분기의 `<Archive … onRestore>` 렌더 · ③ PR #112 「남은 기능이 0 이고 보관소만 있는 상태도 그린다」 |
| `/** 보관소 — 지운 feature 와 되돌릴 수 있는 기한. 기한이 지난 것은 남되 버튼이 닫힌다. */` (내부 fn `Archive`) | 제거 — ① JSX 카피 `보관소` · `…까지 되돌릴 수 있어요` · `disabled={busy \|\| !deletion.restorable}` · ② doc-tracker 6c 행 「기간이 지나면 되돌리기만 거부된다 — 행은 남는다」 — 내부 fn JSDoc(6b 가 `AddFeature.tsx` 의 `decide` 를 걷은 판정) |

결과: 이 범위의 줄 수·지문은 #112 이전 값(141 / `e38dcf20…`)으로 **돌아가지 않는다** — 유지 1행만큼
**142 / `2ff8f6ff…`** 이다(트리거 `fc6d191` 의 146 / `655305bc…` 에서 −4). 파일은 주석 제거 후 부모와 바이트
동일(stripper md5 `d7dcce35`), `npm run build` rc=0.

## 증분 재판정 ③ — #114(슬라이스 6d)가 `acceptance.rs` 에 연 +4행 · 2026-09-22

reconciler task `rct_20260922-0005`. **전건 제거 4행.**

- `stub_logic` 안의 인라인 2행(「트리에 규정 파일이 들어오면 첫 문장이 달라진다 — 코드가 바뀌어 같은 시나리오를
  다르게 읽게 된 재분석의 결정적 재현이다(`repo_scan::Revision::Third`)」) — `docs/doc-tracker/2026-09.md`
  슬라이스 6d 행의 「스텁 트리에 리비전 3(`FEATUREDOC_STUB_REPO_REVISION=3`, 더하기만)을 두어 첫 문장을
  다르게 읽는 코드 변경을 결정적으로 재현했다」 축자(②).
- doc 2행(「리비전 2 는 문장을 바꾸지 않고(`02#시나리오 8` 의 「시나리오 문장은 그대로다」), 리비전 3 만 첫
  문장을 다르게 쓴다」) — `02#시나리오 8` 을 인용한 **제품 문서 재진술**(②)이고, 같은 doc-tracker 행이
  「리비전 2 의 문장 불변은 단위 테스트가 지킨다 — 02#8 의 단정 보호」로 다시 적는다.

**리비전 어휘의 정본은 `backend/src/repo_scan.rs` 의 `Revision` variant doc 으로 두었다** — 리비전을 *정의하는*
자리다(12차 패스의 「그 명제를 강제하는 코드 옆 한 벌」 잣대). 같은 패스에서 그 variant doc 은 유지했다.

줄 수·지문이 **#114 이전 값 142 / `2ff8f6ff…` 로 바이트 동일 복귀**했다.
맥락 [2026-09-22-conflict-axis.md](2026-09-22-conflict-axis.md).
