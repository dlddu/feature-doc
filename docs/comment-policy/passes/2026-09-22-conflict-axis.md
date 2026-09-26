# 판정 상세 — 충돌 해소 축 (슬라이스 6d 가 들여온 새 파일 4개) · 2026-09-22

reconciler task `rct_20260922-0005`(모델 `tbm_feature-doc-comment-redundancy`). 사람 PR **#114**
(`aacd0b4`, 슬라이스 6d — AC3.5 코드 자동 분석과 사용자 편집의 충돌 처리)가 판정 대상 범위에 주석
**+139행 / −5행 = 순 +134행**을 들여왔다(파일 +5). 그중 마이그레이션 `0013_feature_doc_conflicts.sql`
**24행**은 본문 「적용된 마이그레이션」 절의 사람 게이트(전용 PR · 수동 repair · 사람 승인) 몫이라
**이 패스가 다루지 않는다** — 잔여 `0009`(30) · `0010`(30) · `0011`(25) · `0012`(14) 와 합쳐
**123행 한 배치**로 다뤄야 repair 가 한 번에 끝난다(본문 1항).

나머지 **115행**을 한 패스로 판정했다 — **새 파일 4개 / 86행**(이 파일의 본문)과 **판정 완료 원장 행 안으로
들어온 증분 5파일 / 29행**(원장 1·4·7·10행 — 각 원래 패스 파일에 절을 더했다, 아래 「증분」 절).

## 범위와 결과

| 파일 | 유입 | 제거 | 유지 | 비고 |
|---|---|---|---|---|
| `backend/src/doc_conflict.rs` | 54 | **37** | 17 | 모듈 머리 22 → 1(기제 서술 21행이 doc-tracker 6d 행의 사본) · 선언 재진술 · 전송 경계 · **낡아서 거짓인 주석 1건** |
| `backend/tests/doc_conflict.rs` | 9 | **8** | 1 | 모듈 머리 4 → 1 · helper doc 3 · 테스트 의도 인라인 2 — 전부 fn 이름과 바로 아래 단정이 복원 |
| `frontend/src/ResolveConflict.tsx` | 10 | **4** | 6 | prop·helper JSDoc 3 · 화면 머리가 정본인 인라인 1 · **머리 블록 6행은 유지**(목업 매핑 1행 포함) |
| `e2e/tests/sc03-07-auto-vs-user-edit-conflict.spec.ts` | 13 | **6** | 7 | 절 구분 5 · 리비전 3 설명 1 · **머리 블록 7행은 유지**(9차 패스 `sc01-02` · 17차 `sc04-14` 선례) |
| **합** | **86** | **55** | **31** | 범위 지문 **31 / `f43867463dd13fda9123330dfcb247a35df8482e5cc8bc68da63ef769d1e106f`** |

증분 5파일(원장 행 안): `worker_api.rs` 2 → 제거 1 / `acceptance.rs` 4 → 제거 4 / `repo_scan.rs` +14 −5 → 제거 2 /
`AnalysisDiff.tsx` 6 → 제거 1 / `api.ts` 3 → 제거 3 — **29행 중 제거 11 · 유지 18**. 합쳐 **순 제거 66행**.

**diff 기준**: 9파일 69행 삭제 · 3행 삽입(재작성 — `doc_conflict.rs` 2자리, `worker_api.rs` 1자리).
전역 지문 `lines=2687 files=134` → **`lines=2621 files=134`**.

## 이 축의 복원 경로 — doc-tracker 6d 행이 기제를 통째로 적는다

`docs/doc-tracker/2026-09.md` 의 **슬라이스 6d 행**은 이 슬라이스의 설계를 문단째로 적는다. 정책 본문이
「doc 주석 본문이 … `docs/doc-tracker.md` 를 되풀이하는 부분은 제거 대상」이라 이름 붙인 바로 그 경로(②)다.
대조하면 `doc_conflict.rs` 모듈 머리 21행이 **한 문장도 남기지 않고** 그 행 안에서 복원된다.

