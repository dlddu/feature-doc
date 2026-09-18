# 패스 상세 — backend 집중 4파일 (판정일 2026-09-17)

범위: `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` ·
`backend/src/llmkey.rs` — 등록 시점 집중 구간 4파일, 주석 544행(전체 2,777행의 20%).
원장 행: [../ledger.md](../ledger.md). 판정 트리: 5c28852d.

| 결과 | |
|---|---|
| 순 제거 | 58행 — 구분선 16 · 빈 주석 행 3 · 문서·선언 재진술·작업 흔적 39 (총 62행 제거 중 자격증명 불변식 2행 재작성) |
| 유지 | 486행 — 동시성 계약 · 상류 API 거부 조건 · 자격증명 불변식 등 복원 불가 지식 |
| 범위 지문 | 판정 시점 `85da1f07…f91ac` → 판정 후 `aa193195…375ee` |

## 제거 근거표 — 구분선 · 빈 주석 행 (19행)

구분선(`// ── … ──`)은 바로 아래 선언이나 문단이 이미 말하는 절 제목이다. 이들 16행과 함께,
구분선 제거의 형식 정리로 없어진 빈 주석 행(`//`만 있는 행) 3건도 같이 지웠다.

| 파일 | 지운 내용 | 그 자리를 말하는 것 |
|---|---|---|
| `llmkey.rs` | `// ── provider ──` | 다음 줄 `enum Provider` |
| `llmkey.rs` | `// ── views / rows ──` | 다음 줄 `LlmKeyView`·`SealedKey` 문서 |
| `llmkey.rs` | `// ── handlers ──` | 다음 줄 `RegisterReq` |
| `llmkey.rs` | `// ── validation + display helpers ──` | 다음 줄 `validate_key` 문서 |
| `llm.rs` | `// ── real: Anthropic Messages API ──` | 다음 줄 `AnthropicResponse` |
| `llm.rs` | `// ── real: OpenAI Responses API ──` + 뒤따른 빈 주석 행 | 다음 줄 `// The Responses API rather than Chat Completions…` 근거 블록(유지) |
| `llm.rs` | `// ── stub ──` | 다음 줄 `STUB_MODEL` 문서 |
| `llm.rs` | `// ── OpenAI request/response shape ──` (tests) | 다음 줄 `openai_request_carries_the_schema_and_no_sampling_parameters` |
| `worker_api.rs` | `// ── authentication ──` | 다음 줄 `WorkerAuth` 문서 |
| `worker_api.rs` | `// ── claim ──` | 다음 줄 `ClaimReq` |
| `worker_api.rs` | `// ── lease + progress ──` | 다음 줄 `WorkerIdReq` |
| `analysis.rs` | `// ── views / rows ──` | 다음 줄 `AnalysisView` |
| `analysis.rs` | `// ── handlers ──` | 다음 줄 `list_repositories` 문서 |
| `analysis.rs` | `// ── helpers ──` | 다음 줄 `load_detail` 문서 |
| `analysis.rs` | `// ── discovery strategy review · edit · approve (AC1.3) ──` + 뒤따른 빈 주석 행 | 다음 줄 `// Stage 3 proposes; this is where a person decides…` 근거 블록(유지) |
| `analysis.rs` | `// ── feature candidate review (AC1.4) ──` + 뒤따른 빈 주석 행 | 다음 줄 `// Stage 4 extracts; this is where a person sifts…` 근거 블록(유지) |

## 제거 근거표 — 문서·선언 재진술 · 작업 흔적 (39행)

인용은 의미를 보존한 축약이다. 복원 경로: ① 코드 자체 ② 저장소 문서 ③ PR ④ 커밋 메시지.

### `backend/src/llmkey.rs`

