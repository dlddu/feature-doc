# 판정 상세 — 자격증명 · LLM 키 경계 축 (8파일)

- **판정일**: 2026-09-20
- **판정 범위**: `backend/tests/llmkey.rs` ·
  `e2e/tests/sc04-13-unsupported-provider-rejection.spec.ts` ·
  `e2e/tests/sc04-04-revoked-key-blocks-calls.spec.ts` ·
  `e2e/tests/sc04-03-llm-key-registration.spec.ts` ·
  `e2e/tests/sc04-05-credential-log-exposure.spec.ts` ·
  `backend/tests/security.rs` · `backend/src/audit.rs` · `backend/tests/crypto.rs`
- **보류**: `backend/src/crypto.rs` — **D2 사람 게이트 풀**로 이관(아래 「D2 보류」 절)
- **기준 트리**: 부모 **`4a4b593`** (main, 12차 패스 병합 직후)
- **reconciler task**: `tbm_feature-doc-comment-redundancy/rct_20260920-0006`

규칙은 [../README.md](../README.md), 판정 결과의 표면은 [../ledger.md](../ledger.md)에 있다.
이 파일은 이번 범위의 **근거**만 담는다.

## 범위를 이 8파일로 고른 이유

원장이 12차 패스에서 다음 축을 **파일과 행수까지 지목해 뒤 뒀다** — 「자격증명 · LLM 키 경계」
9파일 159행. 그 지목을 독립 재계수로 검증해 **9파일 전건·159행이 일치**하는 것을 확인한 뒤,
그중 `backend/src/crypto.rs`(13행) **한 파일만 빼고** 8파일 146행을 집었다. 뺀 이유는 판정의
난이도가 아니라 **머지 경로가 없기 때문**이다 — 아래 「D2 보류」 절.

이 축을 집으면 **자격증명 계층의 자동 정리 가능한 몫이 닫힌다**: `backend/src/llmkey.rs`(44행)는
원장 1행이 2026-09-17 에 이미 판정했으므로, 이 8파일이 그 둘레의 마지막 미판정 조각이다(남는
`crypto.rs` 는 사람 게이트 풀이다). 다만 11차 패스의 경고를 그대로 물려받는다 — **「닫았다」는 그
시점 트리에서만 참이다.** 열린 PR 이 `backend/src` 에 파일을 더하면 곧바로 거짓이 되므로, 아래
「범위 밖」에 디렉터리 종료를 선언하지 않는다.

## D2 보류 — `backend/src/crypto.rs` 는 무인 슬라이스에서 영구 제외다

`.github/workflows/data-format-review.yml`(`pull_request_target`)이 돌리는 판정기
`tools/check-data-format-change.py` 의 **D2 「저장 계층 핵심」** 규칙은
`backend/src/db.rs` · `backend/src/crypto.rs` · `backend/src/models.rs` · `backend/src/pipeline.rs` ·
`backend/tests/migrations.rs` 다섯 파일의 **경로 소속만** 본다 — D3 와 달리 「주석이 아닌 줄」
예외가 없다. 그래서 **주석 한 줄만 고쳐도** 판정이 `needs_review=true` 로 떨어지고, 워크플로는
commit status `review/data-format` 을 **붙이지 않는다**.

그 status 는 `main` 브랜치 룰셋(`/rules/branches/main`)의 **필수 체크**다(`ci-gate` 와 둘).
없으면 `mergeable_state=blocked` 이 **해소되지 않는다** — CI 지연도 비동기 판정도 아니라서,
사람이 status 를 직접 붙이거나 룰을 풀기 전까지 무인 실행자에게 머지 경로가 없다.

- **실측**: 9파일 판정 트리에서 판정기를 돌리면 `D2 저장 계층 핵심 — 1건: backend/src/crypto.rs`
  로 `⚠️ 사람 리뷰 필요`. `crypto.rs` 만 부모로 되돌린 8파일 트리에서는
  `✅ 변경 없음 (review/data-format = success)` 로 뒤집힌다.
- 나머지 8파일은 안전하다 — `backend/src/audit.rs` 는 D3 대상이지만 **비주석 줄이 안 바뀌어**
  걸리지 않고, `backend/tests/**`(`migrations.rs` 제외) · `e2e/**` · `docs/**` 는 판정기가 아예
  보지 않는다.
