# 판정 상세 — API 셸 · 배포 베이스 축 (5파일)

- **판정일**: 2026-09-20
- **판정 범위**: `deploy/k8s/kustomization.yaml` · `backend/src/util.rs` ·
  `backend/src/main.rs` · `backend/src/error.rs` · `backend/src/state.rs`
- **기준 트리**: 부모 **`445ec57`** (main, 14차 패스 병합 직후)
- **reconciler task**: `tbm_feature-doc-comment-redundancy/rct_20260920-0008`
- **PR**: #97 (squash)

규칙은 [../README.md](../README.md), 판정 결과의 표면은 [../ledger.md](../ledger.md)에 있다.
이 파일은 이번 범위의 **근거**만 담는다.

## 범위를 이 5파일로 고른 이유 — 남은 자유 풀 전량

14차 패스가 잔여를 네 몫으로 갈라 적어 두었다: 마이그레이션 8파일 139행 · D2 저장 계층 4파일
44행 · D6·D5 경로 규칙 2파일 59행 · **자유 풀 5파일 38행**. 앞의 셋은 전부 **사람 게이트**라
무인 슬라이스의 후보가 아니다(각각 수동 repair · `needs_review=true` 로 필수 체크가 붙지 않음).

그래서 이 패스는 **남은 자유 풀을 통째로** 집었다. 쪼갤 이유가 없다 — 다섯 파일 합이 38행이고,
서로 다른 D 규칙에 걸리지도 않으며, 열린 PR 셋 중 어느 것과도 겹치지 않는다. 사람 게이트 3몫과는
**섞지 않았다**(섞으면 축 전체가 필수 체크 벽에 막힌다 — 13차가 `crypto.rs` 로 겪었고 14차가
원장에 명시했다).

**슬라이스 전 필수 절차를 먼저 돌렸다.** 원장이 못박은 대로, 집기 전에 후보 트리에서 판정기를
직접 돌려 무인 머지 경로를 실측했다:

```
$ python3 tools/check-data-format-change.py --base 445ec57 --head 1efcbed --verbose
## 데이터 저장 형식 변경 판정: ✅ 변경 없음 (`review/data-format` = success)
변경 파일 3개 · 범위 445ec57...1efcbed4a257d9a1c6baa35a509c68e3c4974b62
```

다섯 파일 중 실제로 바뀐 셋이 **D1 · D2 · D5-pvc · D6 어디에도 안 걸린다.** 원장의 분류가
추정이 아니라 측정으로 확인됐다.

## 집계

- 범위 주석 **38행** → **유지 19행**, **순 제거 19행**(제거 22행 중 불변식 보존 **2행 재작성**).
- 범위 지문: `445ec57` 의 `8ef1…`(이 5파일 부분) → **`3b0735c725531481568605ca1a47b0573a706afce46b723edfcc66ab382db4e7`**
  (`lines=19 files=4`). **판정은 5파일인데 지문의 파일 수가 4인 것은**
  `deploy/k8s/kustomization.yaml` 이 주석 0행이 되어 지문의 파일 집합에서 빠지기 때문이다
  — 파일 수는 지문의 `files` 가 아니라 이 행의 목록 길이로 센다.
- 전역 주석 지문 예고: `lines=2329 files=109` / `3109240a…` →
  **`lines=2310 files=108` / `7d1ddd5fef6f5df92161606ede330ed0df54396590b519badd47ee09aa494571`**.
- **완료 기준은 절대 지문이 아니라 「부모 `445ec57` 대비 순 제거 19행」이다.** 열린 자매
  PR 셋이 먼저 머지되면 전역 절대값은 움직인다(아래 「경합」).

| 파일 | 부모 | 이 패스 뒤 | 순 제거 |
|---|---|---|---|
| `deploy/k8s/kustomization.yaml` | 15 | **0** | 15 |
| `backend/src/util.rs` | 12 | 9 | 3 |
| `backend/src/error.rs` | 5 | 4 | 1 (3행 → 2행 재작성) |
| `backend/src/main.rs` | 5 | 5 | 0 |
| `backend/src/state.rs` | 1 | 1 | 0 |
| **합** | **38** | **19** | **19** |

## 제거한 것 — 복원 경로별

### ② 저장소 문서 재진술 — `kustomization.yaml` 머리 전체 15행

이 파일의 주석은 **`README.md` 가 이미 말하는 것을 영어로 다시 쓴 것**이다. 문장 단위로 대조하면
거의 축자에 가깝다.

