# 판정 상세 — GitHub App · 인증 경계 축 (14파일)

- **판정일**: 2026-09-20
- **판정 범위**: `backend/src/github_app.rs` · `backend/src/github.rs` ·
  `backend/src/github_api.rs` · `backend/src/github_tokens.rs` · `backend/src/auth.rs` ·
  `backend/src/session.rs` · `backend/src/cookies.rs` · `backend/src/installations.rs` ·
  `backend/src/users.rs` · `backend/tests/github.rs` · `backend/tests/auth.rs` ·
  `e2e/tests/sc04-01-app-install-and-scope.spec.ts` ·
  `e2e/tests/sc04-11-unauthenticated-block-and-signin.spec.ts` ·
  `e2e/tests/sc04-12-logout-session-invalidation.spec.ts`
- **기준 트리**: 부모 **`f5a2937`** (main)
- **reconciler task**: `tbm_feature-doc-comment-redundancy/rct_20260920-0005`

규칙은 [../README.md](../README.md), 판정 결과의 표면은 [../ledger.md](../ledger.md)에 있다.
이 파일은 이번 범위의 **근거**만 담는다.

## 범위를 이 14파일로 고른 이유

원장이 11차 패스에서 다음 축을 이미 지목해 뒀다 — 「GitHub App · 인증 경계(`github_app.rs` ·
`github.rs` · `github_api.rs` · `github_tokens.rs` · `auth.rs` · `tests/github.rs` ·
`tests/auth.rs` + sc04 계열). 상류 API 의 문서화되지 않은 거부 조건이 몰린 자리라 **유지 판정
비율이 높을 축**」. 그 지목대로 집었다.

지목에 **인접 소형 4파일**(`session.rs` 5 · `cookies.rs` 3 · `installations.rs` 3 ·
`users.rs` 2 — 합 13행)을 더했다. 이 넷은 로그인 왕복이 실제로 쓰는 저장·쿠키 계층이고,
빼 두면 다음 패스의 범위가 **13행짜리 「나머지 조각」**이 된다. 10차 패스가 같은 이유로
`SignIn.tsx`·`format.ts` 를 끌어들였고, 이번에도 같은 판단을 했다.

세 e2e spec(`sc04-01` · `sc04-11` · `sc04-12`)을 같은 패스에 넣은 이유는 **서로가 서로의
원본**이기 때문이다. 셋은 「기본 stub 사용자를 누가 소유하는가」를 각자 자기 머리에 한 벌씩
적어 두었고(`sc04-11` 이 소유자라고 셋이 모두 말한다), 한 파일만 집으면 어느 벌이 정본인지
정할 수 없다.

**이 축은 복원처가 이 레포에서 가장 촘촘하다.** 세 SSOT 가 같은 자리를 축자로 덮는다:

| 복원처 | 무엇을 덮는가 |
|---|---|
| `docs/prd/04-platform.md` AC4.1 | 「최소 권한(콘텐츠·메타데이터 읽기)만 요청하며, 설치 전에 그 권한 범위를 안내한다」 · 「장기 사용자 비밀을 보관하지 않고, 호출 시점에 단기 설치 액세스 토큰을 발급해 사용한다」 |
| 같은 문서 AC4.3 | 「(a) 저장 시 암호화, (b) 필요한 호출 직전에만 복호화, (c) 로그/오류 메시지에 출력 금지」 |
| 같은 문서 AC4.8 | 「HttpOnly 쿠키로만 유지」 · 「미인증 접근 거부·로그인 유도」 · 「재로그인 시 계정 미중복」 · 「로그아웃 시 즉시 무효화」 |
| `docs/e2e-mocking-policy.md` 지문-등재 표 1~4행 | `auth.rs` · `github_api.rs` · `github.rs` · `github_app.rs` 의 `Mode::Stub` 자리를 **파일·심볼 단위로** EXT-01/EXT-02 에 매어 둔다 |
| `docs/doc-tracker/2026-09.md` 「e2e 매핑」 | sc04-01·11·12 의 「이 spec 이 검증하는 것」과 **「자동화 밖 잔여」** 칸, 그리고 89행의 `?as=<handle>` 격리 규약(「`sc04-11` 만 UI 의 Sign In 버튼(기본 `stub` 사용자)을 소유한다」) |

## 집계

