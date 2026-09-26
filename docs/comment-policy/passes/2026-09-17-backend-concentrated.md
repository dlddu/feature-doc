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

**— 집행됨(2026-09-25 · 31차 패스 · `rct_20260925-0008`).** #49 는 `2026-09-18T15:40:05Z` 에 머지됐고,
이 패스가 그 인계대로 **이 3행을 첫 항목으로** 집행했다. 아래 「보류분 집행」 절.

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

## 증분 재판정 ⑤ — `#93` 의 「required-but-nullable」 9행 (2026-09-21 · `rct_20260921-0001`)

`#93`(`8205b7a`, 「스키마 선택 필드를 required+nullable 로 — OpenAI strict 400 해소」)이 **이미
판정된 네 범위**에 주석 9행을 들여왔다 — 원장 1행의 `backend/src/llm.rs` 5행 · 4행의
`backend/src/acceptance.rs` 1행 · 6행의 `backend/src/dependencies.rs` 1행 · 8행의
`backend/src/feature_candidates.rs` 2행. 아홉 줄이 **한 명제를 둘러싼 한 묶음**이라 상세는 정본이
있는 이 파일에 한 절로 적고, 다른 세 패스 파일에는 포인터만 둔다(원장 4·6·8행이 그 포인터를 링크한다).
같은 커밋이 `cross_cutting.rs`·`discovery_strategy.rs` 에 더한 것은 테스트 fn 뿐이라 지문 델타 0 이다.

- **창**: 부모 `7724b46`(15차 패스) → `8205b7a`. 부모에서 versionScript 를 재실행한 값이 저장된
  baseline `lines=2310` / `7d1ddd5f…` 과 바이트 동일 — 유입 전부가 #93 단독 귀속이다.
- **판정 결과**: **9행 중 제거 6 · 유지 3** (`llm.rs` 5 → 3 재작성 · `acceptance.rs` −1 ·
  `dependencies.rs` −1 · `feature_candidates.rs` −2). 비주석 코드는 한 글자도 건드리지 않았다.

### 명제는 하나다 — 정본을 강제하는 코드 옆에 두고 사본을 걷었다

아홉 줄이 말하는 것은 한 명제 **P** = 「OpenAI 는 `strict: true` 아래에서 `required` 에 없는
property 를 400 으로 거부하므로, 선택 필드는 optional 이 아니라 required-but-nullable
(`["string","null"]`)로 적는다」이다. #93 은 P 를 **네 자리에 다섯 벌**로 적었고, 동시에 P 를
**강제하는 코드**도 들여왔다 — `llm.rs::assert_strict_schema`(모든 object 의 `additionalProperties:
false` 와 「모든 property 가 `required` 에 있음」을 단정하고, 단정 메시지가 「strict mode needs every
property in `required`; make it nullable instead of optional」이라 축자로 적는다)와, 스키마를 가진
네 모듈 각각의 테스트 `schema_is_accepted_by_openai_strict_mode`.

12차 패스가 세운 규칙(「복제된 명제는 그 명제를 강제하는 코드 옆 **한 벌만** 남긴다」 —
[2026-09-20-github-app-auth-axis.md](2026-09-20-github-app-auth-axis.md) 「복제된 명제」 표)을 그대로
적용했다. 그 표의 「Setup URL 의 `installation_id` 는 스푸핑 가능」 행과 모양이 같다 — 정본은
**검증하는 함수 자신** 옆에, 호출 자리의 사본은 **호출 이름이 이미 말하므로** 걷는다.

| 자리 | 줄 | 판정 | 근거 |
|---|---|---|---|
| `llm.rs` `assert_strict_schema` doc | 5 → 3 | **정본. 명제 단위로 잘라 2행 제거** | 아래 절 |
| `feature_candidates.rs:36-37` | 2 | **제거** | P 의 사본. 같은 파일의 테스트 `schema_is_accepted_by_openai_strict_mode` 가 **이름으로** 정본 자리를 가리키고(①), 단정 메시지가 「make it nullable instead of optional」을 말한다(①). 12차 규칙의 「호출 이름이 말한다」 형 |
| `acceptance.rs:71` · `dependencies.rs:78` | 1 + 1 | **제거** | `// Required-but-nullable; see feature_candidates::schema.` — 앞 절반은 두 줄 위 `required` 배열과 바로 아래 `"type": ["string","null"]` 의 **축자 재진술**(①), 뒤 절반은 **다른 파일 주석으로의 교차 참조뿐**(본문 「링크를 위해서만 문장을 남기지 않는다」). 게다가 **같은 문장의 두 벌**이다. 걷고 나니 두 범위의 줄 수·지문이 #93 이전 값으로 **정확히 되돌아왔다**(141 / `e38dcf20…` · 112 / `e2ff313a…`) |

**지식은 한 줄도 사라지지 않았다.** P 의 두 요소 — 상류 거부 조건(400)과 그 처방(nullable) — 는
`llm.rs` 의 정본 doc 과 `assert_strict_schema` 의 단정 메시지에 남아 있고, 스키마를 가진 네 파일
어느 곳에서 출발해도 같은 파일의 테스트 이름 → `assert_strict_schema` 로 두 번에 닿는다.

### `llm.rs` 의 정본 doc — 명제 단위로 잘랐다

`#[cfg(test)] pub(crate) fn assert_strict_schema` 의 `///` 5행은 네 명제가 줄에 걸쳐 있었다.
줄 단위가 아니라 명제 단위로 판정해 **5행 → 3행**으로 재작성했다.

| 명제 | 판정 | 근거 |
|---|---|---|
| 「every object closes `additionalProperties` and lists *every* property in `required`」 | 제거 | 함수 본문의 두 `assert_eq!`/`assert!` 가 그대로다(①) — 본문 「doc 주석 본문이 시그니처·동작을 되풀이하는 부분」 |
| 「A property left out of `required` is a 400 before the model runs」 | **유지** | 상류 API 의 거부 조건 — 본문 「유지 대상」이 이름으로 든 항목(OpenAI Responses 의 400). 「모델 실행 *전*」이라는 시점은 코드 어디에도 없다 |
| 「it only shows up against the real provider — stub mode never sends the schema anywhere」 | **유지** | stub↔real 충실도 경계. 증분 재판정 ① 이 `llm.rs` 의 같은 계열 21행을 전건 유지한 선례와 일치 |
| 「Optional fields are spelled required-but-nullable (`["string", "null"]`)」 | 제거 | 단정 메시지 「make it nullable instead of optional」(①)과 네 스키마 리터럴 자체(①)가 말한다 |

남긴 3행은 `pub(crate)` 항목의 요약 1줄 + 복원 불가능한 두 명제다.

### 선례와의 정합 — 뒤집은 판정은 없다

- 9·10차 패스가 `cross_cutting.rs` 의 「스키마는 프로바이더에 그대로 가므로 stub 답이 그 안에
  있으면 안 된다 — OpenAI 는 `strict` 아래에서 모르는 키워드를 거부한다」를 **유지**로 닫았다
  ([2026-09-20-pipeline-cross-cutting-axis.md](2026-09-20-pipeline-cross-cutting-axis.md)). 그것은
  **다른 명제 Q**(모르는 키워드 거부)이고 이번에 손대지 않았다. 「상류 거부 조건은 유지」라는 그
  판정은 이번에도 같다 — P 의 거부 조건은 `llm.rs` 정본에 **남아 있다**. 걷은 것은 조건이 아니라
  **사본**이다.
- 복원 경로 ③④ 도 열려 있었다(PR #93 본문 「## 원인」이 P 를 축자로 재진술하고, 커밋 제목이
  「OpenAI strict 400 해소」다). 그러나 본문 「유지 대상」이 상류 거부 조건을 **이름으로** 유지 목록에
  둔 이상, ③④ 만을 근거로 정본까지 걷지는 않는다 — 12차 패스 「자격증명 비노출 불변식만은 두 벌을
  일부러 남겼다」와 같은 결의 판단이다. 정본 한 벌은 남기고 사본만 걷는 것이 두 규칙을 동시에
  만족하는 유일한 형태다.
- **판단이 갈려 남긴 것: 0건.** 네 자리 모두 정본이 코드 두 번 거리 안에 있어 「애매하면 남긴다」가
  발동할 자리가 없었다.

### 판정 표면 밖에서 본 것 (이번에 고치지 않음)

