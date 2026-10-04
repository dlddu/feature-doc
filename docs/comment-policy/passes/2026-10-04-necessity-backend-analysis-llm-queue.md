# 2026-10-04 — 필요성 판정 (backend 집중 4파일: 분석 작업 · LLM 호출 경계 · 워커 큐 프로토콜 · LLM 키)

- **reconciler task**: `tbm_feature-doc-comment-necessity/rct_20261004-0003`
- **기준**: [README.md](../README.md) 「필요성 시험」 — *이 주석을 지우면, 이 코드를 고치러 온 사람이 틀린
  판단을 하거나 그 판단에 필요한 사실을 확인하는 데 상당한 비용을 치르는가?* 사유를 한 문장으로 댈 수
  없으면 지운다.
- **범위 선택**: 판정 칸이 `—` 인 행은 L 7 · E 1 이었다. 마이그레이션 `.sql` 5행과 `backend/src/db.rs` 행은 열린
  PR #223(`rct_20261001-0004`, 마이그레이션 repair 창)의 몫이라 뺐다. 남은 것은 backend 집중 4파일 덩어리 하나 —
  L 628 + 같은 파일 `analysis.rs` 의 E 1행 3줄 = 631줄이다. 덩어리 하나가 예산 400을 넘으면 그 덩어리만 가져간다는
  정의(「판정 슬라이스」)에 따라 이 슬라이스는 그 덩어리만 가져간다. #223 과 소스 교집합은 0이다.

## 판정 — L 1행 628줄 · E 1행 3줄

| 파일 | 표면 | 판정 전 | 판정 뒤 | 제거 |
|---|---|---|---|---|
| `backend/src/analysis.rs` | L | 252 | 213 | 39 |
| `backend/src/llm.rs` | L | 161 | 142 | 19 |
| `backend/src/worker_api.rs` | L | 171 | 169 | 2 |
| `backend/src/llmkey.rs` | L | 44 | 40 | 4 |
| L 합 | | **628** | **564** | **64** |
| `backend/src/analysis.rs` | E | 3 | 3 | 0 |

비주석 diff 는 0줄이다. `worker_api.rs` 의 순 −2 는 제거 3줄과 아래 「틀린 주석」의 재작성 +1 의 합이다.

### 제거 목록