| 파일 | 판정 전 | 판정 후 | 감소 |
|---|---|---|---|
| `backend/src/github_app.rs` | 35 | 13 | 22 |
| `backend/src/github.rs` | 33 | 11 | 22 |
| `backend/tests/auth.rs` | 22 | 8 | 14 |
| `backend/src/auth.rs` | 17 | 4 | 13 |
| `backend/src/github_api.rs` | 15 | 8 | 7 |
| `backend/tests/github.rs` | 15 | 4 | 11 |
| `backend/src/github_tokens.rs` | 7 | 2 | 5 |
| `backend/src/session.rs` | 5 | 1 | 4 |
| `backend/src/cookies.rs` | 3 | 1 | 2 |
| `backend/src/installations.rs` | 3 | 1 | 2 |
| `backend/src/users.rs` | 2 | 1 | 1 |
| `e2e/tests/sc04-01-app-install-and-scope.spec.ts` | 15 | 5 | 10 |
| `e2e/tests/sc04-11-unauthenticated-block-and-signin.spec.ts` | 18 | 5 | 13 |
| `e2e/tests/sc04-12-logout-session-invalidation.spec.ts` | 33 | 7 | 26 |
| **합계** | **223** | **71** | **152** |

범위 지문은 `5a70f045…`(223) → **`261ad97902dd63b0f049c13eed08d8c02118e7c2074b965ebeb7a99136967259`**(71).

**지문 감소와 diff 삭제 줄 수가 이번엔 같다**(둘 다 152). 11차 패스에서 둘이 갈렸던 이유
(파이썬 docstring 이 지문에 안 잡힌다)가 이 범위엔 없다 — Rust·TypeScript 뿐이고, 제거·재작성이
전부 줄머리 `//`·`///`·`//!` 이다. 줄 끝 주석은 애초에 지문 밖이라 건드리지 않았다
(`session.rs` 의 `const SESSION_TTL_SECS … ; // 30 days` 가 그 예다 — 남아 있다).

**동작 코드 무접촉**: 14파일 전부 **주석을 제거한 뒤 부모와 바이트 동일**하다(14/14 IDENTICAL).
문자열·문자 리터럴을 인식하는 stripper 로 줄 주석과 블록 주석을 걷어낸 뒤 비교했다 — 「비주석
diff 0줄」 검사는 주석 재작성에 딸려 사라진 선언을 놓치므로 쓰지 않았다(10차 패스의 경고).

## 복제된 명제 — **정본을 한 곳으로 모았다**

이 축의 중복은 「한 문장이 여러 파일에 복사돼 있다」는 모양이 가장 많았다. 정본을 하나 고르고
나머지를 걷었다.

| 명제 | 복사본이 있던 자리 | 남긴 정본 | 걷은 근거 |
|---|---|---|---|
| **Setup URL 의 `installation_id` 는 스푸핑 가능하다** | `github_app.rs:347` · `github.rs:94` · `github_tokens.rs:4` · (범위 밖) `backend/migrations/0002_github_tokens.sql:3` | `github_app.rs` — **검증하는 함수 자신**(`verify_user_owns_installation`) | `github.rs` 의 것은 바로 아래 호출 `verify_user_owns_installation(...)` 이 이름으로 말한다(①). `github_tokens.rs` 의 것은 괄호 안 곁다리였다 |
| **OAuth 토큰을 왜 보관하는가**(설치 소유 확인 전용) | `github_api.rs:23-24` · `auth.rs:98` · `github_tokens.rs:3-4` | `github_tokens.rs` — **보관하는 모듈 자신** | 나머지 둘은 같은 문장의 2·3벌째다 |
| **미리보기는 자기 콜백 URL 을 App 에 등록할 수 없어 state 에 PR 번호를 태그한다** | `auth.rs:54-57` · `github.rs:41-44` · `tests/auth.rs:146-150` | `auth.rs` — **state 와 `redirect_uri` 를 만드는 자리** | `github.rs` 에는 Setup URL 쪽 고유분(등록 origin 이 하나뿐이라는 사실)만 2줄 남겼다. 테스트의 5줄은 fn 이름 `preview_login_tags_state_with_pull_request_number` 와 단정 메시지 「cookie must hold the tagged state verbatim」 이 그대로 복원한다(①) |
| **adoption 은 best-effort 다**(실패하면 "not installed" 로 떨어진다) | `github.rs:189-192` · `tests/github.rs:279-280` | `github.rs` — **그 동작을 강제하는 코드 옆** | 테스트의 것은 fn 이름 `connection_without_a_usable_token_reports_not_installed` 의 재진술이다(①) |
| **콜백이 안 닿은 사용자는 두 번째 설치를 제안받게 된다** | `github.rs:153-157` · `tests/github.rs:225-227` | `github.rs` — 그 분기 자리(5줄 → 2줄) | 테스트 쪽은 1줄로 줄였다 |
| **기본 stub 사용자는 `sc04-11` 의 것이고 나머지는 `?as=<handle>`** | `sc04-01:8-9` · `sc04-11:11-14` · `sc04-12:22-25` + `doc-tracker/2026-09.md:89` | 각 spec 에 **자기 신원 1~2줄**만 | 「다른 spec 은 어떻게 하는가」라는 **일반 규약**은 doc-tracker 89행의 자리다(②). 각 파일에는 「이 spec 이 어느 신원을 쓰는가」만 남겼다 |

