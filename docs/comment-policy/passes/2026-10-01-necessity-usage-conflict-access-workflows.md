# 2026-10-01 — 필요성 판정 네 번째 슬라이스 (비용 집계 · 충돌 해소 · 공유 요청 · CI 워크플로)

- **reconciler task**: `tbm_feature-doc-comment-necessity/rct_20261001-0003`
- **기준**: [README.md](../README.md) 「필요성 시험」 — *이 주석을 지우면, 이 코드를 고치러 온 사람이 틀린
  판단을 하거나 그 판단에 필요한 사실을 확인하는 데 상당한 비용을 치르는가?* 사유를 한 문장으로 댈 수
  없으면 지운다.
- **범위 선택**: 판정 칸이 `—` 인 행은 L 19 · E 1 이었다. 그중 열린 PR #210(Web Push 슬라이스 — 22파일)과
  파일이 겹치는 행 10(L 9 · E 1)과 마이그레이션 `.sql` 행 6(전용 PR · 사람 repair 몫)을 빼면 남는 것은
  네 행 157줄이고, 이번 슬라이스가 그 전부를 판정한다. 예산 400줄에 못 미치는 사유는 **남은 대상이
  없음**이다 — 나머지 `—` 행은 #210 착지 뒤 또는 사람 게이트 몫이다.
- **수동 승인 판정기**: `.github/workflows/` 행은 `data-format-review.yml`(판정기 SELF_PATHS)을 품는다. 이
  파일의 주석 13줄은 전부 유지로 판정해 **파일을 건드리지 않았다** — 그래서 `review/manual-approval` 은 판정기
  자동 부착 그대로다. 행의 나머지 파일에서 지운 두 줄(`ci.yml` · `image.yml`)은 SELF_PATHS 가 아니다.

## 판정 — 4행 157줄

| 덩어리 | 표면 | 판정 전 | 판정 뒤 | 제거 |
|---|---|---|---|---|
| 비용 집계 축 2파일(`backend/src/usage.rs` · `backend/tests/usage.rs`) | L | 41 | 41 | 0 |
| 충돌 해소 축 4파일(`backend/src/doc_conflict.rs` …) | L | 32 | 29 | 3 |
| 공유 요청 축 3파일(`backend/src/access_request.rs` …) | L | 24 | 21 | 3 |
| CI 워크플로 7파일(`.github/workflows/cache-warm.yml` …) | L | 60 | 58 | 2 |
| 합 | | **157** | **149** | **8** |

비주석 diff 는 0줄이다.

### 제거 목록

| 자리 | 유형 |
|---|---|
| `backend/src/doc_conflict.rs` `after_restore` 앞 「이 행이 어느 복원 뒤에 서는지(0014).」 | 코드 재진술 — 바로 아래 변수 이름과 `current_restore` 호출. 괄호의 마이그레이션 번호는 이력이다 |
| 같은 파일 `merge` doc(비 `pub`) | 코드 재진술 — 함수 이름(「두 문장을 합친 제안을 받는다」) |
| 같은 파일 `pending_proposal` doc(비 `pub`) | 코드 재진술 — 함수 이름과 반환 타입(`Option<String>`) |
| `backend/tests/access_request.rs` `ABSENT_ID` doc | 코드 재진술 — 상수 이름. 같은 값·같은 이름의 `backend/tests/analysis_cancel.rs` 상수는 주석 없이 읽힌다 |
| `e2e/tests/sc04-15-…` `ABSENT` JSDoc(비 export) | 코드 재진술·문서 재진술 — 상수 이름과 시나리오 문면 인용 |
| 같은 파일 `walk` JSDoc(비 export) | 코드 재진술 — 함수 본문의 세 단계 |
| `.github/workflows/ci.yml` `Classify changed files` 앞 「diff 가 실패하면 거르지 않고 전부 돌린다.」 | 코드 재진술 — 스텝 첫 분기가 `::warning::변경 파일을 구하지 못했다 — 전부 돌린다.` 를 찍고 둘 다 `true` 로 둔다 |
| `.github/workflows/image.yml` 핀 확인 루프 앞 「sed가 조용히 빗나가지 않았는지 확인 …」 | 코드 재진술 — 바로 아래 루프와 그 오류 문구 `expected 1 pinned image line` |

### 유지 목록 (묶음마다 필요 사유 한 문장)

**비용 집계 축** (L 41)
- `usage.rs` 모듈 머리 요약 · 「카운터 열이 아니라 파생 합계」 — 카운터 열을 들이면 쓰기 자리마다 맞아야 하고
  한 번 어긋나면 계속 틀린다는 것을 모르면 성능을 이유로 열을 더하는 판단을 한다.
- 단가 상수 doc(모델별 표가 아닌 이유) — 모델별 단가표를 더하면 낡은 표가 사실처럼 읽힌다는 결과를 막는다.
- `pub` 구조체 요약 · `by_stage` 요약 — doc 주석 수준(정책). `by_stage` 본문의 두 귀결(호출·토큰은 총합 이하,
  금액은 단계 올림으로 총합 초과 가능)은 호출자가 합계 일치를 단정하는 틀린 판단을 막는다.