| 자리 | 유형 |
|---|---|
| `analysis.rs` 모듈 머리의 둘째 문단부터 끝까지(30줄 — 「The enqueue half」 · 「Draining the queue」 · 「The progress half」 · 「And the documents」) | 문서 재진술 — 라우트 목록과 AC1.1 · AC1.3 · AC1.5 · AC1.2 조항을 되풀이한다. 그 안의 불변식은 저마다 지켜지는 자리에 이미 적혀 있다: 진행률이 서버 상태라는 것은 `AnalysisDetailView` doc, 승인이 다음 단계를 연다는 것은 `approve_strategy` doc, 해시 비교는 `worker_api::content_hash` doc, 단계 행과 작업이 한 트랜잭션이라는 것은 `create` 의 주석과 `worker_api.rs` 머리. 요약 두 줄은 doc 주석 수준이라 남겼다 |
| `analysis.rs` `stages_total` doc 의 「The stages themselves belong to Analysis Progress — see [`detail`].」 | 링크만을 위한 교차 참조(정책 doc 주석 수준) — 재작성 1줄 |
| `analysis.rs` `StageView::error` 「Why the stage failed, when it did. Retryable on its own (AC1.5).」 | 코드 재진술 · AC 재진술 |
| `analysis.rs` `ReproducibilityView::compared_to` 「The analysis this was compared against, when there was one.」 | 코드 재진술 — 필드 이름과 `Option` 이 말하고, 바로 위 enum doc 이 `first` 의 의미를 말한다 |
| `analysis.rs` `usage::by_stage` 호출 앞 「One read for the whole pipeline, not one per stage …」 | 코드 재진술 — 호출 한 번과 바로 아래 기본값 0 이 그 문장이다 |
| `analysis.rs` 전략 절 머리의 「(migration 0006) and is what AC1.3 calls "승인된 전략"」 | 이력 · AC 재진술 — 재작성 1줄 |
| `analysis.rs` 후보 절 머리의 「(migration 0007). AC1.4's 검증 방법 names four actions … which is what [`previous_rejection`] reads back.」 | AC 조항 재진술 · 링크 교차 참조 — 6줄을 3줄로 재작성. 거부 사유를 다음 분석에 싣는 이유는 `previously_rejected` 필드 doc 이 말한다 |
| `llm.rs` 모듈 머리 「Stage 1 (`fetch`) needed no model; every stage after it does.」 | 작업 흔적(경위) — 같은 문단을 2줄로 재작성 |
| `llm.rs` 모듈 머리 「It mirrors the `Mode` split …」 문단(빈 행 포함 5줄) | 코드 재진술 — `Mode` 분기와 stub 의 결정성은 `stub_answer` doc 과 `ask` 의 분기가 말한다 |
| `llm.rs` `ANTHROPIC_MODEL` doc 2줄 | 코드 재진술 — 상수 이름과 「상수로 한 곳에 둔다」는 형태 자체가 그 문장이다 |
| `llm.rs` `output_config` 앞 「`effort` bounds … Neither is a sampling parameter.」 | 같은 파일 재진술 — `EFFORT` · `MAX_TOKENS` doc 과 모듈 머리 「No sampling parameters」가 같은 것을 말한다 |
| `llm.rs` `stub_answer` doc 의 「before this trigger stub mode could not reach … and sc01-06 had to enter it through the tree 404 instead」 | 작업 흔적(경위) — 트리거가 왜 있는지는 남기고 경위만 걷었다(4줄 → 3줄 재작성) |
| `llm.rs` 실패 트리거 테스트 doc 의 앞 4문장 | 같은 파일 재진술 — `stub_answer` doc 의 트리거 의미를 되풀이한다. 프로세스 전역 env 경합 경고만 남겼다(7줄 → 3줄 재작성) |
| `llm.rs` 미지원 제공자 테스트 doc 의 「`llmkey::register` now refuses these at the door …」 | 같은 파일 재진술 — `ask` 의 미지원 분기 주석이 같은 사실을 그 자리에서 말한다(4줄 → 1줄 재작성) |
| `worker_api.rs` 모듈 머리 「AC4.5's separation (…) and horizontal scaling both hold, without a second datastore.」 | AC 조항 재진술 — 재작성 1줄 |
| `worker_api.rs` `executable_stages` doc 의 「The rule is spelled out in [`offered_stages`].」 | 링크만을 위한 교차 참조 — 재작성 1줄 |
| `llmkey.rs` `Provider::llm` doc 요약 「The call-side view of this provider.」와 빈 doc 행 | 코드 재진술 — 비공개 함수이고 시그니처(`-> crate::llm::Provider`)가 그 문장이다. 두 enum 을 가르는 이유 문단은 남겼다 |
| `llmkey.rs` `ACTIVE_KEY_SQL` 「Users with a single active key are unaffected.」 | 코드 재진술 — 정렬 규칙에서 바로 따라 나온다 |
| `llmkey.rs` `mask` 「Never reveals any secret portion of the key.」 | 같은 doc 재진술 — 바로 앞 문장 「reveal just the provider's public prefix」가 그 문장이다 |

### 틀린 주석 — 고침

| 자리 | 무엇이 틀렸나 | 고친 것 |
|---|---|---|
| `worker_api.rs` `stop_for_revoked_access` 위의 doc 블록 | 클레임 doc(「Atomically takes the oldest claimable job …」 15줄)이 `stop_for_revoked_access` 에 붙어 있었다. #158 이 그 함수를 `claim` 의 doc 과 `claim` 사이에 끼워 넣었고, #162 가 그 함수 자신의 doc 을 2줄로 줄이며 그대로 이어 붙였다. rustdoc 과 에디터는 클레임의 원자성 · 임대 만료 회수 설명을 접근 해제 함수의 설명으로 보여 준다 | 클레임 doc 15줄을 `async fn claim` 위로 옮기고(문면 무변경), `stop_for_revoked_access` 에는 홀로 서는 요약을 달았다 — 「Closes a job whose repository access was revoked (AC4.1). Call it only on a lease boundary, between the worker's calls — that, not anything here, is what lets the call already in flight finish before the job stops.」(2줄 → 3줄) |

