# 판정 상세 — 프런트 데이터·셸 축 `frontend/src` 전량 8파일 (2026-09-20)

판정 범위: `frontend/src/api.ts` · `frontend/src/App.tsx` ·
`frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` ·
`frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` ·
`frontend/src/index.css` · `frontend/src/format.ts`

판정 시점 트리: `79c1cb4`(main tip). 범위의 주석 **340행 → 80행**, 순 제거 **260행**.
여기에 원장 3행의 **증분 재판정 ②**(순 제거 2행)와 9행의 **증분 재판정 ①**(순 제거 2행)을 더해
이 패스의 총 순 제거는 **264행**이다.

규칙은 [README.md](../README.md), 결과 표면은 [ledger.md](../ledger.md)에 있다.

## 이 범위를 고른 이유

**이 범위는 `frontend/src` 를 닫는다.** 직전 9차 패스가 예고한 후보는 「프런트 데이터 계층을 한
범위로」(`api.ts` 96 + `App.tsx` 53)였고 detector 브리프도 6파일 / 323행을 넘겼는데, 계획 시점에
`frontend/src` 전수를 재보니 **미판정 파일이 그 6개에 `SignIn.tsx`(12) · `format.ts`(5) 둘을
더한 8개뿐**이었다. 둘을 빼면 디렉터리에 12·5행짜리 조각이 남아 다음 패스의 범위가 「나머지」가
된다. 17행을 더 넣어 **디렉터리 단위로 완결되는 범위**를 만드는 쪽을 골랐다.

나머지 7개 `frontend/src/*.tsx`(`FeatureAcceptance` · `FeatureDependencies` · `AnalysisDiff` ·
`AnalysisProgress` · `DiscoveryStrategy` · `FeatureCandidates` · `CrossCuttingConcerns`)는
원장 4·6·7·8·9행이 이미 판정했고, `main.tsx` · `vite-env.d.ts` 는 판정 대상 주석이 0행이다
(후자의 `/// <reference ...>` 는 기계 판독이라 지문·판정 모두에서 제외된다).

한 화면의 전 층이 아니라 **한 디렉터리의 전 파일**을 범위로 잡은 덕에, 앞 패스들이 층 사이에서
보던 복제를 이번에는 **화면 사이에서** 볼 수 있었다 — 「권한 부여 → 키 등록」 인계 계약이 네 벌,
「해시가 주소가 되는 이유」가 세 벌이다(아래 ③).

### 경합은 0이다 — 열린 PR 이 하나뿐이고 파일이 겹치지 않는다