| 모듈 머리의 명제 | doc-tracker 6d 행의 대응 문장 |
|---|---|
| 재분석 문서가 저장되는 순간 직전 분석의 편집을 재생한다 | 「재분석 문서가 저장되는 자리(`worker_api::submit_document`)에서 … 새 자동 문서 위에 **재생**한다」 |
| `before` 가 같은 자리(없으면 그 feature 의 다른 자리)에 있으면 이월 | 「편집의 `before` 가 같은 자리(없으면 그 feature 의 다른 자리)에 그대로 있으면 편집을 이 분석으로 이월하고(`feature_doc_edits.carried_from`)」 |
| 다르게 썼으면 어느 쪽도 얹지 않고 충돌 행 | 「자동 결과가 그 자리를 다르게 썼으면 어느 쪽도 얹지 않고 충돌 행(`feature_doc_conflicts`)을 세운다」 |
| 열린 동안 문서엔 자동 결과, 사용자 문장은 앞선 분석에 남는다 | 「열린 동안 문서에는 자동 결과가 서고 사용자 문장은 앞선 분석의 행에 남는다 — 「덮어쓰지 않는다」는 저장 방식의 성질이다」 |
| 결정 셋(`auto`·`mine`·`merge`)과 승인 규약 | 「결정 셋 — `auto` 는 닫기만, `mine` 은 … `merge` 는 … 사람이 확정해야 `merged` 로 닫힌다」 |
| 거부하면 열린 채, 거부 문면이 다음 합치기의 회피 입력 | 「거부하면 열린 채 남고 거부 문면이 다음 합치기의 회피 입력」 |
| 미결정 충돌은 다음 재분석에서 편집과 같은 자격으로 재생 | 「결정하지 않은 충돌은 다음 재분석에서 편집과 같은 자격으로 재생되어 다시 선다」 |
| 자리가 사라진 편집·없는 feature 는 충돌로 세우지 않는다 | 「범위 밖: 자리 자체가 사라진 편집 · 새 문서에 없는 feature 의 편집(둘 다 충돌로 세우지 않고 앞선 이력에만 남는다)」 |

**유지한 모듈 머리 1행**은 `//! AC3.5: 코드 자동 분석과 사용자 편집의 충돌 처리.` — 정책 본문이 유지 대상으로
이름 붙인 모듈 머리 `//!` 요약이고, 자매 축 `doc_edit.rs`(AC3.1) · `feature_add.rs`(AC3.2) ·
`feature_delete.rs`(AC3.3)가 판정 뒤에도 **정확히 같은 한 줄 형태**로 들고 있다. 이 축만 다르게 자르지 않는다.

## `backend/src/doc_conflict.rs` — 제거 37행

### 모듈 머리 `//!` 22행 → 1행 (제거 21)

위 표가 전건이다. 남긴 것은 요약 1행뿐.

### 낡아서 거짓이 된 주석 — `status` 모듈 doc (제거 1)

- 제거: `/// 충돌 행의 상태 어휘. 0012 의 CHECK 와 같은 값이어야 한다.`
- **그 CHECK 는 `0012` 에 없다.** 충돌 상태의 CHECK 는 `0013_feature_doc_conflicts.sql:43`
  (`CHECK (status IN ('open', 'auto', 'mine', 'merged'))`)이고, `0012_llm_language.sql` 은 자기 머리에
  「값의 어휘('ko' · 'en')는 코드가 판정하며 이 파일에 CHECK 로 박지 [않는다]」고 **정반대로** 적는다.
- 어휘 자체는 바로 아래 네 `const` 가 축자로 적는다(①). 정책 본문: 되풀이된 주석이 낡아 틀려 있으면
  **제거 근거를 강화한다.** 어휘의 정본은 **0013 의 CHECK** 이고, 0013 을 판정할 때 그 자리를 정본으로 못박는다.

### 전송 경계·선언 재진술 (제거 9)

- `MERGE_REQUEST` doc 1행 — 「0009 의 `request` 칸에 남기는 문장」은 `0009_feature_doc_edits.sql` 머리가
  「무엇을 고쳐 달라 했고(`request`)」로 이미 적는다(①).
