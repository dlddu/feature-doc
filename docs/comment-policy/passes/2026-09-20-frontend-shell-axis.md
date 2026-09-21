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