### 유지 목록 (묶음마다 필요 사유 한 문장)

**분석 작업 — `backend/src/analysis.rs`** (L 213 · E 3)
- 모듈 머리 요약 2줄과 `pub` 항목 요약 — doc 주석 수준(정책).
- 후보 · 의존성 라우트의 「feature key 는 경로 구분자를 품어 본문·쿼리로 받는다」(두 곳) · 역방향 질의가 이 사용자의 분석 전부를 가로지른다는 것 — 지우면 경로 세그먼트로 옮기는 「정리」가 라우트를 깨고, 역방향 질의를 분석 하나에 매는 실수를 막을 것이 없다.
- `AnalysisDetailView` 의 진행률이 서버 상태뿐이라는 것 · `access_revoked` 가 저장된 사유에서 파생되는 이유 · `spend` 가 `est_*` 와 다른 실측이라는 것 · `analysis_columns` 가 조인 대신 서브쿼리인 이유와 `const` 가 아닌 이유 — 각각 두 값이 어긋나지 않게 하는 구조적 선택이다.
- 소유자 스코프 404(존재를 확인해 주지 않는다 — `detail` · `document` · `owned_analysis`) — 격리 불변식이 왜 그 자리에서 지켜져야 하는지다.
- `retry_stage` 가 큐 연산인 이유 · 임대 중 재큐잉 금지 · `create` 의 언어 복사와 작업·단계 행 한 트랜잭션 · `approve_strategy` 의 승인·재큐잉 한 트랜잭션과 `status <> running` 가드 · `requeue` 의 같은 불변식 · `decide_candidate` 의 같은 원리(거부는 재큐잉하지 않는다) · `request_dependencies` 의 한 트랜잭션 — 큐 · 임대 동시성 계약이다.
- 재현성 비교의 `(created_at, rowid)` 정렬(초 단위 타이로 진짜 선행을 놓친다 — 본문과 `carried_over` · `previous_rejection` · `analysis_diff` 의 참조) — 저장소 제약의 함정이다.
- `ReproducibilityView` 의 `first` / `unchanged` / `changed` 의미 · 와이어 형태 하이픈 · `preflight` · `create` 의 범위 밖 대상 처리와 「암묵 시작 없음」 · `has_access=false` 가 오류가 아니라는 것 · `list_repositories` 의 빈 목록 라우팅 · `ACCESS_REVOKED` 가 설치 id · 저장소 · 토큰을 담지 않는 이유 · `still_granted` 가 상류 실패를 해제로 읽지 않는 이유 — 각각 화면 · 운영자가 상태를 오독하게 만드는 함정이다.
- `Estimate` doc 과 E 3줄(`~3 KiB per source file` · `batched scans + fixed pipeline steps` · `~$0.006 per call`) — 세 상수의 근거는 코드 · 문서 · PR 어디에도 없는 복원 불가능한 지식이고 한 가족이다(옛 기준 판정도 3줄 유지).
- 전략 절: 제안은 문서에 · 사용자 편집은 별도 테이블(재현성 유지) · 목록 상한이 모델 상한의 두 배인 이유 · `source` 가 다음 분석으로 넘어가는 규칙 · 전체 PUT 이 diff 보다 나은 이유(중복 원소의 모호함) · 지연 시드의 멱등성 · `OR IGNORE` 경합 · 404 와 빈 전략의 구분 · 승인 뒤 편집 거부 · 빈 전략 승인 거부 · 사용자 항목만 이월 — 각각 검토 게이트의 의미를 지키는 규칙이다.
- 후보 절: 결정은 별도 테이블 · `merged_into` · `previously_rejected` 가 자동 결정이 아니라는 것 · `undecided` 가 서버 계수인 이유 · `extracted=false` 가 오류가 아니라는 것 · 병합 페이로드의 두 필드(`into` · `keys` — 이름만으로는 방향이 모호하다) · 재실행이 결정을 지우지 않는 `INSERT OR IGNORE` · 딥링크 지연 시드 · 이름 바꾸기가 키를 옮기지 않는 이유 · 병합 행을 지우지 않는 이유 · 병합된 후보의 결정을 비우는 이유 · 사유 없는 거부 거절 — 각각 다음 분석의 식별과 화면의 두 축을 지키는 규칙이다.
- 의존성 절: 의존성이 단계가 아니라 feature 하나의 요청이라는 것 · `evidence=None` 이 「근거 없음」이라는 사실 · `status=None` 과 「물었더니 없음」의 구분 · 빈 범주도 답이라는 것 · 확정된 feature 만 추적 · 마지막 실행까지의 결과 · 내보내기가 첨부인 이유 · `analysis_diff` 의 비교 대상 고정과 「사라진 feature 를 싣지 않는」 이유 · 빈 문서 · `None` 의 구분(0008 의 두 테이블) · 첫 분석의 `compared_to=None` — 각각 「없음」의 서로 다른 의미가 화면에서 섞이지 않게 하는 규칙이다.