- 따라서 이 다섯 파일은 `backend/migrations/*.sql` 8파일과 **같은 성격의 사람 게이트 풀**이지
  자유 풀이 아니다. 원장 「미판정 잔여」를 그렇게 다시 나눴다.
- `crypto.rs` 의 13행에 대한 판정 자체는 이미 끝나 있다(제거 6 · 유지 7). 사람 게이트 패스가 열릴
  때 그대로 쓸 수 있도록 근거를 아래 「보류분의 판정 결과」에 남긴다.

**네 e2e spec 을 한 패스에 넣은 이유는 서로가 서로의 원본이기 때문이다.** 넷은 「키가 쓰이려면
App 연결이 선행돼야 한다」를 각자 자기 본문에 한 벌씩 적어 두었고(4벌), 한 파일만 집으면 어느
벌이 정본인지 정할 수 없다. 12차 패스가 `sc04-01`·`sc04-11`·`sc04-12` 를 같은 이유로 묶은 것과
같은 판단이다.

**복원처가 세 층으로 촘촘하다.** 이 축의 주석이 되풀이하는 것은 거의 전부 아래 표에 있다:

| 복원처 | 무엇을 덮는가 |
|---|---|
| `docs/prd/04-platform.md` AC4.2 | 「등록 대상은 **시스템이 분석 호출을 실제로 수행할 수 있는 제공자**로 한정한다. 호출이 아직 구현되지 않은 제공자의 키 등록 요청은 그 사유와 대안을 밝히며 거부되고, 키는 저장되지 않는다」 · 「쓸 수 없는 키가 "이 분석이 쓸 키"로 선택되어 이미 동작하던 분석을 중단시키는 일이 생기지 않는다」 |
| 같은 문서 AC4.3 | 「(a) 저장 시 암호화, (b) 메모리상에서도 필요한 호출 직전에만 복호화, (c) 로그/오류 메시지에 출력 금지」 · 「자격증명을 다루는 코드 경로는 감사 가능하도록 기록된다」 · 「키 사용 이력은 사용자가 조회 가능」 |
| `docs/test/04-platform.md` 시나리오 3·4·5·13 | 각 spec 머리가 **축자로 인용하던 원문** — 시나리오 4 의 「신규 LLM 호출이 차단되며, "키가 없거나 폐기되었습니다" 메시지가 표시된다」, 시나리오 13 의 기대 결과 전문(거부·미저장·자리 보전·판정의 파생), 시나리오 13 머리의 블록인용(「기존 번호가 다른 AC 의 자동화·문서에 인용돼 있어 재배치하지 않는다」) |
| `docs/doc-tracker/2026-09.md` 「e2e 매핑」 112~114·119행 | 네 spec 의 「이 spec 이 검증하는 것」과 **「자동화 밖 잔여」** 칸 전문 |
| 같은 문서 614행 (2026-09-16 변경 이력) | `sc04-03` 에서 `sc04-04`·`sc04-13` 을 갈라낸 **분리 이력**과 「04#4 의 거부 사유 메시지 실측」이라는 보강 내역 — 주석이 `rct_20260916-0002` 로 가리키던 바로 그 기록 |

## 집계

| 파일 | 판정 전 | 판정 후 | 감소 |
|---|---|---|---|
| `backend/tests/llmkey.rs` | 33 | 10 | 23 |
| `e2e/tests/sc04-13-unsupported-provider-rejection.spec.ts` | 30 | 8 | 22 |
| `e2e/tests/sc04-04-revoked-key-blocks-calls.spec.ts` | 24 | 4 | 20 |
| `e2e/tests/sc04-03-llm-key-registration.spec.ts` | 23 | 8 | 15 |
| `e2e/tests/sc04-05-credential-log-exposure.spec.ts` | 18 | 6 | 12 |
| `backend/tests/security.rs` | 8 | 1 | 7 |
| `backend/src/audit.rs` | 7 | 5 | 2 |
| `backend/tests/crypto.rs` | 3 | **0** | 3 |
| **합** | **146** | **42** | **104** |
| ~~`backend/src/crypto.rs`~~ (D2 보류) | ~~13~~ | ~~7~~ | ~~6~~ |

