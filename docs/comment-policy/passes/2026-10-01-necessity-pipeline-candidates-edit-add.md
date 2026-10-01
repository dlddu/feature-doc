# 2026-10-01 — 필요성 판정 두 번째 슬라이스 (파이프라인 · 후보·전략 · 문서 편집 · 직접 추가 · e2e 잔여)

- **reconciler task**: `tbm_feature-doc-comment-necessity/rct_20261001-0001`
- **기준**: [README.md](../README.md) 「필요성 시험」 — *이 주석을 지우면, 이 코드를 고치러 온 사람이 틀린
  판단을 하거나 그 판단에 필요한 사실을 확인하는 데 상당한 비용을 치르는가?* 사유를 한 문장으로 댈 수
  없으면 지운다.
- **범위 선택**: 판정 칸이 `—` 인 행 중, 열린 PR #210(Web Push 슬라이스 — `backend/src/analysis.rs`·`config.rs`·
  `lib.rs`·`worker_api.rs`·`backend/tests/{auth,diff,migrations,security}.rs`·`tests/common/mod.rs`·
  `frontend/src/{api.ts,HomeRepositories.tsx}`·`deploy/k8s/secret.yaml.example`·`backend/Cargo.toml` 등)과 파일이
  겹치지 않는 덩어리를 예산 400줄까지 채웠다. 마이그레이션 `.sql` 행은 전용 PR·사람 repair 몫이라 뺐고,
  `.github/workflows/` 행은 수동 승인 판정기 자신(`data-format-review.yml`)을 품어 뺐다.

## 판정 — 다섯 덩어리 400줄

| 덩어리 | 표면 | 판정 전 | 판정 뒤 | 제거 |
|---|---|---|---|---|
| 파이프라인 · 횡단 관심사 축 9파일(`backend/src/pipeline.rs` …) | L · E | 149 · 1 | 115 · 1 | 34 |
| 후보·전략 축 8파일(`backend/src/discovery_strategy.rs` …) | L · E | 142 · 1 | 126 · 1 | 16 |
| 문서 편집 축 6파일(`backend/src/doc_edit.rs` …) | L | 59 | 58 | 1 |
| 빠진 feature 직접 추가 축 5파일(`backend/src/feature_add.rs` …) | L | 45 | 45 | 0 |
| e2e 잔여 2파일(`e2e/support/github-app.ts` · `sc01-08`) | L | 3 | 3 | 0 |
| 합 | | **400** | **349** | **51** |

「제거」는 줄 수 순감이다. 그 밖에 경위만 걷은 제자리 재작성 1곳(`feature_candidates.rs` 머리 — 3줄로)이 있다.
비주석 diff 는 0줄이다.

### 제거 목록