**자격증명 비노출 불변식만은 두 벌을 일부러 남겼다.** `github_app.rs` 와 `github_api.rs` 는 각각
자기 `map_err` 에서 상류 오류를 **버린다**. 「왜 `{e}` 를 끼워 넣지 않는가」는 그 자리에서만
지켜지는 것이라 정책이 유지 대상으로 이름 붙인 「불변식이 **왜 그 자리에서** 지켜져야 하는지」에
해당한다. 다만 두 벌 모두 AC 인용(`(AC4.3)`)은 걷고, 「무엇이 새면 안 되는가」를 파일마다 다르게
적었다(App JWT·설치 토큰·개인키 / client secret·OAuth code·access token).

## 제거한 것 — 복원 경로별

### ① 선언 · 시그니처 재진술 (가장 큰 몫)

이 축의 `///` 요약은 대부분 **함수 이름을 영어 문장으로 풀어 쓴 것**이었다. 정책이 유지하라는
「`pub` 항목의 요약 1줄」은 *요약*이지 *이름의 번역*이 아니다 — 9차 패스가 `dependencies.rs` 의
`pub const` 7개에서 같은 판단을 했다(그 일곱은 지금도 doc 주석이 없다).

| 지운 주석 | 바로 아래 선언 |
|---|---|
| `/// Mints a fresh installation access token for `installation_id`.` | `pub async fn mint_installation_token(state, installation_id) -> InstallationToken` |
| `/// Looks up installation metadata (account + repository selection) via the App JWT.` | `fetch_installation(...) -> InstallationInfo { account_login, account_type, repository_selection }` |
| `/// Best-effort count of repositories the installation can access (for display).` | `pub async fn repository_count(...) -> Option<i64>` |
| `/// One installation of *this* App that the signed-in user can reach, as `GET /user/installations` reports it.` | `pub struct UserInstallation` + 아래 함수의 URL 문자열 |
| `/// A short-lived installation access token. … never written to the database.` | `pub struct InstallationToken { token, expires_at }` (+ AC4.1, + `tests/github.rs` 의 `installation_token_is_short_lived_and_not_persisted`) |
| `/// Stores (or replaces) the user's OAuth token, sealed under the KEK.` | `store(db, kek, user_id, token)` + `crypto::seal(kek, …)` + `ON CONFLICT … DO UPDATE` |
| `/// Loads and decrypts the user's OAuth token, if one is stored.` | `load(…) -> Result<Option<String>>` + `crypto::open` |
| `/// Creates a new session for `user_id`, returning the opaque token to set as a cookie.` | `create(db, user_id) -> Result<String>` + `random_token()` |
| `/// Resolves a (non-expired) session token to its owning user, if any.` | `lookup_user(…)` + `WHERE s.id = ? AND s.expires_at > ?` |
| `/// Deletes a session (logout).` | `delete(…)` + `DELETE FROM sessions WHERE id = ?` |
| `/// Builds an HttpOnly, SameSite=Lax, Path=/ cookie (Secure when configured).` | 바로 아래 네 줄 `set_http_only(true)` · `set_same_site(Lax)` · `set_path("/")` · `set_secure(…)` |
| `/// Builds the matching removal cookie (same name + path) for clearing.` | `Cookie::new(name, "")` + `set_path("/")` |
| `/// The user's most recent installation, if any.` | `ORDER BY created_at DESC LIMIT 1` |
| `/// Inserts a new user or refreshes the profile of an existing one, keyed by `github_id`.` | `WHERE github_id = ?` + 아래 `UPDATE`/`INSERT` (+ `tests/auth.rs::upsert_is_idempotent_by_github_id`) |
| `/// Completes login: validates the CSRF state, resolves the GitHub user, upserts it, opens a session, and redirects to the SPA.` | 바로 아래 다섯 문장을 순서대로 나열한 것 |
| `/// Extractor that resolves the session cookie to the current [`User`], or rejects with 401.` | `impl FromRequestParts for CurrentUser` 본문 전체 |
| `/// Returns the authenticated user, or 401.` · `/// Ends the session and clears the cookie.` | `me` · `logout` 본문 |
| `/// A throwaway stand-in for GitHub's API: answers `GET /user/installations` with `body` … Returns the origin to point `api_base` at.` | `fake_github` 본문(라우트 문자열 · `format!("http://{addr}")`) |
| `// Unknown user -> None.` | `assert!(…load(…, "nobody").…is_none())` |