원장 「미판정 잔여」의 ② 경합 블록은 `tools/check-mockup-render.py`(#26) ·
`frontend/src/index.css`(#17) · `frontend/src/HomeRepositories.tsx`(#17) ·
`frontend/src/CredentialsSetup.tsx`(#17) 4파일 / 217행이라 적혀 있었다. **그 분류는 낡았다** —
계획 직전 `/pulls?state=open` 전수 실측에서 **#17 과 #26 은 둘 다 CLOSED**(머지 아님)이고,
열린 PR 은 **#64 하나뿐**이다.

| 열린 PR | 건드리는 파일 | in-scope 주석 파일인가 |
|---|---|---|
| #64 (`ci: 저장 형식 무변경 PR 에 commit status`) | `.github/workflows/data-format-review.yml`(신규) · `README.md` · `tools/check-data-format-change.py`(신규) | 아니다 — 워크플로·마크다운은 비대상이고 `tools/` 쪽은 **신규 추가뿐** |

그래서 `index.css` · `HomeRepositories.tsx` 는 이번에 경합 없이 집을 수 있었다. `CredentialsSetup.tsx`
는 #83 이 `GrantRepoAccess.tsx` + `RegisterLlmKey.tsx` 로 쪼개며 **파일 자체가 사라졌다**.

## 제거한 것 — 복원 경로별

### ① 선언·시그니처 재진술 (복원 경로 ① 코드 자체)

`api.ts` 가 이 유형의 집중 구간이었다. 96행 중 대다수가 **바로 아래 선언을 영어로 옮겨 적은
JSDoc** 이다.

| 지운 주석 | 아래 선언이 이미 말하는 것 |
|---|---|
| `/** Current user, or null when unauthenticated (401). */` | `Promise<User \| null>` + `if (res.status === 401) return null;` |
| `/** One pipeline step of an analysis (Analysis Progress). ``pending`` until a worker runs it. */` | `type Stage` + `status: 'pending' \| 'running' \| ...` |
| `/** Resolves a typed target and estimates the analysis scale before triggering. */` | `preflightAnalysis(repoUrl, branch): Promise<Preflight>` |
| `/** One place the next stage will look. ``user`` entries are the reviewer's own. */` | `type StrategyEntry` + `source: 'generated' \| 'user'` |
| `/** 달라진 인수 시나리오 한 줄. ``mark`` 는 ``+``(추가) 또는 ``-``(제거). */` | `type ScenarioLine = { mark: string; text: string }` |
| `/** A repository row: the repo itself plus its most recent analysis, if any. */` | `type Row = { ... latest: Analysis \| null ... }` |
| `// undefined = still loading the session` | `useState<User \| null \| undefined>(undefined)` |

`format.ts` 는 **5행 전건이 이 유형**이라 판정 후 주석이 0행이 됐다 — `formatCost` 의
`/** Cents → $1.23 ... */` 는 함수 이름과 `cents / 100` 이, `formatAgo` 의 `/** Unix seconds →
4m ago ... */` 는 `unixSeconds` 파라미터 이름과 본문 분기가 그대로 말한다.

### ② AC 조항·시나리오·목업 카피 인용 (복원 경로 ② 저장소 문서)

- `api.ts` 의 절 제목 5개가 전부 AC 꼬리표를 달고 있었다(`// ── analyses (AC1.1) ──`,
  `// ── pipeline documents (AC1.2) ──`, `// ── discovery strategy (AC1.3) ──`,
  `// ── acceptance scenarios (AC2.1 · AC2.2 · AC2.3) ──`, `// ── 재분석 diff (AC2.6) ──`).
  절 제목은 유형 ④(구분선)이고 AC 꼬리표는 이 유형이라, 한 줄이 두 근거로 제거된다.
- `App.tsx` 의 `analysisRouteFromHash` doc 12행은 **라우트 표를 산문으로 옮긴 뒤**
  (`#/analyses/<id>` → Analysis Progress, `.../cross-cutting` → …) 목업 카피
  「여기까지 저장하고 나가기」와 여정 문장 「나중에 특정 기능의 문서가 미심쩍어서 다시 들어옴」을
  인용한다. 표는 바로 아래 정규식 두 개가, 인용문은 `docs/mockups/` 와 `docs/user-journey/` 가
  가지고 있다.
- `HomeRepositories.tsx` 머리 18행은 `STP-confirm-cost` 터치포인트 문장
  (「저장소 연결(pre-flight 추정 영역 + 시작 버튼)」)과 `AC1.1` · `AC4.6` · `test/01 시나리오 2` 를
  차례로 인용한다.
- `index.css` 의 `design-system` 조항 인용(`§4.1` · `§4.2` · `§4.7` · `§4.11` · `§4.12` ·
  `§5.2` · `3.1: 22px gutters` · `2.3`)과 목업 URL 인용 7건.

### ③ 여러 벌로 복제된 명제 — 한 벌만 남겼다

범위를 디렉터리 전체로 잡자 **화면 파일 사이의 복제**가 드러났다.

| 명제 | 발견된 벌 | 남긴 자리 |
|---|---|---|
| 「권한 부여는 설치가 서면 스스로 키 등록으로 넘기고, 되돌아올 땐 그 인계를 눌러야 한다 — 아니면 두 화면이 서로 튕긴다」 | **4벌** — `App.tsx` 머리 · `App.tsx` `handOff` 선언 · `App.tsx` `openCredentials` doc · `GrantRepoAccess.tsx` `useEffect` | `App.tsx` 의 **`handOff` 상태 선언 옆** — 함정이 사는 자리가 그 상태다 |
| 「주소로 갈 수 없는 화면은 그 내용이 서버 상태임을 보일 수 없다(그래서 해시)」 | **3벌** — `App.tsx` 머리 · `analysisRouteFromHash` doc · `openDiff` doc | `App.tsx` **머리** — 파일 전체의 설계 결정이다 |
| 「404 는 『아직 안 만들어졌다』이고 『돌았는데 없다』와 다른 상태다」 | **4벌** — `getCrossCutting` · `getDiscoveryStrategy` · `getAcceptance` · `CandidateList.extracted` | `getCrossCutting` 에 한 벌 + `getAcceptance` 에 **다른 명제**(404 를 던지지 않고 `null` 을 주는 이유)만 |
| 「정적 목업에는 in-flight 상태가 없어서 대기 문구를 만들지 않았다」 | **2벌** — `index.css` `.btn-link` 블록 · `HomeRepositories.tsx` 버튼 안 JSX 주석 | 없음 — 목업에 없는 것을 안 그렸다는 사실은 원장(편차)의 자리다 |

### ④ 작업 흔적 — 슬라이스 번호 · task id · 판정일 (복원 경로 ③④)

`#83`(수렴 슬라이스 ⑦)이 들여온 주석이 이 유형의 최신 표본이다.

- `RegisterLlmKey.tsx` / `GrantRepoAccess.tsx` 머리의
  `Split out of CredentialsSetup.tsx by 수렴 슬라이스 ⑦ (rct_20260919-0007)`
- 같은 머리의 `the 2026-09-18 authority-order ruling` — 판정일과 판정 결과는
  `docs/doc-tracker/2026-09.md` 의 자리다
- `SignIn.tsx` 머리의 같은 문장(그 위에 **이미 없는 화면** `Credentials Setup` 을 현재형으로
  서술한다 — 낡아서 거짓이 된 주석이고, 유형 ⑥ 을 겸한다)
- `index.css` 의 `슬라이스 ⑥ 이 홈·연결 병합과 함께 …`, `슬라이스 ⑦ 이 권한 부여 화면에서 …`
- `HomeRepositories.tsx` 머리의 `slice ⑥ moves the composition back onto the mockup` ·
  `Reads the slice-2a enqueue contract`

### ⑤ 구분선 주석 (절 제목)

`index.css` 에서 지운 42행 중 **33행이 절 제목**이었다 —
`/* ── app bar — back / title / trailing slot ── */` 처럼 바로 아래 선택자가 이미 말하는 이름표다.
`api.ts` 의 5개(② 참조)와 합쳐 이 패스의 최대 유형이다.

### ⑥ 낡아서 거짓이 된 주석 — 제거 근거를 강화한 것

**`SignIn.tsx` 머리가 존재하지 않는 화면을 현재형으로 서술한다.** 「the unauthenticated branch
leaves Credentials Setup and becomes this screen. Credentials Setup keeps only what it is named
for」 — `CredentialsSetup.tsx` 는 `#83` 이 `GrantRepoAccess.tsx` + `RegisterLlmKey.tsx` 로 쪼개며
**삭제했다**(같은 커밋, `442 −`). 원본이 사라져도 인용은 남는다는, 이 모델이 말하는 실패 그대로다.

## 남긴 것 — 판단이 갈려 보존한 12건

정의의 **비대칭 비용** 원칙(애매하면 남긴다)을 적용한 줄이다. 전부 「코드만 보면 실수로 보이는
의도적 선택」이거나 「다른 파일과 함께 움직여야 하는 계약」이다.

1. `api.ts` 머리 — **SPA 와 API 가 모든 배포에서 오리진을 공유한다**. 모든 호출의
   `credentials: 'same-origin'` 이 기대는 배포 위상이고, 코드 어디에도 없다.
2. `api.ts` `LOGIN_URL` — **fetch 가 아니라 전체 페이지 이동**이어야 OAuth 리디렉션 연쇄를
   브라우저가 따라간다.
3. `api.ts` `Preflight.hasAccess` — **`false` 는 오류가 아니라 답**이다.
4. `api.ts` `retryStage` — 응답이 이미 초기화된 진행을 실어 오므로 **호출자가 다시 읽지 않는다**.
5. `api.ts` `Analysis.stagesDone` — 두 번째 요청을 없애려고 **목록 행에 비정규화**한 필드.
6. `api.ts` `CandidateList.undecided` — 화면과 「결정 끝」 게이트가 **한 숫자를 읽게** 서버가 센다.
7. `RegisterLlmKey.tsx` `activeProviderOf` — **`llmkey::ACTIVE_KEY_SQL` 을 복제**한 규칙이다.
   둘이 같이 움직이지 않으면 화면이 파이프라인과 **다른 키를 이름 댄다**.
8. `RegisterLlmKey.tsx` `formatBad` — 형식 검사가 **제출을 막지 않는다**. 판정 주체는 화면이
   아니라 제공자다(코드만 보면 검사를 걸다 만 것처럼 보인다).
9. `HomeRepositories.tsx` `buildRows` — 설치 범위에서 빠진 저장소의 잡이 **조용히 사라지면 안 된다**.
10. `HomeRepositories.tsx` `signOut` — 실패를 삼키지 않는다. **됐다고 믿는 로그아웃이 안 된 것**이
    끼어들 값어치가 있는 유일한 결과다.
11. `index.css` `.esrc` — `.dep` 아래로 한정한다. **전역으로 풀면 이 규칙을 기대하지 않는 화면의
    렌더가 함께 움직인다**.
12. `HomeRepositories.tsx` `awaiting_pipeline` 라벨 — 큐는 비었지만 LLM 단계가 아직 없어서
    **일부러 `Synced` 가 아니다**.

## 보존한 기계 판독 주석

- **화면 머리의 목업 매핑 4건** — `check-mockup-render.py::discover_screens()` 가
  `frontend/src/*.tsx` 의 **앞 2,000자**에서 읽는 M1 의 입력이다:
  `GrantRepoAccess.tsx:2` · `RegisterLlmKey.tsx:2` · `HomeRepositories.tsx:2,3` ·
  `SignIn.tsx:2`. 이 줄들은 **바이트 그대로** 두었고, 위쪽 머리 문단을 지웠으므로 2,000자 창
  안에 더 여유 있게 들어온다. `index.css` 의 목업 URL 은 `.tsx` 가 아니라 게이트 입력이 아니고,
  그래서 지웠다(9차 패스가 `.rs` 에서 확인한 것과 같은 경계).
- `frontend/src/vite-env.d.ts` 의 `/// <reference types="vite/client" />` — 제외 패턴에 있어
  지문·판정 모두에서 빠진다. 손대지 않았다.

## 지문에 보이지 않은 것 — `index.css` 의 오탐 1건과 블록 주석

- `index.css:48` 의 `* {`(전역 선택자)가 지문 패턴 `\*([[:space:]]|$)` 에 걸려 **주석으로 세어진다**.
  README 「지문과 사각지대」가 말하는 오탐이고 결정적이라 무해하다. **코드이므로 건드리지 않았다** —
  이 패스 뒤에도 `index.css` 의 지문 3행 중 한 행은 이 줄이다.
- 반대로 `index.css` 머리의 12행 블록 주석은 **첫 줄만 지문에 보인다**(이어지는 줄이 공백+글자로
  시작해 패턴에 안 걸린다). 그래서 diff 기준 삭제(367행)가 지문 기준 순 제거(264행)보다 크다.

## 증분 재판정 ② — 원장 3행 (e2e 하네스)

`#83` 이 `sc01-01-full-pipeline-run.spec.ts` 에 **순증 2행**을 열었다(142 → 144). 화면 분할 뒤의
진입 경로를 4행으로 다시 서술한 주석이다.

- **제거** — 「슬라이스 ⑦ 이후 그 경로는 두 화면이다」(작업 흔적) · 「권한 부여 화면이 스스로 키
  등록으로 넘기고 … `저장하고 계속` 한 번이 …」(`App.tsx`·`GrantRepoAccess.tsx` 코드가 말한다) ·
  「(선례: sc01-05)」(이름으로 복원되는 교차 참조).
- **유지** — 「셋업을 API 로 끝냈어도 로드는 자격증명 화면에서 시작한다 — 라우팅이 서버 게이트가
  아니라 상태 머신이기 때문이다」. `page.goto('/')` 직후 왜 버튼을 눌러야 하는지는 스펙 어디에도
  없다.

3행은 **142행**으로 돌아왔으나 **지문은 `0e5c3d31…` 가 아니라 `93be69ea…`** 다 — 살아남은 줄의
문면이 `#83` 이전과 다르다. 9차 패스가 같은 함정을 적었다: 줄 수 일치를 원상 복구로 읽지 말 것.

## 증분 재판정 ① — 원장 9행 (파이프라인 · 횡단 관심사)

`#83` 이 `sc01-02`(+1) 와 `sc01-05`(+2) 에 **순증 3행**을 열었다(139 → 142).

- `sc01-02` — 「같은 버튼(목업의 `저장하고 계속`)이, 입력이 빈 채로 눌리면 pre-flight 로 …」
  2행에서 **목업 카피 인용을 걷고 1행**으로. 남긴 이유: 같은 `register-key` 를 연달아 두 번
  클릭하는 코드는 주석 없이는 복사 실수로 읽힌다.
- `sc01-05` — 「… (슬라이스 ⑦ 의 두 화면 분할)」 2행에서 **작업 흔적을 걷고 1행**으로.

9행은 **140행** / `03babd78…` 이다. 판정 이전 값(139)으로 돌아가지 않는다 — 위와 같은 이유다.

## 증분 재판정 — 원장 10행에 #91 이 연 +5행 (2026-09-21 · `rct_20260921-0002`)

**창.** `#91`(`3567755`, 반응형 레이아웃 — Compact·Medium·Expanded 3단 배치)이
`frontend/src/index.css` 에 지문 기준 **순증 5행**(물리 9줄)을 얹었다(10행 80 → 85). 지문 원본 diff 는
`>` 5줄뿐이고 파일 집합 불변이라 판정 대상은 이 5행이 전부다. 같은 커밋이 `docs/design-system.md` 에
§3.4 Responsive layout · §4.7 의 rail 변형 · §5.4 Responsive shell 을 신설했으므로 **복원 경로 ②가
같은 커밋 안에서 열렸다** — 증분 재판정 ①(#85) 이 「같은 슬라이스가 같은 동작을 세 번 적은」 형이었다면
이번은 「구현이 자기 문서를 다시 적은」 형이다. 원장 규약대로 새 행을 만들지 않고 10행의 결과 칸을 갱신한다.

| 위치 (`3567755` 트리) | 주석 | 판정 | 근거 |
|---|---|---|---|
| `index.css:728-733` (지문 1행 · 물리 6줄) | `/* ── Responsive layout — design-system §3.4 ── Compact(< 600) is the base design above. Wider windows change placement only; tokens, type scale and component internals stay the same. In the browser the app window is the viewport, so the §3.4 thresholds are @media here. (A container on an ancestor would become the containing block of the fixed tab bar, which lives inside .screen, and pin it to the page.) */` | **제거** | 첫 줄은 절 제목(이 패스가 같은 파일에서 33행 지운 유형 ⑤). 본문 두 문장은 §3.4 첫 문단 「모바일(Compact)이 기본 설계이고, 넓은 창은 **배치만** 바꿉니다. 토큰·타이포 스케일·컴포넌트 내부는 폭에 따라 바뀌지 않아요」 와 둘째 문단 「브라우저에서는 앱 창이 곧 뷰포트라서 구현은 같은 기준값의 `@media` 로 분기합니다 … 컨테이너가 그 안의 `position: fixed` 요소(탭바)의 기준 상자가 되어 탭바가 화면이 아니라 페이지에 붙어 버리기 때문이에요」 의 **축자 재진술**(②). 「앱 루트에 container 를 달지 않는 이유」는 함정처럼 읽히지만 문서가 이유까지 적는다. 이름표가 없어져도 §5.4 가 「구현(`frontend/src/index.css` 끝의 Responsive layout 블록)」으로 이 자리를 역참조하므로 문서 → 코드 방향은 닫혀 있다. |
| `index.css:739` | `centered content column; the tab bar is fixed and not part of it` | **제거** | 바로 아래 선택자 `.screen > :not(.tabbar)` + `max-width` + `margin-left/right: auto` 가 그 문장 자체(① — 증분 재판정 ① 이 같은 파일에서 지운 선택자 재진술과 같은 유형). §3.4 표 「콘텐츠 컬럼 560, 가운데」(②). |
| `index.css:746` | `bottom tab bar → left rail: same element, same 4 slots (§4.7)` | **제거** | §4.7 「Medium 이상에서는 좌측 rail — 같은 요소·같은 4 슬롯이 창 왼쪽에 세로로 섭니다 … 새 컴포넌트가 아니라 배치 변형이에요」 의 축자(②)이고 §5.4 CSS 블록에 동명 주석 `/* bottom tab bar → left rail */` 이 한 벌 더 있다(②). 딸린 선언 `width: 72px · flex-direction: column · border-right: 1px solid var(--border-subtle)` 가 §4.7 의 수치 그대로(①). |
| `index.css:770` | `only repeated card lists widen out of the column, into 2 columns` | **제거** | §3.4 불릿 「넓어지는 것은 반복 카드 목록 하나 — … Expanded 에서 … 2열이 됩니다」(②) + 바로 아래 `display: grid; grid-template-columns: repeat(2, …)`(①). |
| `index.css:326` | `the tab bar is fixed chrome, not content: rise's transform would replace its translateX(-50%) centering and push it half off the right edge` | **유지** | `.screen > .tabbar { animation: none; }` 은 코드만 보면 애니메이션 예외 하나로 읽히고, 지우면 다시 깨지는 **이유**(진입 애니메이션 `rise` 의 `transform` 키프레임이 탭바의 `translateX(-50%)` 중앙 정렬을 덮어써 탭바가 오른쪽으로 절반 밀린다)는 코드·`docs/` 어디에도 없다 — 목업은 탭바를 애니메이션에서 빼지 않아 이 충돌 자체가 없다. 본문 「실패 모드의 함정」 유지 대상이고, 이 패스가 같은 파일에서 남긴 11번(`.esrc` 의 한정 이유)과 같은 형이다. **다만 PR #91 본문 「기존 버그 수정」 절이 원인·수정을 같은 문장으로 적는다(③)** — 감지는 「PR 본문에도 없다」고 넘겼지만 실측은 다르다. 그래도 ③ 은 본문이 「작업 흔적의 자리」로 이름 붙인 경로이고 이 문장은 경위가 아니라 CSS 한 줄을 지키는 함정이라, 「애매하면 남긴다」로 보존한다. **판단이 갈려 남긴 것 13건째.** 뒤집으려면 정책 개정(「③ 만으로 복원되는 함정도 제거」)이 선행이지 다음 패스의 재판정이 아니다. |

**증분 5행 중 제거 4 · 유지 1.** 물리 diff 는 `−9 / +0`(블록 6줄 + 인라인 3줄), 비주석 코드 무접촉 —
블록 주석과 빈 줄을 걷어 낸 스트립 잔여가 부모와 바이트 동일(17,220자)이고, `vite build` 산출
`dist/assets/index-BmaRMcZ8.css` 의 sha256(`b8d0f4e5…`)이 부모와 **동일**하다(파일명 해시까지 같다).
게이트 `check-mockup-render.py` 의 M2 는 `:root` 토큰만 읽으므로 이 블록에 닿지 않는다.

### 10행은 판정 전 값으로 돌아가지 않는다

| 원장 행 | #91 이전 (`8205b7a`) | #91 이후 (`3567755` = main `899800e`) | 이 재판정 뒤 |
|---|---|---|---|
| 10행 | 80 / `641e9457…` | 85 / `75b6a8fd…` | **81 / `af9fbb9d…`** |

유지 1행이 남아 80 으로 돌아가지 않는다 — 9차·10차 패스의 경고 그대로 줄 수·지문 칸을 실측값으로
갱신했다(지문 규약: 범위 파일 경로 접두사 포함 · 정규화·정렬 · 후행 개행 포함 sha256 — `641e9457…` ·
`75b6a8fd…` 가 이 규약으로 재현됨을 먼저 확인했다). 전역 as-is 는 `lines=2349`/`e917bb5e…` →
**`lines=2345 files=108`/`97dbbedf…`**(−4, 부모 main `899800e` 기준). 자매 PR #100(`rct_20260921-0001`,
#93 의 9행)이 먼저 착지하면 절대값은 그만큼 더 내려가고 이 패스의 몫은 **차분 −4** 다 — 원장 합계 문단은
나중에 착지하는 쪽이 양쪽 증분을 합쳐 실측으로 다시 쓴다.

### 검증

1. 스트립 잔여 바이트 동일 · `npm run build` CSS 산출물 sha256 동일(위).
2. `python3 tools/check-mockup-render.py` · `check-journey-mockup.py` · `check-scenario-e2e.py` 전부
   rc=0. 코드 편집 직후엔 셋 다 출력이 부모와 `diff` 0 이었고, 원장·패스 문서를 쓴 뒤에는
   `check-journey-mockup.py` 의 「docs/ 상대 링크」 계수만 303 → 304 로 움직였다(원장 10행이 이 절로
   거는 링크 1건 — 그 검사가 링크의 실재를 확인한 것이다). `node tools/check-journey-prototype.js`
   「여정 프로토타입 5개 · 단언 408건 실행 · 통과」.
3. `python3 tools/check-data-format-change.py --base main --head <branch>` → `✅ 변경 없음`(D-규칙
   경로 무접촉).

## 이 패스 뒤의 잔여

판정한 파일은 **63개**이고, 그중 `format.ts` 는 이 패스로 주석이 0행이 되어 지문의 파일 집합에서
빠졌다 — 그래서 지문 기준으로는 **62파일 / 1,804행**이다. 전역 `lines=2680 files=110` 기준
**미판정 잔여 48파일 / 876행**. `frontend/src` 는 이 패스로 **전량 판정 완료**다. 잔여는 둘로
갈린다(경합은 0이라 분할이 아니다).

- **`backend/migrations/*.sql` 8파일 / 139행** — README 「적용된 마이그레이션」 절의 전용 PR ·
  수동 repair · 사람 승인 게이트를 거치는 **별도 패스**다. 이번 트리거로 움직이지 않았다.
- **자유 풀 40파일 / 737행** — 등록 이래 가장 넓다. 큰 후보는
  `tools/check-mockup-render.py` 91(#26 이 CLOSED 되며 풀렸다) · `tools/check-journey-mockup.py` 62 ·
  `tools/check-scenario-e2e.py` 44 · `backend/src/github_app.rs` 35 · `backend/tests/worker.rs` 35.
  다음 패스의 자연스러운 한 축은 **`tools/` 체커 전량**이다 — `check-journey-prototype.js` 는 원장
  2행이 이미 판정했으므로 그 축을 집으면 `tools/` 도 디렉터리 단위로 닫힌다.

**경합은 0이지만 잔여는 곧 늘어난다.** 열린 PR #64 는 in-scope 주석 파일에 신규 추가만 하므로
지금은 경합이 아니지만, 머지되면 `tools/check-data-format-change.py`(298행 신규)가 **새 미판정
파일로 잔여에 들어온다**. `tools/` 축을 다음에 집는다면 그 파일이 이미 들어와 있는지부터 잴 것.

## 증분 재판정 ③ — 원장 10행에 #92 가 연 +7행 (2026-09-21 · `rct_20260921-0007`)

`#92`(`51daa9c`, 슬라이스 6a)가 `api.ts` 에 편집 API 네 함수·타입 셋을(+5행), `App.tsx` 에 두 라우트를
(+2행) 더하며 쓴 주석 7행을 판정해 **제거 5 · 유지 2**. 새 화면 둘(`RequestEdit.tsx`·`DecideDiff.tsx`)은
새 파일이라 이 행이 아니라 [2026-09-21-doc-edit-axis.md](2026-09-21-doc-edit-axis.md) 의 새 행이다.

| 파일 | 줄 | 판정 · 복원 경로 |
|---|---|---|
| `api.ts` | `/** 한 시나리오의 세 문장. 서버가 저장·제안·이력에서 모두 이 모양을 쓴다. */` (`type Sentences`) | 제거 — ① 필드 `given`·`when`·`then` · `backend/src/doc_edit.rs` `pub struct Sentences` 의 요약 1줄(정본, pub 요약) |
| `api.ts` | `/** AC3.4 의 출처. 「바꾼 주체」가 읽는 값이다. */` (`EditProposal.source`) | 제거 — ① 이름 · 0009 `source` 문단 · `doc_edit.rs` 의 같은 문장도 함께 걷었다 — AC 꼬리표 |
| `api.ts` | `/** 그 자리에 설 시나리오들. 한 건이면 고쳐 쓴 것이고, 여럿이면 사례가 늘어난 것이다. */` (`EditProposal.after`) | 제거 — ① `doc_edit.rs` `lines()` doc 이 이 구분의 정본(칸 단위 vs 시나리오 단위) |
| `api.ts` | `/** 제안을 만든다. 문서는 승인 전까지 그대로다. */` (`proposeEdit`) | **유지** — export 함수의 JSDoc 요약 1줄(정책 유지 대상; 이 패스가 `requestDependencies` 의 「Records the request and re-queues the analysis.」를 남긴 것과 같은 모양). `doc_edit.rs` `propose()` 의 같은 문장은 private fn 이라 이쪽을 정본으로 걷었다 |
| `api.ts` | `/** 승인이면 그 자리에 얹히고, 거부면 다음 제안이 피해야 할 것이 된다. */` (`decideEdit`) | **유지** — 같은 이유 |
| `App.tsx` | 「`edit` 이 고칠 시나리오의 자리(0-based).」 (`AnalysisRoute.scenarioIndex` 의 JSDoc) | 제거 — ① 0009 「`scenario_index` 는 … 자리(0-based)」 정본 · 해시 정규식 `scenarios/(\d+)/edit` |
| `App.tsx` | 「`proposal` 이 그리는 제안. 화면이 아니라 서버가 들고 있는 값이다.」 (`AnalysisRoute.proposalId` 의 JSDoc) | 제거 — ① `doc_edit.rs` `routes()` 의 「제안도 주소를 가진다 — 화면이 들고 있는 값이 아니라 서버가 들고 있는 값이라야 새로고침이 같은 제안을 다시 그린다」(정본) — 4벌 중 하나 |

결과: 10행의 줄 수·지문은 #92 이전 값(81 / `af9fbb9d…`)으로 **돌아가지 않는다** — 유지 2행만큼
**83 / `ec54e678…`** 이다(트리거 `51daa9c` 의 88 / `4346a888…` 에서 −5). `api.ts`·`App.tsx` 는 주석
제거 후 부모와 바이트 동일(stripper md5 `2995df01` · `2fa2b7b0`), `npm run build` rc=0.

## 증분 재판정 ④ — 원장 10행에 #107 이 연 +7행 (2026-09-21 · `rct_20260921-0010`)

`#107`(`89a1625`, 슬라이스 6b)이 `api.ts` 에 추가 API 네 함수·타입 넷을 더하고 `AcceptanceScenario.source` ·
`FeatureAcceptance.location` 의 타입을 넓히며 쓴 주석 7행을 판정해 **제거 5 · 유지 2** — ③ 과 같은 모양이다.
새 화면 `AddFeature.tsx` 는 새 파일이라 이 행이 아니라 [2026-09-21-feature-add-axis.md](2026-09-21-feature-add-axis.md)
의 새 행이다.

| 줄 | 판정 · 복원 경로 |
|---|---|
| `/** 두 자동 패스, 또는 사람이 직접 더한 feature 의 출처(AC3.2·AC3.4). */` (`AcceptanceScenario.source`) | 제거 — ① 유니언 리터럴 `'logic' \| 'test' \| 'user_llm' \| 'user_direct'` 자체 · 0010 `source` 문단 — AC 꼬리표 |
| `/** \`null\` 은 사람이 근거 없이 직접 더한 feature 다 — 위치가 없는 것이 기록된 사실이다. */` (`FeatureAcceptance.location`) | 제거 — ① 0010 `key` 문단 「사람이 더한 feature 는 위치가 없을 수 있으므로(근거 없음)」 · `feature_add.rs` `feature_json` 의 `"location": draft.scenarios.first()…` · ② 여정 `JRN-review-feature` 예외 표(「근거 없음」은 기록된 사실) |
| `/** 근거를 찾았는가. 「근거 있음」 배지와 「근거 없음」 안내가 이 값으로 갈린다. */` (`FeatureAddition.evidenceFound`) | 제거 — ① `AddFeature.tsx` 의 조건 렌더 · `feature_add.rs` `AdditionView.evidence_found` 의 같은 문장도 함께 걷었다 |
| `/** AC3.4 의 출처. 확정 전에는 아직 어느 쪽도 아니다. */` (`FeatureAddition.source`) | 제거 — ① 0010 「확정 전에는 아직 어느 쪽도 아니므로 NULL」 · Rust 쪽 같은 문장 함께 제거 — AC 꼬리표 |
| `/** 확정될 목록의 수 — 승인된 후보 + 직접 추가. */` (`FeatureAdditions.finalCount`) | 제거 — ① 이웃 필드 `approvedCandidates`·`confirmedAdditions` · `feature_add.rs` `list()` 의 `final_count: approved + confirmed`(Rust 쪽 `ListView` doc 3행도 함께 걷었다) |
| `/** 초안을 만든다. 근거를 못 찾으면 초안은 비어 오고, 문서는 확정 전까지 그대로다. */` (`draftAddition`) | **유지** — export 함수의 JSDoc 요약 1줄(정책 유지 대상; ③ 이 `proposeEdit` 를 남긴 것과 같은 모양). `feature_add.rs` `draft()` 의 같은 문장은 private fn 이라 이쪽을 정본으로 걷었다 |
| `/** 확정이면 feature 가 되어 문서 끝에 얹히고, 취소면 시도로만 남는다. */` (`decideAddition`) | **유지** — 같은 이유. `feature_add.rs` `decide()` 와 `AddFeature.tsx` 내부 `decide` 의 같은 문장은 이쪽을 정본으로 걷었다 |

결과: 10행의 줄 수·지문은 #107 이전 값(83 / `ec54e678…`)으로 **돌아가지 않는다** — 유지 2행만큼
**85 / `89959a07…`** 이다(트리거 `89a1625`~`269a5f2` 의 90 / `d88323c6…` 에서 −5). `api.ts` 는 주석 제거 후
부모와 바이트 동일(stripper md5 `2cceebc5`), `npm run build` rc=0.

## 증분 재판정 ⑤ — 원장 10행에 #112 가 연 +2행 (2026-09-21 · `rct_20260921-0012`)

`#112`(`fc6d191`, 슬라이스 6c)가 `api.ts` 에 삭제 API 세 함수·타입 셋을 더하며 쓴 주석 2행을 판정해 **전건
제거** — ③·④ 와 같은 모양이나, 이번엔 export 함수의 JSDoc 요약을 #112 가 더하지 않아 유지분이 없다. 새 화면은
없고(기존 두 화면에 얹혔다) 새 파일 쪽 판정은 [2026-09-21-feature-delete-axis.md](2026-09-21-feature-delete-axis.md).

| 줄 | 판정 · 복원 경로 |
|---|---|
| `/** AC3.3 — 같은 대상의 앞선 분석에서 지웠고 아직 되돌리지 않은 자리. 표시일 뿐 결정이 아니다. */` (`Candidate.previouslyDeleted`) | 제거 — ① 이 값을 채우는 `feature_delete::previous_deletion` 의 pub doc(정본) · ② doc-tracker 6c 행 「표시일 뿐 결정이 아니다」 · 이웃 필드 `previouslyRejected` 에는 doc 이 없다 — AC 꼬리표 |
| `/** 보관소의 한 건(AC3.3) — 지운 feature 는 곧바로 없어지지 않고 \`restoreUntil\` 까지 되돌릴 수 있다. */` (`FeatureDeletion` 타입) | 제거 — ① 필드 `restoreUntil` · `restorable` · ① 0011 머리 「보관 기간 안에는 되돌릴 수 있으며」 · ② doc-tracker · 이웃 타입 `FeatureAddition`·`PreviousRejection` 에는 doc 이 없다 — AC 꼬리표 |

결과: 10행의 줄 수·지문이 #112 이전 값 **85 / `89959a07…`** 으로 되돌아왔다(트리거 `fc6d191` 의 87 /
`673c2f59…` 에서 −2 — 부모 `bf48b45` 재계산과 바이트 동일). `api.ts` 는 주석 제거 후 부모와 바이트
동일(stripper md5 `9f2f4f0f`), `npm run build` rc=0.

## 증분 재판정 ⑥ — 원장 10행에 #108 이 연 +6행 (2026-09-22 · `rct_20260922-0001`)

사람 PR **#108**(AC4.9 출력 언어 설정)이 `frontend/src/api.ts` 2 · `frontend/src/RegisterLlmKey.tsx` 4 =
**6행**을 들여왔다. 판정 맥락은 [2026-09-22-output-language-axis.md](2026-09-22-output-language-axis.md).
**제거 3 · 유지 3.**

| 자리 | 판정 | 근거 |
|---|---|---|
| `api.ts` `/** The languages the backend's `llm::Language` accepts. */` (바로 아래 `export type LlmLanguage = 'ko' \| 'en';`) | **제거 1** | 선언 재진술(①) — 주석이 말하는 목록을 선언이 그대로 적는다. 백엔드 타입으로의 교차 참조도 이름에서 복원된다 |
| `api.ts` `Analysis.llmLanguage` 의 `/** Fixed when the run was triggered; `null` for a run that predates the setting. */` | **제거 1** | 두 문장 모두 사본 — 스냅숏 명제(정본 `analysis.rs::create`)와 일곱 벌 명제(정본 `settings.rs::analysis_language`) |
| `RegisterLlmKey.tsx` `// Order is display order; labels are each language's own name for itself.` | **제거 1** | 선언 재진술(①). 배열의 순서가 곧 표시 순서이고(바로 아래 `.map`), 라벨이 각 언어의 자기 이름이라는 것은 리터럴 `'한국어'` · `'English'` 가 말한다. 바로 위에 남아 있는 provider 쪽 주석(「Screen-only: this order and its first entry decide what a user who has never chosen starts on …」)은 *기본값을 정한다*는 다른 명제라 영향 없다 |
| `RegisterLlmKey.tsx` `// Saved on tap: the choice is independent of the key form, so it must not wait for — or be lost with — "저장하고 계속".` 2행 | **유지 2** | 선택이 키 등록 폼과 **독립**이라는 것은 `selectLanguage` 본문에서 복원되지 않는다(폼 제출 경로가 다른 함수다). 「저장하고 계속」과 함께 잃으면 안 된다는 금지도 코드가 말하지 않는다 |
| `RegisterLlmKey.tsx` `// `null` until the stored value arrives, so no option reads as chosen before then.` 1행 | **유지 1** | `useState<LlmLanguage \| null>(null)` 은 초기값만 말하고, *왜* `null` 인지(도착 전에 어느 선택지도 선택으로 읽히면 안 된다)는 `disabled={language === null}` 과 `checked={l.id === language}`(`:289-294` 의 native radio) 를 묶어야 보인다. 정책의 「애매하면 남긴다」 |

지문: **91행 `e4a3bdeb…` → 88행 `fd84aea84c1e740650d84f0eb07f00721c5b61153c59a7747d88bf607585b2e9`**.

## 증분 재판정 ⑦ — 원장 10행에 #119 가 고쳐 쓴 1행 (2026-09-22 · `rct_20260922-0004`)

자매 슬라이스 **#119**(목업↔구현 수렴 — 출력 언어 선택을 실제 폼 요소로, `rct_20260922-0003` /
`tbm_feature-doc-mockup-render`)가 `<button aria-pressed>` 2개를 `<label><input type="radio" name="out-lang">`
2개로 바꾸면서, 바로 위 **유지 1행**의 낱말 하나를 따라 고쳤다.

> `// `null` until the stored value arrives, so no ~~button~~ **option** reads as chosen before then.`

**전건 유지 · 순 제거 0 · 주석 줄 수 88행 불변.** 주석의 **추가·삭제는 0행**이고 제자리 수정 1행뿐이라
행 열의 합(2,350)과 「전역 − 잔여」 산술은 움직이지 않는다 — 이 절이 존재하는 이유가 그것이다.
판정은 ⑥ 의 결론을 그대로 유지한다: 초기값이 `null` 이라는 사실은 `useState<LlmLanguage | null>(null)`
이 말하지만 *왜* `null` 인지는 말하지 않고, 그 이유는 `disabled={language === null}` 과
`checked={l.id === language}` 를 묶어야 보인다(복원 경로 ①이 닿지 않는다). 정책의 「애매하면 남긴다」.

`aria-pressed` 는 이 레포의 판정 대상 주석에서 사라졌고 `tools/check-journey-prototype.js:222` 의
블록 주석에만 **현존**한다(그 인용은 낡지 않았다 — [2026-09-18-uncontested-harness-config.md](2026-09-18-uncontested-harness-config.md)).

지문: **88행 `fd84aea84c1e740650d84f0eb07f00721c5b61153c59a7747d88bf607585b2e9` → 88행
`7e6915aac99e80dc5c903a0a962066d446091e80602ff26ee65d21f84cd6bd60`**.

## 증분 재판정 ⑧ — #114(슬라이스 6d)가 `api.ts` 에 연 +3행 · 2026-09-22

reconciler task `rct_20260922-0005`. **전건 제거 3행.**

- `DocConflict` 타입 JSDoc(「자동 재분석이 사람이 고쳐 둔 문장과 같은 자리에서 갈린 것(AC3.5). 열린 동안
  문서에는 자동 결과가 선다.」) — 정본은 `backend/src/doc_conflict.rs`(같은 패스에서 모듈 머리 요약을 유지).
  `(AC3.5)` 꼬리표는 ③, 뒤 문장은 doc-tracker 6d 행의 「열린 동안 문서에는 자동 결과가 서고 …」 축자.
- `decideConflict` JSDoc(「`auto` 는 그대로 닫고 `mine` 은 사용자 문장을 다시 세운다 — 둘 다 고른 순간이
  결정이다.」) — doc-tracker 6d 행의 「결정 셋」 문장의 사본.
- `proposeMerge` JSDoc(「합친 문장을 제안받는다. 제안은 결정이 아니라 확정해야 문서에 선다.」) —
  `doc_conflict::merge` 의 요약(유지)과 같은 명제.

셋 다 **전송 경계(API 타입·함수)의 재진술**이라 ⑥ 의 「한 명제의 일곱 벌 — 나머지 여섯 벌은 전송 경계의
재진술이라 제거했다」와 같은 잣대다. ⚠️ ③·④ 가 「export 함수 JSDoc 요약 1줄은 유지」로 남긴 선례와 갈리는데,
그 둘은 **요약이 함수 이름 너머를 말하는 경우**였다. 여기 두 함수의 JSDoc 은 결정 규약 자체를 다시 적는
본문이라 요약 1줄의 자리가 아니다 — 이름(`decideConflict` · `proposeMerge`)이 이미 요약이다.

줄 수·지문이 **#114 이전 값 88 / `7e6915aa…` 로 바이트 동일 복귀**했다.
맥락 [2026-09-22-conflict-axis.md](2026-09-22-conflict-axis.md).

## 증분 재판정 ⑨ — 원장 10행에 #132 가 연 +2행 (`index.css`) · 2026-09-24

reconciler task `rct_20260922-0009`. **순 제거 1행 · 유지 1행**(판단 갈림 1건).

행 기재값 88 / `7e6915aa…` 는 트리거 `27d9b81` 에서 바이트 일치하고 HEAD `074c325` 에서
90 / `fa276b84…` 로 갈린다. 지문에 들어오는 것은 블록 주석의 **여는 줄뿐**이므로(`^\s*/\*`)
물리적으로는 6행이지만 지문상 `+2` 다.

### 제거 1행 — `.ev` 이관 블록 (네 겹 복원)

```
/* 횡단 관심사의 근거 줄 — 목업 `.ev` 를 옮긴다. 목업은 경로를 하나만 그리지만 구현은
   근거 경로를 전부 그리므로, 경로는 한 줄에 하나씩 두고 긴 경로는 아무 데서나 꺾어
   카드 밖으로 새지 않게 한다. 이름 칸은 최소 42% 를 지켜 한 글자씩 세로로 쌓이지
   않는다. `.ename`·`.esrc` 는 `.ev` 아래로 한정한다(위 `.dep` 과 같은 방침). */
```

- 「목업 `.ev` 를 옮긴다」 — 목업 파일 자신(②)과 PR #132 「근거 줄을 목업 `.ev`/`.ename`/`.esrc`
  구조로 이관」(③).
- 「목업은 경로를 하나만 그리지만 구현은 근거 경로를 전부 그린다」 — `frontend/src/CrossCuttingConcerns.tsx`
  머리의 **유지된 정본**(「an item renders *every* path it cites where the mockup draws one」)의 두 벌째.
  그 자리는 목업 대조 예외를 적는 자리로 이미 유지 판정을 받았다.
- 「이름 칸은 최소 42%」 — 바로 아래 `.ev .esrc { max-width: 58% }` 의 **산술 여집합**이라 리터럴 재진술(①).
  값이 한쪽만 바뀌면 조용히 거짓이 되는 형태이기도 하다.
- 「`.ename`·`.esrc` 는 `.ev` 아래로 한정한다(위 `.dep` 과 같은 방침)」 — 같은 파일 위쪽
  `/* `.esrc` 는 `.dep` 아래로 한정한다 — 전역으로 풀면 … */` 의 두 벌째이고, **이유까지 적는 쪽은 그쪽**이다.

### 유지 1행 — 판단 갈림 (iOS Safari 텍스트 자동 확대)

```
/* iOS Safari 는 넓게 번진 블록의 글자를 제멋대로 키운다(text autosizing).
   한 줄이 넘쳐도 글자 크기는 디자인 시스템 값 그대로 둔다. */
```

`-webkit-text-size-adjust: 100%` 라는 선언은 **무엇을** 하는지만 말하고, 그 선언이 없으면 무슨 일이
벌어지는지는 상류 브라우저의 **문서화되지 않은 동작**이다. 복원 경로를 하나씩 재면:

- ② `docs/design-system.md` §5.3 「Base reset」은 `-webkit-text-size-adjust: 100%` · `text-size-adjust: 100%`
  **두 줄을 선언으로만** 옮겨 적고 iOS·autosizing 을 한 번도 언급하지 않는다 ⇒ **반쪽**이다.
- ③ PR #132 본문은 「iOS 텍스트 자동 확대로 그 줄만 글자가 커짐」으로 **한 스크린샷(IMG_8657)의 증상**으로만
  적는다 ⇒ 일반 동작 서술과 「디자인 시스템 값을 유지한다」는 의도는 복원되지 않는다.
- ① 코드에는 이 동작을 강제·단정하는 것이 없다(CSS 선언 자체가 유일한 흔적).

정책 본문의 「복원 불가능한 지식 — 상류의 문서화되지 않은 동작」에 해당하고, 「**애매하면 남긴다**」의
비대칭 비용(유일한 지식을 지우면 어디에도 남지 않는다)이 그대로 걸린다. **판단 갈림 1건으로 남긴다.**

> 원본을 고쳐 복원 가능하게 만드는 길(정책 본문 「원본이 부실하면 원본을 고친다」)은 열려 있다 —
> `docs/design-system.md` §5.3 에 이 두 선언이 왜 있는지 한 줄을 붙이면 다음 패스에서 ② 로 닫힌다.
> 이번 범위(주석 판정)를 넘으므로 아래 「범위 밖」에 남긴다.

### 값

88(기재 · 트리거 바이트 일치) → 90(유입 후) → **89 /
`39baa2ff0e02877a92978a28d415bf7c128f75fc4dfd1fbf396948b781f8c5d6`**.

### 범위 밖 (후속)

- `docs/design-system.md` §5.3 에 `text-size-adjust` 의 **이유** 한 줄을 붙이는 문서 수정 — 붙으면 위 유지
  1행이 ② 로 복원되어 다음 패스에서 제거된다. 이 패스는 주석만 판정하므로 손대지 않았다.
- 열린 PR **#126** 이 `frontend/src/api.ts`(+70) · `frontend/src/App.tsx`(+28) 에 들일 주석 — 10행의 범위라
  착지 뒤 다음 감지의 몫이다.

## 증분 재판정 ⑪ — 원장 10행에 #152 가 연 `.disclosure` 머리 주석 (2026-09-25 · `rct_20260925-0007`, 30차 패스)

자매 모델의 PR **#152**(`ea04fe0`, AC4.4 슬라이스 7a)가 `frontend/src/index.css` 말미에 `.disclosure`
블록을 더하면서 머리 주석 **물리 3행**(지문 1행 — 줄머리 필터는 continuation 2행을 보지 못한다)을 들여왔다.

```
/* 긴 결과의 요약 → 상세 단계 노출(AC4.4). Compact 에서는 제목 줄만 남기고 접히고,
   첫 확장 브레이크포인트부터는 같은 화면이 펼쳐진 채로 선다 — 여는 주체는 화면이
   넘겨 주는 `open` 이므로 이 파일은 표식만 그린다. */
```

**제자리 재작성 — 물리 2행 제거 · 유지 1행.**

| 절 | 판정 | 근거 |
|---|---|---|
| 「긴 결과의 요약 → 상세 단계 노출(AC4.4)」 | 제거 | **AC 조항 재진술** — 정책 본문이 doc 주석의 제거 대상으로 이름 붙인 형태 그대로. PRD `04-platform.md` AC4.4 「긴 분석 결과는 단계적으로 노출(요약 → 상세)된다」 축자(②) |
| 「Compact 에서는 제목 줄만 남기고 접히고, 첫 확장 브레이크포인트부터는 같은 화면이 펼쳐진 채로 선다」 | 제거 | `docs/doc-tracker/2026-09.md` 편차 표 **세 행이 각각 같은 문장을 적는다**(「Compact(< 600px)에서는 그 줄이 목록을 여는 자리가 되어 … 첫 확장 브레이크포인트(600px)부터는 목업과 같이 펼쳐진다」 — `AnalysisProgress` · `CrossCuttingConcerns` · `FeatureAcceptance`) ⇒ ② 의 **네 벌째** |
| 「여는 주체는 화면이 넘겨 주는 `open` 이므로 이 파일은 표식만 그린다」 | **유지**(재작성) | 정책이 유지 대상으로 이름 붙인 「**무엇을 넣지 말라**」. 이 파일에 `display`·`max-height` 를 더해 CSS 로 펼치면 `open` 없이 펼쳐져 **DOM 상태와 보이는 상태가 갈리는데**, 그 파손은 어느 게이트도 잡지 못한다(정적 게이트 4종은 CSS 선언을 보지 않는다) — 조용히 깨지는 유일한 경로다 |

남긴 1행은 그 경로를 이유까지 말하도록 고쳐 썼다:

```
/* 여는 주체는 화면이 넘기는 `open` — CSS 로 펼치면 DOM 상태와 보이는 상태가 갈리므로 표식만 그린다. */
```

③ 로 닫지 않은 이유는 위 두 절과 같은 잣대다 — PR #152 본문도 doc-tracker 변경 이력도 같은 말을 적지만
(「CSS 만으로 펼치면 `open` 없이 펼쳐져 DOM 상태와 보이는 상태가 갈리므로 표식만 CSS 가 그린다」),
**그 문장이 막는 편집은 이 파일 안에서 일어난다.** 반대로 앞 두 절이 막는 것은 없다.

### 값

91(기재 · 부모 `f4b7db3` 에서 바이트 일치) → 92(유입 후) → **92 /
`2739379556afad297c706b16b665633eb7963f05a0d2ca243ef2a6f945e32440`**(판정 후).
**지문 줄 수는 제거 전후가 같다** — 걷어 낸 2행이 줄머리 필터에 보이지 않는 continuation 이기 때문이고
(본문 「지문과 사각지대」), 움직인 것은 살아남은 1행의 문면이다. 물리 줄 수는 3 → 1.

코드 불변은 **블록 주석 인식 stripper** 로 쟀다 — 줄머리 필터는 이 축의 continuation 을 코드로 오인한다.
`/* … */` 스팬을 제거한 잔여가 부모와 **바이트 동일**(md5 `4d98ba8c`)이고, 선언 `;` 619 == 619 ·
규칙 `{` 184 == 184 다. `check-mockup-render.py` rc=0 — **M3A 346 · M3B 314 · M5 33/33 전후 불변**.

## 증분 재판정 ⑫ — 원장 10행에 #156 이 연 `@media (min-width: 600px)` 머리 주석 (2026-09-25 · `rct_20260925-0010`, 32차 패스)

⑪ 이 `.disclosure` 머리 주석에서 AC4.4 축자를 걷어낸 **바로 그 블록 아래**에, #156(`84d312a`)이
접힘 affordance 를 폭으로 자르는 `@media` 블록을 붙이면서 머리 주석 4행(지문 1행)을 새로 들였다.

### 제거 — AC4.4 검증 방법 축자 (물리 2행)

```
/* 마커와 커서는 **접힐 수 있는 폭에서만** 그린다 — `open` 이 이미 펼쳐 놓은 폭의
   목업은 같은 줄을 정적인 `div.section-title` 로 그리고, AC4.4 의 검증 방법도
   「데스크톱은 동일 화면을 확장 적용한다」다. …
```

- 「목업은 같은 줄을 정적인 `div.section-title` 로 그린다」 → `tools/check-mockup-render.py` 의
  M8 `fail()` 문면 「그 폭의 목업은 정적인 `div.section-title` 다」 **축자**(①).
- 「AC4.4 의 검증 방법도 「데스크톱은 동일 화면을 확장 적용한다」다」 → `docs/prd/04-platform.md:36`
  **축자**(②).
- **같은 날 같은 문장이 이미 제거 판정을 받았다** —
  [2026-09-25-residual-pool-closeout.md](2026-09-25-residual-pool-closeout.md) 의 `sc04-06` 표 행
  `:110-111` 이 「② PRD AC4.4 검증 방법 축자 · ① 바로 아래 네 줄이 그것을 집행한다 → **제거 2행**」이다.
  ⑪ 이 이 블록에서 지운 자리에 #156 이 **다른 문장으로 되살린** 꼴이라, 같은 잣대를 그대로 적용한다.

### 유지 — 배치 가드 1행

```
/* 이 블록은 위 규칙들보다 **뒤**에 서 있어야 한다 — 앞으로 옮기면 같은 명세도라
   조용히 무력해진다(게이트 M8 이 그것을 잡는다). */
```

정책이 유지 대상으로 이름 붙인 「이 순서를 바꾸면 무엇이 조용히 깨지는가」다. `docs/doc-tracker/2026-09.md:716`
의 음성 프로브 ⓓ 가 같은 함정을 더 길게 적지만(②), **그 문서는 이 블록을 옮기려는 사람의 편집 지점에서
읽히지 않는다** — 같은 날 `sc04-06 :53`(「세션은 여기서 시작한다」)이 ② 축자임에도 같은 사유로 유지된
선례를 따른다. ⑪ 의 유지 1행(「여는 주체는 화면이 넘기는 `open`」)과도 같은 모양이다.

**집행이 살아 있음을 실측했다** — 판정 후 트리에서 이 `@media` 블록을 `.disclosure > summary::after`
앞으로 옮기면 `check-mockup-render.py` 가 **rc=1 · M8 2건**(`::after { content }` 두 선택자)으로 잡는다.
가드가 가리키는 함정이 문장이 아니라 게이트로 실재한다.

### 값

92(기재 · 창 부모 `a473767` 에서 바이트 일치) → 93(#156 유입 후) → **93 /
`407308c167187925a200ce94eed5243c8e9b8c33b5171bd1eaf32dff3b6a032c`**(판정 후).
**지문 줄 수는 제거 전후가 같다** — 걷어 낸 2행이 continuation 이라 줄머리 필터에 보이지 않고,
움직인 것은 살아남은 1행의 문면이다(⑪ 과 같은 모양). 물리 줄 수는 4 → 2.

코드 불변: `/* … */` 스팬을 제거한 잔여가 부모와 **바이트 동일**(md5 `e50755cb0e304c6122a4a9bac5da3279`).
`check-mockup-render.py` 출력이 부모와 **바이트 동일** — M8 요약 「접힘 4건 · `.disclosure` 선언 6개 ·
600px 잔존 affordance 0건」 불변.

## 증분 재판정 ⑭ (2026-09-26 · `rct_20260926-0002`) — `api.ts` 의 `Spend`·`Usage` 타입 doc 5행

- **기준 트리**: 부모 **`0522f33`**(main, #172 착지 직후)
- **유입원**: PR **#166**(`ca9e14a`, 슬라이스 7d — AC4.6)
- **PR**: #NNN (37차 패스)

| 자리 | 문면(요약) | 판정 | 근거 |
|---|---|---|---|
| `Spend` JSDoc 3 → 1 | 「측정된 지출(AC4.6) — 이 분석의 호출이 실제로 보고한 값이고, Connect Repository 화면이 보여 준 pre-flight `est*` 추측이 아니다」 | **제거 2 · 유지 1** | 유지: 프런트가 **어느 숫자를 그릴지 고르는 자리**이고 `AnalysisDetail` 안에 `spend` 와 `est*` 가 나란히 있어 바꿔 써도 타입이 같다(조용한 파손) — 1줄로 압축. 제거: 「Connect Repository 화면이 보여 준」 경위와 AC 꼬리표는 ② `doc-tracker` · ① `analysis.rs` 의 `Estimate` doc |
| `costCents` 필드 JSDoc 1 | 「측정된 토큰에서 추정한 값이다. 제공자의 청구서는 그쪽 것이다」 | **제거 1** | 「추정이지 청구서가 아니다」의 **정본은 `backend/src/usage.rs` 의 단가 상수 doc**(신설 행 34) — 단가를 고치는 사람이 읽는 자리다. 여기서는 필드 이름(`costCents`)과 `Spend` 요약이 남긴 것 이상을 말하지 않는다 |
| `Usage` JSDoc 1 | 「작업별과 전체별을 한 번에 — `/api/usage` (test/04 시나리오 9)」 | **제거 1** | ① 타입 본문 `{ total, analyses }` 가 그 문장이다 · ② 문서를 **이름으로 지목**하는 인용(원장 행 12 가 「시나리오 원문 축자 인용」으로 닫아 둔 형태) |

`*/` 닫는 줄은 지문 패턴(`\*` 뒤 공백·줄끝)에 걸리지 않으므로 **물리 7행 제거 / 지문 4행 제거**다.

**순 제거 4행 · 유지 1행** · 줄 수·지문 96 → **97 / `e497d739…`**

---

## 증분 재판정 ⑮ (2026-09-26 · `rct_20260926-0012`, 45차 패스) — #174·#179 가 연 8행

- **앵커**: 이 행이 마지막으로 판정을 마친 트리 **`cacb429`**(37차 패스 착지, 97행 / `e497d739…`)
- **유입원**: PR **#174**(`6a572e0`, 슬라이스 7e — AC4.6 누적 표시) · PR **#179**(`9a32b76`, 슬라이스 7f — 단계별 비용)
- **기준 트리**: `71455fd`(main tip, 44차 패스 착지 직후)
- **증분 추출**: 앵커와 tip 에서 이 행의 8파일에 **행 지문 규약 그대로**(줄머리 패턴 · DIRECTIVE 제외 ·
  공백 정규화 · 정렬) 히트를 뽑아 집합 차를 냈다 — **추가 8 · 제거 0 · 공통 92**. 개작(rewrite)이
  없으므로 이 행에서는 gross 와 net 이 같다. 앵커 지문 `e497d739…`·tip 지문 `0c670856…` 둘 다
  원장 기재와 바이트 일치했다(계측기 검증).

| 자리 | 문면(요약) | 판정 | 근거 |
|---|---|---|---|
| `api.ts` `Stage.spend` 필드 doc 1 | 「이 단계 몫의 측정된 지출(AC4.6) — 호출하지 않은 단계는 0」 | **제거 1** | `(AC4.6)` 꼬리표는 ③ PR #179 머리(「슬라이스 7f · AC4.6」)이자 ④ 커밋 제목 축자 · 「호출하지 않은 단계는 0」은 ② `doc-tracker/2026-09.md:738` 축자(「`StageView` 에 `#[sqlx(skip)]` 로 채워 문서 없는 단계는 0 을 읽는다」) · 「이 단계 몫의 측정된 지출」은 ① 필드 이름 `spend` + 같은 파일에서 **유지된** `Spend` 요약(「Measured — not the `est*` pre-flight guess that sits beside it」) · **40차 패스가 backend 쪽 쌍둥이 `StageView.spend` doc 을 같은 근거로 이미 제거**했다(행 `backend/src/analysis.rs` 증분 ①) — 남은 사본을 걷는 것이라 새 판정이 아니라 그 판정의 적용이다 |
| `format.ts` `formatCount` doc 4 → 1 | 「천 단위 구분자만 넣는다 — `toLocaleString` 을 쓰지 않는 이유는 그 출력이 실행 환경의 ICU 에 걸려 Node 에서 만들어 대조하는 e2e 단정과 어긋날 수 있기 때문이다. 여기서 나오는 문자열은 어디서 만들어도 같다」 | **제거 3 · 유지 1** | 제거: 「천 단위 구분자만 넣는다」는 ① 바로 아래 `replace(/\B(?=(\d{3})+(?!\d))/g, ',')` 가 그 문장이고, 「Node 에서 만들어 대조하는 e2e 단정」·「어디서 만들어도 같다」는 ③ PR #174 「주의」 절 축자다. 유지 1행(**판단 갈림**): 어길 사람이 읽는 자리는 **구현 쪽**인데, 사본은 `e2e/tests/sc04-09-per-user-cost-visibility.spec.ts` 의 `count()` 머리에 있고 그 행은 **판정 완료**라 걷으면 그 행의 축이 `—` 로 되돌아간다 — 정본을 여기 세우고 사본을 그대로 둔다. 파손이 조용한 것도 실측이다: CI 의 Node 와 Chromium 은 ICU 가 같아 `toLocaleString` 으로 바꿔도 `sc04-09` 가 **초록인 채** 지나가고, 갈리는 것은 다른 로케일·런타임이다 |
| `HomeRepositories.tsx` 누적 사용량 로드 머리 3 → 1 | 「누적 사용량은 저장소 목록과 다른 주소에서 오고, 없어도 홈이 하는 일은 그대로 선다 — 그래서 위 로드와 묶지 않고 실패를 삼킨다. 묶으면 `/api/usage` 한 곳의 장애가 목록까지 비운다」 | **제거 2 · 유지 1** | 제거: 「다른 주소에서 온다」는 ① 바로 아래 별 `useEffect` 의 `getUsage()` 호출이 말하고, 「없어도 홈이 하는 일(저장소를 고르고 분석을 거는 것)은 그대로 선다」는 ③ PR #174 변경 표의 이 파일 행 축자다. 유지 1행(**판단 갈림**): 「위 로드와 묶지 않는다」는 정책이 유지 대상으로 이름 붙인 「무엇을 넣지 말라」이고, 어겼을 때 **아무 것도 붉지 않는다** — `/api/usage` 가 실패하면서 목록은 성공하는 경로를 재는 테스트·게이트가 전수 **0건**(실측: `api/usage` 히트는 구현 1 · `sc04-09` 의 성공 경로 2 · `backend/tests/usage.rs` 뿐이고 프런트 단위 스위트는 없다). ③ 히트(#174 표 행)가 있지만 편집 지점에서 읽히지 않는다 — 40차 패스가 `backend/src/analysis.rs` 의 「One read for the whole pipeline」을 같은 판별식으로 유지한 자리와 같은 모양이다 |

**④ 실측 0건** — 유입원 두 커밋(`6a572e0`·`9a32b76`)의 메시지는 squash 제목 한 줄 + trailer 뿐이라
이 축에서 공전한다(근거로 쓴 줄 0).

**비주석 코드 무접촉**: 이 행의 8파일 diff 에서 주석 줄과 그에 딸린 JSDoc 구분 줄을 뺀 비주석 변경은
**0줄**이다(`git diff -U0` 필터 실측).

**순 제거 6행 · 유지 2행** · 줄 수·지문 105/`0c670856…` → **99 / `4423b0d47b60e37b1304d93939449a08041f688892e884395bd7d5ed47f43119`**

---

## 원장에서 옮겨 온 증분 재판정 기록 (2026-09-26 형식 이전)

아래는 `ledger.md`의 결과 칸에 쌓여 있던 증분 재판정·정정 기록을 **문면 그대로** 옮긴
것이다. 형식 이전(템플릿 「원장 형식」)이 원장에 표와 「읽는 법」만 두기로 하면서, 각 행의
경위는 그 행의 패스 파일로 돌아왔다. 옮기면서 한 글자도 고치지 않았고 판정을 새로 하지
않았다 — 행을 가리키는 순번도 당시 표기 그대로다.

### 원장 행 3 — `e2e/tests/sc01-01-full-pipeline-run.spec.ts` · `e2e/tests/sc01-06-partial-retry.spec.ts` · `e2e/support/cluster.ts` · `e2e/smoke.sh` · `e2e/playwright.config.ts` (e2e 하네스 비경합 5파일)

**증분 재판정 ②**(2026-09-20): #83 이 `sc01-01` 에 연 순증 2행을 판정해 **순 제거 2행**(슬라이스 번호를 단 작업 흔적 · 화면 전이 서술 · 교차 참조 `(선례: sc01-05)`) · 유지 2행(「셋업을 API 로 끝내도 로드는 자격증명 화면에서 시작한다」는 상태 머신 함정) · 범위는 144행 → 142행이지만 **지문은 `0e5c3d31…` 가 아니라 `93be69ea…`** 다 — [passes/2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md)

### 원장 행 9 — `backend/src/pipeline.rs` · `backend/src/cross_cutting.rs` · `backend/tests/documents.rs` · `backend/tests/progress.rs` · `frontend/src/CrossCuttingConcerns.tsx` · `e2e/tests/sc01-02-repo-out-of-scope.spec.ts` · `e2e/tests/sc01-03-cross-cutting-determinism.spec.ts` · `e2e/tests/sc01-05-resume-after-app-exit.spec.ts` · `e2e/tests/sc02-03-contradiction-separation.spec.ts` (파이프라인 · 횡단 관심사 축 비경합 9파일)

**증분 재판정 ①**(2026-09-20): #83 이 `sc01-02`(+1) · `sc01-05`(+2) 에 연 순증 3행을 판정해 **순 제거 2행**(목업 카피 인용 · 슬라이스 ⑦ 작업 흔적) · 유지 2행(같은 버튼을 연달아 두 번 누르는 코드가 복사 실수로 읽히지 않게 하는 한 줄씩) · 139행이 아니라 **140행**으로 내려온다 — [passes/2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md)

### 원장 행 10 — `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일)

**증분 재판정 ②**(2026-09-21): #91(`3567755`, 반응형 레이아웃)이 `index.css` 의 반응형 블록에 연 순증 5행(물리 9줄)을 판정해 **순 제거 4행**(절 제목 겸 §3.4 축자 재진술 블록 1 — 물리 6줄 · 선택자 재진술 1 · §4.7 축자 1 · §3.4 불릿 축자 1 — 전부 같은 커밋이 신설한 `docs/design-system.md` §3.4·§4.7·§5.4 로 복원) · **유지 1행**(`.screen > .tabbar { animation: none }` 위의 함정 — 진입 애니메이션 `rise` 의 `transform` 이 탭바의 `translateX(-50%)` 를 덮어쓴다; PR #91 본문에도 있으나 「실패 모드의 함정」이라 판단이 갈려 남긴 것 **13건째**) · 유지분이 남아 줄 수·지문은 #91 이전 값(80 / `641e9457…`)으로 **돌아가지 않는다**(81 / `af9fbb9d…`) · 비주석 코드 무접촉(빌드 CSS 산출물 sha256 부모와 동일) — [passes/2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md) 「증분 재판정 — 원장 10행에 #91 이 연 +5행」 · **미판정 증분 없음**

### 원장 행 10 — `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일)

**증분 재판정 ③**(2026-09-21): #92가 `api.ts` 에 더한 5행 · `App.tsx` 에 더한 2행을 판정해 **순 제거 5행**(타입·필드 JSDoc 3 — `doc_edit.rs` 의 `Sentences` 요약·`lines()` doc·0009 가 정본 · `App.tsx` 라우트 필드 2 — 0-based 자리 계약과 「서버가 들고 있는 제안」의 사본) · 유지 2행(`proposeEdit`·`decideEdit` 의 export 함수 JSDoc 요약 1줄) · 줄 수·지문은 88/`4346a888…` → **83/`ec54e678…`** — [passes/2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md) 「증분 재판정 ③」

### 원장 행 10 — `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일)

**증분 재판정 ④**(2026-09-21): #107 이 `api.ts` 에 더한 7행을 판정해 **순 제거 5행**(타입·필드 JSDoc 5 — 0010 `source`·`key` 문단 · `feature_add.rs` 의 같은 문장(함께 제거) · `AddFeature.tsx` 의 조건 렌더가 정본) · 유지 2행(`draftAddition`·`decideAddition` 의 export 함수 JSDoc 요약 1줄 — ③ 과 같은 모양) · 줄 수·지문은 83/`ec54e678…` → 90/`d88323c6…`(트리거) → **85/`89959a07…`** — [passes/2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md) 「증분 재판정 ④」

### 원장 행 10 — `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일)

**증분 재판정 ⑤**(2026-09-21): #112 가 `api.ts` 에 더한 2행(`previouslyDeleted` 필드 JSDoc — `feature_delete::previous_deletion` pub doc 이 정본 · `FeatureDeletion` 타입 JSDoc — 필드 `restoreUntil`·`restorable` 과 0011 머리)을 **전건 제거** · 줄 수·지문은 #112 이전 값 85/`89959a07…` 로 바이트 동일 복귀(export 함수 JSDoc 요약은 #112 가 더하지 않았다) — [passes/2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑤」 · **미판정 증분 없음**

### 원장 행 10 — `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일)

**증분 재판정 ⑥**(2026-09-22): #108 이 `api.ts` 2 · `RegisterLlmKey.tsx` 4 = 6행을 들여와 **제거 3 · 유지 3** — 제거는 `LlmLanguage` 타입 JSDoc(선언이 목록을 그대로 적는다) · `Analysis.llmLanguage` JSDoc(두 명제의 사본) · `LANGUAGES` 위 1행(순서·라벨을 리터럴이 말한다), 유지는 「저장은 키 등록 폼과 독립」 2행과 「`null` until the stored value arrives」 1행 — [passes/2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑥」 · 맥락 [passes/2026-09-22-output-language-axis.md](2026-09-22-output-language-axis.md)

### 원장 행 10 — `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일)

**증분 재판정 ⑦**(2026-09-22): #119 가 `RegisterLlmKey.tsx` 의 유지 1행을 `button`→`option` 으로 **고쳐 쓰기만** 해 **전건 유지 · 순 제거 0 · 88행 불변**, 지문만 `fd84aea8…` → `7e6915aa…` 로 이동 — [passes/2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑦」

### 원장 행 10 — `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일)

**증분 재판정 ⑨**(2026-09-24 · `rct_20260922-0009`): #132 가 `index.css` 에 더한 2행을 판정해 **순 제거 1행** — `.ev` 이관 블록은 네 겹으로 복원된다: 「목업 `.ev` 를 옮긴다」는 ②(목업)와 ③(PR #132 「근거 줄을 목업 `.ev`/`.ename`/`.esrc` 구조로 이관」) · 「목업은 경로를 하나만 그리지만 구현은 근거 경로를 전부 그린다」는 `frontend/src/CrossCuttingConcerns.tsx` 머리의 **유지된 정본**(「an item renders *every* path it cites where the mockup draws one」)의 두 벌째 · 「이름 칸은 최소 42%」는 바로 아래 `.ev .esrc { max-width: 58% }` 의 산술 여집합이라 리터럴 재진술(①) · 「`.ename`·`.esrc` 는 `.ev` 아래로 한정한다」는 같은 파일 위쪽 `.dep` 스코프 주석의 두 벌째이고 그 주석이 이유까지 적는다 · **유지 1행**은 `-webkit-text-size-adjust` 위의 iOS Safari 텍스트 자동 확대 — 상류 브라우저의 **문서화되지 않은 동작**으로, `docs/design-system.md` §5.3 은 선언 두 줄만 옮겨 적어 *이유* 를 복원하지 않고(② 는 반쪽), PR #132 본문은 한 스크린샷의 증상으로만 적는다 ⇒ 정책 본문 「애매하면 남긴다」의 비대칭 비용이 그대로 걸리는 자리라 **판단 갈림 1건**으로 남긴다 · 기재 88(트리거 `27d9b81` 에서 바이트 일치) → 유입 후 90 → 판정 후 **89 / `39baa2ff0e02877a92978a28d415bf7c128f75fc4dfd1fbf396948b781f8c5d6`** — [passes/2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑨」 · **미판정으로 남는 것**: 열린 PR #126 이 `api.ts`·`App.tsx` 에 들일 주석 — 착지 뒤 다음 감지의 몫이다.

### 원장 행 10 — `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일)

**증분 재판정 ⑪**(2026-09-25, 30차 패스): #152 가 `index.css` 에 연 `.disclosure` 머리 주석(지문 1행 · 물리 3행)을 **제자리 재작성해 물리 2행 제거** — 「긴 결과의 요약 → 상세 단계 노출(AC4.4)」은 PRD AC4.4 축자(②)이고 「Compact 에서는 제목 줄만 남기고 접히고 첫 확장 브레이크포인트부터 펼쳐진다」는 `docs/doc-tracker/2026-09.md` 편차 표 세 행(`AnalysisProgress`·`CrossCuttingConcerns`·`FeatureAcceptance`)의 축자다(②) · **유지 1행**은 「여는 주체는 화면이 넘기는 `open` — CSS 로 펼치면 DOM 상태와 보이는 상태가 갈리므로 표식만 그린다」로, 정책이 유지 대상으로 이름 붙인 「무엇을 넣지 말라」이고 이 파일에 `display` 를 더하는 것이 **조용히 깨지는 유일한 경로**다 · 지문 줄 수는 91 → **92**(줄머리 필터에 continuation 2행이 보이지 않는다 — 본문 「지문과 사각지대」) — [passes/2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑪」

### 원장 행 10 — `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일)

**증분 재판정 ⑭**(2026-09-26, 37차 패스, `rct_20260926-0002`): #166 이 `frontend/src/api.ts` 에 연 **5행**을 판정해 **순 제거 4행**(지문) · **물리 7행**(JSDoc 닫는 `*/` 는 지문 패턴에 안 걸린다) · 유지 1행 — 제거는 `Spend` JSDoc 의 「Connect Repository 화면이 보여 준」 경위·AC 꼬리표 2 · `costCents` 필드 JSDoc 1(「추정이지 청구서가 아니다」의 **정본은 `backend/src/usage.rs` 의 단가 상수 doc** — 단가를 고치는 사람이 읽는 자리다) · `Usage` JSDoc 1(① `{ total, analyses }` + ② 문서 이름 지목 `test/04 시나리오 9`) · 유지는 `Spend` 요약 1줄 — 프런트가 **어느 숫자를 그릴지 고르는 자리**이고 `AnalysisDetail` 안에 `spend` 와 `est*` 가 나란히 있어 바꿔 써도 타입이 같다 — [passes/2026-09-20-frontend-shell-axis.md](2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑭」