| 자리 | 유형 |
|---|---|
| `backend/src/pipeline.rs` 모듈 머리 「Every analysis is seeded with one `analysis_stages` row per entry at enqueue.」 | 코드 재진술 — 시드는 `backend/src/analysis.rs` 의 `pipeline::STAGES.iter()` 루프가 말한다 |
| `backend/src/cross_cutting.rs` `SKIPPED_DIRS` · `ENTRY_DIR_HINTS` · `MAX_PATHS` doc 5행 | 코드 재진술 — 이름과 값 목록이 말한다 |
| `backend/tests/documents.rs` 테스트 doc 셋(`claim_offers_the_cross_cutting_stage` · `the_first_analysis_reports_no_comparison` · `a_different_repository_is_not_treated_as_a_rerun`) 6행 | 코드 재진술 — 테스트 이름과 단정 메시지 |
| 같은 파일 「Never claimed — nobody holds a lease on this job.」 | 코드 재진술 — 테스트 이름 `a_worker_without_the_lease_cannot_store_a_document` |
| `backend/tests/progress.rs` `claim` 헬퍼 doc | 중복 — 모듈 머리가 `/internal` 경로로만 상태를 만든다는 이유를 이미 말한다 |
| 같은 파일 `progress_is_persisted_…` doc 2행 · 「Reading again changes nothing …」 · 「The home list carries the same fraction …」 | 코드 재진술 — 테스트 이름·단정 메시지 |
| 같은 파일 `retry_is_refused_for_a_stage_that_has_not_finished` doc 2행 | 제품 문서 재진술(부분 재시도 규칙) — 테스트 이름이 규칙을 말한다 |
| `e2e/tests/sc01-02-…` 머리 「앞부분의 홈 목록 → pre-flight → `Queued` 는 … 시나리오 1 전용 spec 이 신설될 때 이어받는다」 3행 + 빈 주석 1행 | 틀린 주석(시나리오 1 전용 spec `sc01-01` 은 이미 있다) + doc-tracker 재진술 — 사유 없음 |
| 같은 파일 「Nothing was queued by the refused attempt.」 | 코드 재진술 — 바로 아래 단정 |
| `e2e/tests/sc01-03-…` 「404, not an empty document …」 2행 | 중복 — 같은 구분의 사유는 그 응답을 만드는 쪽(`backend/tests/documents.rs` `a_document_that_was_never_produced_is_not_found` doc)이 말한다 |
| `e2e/tests/sc02-03-…` 머리 「원문의 기대 결과 중 여기서 단정하지 않는 것」 3행 + 빈 주석 1행 | 저장소 문서 재진술 — 등재 자리가 `docs/doc-tracker.md` 「e2e 매핑」 자동화 밖 잔여 칸이라고 스스로 말한다 |
| 같은 파일 「그리고 그 상황은 본 시나리오에서 찾을 수 있어야 한다 …」 2행 | 시나리오 문서 재진술 — 단정이 무엇을 보는지는 단정 자신이 말한다 |
| `backend/tests/strategy.rs` `propose` doc · 「Blank and duplicate entries …」 · 「Asserted where the gate is enforced …」 · 「A different repository does not inherit it …」 | 코드 재진술 — 409 실패가 즉시 드러내는 사실, 바로 아래 dedup·단정 |
| `backend/tests/candidates.rs` `run_through_stage_three` doc 2행 · `run_stage_four` doc | 코드 재진술 — 함수 이름과 모듈 머리 |
| 같은 파일 「Kept, not deleted …」 · 「The flag is information, never an automatic decision.」 · 「Parked: nothing would ever hand this job back to a worker.」 | 제품 문서 재진술 — 병합 보존·플래그의 성격·`awaiting_pipeline` 의 뜻은 PRD 와 `pipeline::status` doc 이 말한다 |
| `e2e/tests/sc01-07-…` `runToCandidates` · `cssEscape` JSDoc(비 export) | 코드 재진술 — 함수 이름 |
| 같은 파일 「Not a 404 — "아직 추출 전" …」 2행 | 중복 — 응답을 만드는 쪽(`backend/tests/candidates.rs` 「Not a 404 and not an error」)이 말한다 |
| 같은 파일 「화면만의 규칙이 아니다 — 서버도 같은 이유로 거절한다.」 | 코드 재진술 — 바로 아래 API 호출 |
| `backend/src/doc_edit.rs` `splice` doc(비 `pub`) | 코드 재진술 — 이름과 시그니처 |
| `backend/src/feature_candidates.rs` 머리 「The approved patterns arrive on the claim response, so this stage needs no second round-trip.」(제자리 재작성) | 코드 재진술 — `extract` 시그니처의 `patterns` 인자 |

### 유지 목록 (묶음마다 필요 사유 한 문장)

**파이프라인 · 횡단 관심사 축** (L 115 · E 1)
- `pipeline.rs` 모듈 머리 요약과 `ACCEPTANCE_DEPENDENCIES` 키 문단 — 키가 이름과 달리 한 쪽만 돈다는 것을 보고
  개명하면 `/internal/.../stages/{key}`·`analysis_documents.kind` 와이어 계약이 깨진다.
- `Stage`·`FETCH`·`status`·`stage_status` 의 `pub` 요약 — doc 주석 수준(정책)이 요구한다.
- `FEATURE_CANDIDATES`·`ACCEPTANCE_DEPENDENCIES` 둘째 줄 — 승인 게이트를 워커 규칙으로 옮기면 게이트를 잊는
  워커 하나가 승인 전 단계를 돌린다(게이트는 큐의 속성).
