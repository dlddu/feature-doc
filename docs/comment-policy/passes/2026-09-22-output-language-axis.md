# 판정 상세 — 출력 언어 설정 축 (AC4.9 가 들여온 새 파일 3개) · 2026-09-22

reconciler task `rct_20260922-0001`(모델 `tbm_feature-doc-comment-redundancy`). 사람 PR **#108**
(`b1c4efe`, AC4.9 LLM 산출물의 출력 언어 사용자 설정)이 판정 대상 범위에 주석 **순 +86행 / 제거 0행**을
들여왔다(파일 12개). 그중 마이그레이션 `0012_llm_language.sql` **14행**은 본문 「적용된 마이그레이션」 절의
사람 게이트(전용 PR · 수동 repair · 사람 승인) 몫이라 **이 패스가 다루지 않는다** — 잔여 `0009`(30) ·
`0010`(30) · `0011`(25) 와 합쳐 **99행 한 배치**로 다뤄야 repair 가 한 번에 끝난다.

나머지 **72행**을 한 패스로 판정했다 — **새 파일 3개 / 31행**(이 파일의 본문)과 **판정 완료 원장 행 안으로
들어온 증분 8파일 / 41행**(원장 1·5·10·17·18행 — 각 원래 패스 파일에 절을 더했다, 아래 「증분」 절).

## 범위와 결과

| 파일 | 유입 | 제거 | 유지 | 비고 |
|---|---|---|---|---|
| `backend/src/settings.rs` | 15 | **5** | 10 | 모듈 머리 6 → 2(스냅숏 계약 3 + 빈 `//!` 1 제거) · `SettingsView.llm_language` 의 rustdoc 링크 전용 1 · `analysis_language` 꼬리 1행 축소(±0) |
| `backend/tests/settings.rs` | 4 | **4** | 0 | 테스트 fn doc 2건(각 2행) — 둘 다 fn 이름과 바로 아래 단정이 그대로 복원한다 |
| `e2e/tests/sc04-14-llm-output-language.spec.ts` | 12 | **5** | 7 | 단정 재진술 한국어 한 줄 주석 5 · 머리 블록 7행은 **유지**(9차 패스 `sc01-02` 선례) |
| **합** | **31** | **14** | **17** | 범위 지문 **17 / `9458795199cc7a8cc2c0f5ea12240c6657343d398d73fc9b9854d3edc64faba6`** |

증분 8파일(원장 행 안): `analysis.rs` 5 → 제거 2 / `llm.rs` 18 → 제거 10 / `worker_api.rs` 3 → 제거 3 /
`bin/worker.rs` 7 → 제거 7 / `api.ts` 2 → 제거 2 / `RegisterLlmKey.tsx` 4 → 제거 1 / `doc_edit.rs` 1 → 제거 1 /
`feature_add.rs` 1 → 제거 1 — **41행 중 제거 27 · 유지 14**. 합쳐 **순 제거 41행**.

**diff 기준**: 11파일 45행 삭제 · 4행 삽입(재작성 — `llm.rs` 3자리, `settings.rs` 1자리). 지문 감소(41)와
diff 삭제 줄 수(45)가 갈리는 것은 재작성 때문이다.

## 이 축의 복원 경로 — 한 명제의 일곱 벌

#108 은 한 설계 판단을 **일곱 자리에 적었다**. 「`None`/`null` 은 설정이 생기기 전에 시작된 분석이다」가
`analysis.rs`(뷰 필드 doc) · `bin/worker.rs`(claim 필드 doc) · `worker_api.rs`(claim 뷰 doc) · `settings.rs`
(`analysis_language` doc) · `backend/tests/settings.rs`(테스트 fn doc) · `frontend/src/api.ts`(필드 JSDoc) ·
`llm.rs`(`Ask::language` 필드 doc) 일곱 벌이고, `0012_llm_language.sql` 머리가 여덟 벌째다. 원장 **12차 패스**가
네 벌까지 복제된 명제 여섯 건을 전부 「그 명제를 강제하는 코드 옆 한 벌」로 줄인 선례가 있으므로 같은 잣대를
썼다.

- **정본 = `backend/src/settings.rs::analysis_language`.** 설정↔분석의 의미를 소유하고, 이 값을 얻는 모든
  호출자(`doc_edit::propose` · `feature_add::draft` · claim 경로)가 지나는 한 함수다. 나머지 여섯 벌은
  전송 경계(뷰·claim·API 타입)의 재진술이라 제거했다. 꼬리의 「the caller then adds no instruction」은
  `llm::Ask::system_turn` 의 `None => Cow::Borrowed(self.system)` 이 그대로 말하므로(복원 경로 ①) 함께 걷어
  「`None` for an analysis triggered before the setting existed.」로 끝맺었다.