**LLM 호출 경계 — `backend/src/llm.rs`** (L 142)
- 모듈 머리 — 이 모듈이 제공자와 말하는 유일한 자리라는 것 · prefill 이 현재 Claude 모델에서 400 이라 structured output 을 쓰는 이유 · `temperature` 가 400 이라 샘플링 파라미터를 보내지 않는 이유와 결정성을 해시로 관측하는 방식 · 키는 인자이지 필드가 아니고 오류 문면에 나오지 않는다는 것 — 상류 API 의 문서화되지 않은 거부 조건과 자격증명 비노출 불변식이다.
- `MAX_TOKENS`(추론과 응답이 예산을 나눈다) · `EFFORT`(샘플링 파라미터가 아니다) · `SCHEMA_NAME`(OpenAI 가 요구한다) · `Ask::schema`(`strict` 가 모르는 키워드를 거부한다) · `Ask::stub`(스키마와 함께 제공자에게 가지 않게) · `Answer.calls` 가 상수가 아닌 이유 · `strict` 가 스키마를 구속력 있게 만든다는 것 · Responses API 를 고른 이유 · `status` · `type` 필드의 상류 의미 · 거절이 200 이라는 것(두 경로) · 잘림도 200 이고 「malformed」로 보고하면 운영자를 엉뚱한 곳으로 보낸다는 것 · 서빙된 모델명이 고정 이름과 다를 수 있다는 것 — 상류의 문서화되지 않은 동작이다.
- `Language` 의 「산문만 옮긴다 — 경로 · 기호 · 키를 번역하면 조인이 깨진다」 · 지시문 한 문장씩의 이유 · `supports_analysis` 가 `default_model` 에서 파생되는 이유(등록 게이트와 디스패치가 갈라지지 않게) · `ALL_PROVIDERS` 가 있는 이유 · `DEFAULT_PROVIDER` 가 세 곳과 같은 답이어야 하는 이유 · `ask` 의 오류 문면 규약 · 미지원 분기가 조용한 stub 대신 시끄럽게 실패하는 이유 · 오류 문면에 요청 · 키를 넣지 않는다(두 곳) · 사유 코드만 정화해 싣는다 — 각각 갈라지면 과금 · 비노출이 깨지는 규칙이다.
- `STUB_MODEL` 이 실물과 구분되는 이유 · `stub_answer` 의 토큰 계수와 실패 트리거의 의미(양쪽이 맞아야 발화) · 실물 모양의 429 문면 · `strict` 스키마 단정이 stub 에서는 드러나지 않는다는 것 — stub 이 real 과 갈리는 충실도 경계다.
- 테스트 doc — 실패 트리거 env 가 프로세스 전역이라는 경고 · 키 없는 Real 이 stub 으로 새지 않아야 하는 이유 · 미지원 제공자 단정을 `ALL_PROVIDERS` 전체에 거는 이유 · 「등록 가능 iff 호출 가능」 · 기본값이 등록 가능해야 하는 이유 · 기본값을 정의 자리에서 단정하는 이유 · 거부되는 샘플링 파라미터 목록의 출처 · stub 답이 스키마에 실리지 않아야 하는 이유 · 언어가 사용자 턴으로 새지 않아야 한다는 것 · 추론만 있는 출력이 빈 문서가 아니라는 것 — 각각 테스트가 무엇을 지키는지 모르면 「정리」로 공허하게 만들 수 있는 자리다.