### ② 저장소 문서 재진술 — AC 조항 · 시나리오 원문 · 등재 표

- **AC 꼬리표** `(AC4.1)` · `(AC4.3)` · `(AC4.7)` · `(AC4.8·AC4.3)` — 전건 제거. AC 본문과의
  매핑은 `docs/doc-tracker/2026-09.md` 의 자리다.
- **`sc04-12` 머리의 시나리오 인용 3줄** — 「docs/test/04-platform.md 시나리오 12를 그대로
  따라간다: 로그인 상태에서 로그아웃한 뒤 … 평문 노출되지 않는다」. **원문의 축자 인용**이고,
  1행의 기계 판독 선언(`// 검증 시나리오: 04-platform.md#시나리오 12`)이 이미 그 문서를
  가리킨다.
- **세 spec 의 제목 줄** — 「「GitHub App 설치와 권한 범위 안내」 전용 spec」 류. 시나리오
  제목의 축자 복사이고, 같은 1행 선언이 복원 경로다.
- **`sc04-01` 의 「자동화 밖 잔여」 2줄** — 주석 자신이 「(doc-tracker "e2e 매핑" 참조)」라고
  **복원처를 신고한다**. doc-tracker 111행 4번째 칸에 같은 문장이 있다. 3차 패스가
  `e2e/support/cluster.ts` 에서 똑같은 모양(「states the same rule in prose」)을 걷은 선례를
  따랐다.
- **`sc04-12` 의 「운영 로그 절반은 … 등재 SSOT 의 「자동화 밖 잔여」에 남는다」 3줄** — 같은 형태.
- **본문 절 제목 9줄**(세 spec) — 「미인증 요청은 보호 API에서 거부된다.」 ·
  「재로그인해도 같은 사용자로 해석된다(계정 중복 생성 없음).」 · 「설치 전: 요청할 읽기 전용
  최소 권한을 먼저 보여준다(여정 F1).」 등. 앞의 둘은 **AC4.8 검증 방법의 축자 복사**이고, 셋 다
  바로 아래 `expect(...)` 가 같은 말을 한다.
- **`github.rs` 의 `/// The read-only scopes the App requests — always present so the screen
  can show them before installation too.`** — 「설치 전에도 보인다」는 `tests/github.rs` 의
  테스트 이름 `connection_when_not_installed_still_lists_requested_permissions` 가 **문장 그대로**
  복원한다(①). 그리고 AC4.1 이 규정한다(②).
- **`github_app.rs` · `github_api.rs` 머리의 stub/real 서술** — 「`Mode::Stub` answers everything
  in-process. Real mode signs an App JWT …」 · 「In `Mode::Stub` every call is answered by a
  deterministic in-process double … (plan: "테스트 더블로 모킹")」. `docs/e2e-mocking-policy.md`
  의 지문-등재 표가 이 두 파일의 stub 자리를 **행으로** 등재해 두었고(2·4행), 파일 안의
  `mock-exception: EXT-01/02` 주석(기계 판독이라 지문·판정 밖)이 각 분기에 그대로 붙어 있다.

### ③ 작업 흔적 — task id · 분리 이력 · 정정 이력