두 번째 복제 명제 「스테이지는 게이트마다 따로 claim 되므로 언어가 그 사이에 움직이면 안 된다 — 그래서 참조가
아니라 복사」는 `analysis.rs::create`(복사 지점) · `bin/worker.rs::language_for`(읽는 지점) · `settings.rs`
모듈 머리 세 벌이었다.

- **정본 = `analysis.rs::create` 의 복사 지점.** 불변식을 *만드는* 코드 옆이다(12차 패스의 잣대). 읽는 쪽
  `language_for` 는 본문이 문자 그대로 `job.llm_language` 를 읽고, 바로 위 `provider_for` 의 doc 이 같은 이유
  (「Shared by every LLM-backed stage so they cannot disagree about it mid-job」)를 이미 말하므로 재진술 5행을
  전부 걷었다. 모듈 머리 3행도 같은 재진술이며 「see the worker's `language_for`」는 링크 전용 교차 참조다.

## `backend/src/settings.rs` — 제거 5행

### 모듈 머리 `//!` 6행 → 2행 (제거 4)

- 유지: `//! Per-user preferences that shape what the pipeline produces rather than who may` /
  `//! run it. Today that is one value: the language LLM-written prose comes back in.` — 모듈 요약(정책 본문
  「doc 주석」 조항이 유지 대상으로 이름 붙인 `//!` 머리).
- 제거: 빈 `//!` 1행 + 「The setting is read once, when an analysis is triggered, and copied onto that analysis
  (`analysis::create`). Changing it therefore steers the *next* analysis and never rewrites one already under
  way — see the worker's `language_for`.」 3행 — 위 「정본」 판정대로 `analysis.rs::create` 의 사본이고, 꼬리는
  rustdoc 링크 전용 교차 참조다(정책 본문: 링크를 위해서만 문장을 남기지 않는다).

### `SettingsView.llm_language` doc 1행 (제거 1)

- 제거: `/// `"ko"` or `"en"` — the codes [`Language::parse`] accepts.` — 복원 경로 ①. `Language::parse` 가
  받는 코드는 그 함수의 `match` 가 축자로 적고(`"ko"` · `"en"`), 주석은 링크로 그 함수를 가리키기만 한다.
  원장 증분 ③ 이 같은 모양(「rustdoc 링크만의 교차 참조」)을 제거로 닫은 선례와 같다.

### `analysis_language` doc 5행 → 5행 (축소, ±0)

- 유지: 「The language one analysis was fixed to when it was triggered — what every LLM call made *for* that
  analysis writes in, including the ones the API makes directly (edit proposals, manual feature drafts), so
  their prose matches the document it lands in.」 — **이 축의 정본**이다. 「API 가 직접 거는 호출까지 이 언어로
  쓴다」는 명제는 `doc_edit`·`feature_add` 두 호출부의 한 줄 주석이 각각 되풀이하던 것이고(둘 다 제거), 함수
  이름만으로는 *왜* 분석의 언어인지가 복원되지 않는다.
- 축소: 꼬리 「existed: the caller then adds no instruction.」 → 「existed.」 — 호출자가 무엇을 하는지는
  `llm::Ask::system_turn` 의 `None` 갈래가 말한다(복원 경로 ①).

### 유지 — `llm_language` 의 폴백 사유 3행

「A value the code no longer recognises reads as the default instead of failing the screen: the row is the
user's, the vocabulary is the code's, and the code is the one that moved.」 — 코드는 `unwrap_or(DEFAULT_LANGUAGE)`
가 *무엇을* 하는지만 말한다. *왜* 실패가 아니라 기본값인지(행의 소유자와 어휘의 소유자가 다르고, 움직인 쪽은
코드다)는 어느 복원 경로에도 없다. 정책의 「애매하면 남긴다」 이전에, 애초에 복원되지 않는다.

## `backend/tests/settings.rs` — 제거 4행 (전건)

두 테스트 fn doc 이 각각 2행이고, 둘 다 **fn 이름과 바로 아래 단정이 그 문장 자체**다.

| 제거한 doc | 복원 경로 |
|---|---|
| 「The analysis is written in the language that was set when it was triggered — changing the setting afterwards steers the next analysis, not this one.」 | ① fn 이름 `an_analysis_keeps_the_language_it_was_triggered_with` + 본문이 `en` 으로 설정→enqueue→`ko` 로 변경→첫 건은 여전히 `en`·새 건은 `ko` 를 단정하고, 그 단정의 메시지가 「the worker reads the analysis, not the setting」이다 |
| 「A run triggered before the setting existed has no language on it, and the claim says so rather than inventing one — the worker then leaves its prompt untouched.」 | ① fn 이름 `a_run_without_a_language_is_claimed_without_one` + 본문이 `UPDATE ... SET llm_language = NULL` 뒤 `job["llmLanguage"].is_null()` 을 단정. 꼬리의 「워커는 프롬프트를 건드리지 않는다」는 `llm::Ask::system_turn` 의 `None` 갈래 |