**Q 는 지금 `llm.rs` 에 세 벌 + `cross_cutting.rs` 에 한 벌**이다 — `Ask::schema` 필드 doc(`llm.rs:59-61`)
· `openai_body` 의 인라인(`:334-335`) · 테스트 `openai_request_schema_holds_only_the_schema` 의
doc(`:666-667`) · `cross_cutting.rs::the_schema_is_only_a_schema` 의 doc(`:265-266`). 1차·9·10차
패스가 각각 유지로 닫은 자리라 원장 규칙(「같은 명제를 패스마다 반대로 판정하면 그 자체가 drift」)상
**한 패스에서 네 벌을 함께** 다시 봐야 한다. 이번 창의 유입이 아니므로 열지 않았다 — 다음에 1행이
열릴 때의 첫 항목으로 남긴다. 12차 규칙을 적용하면 정본은 `openai_body` 의 `"strict": true` 를 쓰는
자리(`:334`)이고 나머지 셋이 사본 후보다.

### 검증 (판정 시점 로컬 실측, 부모 = main `899800e`)

1. **동작 코드 무접촉**: 문자열·문자 리터럴을 인식하는 스트리퍼로 주석·빈 줄을 걷어낸 잔여가
   부모와 **네 파일 전부 바이트 동일**(`llm.rs` 547 == 547 · `acceptance.rs` 526 == 526 ·
   `dependencies.rs` 450 == 450 · `feature_candidates.rs` 265 == 265 줄). diff 의 주석 아닌
   `+`/`-` 행 0건.