- `sc04-11` 의 「한 파일에 있던 두 시나리오를 분리한 것은 `rct_20260916-0002`가 닫았다」 —
  9차 패스가 `sc01-05` 에서 **같은 문장**을 같은 근거로 걷었다.
- `sc04-12` 의 7줄 — 「규칙 2 상 분리 대상으로 등재됐다가 이 파일로 옮겨왔다. 옮기면서 원문이
  요구하는 관측으로 바로잡았다: 기존 단정은 … 재사용한 것이 아니었다」. **정정 이력**은 커밋
  메시지(④)의 자리이고, doc-tracker 118행·614행이 둘 다 적어 두었다(②). 같은 블록의
  「서버측 무효화는 제품의 실제 동작이다(`session.rs` `delete()` = `DELETE FROM sessions`)」는
  그 코드 자체다(①).
- `tests/auth.rs` 머리의 「this file held a second copy that keyed only on (pid, nanos)」 —
  **무엇이 있었는가**는 이력의 자리다. 남는 위험(무엇이 깨지는가)만 3줄로 재작성했다.

### ④ 낡아서 **거짓이 된** 주석 2건 — 제거 근거를 강화한다

- `github.rs:237` `/// Deterministic per-user stub installation id (distinct users → distinct ids).`
  — 아래 식은 `10_000 + github_id.rem_euclid(90_000)` 이라 **90,000 을 주기로 충돌한다.**
  「distinct users → distinct ids」는 참이 아니다. 결정성은 식이 말하므로 줄째로 걷었다.
- `tests/auth.rs:1` `//! Auth surface: 401 without a session, 200 with one, idempotent upsert,
  stub login redirect.` — 네 개를 열거하는데 파일에는 **테스트가 여섯 개**다(미리보기 2건이
  나중에 들어왔다). 열거를 버리고 한 줄로 다시 썼다.

## 유지한 것 — 복원 불가능한 지식

정책이 **이름으로 열거한** 유지 대상에 그대로 해당하는 것들이다.

- **상류 API 의 문서화되지 않은 동작** (원장이 「유지 비율이 높을 축」이라 예고한 바로 그 몫)
  - `github.rs` — 「**GitHub does not forward state to the Setup URL**, so only enforce a match
    when a state was actually echoed」. 조건부 CSRF 검사가 **버그가 아닌 이유**이고, GitHub 문서
    어디에도 이 문장은 없다.
  - `github_app.rs` — 「`/user/installations` 는 **토큰을 발급한 App 으로 이미 한정돼 있어**
    app id 로 거를 것이 없다」. 코드에 없는 것(필터 부재)을 설명한다.
  - `github_app.rs` — 「`iss` 는 numeric App ID 가 아니라 **client ID**: 오늘은 둘 다 인증되지만
    GitHub 이 앞으로의 호환을 client ID 에 걸었다(2024-05 기준)」. **기각된 대안**의 기록이다.
  - `github_api.rs` — 「`redirect_uri` 는 `/login/oauth/authorize` 에 보낸 것과 정확히 같아야
    한다 — 미리보기에서는 우리 origin 과 다르다」. **상류의 거부 조건**이다.
- **자격증명 비노출 불변식이 왜 그 자리에서 지켜지는가** — `github_app.rs` · `github_api.rs`
  머리의 「상류 실패는 고정 문자열로 사상하고 **끼워 넣지 않는다**」 두 벌(위 「일부러 남긴
  두 벌」 참조).
- **저장소 제약의 함정** — `github_api.rs` 의 `// Shift to guarantee a positive,
  SQLite-friendly i64.`. `>> 1` 이 왜 있는지는 식만 보면 읽히지 않는다.
- **stub 이 real 과 갈리는 지점(충실도 경계)**
  - `github_app.rs` — 「Stub mode has no GitHub to ask and reports none — the stub install flow
    writes its row through the Setup URL instead」. 이른 `return Ok(Vec::new())` 이 **누락이
    아니라 설계**임을 말한다.
  - `github_app.rs` — 「세 개다 — [`repository_count`] 의 stub 분기와 **수가 맞아야 한다**」.
    `stub_repositories()` 의 3과 `repository_count()` 의 `Some(3)` 은 **아무도 대조하지 않는**
    한 쌍이다.
  - `github_api.rs` — 「서로 다른 `code` 는 서로 다른 사용자를 내야 한다 — `?as=<handle>` 의
    spec 격리가 여기에 얹혀 있다」. 층을 건너뛰는 계약이라 어느 쪽에서도 안 보인다.
