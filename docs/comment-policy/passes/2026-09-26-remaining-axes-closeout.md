# 43차 패스 — L 표면 잔여 9행의 남은 축(③④) 마감

- task: `rct_20260926-0010` (`tbm_feature-doc-comment-redundancy`)
- 기준 커밋: `3e0c8d9` (42차 패스 #188 착지 직후)
- 범위: L 표면 미판정 14행 중 **9행 267줄** — 남은 축만 채운다(①② 는 앞선 패스가 닫았다)

## 슬라이스 경계 — 왜 267줄인가 (예산 400줄)

미판정 잔여는 14행 550줄이었다. 두 덩어리를 뺐다.

| 뺀 덩어리 | 줄 | 사유 |
|---|---|---|
| `0001`~`0008` 풀 · `0014` · `0015` | 123 + 33 + 11 = **167** | 정책 본문 「적용된 마이그레이션의 주석」 절 — 전용 PR 하나 · 사람 수동 repair · 사람 승인. 예산 규칙이 이 분리를 무르지 않는다 |
| `frontend/src/api.ts` 축 · `NoAccess.tsx` | 105 + 11 = **116** | **열린 PR #187** 이 `frontend/src/api.ts`·`index.css`·`NoAccess.tsx` 를 실제로 만진다(정의가 인정하는 유일한 축소 사유). #187 이 `NoAccess.tsx` 를 11 → 27줄로 올리므로 지금 판정하면 I3 지문이 머지와 동시에 깨진다 |

남은 9행이 곧 267줄이고, 이것이 지금 집을 수 있는 **전부**다. 예산 미달의 사유는 「남은 대상이
없음 + 파일이 겹치는 열린 PR」 둘 다이며, 정의 「판정 슬라이스」 절이 요구하는 기록이다.

**파일 겹침 실측**: 9행의 파일 31개와 #187 의 변경 파일 12개의 교집합은 **0** 이다(`docs/comment-policy/ledger.md`
는 두 PR 이 함께 만지지만 행이 서로 달라 표의 다른 줄이다).

## ④ 축의 실체 — 제목 한 줄

이 9행의 파일을 건드린 커밋 **27개**를 전수로 떠서 본문(`%b`)을 쟀다. **27/27 이 `Co-authored-by:`
트레일러뿐이고, 트레일러를 지운 산문은 0줄**이다. 즉 이 레포에서 ④ 의 실체는 squash **제목 한 줄**이다.

그 제목 corpus 와 9행의 유지 줄 전부를 **최장 연속 일치**로 대조했다. 10자 이상 히트는 6건이고
전부 **모듈 머리 `//!` 요약 1줄**에 떨어졌다.

| 줄 | 일치 | 판정 |
|---|---|---|
| `backend/src/doc_conflict.rs:1` | 24자 | 유지 |
| `backend/tests/doc_history.rs:1` | 21자 | 유지 |
| `backend/src/feature_delete.rs:1` | 19자 | 유지 |
| `backend/src/doc_history.rs:1` | 18자 | 유지 |
| `backend/tests/feature_delete.rs:1` | 18자 | 유지 |
| `backend/tests/doc_conflict.rs:1` | 17자 | 유지 |

### 왜 전건 유지인가 — 반대 증거를 부모 트리에서 셌다

정책 「유지 대상」의 doc 주석 조항은 모듈 머리 `//!` 를 유지한다. 문제는 그 안의 **AC 꼬리표**를
걷는 재작성이 가능한가였다. 부모 트리에서 세었다.

- **①②③④ 로 닫힌 행에 살아 있는 「AC 꼬리표 붙은 모듈 머리」가 8파일 20줄**이다 —
  `backend/src/analysis.rs` 9 · `worker_api.rs` 4 · `llm.rs` 3 · `usage.rs` 1 · `config.rs` 1 ·
  `llmkey.rs` 1 · `doc_edit.rs` 1 · `backend/tests/doc_edit.rs` 1(+ `tests/usage.rs` 1).
  이 줄들은 네 축을 **다 물은 뒤 유지로 닫힌** 것이다.
