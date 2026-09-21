# 판정 상세 — feature 삭제·보존 축 (슬라이스 6c 가 들여온 새 파일 4개) · 2026-09-21

reconciler task `rct_20260921-0012`(모델 `tbm_feature-doc-comment-redundancy`). 자매 모델
`tbm_feature-doc-docs-impl` 의 슬라이스 6c(**PR #112**, `fc6d191`, AC3.3 feature 문서의 삭제와 보존)가 판정
대상 범위에 주석 **순 +118행 / 새 파일 5**를 들여왔다(삭제 0 — 지워진 주석은 없다). 그중 마이그레이션
`0011_feature_deletions.sql` 25행은 본문 「적용된 마이그레이션」 절의 사람 게이트 몫이라 **이 패스가 다루지
않는다**. 나머지 93행을 한 패스로 판정했다 — **새 파일 4개 / 80행**(이 파일의 본문)과 **판정 완료 원장 행
안으로 들어온 증분 4파일 / 13행**(원장 1·4·8·10행 — 각 원래 패스 파일에 절을 더했다, 아래 「증분」 절).
문서 편집 축([2026-09-21-doc-edit-axis.md](2026-09-21-doc-edit-axis.md), #92 → PR #106)·빠진 feature 직접 추가
축([2026-09-21-feature-add-axis.md](2026-09-21-feature-add-axis.md), #107 → PR #111)과 **같은 판정형**이고, #112 가
6a·6b 의 축(자동 문서를 고치지 않고 읽는 자리에서 겹친다)을 그대로 재사용했으므로 그 두 패스의 판정을 같은
자리마다 같은 방향으로 적용했다.

## 범위와 결과

| 파일 | 유입 | 제거 | 유지 | 비고 |
|---|---|---|---|---|
| `backend/src/feature_delete.rs` | 30 | **20** | 10 | 모듈 머리 17행 → 요약 1행 · 필드·private fn doc · `overlay` doc 을 순서 계약의 정본으로 재작성(±0) · `current_document` 2 → 1 |
| `backend/tests/feature_delete.rs` | 14 | **13** | 1 | 모듈 머리 7행 → 요약 1행 · 픽스처 fn doc 2 · 단정 옆 주석 5 |
| `e2e/tests/sc03-05-feature-delete-and-restore.spec.ts` | 17 | **15** | 2 | 머리 문단 4행(doc-tracker 매핑 행 축자) · Isolation 4행 → 1행 · 단정 옆 주석 7 |
| `e2e/tests/sc03-06-deleted-feature-rediscovery.spec.ts` | 19 | **17** | 2 | 머리 문단 4 + 셋업 사유 2 · Isolation 4 → 1 · 내부 fn JSDoc 1 · 단정 옆 주석 5 |
| **합** | **80** | **65** | **15** | 범위 지문 **15 / `3dfb5050…`** |

증분 4파일(원장 행 안): `backend/src/analysis.rs` 5 → 제거 5 / `frontend/src/FeatureAcceptance.tsx` 5 → 제거 4 ·
유지 1 / `frontend/src/FeatureCandidates.tsx` 1 → 제거 1 / `frontend/src/api.ts` 2 → 제거 2 — **13행 중 제거 12 ·
유지 1**. 합쳐 **순 제거 77행**.

**diff 기준**: 8파일 83행 삭제 · 6행 삽입(재작성 — `feature_delete.rs` 의 `overlay`·`current_document` doc,
`FeatureAcceptance.tsx` 머리 1행, 두 spec 의 Isolation 1행). 지문 감소(77)와 diff 삭제 줄 수가 갈리는 것은
재작성 때문이다.

## 이 축의 복원 경로 — 6a·6b 와 같은 세 겹에 마이그레이션 한 겹

#112 는 자기 설계 판단을 **세 자리에 이미 적어 두었다**: ② `docs/doc-tracker/2026-09.md` 의 슬라이스 6c
변경 이력 행(「지우면 `feature_deletions` 에 열린 행이 서고 인수 문서를 내보내는 자리가 그 키를 가린다(확정된
추가 겹침 **뒤**, 편집 겹침 **앞** — 더해진 feature 도 지울 수 있고, 가려진 feature 의 편집은 얹을 자리가
없다)」 · 「보관 기한(30일)은 지운 시점의 행에 적힌다 — 「일정 기간」은 그때의 약속이지 지금의 설정이 아니다」 ·
「되돌리면 행이 닫혀 문서에 다시 서고 행은 남는다(이력). 기한 뒤의 되돌리기는 409 로 거부되며 영구 제거는
요구가 아니라 하지 않는다」 · 「재발견(03#6)은 후보 거부의 이월과 같은 통로다 — … `previouslyDeleted`(사유 ·
분석 id)가 실리되 결정은 `undecided` 그대로라 사용자 확인 없이 되살아날 길이 없다」) · ② 같은 파일의 e2e 매핑
행 두 개(`sc03-05`·`sc03-06` — 「검증 내용」과 「관측 대상이 아닌 것」 칸) · ③ PR #112 본문(같은 문장들) · ①
마이그레이션 `0011_feature_deletions.sql` 의 머리 주석(보관소 원리 · 한 행이 삭제 한 번의 일생이고 되돌린 행도
남는다 · `analysis_documents` 를 고치지 않는 이유(`content_hash`) · `restore_until` 을 행에 적는 이유 · `reason`
선택 · 부분 유일 인덱스). 그래서 `feature_delete.rs` 모듈 머리 16행은 네 벌째였고, 두 spec 의 머리 문단은 매핑
행의 축자였다. 0011 주석은 사람 게이트 몫이라 이 패스가 손대지 않으며, **저장 계약의 정본은 그 파일이다** —
0009(문서 편집 축)·0010(빠진 feature 직접 추가 축)과 같은 판정이다.

## `backend/src/feature_delete.rs` — 제거 20행

### 모듈 머리 `//!` 17행 → 1행 (제거 16)

- 유지: `//! AC3.3: feature 문서의 삭제와 보존.` — 모듈 요약 1행(`doc_edit.rs`·`feature_add.rs` 와 같은 모양).
- 제거 「사람이 feature 를 지우면 곧바로 없어지지 않고 **보관소**로 옮겨진다 — 삭제 행이 열리고, 인수 문서를
  읽는 자리가 그 feature 를 가린다([`overlay`]). 보관 기간([`RETENTION_DAYS`]) 안에는 되돌릴 수 있고, 되돌리면
  행이 닫혀 문서에 다시 보인다. 기간이 지나면 되돌리기만 거부된다 — 행은 남는다. 「즉시 영구 제거되지
  않는다」의 반대편(영구 제거)은 이 흐름의 요구가 아니라 별도 정리 작업의 몫이고, 여기서는 아무것도 지우지
  않는다」 6행 + 빈 2 — ① 0011 머리 1~8행(보관소 · 되돌린 행도 지우지 않는다) · ② doc-tracker 6c 행(「되돌리면
  행이 닫혀 … 기한 뒤의 되돌리기는 409 … 영구 제거는 요구가 아니라 하지 않는다」) · ② `sc03-05` 매핑 행
  「관측 대상이 아닌 것」 칸(「영구 제거는 요구가 아니라 하지 않는다」) · ③ PR #112 · ① `restore()` doc(유지)과
  `restorable()` 본문.
- 제거 「**자동 문서를 고치지 않는다.** 편집(AC3.1)·추가(AC3.2)와 같은 이유다 — 자동 산출물의 `content_hash`
  는 재현성·재분석 diff 의 기준값이라 사람이 지웠다는 이유로 달라지면 안 된다. 지운 것은 문서를 내보내는
  시점에 걸러 낸다」 3행 + 빈 1 — ① 0011 「왜 인수 문서(`analysis_documents`)에서 feature 를 빼지 않는가 …
  (0009 · 0010 의 주석과 같은 이유)」 문단(정본) · ② doc-tracker 6c 행 「6a·6b 의 축을 그대로 썼다: 자동 문서를
  고치지 않고 읽는 자리에서 겹친다」 · ③ PR 본문 — `doc_edit.rs`·`feature_add.rs` 머리의 같은 문단과 같은 판정.
- 제거 「**재발견은 표시일 뿐 결정이 아니다.** 같은 저장소의 다음 자동 분석이 같은 자리를 다시 후보로 잡으면
  후보 목록이 「이전 분석에서 삭제한 항목」을 사유와 함께 싣는다([`previous_deletion`]) — 후보 거부의
  이월(AC1.4)과 같은 통로다. 후보는 사람이 승인하기 전까지 `undecided` 이므로 사용자 확인 없이 다시 활성화될
  길이 없다」 4행 — ② doc-tracker 6c 행 축자 · ② `sc03-06` 매핑 행 · ③ PR 본문 · ① `previous_deletion` pub
  doc(유지) · ① 테스트 fn 이름 `a_deleted_feature_rediscovered_by_the_next_analysis_is_marked_not_reactivated`
  와 단정 메시지 「사용자 확인 없이 다시 활성화됐다」.

### 선언·필드·private fn doc (제거 4)

| 줄 | 복원 경로 |
|---|---|
| `DeletionView.restorable` 「지금 되돌릴 수 있는가 — 열려 있고 기한 안이다」 | ① `restorable()` 본문 `restored_at.is_none() && now <= restore_until` — 선언 재진술(`AdditionView.evidence_found` 와 같은 판정) |
| `ListView.deletions` 「보관소 — 열린(되돌리지 않은) 삭제만, 최근 것부터」 | ① `open_rows` 의 SQL `WHERE … restored_at IS NULL ORDER BY deleted_at DESC, rowid DESC` · 화면 `Archive` 의 `보관소` 카피 |
| `delete()` 「지금 사람이 보는 문서에 있는 feature 만 지울 수 있다 — 이미 지운 것, 없는 것은 404」 | ① 본문 — `current_document`(겹침 적용) 에서 `feature_name(…).ok_or(NotFound)` · ③ PR #112 「지금 보는 문서에 있는 feature 만 지울 수 있다(없으면 404)」 · ① 테스트의 `NOT_FOUND` 단정 |
| `current_document()` 「사람이 지금 보는 인수 문서 — 자동 문서 위에 확정된 추가와 열린 삭제를 겹친 것」 (2행 → 1행, 재작성) | ① 본문의 두 `overlay` 호출. 뒤따르는 「편집 겹침은 이름·키를 바꾸지 않으므로 여기서는 얹지 않는다」는 **유지** — 세 번째 겹침을 *빼는* 이유가 이 자리에만 있다 |

### `overlay` doc — 순서 계약의 정본으로 재작성 (±0)

「열린 삭제가 있는 feature 를 문서에서 가린다. 자동 문서·확정된 추가 어느 쪽이든 같은 규칙이다 — 키가
정체성이므로」 2행 → 「열린 삭제가 있는 feature 를 문서에서 가린다. 확정된 추가 겹침 **뒤**, 편집 겹침 **앞**에
불러야 한다 — 더해진 feature 도 지울 수 있고, 가려진 feature 의 편집은 얹을 자리가 없다」 2행. 걷은 뒷문장은 ①
`hide()` 의 `key` 비교 그 자체이고, 대신 넣은 순서 계약은 #112 가 호출부(`analysis.rs::document`)에 인라인으로
적어 둔 것을 **정본 자리로 옮긴 것**이다 — `feature_add::overlay` doc 이 「편집 겹치기보다 먼저」를 들고 있고 두
호출부의 사본을 그쪽을 가리켜 걷은 6a·6b 의 판정과 같은 모양(호출부 사본은 「증분」 절).

### 유지 10행 — 사유

`//!` 요약 1행 · `RETENTION_DAYS` 「되돌릴 수 있는 기간. 「일정 기간」의 값이고, 지운 시점의 행에 기한으로 적힌다」
1행(pub const 요약 — 0011 `restore_until` 문단이 정본이지만 pub 항목 요약 1줄 유지 조항) · `PreviousDeletion`
「재발견 표시에 실리는 앞선 삭제」 1행(pub struct 요약) · `restore()` 「보관 기간 안의 열린 삭제만 되돌린다. 이미
되돌린 것은 409, 기한이 지난 것도 409 — 둘 다 「지금은 할 수 없는 일」이지 「없는 것」이 아니다」 2행(**409 를
404 가 아니라 고른 이유** — 거부 조건의 why 는 doc-tracker·PR 이 「409 로 거부」까지만 적는다; `owned_analysis` 의
「404 not 403」 선례) · `overlay` 2행(재작성, 정본) · `previous_deletion` 「같은 대상(사용자·저장소·브랜치)의 앞선
분석에서 이 키를 지웠고 아직 되돌리지 않은 가장 최근 삭제. 후보 거부 이월과 같은 순서 규칙 — unix 초가 같으면
`rowid` 로 가른다」 2행(pub fn 요약 + `(created_at, rowid)` 타이브레이커 함정 — 1차 패스가 `analysis.rs`
`previous_rejection`·`carried_over` 에서 유지한 그대로) · `current_document` 1행(재작성).

**판단이 갈려 남긴 것**: `current_document` 의 「편집 겹침은 이름·키를 바꾸지 않으므로 여기서는 얹지 않는다」 —
`doc_edit::overlay` 가 시나리오 문장만 바꾼다는 것은 그 코드에서 따라오지만, 그래서 *이 자리에서 뺀다*는
결정은 어디에도 없다(1건).

## `backend/tests/feature_delete.rs` — 제거 13행

- 모듈 머리 7행 → 요약 1행 「feature 문서의 삭제와 보존(AC3.3) — 라우터를 그대로 돌려 본다」. 제거 「인수
  문서는 워커의 `/internal` 경로로 넣는다(형제 테스트와 같은 방침)」 — `backend/tests/dependencies.rs:3-4` 가
  정본(문서 편집·직접 추가 축과 같은 판정). 제거 「브라우저에서 관측되는 흐름은 `e2e/tests/sc03-05`·`sc03-06` 이
  지킨다. 이 파일이 지키는 것은 그 아래의 규칙이다 — 지우면 문서에서 가려질 뿐 행은 남는다 · … · 남의 분석에는
  닿지 않음」 4행 — ① 세 테스트의 fn 이름이 그 목록이다 · ② doc-tracker e2e 매핑 표. 빈 `//!` 1행이 딸려 나갔다.
- `analysis()` 「큐에 든 분석 하나 — 5단계가 아직 아무것도 쓰지 않은 상태」 · `analysis_with_document()` 「인수
  문서가 서 있는 분석 하나」 — 6b 의 같은 두 줄과 같은 판정(① fn 이름 · 본문).
- 단정 옆 5행: 「문서에서는 가려지지만 자동 문서 자체는 그대로다 — 문서는 여전히 있다(404 가 아니다)」 — ①
  바로 아래 두 단정과 메시지(「지운 feature 가 문서에서 가려지지 않았다」 · 「자동 문서를 고쳤다 — 재현성의
  기준값이 움직인다」). 「이미 지운 것은 지금 보는 문서에 없으므로 다시 지울 수 없다」 — ① `NOT_FOUND` 단정 · ③
  PR. 「되돌린 뒤 다시 지우면 새 행이 선다 — 이력은 쌓인다」 — ① 0011 「되돌린 행도 지우지 않는다: … 같은 feature
  를 다시 지우면 새 행이 선다」(정본) · `assert_ne!(again["id"], …)` · 메시지 「되돌린 행을 지웠다」. 「같은
  저장소의 다음 자동 분석이 같은 자리를 후보로 잡는다」 — ① 바로 아래 셋업(`analysis()` → claim → 같은 `FEATURE`
  로 후보 제출). 「되돌리면 표시도 사라진다 — 지금 지워져 있는 것만 「이전에 삭제」다」 — ① 바로 아래
  `previouslyDeleted.is_null()` 단정 · `previous_deletion` 의 `restored_at IS NULL`.
- **유지**: 요약 1행. 픽스처 상수 `FEATURE`·`WHY` 에는 doc 이 없어 6b 의 「픽스처 상수 doc」 판단 갈림은 이
  파일에 생기지 않았다.

## 두 spec — 제거 32행

- **Isolation 4행 → 「Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.」 1행(둘 다, −3씩).**
  13차 패스가 정한 규약이고 6a 가 `sc03-01`·`sc03-02` 에, 6b 가 `sc03-03` 에 적용한 그대로 — 새로 들어온 파일에는
  최신 규약. **이미 판정된 spec 12개의 Isolation 블록은 이 패스 범위 밖**이며 후속 후보로 그대로 남는다.
- **머리 문단은 두 spec 모두 걷었다** — 6b 와 같은 이유로, 여기 문단은 설계의 이유가 아니라 **무엇을
  관측하는가**다. `sc03-05` 「「보관소로 이동하며, 일정 기간 내 복구 가능. 즉시 영구 삭제되지 않는다」를
  **관측**한다 — 지운 feature 는 인수 문서에서 사라지되 보관소 목록에 사유·기한과 함께 남고, 되돌리면 문서에
  다시 선다. 기한은 화면이 아니라 서버가 지운 시점에 적은 값이다(`restoreUntil`). 「즉시 영구 삭제되지
  않는다」의 관측은 되돌리기가 성립한다는 것 자체다」 4행 — ② doc-tracker e2e 매핑 행(`03#시나리오 5`)의 「검증
  내용」 칸(「feature 가 문서에서 사라지고 **보관소**에 이름·사유·기한과 함께 선다 → 문서는 여전히 있되(404 가
  아니다) … `restoreUntil − deletedAt` = 보관 일수(「일정 기간」은 지운 시점의 사실) → … 되돌리기 → 문서에 같은
  시나리오 수로 다시 선다」)이 문장 단위로 같다. `sc03-06` 「「동일 feature 가 자동으로 발견되더라도 '이전에
  거부/삭제된 항목입니다' 로 표시되며, 사용자 확인 없이는 다시 활성화되지 않는다」를 **관측**한다. 같은 저장소의
  두 번째 자동 분석이 같은 자리(키 = 발견 위치)를 후보로 잡으면, 그 후보가 앞선 분석의 삭제와 사유를 달고 오되
  결정은 `undecided` 이고, 인수 문서는 사람이 승인하기 전까지 서지 않는다」 4행 + 「사전 조건(시나리오 5 의
  삭제)은 API 로 만든다 — 다른 시나리오의 화면을 걷는 것은 셋업이지 검증이 아니다」 2행 — ② 같은 표의
  `03#시나리오 6` 행(「시나리오 5 의 삭제(API 로 셋업) → 같은 저장소 자동 재분석을 4단계까지 → 같은 자리(키 =
  발견 위치)의 후보가 `previouslyDeleted` … 를 달고 오되 `decision=undecided` … 승인하지 않았으므로 재분석의 인수
  문서는 서지 않는다(404)」) · 6b 가 `sc03-03` 의 「셋업이지 검증이 아니다」 를 걷은 판정. 빈 `//` 4행이 딸려 나갔다.
- `sc03-06` 내부 fn `reanalyseToCandidates` 의 JSDoc 「같은 저장소의 자동 재분석 — 4단계(후보 추출)까지만 걷고
  아무것도 승인하지 않는다」 — ① fn 이름 · 본문(후보 > 0 까지 poll, 승인 호출 없음) · ② 매핑 행 「4단계까지」.
- `sc03-05` 단정 옆 7행: 「"삭제" 탭 → 사유 입력 → 확인」 — ① 바로 아래 세 동작 · ② 테스트 문서의 실행 단계.
  「문서에서는 사라지고 보관소에 선다 — 사유와 기한을 달고」 — ① 다섯 단정. 「저장이 기준이다 — 문서는 여전히
  있되(404 가 아니다) 그 feature 만 가려졌다」 — ① 단정 메시지 「문서 자체가 사라졌다 — 자동 문서를 고쳤다는
  뜻이다」(`sc03-01` 선례). 「「일정 기간」은 지운 시점의 사실이다 — 기한이 지운 시각보다 뒤에, 보관 일수만큼
  떨어져 있다」 — ① 두 단정 · ② 매핑 행 축자 · 0011 `restore_until` 문단. 「새로고침해도 같다 — 보관소는 서버
  상태다」 — ① `page.reload()` + 단정 · ② 「새로고침 후에도 동일」. 「되돌리기 → 문서에 다시 선다」 — ① 클릭 + 네
  단정. 「되돌린 뒤 보관소는 비고, 그 행은 닫힌 채 남는다(두 번 되돌리기는 거부)」 — ① `deletions.length` 0 ·
  `409` 단정 · ② 「보관소는 비고 두 번째 되돌리기는 409」.
- `sc03-06` 단정 옆 5행: 「사전 조건: 시나리오 5 에서 삭제된 feature」 — ① 바로 아래 POST · ② 매핑 행. 「실행
  단계: 같은 저장소 자동 재분석」 — ① `reanalyseToCandidates` 이름. 「화면이 그 표시를 그린다」 — ① `goto` + 네
  단정. 「승인하지 않았으므로 재분석의 인수 문서는 서지 않는다 — 되살아난 것이 없다」 — ① 단정 메시지 「사용자
  확인 없이 문서가 섰다」 · ② 매핑 행. 「첫 분석의 삭제도 그대로다」 — ① 단정 · ② 매핑 행 축자.
- **유지**: 두 spec 각각 `//` 빈 행 1 + Isolation 1행. `.legend` text-transform 같은 Playwright 함정은 이 두 spec
  에 없다(둘 다 `toHaveText`/`toContainText` 로 충분한 자리).

## 증분 — 판정 완료 원장 행 안의 13행 (원장 1·4·8·10행)

각 원래 패스 파일에 절을 더했다: [2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md)
「증분 재판정 ⑧」(원장 1행 `analysis.rs` 5행 → 제거 5 — `document()` 의 삭제 겹침 호출 위 1행은 `feature_delete::overlay`
doc 으로 옮긴 순서 계약의 사본, `CandidateView.previously_deleted` 의 AC3.3 축자 인용 4행은 `previous_deletion`
pub doc·테스트 이름·doc-tracker 의 사본) · [2026-09-18-acceptance-axis.md](2026-09-18-acceptance-axis.md)
「증분 재판정 ②」(원장 4행 `FeatureAcceptance.tsx` 5행 → 제거 4 · 유지 1) ·
[2026-09-19-candidate-strategy-axis.md](2026-09-19-candidate-strategy-axis.md) 「증분 재판정 ④」(원장 8행
`FeatureCandidates.tsx` 1행 → 제거 1) · [2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md)
「증분 재판정 ⑤」(원장 10행 `api.ts` 2행 → 제거 2). 1·8·10행은 줄 수·지문이 #112 이전 값으로 **바이트 동일하게
되돌아왔고**(590/`9757b6b5…` · 141/`a431e905…` · 85/`89959a07…` — 부모 `bf48b45` 재계산과 동일; #112 가 그
행들에 더한 것이 전건 제거된 주석뿐이기 때문), 4행은 `FeatureAcceptance.tsx` 유지 1행만큼 늘어
141/`e38dcf20…` → 146/`655305bc…`(트리거) → **142 / `2ff8f6ff…`** 이다.

## 검증

- **판정기**: `python3 tools/check-data-format-change.py --base fc6d191 --head <준비 브랜치 head> --verbose`
  → **`✅ 변경 없음 (review/data-format = success)`**(검사 파일 8/8 — D1·D2·D5·D6 어느 경로에도 안 걸린다). 감지가
  새 파일에 1줄 probe 로 잰 것을 실제 후보 트리에서 다시 돌린 것이다(「슬라이스 전 필수 절차」).
- **주석 제거 후 부모와 바이트 동일 8/8** — 줄머리 `//`·`///`·`//!`·`#`·JSDoc 과 블록 주석을 걷어낸 잔여의 md5 가
  `fc6d191` 과 같다(`feature_delete.rs` `49bea007` · `tests/feature_delete.rs` `e85d43fb` · `sc03-05` `9140733f` ·
  `sc03-06` `93ba015b` · `analysis.rs` `5ae2c305` · `FeatureAcceptance.tsx` `d7dcce35` · `FeatureCandidates.tsx`
  `f1779027` · `api.ts` `9f2f4f0f`). diff 의 `+`/`−` 줄 중 주석 줄머리가 아닌 것은 0.
- 문서 게이트 4종 rc=0(`check-scenario-e2e` — 매핑 30 · 선언 30 · `check-mockup-render` · `check-journey-mockup` ·
  `check-journey-prototype`) · `npm run build`(tsc + vite) rc=0 · `cargo test --release`(rust 1.94 호스트 부트스트랩)
  22 스위트 **200 passed / 0 failed**(#112 의 baseline 과 동일) — Rust 쪽 변경은 `///`·`//!`·`//` 줄뿐이라 컴파일 표면을 건드리지 않는다(stripper 동일). kind
  e2e(`sc03-05`·`sc03-06`)는 CI 가 집행자다.
- 전역 지문: 부모(= 트리거) `lines=2584 files=126` / `3a8bf830…` → **`lines=2507 files=126` / `3d9af1f4…`** — 순
  제거 **77**, 파일 수 불변(주석 0행이 된 파일 없음 — `tests/feature_delete.rs` 는 요약 1행이 남는다).
- 원장 산술(준비 시점 트리, base `fc6d191`): 행 열 합 2,302 + 4행 증분 +1 + 새 행 15 = **판정 완료 2,318** ·
  미판정 증분 **0** · 잔여 = 2,507 − 2,318 = **189** = 전역 파일 목록에서 원장 121파일을 집합으로 뺀 9파일
  (마이그레이션 `0009`·`0010`·`0011` 85 · D2 4 / 45 · D6·D5 2 / 59)의 행수 합과 일치.

## 범위 밖 (후속)

- **`backend/migrations/0011_feature_deletions.sql` 25행** — 사람 게이트(본문 「적용된 마이그레이션」). #112 가
  main 에 들어가 `pin deployment image` → `deploy` 브랜치(`63fec7f`)로 롤아웃됐으므로 **적용된 마이그레이션**이다.
  `0009`·`0010` 60행과 함께 **한 번의 repair** 로 모으는 것이 본문 1항에 맞다 — 그 패스를 여는 것은 사람의
  결정이다. 이 패스가 0011 의 머리 문단(보관소 원리 · 되돌린 행도 남는다 · `content_hash` 불변 · `restore_until`
  이유 · `reason` 선택 · 부분 유일 인덱스)을 코드 쪽 사본들의 **정본**으로 삼았으므로, 0011 을 판정할 때 그
  문단들은 유지 후보다.
- **이미 판정된 spec 12개의 Isolation 블록** — 13차·6a·6b 가 적어 둔 후속 후보 그대로. 이 패스는 새 파일
  `sc03-05`·`sc03-06` 에만 최신 규약을 적용했다.
- 잔여 D2 4파일/45행 · D6·D5 2파일/59행 — 사람 게이트, 변동 없음. 열린 사람 PR #108(AC4.9)은
  `0010_llm_language.sql` 번호 충돌로 main 과 이미 충돌 중이며 이 패스의 8파일과 겹치지 않는다.