- **의도적 오류 삼킴** — `github.rs` 의 adoption best-effort 3줄. `Option` 반환과
  `tracing::debug!` 를 `Result` 로 「고치면」 화면이 죽는다.
- **실패 모드의 함정 / 실제 사고 기록**
  - `tests/auth.rs` — 경로 할당을 공유하는 이유(시계 해상도가 거친 호스트에서 한 바이너리의 두
    테스트가 같은 경로를 뽑아 마이그레이션이 경합한다).
  - `tests/auth.rs` — 「`Set-Cookie` 가 둘 내려오고 **순서가 보장되지 않는다**」. 첫 헤더만 읽던
    단정이 실제로 깨졌던 자리다.
- **부작용이 코드에서 안 보이는 단정** — `tests/github.rs` 의 「Adopted for good, not re-fetched
  on every render.」(응답이 아니라 DB 행을 읽는 이유) · 「The token must be encrypted at rest,
  not stored as plaintext.」(`windows(9)` 바이트 검색의 의도).
- **설치 전/후로 갈리는 관측** — `sc04-12` 본문의 「직전 세션 쿠키를 **포착**해 둔다 — 컨텍스트가
  지워주는 것과는 다른 관측이다」 2줄. 이 파일이 존재하는 이유(9차 패스가 바로잡은 그 관측)를
  지키는 **유일한** 자리가 됐다. 머리의 이력 7줄을 걷었으므로 이 2줄은 남겨야 한다.
- **범위 밖 선언** — `github_app.rs` 의 「`size_kb` 는 GitHub 이 보고한 크기: **사전 추정용이지
  접근 판단이 아니다**」. 금지는 부재라 코드에서 읽히지 않는다.

## 판단이 갈려 남긴 것 (2건)

정책의 「애매하면 남긴다」를 적용했다.

1. **세 spec 의 「Runs against the e2e deployment (FEATUREDOC_DOUBLE_*=stub)」 블록.**
   `docs/e2e-mocking-policy.md` 의 env 표가 같은 배선을 등재하므로 ② 로 걷을 여지가 있다.
   그런데 **9차 패스(`2026-09-20-pipeline-cross-cutting-axis.md`)가 `sc01-02` 에서 같은 모양을
   판정해 「유지」로 닫았다** — 「`toHaveCount(3)` 의 3 이 어디서 오는지는 코드에 없다」가 근거다.
   같은 명제를 패스마다 반대로 판정하면 그 자체가 drift 이므로 선례를 따랐다. 대신 ① 로 확실히
   복원되는 부분(「round-trip bounces through our own setup callback」 — `github.rs` 의 stub 분기
   그 자체)만 걷어 블록을 줄였다. **이 판정을 뒤집으려면 `sc01-02` 를 포함한 증분 재판정으로
   한 번에 해야 한다.**
2. **`auth.rs` 머리 1줄** `//! Login, logout, session cookie, and the authenticated-user extractor.`
   — 바로 아래 `routes()` 가 같은 넷을 나열한다(①). 다만 정책이 「모듈 머리 `//!` 를 유지한다」고
   명시했고, 3차 패스가 `e2e/smoke.sh` 에서 「파일이 무엇인지 한 줄」을 같은 이유로 남겼다.

## 이 패스가 병합되면

- 전역 주석 지문: `lines=2653 files=111` / `15cb6b24…` → **`lines=2501 files=111` /
  `4e81c916e413f7badaa79eac340f5cd6685857ec2e5e90b0271e0a36ccac8dd8`**
  (**부모 대비 순 제거 152행**, 파일 수 불변 — 주석이 0행이 된 파일이 없다).
- 판정 완료: 66파일 / 유지 1,916행 → **80파일 / 유지 1,987행**, 누적 순 제거 1,568 → **1,720행**.
- 미판정 잔여: 46파일 / 737행 → **32파일 / 514행**
  (= 마이그레이션 8파일 139행 + 자유 풀 **24파일 375행**).