- **순 제거 104행** · **유지 42행** (제거율 71%).
- diff 기준으로는 **125행 삭제 · 18행 재작성**이다. 지문 기준 감소(104)와 어긋나는 것은 재작성
  때문이다 — 불변식·함정을 담은 주석은 지우지 않고 **되풀이된 절반만 걷어 다시 썼다**.
- `backend/tests/crypto.rs` 는 **전건 제거로 주석 0행**이 되어 지문의 파일 집합에서 빠진다.
  그래서 **판정은 8파일이지만 지문은 `files=7`** 이다 — 10차 패스의 `format.ts` 와 같은 일이고,
  잔여를 「집합 차」로 세야 하는 이유이기도 하다.
- 마지막 줄의 `crypto.rs` 는 **합에 들어가지 않는다**. 판정은 끝났지만 이 패스가 옮기지 않으므로
  그 13행은 잔여에 그대로 남는다(사람 게이트 풀 4파일 / 44행의 일부).

## 복제된 명제 — **정본을 한 곳으로 모았다**

| 명제 | 판정 전 | 정본 | 처리 |
|---|---|---|---|
| 「키가 실제로 쓰이려면 App 연결이 선행돼야 한다 / 설치가 서면 권한 부여 화면이 스스로 키 등록으로 넘긴다」 | **4벌** (`sc04-03` · `sc04-04` · `sc04-05` · `sc04-13`) | `sc04-03` | 정본 1벌만 남기고 3벌 제거. 정본도 「AC4.1 의 화면을 경유만 한다」(②)·「슬라이스 ⑦ 의 두 화면 분할」(③④)을 걷고 **자동 전이 때문에 조작이 없다**는 절반만 남겼다 |
| 「이 단정은 `sc04-03` 과 한 파일에 있었다 — 규칙 2 상 분리 대상으로 등재됐다가 옮겨왔다(`rct_20260916-0002`)」 | **3벌** (`sc04-03` · `sc04-04` · `sc04-13`) | doc-tracker 614행 | **전건 제거**. 레포 안 정본이 주석이 아니라 변경 이력이다 |
| 「등록 시점 타임스탬프는 초 단위라 backdate 하지 않으면 두 행이 동률이 되어 테스트가 아무것도 증명하지 못한다」 | 2벌 (`llmkey.rs` 두 테스트) | 앞의 테스트 | 정본은 그대로, 뒤의 벌은 「위와 같이 backdate」로 줄이고 **그 테스트에만 있는 음성 대조**(게이트를 떼고 재면 preflight 가 `google` 을 답한다)를 남겼다 |
| 「이 키 리터럴은 stub 검증을 통과하도록 **일부러** 형식이 올바르다 — 거부는 키 검증이 아니라 지원 범위에서 나와야 한다」 | 2벌 (`llmkey.rs` 상수 doc · `sc04-13` 상수 doc) + `sc04-13` 본문에 3벌째 | 각 파일의 상수 자리 | 본문의 3벌째만 제거하고 **두 상수 doc 은 둘 다 남겼다** — 아래 「판단이 갈려 남긴 것」 ① |

## 제거한 것 — 복원 경로별

### ① 코드 자체가 이미 말하는 것 — 단정·선언 재진술 (가장 큰 몫)

네 spec 과 세 테스트 파일에서 가장 많이 걷어낸 유형이다. 주석 바로 아래 한두 줄이 그 문장
자체인 경우다.

- `sc04-13`: `// 거부됐으므로 저장되지 않는다 — 이 제공자에 활성 키가 생기지 않는다.` 바로 아래가
  `await expect(page.getByTestId('active-key')).toHaveCount(0);`.
- `sc04-05`: `// 자격증명을 노출할 수 있는 응답 어디에도 평문이 없다.` 바로 아래가 세 엔드포인트를
  도는 `expect(body).not.toContain(...)` 루프.
- `security.rs`: `// The whole audit payload must not contain the plaintext key anywhere.` —
  바로 아래 `assert!` 의 실패 메시지가 이미 `"audit leaked the key"` 다.
- `llmkey.rs`: `// Revoke.` · `// Usable before revocation.` · `// Bob sees none of Alice's keys.` ·
  `// Alice's key is still usable.` 등 8건.
