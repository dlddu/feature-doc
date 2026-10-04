# 2026-10-04 — 필요성 판정 (재분석 diff · 프런트 데이터·셸 · 프로토타입 하네스·설정 · 빌드 설정)

- **reconciler task**: `tbm_feature-doc-comment-necessity/rct_20261004-0001`
- **기준**: [README.md](../README.md) 「필요성 시험」 — *이 주석을 지우면, 이 코드를 고치러 온 사람이 틀린
  판단을 하거나 그 판단에 필요한 사실을 확인하는 데 상당한 비용을 치르는가?* 사유를 한 문장으로 댈 수
  없으면 지운다.
- **범위 선택**: 판정 칸이 `—` 인 행은 L 16 · E 1 이었다. 마이그레이션 `.sql` 5행은 전용 PR · 사람 repair 몫이고
  (열린 PR #223), `backend/src/db.rs` 행은 같은 PR 의 `backend/tests/migrations.rs` 와 파일이 겹쳐 뺐다. 남은 L 10행
  · E 1행 중 이번 슬라이스는 덩어리 넷을 예산 400줄 정확히 가져간다 — 재분석 diff 축(151) · 프로토타입
  하네스·설정(141) · 프런트 데이터·셸 축(99) · 빌드·패키징 설정(9). 나머지(backend 집중 4파일 628 + 같은 파일의
  E 1행 · 워커 190 · GitHub App 84 · llmkey tests 42 · worker tests 36 · `push.rs` 1)는 다음 슬라이스 몫이다 —
  628줄 덩어리는 단독으로 예산을 넘으므로 그 덩어리만 한 슬라이스가 가져간다.
- **지문 사각지대**: `tools/check-journey-prototype.js` 의 블록 주석 중 `*` 로 시작하지 않는 연속 줄은 지문에 보이지
  않지만(README 「지문과 사각지대」), 판정은 블록 전체를 대상으로 했다 — 지운 INPUT_PROBE 경위 블록의 연속 두 줄이
  그 예다.

## 판정 — 4행 400줄

| 덩어리 | 표면 | 판정 전 | 판정 뒤 | 제거 |
|---|---|---|---|---|
| 빌드·패키징 설정 3파일(`Dockerfile` · `backend/Cargo.toml` · `deploy/k8s/.gitignore`) | L | 9 | 9 | 0 |
| 재분석 diff 축 7파일(`backend/src/diff.rs` …) | L | 151 | 147 | 4 |
| 프런트 데이터·셸 축 8파일(`frontend/src/api.ts` …) | L | 99 | 99 | 0 |
| 프로토타입 하네스·설정 2파일(`tools/check-journey-prototype.js` · `backend/src/config.rs`) | L | 141 | 102 | 39 |
| 합 | | **400** | **357** | **43** |

비주석 diff 는 0줄이다. `tools/check-journey-prototype.js` 의 출력은 편집 전후 바이트 동일하다(「여정 프로토타입 5개 ·
단언 425건 실행 — 통과」).

### 제거 목록

| 자리 | 유형 |
|---|---|
| `backend/tests/diff.rs` `trace` doc 「`tree` 는 그 시점의 저장소 트리다 …」 | 코드 재진술 — 매개변수 이름과 두 호출부가 넘기는 서로 다른 트리 |
| `frontend/src/AnalysisDiff.tsx` `onBack` · `onOpenFeature` prop JSDoc 둘 | 코드 재진술 — 콜백이 어디로 가는지는 `App.tsx` 의 배선이 말한다 |
| `frontend/src/AnalysisProgress.tsx` `ACTIVE` JSDoc | 코드 재진술 — 집합 이름과 폴링 조건에서의 쓰임 |
| `tools/check-journey-prototype.js` ARM 의 단계별 한 줄 6개(「후보 N건을 전부 결정해야 …」 · 「근거를 한 건도 열지 않으면 …」 등) | 문서 재진술 — 단계의 전진 조건은 목업과 여정 문서가 정하고, 최소 입력이 모자라면 하네스 자신이 P2 로 실패한다 |
| 같은 파일 INPUT_PROBE 앞 경위 블록(「⚠️ 이 등록부가 없던 시절 …」 3줄) | 작업 흔적 — 등록부가 있는 이유는 파일 머리 「등록부 4종」 문단이 이미 말한다. 머리의 「(실제로 그랬다. 아래 INPUT_PROBE 주석 참조)」도 같은 경위의 지시라 함께 걷었다(재작성 1) |
| 같은 파일 INPUT_PROBE 의 번호 붙은 프로브 제목 17줄(「① 거부 사유(textarea) — …(F7 재발 방지)」 등) | 구분선·코드 재진술 — 바로 아래 `ok(…)` 단언 문구가 같은 명제를 말하고, `F7`·`F8` 은 이력 꼬리표다. `③ 재분석 회차` 의 둘째 줄(두 회차 수치가 겹치지 않는 이유)은 남기고 홀로 서게 고쳤다(재작성 1) |
| 같은 파일 PRODUCT_PATHS 의 기대 동작 한 줄 15개(「자동 채택하지 않는다 …」 · 「이어하기 — …」 등) | 문서 재진술 — 여정 문서 §4 분기표의 기대 결과이고, 바로 아래 변수 이름(`notAdopted` · `kept` · `noFabrication` …)이 같은 것을 말한다 |

### 틀린 주석 — 고침

| 자리 | 무엇이 틀렸나 | 고친 문장 |
|---|---|---|
| `frontend/src/HomeRepositories.tsx` `awaiting_pipeline` 배지 | 「LLM 단계가 아직 구현되지 않았다」— LLM 단계는 구현돼 있고, 지금 이 상태는 사람 게이트 뒤 단계가 기다린다는 뜻이다(`backend/src/pipeline.rs` `AWAITING_PIPELINE` doc · `bin/worker.rs` finish 주석) | 「`Synced` 가 아닌 이유: 사람 게이트 뒤 단계(전략 승인의 4단계 · 확정된 기능의 5단계)가 아직 돌지 않았다」 |
| `backend/src/repo_scan.rs` 트리 잘림 검사 | 꼬리 「paging arrives with the real pipeline」은 이미 지난 미래 약속(실 파이프라인은 들어왔고 페이징은 없다) | 꼬리를 걷고 「잘린 트리는 조용히 적게 센다 — 믿을 수 없는 수를 내지 말고 그렇다고 말한다」만 남겼다 |

### 유지 목록 (묶음마다 필요 사유 한 문장)

**빌드·패키징 설정** (L 9)
- `Dockerfile` 스텁 빌드의 두 번째 placeholder — `[[bin]]` 둘이 다 있어야 의존성 캐시 빌드가 풀린다는 것을 모르면 하나를 「군더더기」로 지워 캐시 단계가 깨진다.
- `Dockerfile` 마이그레이션 복사 — `sqlx::migrate!` 가 컴파일 시점에 임베드하므로 빌드 단계엔 있어야 하고 런타임 이미지엔 필요 없다는 것을 모르면 어느 한쪽으로 「정리」한다.
- `Dockerfile` 「두 워크로드는 한 이미지 — 따로 버전을 매기면 안 된다」 — 금지가 적힌 유일한 자리라(README 는 사실만 적는다) 지우면 이미지를 쪼개는 판단을 막을 것이 없다.
- `backend/Cargo.toml` 「LTO 와 CGU 는 둘 다 바꿔야 줄어든다」 · 「줄 번호만 남겨도 backtrace 의 파일:줄은 보인다」 — 하나만 바꾸는 최적화와, 디버그 정보를 되살리는 판단을 각각 막는다.

**재분석 diff 축** (L 147)
- `backend/src/diff.rs` 모듈 머리 · 정체성 축 `(category, name)` 이 0008 `UNIQUE` 와 같아야 하는 이유 — 둘이 어긋나면 저장에서 한 행인 것이 diff 에서 두 항목이 된다.
- `pub` 구조체·필드·함수 요약(`Scenario` · `identity` · `text` · `location` · `scenario_of` · `feature_keys` · `feature_name` …) — doc 주석 수준(정책). `scenarios` 필드의 「이번 시나리오만 센다」 · `scenarios_of` 의 「누락은 오류가 아니라 버린다」 · `feature_diff` 의 「`None` 은 묻지 않았다는 뜻」은 각각 셈의 기준, 검증 안 된 문서의 처리, 의존성이 통째로 사라진 것처럼 보이는 오독을 막는다.
- `norm` doc — `acceptance::situation` 과 같은 규칙이라는 결합을 모르면 한쪽만 바꿔 공백 차이를 변경으로 표시한다.
- `backend/tests/diff.rs` 머리(단위 테스트와의 경계 · 인수 문서를 손으로 만드는 이유) · `SIFTED`(stub 트리에 실재하는 경로) · lazy seed GET — 각각 테스트 중복, 픽스처가 공허해지는 것, `load_strategy` 404 를 막는다.
- `frontend/src/AnalysisDiff.tsx` 목업 매핑(게이트가 읽는다) · 「서버가 보낸 것만 그린다」 · `markOf` 의 U+2212 · 열어 본 표식의 수명 · 「견줄 상대 없음 ≠ 바뀐 것 없음」 · 배너가 세는 단위 · 목업 카피 없는 세 상태 · appbar 세 칸(M7) — 각각 클라이언트 필터링, 보이지 않는 글리프 차이, 게이트 조건, 오도하는 첫 분석 문구, 셈 단위, 카피 출처, 게이트가 세는 칸을 지키는 이유다.
- `e2e/tests/sc02-08-…` Isolation 블록(e2e spec 공유 규약) · `REVISION` 이 stub 더블 안의 스위치라는 것 · 「무엇이 늘었는지는 두 추적 결과가 정한다」 — 기대값을 하드코딩하는 판단을 막는다.
- `backend/src/repo_scan.rs` — 자유 함수인 이유(워커엔 `AppState` 가 없다) · `paths` 를 들고 가는 이유 · stub 브랜치가 유일한 ref · 비율 휴리스틱 일치와 404 충실도 경계 · 리비전은 경로를 더하기만 · 리비전 분리의 테스트 경합 이유 · 1단계 수치 증가 · env 트리거의 값 규칙 · 경로 목록 상한 · `truncated` 를 모델에 보이는 이유 · 응답·토큰을 메시지에 넣지 않는 불변식 · 트리 잘림 오류 · 리비전 off 바이트 동일 테스트 — stub 이 real 과 갈리는 지점과 자격증명 비노출을 포함해 지우면 각각 틀린 단순화로 이어진다.
- `frontend/src/AnalysisProgress.tsx` 목업 매핑 · 「진행을 들고 있지 않다」 · 백그라운드에서도 작업이 계속된다는 `onBack` 계약 · tick 상태와 `useRef`(타이머 재시작 함정) · 실패 단계는 하나 · 회수된 접근에 재시도 숨김 · `.card` 감싸기 · 성공 단계만 링크(404 회피) · 5단계 전 견줄 표현 없음 — 상태 소유와 화면 조건의 이유다.
- `backend/src/lib.rs` 모듈 머리 · `build_router` 요약 · `init_tracing` 의 ANSI 판단 — 파이프 출력에 이스케이프가 섞여 `kubectl logs` 가 읽히지 않는 실패 모드를 말한다.

**프런트 데이터·셸 축** (L 99)
- `frontend/src/api.ts` — 같은 origin 전제(`credentials: 'same-origin'`) · OAuth 는 fetch 가 아니라 전체 이동 · 비정규화 필드 · 실측 vs 추정 · `accessRevoked` 를 문장으로 맞추지 않는 이유 · `hasAccess: false` 는 답 · 빈 목록은 오류 아님 · 재시도 응답이 초기화된 진행을 싣는다 · 범위 밖 대상은 큐에 안 든다 · 404/`null` 의 「아직 없음」 vs 「찾은 것 없음」 · 서버가 센 `undecided` · 이름 바꾸기에도 키 불변 · 병합 행 보존 · `evidence: null` · `status: null` · `comparedTo: null` · 제안·초안·확정의 문서 불변 계약 · 이력 항목 종류 — 서버 계약의 상태 구분이라 지우면 호출자가 상태를 납작하게 만든다. 나머지는 export 요약 1줄(정책).
- `frontend/src/App.tsx` — 라우터 대신 상태 머신 + 해시 주소 · 인계 억제 · 홈 재조회 epoch · 로그아웃 시 해시 먼저 비우기 · 딥링크를 상태 머신보다 먼저 읽기 — 각각 화면 왕복, 낡은 목록, 로그인 화면 위 덮어 그리기를 막는다.
- `RegisterLlmKey.tsx` — 목업 매핑 · 화면 전용 제공자 순서(백엔드 규칙과 별개 축) · `llmkey::ACTIVE_KEY_SQL` 과 함께 움직여야 하는 중복 규칙 · `null` 초기값 · 마운트 1회 · 탭 즉시 저장 · 형식 경고만 하고 막지 않는 이유.
- `GrantRepoAccess.tsx` · `SignIn.tsx` · `HomeRepositories.tsx` — 목업 매핑 · 비인증은 다른 화면 · `signin` testid 를 지키는 이유 · `awaiting_pipeline` 배지(위에서 고침) · 설치 범위를 벗어난 작업을 숨기지 않는 합집합 · `/api/usage` 장애 격리 · 로그아웃 실패 표시 · 편집 시 추정 무효화.
- `index.css` — 토큰만 쓰는 규칙 · iOS 글자 자동 확대 · 탭바 애니메이션 예외 · `.esrc` 한정 · `open` 소유 · 블록 순서(M8) — 지우면 시각 회귀를 조용히 들인다. `format.ts` — `toLocaleString` 금지(ICU 차이로 `sc04-09` 단정과 어긋난다).

**프로토타입 하네스·설정** (L 102)
- `tools/check-journey-prototype.js` 머리 블록 — 정적 체커가 아닌 이유 · 기대값의 원천이 여정 문서인 이유(자기참조 금지) · 검사 P1~P7 · 등록부 4종과 P0 — 하네스를 고치는 사람이 무엇을 왜 굴리는지 확인할 다른 자리가 없다.
- google 키 접두사 · 출력 언어 radio 검사 · 두 회차 수치가 겹치지 않는 이유 — 각각 프로브가 공허해지는 조건을 말한다.
- SECOND_END 세 갈래의 「분기 목록이 아니라 화면 안 행동과 갈래 끝」 · 보조 레이어에서 장전하는 네 자리(운영 측 실패 · 서버가 정하는 충돌 · 열린 자리) · 「모순은 분석 결과에 딸려 온다」 — P2 가 보조 레이어 전진을 금지하므로 예외의 근거를 적어 둔 자리다.
- P1 해시 변경 추적 블록 — 뮤테이션이 잡아낸 사각(진입 시 해시만 읽는 페이지)이라 지우면 검사를 군더더기로 보고 걷는다.
- `backend/src/config.rs` 전건 — 비밀을 `Debug`/`Display` 로 유도하지 않는 이유 · 경계별 모드 선택 · env 하나가 한 경계에만 닿는 이유 · 테스트용 `all` · GitHub App 필드 요약과 `Secret` 표식 · 콜백 URL 와일드카드 불가(상류 동작)와 preview 프록시 · `preview_id` 를 숫자로 제한하는 이유(state 위조) · 빈 `worker_token` 이 닫힘이라는 것 · 파일 투영 비밀 — 자격증명 비노출과 상류 제약이 왜 그 자리에서 지켜져야 하는지를 말한다.