- **완료 기준은 절대 지문이 아니라 「부모 대비 순 제거 152행」이다.** 열린 자매 PR
  **#91**(`frontend/src/index.css` +57) · **#92**(`backend/src` 신규 3파일 + e2e 2파일) ·
  **#93**(`backend/src/llm.rs` +35 외)이 먼저 머지되면 전역 절대값은 움직인다. 셋 다 이 14파일과
  **파일 겹침이 0** 이라 순 제거 152 는 그대로다(11차 패스가 #64 에 당한 뒤 원장에 남긴 경고).

## 검증 (판정 시점 로컬 실측, 부모 `f5a2937`)

| 검사 | 방법 | 결과 |
|---|---|---|
| 범위 주석 재계수 | 모델 versionScript 의 추출식을 14파일에 적용 | 223 → **71** (파일별 수치는 위 집계표와 전건 일치) |
| 범위 지문 | `echo "$HITS" \| sha256sum` (후행 개행 **포함** — 원장 규약) | `5a70f045…` → `261ad979…` |
| 전역 지문 | versionScript 그대로 (`printf '%s'` — 후행 개행 **제외**) | `15cb6b24…` → `4e81c916…`, 차 **152행** |
| 동작 코드 무접촉 | 리터럴 인식 stripper 로 줄·블록 주석을 걷어낸 뒤 부모와 바이트 비교 | **14/14 IDENTICAL** |
| 백엔드 테스트 | `cargo test`(offline) | 통과 — 단위 63 + 통합 전 스위트 |
| 레포 게이트 3종 | `check-mockup-render.py` · `check-scenario-e2e.py` · `check-journey-mockup.py` 를 부모·head 두 지점에서 실행 | 전부 rc=0. 앞 둘은 **출력 바이트 동일**, 세 번째만 이 패스가 더한 `passes/` 문서 1건과 그 허브 등재만큼 달라진다(R8·R9) |

## 범위 밖 (후속)

- **마이그레이션 8파일 139행** — 사람 repair 게이트를 거치는 전용 PR 경로(README 「적용된
  마이그레이션」). 이 패스가 `backend/migrations/0002_github_tokens.sql:3` 에서 「spoofable」의
  **4벌째**를 확인했으나 손대지 않았다. 마이그레이션 패스가 이 표의 정본 지정을 이어받으면 된다.
- **자유 풀 24파일 375행** — 다음 축 후보는 **자격증명 · LLM 키 경계**
  (`backend/tests/llmkey.rs` 33 · `crypto.rs` 13 · `tests/crypto.rs` 3 · `audit.rs` 7 ·
  `tests/security.rs` 8 + `sc04-03` 23 · `sc04-04` 24 · `sc04-05` 18 · `sc04-13` 30 —
  합 **159행**). `llmkey.rs`(44)는 원장 1행이 이미 판정했으므로 그 축을 집으면 자격증명
  디렉터리가 닫힌다 — 단, 11차 패스의 경고대로 **「닫았다」는 그 시점 트리에서만 참이다.**
  그 다음은 신규 유입 `tools/check-data-format-change.py` 58 과 `backend/tests/worker.rs` 35.
- **`backend/src/llm.rs:494-499` 의 주석이 거짓이고, 그 거짓이 실제 flake 를 덮고 있다.**
  「The env writes stay inside this one test; **no other test reads this variable**, so parallel
  test runs cannot race on it」이라고 적혀 있는데, 같은 파일 449행의 `stub_answer` 가
  `FEATUREDOC_STUB_LLM_FAIL` 을 읽고 **`stub_is_deterministic_for_the_same_ask` 가 그 경로를
  탄다.** 둘 다 `#[tokio::test]` 라 같은 프로세스에서 병렬로 돌고, 실측 **15회 중 1회**
  `stub_is_deterministic_for_the_same_ask` 가 실패한다(부모 `f5a2937` 에서도 재현 — 이 패스와
  무관하다). `llm.rs` 는 원장 1행이 이미 판정한 범위라 **증분 재판정**으로 다뤄야 하고, 주석만
  고치는 것으로는 flake 가 남으므로 **제품/테스트 수정이 따라붙는 별개 작업**이다. 여기서는
  기록만 한다.
- **지문 사각지대(파이썬 `"""docstring"""`)** — 판정 기준 자체의 개정이라
  `reconciler-tobe-modeler` 의 몫이다(11차 패스가 이미 등재했다).