| 지운 주석 | 복원 경로 |
|---|---|
| `Production base. Points at the image CI publishes (ghcr.io/dlddu/featuredoc) at an immutable commit-SHA tag.` | ② `README.md` §배포 — 「`deploy/k8s/`는 운영용 kustomize 베이스로, CI가 커밋마다 고정해 넣는 불변 태그 `ghcr.io/dlddu/featuredoc:<commit sha>`를 가리킵니다」 |
| `CI's `pin` job rewrites that tag in the two Deployments and publishes the result to the `deploy` branch after each main build (Flux tracks `deploy`)` | ② `README.md` §CI 의 **`pin`** 불릿 — 같은 문장을 더 자세히 적는다(force-push · `Source-Commit:` 트레일러 · Flux 가 `deploy` 추적까지) |
| `so this directory on `deploy` always states exactly which commit is running` | ② `README.md` 의 「**태그 = 배포 상태**」 인용 블록 — 「지금 운영에 무엇이 떠 있는지는 `deploy` 브랜치의 `deploy/k8s/deployment.yaml` 태그 한 줄이 그대로 말해줍니다」 |
| `— no rolling tag, no :latest` | ② `README.md` §CI 의 **`push`** 불릿 — 「태그는 커밋 SHA 하나뿐입니다 — `latest`도, 브랜치 롤링 태그도 만들지 않습니다」 |
| `and no imagePullPolicy needed (IfNotPresent is correct for an immutable tag).` | ② `README.md` §배포 — 「태그가 불변이라 `imagePullPolicy`는 기본값 `IfNotPresent`가 맞고」 |
| `Two workloads out of that one image (AC4.5): `featuredoc` serves the API and owns the SQLite volume; `featuredoc-worker` drains the analysis queue over the API's /internal routes and mounts nothing.` | ① 바로 아래 `resources:` 의 `deployment.yaml`·`worker-deployment.yaml` 두 줄 + 두 파일의 `image:`·`command:`·`volumeMounts:` · ② `README.md` 트리의 「worker-deployment.yaml # 분석 워커 워크로드 — 같은 이미지, 커맨드만 다름, **볼륨 없음** (AC4.5)」 · ③ `(AC4.5)` 꼬리표 자체가 작업 흔적의 자리 |
| `The Deployment consumes `featuredoc-secrets` via envFrom.` | ① `deploy/k8s/deployment.yaml` 의 `envFrom: - secretRef: name: featuredoc-secrets` 그 자체 |
| `That Secret is provisioned out-of-band (copy secret.yaml.example -> secret.yaml, fill in, and apply it) and is intentionally NOT generated here` | ② `README.md` 트리의 「k8s/ … secret은 **외부 제공**」과 「secret.yaml.example — featuredoc-secrets 템플릿, **복사·기입 후 적용**」 · ① 바로 아래 `resources:` 에 `secretGenerator` 가 **없다는 사실** |

빈 주석 행 `#` 2행도 함께 걷었다(남길 문장이 없다). 파일은 주석 0행이 되고 `apiVersion` ·
`kind` · `resources` 는 한 글자도 바뀌지 않았다.

### ① 이름·시그니처를 영어로 옮긴 `///` 3행 (`util.rs`)

12차 패스가 세운 판정 규칙을 그대로 적용했다 — 「정책이 유지하라는 `pub` 항목의 **요약 1줄**은
*요약*이지 *이름의 번역*이 아니다」(9차 패스가 `dependencies.rs` 의 `pub const` 7개에서 같은
판단을 했고, 12차가 `github_app.rs`·`auth.rs` 에서 19건을 같은 근거로 걷었다).

| 지운 주석 | 바로 아래 선언 |
|---|---|
| `/// Current wall-clock time as unix epoch seconds.` | `pub fn now_unix() -> i64` + `SystemTime::now().duration_since(UNIX_EPOCH)…as_secs() as i64` — 이름·반환 타입·본문 셋 다 같은 말을 한다 |
| `/// 256 bits of OS randomness, hex-encoded. Used for session ids and OAuth state nonces.` | `pub fn random_token() -> String` + `[0u8; 32]` · `OsRng.fill_bytes` · `hex::encode` (앞 문장) / 호출자 `auth::session::create` · 바로 아래 `oauth_state` (뒤 문장 — 호출자 재진술) |
| `/// Parses an RFC 3339 timestamp (as GitHub returns for token expiry) to unix seconds.` | `pub fn rfc3339_to_unix(s: &str) -> Option<i64>` — 이름이 문장 전체다. 괄호 안은 호출자(`github_app.rs` 의 `expires_at`) 재진술 |