- `AWAITING_PIPELINE` 본문 — 이 상태를 `succeeded` 로 바꾸는 「단순화」가 돌지 않은 단계를 완료로 보고한다.
- `cross_cutting.rs` 모듈 머리 — 허용 목록·바이트 상한을 넓히면 트리의 나머지가 LLM 경계를 넘는다.
- `AXES` doc — 순서를 바꾸면 화면 축 순서가 바뀐다(PRD 순서 고정).
- `AXIS_GUIDE` doc — 안내에 구체 경로를 넣으면 트리에 없는 경로가 근거로 인용된다.
- `KEY_FILE_NAMES` doc — 패턴으로 바꾸면 자격증명 파일이 프롬프트에 실린다.
- `MAX_KEY_FILE_DEPTH`·`directory_of` doc — 상한·두 단계의 수치 근거라, 바꾸면 하위 패키지 매니페스트를
  고르거나 모노레포 큐가 잘게 쪼개진다.
- `schema` doc · 테스트 `735` 행 doc — 스키마 안에 스텁 답을 넣으면 OpenAI `strict` 가 알 수 없는 키워드로
  거부한다(상류 거부 조건).
- `stub_answer` doc · 위치 선택 인라인 — 스텁을 상수로 바꾸면 배선이 끊겨도 근거 단정이 통과한다.
- `key_files`·`input_paths`·`keep_listed_evidence`·`detail` doc — `pub` 요약이고, `key_files` 는 프롬프트에
  없는 경로를 발췌하면 인용 불가 근거가 생긴다는 불변식이다.
- E `"src/main.rs"` 줄 끝 「duplicate — must be collapsed」 — 픽스처의 중복을 실수로 보고 지우면 dedup 단정이 공허해진다.
- `backend/tests/documents.rs`·`progress.rs` 모듈 머리 — 픽스처를 행 직접 삽입으로 바꾸면 워커가 실제로 내는
  모양과 어긋나도 테스트가 통과한다.
- `documents.rs` 「No key registered …」 — `null` 단정을 버그로 보고 키를 지어내게 고치는 판단을 막는다.
- `documents.rs` `a_document_that_was_never_produced_is_not_found` doc · `progress.rs` 404 doc — 200 빈 문서·403 으로
  바꾸면 화면이 「안 돌았다」와 「돌았는데 비었다」를 못 가르고, 남의 id 존재를 확인해 준다.
- `progress.rs` 임대 중 재시도 doc · 「Deliberately no `finish`」 — 테스트를 「고쳐」 `finish` 를 넣으면 검증 대상인
  임대 중 상태가 사라진다(동시성 계약).
- `frontend/src/CrossCuttingConcerns.tsx` 머리 1행 — `tools/check-mockup-render.py` 가 읽는 목업 매핑 선언이다.
- 같은 파일 머리 본문 · `AXIS_LABELS` · 별도 읽기 · 빈 대기 · 빠진 축 제목 — 각각 클라이언트 파생값 도입,
  축 순서 변경, 읽기 합치기(실패가 페이지를 막음), 목업에 없는 카피 추가(M3A), 빠진 축을 조용히 생략하는
  변경이 틀리는 이유다.
- e2e 다섯 spec 의 머리(`Runs against the e2e deployment …` · per-user 로그인 · `Leases the analysis worker …`)와
  「The overlay already rests at 0 …」 — spec 들이 공유하는 규약 블록(임대·격리·스텁 저장소)이다.
- `sc01-02` 「같은 버튼을 다시 누른다 …」 · `sc01-05` 「설치가 이미 서 있으면 …」 — 화면 전이가 상태 머신이라
  같은 버튼이 두 뜻을 갖는다는 것을 모르면 클릭을 「중복」으로 지운다.
- `sc01-03`·`sc02-03` 머리 판정 기준 문단 · `AXES` JSDoc · `isTestPath` JSDoc — 기대값 상수화·사본 불일치·
  주장값 판정으로 바꾸면 근거 없는 산출물에도 초록이 된다.