- `ConflictView.mine` · `.auto` 각 1행 — 필드 이름과 타입이 그대로 말한다. **`before` 1행은 유지** —
  「그 편집이 고쳐 쓴 **당시의** 자동 문장」은 이름 `before` 만으로 「무엇의 이전인가」가 복원되지 않는다.
- `ListView.open` 1행 · `Inherited` struct doc 1행(아래 `inherited_from` 의 doc 이 같은 명제의 정본) ·
  `same()` 1행(본문이 공백 정규화 그 자체 + rustdoc 링크 전용 교차 참조) ·
  `inherit()` 본문 3행(단계 재시도 시 재생 흔적을 걷는다 — `DELETE … WHERE carried_from IS NOT NULL` 과
  `status = 'open'` 조건이 축자로 말하고, `backend/tests/doc_conflict.rs` 가 같은 문장으로 단정한다).

### 결정·합치기 doc (제거 5, 2자리 재작성)

- `decide()` 2행 · `merge()` 본문 2행 — doc-tracker 6d 행의 「결정 셋」 문장의 사본. `merge()` 는
  요약 1줄(`/// 두 문장을 합친 제안을 받는다.`)로 줄였다.
- `previous_documented()` 2행 → 1행 — 꼬리의 「정렬은 `analysis::analysis_diff` 와 같다」는 링크 전용
  교차 참조(정책 본문: 링크를 위해서만 문장을 남기지 않는다). **앞의 「편집은 문서 위에만 있으므로 문서 없는
  분석은 이어받을 것이 없다」는 유지** — 이 조인이 *왜* `analysis_documents` 를 거치는지는 SQL 에 없다.
- 합치기 언어 인라인 1행 — doc-tracker 6d 행의 「합칠 두 문장이 이미 그 언어라 결과도 같은 언어여야 한다」 축자.

### 유지 — 17행

`inherited_from()` 2행(**재생 순서 계약**: 열린 충돌 먼저, 승인 편집을 적용 순서대로 — SQL 두 질의의 순서가
*왜* 그 순서인지는 코드에 없다) · `REJECT_REASON` 2행(고정 사유로도 회피가 도는 이유 = 회피의 단위가 사유가
아니라 문면 — 「애매하면 남긴다」) · `ConflictView.before` 1행 · `current_index()` 2행(이월이 앞자리를 늘리므로
저장된 번호가 아니라 지금 문서에서 찾는다 — **함정**) · `stub_merge()` 2행(스텁의 회피 규칙) · 모듈 머리 1행 ·
그 밖 요약 1줄들.

## `backend/tests/doc_conflict.rs` — 제거 8행 (모듈 머리 1행만 유지)

모듈 머리 4행 중 3행(「두 분석은 같은 타깃이고 … 문장 하나로 재현한다」)은 doc-tracker 6d 행의
「스텁 트리에 리비전 3(`FEATUREDOC_STUB_REPO_REVISION=3`, 더하기만)을 두어 첫 문장을 다르게 읽는 코드 변경을
결정적으로 재현했다」의 사본이다. helper doc 3행과 테스트 의도 인라인 2행은 **fn 이름과 바로 아래 단정이 그
문장 자체**라 17차 패스가 `backend/tests/settings.rs` 를 전건 제거한 것과 같은 잣대로 걷었다.

## `frontend/src/ResolveConflict.tsx` — 제거 4행

- 제거: `onBack`·`onSaved` prop JSDoc 2행(이름 + 화면 카피의 재진술) · `dayOf()` 1행(`toISOString().slice(0,10)`
  그 자체) · 「결정되지 않은 합친 제안이 서 있으면 그 자리로 돌아온다」 인라인 1행 — **바로 위 화면 머리가
  이미 그 명제의 정본**이다(「제안은 서버가 만들어 주소를 가지므로(새로고침이 같은 제안을 다시 그린다)」).
- **유지 6행** — 화면 머리 블록. 1행은 목업 매핑(`docs/mockups/JRN-follow-code-change.html#STP-resolve-conflict`
  — 게이트 M1 이 읽는다, `AddFeature.tsx`·`DecideDiff.tsx`·`RequestEdit.tsx` 가 판정 뒤에도 들고 있는 관례)이고,
  나머지는 「결정 전에는 어느 쪽도 저장되지 않는다 — 합치기만 예외」라는 **화면 상태 계약**이다. doc-tracker 는
  서버 쪽 결정 규약만 적을 뿐 *이 화면이 무엇을 들고 있는가*는 적지 않는다.