**워커 큐 프로토콜 — `backend/src/worker_api.rs`** (L 169)
- 모듈 머리 — 워커가 DB 를 열지 않는 이유(RWO 볼륨 위 SQLite 의 단일 writer) · 신뢰 경계(세션이 아니라 워커 토큰, 토큰 미설정이면 라우트가 없다, 작업 범위 설치 토큰) — 저장소 제약의 함정과 격리 불변식이다.
- `LEASE_SECONDS` 의 크기 근거 · 의존성 라우트가 본문을 쓰는 이유 · `WorkerAuth` 의 「미설정 = 없음」 · 상수 시간 비교 · `worker_id` 기록 이유 — 임대 계약과 비밀 비노출이다.
- `ClaimView` 필드 doc(두 사람 게이트가 큐의 속성이라는 것 · 이미 성공한 단계를 다시 제안하지 않는 이유 · `cross_cutting_document` 가 실리는 조건 · 승인 패턴 · 후보 · 의존성 요청을 클레임에 싣는 이유 · 설치 토큰과 LLM 키를 저장 · 기록하지 않는다) — 클레임 프로토콜의 의미이고, 비어 있음이 곧 「게이트 전」이다.
- `claim` doc(위에서 제자리로 옮김) — 만료 임대 회수와 N 워커 안전성의 근거(서브쿼리 재평가, `UPDATE` 술어가 오늘은 중복이라는 실측) — 지우면 서브쿼리 필터를 「중복」으로 보고 걷는 실수를 막을 것이 없다. `stop_for_revoked_access` 의 새 요약 — 호출 자리가 정책을 지킨다.
- 설치 토큰 · LLM 키를 그 자리에서 발급 · 복호화하는 이유 · AC1.3 게이트를 클레임 뒤에 읽는 이유 · `approved_patterns` · `approved_candidates` 의 게이트 값 의미 · `acceptance_pending` 이 단계 성공을 보지 않는 이유 · `pending_dependency_requests` 의 행이 곧 게이트 · `offered_stages` 규칙 전문 · `work_remains` 와 `finish` 의 경합 한 쌍 · 실행 중 도착한 의존성 요청 — 큐 동시성 계약이다.
- `heartbeat` · `report_stage` · `submit_document` 의 임대 가드 · `submit_document` 가 upsert 인 이유 · 충돌을 같은 요청에서 이어받는 이유 · `calls` 기본값 1 · `feature_key` 를 본문에 싣는 이유 · `submit_dependencies` 의 교체 · 실패 보존 · `content_hash` 가 재현성을 관측 가능하게 만든다는 것 · `require_lease` 의 409 — 각각 회수된 워커가 후임의 진척을 덮는 것을 막는 규칙이다.

**LLM 키 — `backend/src/llmkey.rs`** (L 40)
- 모듈 머리 — 봉투 암호화와 평문이 머무는 구간(doc 주석 수준 + 비노출 불변식).
- `Provider::llm` 의 두 enum 을 가르는 이유와 이 다리가 유일한 접점이라는 것 · `LlmKeyView` 가 식별자만 낸다는 것 · `ACTIVE_KEY_SQL` 의 정렬 규칙(OpenAI 우선 · 최근 등록)과 세 곳이 같은 답이어야 하는 이유 · `preflight` 와 공유하는 이유 · 정렬이 호출 가능성을 보지 않아도 되는 이유와 등록 범위를 넓히면 그 불변식이 깨진다는 경고 — 과금 대상 제공자가 갈라지는 함정이다.
- `preflight` · just-in-time 복호화 · `active_key_for_user` 의 평문 전달 경로와 `Ok(None)` 의 의미 · `validate_key` 의 stub/real 차이와 키 비반향 · `mask` 요약 — 자격증명 비노출 불변식과 stub 충실도 경계다.