- `sc01-05` 머리 · `766 files` 인라인 — 새로고침 단정을 지우거나 숫자를 픽스처로 오해해 고치는 판단을 막는다.
- `sc02-03` 「stub 더블은 … exactly once」 — 개수 단정이 스텁 고정값에 기대는 충실도 경계다.

**후보·전략 축** (L 126 · E 1)
- `discovery_strategy.rs`·`feature_candidates.rs` 모듈 머리 — 경로 뷰를 단계마다 따로 만들면 단계끼리 저장소
  내용에 대해 어긋난다.
- 두 `stub_answer` doc — 상수 스텁은 배선이 끊겨도 통과한다.
- 상한 인라인 둘(`at most N`) — 지시문을 믿고 상한을 지우면 모델이 검토 불가한 목록·지어낸 위치를 넘긴다.
- `candidate_key` doc — 행 id·이름으로 키를 바꾸면 재분석·개명 뒤 이월이 끊긴다.
- `matching` doc — 와일드카드 없는 패턴이 접두사라는 규칙을 모르고 정확 일치로 고치면 손으로 더한 진입점이
  디렉터리에 닿지 않는다.
- 두 `detail` doc — `pub` 요약.
- `strategy.rs`·`candidates.rs` 모듈 머리 — 위 파이프라인 축 테스트 머리와 같은 사유.
- `strategy.rs` `requeue` doc · 「The claim above left the job `running` …」 — 재큐잉을 지우면 승인 차이가 아니라
  임대가 다음 job 을 가린다.
- 404 인라인·doc 넷(`strategy.rs` 두 곳 · `candidates.rs` 두 곳, 「Not a 404 and not an error」 포함) — 위와 같은
  상태 구분·존재 비노출 사유.
- E 「materialise the draft」 — 결과를 버리는 호출을 죽은 코드로 보고 지우면 전략 행이 생기지 않는다.
- 빈 전략 거부 · 승인 뒤 동결 · 이월 문장 강제 doc — 규칙을 느슨하게 하거나 「중복 테스트」로 지우는 판단을 막는다.
- `candidates.rs` 첫 읽기 실체화 · `created_at` 초 단위 동률 · `approve_strategy` 와 `finish` 의 분담 · 임대 중 승인 —
  lazy seed·정렬 절·임대 불변식을 모르면 헬퍼·픽스처를 「단순화」해 다른 경로를 검증하게 된다.
- `DiscoveryStrategy.tsx`·`FeatureCandidates.tsx` 머리 1~2행 — 목업 매핑 선언(`check-mockup-render.py`).
- 두 화면의 「Nothing here is client-side state」 · 폴링 이유 · 별도 읽기 — 클라이언트 상태·한 번 읽기·읽기
  합치기로 바꾸면 새로고침·대기·실패 격리가 깨진다.
- `mutate` 인자 · `quotedRejection` · `locationOf` JSDoc — M3B 카피 추출기가 썽크·템플릿 리터럴을 제품 카피로
  읽는다는 게이트 함정이다.
- 두 `Appbar` JSDoc — 빈 `icon-btn` 을 쓸모없는 요소로 지우면 제목 정렬이 깨진다.
- `sc01-04`·`sc01-07` 머리 판정 기준 · `/internal` 소관 포인터 · Isolation 블록 · 「Like every spec …」 · 오버레이 0 —
  공유 규약 블록이고, 소관 포인터는 브라우저에서 못 보는 성질을 어디서 지키는지 알린다.
- `sc01-04` 정확 일치 · 생성 항목 재등장 · `sc01-07` 승인 전 4단계 닫힘 · 읽기의 실체화 — strict mode·접두사
  공허·정상 동작을 회귀로 오진하는 판단을 막는다.

**문서 편집 축** (L 58)
- `doc_edit.rs` 모듈 머리 · `pub` 요약(`status` · `Sentences` · `overlay`) — doc 주석 수준. `status` 는 `0009` 의 CHECK
  와 같아야 한다는 동기화 제약, `Sentences` 는 저장·전송·프롬프트가 한 모양을 쓴다는 변경 파급, `overlay` 본문은
  저장 불변·마지막 승인 우선이라는 의미론이다.