## `e2e/tests/sc03-07-auto-vs-user-edit-conflict.spec.ts` — 제거 6행

- 제거: 절 구분 5행(`// ── 사전 조건 …`·`// ── 코드가 바뀐 뒤의 재분석 …` 등) — 정의의 중복 유형 **④ 구분선
  주석**이고, 바로 아래 `test.step` 라벨과 단정이 같은 말을 한다. `/** 세 번째 리비전 … */` 1행은 위 doc-tracker
  문장 + `repo_scan::Revision::Third` 의 variant doc(이 패스가 정본으로 유지)의 세 벌째.
- **유지 7행** — 머리 블록. `// 검증 시나리오:` 1행은 `tools/check-scenario-e2e.py` 가 파싱하는 기계 판독
  주석이라 지문·판정 모두에서 제외되며(본문 「유지 대상」), 나머지는 파일의 소유 경계(「3탭 흐름은 sc03-01 의
  소유」)와 **Isolation 선언**이다 — 9차 패스(`sc01-02`) · 17차 패스(`sc04-14`)가 같은 형태를 유지로 고정했다.

## 증분 — 판정 완료 원장 행 안으로 들어온 29행

원장 「읽는 법」이 정한 대로 **새 행을 만들지 않고 원래 행의 결과 칸을 갱신**했고, 상세는 각 원래 패스 파일에
절로 덧붙였다.

- **원장 1행**(`worker_api.rs` +2) → 제거 1 · **유지 1**. 유지한 것은 「저장과 같은 요청 안에서 이어받아야
  워커가 5단계를 `succeeded` 로 보고하기 전에 충돌이 서 있다」 — 호출 **순서**가 만드는 동시성 계약이라
  정책 본문 「동시성 계약」에 해당하고 어느 복원 경로에도 없다. 앞머리의 자리 서술과 `(AC3.5)` 꼬리표만 걷었다.
- **원장 4행**(`acceptance.rs` +4) → **전건 제거**. 줄 수·지문이 #114 이전 값 **142 / `2ff8f6ff…` 로 바이트
  동일 복귀**했다. 리비전 어휘의 정본은 `repo_scan::Revision` 의 variant doc 으로 두었다.
- **원장 7행**(`repo_scan.rs` +14 −5 · `AnalysisDiff.tsx` +6) → 제거 3 · 유지 12. `repo_scan.rs` 는 **제자리
  재작성**이었다(같은 명제 5행이 한국어에서 영어로). 「리비전은 **더하기만** 한다 — 지우거나 이름을 바꾸면 그
  위에 세워진 feature 키가 함께 움직여 「같은 feature 의 표현이 갱신됐다」를 관측할 수 없다」 4행은 **스텁
  충실도의 불변식**이라 유지했고, variant doc 중 `First`(「The tree every first analysis sees.」)와
  「Paths this revision adds on top of the previous one.」는 variant·필드 이름 그대로라 걷었다.
- **원장 10행**(`api.ts` +3) → **전건 제거**. 셋 다 **전송 경계의 재진술**이고 정본은 `doc_conflict.rs` 쪽이다
  (17차 패스 「한 명제의 일곱 벌」과 같은 잣대). 줄 수·지문이 #114 이전 값 **88 / `7e6915aa…` 로 바이트 동일
  복귀**했다.

## 검산

- 전역 지문 재계산: **`lines=2621 files=134`** (판정 전 `lines=2687 files=134`, 순 −66).
- 원장 **21행 전건**을 이 트리에서 재현 — 줄 수·지문 **21/21 일치**(행 규약 `printf '%s\n'`).
- 행 열의 합 **2,394** + 미판정 잔여 **227** = 전역 **2,621** — 맞물린다.
- **비주석 무접촉**: 9파일 각각에서 전행 주석·빈 줄을 걷어낸 나머지의 sha256 이 부모와 **바이트 동일**.
  (원장 10차 패스 경고대로 「비주석 diff 0줄」이 아니라 스트립 후 동일로 확인했다.)