- 반대쪽 선례는 **1건**뿐이다 — `backend/src/crypto.rs` 의 `(AC4.3)`(19차 패스 #128). 그것도
  꼬리표 단독 판정이 아니라 **모듈 머리 동작 서술 3행 제거에 딸린 재작성**이었다.

20 대 1 이다. 여기서 걷으면 그 관례의 **첫 이탈**이 되고, 정책의 비용 비대칭 조항(「애매하면
남긴다」)이 정확히 이 자리를 가리킨다. **전건 유지**로 닫는다.

`backend/src/feature_add.rs:1` 은 히트조차 아니다 — 제목은 `AC3.2 feature 문서의 추가` 이고
주석은 `AC3.2: 자동 추출이 놓친 feature 를 사람이 한 문장으로 더하는 흐름.` 이라 최장 연속
일치가 10자 미만이다(꼬리표 문자열만 겹친다). 눈으로 보면 같아 보이지만 기계로 재면 갈린다.

## ③ 축 — 세 행

판정 패스 PR(#76·#100·#117·#147)은 모집단에서 뺐다. 「유지」를 설명하려 주석을 인용한 판정 패스를
③ 히트로 세면 자기 무효화 고리가 된다.

### `backend/src/dependencies.rs` 축 (행 `backend/src/dependencies.rs` · 112 → 109줄)

저작 PR **#71**·#73·#92·#93·#148·#166 의 본문과 리뷰 코멘트 **20,342자**를 corpus 로 유지 112행을
전수 대조했다. 커버리지 0.9 이상은 **1줄**뿐이다.

```
주석 : **근거를 지어내지 않는다.** 모델이 든 근거가 이 분석이 본 경로가 아니면 항목을
       버리는 대신 **근거만 떨어뜨린다** — 버리면 의존성이 사라지고, 채우면 거짓이 된다.

PR#71: **3. 근거를 지어내지 않는다.** 모델이 든 근거가 이 분석이 본 경로가 아니면 항목을
       버리는 대신 **근거만 떨어뜨려** 화면의 「근거 없음」이 된다 — 여정 `JRN-review-feature`
       의 예외 표가 그렇게 요구한다(「근거 없음으로 명시. 임의로 채우지 않음」).
       버리면 의존성 자체가 사라지고, 채우면 거짓이 된다.
```

연속 45자 일치이고, **PR 쪽이 초집합**이다(여정 예외 표의 출처까지 갖는다).

**가드가 아니라 서사다.** 유지 판별식은 「어겼을 때 조용히 깨지는가」인데, 이 명제는 어기면 즉시
붉다 — 세 자리가 단정한다.

- `backend/src/dependencies.rs` `fabricated_evidence_becomes_no_evidence` — `validated.len() == 2`
  로 **항목은 남고**, `assert_eq!(validated[0].evidence, None, "지어낸 경로는 근거로 남지 않는다")`
  로 **근거만 떨어진다**. 명제의 두 반쪽을 그대로 단정한다.
- `backend/tests/dependencies.rs:432` — `"근거는 이 분석이 본 경로여야 한다: {evidence}"`.
- `e2e/tests/sc02-05-dependency-extraction.spec.ts:42` — `expect(item.evidence).toContain('payments-api/')`.

→ **순 제거 3행**(¶2 2행 + 딸린 빈 `//!` 1행).

모듈 머리 ¶1(「**파이프라인 단계를 더하지 않는다.**」 3행)은 ③ corpus 에 **부재**라 유지한다.
나머지 최대 히트는 식별자·경로 문면(`dependencies::CATEGORIES` 28자 · `backend/src/dependencies.rs`
33자 · `JRN-review-feature.html` 23자)이라 복원 가능한 **내용**이 아니다.

### `backend/src/settings.rs` 축 (17줄)

저작 PR **#108** + 후속 #119 의 본문·코멘트 **5,891자** 대 유지 17행 — **14자 이상 히트 0**.
④ 도 히트 0. **순 제거 0행**.

### `e2e/support/github-app.ts` 축 (3줄)

저작 PR **#141**(`github-app.ts`) · **#121**(`sc01-08`) + #148, **9,600자** 대 유지 3행.
최대 히트는 export JSDoc 요약의 `the App install` **16자**로 식별자 문면이고, 그 줄은 정책
「유지 대상」의 export 함수 JSDoc 요약 1줄이다. **순 제거 0행**.

## 무영향 증명

- **비주석 diff 0줄** — `backend/src/dependencies.rs` 의 변경은 `//!` 3줄 삭제뿐이다.
- **주석 걷은 코드 바이트 동일 1/1** — 부모와 대조.
- `missing_docs` lint 부재 · 삭제한 줄에 doctest(` ``` `) 0건 → `cargo test --release` 무영향.
- 게이트 4종 rc=0(`check-comment-ledger.py` · `check-journey-mockup.py` · `check-mockup-render.py`
  · `check-scenario-e2e.py`) · `check-journey-prototype.js` rc=0.
- **D1·D6 무접촉** — `backend/migrations/**` 와 `tools/check-data-format-change.py`·
  `.github/workflows/data-format-review.yml` 를 건드리지 않는다 → `review/manual-approval` 불요.

## 원장 집계 (게이트 출력)

|  | 전 | 후 |
|---|---|---|
| L 판정 대상 | 3,007줄 | **3,004줄** |
| L 판정 완료 | 2,457줄 / 26행 (81%) | **2,721줄 / 35행 (90%)** |
| L 축별 미판정 행 | ① 3 · ② 4 · ③ 6 · ④ 12 | **① 3 · ② 4 · ③ 3 · ④ 4** |
| D · E | 100% | 100%(불변) |

## 범위 밖 (후속)

- **마이그레이션 3행 167줄** — `0001`~`0008`(②) · `0014`(④) · `0015`(전축). 전용 PR 하나에 묶어
  사람 repair 창으로. `0015` 는 37차 패스가 제거 근거를 적어 두었으니 집행만 남았다.
- **frontend 2행 116줄** — #187 착지 뒤 재감지의 몫. #187 이 새 행 2개(`0016_access_requests.sql`
  25줄 · `access_request.rs` 축 64줄)를 축 `—` 로 들이고 `NoAccess.tsx` 를 11 → 27줄로 올리므로,
  다음 슬라이스의 잔여는 **약 105줄 늘어난다**.