- `MAX_AVOIDED`·`MAX_PROPOSED` doc — 상한과 최근 우선의 근거라, 바꾸면 회피가 오래된 방향부터 담기거나 한 줄
  요청이 문서 재작성이 된다.
- 라우트의 「제안도 주소를 가진다」 · 키 사용 규약(평문은 호출 동안만) — 화면 상태로 옮기면 새로고침이 제안을
  잃고, 키를 오래 들고 있으면 자격증명 비노출 불변식이 깨진다.
- 출처·근거 승계 · 회피 단위 · `base_document` · 「사람이 지금 보는 문장 기준」 · `lines` diff 단위 — 편집이 출처를
  바꾸거나, 겹쳐 읽기 순서·견줄 단위를 바꾸는 「단순화」가 틀리는 이유다.
- `ADD_HINT` · `stub_edit` doc · 빈 답 테스트 doc — 스텁과 실모델이 갈리는 지점과, 스텁이 회피 규칙을 따르지
  않으면 시나리오 2 가 우연으로 통과한다는 충실도 경계다.
- `backend/tests/doc_edit.rs` 모듈 머리 — doc 주석 수준.
- `sc03-01` 머리 · `── 탭 N ──` 셋 · 탭 계수 밖 펼침 — 「3탭 이내」를 세는 것이 이 spec 의 일이라, 흐름을 API 로
  질러가거나 클릭을 더하면 그 단정이 사라진다.
- `sc03-02` 머리 — 회피를 모델의 선의로 보고 단정을 약하게 바꾸는 판단을 막는다(스텁도 같은 규칙).
- `DecideDiff.tsx`·`RequestEdit.tsx` 머리 — 목업 매핑 선언과, 승인 전 문서 불변·서버가 대상과 거부 이력을 준다는
  불변식. `RequestEdit` 전송 차단 사유 — 서버 거절 때문이라고 오해해 지우면 빈 요청이 사람의 키를 쓴다.
- 두 spec 의 `Leases the analysis worker …` — 공유 규약.

**빠진 feature 직접 추가 축** (L 45)
- `feature_add.rs` 모듈 머리 · `status`(`0010` CHECK 동기화) · `MAX_SCENARIOS` · 초안 주소 · `DraftScenario`·
  `DraftDependency`·`Draft` · 키 사용 규약 — 문서 편집 축의 같은 자리와 같은 사유(각 파일의 독자가 읽는 자리).
- 결정·의존성 한 트랜잭션 — 둘을 나누면 의존성이 반만 실린 확정 feature 가 읽힌다.
- `overlay` doc — 편집 겹치기보다 **먼저** 돌아야 더해진 feature 의 시나리오도 편집 대상이 된다(순서 제약).
- `confirmed_name` · `feature_json` doc — 의존성 화면이 feature 를 인정하는 두 번째 경로와, 모순 목록이 빈 이유.
- 설치 토큰 수명 — 워커 claim 과 같은 비노출 규약.
- `stub_matches` · `stub_draft` doc · `found=true` 테스트 doc — 스텁의 「코드 검색」이 실모델과 갈리는 지점과,
  근거 없는 갈래가 스텁에서도 도달된다는 충실도 경계.
- `backend/tests/feature_add.rs` 머리 · `FOUND`·`UNKNOWN` doc · `sc03-03` `REQUEST` · `sc03-04` `UNKNOWN` — 문장이
  스텁 트리의 어느 경로 낱말에 걸리는지를 알려, 문장을 바꿀 때 근거 갈래가 바뀐다는 것을 알게 한다.
- `sc03-03` `.legend` 대문자 — 렌더 텍스트로 단정하면 `text-transform` 때문에 경로가 대문자로 읽힌다.
- `AddFeature.tsx` 머리 · `dependencyLine` · 전송 차단 — 목업 매핑 선언, 서버 상태 불변식, M3B 함정, 빈 문장으로
  키를 쓰지 않는 이유.

**e2e 잔여** (L 3)
- `e2e/support/github-app.ts` `installApp` JSDoc — export 함수 요약 1줄(정책).
- `sc01-08` 「A *new* attempt …」 — `startedAt` 존재만 보면 리셋 전 값으로 통과한다(리셋이 지우므로 「있고 다름」이어야 재실행).
