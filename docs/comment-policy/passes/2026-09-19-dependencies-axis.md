# 판정 상세 — 의존성 축 (비경합 7파일) + 증분 재판정 2건

- **판정일**: 2026-09-19
- **판정 범위**: `backend/src/dependencies.rs` · `backend/tests/dependencies.rs` ·
  `frontend/src/FeatureDependencies.tsx` · `e2e/support/dependencies.ts` ·
  `e2e/tests/sc02-05-dependency-extraction.spec.ts` ·
  `e2e/tests/sc02-06-dependency-reverse-query.spec.ts` ·
  `e2e/tests/sc02-07-dependency-export.spec.ts`
  · 증분 재판정: `backend/src/worker_api.rs`(원장 1행) · `backend/src/bin/worker.rs`(원장 5행)
- **기준 트리**: `66bb7a5` (main, #74 머지 직후)
- **reconciler task**: `tbm_feature-doc-comment-redundancy/rct_20260919-0002`

규칙은 [../README.md](../README.md), 판정 결과의 표면은 [../ledger.md](../ledger.md)에 있다.
이 파일은 이번 범위의 **근거**만 담는다.

## 범위를 이 7파일로 고른 이유

**⑴ 이번 트리거가 들인 주석이 곧 이 범위다.** 직전 소유자(`rct_20260918-0005`)가 닫힌 뒤
잔여를 키운 것은 #71(`2c7a6d3`, 슬라이스 5b-1)이 들인 **신규 7파일 / 249행**이고, 그 일곱이 곧
「의존성 축」이다. 한 커밋이 백엔드 로직·백엔드 통합 테스트·프론트 화면·e2e 하네스·e2e spec 셋으로
**한 기능의 전 층**을 낸 단위라, 같은 문장이 층마다 몇 벌로 되풀이되는지를 한 번에 볼 수 있다.

**⑵ 열린 PR 전수 대조 — 이 일곱은 무접촉이다** (2026-09-19T01:4xZ, `/pulls/<n>/files` 실측).

| PR | 상태 | 범위 내 접촉 파일 |
|---|---|---|
| #75 | open | `backend/src/analysis.rs` · `repo_scan.rs` · `lib.rs` · `backend/tests/diff.rs`(신규) · `backend/src/diff.rs`(신규) · `frontend/src/{AnalysisDiff(신규),AnalysisProgress,App,api}` · `e2e/tests/sc02-08-…`(신규) |
| #72 | open | `frontend/src/{App,CredentialsSetup,HomeRepositories,api,index.css}` |
| #64 | open | `tools/check-data-format-change.py`(신규) |
| #26 | draft | `tools/check-mockup-render.py` |
| #17 | draft | `frontend/src/{ConnectRepository,CredentialsSetup,HomeRepositories,index.css}` |

이번 7파일은 **다섯 PR 모두와 무접촉**이다. `sc02-08`은 #75가 새로 내는 파일이라 이 패스가 건드리는
`sc02-05·06·07`과 줄이 겹치지 않는다.

**⑶ 증분 2건만 함께 닫았다 — 세 번째는 경합이라 뺐다.** #71은 이미 판정된 범위에도 주석을 더했다:
원장 1행에 **73행**(`analysis.rs` +36 · `worker_api.rs` +37), 원장 5행에 **12행**(`bin/worker.rs`).
이 중 `worker_api.rs`와 `bin/worker.rs`는 무접촉이라 같은 패스에 묶었고(원장 규약: 증분은 새 행이
아니라 원래 행의 결과 칸 갱신), **`analysis.rs` +36행은 #75가 같은 파일에 +162행을 얹고 있어
판정하지 않았다**. 지금 같은 줄을 지우면 확실히 충돌한다. #75가 닫힌 뒤 다음 패스가 집는다.

**⑷ 마이그레이션 `0008_feature_dependencies.sql` 35행은 이 슬라이스에 넣지 않았다.** 본문
「적용된 마이그레이션」 절의 전용 PR · 수동 repair · 사람 승인 · 머지 직전 SHA-384 대조를 거치는
별도 경로다. 이 파일이 들어오며 마이그레이션 잔여는 104행/7파일 → **139행/8파일**이 됐다.

## 집계

| 파일 | 판정 전 | 판정 후 | 지문 기준 감소 | diff |
|---|---|---|---|---|
| `backend/src/dependencies.rs` | 70 | 31 | 39 | 62행 삭제 · 22행 재작성 |
| `backend/tests/dependencies.rs` | 45 | 15 | 30 | 35행 삭제 · 5행 재작성 |
| `frontend/src/FeatureDependencies.tsx` | 47 | 18 | 29 | 38행 삭제 · 8행 재작성 |
| `e2e/tests/sc02-05-dependency-extraction.spec.ts` | 27 | 11 | 16 | 20행 삭제 · 4행 재작성 |
| `e2e/tests/sc02-06-dependency-reverse-query.spec.ts` | 25 | 14 | 11 | 14행 삭제 · 3행 재작성 |
| `e2e/tests/sc02-07-dependency-export.spec.ts` | 20 | 12 | 8 | 11행 삭제 · 3행 재작성 |
| `e2e/support/dependencies.ts` | 15 | 11 | 4 | 9행 삭제 · 5행 재작성 |
| **7파일 합계** | **249** | **112** | **137** | **189행 삭제 · 50행 재작성** |
| 증분 ③ `backend/src/worker_api.rs` | 170 | 152 | 18 | 33행 삭제 · 15행 재작성 |
| 증분 ① `backend/src/bin/worker.rs` | 67 | 60 | 7 | 10행 삭제 · 3행 재작성 |
| **이 패스 전체** | — | — | **162** | **232행 삭제 · 68행 재작성** |

재작성이 많은 것은 직전 인수 축 패스와 같은 이유다 — AC 조항 인용 한 줄만 들어내면 문단이 끊기므로
남는 지식을 한 문장으로 다시 적었다. diff 삭제(232)와 지문 감소(162)의 차는 재작성 68행과 블록 주석
닫는 줄(`*/` — 지문 패턴에 잡히지 않는다)이다.

## 지우지 않은 것부터 — 기계가 읽는 주석

- `// 검증 시나리오: 02-feature-representation.md#시나리오 5 · 6 · 7` (sc02-05·06·07) —
  **손대지 않았다.** `tools/check-scenario-e2e.py`가 파싱한다.
- `// mock-exception: LLM-01 …` 1건(`backend/src/dependencies.rs`의 `stub:` 인자) —
  `docs/e2e-mocking-policy.md`의 표기 규약이다. 그대로 뒀다.

둘 다 as-is 지문의 `DIRECTIVE` 패턴에 걸려 **지문에 보이지 않는다** — 실수로 지워도 수치가 알려주지
않으므로 아래 검증 절에 개수 대조를 따로 넣었다.

## `backend/src/dependencies.rs` — 39행 제거

### ① PRD-2 AC2.4·AC2.5 조항을 통째로 옮겨 적은 모듈 머리 (17행)

머리 24행 중 17행이 복원 경로 ②였다. 「PRD-2 의 뒤쪽 절반이다」로 시작하는 문서 구조 서술(4행),
AC2.4·AC2.5 조항 본문의 불릿 재진술(6행), `docs/test/02` 시나리오 5 실행 단계 인용(3행),
여정 `JRN-review-feature` 예외 표 인용(「근거 없음으로 명시. 임의로 채우지 않음」, 2행).
남긴 7행은 문서 어디에도 없는 두 가지다 — **파이프라인 단계를 더하지 않고 요청 행 하나가 게이트이자
실행 상태를 겸한다**(설계 결정의 부재를 코드는 말하지 않는다)와 **근거를 지어내는 대신 근거만
떨어뜨린다**(버리면 의존성이 사라지고 채우면 거짓이 된다는 비대칭).

### ② 선언 재진술 (9행)

- `MAX_ITEMS` doc의 AC2.4 목적 인용 1행 — 상한이 왜 있는지는 남기고 조항 인용만 뺐다.
- `CATEGORIES` doc의 「AC2.4 가 열거한 7종」·「목업이 전체 7종이라 적어 둔 그 7」 1행 —
  `[&str; 7]`과 상수 이름이 이미 말한다. **한 곳에서만 정의된다**는 교차 파일 불변식은 남겼다.
- `request_status` doc의 첫 줄(모듈 이름 + `feature_dependency_requests.status` 재진술) 2행 —
  중간 상태가 없는 이유(claim 배타성 · 리스 회수 자리 없음)는 동시성 계약이라 남겼다.
- `is_terminal` 1행 · `is_category` 1행 — 함수 이름이 곧 술어다.
- `items()` doc의 동작 서술 3행(「알 수 없는 분류·빈 이름은 버리고 …」) — 함수 본문 그대로다.
  **유일한 해석 지점**이라는 불변식만 남겼다.

### ③ 프롬프트 경로 상한 안내 주석 (3행)

선언이 아니라 **선언의 부재**에 붙은 주석이다(「상한은 `cross_cutting::input_paths` 가 정한 값을
그대로 쓴다」). `derive`가 그 함수를 실제로 호출하는 줄이 같은 말을 하고 있어 복원 경로 ①이다.

### ④ 코드 아래 줄을 그대로 옮긴 인라인 · 테스트 doc (10행)

- `// 숨김 디렉터리(.github 등)와 읽을거리는 의존성이 아니다.` — 바로 아래 `starts_with('.')`와
  확장자 `matches!`가 읽힌다.
- `stub_dependencies` doc 3행 중 「feature 자신이 발견된 자리는 언제나 한 줄로 들어간다」는 함수
  첫 분기 그대로, AC2.4 성질 인용은 ②다. **stub 이 트리에서 파생되는 이유**(배선이 끊겨도 통과하는
  상수를 e2e 가 단정하지 않게)는 충실도 경계라 남겼다.
- `derive` doc 1행 — 반환 문서가 검증된 것이라는 사실만 남기고 검증 방식 서술은 본문이 말한다.
- 단위 테스트 doc 4행(`category_for_skips_what_is_not_a_dependency` ·
  `fabricated_evidence_becomes_no_evidence` · `stub_is_derived_from_the_tree`)과 정렬 인라인 1행 —
  **테스트 이름과 단정 메시지가 같은 문장을 한국어로 한 번 더 쓴 것**이다.

### 판단이 갈려 남긴 것

- `category_for` doc의 「영리하기보다 관습적이다」 — 인수 축 패스가 `is_test_path`의 같은 문장을
  남긴 것과 같은 판단이다(왜 이 함수가 답변 검증의 자리가 아닌지는 코드에서 복원되지 않는다).
- `schema()` doc 1행 — 인수 축에서 같은 문장(`The JSON shape … sent to the provider verbatim`)을
  남겼다. 두 파일이 다르게 판정되지 않도록 맞췄다.
- `detail()`·`document()`의 한 줄 요약 — `pub` 항목의 요약 1줄은 본문 「유지 대상」이다.

## `backend/tests/dependencies.rs` — 30행 제거

### ① 테스트 이름을 한국어로 다시 쓴 doc (11행)

모듈 머리의 세 불릿(「요청이 곧 게이트다」 · 「행으로 적재된다」 · 「실패가 앞의 답을 지우지
않는다」)은 그대로 세 테스트의 이름이다 — `nothing_is_traced_until_someone_asks` ·
`…reverse query…` · `a_failed_retrace_keeps_the_previous_answer`. 각 테스트 위에 붙은 doc 6행도
같은 문장의 세 번째 벌이라 함께 지웠다. 머리에 남긴 것은 **픽스처를 손으로 쓰지 않고
`derive`의 stub 산출물을 워커 라우트로 넣는다**는 한 가지다(충실도 경계).

### ② AC 조항·검증 방법 인용 (8행)

`AC2.4:` · `AC2.5 의 검증 방법 그대로 —` · `AC2.5 의 나머지 절반 —` 로 시작하는 doc 세 벌과
`(AC4.7)` 꼬리표. 전부 `docs/prd/`에서 그대로 확인된다.

### ③ 단정을 되풀이한 인라인 (8행)

`// 분석은 멈춰 있다`(단정 메시지 `queue should be at rest`), `// 한 번 답한 뒤에는 다시 제안되지
않는다`(`the request is settled`), `// 분류가 7종 밖이면 …`(`BAD_REQUEST` 단정),
`// 다시 묻고, 이번엔 실패로 답한다`(바로 아래 `"status": "failed"`),
`// 의존성은 파이프라인 단계가 아니다`(모듈 머리 · `src/dependencies.rs` 머리와 세 벌째),
`run_to_confirmed` doc 1행, 공유 항목 선택 인라인 1행.

### ④ 판단이 갈려 남긴 것 (3건)

- `// 일곱 칸은 언제나 그려진다 — 빈 분류도 정보다.` — 빈 분류를 지우지 않는 것은 설계 선택이고
  단정(`len() == 7`)은 그 선택의 결과만 보여 준다. **e2e 쪽의 같은 문장은 지웠다**(세 벌 중 한 벌만
  남긴다는 기존 판정과 같다).
- `// 남의 분석은 존재하지 않는다.` — 403이 아니라 404를 쓰는 이유(존재 자체를 흘리지 않는다)는
  단정에서 복원되지 않는다.
- `// 사유 없는 실패는 받지 않는다 — 사유 없는 거부를 받지 않는 것과 같은 이유다.` — 다른 라우트와의
  규칙 일치를 가리킨다.
- `urlencoding` doc 2행(바이트 단위 인코딩, `char` 로 하면 코드 포인트가 나간다)은 상류 규약의
  함정이라 손대지 않았다.

## `frontend/src/FeatureDependencies.tsx` — 29행 제거

### ① 목업·여정·doc-tracker 를 옮겨 적은 파일 머리 (21행)

머리 25행 중 21행이 ②였다. 「목업이 자기 화면 카드로 그리므로 독립 화면이다」 + 2026-09-18
「문서 권위 순서」 결정 인용(4행), AC2.5 의 구조화 저장 근거 재진술(6행), 여정의 좁은 화면 규정
인용(3행), `docs/doc-tracker.md` 「알려진 목업↔구현 편차」 2건의 등재 내용 재진술(6행), 제목 줄의
AC 꼬리표. 남긴 4행은 목업 포인터 한 줄과 **화면이 그리는 것은 문서가 아니라 행이고, 목록도
그래프도 같은 행을 읽는다**는 화면 자신의 계약이다.

### ② 나머지 (8행)

- `CATEGORY_LABELS` doc의 AC2.4·목업 라벨 출처 1행 — **키 없는 라벨이 빈 여덟 번째 분류를 조용히
  그린다**는 함정은 남겼다.
- `onBack` prop doc 1행 — 타입과 이름이 그대로 말한다.
- `Graph` doc 1행 — 목업의 예시 노드 수 인용을 빼고 **고정 모양이면 모든 feature 가 같은 그림이
  된다**만 남겼다.
- 편차 상수 블록 doc 2행 — doc-tracker 등재 사실은 ②이므로 「편차를 한 자리에 모은다」만 남겼다.
- `Appbar` doc 3행 — `‹`·`✕ 나가기`와 슬롯 배치는 바로 아래 JSX 가 그린다. **둘 다 같은 곳으로
  돌아간다**만 남겼다.

## e2e 3 spec + 하네스 — 39행 제거

### ① 시나리오 문서 인용 (22행)

세 spec 모두 머리에서 시나리오 5·6·7의 **실행 단계와 기대 결과를 문장째** 옮겨 적었다
(sc02-05 11행 · sc02-06 11행 · sc02-07 9행 중 각각 대부분). `docs/test/02-feature-representation.md`
에서 그대로 확인된다. 남긴 것은 세 spec 공통으로 **기대값을 상수로 박지 않는 이유**(문서의 예시는
저장소의 사실이 아니다) — 인수 축 패스가 sc02-01에 남긴 것과 같은 판단이다.

### ② 구분선 주석 (3행)

sc02-05의 `// ── 누르기 전 ───…` · `// ── 누른 뒤 ───…` · `// ── 종단 의존성 화면이 …` —
바로 아래 코드가 이미 절을 나눈다(본문의 제거 유형 ④).

### ③ 단정·문서를 되풀이한 인라인 (10행)

`// 「각 항목에 코드 근거가 있다」`(기대 결과 인용 + 단정 재진술), `// 분류 필터가 그 축으로 실제로
자른다`(바로 아래 `selectOption` + 개수 단정), `// 일곱 칸은 …`(백엔드 테스트와 두 벌),
`(AC4.7 과 같은 판정)` 꼬리표, 다층 의존성 열거 인용 1행.

### ④ 하네스(`e2e/support/dependencies.ts`) 4행

「세 AC2.4/AC2.5 spec 이 공유한다」의 AC 꼬리표와 「`acceptance.ts` 와 같은 이유로 여기 둔다」의
세 spec 역할 열거. 인수 축이 판정한 `acceptance.ts` 머리와 **같은 모양으로** 맞췄다.
`testDir` 밖이라 시나리오↔spec 단위로 세지 않는다는 2행은 `check-scenario-e2e.py`의 동작을 설명하는
운영 지식이라 남겼다.

### 남긴 것

- sc02-05의 `// 새로고침 후에도 같다 — 의존성은 서버 상태다.` — sc02-01의 같은 문장을 인수 축이
  남겼다.
- sc02-06의 다른 차원 질의 2행 — 기대 개수를 왜 계산해서 쓰는지(그 이름이 다른 분류로도 존재할 수
  있다)는 코드에서 복원되지 않는다.
- sc02-07의 `content-disposition` · `형식이 스스로를 설명한다` 2행 — 무엇을 관측하는지가 단정만으로는
  드러나지 않는다.
- 세 spec 공통의 `Isolation:` 블록 4행 — 워커 임대 계약이라 직전 두 패스에서도 유지했다.

## 증분 재판정 ③ — `backend/src/worker_api.rs` (원장 1행, 18행 제거)

#71이 더한 37행을 판정했다.

- **제거 18행**: `documents/{kind}` 라우트 주석의 AC2.4 인용과 「의존성은 단계가 아니다」 2행
  (`store_dependencies` doc 및 `dependencies.rs` 머리와 세 벌째 — `feature key 가 경로 구분자를
  품는다`만 남겼다), `dependency_requests` 필드 doc의 AC 꼬리표와 「Analysis Progress 가 여섯 번째
  단계를 그리게 된다」 5행, `pending_dependency_requests` doc의 함수 이름 재진술과
  「승인된 후보만 추적한다 (AC2.1)」 4행(SQL의 `c.decision = 'approved'` 가 그대로 말한다),
  `status` 필드의 rustdoc 링크만 있는 교차 참조 1행, `submit_dependencies` doc의 AC2.5 근거 서술
  6행.
- **유지 19행**: **요청 행이 곧 게이트**이고 실패가 영원히 재시도되지 않는다는 계약, 같은 패스 중에
  들어온 요청이 승인과 같은 경합을 탄다는 것, 재실행이 **같은 트랜잭션에서 행을 교체**하고 실패는
  아무것도 지우지 않는다는 것 — 전부 동시성·트랜잭션 계약이라 코드가 아니라 의도의 자리다.

## 증분 재판정 ① — `backend/src/bin/worker.rs` (원장 5행, 7행 제거)

#71이 더한 12행을 판정했다.

- **제거 7행**: `dependency_requests` 필드 doc의 「파이프라인 단계가 아니다 …」 2행(같은 문장의 네
  번째 벌), 루프 인라인의 AC 꼬리표와 「Analysis Progress 는 여전히 다섯 단계를 그린다」 2행,
  `run_dependencies` doc의 라우트 선택 근거 3행(`worker_api.rs` 쪽에 한 벌 남겼다).
- **유지 5행**: **한 feature 의 실패가 잡 전체를 죽이지 않는다**(요청 행이 사유를 들고, 다른
  feature 의 결과는 인질이 되지 않는다)는 실패 격리 계약과 「비어 있으면 아무도 요청하지 않았다」.

## 검증

판정 시점 트리(`66bb7a5`)에서 확인한 것 — 실행 지침의 대조 절과 같은 명령이다.

1. **코드 무변경**: `git diff` 의 `+`/`-` 줄이 **전부 주석 줄이거나 빈 줄**이다(비주석 변경 0줄).
2. **범위 지문**: 7파일 `lines=249` → `lines=112` ·
   `e2ff313a778df1e7388fc7bbb07bd72c323ead751d854c5db743c3c2eee133e9`.
3. **전역 지문**: `lines=3365 files=106` / `a43e22ba…` → `lines=3203 files=106` /
   `afd4b8e3ca44ec129b833a18c8c9d506f664148021b50ef990eb88328f398714`
   (**부모 대비 순 제거 162행**. 절대값은 base 가 움직이면 함께 움직이므로 판정의 기준은 순 제거
   162행이다).
4. **기계 판독 주석 보존**: `검증 시나리오:` 3건(sc02-05·06·07) · `mock-exception:` 1건
   (`dependencies.rs`) 모두 그대로다.
5. **문서 게이트 3종**: `tools/check-scenario-e2e.py` · `check-mockup-render.py` ·
   `check-journey-mockup.py` 모두 `rc=0`(호스트에 node 가 없어 `check-journey-prototype.js` 는
   CI 잡에서 확인한다).

## 증분 재판정 ① — `backend/src/dependencies.rs` +1행 (2026-09-21 · `rct_20260921-0001`)

`#93`(`8205b7a`)이 `schema()` 의 `"evidence"` 위에 더한
`// Required-but-nullable; see feature_candidates::schema.` 1행을 **제거**했다 — 원장 4행
`acceptance.rs` 의 것과 **같은 문장의 두 번째 벌**이고 근거도 같다(인접 선언의 축자 재진술 + 교차
참조뿐). 정본은 `llm.rs::assert_strict_schema` 옆, 이 파일의 테스트
`schema_is_accepted_by_openai_strict_mode` 가 그 자리를 이름으로 가리킨다. 근거 전체는
[2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 「증분 재판정 ⑤」.
결과: 이 범위의 줄 수·지문이 #93 이전 값 **112 / `e2ff313a…`** 으로 되돌아왔다. 비주석 코드 무접촉
(스트립 잔여 450 == 450). 같은 커밋이 이 파일의 프롬프트 문자열(`set evidence to null`)을 바꿨지만
그것은 주석이 아니라 코드다 — 판정 표면 밖.

---

## 원장에서 옮겨 온 증분 재판정 기록 (2026-09-26 형식 이전)

아래는 `ledger.md`의 결과 칸에 쌓여 있던 증분 재판정·정정 기록을 **문면 그대로** 옮긴
것이다. 형식 이전(템플릿 「원장 형식」)이 원장에 표와 「읽는 법」만 두기로 하면서, 각 행의
경위는 그 행의 패스 파일로 돌아왔다. 옮기면서 한 글자도 고치지 않았고 판정을 새로 하지
않았다 — 행을 가리키는 순번도 당시 표기 그대로다.

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ③**(2026-09-19): #71이 `worker_api.rs`에 더한 37행을 판정해 **순 제거 18행**(라우트 선택 근거의 세 벌째 · AC 꼬리표 · 함수 이름 재진술 · SQL이 이미 말하는 「승인된 후보만」 · rustdoc 링크만의 교차 참조) · 유지 19행(요청 행이 곧 게이트 · 실패는 재시도되지 않음 · 같은 트랜잭션에서 행 교체) — [passes/2026-09-19-dependencies-axis.md](2026-09-19-dependencies-axis.md)

### 원장 행 5 — `backend/src/bin/worker.rs` · `deploy/e2e/kustomization.yaml` · `deploy/k8s/deployment.yaml` · `deploy/k8s/secret.yaml.example` · `deploy/k8s/worker-deployment.yaml` · `e2e/tests/sc04-07-api-availability-without-workers.spec.ts` · `e2e/tests/sc04-08-worker-horizontal-scale.spec.ts` (워커 · 더블 배선 축 비경합 7파일)

**증분 재판정 ①**(2026-09-19): #71이 `bin/worker.rs`에 더한 12행을 판정해 **순 제거 7행**(같은 문장의 네 번째 벌 · AC 꼬리표 · 라우트 선택 근거) · 유지 5행(한 feature 의 실패가 잡을 죽이지 않는다는 격리 계약) — [passes/2026-09-19-dependencies-axis.md](2026-09-19-dependencies-axis.md)

### 원장 행 6 — `backend/src/dependencies.rs` · `backend/tests/dependencies.rs` · `frontend/src/FeatureDependencies.tsx` · `e2e/support/dependencies.ts` · `e2e/tests/sc02-05-dependency-extraction.spec.ts` · `e2e/tests/sc02-06-dependency-reverse-query.spec.ts` · `e2e/tests/sc02-07-dependency-export.spec.ts` (의존성 축 비경합 7파일)

**증분 재판정 ①**(2026-09-21): #93이 `dependencies.rs`에 더한 1행(4행과 같은 문장의 두 번째 벌)은 같은 근거로 **제거 1행** — 줄 수·지문이 #93 이전 값(112 / `e2ff313a…`)으로 되돌아왔다 — [passes/2026-09-19-dependencies-axis.md](2026-09-19-dependencies-axis.md) 「증분 재판정 ①」 · **미판정 증분 없음**

