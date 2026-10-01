# 2026-09-29 — 필요성 개정 이전 · 첫 필요성 판정 슬라이스

- **reconciler task**: `tbm_feature-doc-comment-necessity/rct_20260929-0001`
- **기준**: [README.md](../README.md) 「필요성 시험」 — *이 주석을 지우면, 이 코드를 고치러 온 사람이 틀린
  판단을 하거나 그 판단에 필요한 사실을 확인하는 데 상당한 비용을 치르는가?* 사유를 한 문장으로 댈 수
  없으면 지운다.
- 이 패스는 두 몫이다. ⑴ 2026-09-29 필요성 개정의 이전(판정 없음) ⑵ 새 기준의 첫 판정 슬라이스 400줄.

## ⑴ 필요성 개정 이전

판정 기준이 「복원 가능한가」(복원 경로 넷, 애매하면 남긴다)에서 「한 문장 필요 사유를 댈 수 있는가」
(애매하면 지운다)로 바뀌었다. 이전은 주석을 한 줄도 판정하지 않는다.

1. `README.md` — 「복원 경로 넷」·「충돌 시 기본 방향」 절을 「필요성 시험」·「판정 규칙」으로 바꾸고,
   유지 대상의 「복원 불가능한 지식」을 「필요 사유가 있는 주석」(예시이지 면제가 아니다)으로, 제외 ①의
   이름을 「문서」로, 마이그레이션 절 1항의 「애매하면 유지」를 필요 사유 기준으로 고쳤다. 원장 칸의
   이름은 `판정 축` → `판정`이다.
2. `ledger.md` — 모든 행의 판정 칸을 `—`로 되돌렸다. 옛 기준의 `①②③④`는 새 기준의 판정 완료가
   아니다 — 복원 불가로 남긴 주석도 필요 사유를 다시 대야 한다. 각 행의 결과 요약은 옛 기준 판정의
   기록으로 그대로 둔다(아래 ⑵의 네 덩어리만 이 파일로 갈음).
3. `scripts/check-comment-ledger.py` — 판정 칸의 유효 표기를 `완료`·`—` 둘로 좁히고(I4), 판정 완료를
   `완료` 행으로 센다. 출력은 `[L] 판정 완료 N줄 / M행 (P% of T줄) · 미판정 행 K` 꼴이다.
4. 옛 모델 id `tbm_feature-doc-comment-redundancy` 참조 33곳(이 디렉터리의 `passes/` 31파일 ·
   `docs/doc-tracker/2026-09.md` 1곳)을 `tbm_feature-doc-comment-necessity`로 고쳤다 — reconciler 가
   그 모델의 task 기록을 새 id 아래로 옮겨 두어, 고친 참조가 그대로 해석된다. 허브의 정책 제목도
   「필요 사유 없는 주석 제거」로 고쳤다.

## ⑵ 판정 — 네 덩어리 400줄

