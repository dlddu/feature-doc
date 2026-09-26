# 변경 이력·임의 시점 복원 축 (28차 패스)

- **판정일** 2026-09-25 · **task** `rct_20260925-0005` · **base** `06d26ab`
- **범위** 슬라이스 6e(#126)가 들여온 새 파일 4개와, 같은 PR 이 **이미 판정된 행**에 연 증분
- **원장** 새 행 26 · 증분 재판정 행 4(⑤) · 7(⑤) · 10(⑩) · 17(③) · 21(①)

## 왜 이 범위인가 — 벽 종류로 자른다

이 창(`4457c95..06d26ab`)이 남긴 미판정 잔여는 **176행**이고, 셋으로 갈라진다.

| 풀 | 줄 수 | 벽 |
|---|---|---|
| **무인 풀**(이 패스) | 107 | 없음 — `.sql`·`SELF_PATHS` 무접촉이라 판정기가 `✅ 해당 없음` |
| D1 마이그레이션 `0014_feature_doc_restores.sql` | 33 | 판정은 무료지만 **제거는 사람 repair 창**이 필요하다(전용 PR) |
| D6 `tools/check-data-format-change.py` | 36 | `SELF_PATHS` — 건드리면 필수 status 가 **아예 붙지 않는다**. 무인 머지 경로가 원리적으로 없다 |

한 덩어리로 묶으면 가벼운 107행이 무거운 두 풀의 일정에 묶인다. 그래서 이 패스는 무인 풀만
가져가고 나머지 둘은 원장 행 26 에 **집지 않은 이유와 함께** 등재한다.

## 잔여 실측 — 원장 요약이 아니라 독립 재계수

원장 「읽는 법」을 코드로 옮겨 표를 파싱한 값이다(감지 수치를 그대로 받지 않았다).

```
행 열의 합 2,579 + 행 밖 미판정 증분 20 + 행 없는 파일 6개 156 = 2,755 = 전역 지문
```

⚠️ **범위 칸의 백틱을 기계적으로 긁으면 안 된다.** 행 13 은 `backend/src/crypto.rs` 를
「D2 사람 게이트 풀로 보류」로 **빼는** 단서를 달고 있고, 그 파일은 행 22 의 몫이다. 단서를
무시하면 행 22 가 0행으로 보이고 잔여가 7행 부풀어 보인다 — 줄 수 칸이 그 자리의 검산이다.

## 판정 — 네 경로 중 어디서 복원되는가

### 이 축의 복원 원본은 셋이 겹쳐 있다

| 경로 | 원본 | 무엇을 덮는가 |
|---|---|---|
| ② | `docs/doc-tracker/2026-09.md` 슬라이스 6e 절(151행) | 복원의 저장 방식 · 잘린 편집의 이월 제외 · **`FeatureHistory` 가 목업 매핑 주석을 갖지 않는 이유**까지 거의 축자 |
| ② | `docs/prd/03-doc-management.md`(AC3.4) · `docs/user-journey/JRN-restore-history.md` | 출처 어휘 세 값 · 단계 식별자 · 분기표의 「확인만 하고 유지」 |
| ③ | PR #126 본문 | 절 제목 「복원은 문서를 다시 쓰지 않고 재생 구간을 자른다」 · 「순서는 시각이 아니라 세대로 엮는다」 · 「되살아나면 복원이 한 분석짜리 거짓말이 된다」 |

`doc_history.rs` 모듈 머리 네 문단은 이 셋에 **전부** 걸린다 — 같은 명제의 네 벌째다.

### 「자신이 출처를 지목하는」 주석

`FeatureHistory.tsx` 머리의 「**목업이 없다**」 다섯 줄은 마지막 문장에서 스스로
「그 사실은 숨기지 않고 `docs/doc-tracker/` 의 「수용된 위험」과 「활성 대조 대상」 절에
적혀 있다」고 적는다. 지목된 절을 열어 **실제로 그렇게 적혀 있는 것을 확인**했다
(`2026-09.md` 18행 「수용된 위험 2건(`JRN-restore-history` 전체 미시각화 …)」 · 151행). 자백을
그대로 받지 않고 대조한 것은, 자백이 거짓이면 제거가 아니라 **원본을 고치는 일**이 되기 때문이다.

### 유지 — 네 경로 어디에도 없는 것

- `steps()` 의 **어느 세대에도 들지 못한 편집을 마지막에 붙이는 이유** — 「이력에서 조용히
  빠지느니 순서가 거친 편이 낫다」. 세 번째 루프는 *무엇을* 하는지만 말하고 이 **비용 비교**는
  어디에도 없다. PR 본문에도 없다.
- `FeatureHistory.tsx` 의 「고른 시점의 상태는 **서버가 계산한다**」 — 화면이 짐작하면
  「미리 본 것」과 「실제로 되는 것」이 갈린다는 충실도 경계.
- `AUTO` 의 「행이 아니라 **자리**라서 예약어다」 — 값이 `"auto"` 인 이유.
- `routes()` 의 「항목도 주소를 가진다」 — 6a 의 「제안도 주소를 가진다」, 6b 의 「초안도 …」를
  정본으로 유지한 **같은 규약의 세 번째 적용**이다.

### 🔴 공유 규약이라 건드리지 않은 것 — `Isolation` 블록

`sc03-08` 머리의 `// Isolation:` 2행은 **`sc03-07` 의 것과 바이트 동일**하고, 같은 형태가
e2e spec **14개**에 있다. 과거 패스들이 줄인 것은 **4행짜리 옛 형태**이고(`Isolation 4 → 1`,
13차 규약), 이미 줄어든 2행 형태는 9차(`sc01-02`)·17차(`sc04-14`)·22차(`sc03-07`)가 **유지로
고정**했다. 여기서만 지우면 14파일 공유 규약의 **첫 이탈**이 된다 — 뒤집으려면 14파일을 한
증분 재판정으로 묶어야지, 이 축에서 혼자 할 일이 아니다.

절 구분 배너(`── 변경 ①: … ──`) 8행도 같은 이유로 남겼다. 문면은 doc-tracker 매핑 행과
겹치지만, 6a 가 `sc03-01` 의 `── 탭 N ──` 를 「판단이 갈려 남긴 것」으로 닫은 선례가 있고
이 패스는 그 논거를 새로 뒤집을 근거를 찾지 못했다.

## 증분 재판정

`#126` 은 새 파일만 들인 것이 아니라 **이미 판정된 6파일에도 주석을 연다**. 순증은 20행이지만
`doc_edit.rs` 는 2행을 지우고 9행을 더했으므로 **판정 표면은 gross 22행**이다 — 순증만 보면
그 파일의 표면을 7행으로 과소 진술하게 된다.

| 행 | 유입(gross) | 제거 | 유지 | 결과 |
|---|---|---|---|---|
| 4 `FeatureAcceptance.tsx` | 1 | 1 | 0 | **142/`2ff8f6ff…` 바이트 동일 복귀** |
| 7 `diff.rs` | 2 | 1 | 1 | 149 |
| 10 `api.ts` | 5 | 3 | 2 | 91 |
| 17 `doc_edit.rs` | 9(순증 7) | 5 | 4 | 58 |
| 21 `doc_conflict.rs` · `tests/doc_conflict.rs` | 5 | 4 | 1 | 32 |

행 4 는 #126 이 연 1행을 그대로 걷어 **줄 수와 지문이 #126 이전 값으로 바이트 동일 복귀**했다.

## 검증 — 이 호스트에서 실제로 굴린 것

| 검산 | 결과 |
|---|---|
| **비주석 diff** (`git diff -U0` 에서 주석 줄 제외) | **0줄** — 실행 가능한 줄을 한 줄도 건드리지 않았다 |
| 주석 제거 후 부모와 코드 바이트 동일 | **10/10** |
| `python3 tools/check-journey-mockup.py` | rc=0 |
| `python3 tools/check-mockup-render.py` | rc=0 |
| `python3 tools/check-scenario-e2e.py` | rc=0 |
| `python3 tools/check-data-format-change.py` | `✅ 해당 없음` — `.sql`·`SELF_PATHS` 무접촉 |
| 전역 지문 | `lines=2755 files=141` → **`lines=2683 files=141`** (**순 제거 72행**) |
| 행 열의 합 + 잔여 | **2,614 + 69 = 2,683** — **미판정 증분 0** |

> `cargo test --release` 와 `npm run build` 는 이 호스트에 Rust·빌드 툴체인이 없어 굴리지
> 않았다. 대신 **비주석 diff 0줄**로 실행 가능한 줄의 무접촉을 기계적으로 증명했다 —
> doctest 가 될 수 있는 `///` 코드 블록은 이 축에 **0건**이고(` ``` ` 없음), 제거한 doc 주석 중
> `missing_docs` 대상(`pub` 항목의 유일한 doc)은 없다. CI 가 집행자다.

## 범위 밖 — 다음 감지가 여는 몫

- **D1** `backend/migrations/0014_feature_doc_restores.sql` **33행** — 전용 PR + 사람 repair.
  27차 패스가 `0009`~`0013` 을 비운 **바로 그 창에서 `0014` 가 풀을 다시 채웠다**. 슬라이스마다
  마이그레이션이 1건씩 들어오므로 이 풀은 **구조적으로 재충전된다**.
- **D6** `tools/check-data-format-change.py` **36행** — `SELF_PATHS`(소스 44-47행). 고치는 것이
  곧 접촉이라 우회 불가. 사람 게이트 전용 PR 로만 다룰 수 있다.

---

## 원장에서 옮겨 온 증분 재판정 기록 (2026-09-26 형식 이전)

아래는 `ledger.md`의 결과 칸에 쌓여 있던 증분 재판정·정정 기록을 **문면 그대로** 옮긴
것이다. 형식 이전(템플릿 「원장 형식」)이 원장에 표와 「읽는 법」만 두기로 하면서, 각 행의
경위는 그 행의 패스 파일로 돌아왔다. 옮기면서 한 글자도 고치지 않았고 판정을 새로 하지
않았다 — 행을 가리키는 순번도 당시 표기 그대로다.

### 원장 행 4 — `backend/src/acceptance.rs` · `backend/tests/acceptance.rs` · `e2e/support/acceptance.ts` · `e2e/tests/sc02-01-acceptance-from-logic.spec.ts` · `e2e/tests/sc02-04-user-facing-acceptance-doc.spec.ts` · `frontend/src/FeatureAcceptance.tsx` (인수 축 비경합 6파일)

**증분 재판정 ⑤**(2026-09-25, 28차 패스): #126 이 `FeatureAcceptance.tsx` 에 연 1행(이력 진입점의 사유 — 여정 `JRN-restore-history` 「이력 진입점이 검수 화면 깊숙이 있으면 이 여정 자체가 시작되지 않는다」의 축자(②)이고 PR #126 본문도 같은 문장을 적는다(③))을 **제거** · 줄 수·지문은 #126 이전 값 142/`2ff8f6ff…` 로 **바이트 동일 복귀** — [passes/2026-09-25-history-restore-axis.md](2026-09-25-history-restore-axis.md) 「증분 재판정 ⑤」

### 원장 행 7 — `backend/src/diff.rs` · `backend/tests/diff.rs` · `frontend/src/AnalysisDiff.tsx` · `e2e/tests/sc02-08-reanalysis-diff.spec.ts` · `backend/src/repo_scan.rs` · `frontend/src/AnalysisProgress.tsx` · `backend/src/lib.rs` (재분석 diff 축 비경합 7파일)

**증분 재판정 ⑤**(2026-09-25, 28차 패스): #126 이 `diff.rs::scenario_of` doc 에 연 2행 중 1행(「저장된 문서에서 읽을 때(`scenarios_of`)와 같은 규칙」 — 그 호출 관계는 코드가 말한다(①))을 **제거** · 유지 1행(요약) — [passes/2026-09-25-history-restore-axis.md](2026-09-25-history-restore-axis.md) 「증분 재판정 ⑤」

### 원장 행 10 — `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일)

**증분 재판정 ⑩**(2026-09-25, 28차 패스): #126 이 `api.ts` 에 연 5행 중 3행(`carriedFrom` 의 「두 번 세지 않게 한다」 — `doc_history.rs` 같은 문장의 **두 벌째** · `standing` 과 「복원은 재생 구간을 자른다」 — doc-tracker 2026-09 슬라이스 6e 절의 축자(②))을 **제거** · 유지 2행(`HistoryEntry`·`PreviewView` 의 export 요약 1줄씩) — [passes/2026-09-25-history-restore-axis.md](2026-09-25-history-restore-axis.md) 「증분 재판정 ⑩」

### 원장 행 17 — `backend/src/doc_edit.rs` · `backend/tests/doc_edit.rs` · `e2e/tests/sc03-01-llm-assisted-edit.spec.ts` · `e2e/tests/sc03-02-rejected-suggestion-avoided.spec.ts` · `frontend/src/DecideDiff.tsx` · `frontend/src/RequestEdit.tsx` (문서 편집 축 — 슬라이스 6a #92 가 들여온 새 파일 6개)

**증분 재판정 ③**(2026-09-25, 28차 패스): #126 이 `doc_edit.rs` 에 연 9행(순증 7) 중 5행(`SOURCE_AUTO` doc 의 AC3.4 어휘 재진술 · `propose`·`overlay` 의 0014 세대 서술 2 — PR #126 본문 「순서는 시각이 아니라 세대로 엮는다」의 축자(③) · `splice`·`base_document` doc 의 둘째 줄 2 — doc-tracker 슬라이스 6e 절(②))을 **제거** · 유지 4행 — [passes/2026-09-25-history-restore-axis.md](2026-09-25-history-restore-axis.md) 「증분 재판정 ③」

### 원장 행 23 — `backend/src/doc_conflict.rs` · `backend/tests/doc_conflict.rs` · `frontend/src/ResolveConflict.tsx` · `e2e/tests/sc03-07-auto-vs-user-edit-conflict.spec.ts` (충돌 해소 축 — 슬라이스 6d #114 가 들여온 새 파일 4개 — `backend/migrations/0013_feature_doc_conflicts.sql` 는 **D1 사람 게이트 풀**로 보류 (— **해소됨(2026-09-25, 27차 패스, `rct_20260925-0003`)**: 새 행 25 로 판정 — 유지))

**증분 재판정 ①**(2026-09-25, 28차 패스): #126 이 연 5행 중 4행(`inherit` 의 「되살아나면 복원이 한 분석짜리 거짓말이 된다」 2 — PR #126 본문의 축자(③) · `after_restore` 의 NULL 서술 1 — 바인딩이 말한다(①) · `tests/doc_conflict.rs` 의 의도 주석 1 — fn 이름과 단정(①))을 **제거** · 유지 1행 — [passes/2026-09-25-history-restore-axis.md](2026-09-25-history-restore-axis.md) 「증분 재판정 ①」