세 함수 모두 **doc 주석이 없어도 이름 밖의 정보가 1비트도 사라지지 않는다.** 모듈 머리
`//! Small shared helpers: time + cryptographically-random opaque tokens.` 가 이미 파일의 지도
역할을 한다(유지).

### 동작 서술 재진술 — `error.rs` 모듈 머리 (3행 → 2행)

13차 패스가 `crypto.rs` 모듈 머리에서 한 것과 같은 모양이다 — **동작 서술은 걷고 불변식은
남긴다.**

```
-//! Client-facing messages are intentionally terse and never echo credential
-//! material. `Internal` carries an operator-only detail that is logged (never
-//! returned) — and even that is constructed by us, so secrets never flow in.
+//! No variant ever echoes credential material: the detail on `Internal` is always
+//! constructed by us, never woven from caller input or an upstream response body.
```

- 걷은 부분은 **같은 파일의 코드가 그대로 말한다**(①): 「messages are intentionally terse」는
  `parts()` 의 리터럴 넷(`"unauthorized"` · `"forbidden"` · `"not found"` · `"internal error"`),
  「detail that is logged (never returned)」는 `into_response` 의
  `tracing::error!(detail = %detail, …)` 와 `AppError::Internal(_) => (…, "internal error".into())`
  가 나란히 있는 것 자체다.
- 남긴 것은 **불변식**이다 — 「`Internal` 의 detail 은 **언제나 우리가 만든다**」는 이 파일의
  코드에서 복원되지 않는다(모든 생성 지점에 대한 주장이라 `From<sqlx::Error>` ·
  `From<anyhow::Error>` 를 봐도 「앞으로도 그렇다」가 안 나온다). 정책이 이름 붙인
  「자격증명 비노출 불변식이 **왜 그 자리에서** 지켜져야 하는지」다.

## 유지한 것 — 복원 불가능한 지식 (19행)

### PID 1 시그널 함정 (`main.rs` 5행) — **전건 유지**

```rust
// The API is its container's PID 1, and the kernel drops any signal sent to a
// PID-namespace init that has no handler for it. Without these a rollout's
// SIGTERM is ignored and the kubelet SIGKILLs only after the whole grace period
// (30 s by default) — all of it downtime under the Recreate strategy. Installed
// first so that a stop landing during migrations is still seen.
```

정책이 이름 붙인 **실패 모드의 함정**이고, 코드 어디에도 없다. `deployment.yaml` 의
`strategy: type: Recreate` 와 `signal(SignalKind::terminate())` 두 줄은 **사실**만 말하고
「핸들러가 없으면 커널이 버린다」·「그 시간 전부가 다운타임이다」는 말하지 않는다.
**4차 패스가 `bin/worker.rs` 의 같은 함정 5행을 「지우지 않은 것 — 복원 불가능한 지식」으로
닫아 두었다**(`2026-09-18-worker-double-axis.md`). 같은 명제를 패스마다 반대로 판정하면 그
자체가 drift 다. → 아래 「판단이 갈려 남긴 것」 참고.

### OAuth `state` 접두사가 보안 구멍이 아닌 이유 (`util.rs` 8행)

```rust
/// Builds an OAuth `state`: a fresh nonce, prefixed with `pr-<id>~` when this
/// process is a pull-request preview.
///
/// The prefix is routing metadata for the redirect proxy sitting on the App's
/// registered callback host — it tells that proxy which preview to bounce the
/// code back to. It is not a secret and grants nothing: CSRF safety still rests
/// entirely on the whole string matching the cookie the browser kept, and the
/// proxy can only ever rebuild a host inside the preview namespace.
```

- **리다이렉트 프록시는 이 레포에 없다.** App 의 등록된 콜백 호스트 위에 있는 외부 구성이고,
  「접두사를 읽어 preview 로 되튕긴다」·「preview 네임스페이스 안의 호스트만 만들 수 있다」는
  저장소 어디에서도 복원되지 않는다(`README.md` · `docs/` 전수 grep — `redirect proxy` 0히트,
  `callback`·`CSRF`·`nonce` 도 마크다운 전체에서 0히트. `preview` 는 `README.md:183` 의 **PR
  빌드 이미지** 문맥과 목업 HTML 의 CSS 클래스명뿐으로, OAuth 프록시와 무관하다).