2. **범위 지문**: 1행 `b328c080…`(592) → **`9757b6b5a0251379c55161ab6768da3b3f05548f1866b2a98f82f98b4f4dea87`**(590) ·
   4행 `474321bd…`(142) → **`e38dcf20…`**(141, #93 이전 값) · 6행 `6058883e…`(113) →
   **`e2ff313a…`**(112, #93 이전 값) · 8행 `71dc18f5…`(153) →
   **`4f140458be956f26747278ba012e5afa626bf3e6dae7b0af98c05b029fce476c`**(151 — 그중 11행은 #99 가
   `sc01-04` 에 더한 미판정 증분).
3. **전역 as-is**: 부모 `lines=2349 files=108` / `e917bb5e…` → **`lines=2343 files=108` /
   `60eb8b5b47e21c8b6a5b870d502b3245366146384c8dd1cd22d97da08da33a95`**(순 제거 6). 절대값은
   자매 머지로 움직이므로 완료 기준은 **「부모 대비 순 제거 6행」**이다.
4. **데이터 저장 형식 판정기**: `python3 tools/check-data-format-change.py --base main --head <probe>
   --verbose` → `✅ 변경 없음` (네 파일 모두 검사 대상이되 D1·D2·D5·D6 어느 규칙에도 걸리지 않음)
   ⇒ 필수 체크 `review/data-format` 이 붙는다.
5. **허브 등재**: 새 문서를 만들지 않았으므로 `docs/index.html` 의 `Documents` 집계는 움직이지 않는다.

### 범위 밖 (같은 창에 들어온 다른 유입)

- **#91(`3567755`)이 10행 `frontend/src/index.css` 에 더한 5행** — `rct_20260921-0002` 가 열려 있다.
- **#99(`899800e`)가 3행 `sc01-01` 에 더한 14행 · 8행 `sc01-04` 에 더한 11행** — 이 재판정의 창
  밖(감지 대기). 8행의 줄 수 151 에는 포함돼 있고 원장이 「미판정 증분 11행」으로 표기한다.

## 증분 재판정 ⑥ — `#92` 가 `analysis.rs` 에 더한 4행 (2026-09-21 · `rct_20260921-0007`)

`#92`(`51daa9c`, 자매 모델 `tbm_feature-doc-docs-impl` 의 슬라이스 6a)가 `document()` 핸들러에
`doc_edit::overlay(&state, &id, &mut content).await?;` 한 줄을 끼우며 그 위에 쓴 주석 4행을 판정해
**전건 제거**했다. 판정 절차는 이 파일 본문과 같고, 새 파일 쪽 판정은
[2026-09-21-doc-edit-axis.md](2026-09-21-doc-edit-axis.md) 에 있다.

| 줄 | 복원 경로 |
|---|---|
| 「사람이 승인한 편집은 저장을 고치지 않고 읽는 자리에서 겹쳐진다 (AC3.1·AC3.4)」 | ① `doc_edit::overlay` 의 doc 「저장은 건드리지 않는다 — 문서를 내보내는 자리에서만 겹친다」(같은 패스가 정본으로 유지) · ② doc-tracker `2026-09.md` 슬라이스 6a 행 「**자동 문서를 고치지 않는다** — 승인된 편집은 `feature_doc_edits` 행으로 남고 문서를 내보내는 자리에서 겹쳐진다」 — AC 꼬리표 |
| 「아래 재현성 판정이 `row.content_hash` 를 쓰는 것은 그래서 그대로다 — AC1.2 가 묻는 것은 「재분석이 같은 결과를 냈는가」이지 「사람이 그 뒤에 문장을 다듬었는가」가 아니다」 3행 | ① `backend/migrations/0009_feature_doc_edits.sql` 머리 「왜 `analysis_documents` 의 행을 직접 고치지 않는가. 그 행의 `content_hash` 는 … 사람이 문장을 다듬었다는 이유로 「재분석 결과가 달라졌다」가 되면 두 판정이 동시에 거짓이 된다」 · ② doc-tracker 같은 행 「그러지 않으면 사람이 문장을 다듬었다는 이유로 AC1.2 재현성 판정과 AC2.6 재분석 diff 의 기준값(`content_hash`)이 함께 거짓이 된다」 · ③ PR #92 「설계 판단 2」 — **네 벌째** |

**왜 이 자리를 정본으로 두지 않았는가**: 명제를 강제하는 코드는 `overlay` 자신(저장에 쓰지
않는다)이고, 저장 계약의 이유는 마이그레이션 0009 가 「그 시점에 왜 이 변경을 했는가」의 자리로
이미 들고 있다(본문 「적용된 마이그레이션」 1항). `document()` 는 그 결과를 소비할 뿐이다. 12차
패스의 규칙(정본은 명제를 강제하는 코드 옆) 그대로다.

결과: 1행의 줄 수·지문이 #92 이전 값 **590 / `9757b6b5…`** 으로 되돌아왔다 — 부모 `19d58fa`
에서 같은 4파일을 재계산한 값과 바이트 동일(#92 가 이 4파일에 더한 것이 이 4행뿐이고 전건
제거라 재작성이 없다). `analysis.rs` 는 주석 제거 후 부모와 바이트 동일(stripper md5 `10548c24`).

## 증분 재판정 ⑦ — `#107` 이 `analysis.rs` 에 더한 2행 (2026-09-21 · `rct_20260921-0010`)

`#107`(`89a1625`, 자매 모델 `tbm_feature-doc-docs-impl` 의 슬라이스 6b)이 `document()` 핸들러에
`crate::feature_add::overlay(&state, &id, &mut content).await?;` 한 줄을 끼우며 그 위에 쓴 1행과,
`approved_candidate_name` 의 doc 1행에 덧붙인 절을 판정해 **전건 제거**했다. 판정 절차는 ⑥ 과 같고, 새 파일
쪽 판정은 [2026-09-21-feature-add-axis.md](2026-09-21-feature-add-axis.md) 에 있다.

| 줄 | 복원 경로 |
|---|---|
| 「사람이 확정한 추가(AC3.2)는 편집 겹침보다 먼저 얹는다 — 더해진 feature 도 편집의 대상이다」 | ① `feature_add::overlay` 의 doc 「편집 겹치기(`doc_edit::overlay`)보다 **먼저** 실행해야 한다: 그래야 더해진 feature 의 시나리오도 그 뒤의 편집 대상이 된다」(같은 패스가 순서 계약의 정본으로 유지) · ② doc-tracker 6b 행 「편집 겹치기보다 먼저 — 더해진 feature 도 AC3.1 편집의 대상이다」 · ③ PR #107 「추가 겹침을 편집 겹침 **앞에** 실행」 — AC 꼬리표 |
| `/// Dependencies are traced for **confirmed** features only` 에 덧붙인 「— an approved candidate, or a feature the person added themselves and confirmed (AC3.2)」 | ① 함수 본문 — 첫 질의(`decision = approved`)가 비면 `feature_add::confirmed_name` 을 부른다 · `confirmed_name` 의 pub 요약 「의존성 화면이 feature 를 인정하는 두 번째 경로」 · ③ PR #107 「`approved_candidate_name` 이 확정된 추가도 feature 로 인정」. doc 를 #107 이전 한 줄로 되돌렸다 |

결과: 1행의 줄 수·지문이 #107 이전 값 **590 / `9757b6b5…`** 으로 되돌아왔다 — 부모 `24f488d` 에서 같은
4파일을 재계산한 값과 바이트 동일(#107 이 이 4파일에 더한 것이 이 2행뿐이고 전건 제거했으므로).
`analysis.rs` 는 주석 제거 후 부모와 바이트 동일(stripper md5 `ea2b0ec7`).

## 증분 재판정 ⑧ — `#112` 가 `analysis.rs` 에 더한 5행 (2026-09-21 · `rct_20260921-0012`)

`#112`(`fc6d191`, 자매 모델 `tbm_feature-doc-docs-impl` 의 슬라이스 6c)가 `document()` 핸들러에
`crate::feature_delete::overlay(&state, &id, &mut content).await?;` 한 줄을 끼우며 그 위에 쓴 1행과,
`CandidateView` 에 `previously_deleted` 필드를 더하며 쓴 doc 4행을 판정해 **전건 제거**했다. 판정 절차는 ⑥·⑦
과 같고, 새 파일 쪽 판정은 [2026-09-21-feature-delete-axis.md](2026-09-21-feature-delete-axis.md) 에 있다.

| 줄 | 복원 경로 |
|---|---|
| 「지운 feature(AC3.3)는 편집 겹침 앞에서 가린다 — 가려진 feature 의 편집은 얹을 자리가 없다」 | ① `feature_delete::overlay` 의 doc — 이 패스가 「확정된 추가 겹침 **뒤**, 편집 겹침 **앞**에 불러야 한다 — 더해진 feature 도 지울 수 있고, 가려진 feature 의 편집은 얹을 자리가 없다」로 재작성해 순서 계약의 **정본**으로 삼았다(⑦ 이 `feature_add::overlay` 를 정본으로 두고 호출부 사본을 걷은 것과 같은 모양) · ② doc-tracker 6c 행 「확정된 추가 겹침 **뒤**, 편집 겹침 **앞**」 · ③ PR #112 — AC 꼬리표 |
| `/// AC3.3's "같은 feature가 다음 자동 분석에서 다시 발견되면 '이전에 거부된 항목입니다'로 표시된다", read back the same way: an earlier analysis of the same target deleted this same place and has not restored it. Information, never a decision — the candidate stays `undecided` until the reviewer acts.` 4행 | 앞 두 줄은 PRD AC3.3 의 축자 인용(② `docs/prd/`) · 「an earlier analysis of the same target deleted this same place and has not restored it」은 ① 이 필드를 채우는 `feature_delete::previous_deletion` 의 pub doc 「같은 대상(사용자·저장소·브랜치)의 앞선 분석에서 이 키를 지웠고 아직 되돌리지 않은 가장 최근 삭제」(유지, 정본) · 「Information, never a decision — stays `undecided`」는 ① 통합 테스트 fn 이름 `a_deleted_feature_rediscovered_by_the_next_analysis_is_marked_not_reactivated` 와 단정 메시지 「사용자 확인 없이 다시 활성화됐다」 · ② doc-tracker 6c 행 「표시일 뿐 결정이 아니다 … 결정은 `undecided` 그대로」 · ③ PR 본문. 바로 위 `previously_rejected` 의 doc 4행은 1차 패스가 「후보 검토 계약」으로 유지한 것이고 그 자리가 이 계약의 정본이므로, 「read back the same way」 라고 스스로 가리키는 사본은 걷었다 |

결과: 1행의 줄 수·지문이 #112 이전 값 **590 / `9757b6b5…`** 으로 되돌아왔다 — 부모 `bf48b45` 에서 같은
4파일을 재계산한 값과 바이트 동일(#112 가 이 4파일에 더한 것이 이 5행뿐이고 전건 제거했으므로).
`analysis.rs` 는 주석 제거 후 부모와 바이트 동일(stripper md5 `5ae2c305`).

## 증분 재판정 ⑨ — `#108` 이 이 축에 더한 26행 (2026-09-22 · `rct_20260922-0001`)

사람 PR **#108**(AC4.9 출력 언어 설정)이 `analysis.rs` 5 · `llm.rs` 18 · `worker_api.rs` 3 = **26행**을
들여왔다. 판정 맥락과 이 축 밖의 5파일은
[2026-09-22-output-language-axis.md](2026-09-22-output-language-axis.md)에 있다. **제거 15 · 유지 11.**

| 자리 | 판정 | 근거 |
|---|---|---|
| `analysis.rs` `AnalysisView.llm_language` doc 2행 (「The language this run's LLM prose is written in, fixed when it was triggered. `None` for a run triggered before the setting existed.」) | **제거 2** | 앞 문장은 필드 이름·타입의 재진술(①). 뒤 문장은 이 PR 이 일곱 자리에 적은 명제이고 **정본은 `settings.rs::analysis_language`** 로 골랐다 |
| `analysis.rs` `create` 의 복사 지점 주석 3행 (「Copied, not referenced: the stages of this analysis are claimed one gate at a time, and the language they write in must not move with the setting between them.」) | **유지 3** | 이 축의 **정본**. 불변식을 *만드는* 코드(설정을 읽어 INSERT 에 싣는 자리) 옆이고, 12차 패스의 잣대가 고른 자리다 |
| `llm.rs` `Language` enum doc 2행 → 1행 | **제거 1** | 꼬리 「the user's own setting, snapshotted onto each analysis when it is triggered」가 위 정본의 사본. 요약 1행만 남겼다 |
| `llm.rs` `Language` 의 「Only prose moves with it …」 4행 | **유지 4** | **판단 갈림 1.** 지시문이 *무엇을* 고정하는지는 바로 아래 `instruction()` 리터럴이 축자로 복원하지만, *왜*(뒤 단계와 화면이 그 값으로 조인한다 → 번역하면 조인이 깨진다)는 어디에도 없다 |
| `llm.rs` `instruction()` doc 2행 → 1행 | **제거 1** | 「Appended to every stage's system turn」은 `system_turn()` 의 `format!("{}\n{}", self.system, lang.instruction())` 이 그대로 말한다(①). 테스트 설계 제약(「One sentence per concern」)만 남겼다 |
| `llm.rs` `DEFAULT_LANGUAGE` doc 2행 → 1행 | **제거 1** | 「Matches the column default in the schema, which is the value an existing user row reads as」는 `0012_llm_language.sql` 의 `NOT NULL DEFAULT 'ko'` 가 말한다(① — SQL) |
| `llm.rs` `Ask::language` 필드 doc 2행 | **제거 2** | 「`None` adds no instruction」은 바로 아래 `system_turn()` 의 `None => Cow::Borrowed(self.system)` 갈래(①). 뒤 절은 일곱 벌 명제의 사본 |
| `llm.rs` `Ask::system_turn` doc 1행 | **제거 1** | 「the stage's own instruction, then the language line」은 본문 `format!` 이 축자로 적는다(①) |
| `llm.rs` `anthropic_body` doc 2행 | **제거 2** | **rustdoc 링크만의 교차 참조.** `openai_body` 의 doc 이 이미 「built apart from the call so a test can read it without a network」를 말한다 — 정책 본문: 링크를 위해서만 문장을 남기지 않는다. 증분 ③ 과 같은 판정 |
| `llm.rs` 테스트 `the_language_line_rides_the_system_turn_on_both_providers` doc 2행 | **제거 2** | fn 이름 + 본문의 네 단정(두 provider 동일 · `starts_with("sys\n")` · 언어 이름 포함 · `never translate`)이 그 문장 자체(①) |
| `llm.rs` `// The user turn is where the input lives; the language must not leak into it.` 1행 | **유지 1** | **판단 갈림 2.** 바로 아래 단정은 값이 *같다*고만 말하고 「새어 들면 안 된다」는 금지를 말하지 않는다 |
| `worker_api.rs` `ClaimView.llm_language` doc 3행 | **제거 3** | 세 문장 모두 사본이다 — 스냅숏 명제(정본 `analysis.rs`) · 일곱 벌 명제(정본 `settings.rs`) · 「워커는 지시를 더하지 않는다」(`llm.rs` 의 코드). 「see the column's note in the analysis module」은 링크 전용이고, 그 note 자체가 이 패스에서 제거된다 |

지문: **616행 `d1229629…` → 601행 `f9c76bd1802d88aebf6ad5833a32038d3445f4d374941f3e70796cd9f84b7547`**.

## 증분 재판정 ⑩ — #114(슬라이스 6d)가 `worker_api.rs` 에 연 +2행 · 2026-09-22

reconciler task `rct_20260922-0005`. 사람 PR **#114**(`aacd0b4`, AC3.5 충돌 처리)가 `submit_document` 의
`inherit` 호출 위에 2행을 더했다. **순 제거 1행 · 유지 1행**(2행 → 1행 재작성).

- 제거: 「인수 문서가 서는 순간이 직전 분석의 편집을 이어받을 자리다(AC3.5)」 — `docs/doc-tracker/2026-09.md`
  슬라이스 6d 행이 「재분석 문서가 저장되는 자리(`worker_api::submit_document`)에서 … **재생**한다」로 거의
  축자로 적는다(복원 경로 ②). `(AC3.5)` 꼬리표는 중복 유형 ③.
- **유지**: 「저장과 같은 요청 안에서 이어받아야 워커가 5단계를 `succeeded` 로 보고하기 전에 충돌이 서 있다」 —
  *왜 같은 요청 안이어야 하는가*는 **호출 순서가 만드는 계약**이고, doc-tracker 도 PR 본문도 이 순서를 적지
  않는다. 정책 본문이 유지 대상으로 이름 붙인 「워커 임대·큐 드레인·claim/lease 같은 동시성 계약」이다.

줄 수·지문: 601 / `f9c76bd1…` → (트리거) 603 → **602 / `87c1c5cd…`**. 유지분이 남아 #114 이전 값으로는
돌아가지 않는다. 맥락 [2026-09-22-conflict-axis.md](2026-09-22-conflict-axis.md).

## 증분 재판정 ⑪ — #123 이 `llm.rs` 트리거 테스트에 연 +4행 · 2026-09-22

reconciler task `rct_20260922-0006`. 자매 모델 `tbm_feature-doc-e2e-mock-policy` 의 PR **#123**
(`77158c2`, `rct_20260922-0001` — 13차 패스가 등재해 둔 `llm::tests` 프로세스 전역 env 레이스의 해소)이
`stub_llm_fail_trigger_fires_only_on_the_named_input` 에 **추가 6행 · 제거 2행(순 +4)** 을 들여왔다.
18차 패스(#124)는 머지 직전 착지라 줄 수·지문만 재고정하고 판정을 넘겼고, 이 패스가 그것을 받는다.
**순 제거 3행 · 유지 3행.**

| 자리 | 판정 | 근거 |
|---|---|---|
| 테스트 본문 `//` 3행 (「The needle is deliberately unlike any other prompt in this binary: / `an_ask()` is shared with the sibling tests, so it must not carry the / needle while the env is set.」) | **제거 3** | 결론 명제(「needle 은 다른 테스트의 프롬프트가 담을 수 없어야 한다」)가 열 줄 위 `///` doc 의 마지막 절과 **같은 명제의 두 벌째**다. 남는 사실 「`an_ask()` 는 형제 테스트와 공유된다」는 같은 `mod tests` 안에서 네 테스트가 `an_ask()` 를 부르는 것이 그대로 말한다(①). 경위는 PR #123 본문 4항이 `stub_answer()`·`an_ask()`·needle 겹침을 축자에 가깝게 다시 적는다(③) |
| `///` doc 의 재작성분 2행 → 3행 (「… The env writes stay inside this one test, but the variable is / process-global and `stub_answer` reads it for every Stub-mode ask in this / binary — so the needle must be one no other test's prompt can carry.」) | **유지 3** | 정책 본문의 유지 대상 두 항목에 정면으로 걸린다 — 프로세스 전역 env 를 `stub_answer` 가 **모든 Stub 모드 ask 에서** 읽는다는 **동시성 계약**이자, 공유 픽스처와 겹치면 형제 테스트가 429 로 죽는다는 **실패 모드의 함정**이다. 게다가 #123 이 **거짓이던 두 절을 정정한 자리**이므로 여기를 걷으면 원장이 닫은 결함의 이유가 코드 옆에서 사라진다 |

**정본을 어디에 둘 것인가** — 증분 재판정 ⑤·⑨ 의 잣대(「복제된 명제는 강제하는 코드 옆 한 벌만」)를 적용해
**`///` doc** 을 정본으로 골랐다. 두 자리는 같은 함수 안에 열 줄 거리로 붙어 있어 「코드 옆」이 둘 다 참이고,
갈림은 *무엇을 말하는가* 로 갈렸다 — `///` 는 **왜** 고유해야 하는지(전역 env · 공유 읽기)를 말하는 **유일한**
자리이고, `//` 3행은 그 결론만 되풀이한다. ⑤ 에서 `assert_strict_schema` 옆을 정본으로 고른 것과 같은 형태다.

**「애매하면 남긴다」와의 관계** — 제거한 3행이 담은 명제는 남긴 3행이 담은 명제의 **부분집합**이라, 지워서
어디에도 남지 않게 되는 지식이 0이다. 이 유형이 비대칭 비용 규칙에 걸리지 않는 이유다.

**등재된 유보의 해소도 이 패스가 기록했다.** 13차 패스가 「`llm.rs:494-499` 의 주석이 거짓이고 그 거짓이 실제
flake 를 덮는다 … 테스트 수정이 따라붙는 **별개 작업**이다. 여기서는 등재만 한다」로 남긴 항목을 #123 이 이미
수행했다. 원장 1곳과 판정 상세 3곳(13차 등재 · 14차 재확인 · 15차 flake 메모)에 **해소됨**을 덧붙였고, 발견
기록은 지우지 않았다 — 그 블록이 #123 의 착수 근거이고, 지우면 다음 감지가 같은 결함을 처음부터 다시 연다.

줄 수·지문: **이 판정 자신의 효과**는 `backend/src/llm.rs` 단독 **165 → 162 /
`85395e10f1b3950e5992fbfd46fe4547daef91308fdd0822dbdd9cbe3cd73897`** 이다(순 제거 3). 유지분(순 +1)이
남아 #123 이전 값 602/`87c1c5cd…` 로는 돌아가지 않는다.

**원장 1행의 절대값은 이 판정만으로 정해지지 않는다.** 이 패스를 준비하는 사이 자매 PR **#128**
(`c1b39ad`, 19차 패스)과 사람 PR **#121**(`b4a6b30`, AC1.5 — 끝난 단계만 다시 실행)이 차례로 착지했고,
#121 이 1행 범위의 `analysis.rs`(+5) · `worker_api.rs`(+21)에 **순 +26행**을 열었다. 그래서 머지 시점
트리의 1행 값은 **629 / `39b8ea030947a0fcf65b26057198a392c7dd8ec308857ce92c4c3c3f82ca7ba8`** 이고, 그중
**603행이 판정 완료 · 26행이 #121 발 미판정 증분**이다(판정하지 않고 다음 감지에 넘긴다 — 원장 1행의
「자매 착지 재실측」). 같은 착지가 원장 4·5·7·9행에도 순 +13행을 열어 이 패스가 **값만** 재고정했다.
전역 지문은 부모 `b4a6b30` 의 `lines=2649 files=135` / `224bb8ac…` →
**`lines=2646 files=135`** / `bb3c1bf975da6bb48b5ef4f31b0024e5804d3707a5ac7f8e7334ae15ca63bda2`.

**검증** — 문자열·문자 리터럴을 인식하는 stripper 로 주석을 전건 걷어낸 뒤 부모 `b4a6b30` 과 바이트 비교해
`backend/src/llm.rs` **IDENTICAL**(비주석 무접촉 · `git diff` 도 `//` 3행 삭제뿐). 리베이스한 트리에서
`cargo test --offline --release` **224 passed / 0 failed**(#121 이 더한 2건 포함),
`llm::` 모듈만 5회 반복해도 전건 통과 — 13차 패스가 등재한 「15회 중 1회 실패」가 #123 이후 재현되지 않는다.
`check-data-format-change.py` **`✅ 해당 없음`**(⇒ `review/manual-approval` = success) ·
`check-scenario-e2e.py` · `check-mockup-render.py` · `check-journey-mockup.py` rc=0 — 새 `passes/*.md` 를
만들지 않았으므로 허브 R9(`docs/index.html` 의 `Documents` 수·링크 집합)은 건드리지 않았다.


## 증분 재판정 ⑫ — #121 이 `analysis.rs`·`worker_api.rs` 에 연 +26행 · 2026-09-25

reconciler task `rct_20260925-0002`. 사람 PR **#121**(`b4a6b30`, AC1.5 — 끝난 단계도 그 단계만 다시 실행)이
이 행의 두 파일에 **추가 39행 · 제거 13행(순 +26)** 을 들여왔다. 20차 패스(#124)는 머지 직전 착지라 줄 수·지문만
재고정하고 판정을 넘겼고(원장 1행의 「자매 착지 재실측」), 22차 패스(#130)도 「미판정 증분 33행」으로 이어
넘겼다. 이 패스가 그것을 받는다. **순 제거 13행 · 유지 13행.**

⚠️ **25차 패스(#147)는 이 몫이 자기가 닫은 33행이라고 적었다 — 아니다.** 그 패스가 닫은 것은 원장 행 12 의
증분 14행과 행 밖 잔여 17행(합 31행)이고, 「미판정 증분 33행」은 행 1 의 26행과 행 4 의 7행이었다. 그 트리
(`39ee044`)에서 **행 1 은 여전히 629 / `39b8ea03…` 로 재현되고 #121 의 주석 39행이 파일에 그대로 있다**
(실측 — `grep -c 'Later stages are deliberately' backend/src/analysis.rs` = 1). 두 수가 겹쳐 보인 것은
**지문 재현을 판정의 증거로 읽었기 때문**이다. 원장의 「읽는 법」이 적은 대로 재현은 *그 줄이 판정을 받았음*을
뜻하지 않으며, 이 행이 다섯 트리에 걸쳐 초록인 채 열려 있었던 것이 그 증거다. 원장 합계 문단의 해당 문장은
지우지 않고 **실측에 맞춰 정정**했다(발견 기록을 지우면 다음 감지가 같은 오독을 처음부터 다시 한다).

> **유입량은 순증이 아니라 추가분으로 센다.** 원장이 적은 +26 은 **순증**이고, 판정 표면은 #121 이 **새로 쓴
> 39행**이다(제거된 13행은 이미 사라졌다). 아래 표는 그 39행을 자리별로 판정한 것이고, 「순 제거 13행」은
> 그 결과를 순증 축으로 환산한 값이다.

| 자리 | 판정 | 근거 |
|---|---|---|
| `analysis.rs` 모듈 머리 `//!` 라우트 목록 2행 (「`POST /api/analyses/{id}/stages/{key}/retry` — re-run one *finished* stage and / only that one (시나리오 6·8).」) | **유지 2** | 제자리 수정분이라 새 명제가 아니다. 같은 목록의 형제 항목 여섯 개를 앞선 패스들이 모두 유지로 닫았고(바로 위 `GET /api/analyses/{id}` 항목의 `(test/01 시나리오 5)` 꼬리표까지 포함), 이 한 칸만 걷으면 라우트 목록 자체가 깨진다 |
| `retry_stage` 요약 2행 → 1행 재작성 | **순 제거 1** | AC 조항 축자 인용 `"실패했거나 완료된 단계는 그 단계만 다시 실행할 수 있다"` 는 `docs/prd/01-analysis-pipeline.md` AC1.5 설명의 축자(②)이고, **같은 파일 271행의 오류 문자열** `"실패했거나 완료된 단계만 다시 실행할 수 있습니다."` 가 사용자에게 그대로 말한다(① — 정책 본문 「오류 문자열이 이미 말하는 것」). `test/01 시나리오 6·8` 은 꼬리표(②). 첫 절(「Re-runs one finished stage — failed *or* succeeded — and nothing else」)은 doc 요약 1줄로 남겼다 |
| `analysis.rs` 「Later stages are deliberately **not** invalidated …」 4행 + 딸린 `///` 1행 | **제거 5** | **세 벌째**다: ⑴ 같은 doc 세 줄 위 224–225행이 「Sibling stage rows are not touched, which is what keeps already-finished work (and its measured detail) intact」로 이미 말하고, ⑵ 본문 SQL 이 `WHERE analysis_id = ? AND key = ?` 로 한 행만 되돌리며(①), ⑶ PRD AC1.5 **검증 방법**이 「끝난 단계도 그 단계만 다시 실행할 수 있으며, 이때 **뒤 단계의 결과와 사용자가 내린 승인·결정은 그대로 유지된다**」로 축자에 가깝게 적는다(②) |
| `worker_api.rs` `cross_cutting_document` 필드 doc 3행 → 1행 | **순 제거 2** | 「carrying it is what lets the re-run leave stage 2 untouched」 는 `offered_stages` doc 의 「When the document is stored the claim carries it instead (`crossCuttingDocument`), which is what lets re-running stage 3 leave stage 2 untouched」 와 **같은 명제의 두 벌째**다. `(AC1.5)` 는 꼬리표(②) |
| `worker_api.rs` `Gates` 필드 doc 4행 | **제거 4** | 네 필드 이름(`strategy_approved`·`candidates_approved`·`acceptance_pending`·`landscape_stored`)이 각 문장의 술어를 그대로 담고(①), 「stage 4's gate」·「stage 5's gate」는 `offered_stages` 본문의 `if gates.strategy_approved && !done(FEATURE_CANDIDATES)` 류가 축자로 말한다(①). `(AC1.3)` 은 AC 꼬리표(②), `([`acceptance_pending`])` 는 **rustdoc 링크만의 교차 참조**(증분 재판정 ③·④ 의 선례). `pub struct Gates` 자신의 요약 1행은 유지 |
| `worker_api.rs` `stored_landscape` 요약 1행 | **제거 1** | 비공개 fn 이름 `stored_landscape` 와 반환 타입 `Result<Option<serde_json::Value>>` 가 「Stage 2's stored document, when there is one」을 축자로 말한다(①). doc 주석 유지 규칙은 `pub` 항목에만 걸린다 |
| `worker_api.rs` `offered_stages` 규칙 doc 15행 (규칙 진술 4 · `cross_cutting` 5 · Stage 5 6) | **유지 15** | 이 자리가 **정본**이다 — 같은 파일 `executable_stages` doc 이 「The rule is spelled out in [`offered_stages`]」로 독자를 여기로 보내고, 위 두 제거가 성립하는 근거가 바로 이 정본의 존재다. 담은 것도 복원 경로 밖이다: 「끝난 단계가 `pending` 으로 돌아오는 것은 사람이 다시 실행을 요청했을 때뿐」은 `analysis.rs::retry_stage` 와 이 함수 **사이의** 계약이라 어느 한쪽 코드로도 복원되지 않고, 「stage-2 문서가 없으면 3단계가 계획할 대상 없이 도착해 조용히 아무것도 안 한다」는 **실패 모드의 함정**, 「`pending`, not "not succeeded" — 실패한 5단계는 남이 요청한 재실행에 얹혀 가지 않는다」는 **동시성 계약**이다(정책 본문 유지 대상). 「애매하면 남긴다」의 비대칭 비용 조항을 적용했다 |

**정본을 어디에 둘 것인가** — 증분 재판정 ⑤·⑨·⑪ 의 잣대(「복제된 명제는 **강제하는 코드 옆** 한 벌만」)를
그대로 적용했다. 이번 창의 중복 셋은 모두 「규칙을 *정하는* 자리」와 「그 규칙을 *쓰는* 자리」의 쌍이었고,
정본은 언제나 정하는 쪽(`offered_stages`)이다. 쓰는 쪽(필드 doc·핸들러 doc·테스트)이 같은 문장을 다시 적으면
규칙이 바뀔 때 그쪽만 낡는다.

**「애매하면 남긴다」와의 관계** — 제거한 13행이 담은 명제는 모두 남긴 자리(정본 15행 · 요약 1행 · 라우트 목록
2행)의 **부분집합**이라, 지워서 어디에도 남지 않게 되는 지식이 0이다.

줄 수·지문: `analysis.rs` 249 → 243 · `worker_api.rs` 174 → 167. 행 1 전체는
**629/`39b8ea03…` → 616/`df4a5627c1acc0ae760914c1362c336a461e0a3670e050917ed36c59352a7696`** (순 제거 13).
603 판정 완료 + 유지 13 = **616 전건 판정 완료**, 이 행의 미판정 증분은 **0** 이 됐다.

**검증** — 세 파일 모두 주석 줄을 걷어낸 나머지를 부모 `39ee044` 과 md5 비교해 **IDENTICAL**
(`analysis.rs` `288aae8bec5423951e40cf52630383e2` · `worker_api.rs` `221faecd0dcccd31062aff4f4def4962`).
크레이트 어디에도 `missing_docs` 가 없고(`grep -rn missing_docs backend/` 0건 — `Gates` 의 `pub` 필드 doc 을
걷어도 경고가 없다), 이 세 파일에 doctest(```` ``` ```` 펜스)가 **0건**이라 `cargo test --release` 의 대상
집합이 바뀌지 않는다. CI 의 `cargo` job 은 `cargo test --profile ci` 하나이고 clippy·rustdoc 게이트는 없다.

## 보류분 집행 — case 라벨 3행 (2026-09-25 · `rct_20260925-0008`)

31차 패스. **판정이 아니라 집행이다.** 이 절은 위 「제거 후보로 지목하되 이번에 손대지 않은 것」이
명시 인계한 몫을 그대로 받아 닫는다.

### 무엇을 집행했나

`backend/src/llm.rs` `mod tests` 의 `stub_llm_fail_trigger_fires_only_on_the_named_input` 안 case 라벨 3행:

```
// Env unset: nothing changes.
// Env set and the input carries the needle: real-shaped provider failure.
// Env set but the input misses the needle: still the deterministic answer.
```

**판정은 18차 패스의 것을 그대로 쓴다 — 다시 내리지 않는다.** 「바로 아래 두세 줄(`remove_var` /
`set_var` + 단언)이 그대로 말하는 절 제목 — 중복 유형 ④」가 그 판정이고, 이 패스는 거기에
근거를 더하지 않는다. 같은 줄에 두 판정이 서면 그 자체가 drift 이기 때문이다.

덧붙이는 실측은 하나뿐이고, 그것은 판정이 아니라 **판정이 여전히 성립하는지의 확인**이다 —
함수 이름 `stub_llm_fail_trigger_fires_only_on_the_named_input` 이 세 라벨이 나누는 명제를 축자로
담고 있어(①), 세 줄이 사라져도 각 블록이 무엇을 재는지는 코드에서 그대로 읽힌다.

### 왜 이번에 집행할 수 있었나 — 보류 사유의 소멸

보류는 **판정이 아니라 경합**이었다. 막고 있던 것은 열린 draft PR **#49** 하나였고,
위 문단이 「#49 가 머지·폐기된 뒤 이 범위를 다시 열 때 **이 3행이 첫 항목**이다」라고
수신자까지 지정해 인계했다.

**#49 는 `2026-09-18T15:40:05Z` 에 머지됐다**(`gh pr view 49` 실측). 그 뒤 22~29차 **여덟 패스**가
지나갔지만 아무도 이 인계를 집지 않았다 — 사람 게이트도 아니고 무인 집행이 가능한데 조건이
풀린 채 **7일**을 굶었다. 등재가 있다고 닫히는 것이 아니라는 것, 그리고 등재문이 *무엇을 참이라
단언하는지*(「#49 가 막고 있다」)를 실측해야 한다는 것이 이 자리의 교훈이다.

### 수치

| | 집행 전 | 집행 후 |
|---|---|---|
| 원장 1행 주석 줄 수 | 616 | **613** |
| 원장 1행 지문(행 규약 = 개행 포함) | `df4a5627c1acc0ae760914c1362c336a461e0a3670e050917ed36c59352a7696` | **`666abdf8f6db8eaa0b9e8ef84adb9000ebbc0d25278617552ad6e4051ac803b6`** |
| `llm.rs` diff | — | 주석 **3줄 삭제뿐** (`1 file changed, 3 deletions(-)`) |

이 두 값은 **착지 순서에 불변**이다 — 이 창에 열려 있던 #154 · #148 과 이미 착지한 #155 중
1행의 네 파일(`analysis.rs` · `llm.rs` · `worker_api.rs` · `llmkey.rs`)을 건드리는 것이 하나도 없다.

### `cargo test --release` 무영향 — 툴체인 없이 닫는 3줄

1. **주석을 걷어낸 코드가 바이트 동일**하다. 비주석 diff 0줄
   (`git diff -U0 -- backend/src/llm.rs | grep -E '^[+-]' | grep -v '^[+-][+-]' | grep -vE '^-\s*//'` → 0).
2. **`missing_docs` lint 이 없다** — 삭제해도 경고가 서지 않는다.
3. 삭제 대상은 전부 `//` 줄 주석이고 `llm.rs` 에 **doctest 코드펜스가 0건**이라 doctest 표면에 닿지 않는다.

### 범위 밖(후속)

29차 패스가 노출한 미집행 잔량 중 이 3행 **말고는 아무것도 집지 않았다**. 벽의 종류가 다르기 때문이다.

- **D6 몫** — `tools/check-data-format-change.py` 12~15행. 판정기 자신을 건드리므로
  `review/manual-approval` 이 자동으로 붙지 않고 사람 승인이 필요하다. `:21-31` 은 워크플로 헤더의
  정본 지정 포인터라 남긴다.
- **D1 몫** — `.sql` 15~19행(`0014` 의 9~11행 + 27차 패스가 등재한 `0009`~`0013` 의 6~8행).
  운영 DB repair 가 선행해야 하므로 전용 PR 하나로 묶어야 한다.
- **#155 가 다시 연 ⑴ 21행 · ⑵ 22행 + #156 이 더 연 ⑵ 11행**(합 **⑴ 21 · ⑵ 33**) — 이 패스는 세지 않는다.
  30차 패스(#154)는 `a473767` 로 이미 착지했고, #156(`84d312a`)이 `frontend/src/index.css` +1(행 10) ·
  `tools/check-mockup-render.py` +10(행 11) 으로 연 11행은 **세는 주체가 없다** — 재감지가 후속 task 로 연다.

한 덩어리로 묶지 않은 이유는 원장 자신이 이름 붙인 실패 모양 때문이다 — 무인으로 닫히는 3행이
운영 일정에 묶이면 영구 이월된다.

## 증분 재판정 ⑭ (2026-09-26 · `rct_20260926-0002`) — 슬라이스 7d 가 연 21행

- **기준 트리**: 부모 **`0522f33`**(main, #172 착지 직후)
- **유입원**: PR **#166**(`ca9e14a`, 슬라이스 7d — AC4.6 관측 가능성·비용 가시성, 모델 `tbm_feature-doc-docs-impl`)
- **PR**: #NNN (37차 패스)

원장 행 1 의 줄 수 칸은 #166 착지로 **623 → 638** 이 돼 있었다. 증가분은 **gross 21 / net 15** —
`Counter(now) − Counter(old)` 로 21, 역방향으로 6(개작으로 사라진 옛 문면)을 세어 확인했다.
**순증만 보면 표면을 5줄 과소 진술한다.**

| 자리 | 문면(요약) | 판정 | 근거 |
|---|---|---|---|
| `analysis.rs` 모듈 doc ¶ 3(+ 앞의 `//!` 1) | 「이 뷰들에 두 비용이 실린다 — `est_*` 는 pre-flight 추정, `spend` 는 실제 보고값(AC4.6, [`crate::usage`])」 | **제거 4** | 같은 파일 안에 **세 벌**이 있다(모듈 doc · `spend` 필드 doc · `Estimate` doc). 정본은 **값을 고르는 자리**인 필드·구조체 doc 이고 모듈 doc 은 개관 서사다 · 딸린 `//!` 빈 주석 행은 이 행이 1차 판정에서 이미 「빈 주석 행」을 제거로 닫은 유형 |
| `analysis.rs` `spend` 필드 doc 4 → 2 | 「지금까지 실제로 든 비용(AC4.6) — 위의 `est_*` 는 저장소 크기에서 나온 추측이고 화면은 이쪽을 띄우며 추정은 그 결정의 근거로 남는다」 | **제거 2 · 유지 2** | 유지: `est_*` 와 `spend` 를 **한 구조체 안에서 골라 쓰는 사람**이 읽는 자리이고, 바꿔 써도 타입이 같아 **조용히** 틀린다. 제거: 「화면은 이쪽을 띄운다」는 ① `AnalysisProgress.tsx` 의 `cost-so-far` 가, AC 꼬리표는 ② 가 갖는다 |
| `analysis.rs` `Estimate` doc 2 → 1 | 「— 그리고 이후 수정되지 않는다: 실제로 쓴 값은 따로 측정된다([`crate::usage`])」 | **제거 1 · 유지 1**(제자리 재작성) | 유지: 「never revised afterwards」는 `from_size_kb` 를 재호출하려는 사람을 막는 가드로 앞 문장에 합쳤다. 제거: 뒷문장은 **rustdoc 링크만을 위한 문장**(README 명시 금지) |
| `llm.rs` `Answer` doc 8 → 5 | 요약 + 「토큰·`calls` 가 AC4.6 이 드러내는 비용 집계의 실측 면이고 화면이 보고하는 숫자는 여기서 온다」 + 「`calls` 가 상수가 아니라 필드인 이유」 | **제거 3 · 유지 5** | 제거: 「화면의 숫자가 어디서 오는가」는 `analysis.rs`·`api.ts`·`usage.rs` 의 네 번째 벌이고 AC 꼬리표는 ② 다. 유지: **이 축의 정본** — `Answer.calls` 를 상수 1 로 접으려는 사람이 읽는 자리이고, 그 순간 병합 stage 가 호출 하나를 **조용히** 잃는다(신설 행 34 의 「정본 지정」 표) |
| `worker_api.rs` `DocumentReq.calls` doc 2 → 1 | 「이 문서가 든 제공자 호출 수. 옛 워커도 0 이 아니라 흔한 경우를 보고하게 기본값 1」 | **제거 1 · 유지 1** | 제거: 첫 문장은 필드 이름·타입(`calls: i64`) 재진술. 유지: 버전 호환 계약이고 `#[serde(default = "one_call")]` 을 지우려는 사람의 편집 지점이다 |
| `worker_api.rs` 로그 앞 인라인 2 | 「운영자가 보는 자리 — 단계별 호출·토큰이 로그에도 남는다(AC4.6). 화면은 합쳐진 뒤를 읽으므로 어느 단계가 얼마를 썼는지는 여기서만 보인다」 | **제거 2** | ② `docs/doc-tracker/2026-09.md:128` 「단계별 내역은 `worker_api::submit_document` 의 `llm usage recorded` 로그가 남긴다」 축자 · ③ #166 같은 문장 · ① 바로 아래 `tracing::info!` 의 `stage`·`calls`·`model` 필드 |

**순 제거 13행**(gross 21 중 8행 유지) · 줄 수·지문 623 → **625 / `6f7c80da…`**

---

## 원장에서 옮겨 온 증분 재판정 기록 (2026-09-26 형식 이전)

아래는 `ledger.md`의 결과 칸에 쌓여 있던 증분 재판정·정정 기록을 **문면 그대로** 옮긴
것이다. 형식 이전(템플릿 「원장 형식」)이 원장에 표와 「읽는 법」만 두기로 하면서, 각 행의
경위는 그 행의 패스 파일로 돌아왔다. 옮기면서 한 글자도 고치지 않았고 판정을 새로 하지
않았다 — 행을 가리키는 순번도 당시 표기 그대로다.

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ①**(2026-09-18): #55가 더한 `llm.rs` 21행은 stub↔real 충실도 경계라 **전건 유지**, 제거 후보 3행(테스트 case 라벨)은 `llm.rs`가 #49와 경합이라 보류 — **해소됨**(2026-09-25 · 31차 패스 · `rct_20260925-0008`): #49 가 `2026-09-18T15:40:05Z` 에 머지돼 경합이 소멸했으므로 그 3행을 **집행 제거**했다(판정은 18차 패스의 「중복 유형 ④」를 그대로 쓴다 — 재판정 아님) · 616/`df4a5627…` → **613/`666abdf8…`** — [passes/2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 「보류분 집행」

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ②**(2026-09-18): #43이 `analysis.rs`·`worker_api.rs`에 더한 40행을 판정해 **순 제거 10행**(AC 조항 재진술 · 호출자·구조체 본문 재진술 · rustdoc 링크만의 교차 참조) · 유지 30행(리스 계약 · 게이트는 큐의 성질 · `acceptance_pending`의 술어 함정 · `work_remains`의 경합)

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ⑤**(2026-09-21): #93이 `llm.rs`에 더한 `assert_strict_schema` doc 5행을 **명제 단위로** 판정해 **순 제거 2행**(함수 본문의 두 `assert` 재진술 · 단정 메시지가 이미 말하는 「optional 대신 nullable」과 스키마 리터럴 `["string","null"]` 재진술 — 5행 → 3행 재작성) · 유지 3행(`pub(crate)` 요약 1줄 · 「`required` 누락은 모델 실행 전 400」의 상류 거부 조건 · stub 은 스키마를 어디에도 보내지 않는다는 충실도 경계). **이 자리가 「required-but-nullable」 명제의 정본이다** — 강제하는 코드 `assert_strict_schema` 옆(4·6·8행의 같은 명제 사본 4행은 그래서 걷었다) — [passes/2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) · **미판정 증분 없음**

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ⑥**(2026-09-21): #92가 `analysis.rs` `document()` 에 더한 4행(승인 편집의 겹쳐 읽기와 `content_hash` 불변의 이유 — 0009 머리·doc-tracker·PR #92 의 **네 벌째**)을 **전건 제거** · 줄 수·지문은 #92 이전 값으로 바이트 동일 복귀 — [passes/2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 「증분 재판정 ⑥」

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ⑦**(2026-09-21): #107 이 `analysis.rs` 에 더한 2행(`document()` 의 추가 겹침 호출 위 1행 — `feature_add::overlay` doc 의 순서 계약이 정본 · `approved_candidate_name` doc 에 덧붙인 1행 — 함수 본문의 두 질의와 `confirmed_name` 요약)을 **전건 제거** · 줄 수·지문은 #107 이전 값으로 바이트 동일 복귀 — [passes/2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 「증분 재판정 ⑦」

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ⑧**(2026-09-21): #112 가 `analysis.rs` 에 더한 5행(`document()` 의 삭제 겹침 호출 위 1행 — 순서 계약을 `feature_delete::overlay` doc 으로 옮겨 정본화 · `CandidateView.previously_deleted` 의 AC3.3 PRD 축자 인용 4행 — `previous_deletion` pub doc·테스트 fn 이름·doc-tracker 6c 행의 사본)을 **전건 제거** · 줄 수·지문은 #112 이전 값 590/`9757b6b5…` 로 바이트 동일 복귀 — [passes/2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 「증분 재판정 ⑧」

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ⑨**(2026-09-22): 사람 PR #108(AC4.9 출력 언어)이 `analysis.rs` 5 · `llm.rs` 18 · `worker_api.rs` 3 = **26행**을 들여와 **제거 15 · 유지 11** — 한 명제의 일곱 벌 복제를 「정본을 어디에 둘 것인가」로 판정(스냅숏 계약의 정본 = `analysis.rs::create` 의 복사 지점 3행 **유지**, 「`None` = 설정 이전에 시작된 분석」의 정본 = `settings.rs::analysis_language`) · rustdoc 링크 전용 교차 참조 2행(`anthropic_body` → `openai_body`) · 단정·선언 재진술 · 판단 갈림 2건 유지(조인이 깨진다 4행 · 「언어가 user turn 에 새어 들면 안 된다」 1행) — [passes/2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 「증분 재판정 ⑨」 · 맥락 [passes/2026-09-22-output-language-axis.md](2026-09-22-output-language-axis.md)

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ⑪**(2026-09-22 · `rct_20260922-0006`): 그 4행(추가 6 · 제거 2)을 명제 단위로 판정해 **순 제거 3행** — 테스트 본문의 `//` 3행(「The needle is deliberately unlike any other prompt in this binary … `an_ask()` is shared with the sibling tests」)은 열 줄 위 `///` doc 의 마지막 절과 **같은 명제의 두 벌째**이고, 남는 사실(`an_ask()` 가 공유 픽스처)은 같은 `mod tests` 의 네 테스트가 그것을 부르는 코드가 복원하며(①) 경위는 PR #123 본문 4항이 축자에 가깝게 다시 적는다(③) — 증분 재판정 ⑤·⑨ 의 「복제된 명제는 강제하는 코드 옆 한 벌만」에서 **정본을 `///` doc** 으로 골랐다 (두 자리가 같은 함수 안 열 줄 거리라 「코드 옆」은 둘 다 참이고, `///` 만이 *왜* 고유해야 하는지를 말한다) · **유지 3행**은 그 `///` 의 재작성분(2행 → 3행) — 프로세스 전역 env 를 `stub_answer` 가 **모든 Stub 모드 ask 에서** 읽는다는 **동시성 계약**이자 **실패 모드의 함정**(정책 본문의 유지 대상 두 항목)이고, #123 이 거짓이던 두 절을 정정한 자리다 · 유지분(순 +1)이 남아 #123 이전 값 602/`87c1c5cd…` 로는 돌아가지 않는다 · 606/`6c3ba120…` → **603/`1e39fbe5dc8715e585a1517d395aba3ca1f5be09b65c81faee52ea05ae514f7f`** — [passes/2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 「증분 재판정 ⑪」  · **자매 착지 재실측**(2026-09-22 · #121): 20차 패스의 머지 직전에 사람 PR **#121**(`b4a6b30`, AC1.5 — 끝난 단계만 다시 실행)이 착지해 이 행에 **순 +26행**(`analysis.rs` +5 · `worker_api.rs` +21)을 열었다. 판정이 아니라 **머지 시점 트리에서의 줄 수·지문 재고정**이고(「행 지문을 재현하는 법」 — 개행 포함 해시), 값은 603/`1e39fbe5…`(증분 재판정 ⑪ 직후) → **629/`39b8ea030947a0fcf65b26057198a392c7dd8ec308857ce92c4c3c3f82ca7ba8`** 다. 그 26행은 **판정하지 않고 다음 감지에 넘긴다** — **해소됨**(2026-09-25 · `rct_20260925-0002` · 아래 증분 재판정 ⑫).

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ⑫**(2026-09-25 · `rct_20260925-0002`): 그 26행(#121 이 **새로 쓴 39행** · 제거 13행)을 **명제 단위로** 판정해 **순 제거 13행 · 유지 13행** — `analysis.rs` 6(AC1.5 조항 축자 인용과 `test/01 시나리오` 꼬리표를 걷어 요약 2행 → 1행 · 「뒤 단계는 무효화되지 않는다」 4행과 딸린 `///` 은 **세 벌째**라 전건 제거: 같은 doc 세 줄 위의 「Sibling stage rows are not touched…」 · 본문 SQL 의 `AND key = ?` · PRD AC1.5 **검증 방법**의 「뒤 단계의 결과와 사용자가 내린 승인·결정은 그대로 유지된다」) · `worker_api.rs` 7(`Gates` 네 필드 doc 은 필드 이름과 `offered_stages` 본문의 `if` 가 축자로 말하는 위에 `(AC1.3)` 꼬리표와 **rustdoc 링크만의 교차 참조**가 얹혀 제거 4 · `cross_cutting_document` 필드 doc 3행 → 1행 은 `offered_stages` doc 과 같은 명제의 두 벌째 · 비공개 `stored_landscape` 요약 1행은 이름과 `Option` 반환 타입이 축자) · **유지 13행** — `offered_stages` 의 규칙 doc 15행이 파일 전체가 가리키는 **정본**이고(같은 파일 `executable_stages` doc 의 「The rule is spelled out in [`offered_stages`]」), 「끝난 단계가 `pending` 으로 돌아오는 것은 사람이 다시 실행을 요청했을 때뿐」은 `retry_stage` 와 이 함수 **사이의** 계약이라 어느 한쪽 코드로도 복원되지 않으며, 「stage-2 문서가 없으면 3단계가 계획할 대상 없이 도착해 조용히 아무것도 안 한다」와 「`pending`, not "not succeeded"」는 정책 본문의 **실패 모드의 함정**·**동시성 계약**이다(「애매하면 남긴다」) · 모듈 머리 `//!` 라우트 목록 2행도 형제 항목 여섯 개가 모두 유지로 닫힌 목록의 한 칸이라 유지 · 603 판정 완료 + 유지 13 = **616 전건 판정 완료**(이 행의 미판정 증분 0) · 629/`39b8ea03…` → **616/`df4a5627c1acc0ae760914c1362c336a461e0a3670e050917ed36c59352a7696`** — [passes/2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 「증분 재판정 ⑫」

### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ⑭**(2026-09-26, 37차 패스, `rct_20260926-0002`): #166(슬라이스 7d — AC4.6)이 연 **gross 21 / net 15** 행을 판정해 **순 제거 13행** · 유지 8행 — 제거는 `analysis.rs` 의 **「두 비용이 실린다」 모듈 doc 3 + 딸린 빈 `//!` 1**(같은 파일에 모듈 doc·`spend` 필드 doc·`Estimate` doc **세 벌**이고 정본은 값을 고르는 필드 쪽 · 빈 주석 행은 이 행 1차 판정이 이미 닫은 유형) · `spend` 필드 doc 의 「화면은 이쪽을 띄운다」 2 · `Estimate` doc 의 **rustdoc 링크만을 위한 문장** 1 · `llm.rs` `Answer` doc 의 「화면이 보고하는 숫자는 여기서 온다」 3(네 번째 벌) · `worker_api.rs` 의 필드 이름 재진술 1 + **로그 앞 인라인 2**(② `doc-tracker/2026-09.md:128` 「단계별 내역은 `worker_api::submit_document` 의 `llm usage recorded` 로그가 남긴다」 축자 · ③ #166 같은 문장 · ① 바로 아래 `tracing::info!` 의 `stage`·`calls`) · **유지**는 `spend`/`est_*` 판별식 2(한 구조체 안에서 바꿔 써도 타입이 같아 조용히 틀린다) · `Estimate` 의 「never revised」 1 · **`llm.rs` `Answer.calls` 5 — 「한 행이 두 호출을 대표한다」 명제의 정본**(상수 1 로 접으면 병합 stage 가 호출을 조용히 잃는다) · `worker_api.rs` 의 옛 워커 기본값 계약 1 · **비주석 diff 0줄** — [passes/2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 「증분 재판정 ⑭」


### 원장 행 1 — `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일)

**증분 재판정 ⑮**(2026-09-26, 40차 패스, `rct_20260926-0006`): #179(`9a32b76`, 슬라이스 7f)가 `StageView` 에 `spend` 필드를 실으며 연 **+6행**(gross 6 · net 6)을 판정해 **순 제거 4행 · 유지 2행**.

**제거 4 — `spend` 필드 doc.** ⑴ `What this stage spent (AC4.6).` 의 앞부분은 ① 필드 이름 `spend` 와 타입 `crate::usage::Spend` 의 재진술이고 `(AC4.6)` 꼬리표는 **③** PR #179 본문 머리(`**task**: rct_20260926-0004 … 슬라이스 **7f**` · `## gap — AC4.6 검증 방법의 「단계별 비용」`)이자 **④** 커밋 제목(`AC4.6 검증 방법의 「단계별 비용」 — … (슬라이스 7f, rct_20260926-0004) (#179)`) 축자다. ⑵ `Filled after the query from [\`crate::usage::by_stage\`] rather than joined in` 은 **①** — 바로 아래 줄이 `#[sqlx(skip)]` 이고 `load_detail` 이 쿼리 뒤에 `for stage in &mut stages { … }` 로 채운다. ⑶ `the stage rows and the document rows are a 1:1 axis but not a 1:1 join (a stage that wrote no document has no row there, and must read 0 rather than vanish)` 는 **②** `docs/doc-tracker/2026-09.md:738` 이 축자로 소유한다 — 「`StageView` 에 `#[sqlx(skip)]` 로 채워 문서 없는 단계는 0 을 읽는다(조인으로 묶으면 그 단계가 사라진다)」. **③** 도 같다(#179 ⑵ 「조인이 아니라 `#[sqlx(skip)]` + 쿼리 뒤 채움이다: 문서를 쓰지 않은 단계는 조인에서 **사라지는** 대신 0 을 읽어야 한다」).

「조인으로 바꾸지 말라」는 금지형이지만 **가드가 아니다** — 판별식(어겼을 때 조용히 깨지는가)에 걸어 보면 조인으로 바꾸는 순간 `backend/tests/usage.rs` 의 `per_stage_spend_is_attributed_to_the_stage_that_spent_it` 이 `fetch`·`discovery_strategy`·`feature_candidates` 를 `bucket()` 으로 찾다 `panic!("단계 {key} 가 응답에 없다")` 로 **즉시 붉어진다**. 조용하지 않으므로 서사 쪽이다(37차 패스가 세운 세 갈래).

**유지 2(판단 갈림)** — `// One read for the whole pipeline, not one per stage: there are five stages, and / // a stage with nothing charged to it keeps the zero it was built with.` 는 같은 판별식에서 반대 결론이 난다: 단계마다 한 번씩 질의하도록 바꿔도 **어떤 테스트도 붉지 않는다**(값이 같다). ③ 히트(#179 ⑵ 「단계는 다섯이라 N+1 이 아니다」)가 있지만 **PR 본문은 편집 지점에서 읽히지 않는다** — 37차 패스가 `usage.rs` 모듈 doc ¶3(「카운터 열로 두지 않는다」)을 같은 근거로 유지한 선례를 그대로 적용한다. 「there are five stages」는 그 자체로는 ① (`pipeline::STAGES`)이지만 N+1 논거의 전제라 떼면 가드가 근거를 잃으므로 함께 남긴다.

**④ 축 실측**(이 증분에 대해): 이 레포는 squash 머지라 최근 101커밋 중 `Co-authored-by` 외 본문이 있는 것이 **0건**이고, `9a32b76` 도 제목뿐이다 ⇒ ④ 로 복원되는 것은 제목의 AC·슬라이스 꼬리표뿐이며 **유지 2행에 대한 ④ 히트는 0건**이다.

**값**: 625(#178 head) → 631(#179 유입) → **627 / `f2842208d3ea…`**. 비주석·비공백 diff **0줄**. **이 행에 미판정 증분 없음** — [passes/2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 「증분 재판정 ⑮」