- Rust `///` 중 **시그니처만 영어로 풀어 쓴 것** 1건: `audit.rs` 의
  `/// Records an action.`(=`record(...)`). 12차 패스가 같은 유형 19건을 제거한 것과 같은
  판정이다. (`crypto.rs` 의 같은 유형 2건 — `/// Seals …` · `/// Recovers …` — 은 **D2 보류**로
  이 패스에서 빠졌다.)
- **테스트 이름을 다시 쓴 파일 머리 `//!`** 3건 — `tests/crypto.rs`(「round-trip, tamper detection,
  wrong-KEK rejection」) · `tests/llmkey.rs`(「register, reject invalid, revoke, per-user isolation」) ·
  `tests/security.rs`(「secret redaction, per-user isolation, audit recording」). 셋 다 바로 아래
  `fn` 이름들이 그 목록이다.

### ② 저장소 문서 재진술 — 시나리오 원문 · AC 조항 · 등재 표

- **시나리오 원문 축자 인용** 2건: `sc04-04` 머리의 「docs/test/04-platform.md 시나리오 4를 그대로
  따라간다: …」 3행과 `sc04-13` 머리의 같은 구조 6행. 주석 스스로 「그대로 따라간다」고 적으며
  복원처를 지목한다.
- **「자동화 밖 잔여」 4건** — 네 spec 전건. 이 칸은 doc-tracker 「e2e 매핑」의 마지막 열이 정본이고,
  주석 자신이 `(doc-tracker "e2e 매핑" 참조)`로 그렇게 적는다. 12차 패스가 `sc04-01` 에서 같은
  유형을 제거한 선례를 따랐다.
- **AC 꼬리표** 전건: 머리의 `(AC4.2)`·`(AC4.3)`·`(AC4.7 / test#10)` 과 `audit.rs` 의
  `(AC4.3: usage history is user-visible)` · `llmkey.rs` 의
  `/// AC4.2 supported-provider scope.`. (`crypto.rs` 머리의 `(AC4.3)` 은 D2 보류분이다.)
- `sc04-13` 의 「시나리오 3·4와 같은 AC 를 보지만 뒤에 붙은 시나리오다(**문서 자신의 주석 참조** — …)」
  2행 — 괄호 안이 복원처를 명시한다. `docs/test/04-platform.md:88` 의 블록인용이 그 문장이다.
- `sc04-05` 머리의 「사용자에게 도달하는 절반」이라는 **범위 분할 서술** 3행 — 나머지 절반이
  `e2e/smoke.sh` 라는 사실은 doc-tracker 114행이 적는다.
- `llmkey.rs` 의 `// Blocked afterwards with the specified message.` — 「specified」가 가리키는
  곳이 시나리오 4 원문이다.
- `llmkey.rs` 의 `// The rejection names a way forward rather than only what failed.` — AC4.2
  「그 사유와 **대안**을 밝히며 거부되고」가 원본이다.

### ③ 작업 흔적 — 분리 이력 · 슬라이스 번호 · task id

- `rct_20260916-0002` 분리 이력 3벌(위 복제표) · `sc04-04` 의 보강 경위 5행(「분할 경계 원칙에 따라
  옮기면서 하나만 보강했다…」).
- **슬라이스 ⑦** 을 단 흔적 3건: `sc04-03` 의 「슬라이스 ⑦ 이전의 별도 `continue` 버튼이 하던
  일이다」 · `sc04-04` 의 「(슬라이스 ⑦ 이후 진행 버튼)」 · `sc04-05` 의 「(슬라이스 ⑦ 의 두 화면
  분할)」. 경위의 자리는 커밋 메시지와 doc-tracker 625행이다.
- `sc04-03` 의 목업 식별자 인용 「(목업의 `btn-savekey`)」 — 목업↔구현 대조는 자매 모델
  `tbm_feature-doc-mockup-render` 의 원장이 기계로 지키는 표면이다.

### ④ 낡아서 **거짓이 된** 주석 1건 — 제거 근거를 강화한다

`backend/tests/crypto.rs` 머리가 테스트를 **셋** 열거한다(round-trip · tamper detection ·
wrong-KEK rejection). 파일에는 **다섯** 개가 있다 — `distinct_seals_use_distinct_nonces` ·
`malformed_nonce_is_rejected` 가 그 뒤에 추가됐고 머리는 따라오지 않았다. 판정 중 `cargo test
--test crypto` 로 **5 passed** 를 실측해 확인했다. 12차 패스가 `tests/auth.rs` 머리에서 적발한
것(넷을 열거하는데 여섯)과 **같은 유형의 두 번째 사례**이며, 「목록형 파일 머리는 조용히
낡는다」는 관찰을 굳힌다.