- **「접두사를 붙여도 CSRF 안전성이 안 깎인다」는 불변식**이다. 정책의 유지 대상
  「자격증명·CSRF 불변식이 **왜 그 자리에서** 지켜져야 하는지」에 해당한다. 요약 1줄은
  `pub` 항목의 유지 조항이고, 본문 5행이 이 불변식이다.

### 모듈 머리 셋 (`util.rs` 1행 · `error.rs` 1행 + 빈 `//!` 1행 · `state.rs` 1행)

정책이 「모듈 머리 `//!` 를 유지한다」고 명시했고, 12차 패스가 `auth.rs` 머리 1줄에서
(「바로 아래 `routes()` 가 같은 넷을 나열한다」는 ① 근거가 있는데도) 같은 조항으로 남겼다.
3차 패스가 `e2e/smoke.sh` 에서 「파일이 무엇인지 한 줄」을 같은 이유로 남긴 것이 선례다.
`state.rs` 의 `//! Shared application state handed to every handler via `axum::extract::State`.`
1행이 이 파일의 주석 전부다.

## 판단이 갈려 남긴 것 — 1건

**`main.rs` 의 PID 1 함정은 `bin/worker.rs:118-122` 와 같은 명제의 두 번째 벌이다.**
9차 패스가 「같은 명제의 복제는 한 벌만 남긴다」를 적용해 최대 여섯 벌을 걷었으므로, 기계적으로는
여기서 한 벌을 줄일 여지가 있었다. **남긴 이유 셋**:

1. **복원 경로 넷 중 어디에도 안 걸린다.** `worker.rs` 의 그 다섯 줄은 **주석**이지 코드·문서·
   PR·커밋 메시지가 아니다. 「다른 파일의 주석에 적혀 있다」는 정책이 정의한 복원 경로가 아니므로,
   복제 정리는 판정 규칙이 아니라 **재량**이다.
2. **독립된 표면이다.** 두 파일은 별개 바이너리(`featuredoc` · `featuredoc-worker`)이고 별개
   워크로드로 뜬다. 9차 패스도 「화면이 아니라 서버가 기억한다」를 여섯 벌에서 **두 화면 머리 각
   1줄**로 줄였지 한 벌로 합치지 않았다 — 독립 표면마다 한 벌이 그 패스의 선례다.
3. **두 벌이 같은 말만 하는 것도 아니다.** `main.rs` 쪽에만 있는 것이 둘이다 —
   「`Recreate` 아래서는 그 grace period **전부가 다운타임**」(워커는 볼륨이 없어 `RollingUpdate`
   라 해당 없음)과 「**마이그레이션 중**에 떨어진 stop 도 보이도록 먼저 설치」(워커에는
   마이그레이션이 없다).

정책의 「**애매하면 남긴다**」(비용 비대칭)를 적용해 **전건 유지**한다. 뒤집으려면
`bin/worker.rs` 를 포함한 **증분 재판정으로 한 번에** 해야 한다(12차 패스가 `sc01-02` 에서
세운 규약).

## 검증 (판정 시점 로컬 실측, 부모 `445ec57`)

| 검사 | 결과 |
|---|---|
| `python3 tools/check-data-format-change.py --base 445ec57 --head 1efcbed --verbose` | **`✅ 변경 없음`** (`review/data-format` = success) · 검사 파일 3개 |
| `cargo test --tests` (backend) | **159 passed · 0 failed** (부모와 동일) |
| **주석 제거 후 부모와 바이트 동일** | 바뀐 3파일 전건 **SAME** — 줄머리 주석을 걷어낸 sha256 이 `445ec57` 의 같은 파일과 일치. 세 파일에 블록 주석 시작(`/*`)이 **0건**이라 줄 단위 stripper 로 충분하다 |
| 비주석 diff | **0줄** (`git diff -U0` 에서 주석 아닌 `+`/`-` 행 0) |
| `python3 tools/check-journey-mockup.py` | rc=0 — 「여정 6 · 목업 페이지 5 · **문서 41**」 (허브 등재 포함, 아래) |
| `python3 tools/check-mockup-render.py` · `check-scenario-e2e.py` | rc=0 |
| `node tools/check-journey-prototype.js` | rc=0 — 여정 프로토타입 5개 · 단언 408건 (`npm ci` 후. 의존성이 없으면 부모에서도 `MODULE_NOT_FOUND` 로 죽으므로 이 패스의 회귀가 아니다) |