원장 13차 패스가 같은 유형(단정 바로 위 한두 줄이 그 단정의 재진술)을 「순 제거」로 닫은 선례를 따른다. 이
파일은 이로써 판정 대상 주석이 **0행**이 되어 지문의 파일 수에서 빠진다(130 → 129).

## `e2e/tests/sc04-14-llm-output-language.spec.ts` — 제거 5행

### 유지 — 머리 블록 7행 (뒤집지 않는다)

```
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_LLM_KEY=stub). Signs in as its own
// stub identity (`?as=sc0414`); the language setting is per-user state. What language
// the stub model "writes" in is not observable — the stub answers a fixed document —
// so the spec asserts what the product records: the setting, and the language each
// analysis was fixed to when it started. The prompt text itself is unit-tested in
// `backend/src/llm.rs`.
```

- **선례 고정**: 9차 패스가 `sc01-02` 에서 같은 모양의 「Runs against the e2e deployment (FEATUREDOC_DOUBLE_*=stub)」
  블록을 **유지**로 닫았다. 같은 명제를 패스마다 반대로 판정하면 그 자체가 drift 다 — 뒤집으려면 `sc01-02` 를
  포함한 증분 재판정으로 한 번에 해야 한다.
- 내용으로도 유지 쪽이다. 「stub 이 쓰는 언어는 관측할 수 없다 — 그래서 산출물이 아니라 *제품이 기록한 것*을
  단정한다」는 정책이 유지 대상으로 이름 붙인 **stub 이 real 과 갈리는 지점(충실도 경계)** 이고, 이 spec 이
  무엇을 단정하지 *않는지*를 설명하는 유일한 자리다.
- 1행의 `// 검증 시나리오: 04-platform.md#시나리오 14` 는 `tools/check-scenario-e2e.py` 가 파싱하는 기계 판독
  주석이라 지문·판정 모두에서 제외된다.

### 제거 — 단정 재진술 한국어 한 줄 주석 5행

각 주석 **바로 아래 한두 줄이 그 문장 자체**(복원 경로 ①)이고, 동시에 `docs/test/04-platform.md#시나리오 14`
의 「기대 결과」가 같은 문장을 축자에 가깝게 적는다(복원 경로 ②). 두 경로가 겹친다.

| 제거한 주석 | ① 바로 아래 코드 | ② 문서 |
|---|---|---|
| `// 고른 적 없는 사용자는 한국어로 시작한다.` | `expect(getByTestId('lang-ko')).toHaveAttribute('aria-pressed', 'true')` | 「**기본값은 한국어다**」 · 사전 조건 「출력 언어를 고른 적이 없다」 (`docs/prd/04-platform.md` 에 「고른 적 없는 사용자」 축자) |
| `// 누르는 즉시 저장된다 — 키 등록 버튼을 거치지 않는다.` | 클릭 → `aria-pressed=true` → `/api/settings` 폴링이 `en` | 「선택은 **누르는 즉시 저장**되어」 (축자) |
| `// 다시 열어도 유지된다.` | `page.reload()` 뒤 `aria-pressed=true` | 「**다시 열어도 유지**된다」 (축자) |
| `// 이미 시작한 분석은 시작할 때의 언어를 유지한다.` | 설정을 `ko` 로 바꾼 뒤 `firstRun` 의 `llmLanguage` 가 `en` | 「English일 때 시작한 분석은 이후 설정을 한국어로 바꿔도 English로 기록된 채 남고」 |
| `// 지원하지 않는 값은 거부되고 저장된 설정을 덮어쓰지 않는다.` | `PUT {llmLanguage:'ja'}` → 400, 이어서 `GET` 이 `ko` | 「지원하지 않는 언어 값은 거부되고(HTTP 400) 저장된 설정은 바뀌지 않는다」 |

원장 13차 패스가 같은 유형을 「순 제거」로 닫은 선례를 따른다.

## 증분 — 판정 완료 원장 행 안의 41행 (원장 1·5·10·17·18행)

각 행의 원래 패스 파일에 절을 더했다. 여기서는 합계만 적는다.