(대조: `tests/llmkey.rs` 머리는 **넷**을 열거하고 테스트는 **여덟**이라 거짓은 아니고 불완전하다.
`tests/security.rs` 머리의 셋은 테스트 셋과 정확히 맞았다 — 그럼에도 제거한 것은 낡아서가 아니라
**이름 재진술**이기 때문이다.)

## 유지한 것 — 복원 불가능한 지식 (42행)

### 자격증명 비노출 **불변식** (`audit.rs` 5)

정책 본문이 이름으로 지목해 보호하는 유형이다(「봉투 암호화·자격증명 비노출 불변식이 **왜 그
자리에서 지켜져야 하는지**」). 봉투 암호화 쪽(`crypto.rs` 7행)은 **D2 보류**라 이 패스에 없다 —
판정 결과는 아래 「보류분의 판정 결과」에 있다.

- `audit.rs` 모듈 머리: 「`detail` 은 비밀이 아닌 맥락(제공자 이름·계정 login·행 id)만 담고,
  키·토큰·암호문은 **절대** 담지 않는다」 — 이 파일에 이 규칙을 집행하는 코드가 없다. 지우면
  다음 `record()` 호출자가 알 길이 없다.
- `audit.rs` `record`: 「best-effort — 감사 행 쓰기 실패가 사용자 요청을 실패시키면 안 된다」.
  코드가 오류를 삼키는 것은 보이지만, 그것이 **의도**라는 것은 보이지 않는다.

### 테스트 설계의 함정 (`llmkey.rs` 8 · `security.rs` 1)

- 「등록 타임스탬프는 **초 단위**라, backdate 하지 않으면 두 행이 동률이 되어 이 테스트는 아무것도
  증명하지 못한다」 — 정본 1벌. `UPDATE … created_at - 100` 만 보고는 왜 필요한지 복원되지 않는다.
- 「여기 OpenAI 키는 **일부러 더 오래된 쪽**이라, 단순 최신순 규칙이면 Anthropic 이 뽑혀 이 단정이
  깨진다」 — 테스트가 무엇을 **배제**하는지는 코드에 없다.
- 「거부가 **호출 시점이 아니라 등록 시점**이어야 하는 이유: 나중에 등록된 미지원 키가 동작하던
  키를 순위에서 눌러 이후 모든 분석이 실패한다」 — 설계 판단. 다만 `ACTIVE_KEY_SQL` 정렬식을
  옮겨 적은 부분은 `backend/src/llmkey.rs` 에서 복원되므로 걷었다.
- 「게이트를 떼고 재 보면 preflight 가 실제로 `google` 을 답한다」 — **음성 대조 실측**이다.
  이 테스트가 공허하지 않다는 유일한 증거이고 어디에도 기록이 없다.
- `security.rs` 의 `// These suites never call /internal; an empty token keeps it closed.` —
  `worker_token: String::new()` 이 실수가 아니라 **닫아 두는 장치**라는 사실.

### stub ↔ real 충실도 경계 · 사용자 격리 규약 (네 spec, 각 2~3행)

12차 패스가 `sc04-01`·`sc04-11`·`sc04-12` 에서 확정한 머리 형태를 **그대로 물려받았다**:
`// 검증 시나리오:` 선언 → 빈 줄 → stub 이 real 대신 무엇을 하는지 → 왜 자기 `?as=` 신원을 쓰는지.

- 「key validation is a deterministic shape check instead of a provider round-trip」 — stub 분기가
  real 과 갈리는 지점. 정책이 이름으로 지목해 보호한다.
- 「Signs in as its own stub identity (`?as=…`); keys are per-user state.」 — 한 배포를 공유하는
  병렬 워커 사이의 격리 계약. `?as=ac42` 리터럴에서 **왜** 가 복원되지 않는다.

### 화면·버튼의 상태 머신 함정 (`sc04-03` · `sc04-04` · `sc04-13` · `sc04-05`, 각 1~2행)