- `python3 tools/check-data-format-change.py --base main --head HEAD --verbose` → **`✅ 변경 없음`**(rc=0).
- `cargo test --manifest-path backend/Cargo.toml --release` **전건 통과**(실패 0) ·
  `check-scenario-e2e.py` · `check-mockup-render.py` · `check-journey-mockup.py` **rc=0**.

---

## 원장에서 옮겨 온 증분 재판정 기록 (2026-09-26 형식 이전)

아래는 `ledger.md`의 결과 칸에 쌓여 있던 증분 재판정·정정 기록을 **문면 그대로** 옮긴
것이다. 형식 이전(템플릿 「원장 형식」)이 원장에 표와 「읽는 법」만 두기로 하면서, 각 행의
경위는 그 행의 패스 파일로 돌아왔다. 옮기면서 한 글자도 고치지 않았고 판정을 새로 하지
않았다 — 행을 가리키는 순번도 당시 표기 그대로다.

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ⑩**(2026-09-22): #114(슬라이스 6d)가 `worker_api.rs` `submit_document` 에 더한 2행을 판정해 **순 제거 1행** — 앞머리 「인수 문서가 서는 순간이 직전 분석의 편집을 이어받을 자리다(AC3.5)」는 doc-tracker 6d 행이 「재분석 문서가 저장되는 자리(`worker_api::submit_document`)에서 … 재생한다」로 축자에 가깝게 적고(②) AC 꼬리표는 ③ 이라 걷었고, **유지 1행**은 「저장과 같은 요청 안에서 이어받아야 워커가 5단계를 `succeeded` 로 보고하기 전에 충돌이 서 있다」 — 호출 **순서**가 만드는 동시성 계약이라 어느 복원 경로에도 없다(정책 본문 「동시성 계약」) · 2행 → 1행 재작성 — [passes/2026-09-22-conflict-axis.md](2026-09-22-conflict-axis.md) 「증분」 · **자매 착지 재실측**(2026-09-22): 이 패스의 머지 직전에 #123(`77158c2` — `llm.rs` 의 `stub_answer` env 레이스 해소)이 먼저 착지해 이 행에 **순 +4행**(더한 6 · 지운 2)을 열었다. 판정이 아니라 **머지 시점 트리에서의 줄 수·지문 재고정**이고(「행 지문을 재현하는 법」 — 개행 포함 해시), 값은 602/`87c1c5cd…` → **606/`6c3ba120…`** 이다. 그 4행은 아래 **증분 재판정 ⑪** 이 닫았다.

### 원장 행 4 — `backend/src/acceptance.rs` · `backend/tests/acceptance.rs` · `e2e/support/acceptance.ts` · `e2e/tests/sc02-01-acceptance-from-logic.spec.ts` · `e2e/tests/sc02-04-user-facing-acceptance-doc.spec.ts` · `frontend/src/FeatureAcceptance.tsx` (인수 축 비경합 6파일)

**증분 재판정 ③**(2026-09-22): #114 가 `acceptance.rs` 에 더한 4행을 **전건 제거** — 스텁 리비전 3 의 재현 의도 2행은 doc-tracker 6d 행(「스텁 트리에 리비전 3 … 을 두어 첫 문장을 다르게 읽는 코드 변경을 결정적으로 재현했다」)의 사본이고, `stub_logic` 의 2행은 `02#시나리오 8` 을 인용한 제품 문서 재진술이다(같은 doc-tracker 행이 「리비전 2 의 문장 불변은 단위 테스트가 지킨다 — 02#8 의 단정 보호」로 다시 적는다) · 리비전 어휘의 **정본은 `repo_scan::Revision` 의 variant doc** 으로 두었다(같은 패스에서 유지) · 줄 수·지문이 #114 이전 값 **142 / `2ff8f6ff…` 로 바이트 동일 복귀** — [passes/2026-09-22-conflict-axis.md](2026-09-22-conflict-axis.md) 「증분」  · **자매 착지 재실측**(2026-09-22 · #121): 20차 패스의 머지 직전에 사람 PR **#121**(`b4a6b30`, AC1.5 — 끝난 단계만 다시 실행)이 착지해 이 행에 **순 +7행**(`backend/tests/acceptance.rs` +7)을 열었다. 판정이 아니라 **머지 시점 트리에서의 줄 수·지문 재고정**이고(「행 지문을 재현하는 법」 — 개행 포함 해시), 값은 142/`2ff8f6ff…` → **149/`a459c9173ce41a9f347923a75257319978c4f02338cac2d5a9cc2d7695393fd9`** 다. 그 7행은 **판정하지 않고 다음 감지에 넘긴다** — **해소됨**(2026-09-25 · `rct_20260925-0002` · 아래 증분 재판정 ④).

