# 34차 패스 — #158(AC4.1 「해제」 조항)이 연 124행

- 판정일: 2026-09-25
- task: `rct_20260925-0012` / 모델 `tbm_feature-doc-comment-redundancy`
- 창: `b632577`(33차 패스, #160) → `6e443c7`(#158, 슬라이스 7c)
- 유입: **gross 124행 · 제거 0 · 제자리 수정 0**(순증 = gross). 창의 첫 커밋 `b632577` 은 33차
  패스 자신이라 노이즈이고, 판정 대상은 `6e443c7` 이 세운 것뿐이다.
- 산출: **새 행 32**(신설 파일 `sc04-02` 19행) + **행 1·3·7·10·12·14 의 증분 재판정**(105행)

## 판정 규칙 — 33차 패스의 규칙을 그대로 승계한다

#158 본문은 설계 판단을 절 단위로 축자에 가깝게 적어 두었으므로, ③ 를 규칙 없이 적용하면
유지가 0 에 수렴한다. 33차 패스가 세운 갈림을 그대로 쓴다.

- **닫는다**: 서사(「왜 이렇게 만들었나」) · 시나리오·AC 본문의 재진술 · 절 제목 · 사례 서술
- **남긴다**: 편집 지점 가드 — 「이 값이 무엇과 같아야 하는가」 · 「무엇을 넣지 말라」 ·
  「이 순서를 바꾸면 무엇이 조용히 깨지는가」

## 🔴 전제 하나를 실측으로 뒤집었다 — 「AC 꼬리표 전건 제거」는 레포 전역 규칙이 아니다

이 패스의 감지는 「`passes/2026-09-20-github-app-auth-axis.md` 가 AC 꼬리표를 **전건 제거**로
닫았는데 #158 이 그 유형을 6파일에 되살렸다(11행)」를 1순위 근거로 넘겼다. **그 판정은 그 패스의
범위 안에서만 참이다.**

| 재는 법 | 값 |
|---|---|
| `AC[0-9]+\.[0-9]+` 를 담은 live 주석 줄 — 부모 `b632577` | **84행** |
| 같은 값 — tip `6e443c7` | 95행 (= 84 + #158 의 11) |
| 부모 트리의 미판정 잔여 (33차 패스가 닫은 값) | **0** (행 열의 합 2,721 = 전역 live 2,721) |

부모 트리는 잔여가 0 이므로 그 **84행은 전부 판정을 받고 유지된 줄**이다. 즉 이 레포의 살아 있는
선례는 「AC 꼬리표가 들어간 줄은 전부 제거」가 아니라 **「그 줄이 꼬리표 말고 아무것도 말하지 않을
때만 제거」** 다. 2026-09-20 의 그 항목이 든 네 꼬리표(`(AC4.1)`·`(AC4.3)`·`(AC4.7)`·`(AC4.8·AC4.3)`)는
전부 그런 줄이었다.

그래서 이 패스는 **유지로 판정한 줄에서 꼬리표만 떼어내지 않는다.** 떼어내면 살아 있는 84행과
어긋나는 새 규칙을 세우는 것이지, 있는 규칙을 적용하는 것이 아니다.

**같은 이유로, 감지가 권한 「`AC[0-9]+\.[0-9]+` 정규식 CI 게이트」는 이 트리에서 84건의 거짓
양성을 낸다** — 지금 신설하면 켜지는 순간 빨갛고, 초록으로 만들려면 판정을 받고 유지된 84행을
지워야 한다. 게이트가 필요하다면 **절대 규칙이 아니라 래칫**(이 축에서 꼬리표 줄이 늘지 않는다)
이어야 하고, 그것은 이 패스의 범위 밖이다.

## 새 행 32 — `e2e/tests/sc04-02-app-uninstall-during-analysis.spec.ts` (19행)

신설 파일이라 판정 행이 없었다. **제거 12행 · 유지 7행.**

### 제거 — 파일 머리의 시나리오 재진술 6행

| 주석 | 복원 경로 |
|---|---|
| 「이 spec 이 재는 것은 … 진행 중이던 분석이 현재 호출까지만 마무리하고 멈추고, 그 사유가 사용자에게 통지되며, 되지 않을 복구 경로(단계 재시도)는 제시되지 않는다」 3행 | ② `docs/doc-tracker/2026-09.md` 121행 매핑 칸이 「분석이 채택 정책(**현재 호출까지만 마무리하고 중단**)대로 `failed` 로 멈추고 … 복구 경로가 아닌 「이 단계만 다시 시도」·단계 실패 안내는 서지 않는다」로 **축자 소유** · 1행의 기계 판독 선언 `// 검증 시나리오: 04-platform.md#시나리오 2` 가 이미 그 문서를 가리킨다 |
| 「해제는 GitHub 쪽 사건이라 제품 안에 그것을 일으키는 버튼이 없다. 그래서 App 더블의 허용 저장소 목록을 좁혀(`FEATUREDOC_STUB_REPO_ACCESS`) …」 3행 | ② 같은 매핑 칸의 「App 더블의 허용 저장소에서 그 저장소를 빼 **범위 축소**를 만든다(GitHub 쪽 사건이라 제품 안에 버튼이 없다)」 — 괄호까지 축자 |

`sc04-12` 머리의 시나리오 인용 3줄을 같은 논거(「원문의 축자 인용이고 1행 선언이 이미 그 문서를
가리킨다」)로 걷은 **2026-09-20 선례를 그대로 적용**했다. 다만 「재는 것은 "해제를 알아챘는가"가
아니라 채택된 정책이 그대로 일어났는가다」 **1행은 남겼다** — 이 spec 이 **무엇을 단정하지 않는가**를
말하는 줄이고, `sc01-03` 머리의 「기대값을 상수로 박지 않는다」가 같은 형태로 유지된 선례가 있다.

### 제거 — 본문의 단계 표찰 6행

`// 해제 전: 진행 중이고, 통지도 없다.` · `// 사용자가 GitHub 에서 이 저장소를 App 의 접근 범위에서
뺀다.` · `// 진행 상황 확인 — 정책대로 멈춰 있다.` · `// 문면은 서버가 소유한다 — 화면은 그것을
그대로 띄운다.` · `// 재시도는 복구 경로가 아니다 …` · `// 범위를 되돌리면 다시 시작할 수 있다 …`

앞의 셋과 마지막은 **바로 아래 한 줄이 그 문장 자체**다(①: `.toBe('queued')` · `setApiEnv(ACCESS,
'checkout-web,notif-worker')` · `.toBe('failed')` · `setApiEnv(ACCESS, ALL_REPOS)` + `.toBe(201)`).
네 번째는 #158 본문 「사유 문면을 서버(`analysis::ACCESS_REVOKED`)가 소유하고 화면은 그것을 그대로
띄운다」의 축자(③)이며, 다섯 번째는 doc-tracker 121행(②)이다.

**전수 대조**: 다른 spec 14개에 살아남은 본문 주석 141행을 훑으면 유지형은 전부 가드
(「the number is the worker's measurement, not a fixture in this file」 류)이고 단계 표찰은 없다 —
이 축만 예외가 되는 것이 아니다.

### 유지 7행

- **`// Leases the analysis worker *and* one API env var — lease rules in `e2e/support/cluster.ts`.` 2행**
  — spec 10개가 공유하는 **Isolation 규약**이다(`grep -l Leases e2e/tests/*.spec.ts` = 10). 두 개를
  빌리므로 2행일 뿐 형태가 다르지 않다. 한 축에서 혼자 지우면 첫 이탈이 된다.
- **`/** 더블이 그리는 설치의 전체 범위(`backend/src/github_app.rs`의 stub 세 저장소). */` 1행** —
  「이 값이 무엇과 같아야 하는가」. `ALL_REPOS` 의 세 이름이 `github_app` 의 stub 목록과 갈리면
  `finally` 의 반납이 조용히 다른 상태를 남긴다.
- **`// 큐를 비운 채로 분석을 세워 둔다 — "진행 중"이 워커 타이밍에 흔들리지 않게.` 1행** —
  `scaleWorkers(0)` 이 왜 먼저 오는가. 빼면 테스트가 편집 한 번에 조용히 flaky 가 된다.
- 파일 머리 잔여 3행(구분 `//` 2 + 위의 「재는 것은 …」 1).

기계 판독 `// 검증 시나리오:` 1건은 DIRECTIVE 라 지문·판정 양쪽에서 제외된다 — 건드리지 않았다.

## 증분 재판정 — 판정 완료 행 안에 #158 이 연 105행

| 행 | 파일 | 유입 | 제거 | 유지 | 줄 수 | 서수 |
|---|---|---|---|---|---|---|
| 1 | `analysis.rs` 22 · `worker_api.rs` 18 | 40 | **30** | 10 | 613 → **623** | ⑬ |
| 3 | `e2e/support/cluster.ts` | 29 | **0** | 29 | 145 → **174** | ⑤ |
| 7 | `frontend/src/AnalysisProgress.tsx` | 3 | **1** | 2 | 149 → **151** | ⑥ |
| 10 | `frontend/src/api.ts` | 3 | **0** | 3 | 93 → **96** | ⑬ |
| 12 | `backend/src/github_app.rs` | 9 | **3** | 6 | 78 → **84** | ⑬ |
| 14 | `backend/tests/worker.rs` 12 · `scripts/e2e.sh` 9 | 21 | **7** | 14 | 21 → **35** | ⑩ |

### 행 3 `cluster.ts` 29행 — **전건 유지**. 새 블록마다 판정을 받고 유지된 쌍둥이가 같은 파일에 있다

이 파일은 워커용 헬퍼와 API 용 헬퍼를 나란히 둔다. #158 이 더한 것은 **API 쪽 네 블록**인데,
그 각각에 **부모 트리에서 이미 판정을 받고 유지된 워커 쪽 대응물**이 있다.

| #158 이 더한 것 | 부모에 살아 있는 대응물 | 같은 점 |
|---|---|---|
| `/** Pod names … for the API Deployment, in any phase. */` | `/** Pod names … for the worker Deployment, in any phase. */` | `worker`↔`API` 말고 **바이트 동일** |
| `/** Bumped by the API server whenever an edit actually rewrites the pod template. */` | `/** The Deployment's currently desired replica count, as a string. */` | 한 줄 요약 + 상류(k8s) 동작 |
| `waitForApi` doc — 「deliberately *not* `rollout status`」 + 그렇게 해서 깨졌던 실사례 | `scaleWorkers` doc — 「`kubectl scale` + `rollout status` is not enough on the way down … which is exactly how the first run of ac4-5 failed」 | **가드 + 실사례**가 한 덩어리인 형태까지 같다 |
| `setApiEnv` doc — 요약 / 리스 규약 / 쓰는 spec | `setWorkerEnv` doc — 요약 / 리스 규약 / 「Used by sc01-06's LLM failure arc …」 | 절의 구성이 같다 |

즉 이 29행에서 걷을 것을 고르려면 **부모의 유지 판정을 같이 뒤집어야 한다.** 그 논거는 이 창에
없으므로 전건 유지한다(공유 사본 불변식 — 한 벌만 고치면 그 자체가 첫 이탈이다).

`// A no-op edit (clearing a var that was never set) rewrites nothing …` 3행도 유지 — `apiGeneration()`
비교가 **왜 있는가**이고, 지우면 세울 이유가 없는 코드가 된다.

### 행 1 `analysis.rs` 22행 → 제거 14 · 유지 8

- `still_granted` doc **10행 → 2행**. 요약 「Whether the App still grants this user access to this
  repository」는 **함수 이름 + 시그니처가 그 문장 자체**다(①). 「설치 없음과 범위 축소가 한 호출로
  답해진다」·「아무것도 우리에게 push 하지 않는다 — 토큰을 발급해 쓰는 것과 같은 방식으로 필요한
  순간에 GitHub 에 묻는다」는 #158 본문의 「`accessible_repos` 가 「설치 없음(해제)」과 「목록에 없음
  (범위 축소)」을 한 호출로 답한다」·「웹훅 수신면을 짓지 않았다」의 축자(③)이고, 앞부분은
  `accessible_repos` 본문 3줄이 그대로 보인다(①).
  **남긴 2행은 「An upstream failure is *not* revocation: it propagates」** — 이 줄이 없으면 다음
  편집자가 `Err` 를 `false` 로 접어 **GitHub 장애를 사용자의 접근 해제로 오독하는 코드**를 만들 수
  있다. 전형적인 「이 순서를 바꾸면 무엇이 조용히 깨지는가」다.
- `ACCESS_REVOKED` doc **7행 → 4행**. 「사용자가 읽는 통지라서 화면에 다시 적지 않고 원인 옆에 둔다」
  3행은 #158 본문 축자(③)이고, 화면 쪽 정본이 `frontend/src/api.ts` 에 따로 있다(아래 행 10).
  **남긴 것은 요약 1행과 「It names no installation id, repository or token」** — **부재를 말하는
  단정**이라 복원 경로 넷 어디에도 없고, 이 상수를 고칠 사람에게 거는 유일한 제약이다.
- `access_revoked` 필드 doc **5행 → 2행**. 「화면이 문장을 대조하지 않게 여기서 판정한다」·「재시도는
  복구 경로가 아니다」는 각각 `api.ts` 와 `AnalysisProgress.tsx` 에 **같은 명제의 두 벌째**이고
  (아래 「정본 지정」), doc-tracker 121행이 ② 로 소유한다. **남긴 2행은 「두 번째 열이 아니라 저장된
  사유에서 파생한다 — 그래야 둘이 갈릴 수 없다」** — 「무엇을 넣지 말라」(새 열).

### 행 1 `worker_api.rs` 18행 → 제거 16 · 유지 2

- doc **11행 → 2행**. 「AC4.1 의 정책: 진행 중인 호출을 마치고 멈춘다」는 doc-tracker 121행의
  「채택 정책(**현재 호출까지만 마무리하고 중단**)」 축자(②)다. 「리스 경계라 요청이 끊기지 않고 이미
  지불한 LLM 호출을 버리지 않는다」·「워커는 이미 돌려주던 `409` 로 알게 되므로 큐 프로토콜이 바뀌지
  않는다」·「실행 중이던 단계도 같은 사유로 닫아 화면의 이야기가 하나다」는 #158 본문의 같은 이름 절들이
  절 단위로 소유한다(③).
  **남긴 2행은 「The policy is honoured by *where this is called from*, not by anything here: both call
  sites sit on a lease boundary」** — 호출부를 리스 경계 밖으로 옮기면 정책이 **조용히** 깨진다.
- 인라인 블록 **4행·3행 전건 제거**. 두 블록이 말하는 명제(「해제된 job 이 큐를 막지 않는다」·「갱신을
  거절하는 것이지 진행 중인 호출을 선점하는 것이 아니다」)는 **`backend/tests/worker.rs` 의 두 테스트
  이름이 그 문장 자체**다 — 「a queued job whose access is gone is closed by the policy instead of being
  handed out, and the queue does not wedge behind it」 · 「access taken away *while the job runs*. The
  renewal is refused at the lease boundary」. 그 doc 2벌을 **정본으로 지정해 유지**했으므로(행 14)
  명제가 트리에서 사라지지 않는다.

### 행 14 `backend/tests/worker.rs` 12행 → 제거 4 · 유지 8

- 두 테스트의 `///` **6행은 정본으로 유지**(바로 위 근거).
- 헬퍼 doc **4행 → 2행**: 「설치 행이 `accessible_repos` 가 먼저 읽는 것이라 지우면 아무것도 남지
  않는다」는 코드 두 줄이 보이고(①), 「범위 축소 모양은 `sc04-02` 가 end-to-end 로 몬다」는 doc-tracker
  121행이 「「설치 자체가 사라진」 모양은 `backend/tests/worker.rs` 의 두 단정이 지킨다」로 소유한다(②).
  **남긴 것은 「프로세스 전역 stub 스위치를 건드리지 않는다 — 같은 바이너리의 형제 테스트가 그것을
  본다」** — 격리 함정이고 정의가 유지 대상으로 이름 붙인 형태다.
- `// Closed, not merely skipped: a second claim finds nothing left to reconsider.` 1행 — 바로 아래
  단정이 그 문장이다(①). `// The screen reads the flag, not the sentence.` 1행 — `api.ts` 정본의
  두 벌째.

### 행 14 `scripts/e2e.sh` 9행 → 제거 3 · 유지 6 — **정본 지정**

포워드가 죽는 사정을 `e2e.sh` 와 `cluster.ts` 가 **두 벌** 적고 있었다. 12차 패스의 「정본을 어디에
둘 것인가」로 가른다.

| 명제 | 정본 | 근거 |
|---|---|---|
| `port-forward` 는 pod 하나에 묶이고 그 pod 가 가면 죽는다 → 감시하며 재기동한다 | **`scripts/e2e.sh`** | 포워드를 띄우고 되살리는 코드가 여기 있다 |
| 그래서 `setApiEnv` 는 `await` 로 그 창을 덮는다 | **`e2e/support/cluster.ts`** | `await` 를 강제하는 코드가 여기 있다 |
| API 가 `strategy: Recreate` 인 이유 | **`deploy/k8s/deployment.yaml`** | 「SQLite on a ReadWriteOnce volume: never let two pods mount it at once」가 이미 있다(①) |

`e2e.sh` 에서 뗀 3행은 뒤의 두 칸(`(SQLite on a ReadWriteOnce volume)` 인용과 「the counterpart is
`setApiEnv` in e2e/support/cluster.ts, which does not return until this has landed」)이다. 첫 칸은
그대로 남겼다. `# kind e2e only ever runs in CI, so this log is the only surviving record …` 2행도
유지 — 이 `printf` 를 지울 사람에게 거는 제약이다.

### 행 12 `github_app.rs` 9행 → 제거 3 · 유지 6

`stub_granted_names` doc **7행 → 4행**: 요약 2행을 1행으로 줄이고(「so a test can take repository
access away the way a user does on GitHub」은 호출부 `sc04-02` 가 보인다 — ①), 「값은 쉼표로 이은
저장소 이름 목록이고 빈 값은 아무것도 주지 않는다」 3행을 걷었다(파싱 코드가 그 문장이다 — ①).
**남긴 것은 「Unset is the full stub installation, so a deployment that says nothing keeps the three
repositories every other spec relies on」** — 기본값 불변식이고, #158 본문의 음성 프로브가 바로 이
줄이 지키는 것을 실측했다(「`FEATUREDOC_STUB_REPO_ACCESS` 미설정 → `sc04-01` 의 `3 repositories`
단정 불변」). 「Both read [`stub_granted_names`] … one narrowing, two answers that cannot drift apart」
2행도 유지 — 두 답이 갈리지 않아야 한다는 불변식이다.

### 행 7 `AnalysisProgress.tsx` 3행 → 제거 1 · 유지 2

「AC4.1: the job was stopped because the App no longer grants access」와 「the server's reason is what
the user reads」는 doc-tracker 121행(②)과 `api.ts` 정본이 소유한다. **남긴 2행은 「Re-running a stage
cannot recover a revoked access, so the stage-level retry affordances stand down」** — 재시도 버튼을
되살리면 사용자가 **거짓 안내**를 받는다는, 이 JSX 분기가 존재하는 유일한 이유다.

### 행 10 `frontend/src/api.ts` 3행 → **전건 유지**

`accessRevoked` 필드의 JSDoc 3행은 「서버가 저장한 사유에서 판정하므로 화면은 그 문장 자체를 대조하지
않는다」다. **이 명제의 정본을 여기로 지정했다** — 문장을 대조하는 코드를 쓸 사람이 읽는 자리가
화면 쪽이기 때문이다. 그래서 `analysis.rs`·`worker.rs` 의 두 벌째를 걷고 이 3행을 남겼다.
줄 수·지문은 #158 의 +3 이 들어온 값으로 **재측정만** 한다(판정은 「전건 유지」).

## 값

```
전역 live   2,845 → 2,792   (순 제거 53행)
행 열의 합  2,721 → 2,792   (+71 = 유입 124 − 제거 53)
⑴ 행 열 밖 잔여 0 · ⑵ 행 열 안 미판정 증분 0
⇒ 행 열의 합 2,792 + 0 + 0 = 전역 live 2,792
```

## 검산

- **비주석 diff 0줄** — `git diff -U0` 에서 `+`/`-` 줄 중 주석 시작 패턴·빈 줄이 아닌 것이 없다.
- 행 지문 6개를 **부모 `b632577` 에서 먼저 재현**해 원장 기재값과 바이트 동일함을 확인한 뒤
  (613/`666abdf8…` · 145/`97a280f4…` · 149/`bed7da48…` · 93/`407308c1…` · 78/`97ae560b…` ·
  21/`d4414d5f…`) 이 트리에서 다시 계산했다 — 재현이 안 되는 행을 고치는 사고를 막는다.
- 파일 9개의 live 주석 수를 부모·tip·이 트리에서 각각 세어 **유입 124 = 제거 53 + 유지 71** 을
  확인했다.