- **`register-key` 가 등록과 진행을 겸한다** — 입력이 비면 등록 대신 pre-flight 를 거쳐 Home 으로
  넘긴다. 정본은 `sc04-03`(같은 버튼을 세 번째로 누르는 자리가 **세 번째 등록이 아니다**),
  `sc04-04` 는 닫히는 조건(키도 입력도 없을 때)만 한 줄로 받는다. 이 한 줄이 없으면 `remove-key`
  직후의 `toBeDisabled()` 가 오작성으로 읽힌다.
- **활성 키 표시는 고른 제공자의 것만 보여준다**(`sc04-13`) — 제공자를 바꾼 직후의
  `toHaveCount(0)` 이 「방금 등록한 키가 사라졌다」로 읽히는 것을 막는다.
- **reload 해야 입력 필드가 아직 들고 있는 평문이 아니라 서버가 되돌려 준 문서를 본다**(`sc04-05`)
  — `page.reload()` 가 이 단정의 전제인 이유.
- **SENTINEL 은 레포에서 유일해야 관측이 성립한다**(`sc04-05`) — 다른 spec 과 값을 공유하면 이
  spec 이 잡은 「부재」가 저 spec 의 부재로 읽힌다. 값 끝의 `0002` 가 왜 붙었는지의 유일한 기록이다.

## 판단이 갈려 남긴 것 (2건)

정책의 「애매하면 남긴다」에 따라 보존하고 여기에 적는다.

1. **`AIzaSy…` 리터럴의 의도 설명이 두 벌 남았다** — `backend/tests/llmkey.rs` 의 `GOOGLE_KEY` doc 과
   `e2e/tests/sc04-13` 의 상수 주석이 같은 말을 한다(「stub 검증을 통과하도록 일부러 형식이 올바르다;
   거부는 지원 범위에서 나와야 한다」). 복제 규칙대로면 한 벌이 잉여다. 그럼에도 남긴 것은 **두
   자리가 서로 다른 언어·다른 층이고, 각 파일에서 그 리터럴이 놀라움의 원천**이기 때문이다 —
   한쪽을 지우면 그 파일 독자는 300줄 떨어진 다른 언어의 파일을 찾아가야 복원된다. 본문에 있던
   3벌째(「형식이 올바른 키로 등록을 시도한다 — 거부는 키 검증이 아니라 지원 범위에서 나온다」)만
   제거했다.
2. **`audit.rs` 모듈 머리의 「Append-only」** — 이 파일에 `UPDATE`·`DELETE` 가 없다는 사실에서
   ①로 복원된다고 볼 수 있다. 그러나 「없다」는 관측이지 계약이 아니다 — 이 한 단어가 나중에
   정정 경로를 추가하려는 사람에게 걸리는 유일한 표지라 남겼다.

## 보류분의 판정 결과 — `backend/src/crypto.rs` 13행 (이 패스는 옮기지 않는다)

판정은 끝났으나 D2 때문에 이 PR 에서 빠졌다. 사람 게이트 패스가 열릴 때 그대로 쓸 수 있게 남긴다.
**제거 6행 · 유지 7행**이며, 이 값은 위 집계의 합에 들어가 있지 않다.

- **제거** — ① 시그니처 재진술 `///` 2건(`/// Seals `plaintext` under a fresh DEK…` =
  `seal(kek, plaintext)` · `/// Recovers the plaintext from an [`Envelope`]…` = `open(kek, env)`) ·
  모듈 머리의 **동작 서술** 3행(「fresh random 256-bit DEK … sealed under AES-256-GCM … wrapped
  with the KEK」 — `seal()` 본문이 그대로 그 순서다) · ② **AC 꼬리표** `(AC4.3)` 1건.
  `Envelope` doc 의 rustdoc 링크만의 교차 참조(`[`seal`]`·`[`open`]`)는 이름에서 복원되므로 걷고,
  본문의 `// Best-effort scrub …` 인라인은 모듈 머리 불변식 문장에 **흡수**한다(2벌 → 1벌).