- `CALL_ROWS` 의 NULL `model` 설명 · `UNIQUE` 대신 그룹 합 · SQL 이 아닌 곳에서 금액 계산 · 합계 뒤 금액 — 각각
  아직 호출이 없는 행의 의미, 두 번째 행이 생겼을 때 조용히 마지막 값을 쥐는 실패, 단가의 단일 소유, 분석당
  올림 한 번이라는 결과를 말한다.
- `backend/tests/usage.rs` 머리(`/internal` 경로로 쓰는 이유) · 방향만 다른 두 픽스처 둘 · 음성 프로브 셋 —
  테스트 머리 공유 규약, 그리고 지우면 값이 실제로 읽힌다는 증거가 사라져 「중복 단정」으로 걷히는 줄들의 이유.

**충돌 해소 축** (L 29)
- `doc_conflict.rs` 모듈 머리 · `inherit` 요약 — doc 주석 수준.
- `REJECT_REASON` doc — 회피의 단위가 사유가 아니라 문면이라는 것을 모르면 거부 사유 칸을 더하는 판단을 한다.
- `before` 필드 doc — 이름만으로는 「무엇의 이전」인지(그 편집이 고쳐 쓴 당시의 자동 문장) 확인에 비용이 든다.
- `previous_documented` · `inherited_from` · `decide_merge` · `slot_of` · `raw_document` doc — 문서 없는 분석을 건너뛰는
  이유, 재생 순서(열린 충돌 먼저), 거부가 충돌을 닫지 않는다는 계약, 저장된 번호가 아니라 지금 문서에서 찾는
  이유, 겹쳐 읽지 않아도 되는 이유를 말한다.
- `stub_merge` doc — 스텁이 real 과 갈리는 지점(충실도 경계)과 회피가 스텁 규칙이라는 사실.
- `backend/tests/doc_conflict.rs` 머리 — doc 주석 수준.
- `frontend/src/ResolveConflict.tsx` 머리 — 목업 매핑 선언(게이트가 읽는다)과, 결정 전에는 저장되지 않되 합치기만
  예외라는 상태 소유 계약.
- `sc03-07` 머리 판정 기준 · Isolation 블록 — 사전 조건의 소유가 `sc03-01` 이라는 경계와 공유 규약.

**공유 요청 축** (L 21)
- `access_request.rs` 모듈 머리 — 소유자 신원을 요청 경로에 아예 두지 않는다는 비노출 불변식이 왜 이 자리에서
  지켜져야 하는지(요청 시점에 소유자를 해석하는 것이 곧 누출 경로다).
- 응답 구조체 doc 「이 라우트가 답하는 유일한 모양」 — 모양을 갈라 답하는 변경이 곧 존재 누출이라는 결과를 막는다.
- `backend/tests/access_request.rs` 머리 · `ask` doc — 테스트 머리 공유 규약과, 필드별 비교가 새 필드를 흘린다는 이유,
  응답을 바이트로 돌려주는 이유(클라이언트가 관측하는 그대로).
- `sc04-15` 머리 · `OWNER_HANDLE` JSDoc · 절 배너 다섯 — 통째 비교의 이유, 누출 표지의 뜻, 시나리오 단계와 spec
  블록의 대응(공유 규약).

**CI 워크플로 7파일** (L 58)
- `cache-warm.yml` 머리 · `--no-run` 근거 — 캐시 스코프 규칙과 키 동기 의무, dev-dependencies 가 캐시에 드는 조건.
- `ci.yml` — 필수 체크를 늘리는 자리(`needs`) · `app` 제외 목록을 늘리기 전 확인 · 트리 해시 동일성으로 태그를
  재사용하는 근거 · `HEAD^2` 방어 · PR 에서 캐시를 저장하지 않는 둘째 이유 · 샤드별 kind 와 `fail-fast` · merge ref
  테스트 이유 · 서브셸 입출력 · revision 라벨 · 미등장 job 을 pending 으로 · 다시 빌드한 이미지 · 빈 줄 · 캐시 scope
  분리 · `comment-ledger` 에 `changes` 를 안 거는 이유 · `ci-gate` 의 `always()` 와 SKIP_OK 동기 — 각각 지우면
  필수 체크가 조용히 통과·대기로 새거나 캐시·태그가 어긋나는 판단을 하게 된다.
- `data-format-review.yml` 머리 세 문단 · base 체크아웃 · 포크 PR ref — status 부재가 곧 사람 리뷰 라우팅이라는 뜻,
  `ci-gate` 에 묶지 않는 이유, 쓰기 토큰 이벤트에서 PR 코드를 실행하지 않는 보안 불변식(SELF_PATHS — 무접촉).
- `docs-mockup-render.yml`·`docs-scenario-e2e.yml` 머리와 `--verbose` 근거 — setup 스텝이 없는 이유(stdlib 만),
  e2e 를 실행하지 않는 게이트라는 사실, 통과 로그가 공허하지 않음을 보이는 이유.
- `image.yml` — 제외 목록을 늘리기 전 확인 · main 조건 재명시 · 전체 히스토리 · 재귀 없음 · lease — 운영
  매니페스트를 움직이는 job 의 안전 조건이라 지우면 단순화가 사고로 이어진다.