### 원장 행 7 — `backend/src/diff.rs` · `backend/tests/diff.rs` · `frontend/src/AnalysisDiff.tsx` · `e2e/tests/sc02-08-reanalysis-diff.spec.ts` · `backend/src/repo_scan.rs` · `frontend/src/AnalysisProgress.tsx` · `backend/src/lib.rs` (재분석 diff 축 비경합 7파일)

**증분 재판정 ③**(2026-09-22): #114 가 `repo_scan.rs` 에 +14 −5(제자리 재작성 — 같은 명제 5행이 한국어에서 영어로 다시 쓰였다) · `AnalysisDiff.tsx` 에 +6 을 열어 **15행 증분**을 판정해 **순 제거 3행** — `repo_scan.rs` 의 variant doc 2행(「The tree every first analysis sees.」 · 「Paths this revision adds on top of the previous one.」)은 variant·필드 이름이 그대로 말하는 ① 선언 재진술이고, `AnalysisDiff.tsx` 의 `onResolve` prop JSDoc 1행은 이름과 화살표 카피의 재진술 · **유지 12행** — 「리비전은 더하기만 한다(지우거나 이름을 바꾸면 그 위에 세워진 feature 키가 함께 움직여 「같은 feature 의 표현이 갱신됐다」를 관측할 수 없다)」 4행은 **스텁 충실도의 불변식**이고 리비전 2·3 의 variant doc 5행은 이 축의 **정본**(원장 4행의 사본 2건을 이 판정이 걷었다) · 「한 곳을 읽어야 넘어간다」 표시의 수명 4행과 「배너가 세는 것은 충돌 행이 아니라 **기능**이다」 1행은 화면 불변식이라 유지 — [passes/2026-09-22-conflict-axis.md](2026-09-22-conflict-axis.md) 「증분」  · **자매 착지 재실측**(2026-09-22 · #121): 20차 패스의 머지 직전에 사람 PR **#121**(`b4a6b30`, AC1.5 — 끝난 단계만 다시 실행)이 착지해 이 행에 **순 +1행**(`frontend/src/AnalysisProgress.tsx` +1)을 열었다. 판정이 아니라 **머지 시점 트리에서의 줄 수·지문 재고정**이고(「행 지문을 재현하는 법」 — 개행 포함 해시), 값은 145/`2e366521…` → **146/`db03aa4ae962896e95d567dcc8eeacc77d43a73088fc3f6d65bc611832aebfc4`** 다. 그 1행은 **판정하지 않고 다음 감지에 넘긴다** — **해소됨**(2026-09-24 · `rct_20260924-0001` · 아래 증분 재판정 ④).

### 원장 행 10 — `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일)

**증분 재판정 ⑧**(2026-09-22): #114 가 `api.ts` 에 더한 3행(`DocConflict` 타입 JSDoc · `decideConflict` · `proposeMerge` 의 함수 JSDoc)을 **전건 제거** — 셋 다 **전송 경계의 재진술**이고 정본은 `doc_conflict.rs` 쪽이다(⑥ 의 「한 명제의 일곱 벌」과 같은 잣대) · AC3.5 꼬리표는 ③ · 줄 수·지문이 #114 이전 값 **88 / `7e6915aa…` 로 바이트 동일 복귀** — [passes/2026-09-22-conflict-axis.md](2026-09-22-conflict-axis.md) 「증분」