**허브 등재가 함께 갔다.** 이 패스가 `passes/` 에 판정 상세를 더하면서 `docs/index.html` 에
링크 행을 넣고 `Documents` 집계를 **40 → 41** 로 올렸다. 빠뜨리면 `check-journey-mockup.py` 의
R8·R9 가 실패한다(13차 패스가 원장에 적어 둔 함정). 허브는 주석 지문 범위 밖이라 판정 수치에는
영향이 없다.

## 경합 — 열린 PR 3건, 파일 겹침 0

claim 직전(2026-09-20T16:2xZ) `/pulls?state=open` 전수 재조회: **#91**(반응형 레이아웃) ·
**#92**(슬라이스 6a `rct_20260919-0003`) · **#93**(OpenAI strict fix), 셋 다 base `f5a2937`.
각 PR 의 `/pulls/<n>/files` 를 이 5파일과 대조해 **교집합 0**.

> ⚠️ `#92` 가 건드리는 `deploy/e2e/kustomization.yaml` 은 이 패스의
> `deploy/k8s/kustomization.yaml` 과 **다른 파일**이다. 경로가 비슷해 오판하기 쉽다.

셋 다 in-scope 주석 파일을 건드리므로 **먼저 머지되면 전역 지문의 절대값은 움직인다.**
그래서 완료 기준이 절대 지문이 아니라 「부모 대비 순 제거 19행」이고, 위 범위 지문은 이 5파일만의
값이라 자매 머지에 무관하다.

## 이 패스가 병합되면 — 자유 풀이 비고, 남는 것은 전부 사람 게이트

- 판정 완료: 93파일 / 유지 2,049행 → **98파일 / 유지 2,068행**, 누적 순 제거 1,892 → **1,911행**.
- 미판정 잔여: 19파일 / 280행 → **14파일 / 242행**.
- **그 242행은 전부 사람 게이트 3몫이다** — 마이그레이션 8파일 139행(전용 PR · 수동 repair ·
  사람 승인) · D2 저장 계층 4파일 44행 · D6·D5 경로 규칙 2파일 59행. **무인 자유 풀은 0이 된다.**

**이 「자유 풀 0」은 조건부다.** 열린 PR #92·#93 이 in-scope 파일에 주석을 더하고 있고
(#92 는 `backend/src/doc_edit.rs`·`backend/tests/doc_edit.rs` 등 **새 파일**을 들여놓는다),
머지되는 즉시 새 자유 풀이 열린다. **디렉터리·축 단위 종료를 선언하지 않는다** — 11차 패스가
`tools/` 에서, 14차가 `backend/src`·`backend/tests` 에서 배운 그대로다. 다음 감지는 잔여를
**집합 차로 다시 계산**해야 한다.

## 범위 밖 (후속)

- **`backend/src/llm.rs:494-499` 의 주석이 거짓이고 그 거짓이 실제 flake 를 덮고 있다** —
  13차 패스가 등재만 해 둔 그대로다. 「no other test reads this variable, so parallel test runs
  cannot race on it」이라 적혀 있으나 같은 파일 449행의 `stub_answer` 가
  `FEATUREDOC_STUB_LLM_FAIL` 을 읽는다. `llm.rs` 는 원장 1행의 범위라 **증분 재판정**이고, 주석만
  고쳐서는 flake 가 남으므로 **테스트 수정이 따라붙는 별개 작업**이다. 열린 #93 이 `llm.rs` 를
  건드리므로 순서를 그쪽에 맞춰야 한다.
- **`README.md` 의 「secret.yaml은 gitignore」가 사실이 아니다** — `.gitignore` 에는
  `tools/node_modules/` 와 `__pycache__/` 둘뿐이다. 이 패스가
  `kustomization.yaml` 의 secret 문단을 ②로 걷으면서 대조하다 발견했다. **문서 자체의 품질은 이
  모델의 판정 표면 밖**이라(정책 「범위」 절) 여기서는 등재만 한다 — 다만 `secret.yaml` 이 실제로
  무시되지 않는다면 자격증명이 커밋될 수 있으므로 **별개 작업으로 `.gitignore` 를 고치는 쪽**이
  맞다(정책 「복원이 안 되는 것이면 원본을 고친다」).