| 지운 내용 | 경로 | 사유 |
|---|---|---|
| `prefix()` 문서 2행 — "The provider's public key prefix (not secret) — used for stub validation and for masking display." | ① | 세 값(`sk-ant-`·`sk-`·`AIza`)과 두 사용처(`validate_key` stub 형식 검사 · `mask`)가 본문에 있다. |
| `SealedKey` 문서 1행 — "The sealed columns needed to decrypt a key just-in-time at use." | ① | 필드 4개가 곧 봉투 구성이고 `crypto::open(Envelope)`가 그 쓰임을 말한다. |
| `register()` 문서 3행 — "Registers a key: validate provider → … → store ciphertext only. Returns the identifier view (201)." | ① | 본문 절차(파싱→범위 게이트→live 검증→seal→INSERT)와 201 반환을 순서대로 되풀이한다. |
| `register()` 인라인 5행 — "AC4.2 지원 제공자 범위. 분석 호출이 구현되지 않은 제공자의 키는 받지 않는다 — … 이미 동작하던 분석을 조용히 깨뜨린다… 거부는 사용자가 아직 아무것도 맡기지 않은 이 지점에서 일어난다." | ② | `docs/doc-tracker.md` 「LLM 프로바이더 지원 범위」 절이 같은 결정(등록 시점 거부)과 근거(`ACTIVE_KEY_SQL` 등록 순위 실측, `rct_20260902-0002`)를 갖고 있고, PRD-04 AC4.2 「지원 제공자 범위」의 (a)(b)가 지금 이 문장들이다. 코드 측 함정의 경고는 유지되는 `ACTIVE_KEY_SQL` 문서 블록이 같은 자리에 갖고 있어 소실 없다. |
| `list()` 문서 1행 — "Lists the user's keys — identifiers only." | ① | 함수 이름과 SELECT 열 목록이 그대로 말한다. |
| `revoke()` 문서 2행 — "Revokes one of the user's keys. Scoped to the owner — another user's id is a 404 (AC4.7), and new calls are blocked thereafter." | ① | `UPDATE … WHERE id = ? AND user_id = ? AND status = 'active'` + `rows_affected()==0 → 404`가 소유자 스코프·폐기 후 차단을 구현한다. |
| `preflight()` 문서 5행 중 3행 — "Connect Repository's '분석 시작' preflight will call"(연동 예고) · "No active key → blocked with '키가 없거나 폐기되었습니다' (test#4)" · "full per-call delegation (test#3) lands with the analysis pipeline"(후속 예고) | ①·② | 본문의 `ok_or_else` 오류 문자열이 앞 문장을 말하고, 시나리오 귀속(test#4)은 `docs/test/04-platform.md`의 자리, 후속 예고는 작업 흔적(④의 자리). 유지된 2행에 불변식 문장을 남긴다(아래 유지 목록). |
| `fingerprint()` 문서 1행 — "Non-reversible identifier for a key (first 64 bits of its SHA-256, hex)." | ① | 본문이 `Sha256::digest(key)[..8]` hex 그대로다. |

### `backend/src/worker_api.rs`

| 지운 내용 | 경로 | 사유 |
|---|---|---|
| `ClaimView.llm_provider` 문서 1행 — "The provider the LLM key below belongs to (`anthropic` / `openai` / `google`)." | ① | 값 집합은 `llm::Provider`가 소유한다. |
| `FinishReq.status` 문서 1행 — "`awaiting_pipeline` (every implemented stage ran) or `failed`." | ① | `finish()`의 `matches!`가 두 터미널 값을 소유한다. |
| `DocumentReq.content` 문서 1행 — "The stage's JSON output, as produced by the pipeline stage module." | ① | 필드명·`submit_document` 저장 경로가 말한다. |
| `DocumentReq.model` 문서 2행 — "Model identifier and per-call token usage — the cost accounting AC4.6 surfaces in a later slice." | ①·② | 필드 의미는 필드명과 저장 경로에서 복원되고, "AC4.6 이후 슬라이스" 예고는 작업 흔적 — `docs/doc-tracker.md` 구현 로드맵(슬라이스 7)의 자리다. |

### `backend/src/analysis.rs`

| 지운 내용 | 경로 | 사유 |
|---|---|---|
| `AnalysisView` 문서 1행 — "What the API exposes for an analysis job (the home list)." | ① | 필드 구성이 말한다. |
| `StageView` 문서 1행 — "One pipeline step as Analysis Progress renders it, straight off `analysis_stages`." | ① | 구조체 필드가 `analysis_stages` 행 그 자체다. |
| `StageView.detail` 문서 1행 — "The user-facing one-liner the worker measured (\"766 files · 2.2 MB\")." | ①·② | 필드 쓰임은 worker 보고 구조에서 복원되고, 예시 실측값 `766 files · 2.2 MB`은 `docs/doc-tracker.md` 시나리오 5 행이 갖고 있다. |
| `RunRow` 문서 1행 — "The analysis-level columns Analysis Progress adds on top of `AnalysisView`." | ① | 필드 3개가 말한다. |
| `list()` 문서 1행 — "The user's analysis jobs, newest first (the home list). Scoped to the owner (AC4.7)." | ① | `ORDER BY a.created_at DESC` + `WHERE a.user_id = ?`가 그대로다. |
| `load_detail()` 문서 1행 — "Reads one analysis and its stages under the owner's scope, or `404`." | ① | 본문 쿼리와 `.ok_or(AppError::NotFound)`가 말한다. |
| `accessible_repos()` 문서 1행 — "The repositories the user's installation can access, or empty when not installed." | ① | `None => Ok(Vec::new())`가 그 말이다. |
| `resolve_branch()` 문서 1행 — "The requested branch when non-blank, else the repo's default branch." | ① | 본문이 그 한 줄이다. |
| `parse_repo()` 문서 3행 — "Parses a repo target. Accepts `owner/name`, `github.com/owner/name`, and full `https://…(.git)` URLs. Anything else is a validation error…" | ① | 본문이 받는 형식을 그대로 구현한다. |
| `parse_repo()` 인라인 1행 — "Drop scheme + host if present, keep the path." | ① | 바로 아래 `split_once("://")` 한 줄이 말한다. |
| `CandidateView` 문서 1행 — "One candidate as Feature Candidates renders it." | ① | 필드 구성이 말한다. |
| `DecisionReq.key` 문서 1행 — "Which candidate — its `candidate_key`." | ① | 필드명이 말한다. |
| `DecisionReq.decision` 문서 1행 — "`approve` or `reject`." | ① | `decide_candidate()`의 `match`가 소유한다. |
| `seed_candidates()` 문서 1행 — "Materialises `feature_candidates` rows from stage 4's document, once." | ① | 부가 근거 — "once"는 행별 `INSERT OR IGNORE` 사실과 어긋나는 낡은 표현이라 제거 근거를 강화한다. |

## 유지 목록 (요지)

전체 유지 486행의 판정은 [README.md](../README.md)의 유지 대상 기준을 따랐다. 특히 무거운 블록들:

- **`llm.rs` 모듈 머리 전체** — prefill이 현재 모델에서 HTTP 400으로 거부된다는 점, `temperature`를
  포함한 sampling parameter가 400(= no-op 아님)이라는 점, 결정성을 관측(`content_hash`)으로
  확보한다는 점, 키는 인자로만 흐르고 오류에도 나오지 않는다는 점 — 상류 API의 문서화되지 않은
  거부 조건과 충실도 경계의 1차 저장소다.
- **`llmkey.rs` `ACTIVE_KEY_SQL` 문서 블록** — OpenAI 우선·최신순 규칙과 세 지점(등록 화면 초기
  선택·활성 키 규칙·worker 폴백)의 합의, "등록 범위만 넓히면 미지원 키가 동작 중 키를 빼앗는"
  회귀 경고와 그걸 고정하는 테스트 이름. doc-tracker에 같은 결론의 산문이 있어 부분 겹침이지만
  이 자리의 경고 배치가 미래 편집자를 지킨다 — 「애묘하면 남긴다」 판단 갈림 항목이다.
- **`llmkey.rs` 모듈 머리 · `LlmKeyView` · `validate_key` · `mask` · `active_key_for_user`** —
  봉투 암호화·"never returns or logs"/"never reveals"/"never appears in an error" 계열 자격증명
  비노출 불변식과 그 why, 큐 인계(worker가 DB를 열지 않는다) 설계.
- **`worker_api.rs` 모듈 머리 · `LEASE_SECONDS` · `WorkerAuth` · `constant_time_eq` · `claim` ·
  `offered_stages` · `finish` · `submit_document`** — SQLite 단일 writer를 깨지 않는 HTTP 큐
  프로토콜 설계, 임대 튜닝 근거, unset token ≠ 전허용, 타이밍 공격 방지 why, N-워커 원자성 실측,
  승인 게이트가 큐의 성질로 표현된 설계, `cross_cutting` 재제공 예외 규약, 승인 경합 쌍,
  AC1.5 재시도의 upsert 규약.
- **`analysis.rs` 모듈 머리 · `detail`/`owned_analysis`/`document` · `retry_stage` · 재현성
  쿼리 인라인 · 지연 시딩 · `approve_strategy` · `update_strategy` · 후보 검토 계약** — 404(not
  403) 선택의 why, 리스 아래 재큐잉 금지, unix 초 타이 동률의 `rowid` 타이브레이커 실측 함정,
  lazy 시드의 멱등성, 승인+재큐잉 한 트랜잭션의 이유, 거부 사유 강제의 이유, 병합 행 보존의
  이유 등. 모듈 머리 35행은 AC 문구와 일부 겹치지만 모듈 머리 유지 조항을 적용해 통째 보존했다.

## 판단이 갈려 남긴 것

1. `llmkey.rs` `ACTIVE_KEY_SQL` 문서 블록 — `docs/doc-tracker.md` 「LLM 프로바이더 지원 범위」
   절과 같은 내용의 산문이 있어 제거 후보에 들 수 있다. 남긴 근거: 회귀 경고("등록 범위만 넓히면
   안 된다")가 SQL 바로 위에 있어야 미래 편집자가 그 순간에 막히고, 회귀 테스트 이름 고정이
   여기에 있다. 문서가 이유를 대신 쓰더라도 그 지점 옆 경고의 가치가 제거 비용을 넘는다고 본다.
2. `analysis.rs` 모듈 머리 — enqueue 절반의 문장 중 AC1.1·AC1.5 검증 문구와 겹치는 부분이 있다.
   남긴 근거: 모듈 머리는 파일의 지도이고 본문의 유지 대상이 모듈 머리를 명시하므로, 개별 문장
   수술보다 통째 보존이 부작용이 작다.
3. `worker_api.rs` 소형 핸들러 문서(`heartbeat` "Only the holder may extend it", `require_lease`
   "409s unless…" 등) — 순수 what에 가깝지만 한 줄 보존 비용이 무시 가능하고 그 서술이 계약
   (홀더만 연장 · 만료 리스 거부)을 함께 말한다.

## 이후의 증분 재판정

같은 범위에 주석 변화가 오면(감지가 범위 지문 변화로 연다) **이 파일에 절을 더한다** — 새 패스
파일을 만들지 않는다. 이 패스는 아직 병합 전이므로, 병합 커밋 트리에서 범위 지문이
`aa193195…375ee`로 재현되는지 실행·검증 단계에서 확인한다.

전역 지문은 판정 이후 main이 움직여(#51·#52의 `mock-exception:` 외 주석 12행 추가) 판정 시점
예고값 `lines=2719`가 아니라 `lines=2731 files=89`로 재현된다 — 범위 지문은 영향이 없다
(`mock-exception:`은 본문 「유지 대상」의 기계가 읽는 주석으로 지문에서 제외되고, 나머지 추가는
이 패스의 4파일 밖이다). 범위 밖 12행은 미판정 잔여로 남아 다음 패스가 받는다.

## 증분 재판정 ① — `llm.rs` +21행 (2026-09-18 · `rct_20260918-0003`)

`#55`(원장 R1 해소 — stub LLM에 결정적 실패 트리거 추가)가 이 범위의 `backend/src/llm.rs`에
주석 **21행**을 더해 범위 지문이 움직였다. 판정 범위·파일 목록은 그대로이므로 위의 규칙대로
새 패스 파일을 만들지 않고 여기에 절을 더한다.

- **범위 지문**: `aa1931952efd07b621726cc6a524f742085bbe5ebbf3e94b6e3b3c210ab375ee`(486행, 이
  패스 병합 직후) → `83758c065a2a2e2cadeca3b3e93753202625c38559cd4339e2ea94aab4e8e1ca`(507행).
- **판정 결과**: **21행 전건 유지 · 제거 0.**

### 왜 전건 유지인가

21행은 전부 `FEATUREDOC_STUB_LLM_FAIL` 트리거를 둘러싼 서술이고, **stub이 real과 갈리는 지점
(충실도 경계)** 이다 — 본문 「유지 대상 · 복원 불가능한 지식」이 이름으로 열거한 항목이다.
구체적으로 네 가지를 말하는데 어느 것도 코드·문서·이력에서 복원되지 않는다.

1. **트리거가 없던 시절 무엇이 불가능했는가** — 「real LLM 호출은 사용자 입력이 재현할 수 없는
   이유(제공자 한도·타임아웃)로 실패할 수 있는데, 이 트리거 전에는 stub 모드가 LLM 오류로
   단계 실패 분기에 **닿을 수 없어** `sc01-06`이 트리 404로 우회해 들어갔다」. **과거에 없던
   것**이라 현재 코드에서 읽히지 않는다.
2. **트리거의 비대칭 계약** — 「실패를 *더하기만* 하고 빼지 않는다」. 더블이 real보다 관대해지지
   않는다는 불변식이고, 분기를 읽어서 귀납할 수는 있어도 *의도된 방향*은 여기에만 있다.
3. **무발화 조건** — 「env 미설정, 또는 입력이 needle을 담지 않으면 아무것도 바뀌지 않는다」 →
   다른 모든 분석에 불활성이라는 보증.
4. **테스트 격리 근거** — 「env 쓰기는 이 테스트 안에만 머물고 다른 테스트가 이 변수를 읽지
   않으므로 병렬 실행이 경합하지 않는다」. `std::env::set_var`가 프로세스 전역이라는 함정 위에서
   왜 안전한지의 유일한 설명이다.

### 제거 후보로 지목하되 이번에 손대지 않은 것

테스트 본문의 case 라벨 3행(`// Env unset: nothing changes.` ·
`// Env set and the input carries the needle: real-shaped provider failure.` ·
`// Env set but the input misses the needle: still the deterministic answer.`)은 바로 아래
두세 줄(`remove_var` / `set_var` + 단언)이 그대로 말하는 절 제목에 가깝다 — 중복 유형 ④의
후보다.

**그럼에도 이번에 지우지 않은 이유는 판정이 아니라 경합이다**: `backend/src/llm.rs`는 열린 draft
PR **#49**가 건드리는 파일이고, 이 축은 「열린 통합 차량의 충돌 면적을 넓히지 않는다」를 방침으로
두고 있다. #49가 머지·폐기된 뒤 이 범위를 다시 열 때 **이 3행이 첫 항목**이다.

## 증분 재판정 ② — `#43`이 더한 40행 (2026-09-18 · `rct_20260918-0004`)

`#43`(`6484e21`, 슬라이스 5a)이 이 범위의 `backend/src/analysis.rs`·`backend/src/worker_api.rs`에
주석 **40행**을 더해 범위 지문이 다시 움직였다. 직전 패스(`rct_20260918-0003`)가 「1행은 그만큼
증분 재판정 대상이며 이 패스의 범위 밖」이라고 넘긴 바로 그 40행이다. 규칙대로 새 패스 파일을
만들지 않고 여기에 절을 더한다.

- **범위 지문**: `83758c06…`(507행) → `2c4f8e1623bb0890559eb9314d652b462520b1744aaa492d93fd4b1e98e7d961`
  (547행, #43 머지 후) → **`60954a7f865e9449bfe9a367cebbe0400ccf8f86f52358a7d22eb3edaffad401`(537행)**.
- **판정 결과**: **순 제거 10행**(`analysis.rs` 3 · `worker_api.rs` 7) · 유지 30행.
- `llm.rs`·`llmkey.rs`는 이번 창에서 주석 변화 0이다(`#60`이 `llmkey.rs`의 코드를 고쳤지만
  주석 줄은 건드리지 않았다 — 지문으로 확인).

### 제거한 것 (10행)

| 자리 | 지운 문장 | 복원 경로 |
|---|---|---|
| `analysis.rs` `requeue` doc | 「Both approvals (strategy, candidate) go through here so there is one answer to "what does approving do to the queue".」 | ① — 호출자가 둘뿐이고 둘 다 이 함수를 부른다 |
| `analysis.rs` `decide_candidate` | 「Approving is what opens stage 5 … for the same reason `approve_strategy`'s pair is」 | ① — 같은 파일 위쪽의 그 쌍이 보인다 |
| `worker_api.rs` `executable_stages` doc | AC1.3·AC1.4→AC2.1 조항 재진술 + `「승인된 전략만 다음 단계의 입력이 된다」`·`「확정된 feature 에 대해서만 표현을 만든다」` 인용 | ② — `docs/prd/01`·`docs/prd/02`와 `doc-tracker` 5a 행 |
| `worker_api.rs` `approved_candidates` 필드 doc | 「(AC1.4)」 라벨 | ② |
| `worker_api.rs` `CandidateRef` doc | 「the same four fields `feature_candidates::Subject` needs, named as they are stored」 | ① — 구조체 본문이 그 네 필드다 |
| `worker_api.rs` `approved_candidates()` doc | 「(AC2.1~AC2.3 are about a **confirmed** feature …)」 · 「which is exactly what `merged_into` records」 | ②·① — 뒤는 바로 아래 `WHERE … merged_into IS NULL` |
| `worker_api.rs` `offered_stages` 본문 | 「// Stage 5's predicate is coverage, not success — see [`acceptance_pending`].」 | ① — 링크 대상의 doc이 그 말을 한다. 본문 「rustdoc 링크만으로 이루어진 교차 참조는 링크를 위해 문장을 남기지 않는다」 |

### 유지한 것 (30행) — 이유

- `requeue`의 **리스 계약**: 「살아 있는 리스 아래의 잡은 건드리지 않는다(`status <> running`) —
  점유자 밑에서 재큐잉하는 것은 `retry_stage`의 불변식이 금지하는 바로 그것이고,
  `worker_api::finish`가 할 일이 남았음을 보면 스스로 `queued`로 내려놓는다」. 정책이 유지
  대상으로 이름 붙인 **동시성 계약**이다.
- `decide_candidate`의 **트랜잭션 원자성**과 「거부는 아무것도 열지 않으므로 재큐잉하지
  않는다 — 전부 거부된 분석이 돌면 안 된다」.
- `executable_stages`의 「게이트는 큐의 성질이지 워커가 기억하기로 한 규칙이 아니다 / 워커는
  자기가 아는 키 중 제안받은 것만 실행한다」와 「이미 성공한 단계는 다시 제안하지 않는다 —
  그래야 승인 후 재큐잉이 2·3단계의 LLM 호출을 다시 돌려 승인된 문서를 덮어쓰지 않는다」.
- `acceptance_pending` doc **전문**: 「술어는 「단계가 성공했는가」가 아니다 … 검수자는 후보를
  한 번에 하나씩 결정하므로 두 번째 확정이 단계를 다시 열어야 하고, 아니면 그 feature는 영영
  문서를 못 갖는다 … 재실행은 문서를 통째로 다시 쓴다(`(analysis_id, kind)` upsert)」. 술어를
  잘못 고르면 **조용히** 문서 하나가 사라지는 함정이라, doc-tracker에 같은 서술이 있어도
  코드 옆의 이 사본을 남긴다(판단이 갈린 항목 — 이번 패스 상세의 「판단이 갈려 남긴 것」 3번).
- `work_remains` doc **전문**: 「승인이 워커 실행 *중에* 도착해 리스 때문에 재큐잉이 거부되는
  단 하나의 경합을 `finish`에서 닫는다 / 게이트 뒤 단계만 센다 — `fetch`는 항상 제안되므로
  「제안된 것이 있는가」로 물으면 모든 잡을 영원히 재큐잉한다」. 실패 모드의 함정이다.

### 검증

주석·빈 줄을 걷어낸 나머지가 부모 `ebe8657`과 **바이트 동일**이고, 두 파일 diff의 주석 아닌
`+`/`-` 행은 0건이다. 자세한 대조는 [2026-09-18-acceptance-axis.md](2026-09-18-acceptance-axis.md)
「검증」 절에 함께 적었다.
