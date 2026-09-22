# 판정 상세 — 빠진 feature 직접 추가 축 (슬라이스 6b 가 들여온 새 파일 5개) · 2026-09-21

reconciler task `rct_20260921-0010`(모델 `tbm_feature-doc-comment-redundancy`). 자매 모델
`tbm_feature-doc-docs-impl` 의 슬라이스 6b(**PR #107**, `89a1625`, AC3.2 feature 문서의 추가)가 판정
대상 범위에 주석 **순 +197행 / 새 파일 6**을 들여왔다(삭제 2는 기존 doc 문장의 재작성이라 판정된
주석이 걷힌 것은 0). 그중 마이그레이션 `0010_feature_additions.sql` 30행은 본문 「적용된 마이그레이션」
절의 사람 게이트 몫이라 **이 패스가 다루지 않는다**. 나머지 167행을 한 패스로 판정했다 — **새 파일
5개 / 151행**(이 파일의 본문)과 **판정 완료 원장 행 안으로 들어온 증분 6파일 / 16행**(원장
1·2·5·8·10·17행 — 각 원래 패스 파일에 절을 더했다, 아래 「증분」 절). 문서 편집 축
([2026-09-21-doc-edit-axis.md](2026-09-21-doc-edit-axis.md), #92 → PR #106)과 **같은 판정형**이고, #107 이
6a 의 축을 그대로 재사용했으므로 그 패스의 판정을 같은 자리마다 같은 방향으로 적용했다.

## 범위와 결과

| 파일 | 유입 | 제거 | 유지 | 비고 |
|---|---|---|---|---|
| `backend/src/feature_add.rs` | 84 | **56** | 28 | 모듈 머리 21행 → 요약 1행 · `///` 필드·fn doc 대부분 · 단위 테스트 doc 3 |
| `backend/tests/feature_add.rs` | 19 | **15** | 4 | 모듈 머리 10행 → 요약 2행 · 나머지는 테스트 이름·단정 메시지가 복원 |
| `e2e/tests/sc03-03-manual-feature-add-with-evidence.spec.ts` | 22 | **18** | 4 | 머리 문단 4행(doc-tracker 매핑 행 축자) · Isolation 4행 → 1행 · 단정 옆 주석 10행 |
| `e2e/tests/sc03-04-manual-feature-add-no-evidence.spec.ts` | 16 | **15** | 1 | 머리 8행(매핑 행 축자 · 「워커를 임대하지 않는다」) · 단정 옆 주석 6행 |
| `frontend/src/AddFeature.tsx` | 10 | **2** | 8 | 머리 문단의 「지어내지 않는다」 재진술 1행 · 내부 fn JSDoc 1행 |
| **합** | **151** | **106** | **45** | 범위 지문 **45 / `b596e739…`** |

증분 6파일(원장 행 안): `backend/src/analysis.rs` 2 → 제거 2 / `backend/src/config.rs` 3 → 제거 3 /
`deploy/e2e/kustomization.yaml` 2 → 제거 2 / `frontend/src/FeatureCandidates.tsx` 1 → 제거 1 /
`frontend/src/api.ts` 7 → 제거 5 · 유지 2 / `backend/src/doc_edit.rs` 1 → 제거 1 — **16행 중 제거 14 · 유지 2**.
합쳐 **순 제거 120행**.

**diff 기준**: 11파일 131행 삭제 · 11행 삽입(재작성 — `feature_add.rs` 의 `overlay`·`feature_json`·
`stub_matches` doc 과 `DraftDependency`·설치 토큰 줄, `AddFeature.tsx` 머리, `analysis.rs`·`doc_edit.rs` 의
#107 이전 문장 복원). 지문 감소(120)와 diff 삭제 줄 수가 갈리는 것은 재작성 때문이다.

## 이 축의 복원 경로 — 6a 와 같은 세 겹에 마이그레이션 한 겹

#107 은 자기 설계 판단을 **세 자리에 이미 적어 두었다**: ② `docs/doc-tracker/2026-09.md` 의 슬라이스 6b
변경 이력 행(「근거 찾기는 큐를 타지 않고 API 프로세스가 직접 한 번 부르며 … 워커가 1단계에서 본 경로
목록은 저장되지 않으므로 같은 스캔을 한 번 더 한다」 · 「확정된 추가는 자동 문서를 고치지 않고 읽는
자리에서 `features[]` 끝에 겹쳐진다 … 편집 겹치기보다 먼저」 · 「근거를 못 찾으면 지어내지 않는다는
규칙은 코드가 지킨다 — 답의 근거 경로가 트리에 없으면 버리고 남는 시나리오가 없으면 「근거 없음」」 ·
「확정한 의존성 후보는 AC2.5 의 행으로 실려 의존성 화면·역방향 질의가 자동 추출분과 같은 표에서
읽는다」) · ② 같은 파일의 e2e 매핑 행 두 개(`sc03-03`·`sc03-04` — 「무엇을 관측하는가」와 「관측 대상이
아닌 것」 칸) · ③ PR #107 본문(같은 문장들) · ① 마이그레이션 `0010_feature_additions.sql` 의 머리 주석
(`feature_candidates` 에 행을 더하지 않는 이유 · `analysis_documents` 를 고치지 않는 이유 · `key` 접두사 ·
`source` 어휘와 NULL 의 뜻). 그래서 `feature_add.rs` 모듈 머리 20행은 네 벌째였고, 두 spec 의 머리
문단은 매핑 행의 축자였다. 0010 주석은 사람 게이트 몫이라 이 패스가 손대지 않으며, **저장 계약의 정본은
그 파일이다** — 모듈 머리가 스스로 「이유는 0010 마이그레이션 주석에 있다」고 적어 둔 그대로다.

## `backend/src/feature_add.rs` — 제거 56행

### 모듈 머리 `//!` 21행 → 1행 (제거 20)

- 유지: `//! AC3.2: 자동 추출이 놓친 feature 를 사람이 한 문장으로 더하는 흐름.` — 모듈 요약 1행.
- 제거 「사람이 「이건 왜 없지」 싶은 기능을 적으면 … 편집 제안(AC3.1)과 같은 이유로 큐를 타지
  않는다 — 사람이 화면 앞에서 그 답을 기다리는 한 번의 호출이고 … 여정(`STP-add-missing`)이 그리는
  흐름이 아니다」 5행 + 빈 1 — ② doc-tracker 6b 행 「근거 찾기는 큐를 타지 않고 API 프로세스가 직접 한
  번 부르며」 · ② 6a 행 「제안은 큐를 타지 않는다 — 여정이 못 박은 3탭(F8)이 …」 · ③ PR #107 「6a 의
  축을 그대로 재사용」 · `doc_edit.rs` 머리의 같은 문단은 앞 패스가 걷었다.
- 제거 「**근거를 못 찾으면 만들어내지 않는다.** 이것은 프롬프트에 적힌 부탁이 아니라 코드가 지키는
  규칙이다 — … ([`grounded`]) … 4단계·5단계가 자기 답을 같은 방식으로 거르는 것과 같은 방어선이다.
  근거 없는 시도는 그대로 행으로 남고 …」 5행 + 빈 1 — ② doc-tracker 6b 행 축자 · ③ PR 본문 「지어내지
  않는다는 규칙을 코드가 지킨다」 · ① 0010 「근거를 찾지 못한 시도도 행으로 남는다: 그때 사람은 「근거
  없음」을 단 채로 추가하거나 취소할 수 있고」 · ① `grounded` 본문(`known(evidence)` · `if scenarios.is_empty()`).
- 제거 「**확정하기 전에는 아무것도 바뀌지 않는다.** … 확정된 행만 읽는 시점에 인수 문서 끝에
  겹쳐진다([`overlay`]). 자동 문서를 고치지 않는 이유는 0010 마이그레이션 주석에 있다」 3행 + 빈 1 — ①
  0010 「왜 인수 문서(`analysis_documents`)를 직접 고치지 않는가」 문단 · ② doc-tracker · ③ PR · ① `overlay`
  doc(유지) — `doc_edit.rs` 머리의 같은 문단과 같은 판정.
- 제거 「트리는 이 프로세스가 직접 읽는다. 워커가 1단계에서 본 경로 목록은 저장되지 않으므로 … 같은
  스캔을 여기서 한 번 더 한다 — 같은 함수, 같은 더블 이름(`FEATUREDOC_DOUBLE_REPO_SCAN`)이라 워커와
  다른 트리를 볼 수 없다」 3행 + 빈 1 — ② doc-tracker 6b 행 축자 · ② `docs/e2e-mocking-policy.md` EXT-03
  배선 표(#109 가 「API · 워커」로 등재) · ③ PR 본문 · ① `tree_of` 본문(`repo_scan::scan` +
  `doubles.repo_scan`).

### 선언·필드 doc (제거 15)

| 줄 | 복원 경로 |
|---|---|
| `SOURCE_USER_LLM`/`SOURCE_USER_DIRECT` 「AC3.4 의 출처 중 이 흐름이 남기는 두 값 — 모델이 근거를 찾아 초안을 냈으면 … 사람이 직접 한 것이다」 2행 | ① 0010 `source` 문단(같은 문장) · ① 이름·값 — AC 꼬리표. `doc_edit.rs` 의 `SOURCE_USER_LLM` 과 같은 판정 |
| `KEY_PREFIX` 「자동 후보의 키는 발견 위치에서 파생되므로 이 접두사를 가질 수 없고, 그래서 두 공간이 겹치지 않는다」 2행 | ① 0010 `key` 문단(「자동 후보의 키가 발견 위치에서 파생되는 것과 달리 … 접두사를 둔다」) |
| `DraftDependency` 「분류는 AC2.4 의 7종 안이어야 한다」 | 꼬리표만 걷고 요약은 유지(재작성, 지문 ±0) — ① `grounded` 의 `dependencies::is_category` |
| `AdditionView.evidence_found` 「근거를 찾았는가. 화면의 「근거 있음」 배지와 「근거 없음」 안내가 이 값으로 갈린다」 | ① 이름 · `AddFeature.tsx` 의 조건 렌더 · `api.ts` 의 같은 문장(함께 제거) |
| `AdditionView.source` 「AC3.4 의 출처. 확정 전에는 아직 어느 쪽도 아니다」 | ① 0010 「확정 전에는 아직 어느 쪽도 아니므로 NULL」 · `api.ts` 의 같은 문장(함께 제거) — AC 꼬리표 |
| `ListView` 세 필드 「승인된 자동 후보 수 — 확정될 목록의 한 축」 · 「확정된 직접 추가 수 — 다른 한 축」 · 「둘의 합. 화면의 「확정될 목록」 수가 이 값이다」 3행 | ① 이름 · `list()` 의 `final_count: approved + confirmed` · `api.ts` `finalCount` 의 같은 문장(함께 제거) |
| `DraftReq.request` 「… 빈 요청은 트리도 모델도 부르지 않는다」 | ① `draft()` 의 `if request.is_empty() { return Err(BadRequest(…)) }` |
| `DecisionReq.decision` 「`confirm` 또는 `cancel`」 | ① `decide()` 의 `match` 가지 |
| `list()` 「이 분석의 추가 시도 전부와, 확정될 목록의 수」 | ① `ListView` 필드 |
| `draft()` 「한 줄을 초안 하나로 바꾼다. 문서는 아직 그대로다」 | ① 본문(`analysis_documents` 에 쓰지 않는다) · `api.ts` `draftAddition` JSDoc(export 요약 1줄이라 **그쪽을 정본으로 유지**) |
| `addition()` 「이미 만들어진 초안 하나. 화면이 새로고침을 견디는 근거다」 | ① `routes()` 의 「초안도 주소를 가진다 — … 새로고침이 같은 초안을 다시 그린다」(유지, 정본) |
| `decide()` 「확정이면 feature 가 되어 겹쳐 읽히기 시작하고, 취소면 시도로만 남는다」 | `api.ts` `decideAddition` JSDoc 과 같은 문장(export 요약이라 그쪽 유지) · ① `overlay` 의 `status` 필터 |
| `tree_of()` 「이 분석의 저장소 트리 — 워커가 1단계에서 보는 것과 같은 스캔이다」 | ① 본문 `repo_scan::scan(…, doubles.repo_scan, …)` · ② doc-tracker 6b 행 |
| `row_of()` 「이 분석의 시도 한 행. 분석 소유권은 부른 쪽이 이미 확인했다」 | ① 호출자 둘 다 `owned_analysis` 를 먼저 부른다 — 호출자 재진술(`edit_row()` 와 같은 판정) |
| `schema()` 「모델이 답할 모양. 찾았는지, 초안, 의존성 후보 — 세 칸 밖의 것은 받지 않는다」 | ① `additionalProperties: false` · `required` 세 칸 |

### 인라인·`overlay`·`grounded`·스텁 (제거 18)

| 줄 | 복원 경로 |
|---|---|
| `decide()` 「근거를 찾았으면 사람이 모델의 도움을 받은 것이고, 아니면 사람이 직접 더한 것이다」 | ① 바로 아래 `match` (`evidence_found != 0 => SOURCE_USER_LLM`) · 0010 `source` 문단 — 세 벌째 |
| `decide()` 「사람이 확정한 의존성 후보는 AC2.5 가 요구하는 대로 행으로 실린다 — 의존성 화면과 역방향 질의가 자동 추출분과 같은 표에서 읽는다」 2행 | ② doc-tracker 6b 행 축자 · ③ PR 본문 · ① 바로 아래 두 `INSERT` |
| 같은 자리 「요청 행을 함께 두는 이유는 그 화면이 「분석 전」과 「분석했더니 비었음」을 요청 행의 유무로 가르기 때문이다」 2행 | ① `analysis.rs` `traced_dependencies` doc 「묻지 않은 것과 물었더니 없던 것은 다르다(0008 이 요청과 데이터를 두 테이블로 나눈 것과 같은 구분)」(7차 패스가 정본으로 유지) · ① `FeatureDependencies.tsx` 의 `deps.status === null` 분기 · 0008 의 두 테이블 분리 |
| `overlay` doc 「저장은 건드리지 않는다 — 문서를 내보내는 자리에서만 겹친다」 (5행 → 4행, 재작성) | ① `doc_edit::overlay` doc 의 같은 문장(앞 패스가 정본으로 유지) · 0010 「확정된 추가는 문서를 내보내는 시점에 그 끝에 겹쳐 읽는다」. 뒤따르는 「편집 겹치기보다 **먼저** 실행해야 한다: 그래야 …」 2행은 **유지** — 이 순서 계약의 정본이며 두 호출부(`analysis.rs`·`doc_edit.rs`)의 사본은 이쪽을 가리켜 걷었다 |
| `feature_json` 「자동 문서의 feature 가 갖는 칸을 그대로 갖되 위치는 첫 근거이고(근거 없음이면 없다)」 (3행 → 2행, 재작성) | ① 본문 `"location": draft.scenarios.first()…` · `json!` 의 칸 목록. 뒤따르는 「모순 목록은 비어 있다 — 두 패스가 다툰 것이 아니라 사람이 확정한 한 판이다」는 **유지**(`"contradictions": []` 의 이유는 여기에만 있다) |
| `tree_of` 본문 「설치 토큰은 이 호출 동안만 산다 — 워커 claim 과 같은 규약(AC4.1/AC4.3)」 | 꼬리표만 걷음(재작성, 지문 ±0) — 자격증명 비노출 불변식은 유지(`draft()` 의 「이 사용자의 키로 부른다」와 같은 유형) |
| `grounded` doc 「답에서 이 트리에 근거를 둔 것만 꺼낸다 … 시나리오는 근거 경로가 트리에 있어야 남는다. 의존성 후보는 근거가 없어도 남되(`JRN-review-feature` 의 예외 표대로 …) 트리 밖의 경로를 근거로 들면 … 버리고, 분류는 7종 안이어야 한다. `found` 는 모델의 주장이 아니라 남은 시나리오의 유무로 다시 판정한다」 6행 | ① 본문이 문장마다 그 자체다 — `known(evidence)` 필터 · `None => None` / `Some(_) => return None` 가지 · `is_category` · `if scenarios.is_empty() { return Draft::default() }` 와 `draft()` 의 `found = !draft.scenarios.is_empty()` · ② 여정 예외 표 · ② doc-tracker 6b 행. `doc_edit.rs` `proposed()` 와 같은 판정 |
| `stub_matches` doc 「세 글자 이상의 영문·숫자 낱말이 경로에 (대소문자 무시하고) 들어 있으면 그 경로가 근거다 … (테스트 문서 03#시나리오 4)」 (6행 → 4행, 재작성) | ① 본문 `w.len() >= 3` · `to_ascii_lowercase` · ② 테스트 문서 꼬리표. 「실제 모델은 문장 전체를 읽고 스스로 고르지만, 스텁은 결정적이어야 하므로 관측 가능한 규칙 하나로 대신한다 … 「근거 없음」 갈래도 스텁에서 도달된다」는 **유지**(충실도 경계 — `stub_edit` 의 회피 규칙과 같은 판정) |
| 단위 테스트 doc 「요청이 트리의 경로를 가리키면 근거가 그 경로다 — 지어낸 경로가 아니다」 · 「트리에 없는 것을 부탁하면 빈 답이다 — 「근거 없음」이 스텁에서도 도달된다」 · 「모델이 트리 밖의 경로를 근거로 들면 그 시나리오는 버려진다 — 규칙은 코드에 있다」 3행 | ① fn 이름 `the_stub_cites_only_paths_the_request_names` · `a_request_the_tree_does_not_know_finds_no_evidence` · `an_invented_evidence_path_is_dropped` 와 단정 |

### 유지 28행 — 사유

`status` 「0010 의 CHECK 와 같은 값이어야 한다」(저장소 제약 — `doc_edit.rs` `status` 와 같은 판정) ·
`MAX_SCENARIOS`/`MAX_DEPENDENCIES` 「한 문장으로 더한 feature 가 자동 문서보다 길어지는 일은 「초안」이
아니다」 2행(상한의 이유 — `MAX_PROPOSED` 선례) · `routes()` 「초안도 주소를 가진다 …」 2행(정본 —
`addition()` 의 사본은 이쪽을 가리켜 걷었다) · `DraftScenario`·`DraftDependency`·`Draft` 요약 각 1행(pub) ·
`draft()` 「이 사용자의 키로 부른다 — … 평문은 이 호출 동안만 산다」(자격증명 비노출 불변식이 **이
자리에서** 지켜지는 이유 — `propose()` 와 같은 판정) · `decide()` 「결정과 의존성 적재는 한 트랜잭션이다 —
확정된 feature 의 의존성이 반만 실린 채 읽히는 순간이 없어야 한다」 2행(트랜잭션 계약 — 1·7차 패스
선례) · `overlay` 요약 1 + 빈 1 + 순서 계약 2 · `confirmed_name` 요약 1행(pub) · `feature_json` 2행(재작성) ·
`tree_of` 본문 「설치 토큰은 이 호출 동안만 산다」 1행(재작성) · `stub_matches` 4행(재작성) · `stub_draft`
「근거 경로에서 파생하지 고정 문자열이 아니다 — 배선이 끊겨도 통과하는 상수는 … 숨긴다」 2행(2~5단계의
`stub_*` 가 각자 들고 있는 충실도 경계 — 6·8차 패스가 단계마다 유지) · 단위 테스트
`a_claim_without_grounded_scenarios_is_not_found` 의 「`found=true` 라고 주장해도 남는 시나리오가 없으면
근거 없음이다 — 의존성만으로 feature 를 세우지 않는다」 2행(뒷문장의 이유가 이 자리에만 있다 —
`an_empty_answer_is_not_a_proposal` 선례) · `// mock-exception: LLM-01` 은 기계 판독이라 지문·판정 밖.

**판단이 갈려 남긴 것**: 위 테스트 doc 2행 — 「의존성만으로 feature 를 세우지 않는다」는 `grounded` 의
`if scenarios.is_empty() { return Draft::default() }` 에서 따라오지만 그것이 *의도*라는 서술은 어디에도
없다(1건).

## `backend/tests/feature_add.rs` — 제거 15행

- 모듈 머리 10행 → 요약 2행 「자동 추출이 놓친 feature 의 직접 추가(AC3.2)와 그 출처 보존(AC3.4) —
  라우터를 그대로 돌려 본다」(한 문장이 두 줄). 제거 「인수 문서는 워커의 `/internal` 경로로
  넣는다(형제 테스트와 같은 방침) — …」 2행 — `backend/tests/dependencies.rs:3-4` 가 정본(문서 편집 축과
  같은 판정). 제거 「브라우저에서 관측되는 흐름은 `e2e/tests/sc03-03`·`sc03-04` 가 지킨다. 이 파일이
  지키는 것은 그 아래의 규칙이다 — 근거는 트리 안 경로뿐 · … · 남의 분석에는 닿지 않음」 4행 — ① 여섯
  테스트의 fn 이름이 그 목록이다 · ② doc-tracker e2e 매핑 표. 빈 `//!` 2행이 딸려 나갔다.
- `analysis()` 「큐에 든 분석 하나 — 5단계가 아직 아무것도 쓰지 않은 상태」 — ① fn 이름 · 본문(`POST
  /api/analyses` → `CREATED` 뿐). `analysis_with_document()` 「인수 문서가 서 있는 분석 하나」 — ① fn 이름.
  인라인 「확정 전에는 아무것도 바뀌지 않는다」 — ① 바로 아래 단정 + 「결정 전에는 목록에 세지 않는다」
  메시지. 「확정한 의존성 후보는 의존성 화면이 읽는 행으로 실렸다」 — ① 바로 아래 `GET …/dependencies`
  단정. `registration_does_not_need_the_automatic_document` 의 doc — ① fn 이름 · 단정.
  `an_added_feature_can_be_edited_like_any_other` 「더해진 feature 는 편집(AC3.1)의 대상이기도 하다 —
  추가 겹침이 편집 겹침보다 먼저다」 — ① fn 이름 · `feature_add::overlay` doc 의 순서 계약(정본) · ②
  doc-tracker. `another_users_analysis_cannot_be_added_to` 「AC4.7 — 남의 분석은 없는 것과 같다 …」 — ① fn
  이름 · `NOT_FOUND` 단정 · `analysis.rs` `owned_analysis` doc(「404 not 403 … AC4.7」 정본) — 문서 편집 축의
  같은 줄과 같은 판정.
- **유지**: 픽스처 상수 `FOUND`·`UNKNOWN` 의 doc 2행(「스텁 트리의 두 경로(`src/api/routes.rs` ·
  `src/middleware/auth.rs`)에 든 낱말을 품은 문장」 · 「스텁 트리의 어느 경로에도 없는 문장」) — 문장이
  *왜 그 문장인지*는 `repo_scan` 스텁 트리의 경로 목록과 `stub_matches` 규칙을 함께 읽어야 복원되므로
  「애매하면 남긴다」. 두 spec 의 같은 모양(`REQUEST`·`UNKNOWN`)도 같은 판정 — **판단이 갈려 남긴 것**(2건째,
  4파일에 걸친 한 판정).

## 두 spec — 제거 33행

- **Isolation 4행 → 「Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.」 1행(`sc03-03`,
  −3).** 13차 패스가 정한 규약이고 문서 편집 축이 `sc03-01`·`sc03-02` 에 적용한 그대로다 — 새로 들어온
  파일에는 최신 규약. **이미 판정된 spec 의 Isolation 블록 12개는 이 패스 범위 밖**이며 후속 후보로 그대로
  남는다(아래 「범위 밖」). `sc03-04` 는 워커를 임대하지 않으므로 이 한 줄도 없다 — 임대하지 않는 다른
  spec(`sc04-03`·`sc04-13` 등)과 같은 모양이다.
- **머리 문단은 두 spec 모두 걷었다** — 문서 편집 축이 `sc03-01`·`sc03-02` 의 머리 문단을 「테스트 **설계**의
  이유(3탭을 세므로 API 로 질러가지 않는다 · 같은 요청을 두 번 보낸다)」로 유지한 것과 갈리는 것처럼
  보이지만, 여기 두 문단은 설계의 이유가 아니라 **무엇을 관측하는가**다: `sc03-03` 「「근거와 함께
  제시된다」를 관측한다 — 초안의 모든 근거가 이 분석의 트리 안 경로이고 그 경로가 사람이 문장으로
  가리킨 곳이라는 것. 스텁도 같은 규칙이다(… `stub_draft` 는 요청에 든 낱말이 들어 있는 경로만 근거로
  든다). 확정 전에는 문서가 그대로이고, 확정하면 새 feature 가 인수 문서 끝에 선다」 4행 — ② doc-tracker
  e2e 매핑 행(`03#시나리오 3`)의 「검증 내용」 칸(「초안의 **모든 근거가 이 분석의 트리 안 경로**이고
  문장이 가리킨 곳이다 … **확정 전 문서 불변** → 확정 → 인수 문서 끝에 새 feature」)과 「관측 대상이 아닌
  것」 칸(「stub 은 요청에 든 낱말이 들어 있는 경로만 근거로 드는 결정적 규칙」)이 문장 단위로 같다.
  `sc03-04` 「「강제로 만들어내지 않는다」를 관측한다 — 트리에 없는 것을 부탁하면 초안이 비어 오고 화면은
  「근거 없음」을 그대로 보인다. 사람은 그 상태로 추가하거나(출처는 「직접」) 취소할 수 있고, 어느 쪽이든
  자동 문서는 그대로다」 3행 + 「워커를 임대하지 않는다 — 근거 찾기는 이 분석의 트리와 모델을 API
  프로세스가 직접 부르므로 파이프라인이 한 단계도 돌지 않은 분석에서도 성립한다(문서 부재 404 의 뜻 …
  함께 본다). Like every spec it signs in as its own stub user」 4행 — ② 같은 표의 `03#시나리오 4` 행
  (「파이프라인이 한 단계도 돌지 않은 분석에서(워커 임대 없음) … 자동 문서 부재 404 의 뜻(「아직 생성
  전」)이 그대로」) · ② 6b 변경 이력 행(「API 프로세스가 직접 한 번 부르며」) · ① `signInWithCredentials`
  호출. 빈 `//` 3행이 딸려 나갔다.
- `sc03-03` 단정 옆 10행: 「남은 후보를 전부 결정한다 — 목업이 「결정 끝」 뒤에만 이 단계를 연다(셋업이지
  검증이 아니다)」 — ① reject 요청의 `reason` 리터럴 「이 spec 의 셋업 — 사용자 기능이 아님」 · ② 매핑 행
  「후보를 전부 결정한 뒤 「결정 끝」 CTA 로 진입」. 「확정될 목록 = 승인된 후보 1 + 직접 추가 0」 — ①
  `toHaveText('1')` · `finalCount` 의 정의. 「문장이 비어 있는 동안에는 근거를 찾을 수 없다」 — ① 바로 아래
  `toBeDisabled()`(`sc03-01` 의 같은 줄과 같은 판정). 「모든 근거가 이 분석의 트리 안 경로이고, 문장이
  가리킨 곳이다」 — ① 두 단정 메시지(「트리 밖의 경로를 근거로 들었다」 · 「문장이 가리키지 않은 곳을
  근거로 들었다」). 「의존성 후보도 7종 안의 분류와 트리 안 근거를 든다」 — ① 바로 아래 `CATEGORIES`
  `toContain` · `/^payments-api\//`. 「확정 전에는 아무것도 바뀌지 않는다」 — ① 단정 메시지. 「새 feature 가
  인수 문서 끝에 섰다 — 화면이 아니라 저장이 기준이다」 — ① 단정(`sc03-01` 선례). 「자동 문서의 feature 는
  지워지지 않는다」 — ① 단정 메시지 「자동 feature 가 사라졌다」. 「확정한 의존성 후보는 의존성 화면이
  읽는 행으로 실렸다」 — ① `dependenciesOf` 단정. 「시도 자체도 주소를 가진다 — 확정과 출처가 그 기록에
  남는다」 — ① `GET …/additions` 의 네 단정.
- `sc03-04` 단정 옆 6행: 「근거 없음이 명시되고, 초안은 하나도 그려지지 않는다」 — ① 바로 아래 네 단정.
  「서버도 같은 말을 한다 — 시도는 남되 초안은 비었다」 — ① `additionsOf` 단정 다섯. 「「근거 없음」으로
  추가 — 출처는 사람이 직접」 — ① `add-anyway` 클릭 · 단정 메시지 「AC3.4 가 요구하는 출처 — 직접」. 「취소
  경로 — 시도는 남지만 목록에 들지 않는다」 — ① `cancel-add` 클릭 · `['cancelled', 'confirmed']` 단정.
  「자동 문서의 뜻은 그대로다 — …」 — ① `toBeNull()` · 앞선 단정 메시지 「아직 자동 문서는 없다」. 「남의
  분석에는 닿지 않는다」 — ① 단정 메시지 「남의 분석은 존재조차 확인해 주지 않는다」.
- **유지**: `sc03-03` 의 「`.legend` 가 대문자로 그리므로(text-transform) 렌더 텍스트가 아니라 DOM 텍스트를
  읽는다」 — `allTextContents()` 를 고른 이유가 코드에 없고(`toHaveText` 였다면 실패한다), 8차 패스가
  Playwright 함정을 유지한 선례. 두 spec 의 픽스처 상수 doc 각 1행(위 「판단이 갈려 남긴 것」 2건째).

## `frontend/src/AddFeature.tsx` — 제거 2행

- 머리 5행 → 4행: 「한 문장을 받아 초안 하나를 만드는 화면. 근거를 찾았는지, 초안이 무엇인지, 확정될 목록이
  몇인지는 전부 서버가 준다 — 화면이 기억하는 것은 사람이 지금 치고 있는 문장과 방금 받은 초안뿐이다」까지
  남기고(`RequestEdit.tsx` 머리의 화면 불변식과 같은 모양 — 문서 편집 축이 유지) 뒤의 「근거를 못 찾으면
  초안은 비어 오고, 화면은 그것을 「근거 없음」으로 그대로 보인다(지어내지 않는다)」를 걷었다 — ① `api.ts`
  `draftAddition` JSDoc(유지) · ② doc-tracker · ② PRD AC3.2. 마지막 행을 「… 초안뿐이다.」로 잘라 재작성.
- 내부 fn `decide` 의 JSDoc 「확정이면 feature 가 된다 — 근거가 있었으면 도움받아, 없었으면 직접. 취소면
  시도로만 남는다」 — ① `api.ts` `decideAddition` JSDoc(export 요약, 유지) · 0010 `source` 문단.
- **유지**: 1행의 목업 매핑 `// docs/mockups/JRN-discover-features.html#STP-add-missing 의 구현.` — 7·8차
  패스가 찾은 **M1 의 입력**(`check-mockup-render.py::discover_screens()`), 지우면 게이트가 붉어진다.
  `dependencyLine` 의 「Concatenated for the reason `FeatureCandidates` gives: a template literal reads as
  one product string to the copy gate …」 2행 — 8차 패스가 M3B 추출기 함정으로 유지한 `quotedRejection` ·
  `locationOf` 와 4차 패스 뒤 남아 있는 `FeatureAcceptance.tsx` `evidenceOf` 의 같은 모양. 「보내기 전에
  화면이 막는 이유는 빈 문장으로 사람의 키를 쓰는 호출을 하지 않기 위해서다」 — 문서 편집 축이
  `RequestEdit.tsx` 의 같은 문장을 **판단 갈림**으로 남긴 그 사유 그대로(서버 `draft()` 도 빈 요청을
  `active_key_for_user` **전에** 거절한다) — 3건째, 승계.

## 증분 — 판정 완료 원장 행 안의 16행 (원장 1·2·5·8·10·17행)

각 원래 패스 파일에 절을 더했다: [2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md)
「증분 재판정 ⑦」(원장 1행 `analysis.rs` 2행 → 제거 2) · [2026-09-18-uncontested-harness-config.md](2026-09-18-uncontested-harness-config.md)
「증분 재판정 ③」(원장 2행 `config.rs` 3행 → 제거 3) · [2026-09-18-worker-double-axis.md](2026-09-18-worker-double-axis.md)
「증분 재판정 ③」(원장 5행 `deploy/e2e/kustomization.yaml` 2행 → 제거 2) ·
[2026-09-19-candidate-strategy-axis.md](2026-09-19-candidate-strategy-axis.md) 「증분 재판정 ③」(원장 8행
`FeatureCandidates.tsx` 1행 → 제거 1) · [2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md)
「증분 재판정 ④」(원장 10행 `api.ts` 7행 → 제거 5 · 유지 2) · [2026-09-21-doc-edit-axis.md](2026-09-21-doc-edit-axis.md)
「증분 재판정 ①」(원장 17행 `doc_edit.rs` 1행 → 제거 1). 1·2·5·8·17행은 줄 수·지문이 #107 이전 값으로
**바이트 동일하게 되돌아왔고**(590/`9757b6b5…` · 140/`ddacce0b…` · 190/`632b0475…` · 141/`a431e905…` ·
56/`b9e6778d…` — 부모 `24f488d` 재계산과 동일; #107 이 그 행들에 더한 것이 전건 제거된 주석뿐이기 때문),
10행은 `api.ts` 유지 2행만큼 늘어 83/`ec54e678…` → 90/`d88323c6…`(트리거) → **85 / `89959a07…`** 이다.

## 검증

- **판정기**: `python3 tools/check-data-format-change.py --base 269a5f2 --head <준비 브랜치 head> --verbose`
  → **`✅ 변경 없음`**(검사 파일 11/11 — D1·D2·D5·D6 어느 경로에도 안 걸린다). 감지가 새 파일 5개에 1줄
  probe 로 잰 것을 실제 후보 트리에서 다시 돌린 것이다(「슬라이스 전 필수 절차」).
- **주석 제거 후 부모와 바이트 동일 11/11** — 줄머리 `//`·`///`·`//!`·`#`·JSDoc 과 블록 주석을 걷어낸
  잔여의 md5 가 `269a5f2` 와 같다(`analysis.rs` `ea2b0ec7` · `config.rs` `223cc58e` · `doc_edit.rs` `c9a1e102` ·
  `feature_add.rs` `2cbc81e3` · `tests/feature_add.rs` `c0c41c79` · `kustomization.yaml` `e39333f3` · `sc03-03`
  `3e910a04` · `sc03-04` `de6f18ac` · `AddFeature.tsx` `18976352` · `FeatureCandidates.tsx` `11ac8ad5` · `api.ts`
  `2cceebc5`). diff 의 `+`/`−` 줄 중 주석 줄머리가 아닌 것은 0.
- 문서 게이트 4종 rc=0(`check-scenario-e2e` · `check-mockup-render` — `AddFeature` 매핑이 M1 발견 집합에
  그대로 · `check-journey-mockup` · `check-journey-prototype`) · `npm run build`(tsc + vite) rc=0. `cargo test` 와
  kind e2e 는 이 호스트에 Rust 가 없어 CI 가 집행자다 — Rust 쪽 변경은 `///`·`//!`·`//` 줄뿐이라 컴파일 표면을
  건드리지 않는다(stripper 동일).
- 전역 지문: 부모(= 트리거) `lines=2586 files=121` / `f80703d2…` → **`lines=2466 files=121` /
  `1f34b522…`** — 순 제거 **120**, 파일 수 불변(주석 0행이 된 파일 없음 — `sc03-04` 는 `UNKNOWN` doc 1행이
  남는다).
- 원장 산술(준비 시점 트리, base `269a5f2`): 행 열 합 2,255 + 10행 증분 +2 + 새 행 45 = **판정 완료 2,302** ·
  미판정 증분 **0** · 잔여 = 2,466 − 2,302 = **164** = 전역 파일 목록에서 원장 117파일을 집합으로 뺀 8파일
  (마이그레이션 `0009`·`0010` 60 · D2 4 / 45 · D6·D5 2 / 59)의 행수 합과 일치.

## 범위 밖 (후속)

- **`backend/migrations/0010_feature_additions.sql` 30행** — 사람 게이트(본문 「적용된 마이그레이션」). #107 이
  main 에 들어가 `pin deployment image` → `deploy` 브랜치로 롤아웃됐으므로 **적용된 마이그레이션**이다.
  `0009` 30행과 함께 **한 번의 repair** 로 모으는 것이 본문 1항(「한 패스에서 마이그레이션 판정을 끝까지
  모아」)에 맞다 — 그 패스를 여는 것은 사람의 결정이다. 이 패스가 0010 의 머리 문단(`feature_candidates` 에
  섞지 않는 이유 · `analysis_documents` 불변 · `key` 접두사 · `source` 어휘)을 코드 쪽 사본들의 **정본**으로
  삼았으므로, 0010 을 판정할 때 그 문단들은 유지 후보다.
- **이미 판정된 spec 12개의 Isolation 블록** — 13차·문서 편집 축이 적어 둔 후속 후보 그대로(원장 3·4·5·6·7·8행에
  걸치는 증분 재판정, 한 패스로). 이 패스는 새 파일 `sc03-03` 에만 최신 규약을 적용했고 12개는 손대지 않았다.
- 잔여 D2 4파일/45행 · D6·D5 2파일/59행 — 사람 게이트, 변동 없음.

## 증분 재판정 ① — `#108` 이 `feature_add.rs` 에 더한 1행 (2026-09-22 · `rct_20260922-0001`)

사람 PR **#108**(AC4.9 출력 언어 설정)이 `backend/src/feature_add.rs::draft` 에 1행을 들여왔다:
`// 초안은 이 분석의 목록에 들어가므로 목록의 나머지와 같은 언어로 쓴다.`
(바로 아래 `let language = crate::settings::analysis_language(&state.db, &id).await?;`)

**제거 1.** 「이 분석의 … 언어」는 호출하는 함수 이름 `analysis_language` 가 말하고(①), 「산출물이 들어가는
문서와 같은 언어로 쓴다」는 그 함수의 doc 이 **이 축의 정본**으로 적는다 — 「including the ones the API makes
directly (edit proposals, manual feature drafts), so their prose matches the document it lands in」. 같은 자리가
`doc_edit::propose` 에도 한 벌 있었고 같은 판정으로 걷었다(원장 17행). 판정 맥락은
[2026-09-22-output-language-axis.md](2026-09-22-output-language-axis.md).

지문: **46행 `3fb54093…` → 45행 `b596e7392db0110e03721163ee6946ca87d58b58aba894f6a556f9ec481294a7`** —
**이 패스 직전 원장 값으로 바이트 그대로 복귀**했다.