| 덩어리 | 표면 | 판정 전 | 판정 뒤 | 제거 |
|---|---|---|---|---|
| `tools/check-data-format-change.py` (인계 행 — #218 이 들인 포인터 2행) | L · D | 3 · 0 | 3 · 0 | 0 |
| `tools/` 체커 `check-mockup-render.py` · `check-journey-mockup.py` · `check-scenario-e2e.py` | L · D · E | 162 · 41 · 3 | 152 · 38 · 2 | 14 |
| `scripts/check-comment-ledger.py` (원장 게이트) | L · D · E | 0 · 18 · 0 | 0 · 17 · 0 | 1 |
| e2e 하네스 `sc01-01` · `sc01-06` · `e2e/support/cluster.ts` · `e2e/smoke.sh` · `e2e/playwright.config.ts` | L | 173 | 150 | 23 |
| 합 | | **400** | **362** | **38** |

「제거」는 줄 수 순감이다. 그 밖에 틀린 주석 수정 4곳과 경위만 걷은 제자리 재작성 6곳이 있다(줄 수 불변).
`tools/check-data-format-change.py` 는 파일 무접촉이다(수동 승인 D6 대상 — 유지 판정이라 편집이 없다).

### 제거 목록

| 자리 | 유형 |
|---|---|
| `tools/check-scenario-e2e.py` 「보지 않는 것」 첫 항의 「모델 정의도 그것을 task 단계의 판단 사항으로 남긴다」 | 문서(모델 정의) 재진술 |
| `tools/check-journey-mockup.py` 모듈 docstring 제목 「사용자 여정 ↔ 목업 1:1 정합성 체커.」 | 코드(파일 이름) 재진술 |
| 같은 docstring R6 아래 「터치포인트 줄에서 파싱한 화면 집합(2026-09-03 폐지)」 | 틀린 주석 — 폐지된 규칙의 잔재가 R6 설명 아래 떨어져 남은 줄. 사유 없음 |
| 같은 파일 R7 앞 「화면 단위 목업(`sNN-*.html`)과 그 이관 대기 원장은 2026-09-03 …」 3행 | 경위 — 규칙 자체는 바로 아래 `fail("R7", …)` 문면이 말한다 |
| 같은 파일 R5 앞 「단계 섹션의 id 와 data-step 이 같은 값이어야 딥링크가 성립한다 (규칙 5e)」 | 문서 재진술 — `docs/mockups/README.md` 규칙 5e 와 바로 아래 실패 문면 |
| 같은 파일 줄 끝 「속성값(id/data-step)은 텍스트가 아니다」 (E) | 코드 재진술 — 변수 이름 `visible_text` |
| 같은 파일 139행대 「— 네거티브 컨트롤이 잡아낸 결함이다」 · 435행대 「(2026-08-27 문서 포털)」 | 경위(제자리 재작성 — 명제는 남김) |
| `tools/check-mockup-render.py` 「보지 않는 것」 M7·M8·M9 도입 경위 7행 → 3행 | 경위·실측 스냅샷(날짜·사고·프로브 값). 「카피 게이트의 사각지대」 명제만 남김 |
| 같은 절의 「(2026-09-25, AC4.1 접근 해제 통지가 …)」 · 「(2026-09-02, Analysis Progress 승격이 …)」 · 「2026-09-18 에」 | 경위(제자리 재작성) |
| 같은 절 「실행 스크린샷 픽셀 비교는 모델 정의상 범위 밖이다」 | 문서(모델 정의) 재진술 — 아무도 그렇게 가정하지 않는 부재 선언 |
| 같은 파일 `mockup_steps` docstring | 코드 재진술 — 함수 본문이 `placeholder`·`aria-label` 추출을 그대로 보인다 |
| `scripts/check-comment-ledger.py` `slash_lang` docstring | 중복 — `DeScanner` docstring 이 이미 「지문 스크립트 논리 무수정 이식」을 못박아 이 분기를 고치지 말라고 한다 |
| `e2e/support/cluster.ts` 파일 머리 「Cluster handles shared by …」 2행 + 빈 주석 1행 | 코드 재진술 |
| 같은 파일 `apiPods` JSDoc(비 export) | 코드 재진술 — 이름과 `kubectl get pods -l …` |
| 같은 파일 `setWorkerEnv` doc 「Used by sc01-06's LLM failure arc …」 + 빈 주석 행 | 호출처 재진술 — `git grep setWorkerEnv` |
| 같은 파일 `scaleWorkers` doc 「which is exactly how the first run of ac4-5 failed …」 · `setApiEnv` doc 「A rollout that nobody waited through is what broke …」 | 경위 — 함정 명제는 앞 문장이 말한다 |
| 같은 파일 `waitForApi` doc 「let sc04-02 … the eleven sc04-* specs …」 | 경위(제자리 재작성 — 실패 모양 `ECONNREFUSED :8080` 은 남김) |
| `sc01-01` 「시작 전에는 어떤 저장소도 실행 이력이 없다」 · 「시나리오는 브랜치 입력을 명시한다 …」 · 「아직 열리지 않은 5단계는 대기다 …」 | 코드 재진술 — 바로 아래 단정·입력 한 줄 |
| `sc01-06` 머리 「The second declaration closes blocker-ledger R1 …」 9행 | 문서 재진술(`docs/e2e-mocking-policy.md` 원장 R1·LLM-01 충실도) + 경위. 트리거 env 의 임대 창 규칙은 `cluster.ts` `setWorkerEnv` doc 과 이 파일 167행대 주석이 말한다 |

### 틀린 주석 — 수정

| 자리 | 무엇이 틀렸나 |
|---|---|
| `e2e/playwright.config.ts` `workers: 1` 주석 | `ac4-5`·`ac1-5` 는 `sc*` 로 개명돼 없는 파일 — 「Specs that lease the analysis worker」로 |
| `sc01-01` 머리 「모바일에서」 문단 | 「AC4.4 — 구현 대기」「폭 규칙이 착지하면」은 거짓 — AC4.4 는 `sc04-06` 이 검증한다 |
| `e2e/support/cluster.ts` `waitForApi` doc | 특정 사고(`sc04-02` 뒤 11개)를 일반 실패 모양으로 |
| `scripts/check-comment-ledger.py` docstring I4 | 새 표기(`완료`·`—`) — 이전 3항과 같은 편집 |

### 유지 목록 (묶음마다 필요 사유 한 문장)

**`tools/check-data-format-change.py`** (L 3)
- 머리 2행(「케이스 D1·D6 의 대상·이유와 … `docs/review-policy.md` 가 SSOT 다」) — 판정기의 라벨 `D1`·`D6` 은
  그 문서의 케이스 표에서만 정의되므로, 이 포인터가 없으면 `classify` 에 케이스를 더하거나 빼는 사람이
  정책 문서의 케이스·앵커 표를 함께 고쳐야 한다는 것을 모르고 둘을 어긋나게 만든다.
- `--no-renames` 인라인 1행 — 이 플래그를 불필요한 옵션으로 보고 지우면 `backend/migrations/` 밖으로 옮겨진
  마이그레이션이 새 경로로만 보여 D1 을 조용히 빠져나간다.

**`tools/check-scenario-e2e.py`** (L 25)
- 머리 제목 1행 — 규칙의 원천이 레포 밖 reconciler 모델 `tbm_feature-doc-scenario-e2e` 라는 것을 알려,
  규칙만 여기서 바꾸면 그 모델의 재감지가 되돌린다는 판단을 가능하게 한다.
- 「규칙」 S0~S5 범례(빈 줄·절 제목 포함) — 실패 출력이 규칙을 `S0`~`S5` 코드로만 가리키므로, 코드별
  적용 범위를 정의하는 유일한 자리다. 지우면 실패 한 줄을 읽기 위해 `main()` 전체를 역산해야 한다.
- 「보지 않는 것」 셋 — 초록을 「spec 이 시나리오를 제대로 검증한다」「AC 층까지 맞다」로 오독하는 판단을
  막는다(의미 일치·AC 층·지원 파일은 이 게이트 밖이다).

**`tools/check-journey-mockup.py`** (L 29 · D 27)
- 모듈 docstring 「규약은 … 「여정 페이지 규약」」 문단 — 규칙의 원문이 `docs/mockups/README.md` 에 있어
  체커만 고치면 규약과 어긋난다는 것을 알린다.
- 「SSOT 방향」 도식 — 실패가 났을 때 여정 문서가 아니라 반영물(목업·README·허브·추적)을 고치라는 수리
  방향을 정한다. 반대로 고치면 원천이 반영물에 끌려간다.
- 「규칙」 R0~R11 범례 — 실패 출력의 `R0`~`R11` 코드를 정의하는 유일한 자리다.
- 「화면 본문을 원본과 바이트 동일하게 고정하는 검사는 쓰지 않는다」 문단 — 구 R6 의 `data-sha256`
  을 「더 엄격한 개선」으로 다시 넣으면 화면에 실제 입력 요소·상태 변형을 넣을 길이 막힌다.
- `marked` docstring — 헤딩·산문으로 구간을 잡도록 바꾸면 산문 편집이 파서 입력을 깨뜨린다.
- `<body>` 분리 4행 — 한 정규식으로 합치면 같은 태그의 두 번째 `data-journey` 를 놓쳐 「정확히 1개」가
  공허해지고, 문서 주석 안의 `<body>` 언급이 경계를 속인다.
- R4 순서 대조 3행 — 멤버십 검사로 줄이면 버튼 대상을 바꿔도 늘 통과한다.
- R5 handoff 3행 — 선언 대상의 실재를 페이지 안에서 확인하도록 바꾸면 자기참조로 늘 통과한다.
- `<head>` 제외 1행 — title·meta 의 식별자를 누출로 잡는 오탐을 막는다.
- `<script>`/`<style>` 제외 2행 — 마크다운 렌더러의 치환 템플릿이 링크로 잡히는 오탐을 막는다.
- `.nojekyll` 4행 — `.md` 직접 링크가 Pages 에서 다운로드가 된다는 배포 함정을 모르고 규칙을 풀게 된다.
- 다른 HTML 앵커 검사 4행 — R5 와 중복으로 보고 지우면 허브 → 여정 단계 링크가 다시 무검사가 된다.
- R8 프래그먼트 2행 — R8 에 프래그먼트 검사를 더하면 R5 와 이중 판정이 된다.
- R11 6행 — 특정 문구만 막도록 좁히면 같은 주장을 바꿔 써 빠져나간다.

**`tools/check-mockup-render.py`** (L 98 · D 11 · E 2)
- 머리 제목 1행 — 원천 모델 `tbm_feature-doc-mockup-render` 포인터(`check-scenario-e2e.py` 와 같은 사유).
- 「규칙」 M0~M12 범례 — 실패 출력의 `M0`~`M12` 코드를 정의하는 유일한 자리다.
- 「보지 않는 것」 절 — 규칙 5 중 자동화된 다섯과 사람 몫(px·색), M7·M8·M9 가 있는 이유(카피 게이트의
  사각지대), M10 보류의 뜻, 조건 축 미계수, M3A 분모, M3B 의 영문 라벨·CSS 길이·`aria-label` 처리 —
  각 항목이 「초록이면 그 축도 맞다」는 오독과 「이 규칙은 M3 와 중복」이라는 삭제 판단을 막는다.
- `PROTO_CLASSES`·`PROTO_PROPS` 줄 끝 2행과 `.on` 연속 1행 — 목업 전용 장치를 대조에서 빼는 이유라,
  지우면 제외를 편차 은폐로 보고 되돌리게 된다.
- 순수 텍스트 본문 감싸기 3행 — 감싸지 않으면 자식 없는 `notice` 한 줄 안내를 「문면이 식이다」로 잘못 센다.
- 「현재 구현」 칸 탐색 2행 — 단계 단위 행의 대상 파일이 구현 칸에만 있다는 원장 관례를 모르면 탐색을 좁힌다.
- 앱바 부재 4행 — 온보딩 계열의 합의된 부재를 누락으로 오진하지 않게 목업 쪽을 먼저 푸는 순서를 지킨다.
- M10 순서 가드 2행 · M12 분모 가드 3행 — 순서를 뒤집거나 분모를 넓히고 좁히면 오진·은폐가 생긴다.
- docstring `table_after` · `ticked` · `jsx_text_nodes` · `css_declarations` · `open_tag` · `element_body` ·
  `same_state` — 각각 헤더 키워드 필터·셀 산문 파싱·TS 제네릭 오인·`@media` 순서 무력화·화살표 함수의 `>`·
  중첩 요소 절단·임의 임계 도입이라는, 코드만 보고는 「단순화」로 보이는 변경이 틀리는 이유다.

**`scripts/check-comment-ledger.py`** (D 17)
- 모듈 docstring 머리 2행 — 표면 하나를 게이트 밖으로 빼는 「단순화」가 그 표를 손으로 유지하는 값으로
  만든다는 것을 알린다.
- I1~I5 범례 — 실패 출력의 `I1`~`I5` 코드를 정의하는 유일한 자리다.
- 「측정 범위와 표면 추출은 모델 `asIs.versionScript` 와 같은 규칙」 3행 — 추출 규칙을 여기서만 고치면
  레포 밖 지문과 갈라진다.
- `DeScanner` docstring — 규칙을 고치지 말아야 하는 이유와 등가 증명 방법(편집 전 트리에서 D·E 덤프 대조).
- `rescue` docstring — 지시자 줄 뒤 사유를 여기서 건지지 않으면 어느 표면에도 잡히지 않는다.
- `parse_ledger` docstring — bare 이름 resolve 를 `.sql` 밖으로 넓히면 `Dockerfile` 같은 루트 파일이 없는
  경로가 된다.

**e2e 하네스** (L 150)
- `cluster.ts` 임대 규칙 문단(「The worker replica count is *deployment-wide* state …」) — 14개 spec 의
  Isolation 블록이 「see `e2e/support/cluster.ts`」로 가리키는 정본이며, 지우면 자기가 만들지 않은 job 을
  단정하는 spec 이 큐 드레인에 흔들린다.
- `workerPods`·`desiredWorkerReplicas`·`workerLogs`·`scaleWorkers`·`setWorkerEnv`·`setApiEnv`
  JSDoc 요약 — export 함수 JSDoc 요약 1줄은 정책(doc 주석 수준)이 요구한다. `workerLogs` 의 `--tail=10`
  기본값 언급은 `--tail=-1` 을 지우면 로그가 잘린다는 사유다.
- `scaleWorkers` 본문 — `kubectl scale`·`rollout status` 가 pod 정착 전에 돌아와 옛 pod 이 큐를 드레인한다는
  실패 모드의 함정.
- `setWorkerEnv`·`setApiEnv` 임대 순서 — env 를 scale 0 뒤에 세우고 scale down 뒤에 지우는 순서를 어기면
  임대 창 밖 pod 이 트리거를 들고 뜬다.
- `waitForApi` doc(비 export — 요약 줄은 본문의 주어) · `setApiEnv` 의 `Recreate` 문단 — 포트포워드가 pod 하나에 묶여 있어 `rollout status`
  를 기다리면 `ECONNREFUSED` 로 뒤 spec 전부가 쓰러진다는 함정과, 세 가지 대기의 이유.
- `apiGeneration` JSDoc(비 export) — generation 은 템플릿이 실제로 바뀔 때만 오른다는 사실이 no-op 판정의
  근거라, 지우면 아래 no-op 분기를 오해한다. no-op 3행도 같은 사유(없는 롤아웃을 기다리면 타임아웃).
- `playwright.config.ts` 전건 — `workers: 1` 은 임대 규칙의 전제이고 동시성은 샤드별 클러스터로만 얻는다는
  규칙, `actionTimeout` 30s 의 근거(관측 최장 21.2s · 상한 600s).
- `smoke.sh` 머리 1행 — 스크립트가 stub 로그인(`?as=smoke`)을 쓰므로 실 배포를 향해 돌리면 안 된다.
- `sc01-01`·`sc01-06` 의 Isolation 블록과 「Like every spec …」 문단 — 임대 규약(자기가 만든 job 만 단정)과
  자격증명 화면을 API 셋업으로 우회하는 이유(그 화면은 `sc04-01`/`sc04-03` 소유). spec 14개가 공유하는
  규약 블록이다.
- `sc01-01` 머리 「순서·제시·코드 근거 부착」 문단 — 세부 단정을 여기에 더하면 전용 spec 과 이중 계상된다.
- `sc01-01` 「모바일에서」 문단(수정본) — 모바일 폭 규칙을 이 spec 에 더하지 않는 이유.
- `sc01-06` 머리 「도달성은 제품 표면만으로 충분하다」 — 테스트 전용 훅을 더하는 판단을 막는다.
- `sc01-06` 의 `LATER_STAGES` JSDoc — `backend/src/pipeline.rs` 의 사본이라는 것을 알려, 단계가 바뀔 때 함께
  고칠 자리를 찾게 한다.
- 두 spec 의 인라인 — 오버레이 0 전제 명시, 라우팅이 상태 머신이라 자격증명 화면에서 시작, 큐잉 단정
  생략(임대 규약), 화면 자동 갱신 없음, 4단계가 승인 전 닫힘과 그 불변식의 소관(`backend/tests/worker.rs`),
  근거 유효성은 저장소 기준, 화면 진입이 전략을 실체화(lazy seed), 승인의 재큐잉과 워커 재청구, CTA 경로가
  「순서대로 제시」의 관측 수단, detail 대조의 이유, 격리 대조용 두 번째 분석, API 로 리셋을 보는 이유(2초
  렌더 경합), 두 신호를 함께 폴링하는 이유(`startedAt` 은 리셋이 지운다), 실패 needle 이 4단계 프롬프트에만
  있어 원문이 지목한 단계에 착지, 트리거 env 를 pod 없는 때 세우는 순서, 「유지」 반쪽을 실패 전에 고정,
  API 로 전략을 여는 것이 setup 이고 화면 검증은 `sc01-04` 소유 — 각각 지우면 그 자리를 「단순화」해 경합·
  이중 계상·거짓 초록을 만드는 결정의 이유다.