- **유지 7행** — 모듈 머리의 불변식(「persist 되는 것은 wrapped DEK 와 ciphertext 뿐 — 평문 비밀과
  평문 DEK 는 디스크에 닿지 않고, DEK 는 wrap 되는 즉시 best-effort 로 지워진다」; 「best-effort」는
  Rust 가 소거를 보장하지 않으므로 코드에서 복원되지 않는다) · `Envelope` 의 「저장되는 전부이며
  KEK 없이는 어느 것도 비밀을 드러내지 않는다」 · `open` 의 「변조(GCM 태그 불일치)나 잘못된 KEK 는
  **오류가 되지, 쓰레기 평문이 되지 않는다**」(호출자가 기대는 계약).

## 이 패스가 병합되면

- 판정 범위의 지문은 **42행 / `96636e7917d30bcacf740813a2cafe519bba1d3638f2644f38f628029856a22a`**
  로 선다(지문 파일 집합 7 — `tests/crypto.rs` 는 주석 0행이라 빠진다).
- 전역 지문은 부모 `4a4b593` 의 `lines=2501 files=111` 에서 **`lines=2397 files=110`** 으로 내려간다.
  **다만 절대값은 완료 기준이 아니다** — 아래 「경합」 참조.
- 원장 합계는 판정 **88파일** · 순 제거 누적 **1,824행** · 판정 범위 현재 합계 **2,029행** ·
  미판정 잔여 **24파일 / 368행**(마이그레이션 8 / 139 · **D2 사람 게이트 4 / 44** · 자유 풀 12 / 185)
  이 된다.

## 검증 (판정 시점 로컬 실측, 부모 `4a4b593`)

| 무엇 | 방법 | 결과 |
|---|---|---|
| 전역 지문 재현 | 모델 `asIs.versionScript` 를 부모 트리에서 재실행 | `lines=2501 files=111` / `4e81c916…` — 저장된 `lastObservedVersion` 과 **바이트 동일** |
| 잔여 집합 차 | 전역 지문 파일 목록 − 원장 판정 80파일 | **32파일 / 514행**; 뺄셈(2,501 − 1,987)과 원장 기재까지 **3자 일치** |
| **필수 status 판정** | `python3 tools/check-data-format-change.py --base 4a4b593 --head <PR head>` | **`✅ 변경 없음 (review/data-format = success)`** — 9파일 트리에서는 `⚠️ 사람 리뷰 필요`(D2 1건: `crypto.rs`)였다 |
| **주석만 바꿨는가** | 주석 제거 후(문자열 리터럴 보존 스트리퍼) 부모와 sha256 대조 | **8/8 바이트 동일** — 「비주석 diff 0줄」보다 강한 검사다(주석 블록에 딸려 지워진 선언까지 잡는다) |
| Rust 컴파일 | `cargo test --no-run` | rc=0 |
| Rust 테스트 | `cargo test --test crypto --test security --test llmkey` | **16 passed / 0 failed** (crypto 5 · llmkey 8 · security 3) |
| 문서 게이트 3종 | `check-scenario-e2e.py` · `check-mockup-render.py` · `check-journey-mockup.py` | 모두 **rc=0** |
| 기계 판독 주석 보존 | `// 검증 시나리오:` 선언 4건(spec 당 정확히 1개) | S1 규칙 통과 — 게이트가 읽는 것은 이 줄뿐이고 **위치에 무관**함을 파서에서 확인했다 |
| 허브 등재 | `docs/index.html` 에 패스 상세 행 추가 + Documents 38 → 39 | R9 통과 |
| 범위 지문 | 판정 8파일만의 지문(부모 → head) | 146행 / `cdd213b0…` → **42행 / `96636e79…`** — 감소 104 가 전역 감소(2,501 → 2,397)와 **같은 값**이라 범위 밖 주석 이동 0 |

**로컬에서 못 돈 것**: kind e2e(Playwright) — 호스트에 docker/kind 가 없어 CI 가 유일한 집행자다.
네 spec 의 변경이 **주석 제거 후 바이트 동일**임을 증명했으므로 실행 의미는 바뀌지 않는다.

## 경합 — 열린 PR 3건, 파일 겹침 0

판정 직전 `/pulls?state=open` 전수와 각 PR 의 `/pulls/<n>/files` 를 대조했다:

| PR | 내용 | 이 패스 8파일과의 겹침 |
|---|---|---|
| #91 | 반응형 레이아웃(목업 · `frontend/src`) | 0 |
| #92 | 슬라이스 6a — AC3.1 LLM 보조 문서 수정 | 0 (`backend/tests/migrations.rs` 를 건드리지만 잔여 자유 풀이고 이 범위 밖) |
| #93 | OpenAI strict 스키마 fix(`backend/src` 6파일) | 0 |

셋 다 in-scope 주석 파일을 건드리므로 **먼저 머지되면 전역 지문의 절대값은 움직인다**(#92 는
주석을 가진 새 파일을 여럿 더한다). 그래서 이 패스의 완료 기준은 절대 지문이 아니라
**「부모 대비 순 제거 104행」**이고, 원장의 범위 지문은 이 8파일만의 값이라 자매 머지에 무관하다.

## 범위 밖 (후속)

잔여 **24파일 / 368행**은 세 몫으로 갈린다. 앞의 둘은 **사람 게이트**라 무인 슬라이스의 후보 풀이
아니다.

- **`backend/migrations/*.sql` 8파일 / 139행** — 본문 「적용된 마이그레이션」 절의 전용 PR · 수동
  repair · 사람 승인 게이트를 거치는 **별도 패스**다. 다른 정리와 섞지 않는다. 이번 트리거로
  움직이지 않았다.
- **D2 「저장 계층 핵심」 4파일 / 44행 — `crypto.rs` 13 · `backend/src/db.rs` 17 ·
  `backend/tests/migrations.rs` 13 · `backend/src/models.rs` 1** (다섯 번째인 `pipeline.rs` 는 범위
  안 주석이 0행이다). 위 「D2 보류」 절의 이유로 **무인 슬라이스에서 영구 제외**다 — 주석만 고쳐도
  필수 status `review/data-format` 을 받지 못해 머지가 막힌다. 마이그레이션과 같은 사람 게이트
  풀이며, 셋 이상을 모아 한 번의 사람 리뷰로 끝내는 것이 싸다.
- **자유 풀 12파일 / 185행** — 다음 축으로는 **`tools/check-data-format-change.py` 58**
  (#64 가 들여놓았고 아직 아무도 안 봤다 — 단일 파일로는 잔여 최대) 과
  **`backend/tests/worker.rs` 35 · `backend/tests/common/mod.rs` 16 · `backend/tests/analyses.rs` 7**
  (테스트 하네스 축 **3파일 58행**)이 잡힌다. 하네스 축에서 `backend/tests/migrations.rs` 13 을
  **뺀 값**임에 주의 — 그 파일은 D2 다섯 중 하나라 섞으면 축 전체가 막힌다. 그 다음은
  `sc02-02` 23 · `deploy/k8s/kustomization.yaml` 15 · `util.rs` 12 · `scripts/e2e.sh` 7 ·
  `error.rs` 5 · `main.rs` 5 · `state.rs` 1 · `deploy/k8s/pvc.yaml` 1.
- **`backend/src` 는 여전히 닫히지 않았다.** 이 패스가 `audit.rs` 를 치웠어도 `db.rs` 17 ·
  `crypto.rs` 13 · `util.rs` 12 · `main.rs` 5 · `error.rs` 5 · `state.rs` 1 · `models.rs` 1 이
  남는다(앞의 둘과 `models.rs` 는 D2). 11차·12차 패스와 같이 **디렉터리 단위 종료를 선언하지
  않는다** — 열린 PR #92·#93 이 `backend/src` 에 파일과 주석을 더하는 중이므로, 「닫았다」는 다음
  재감지에서 곧바로 거짓이 된다.

## 슬라이스를 고를 때 반드시 돌릴 것 (이 패스가 배운 것)

파일 목록을 확정한 **직후**, PR 을 열기 전에:

```
python3 tools/check-data-format-change.py --base <main tip> --head <슬라이스 트리> --verbose
```

`✅ 변경 없음` 이 아니면 그 슬라이스는 무인 머지가 불가능하다. 걸린 파일을 빼고 다시 재면 된다 —
이 패스가 `crypto.rs` 하나를 빼 `⚠️` 를 `✅` 로 뒤집은 것이 그 예다. `review/data-format` 은
「리뷰 라우팅 신호」로 읽히기 쉽지만 룰셋상 **`ci-gate` 와 나란한 필수 체크**이므로, 없으면
`mergeable_state=blocked` 이 풀리지 않는다.