| 원장 행 | 파일 · 유입 | 제거 | 유지 | 절 |
|---|---|---|---|---|
| ① backend 집중 4파일 | `analysis.rs` 5 · `llm.rs` 18 · `worker_api.rs` 3 = 26 | **15** | 11 | [2026-09-17-backend-concentrated.md](2026-09-17-backend-concentrated.md) 증분 재판정 ⑨ |
| ⑤ 워커·더블 배선 축 | `bin/worker.rs` 7 | **7** | 0 | [2026-09-18-worker-double-axis.md](2026-09-18-worker-double-axis.md) 증분 재판정 ④ |
| ⑩ 프런트 데이터·셸 축 | `api.ts` 2 · `RegisterLlmKey.tsx` 4 = 6 | **3** | 3 | [2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md) 증분 재판정 ⑥ |
| ⑰ 문서 편집 축 | `doc_edit.rs` 1 | **1** | 0 | [2026-09-21-doc-edit-axis.md](2026-09-21-doc-edit-axis.md) 증분 재판정 ② |
| ⑱ 빠진 feature 직접 추가 축 | `feature_add.rs` 1 | **1** | 0 | [2026-09-21-feature-add-axis.md](2026-09-21-feature-add-axis.md) 증분 재판정 ① |

**세 행은 지문이 바이트 그대로 복귀했다** — ⑤ `632b0475…`(190행) · ⑰ `b9e6778d…`(56행) · ⑱ `b596e739…`(45행)
는 이 패스 직전 원장에 적혀 있던 값과 **완전히 같다**. 그 행들에 #108 이 들여온 주석이 전건 제거로 판정됐다는
뜻이고, 동시에 편집이 다른 줄을 건드리지 않았다는 독립 증거다.

## 유지한 판단 갈림 2건

정책의 「애매하면 남긴다」(비대칭 비용)를 적용한 자리다.

1. `backend/src/llm.rs` 의 「Only prose moves with it. Paths, symbols, glob patterns, enum values and JSON keys
   are what later stages and the screens match on, so the instruction pins them verbatim; translating one would
   break the join, not just the wording.」 4행 — 지시문 문자열이 *무엇을* 고정하는지는 바로 아래 `instruction()`
   의 리터럴이 축자로 복원한다. 그러나 *왜*(뒤 단계와 화면이 그 값으로 조인하므로 번역하면 조인이 깨진다)는
   코드·문서·PR·커밋 어디에도 없다. 제거 쪽 근거가 절반뿐이라 남겼다.
2. `backend/src/llm.rs` 의 `// The user turn is where the input lives; the language must not leak into it.` 1행 —
   바로 아래 `assert_eq!(openai_body(&ask)["input"], "a\nb\nc")` 는 값이 *같다*고만 말하고 「언어가 새어 들면
   안 된다」는 **금지**를 말하지 않는다. 단정 재진술 유형과 경계에 있어 남겼다.

## 검증

- **지문**: 전역 `lines=2593 files=130` / `ebd57171…` → **`lines=2552 files=129` / `623333f7d4d6afbca9a4d0efe0f2e14421134151d43a4a3f653e80521cc79fdd`**.
  파일 수가 하나 주는 것은 `backend/tests/settings.rs` 의 주석이 0행이 되기 때문이다(10차 패스의 `format.ts` 와 같은 어긋남).
- **비주석 바이트 불변**: 문자열·문자 리터럴을 인식하는 stripper 로 줄 주석(`//`·`///`·`//!`)과 블록 주석을
  걷어낸 뒤 부모와 비교해 **11/11 IDENTICAL**. 「비주석 diff 0줄」은 재작성에 딸려 사라진 선언을 놓치므로 쓰지
  않았다(10차 패스의 경고).
- **마이그레이션 무접촉**: `backend/migrations/` 는 한 파일도 바뀌지 않는다(`git diff --name-only` 로 확인).
- 원장의 **범위 지문**은 후행 개행을 **포함**하고(`echo "$HITS" | sha256sum`), 모델의 **전역 지문**은
  versionScript 그대로 후행 개행을 **제외**한다(`printf '%s'`). 같은 입력에도 두 값은 다르다.

## 범위 밖 (후속)

- **마이그레이션 `0012_llm_language.sql` 14행** — 사람 게이트. 잔여 `0009`(30) · `0010`(30) · `0011`(25) 와
  합쳐 **99행 한 배치**로 다뤄야 repair 가 한 번에 끝난다.
- D2 `CORE_FILES` 풀(`db.rs` 17 · `models.rs` 1 · `tests/migrations.rs` 14 · `crypto.rs` 13) · D5
  `deploy/k8s/pvc.yaml` 1 · D6 `tools/check-data-format-change.py` 58 — 해제 조건이 모두 사람 쪽이다.
- `sc04-14` 머리 블록의 재판정은 `sc01-02` 를 포함한 증분 재판정으로만 할 수 있다(위 「뒤집지 않는다」).
