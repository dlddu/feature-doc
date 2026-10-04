# 2026-10-04 — 필요성 판정 (워커·배포 매니페스트 · GitHub App·인증 · 자격증명·LLM 키 · 테스트 하네스)

- **reconciler task**: `tbm_feature-doc-comment-necessity/rct_20261004-0002`
- **기준**: [README.md](../README.md) 「필요성 시험」 — *이 주석을 지우면, 이 코드를 고치러 온 사람이 틀린
  판단을 하거나 그 판단에 필요한 사실을 확인하는 데 상당한 비용을 치르는가?* 사유를 한 문장으로 댈 수
  없으면 지운다.
- **범위 선택**: 판정 칸이 `—` 인 행은 L 12 · E 1 이었다. 마이그레이션 `.sql` 5행은 전용 PR · 사람 repair 몫이고
  (열린 PR #223), `backend/src/db.rs` 행은 같은 PR 의 `backend/tests/migrations.rs` 와 파일이 겹쳐 뺐다. 남은 것은
  backend 집중 4파일 덩어리(L 628 + 같은 파일 `analysis.rs` 의 E 1행 3줄 = 631)와 덩어리 다섯(워커 190 · GitHub App
  84 · 자격증명·LLM 키 42 · 테스트 하네스 36 · `push.rs` 1 = 353)이다. 631 덩어리는 단독으로 예산 400을 넘으므로
  그 덩어리만 한 슬라이스가 가져가야 하고, 나머지 다섯을 더해도 400에 못 미치므로 이번 슬라이스는 다섯 덩어리
  353줄을 가져간다(예산보다 작게 끝나는 사유: 남은 대상이 631 덩어리뿐이다).

## 판정 — 5행 353줄

| 덩어리 | 표면 | 판정 전 | 판정 뒤 | 제거 |
|---|---|---|---|---|
| 워커 · 더블 배선 축 7파일(`backend/src/bin/worker.rs` · `deploy/` 4 · `sc04-07` · `sc04-08`) | L | 190 | 167 | 23 |
| GitHub App · 인증 경계 축 14파일(`backend/src/github_app.rs` …) | L | 84 | 81 | 3 |
| Web Push 1파일(`backend/src/push.rs`) | L | 1 | 1 | 0 |
| 자격증명 · LLM 키 경계 축 8파일(`backend/tests/llmkey.rs` …) | L | 42 | 42 | 0 |
| 테스트 하네스 축 5파일(`backend/tests/worker.rs` …) | L | 36 | 35 | 1 |
| 합 | | **353** | **326** | **27** |

비주석 diff 는 0줄이다.

### 제거 목록

| 자리 | 유형 |
|---|---|
| `backend/src/bin/worker.rs` `ERROR_BACKOFF` doc | 코드 재진술 — 이름과 값이 백오프라는 것을 말한다 |
| 같은 파일 `WorkerDoubles` 요약 줄과 빈 doc 행 | 코드 재진술 — 「어느 경계를 더블로 답하는가」는 구조체 이름과 두 필드가 말한다. 남긴 둘째 문단(「`config::Doubles` 와 공유하지 않는 이유」)은 홀로 서게 고쳤다(재작성 2) |
| `deploy/k8s/deployment.yaml` 「SQLite lives on the mounted PVC; created + migrated on first boot.」 | 코드 재진술 — `DATABASE_URL` 의 `/data` · `mode=rwc` 와 바로 아래 readiness 주석이 같은 것을 말한다 |
| `e2e/tests/sc04-07-…` · `sc04-08-…` Isolation 장문(9줄 · 8줄) | 문서 재진술 — `e2e/support/cluster.ts` 머리의 임대 규약을 거의 축자로 되풀이하고, `sc04-07` 쪽은 옛 식별자(`ac1-1` · `ac1-5`)를 가리킨다. 임대하는 다른 spec 12개가 쓰는 1줄 규약 「Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.」으로 바꿨다(각 1줄 재작성) — 공유 규약에서의 이탈이 아니라 그 규약으로의 수렴이다 |
| 같은 두 spec `finally` 의 「Back to the overlay's resting state (0) …」 2줄씩 | 문서 재진술 — `cluster.ts` 의 임대 규약(「back to 0 in `finally`」)이고, 다른 임대 spec 은 같은 `finally` 에 주석이 없다 |
| `backend/src/github_app.rs` `STUB_ACCESS` 요약 줄(「Narrows what the stub installation grants (AC4.1's …)」)과 빈 doc 행 | 코드 재진술 · 작업 흔적 — env 이름과 `stub_granted_names` 가 말하고, AC 꼬리표는 이력이다. 「설정하지 않으면 전체 설치」 문단은 남겼다 |
| `backend/tests/github.rs` 「The token must be encrypted at rest, not stored as plaintext.」 | 단정 재진술 — 바로 아래 `ciphertext` 조회와 단정이 그 문장이다 |

### 틀린 주석 — 고침

| 자리 | 무엇이 틀렸나 | 고친 문장 |
|---|---|---|
| `backend/tests/worker.rs` `claim_hands_over_the_target_and_the_executable_stage` 의 단계 4 주석 | 「단계 4 가 빠진 원인은 둘 — 미구현, 그리고 전략 승인 게이트」— 단계 4 는 구현돼 있다(`bin/worker.rs` 의 `approved_patterns` · `pipeline.rs` 의 승인 대기). 남은 원인은 게이트 하나다 | 「Stage 4 is not offered until the user approves the strategy.」(2줄 → 1줄) |

### 유지 목록 (묶음마다 필요 사유 한 문장)

**워커 · 더블 배선 축** (L 167)
- `backend/src/bin/worker.rs` 모듈 머리 — 워커가 DB·볼륨을 갖지 않아 0 과 N 으로 스케일된다는 것과, 승인 뒤 재클레임에서 이미 성공한 단계를 다시 돌리면 LLM 예산을 다시 쓰고 승인된 제안을 갈아엎는다는 것 — 지우면 「빠진 단계를 마저 돌리는」 판단을 막을 것이 없다.
- `Claim` 필드 넷(`cross_cutting_document` · `approved_patterns` · `approved_candidates` · `dependency_requests`) — 비어 있는 것이 곧 「아직 게이트 전」이라는 프로토콜 의미라, 지우면 빈 값을 오류로 다룬다.
- `WorkerDoubles` 의 「`config::Doubles` 와 공유하지 않는 이유」 — 공유하면 한 프로세스가 자기가 돌리지 않는 더블을 켤 수 있게 된다.
- PID 1 신호 처리 · 「정지는 클레임 사이에서만」 · 정지가 0 대기보다 이기는 이유 · 모르는 단계는 돌려준다 · 네트워크 호출 전 임대 연장(셋) · 신·구 빌드 혼재에서 일찍 멈추는 이유 · 단계 실패 ≠ 작업 실패 · 승인 뒤 재클레임이 앞 단계 산출을 입력으로 받지 않는 이유 · 기능 하나의 실패가 작업을 죽이지 않음 · `succeeded` 가 아닌 종료 · 의존 단계의 별도 경로 · 제공자 단일화와 stub 이 real 보다 관대해지지 않는 조건 · 실패 보고를 호출자에 두는 규약 — 각각 동시성 계약(임대·클레임)과 실패 모드의 함정이다.
- `deploy/e2e/kustomization.yaml` 전건 — `IfNotPresent` 강제(노드가 로드한 이미지를 무시하고 GHCR 에서 당긴다) · 시크릿 없이 도는 이유 · 워커 0 에서 시작하는 이유와 임대 · `FEATUREDOC_STUB_LLM_FAIL` 을 여기 두지 않는 이유(`sc01-06` 이 임대 안에서 넣고 뺀다).
- `deploy/k8s/deployment.yaml` — RWO 볼륨 위 SQLite 라 두 파드 동시 마운트 금지 · CI 가 관리하는 이미지 줄(손으로 고치지 않는다) · `imagePullPolicy` 생략 이유 · 더블은 경계별 opt-in 이라 운영은 아무것도 켜지 않는다 · `/hello` 가 마이그레이션 뒤에만 답한다.
- `deploy/k8s/secret.yaml.example` 전건 — 배포 절차(시크릿은 kustomization 밖에서 따로 적용)가 적힌 유일한 자리(README 는 한 줄 요약)이고, KEK 길이 · App JWT 발급자 · 빈 워커 토큰이 안전한 기본값인 이유는 값을 채우는 사람이 다른 데서 확인할 수 없다.
- `deploy/k8s/worker-deployment.yaml` 전건 — 원자적 클레임이라 레플리카 수가 처리량만 바꾼다 · 볼륨이 없어 무중단 롤링 · 같은 이미지·같은 SHA 핀 · `DATABASE_URL` 이 없는 이유(단일 writer) · 워커 id 기록 · 토큰을 API 와 공유하는 이유.
- `sc04-07` · `sc04-08` 머리(브라우저 아래에서 검증하는 이유 · exactly-once 는 `backend/tests/worker.rs` 가 먼저 지킨다는 경계) · 임대 규약 1줄 · 스텁 사용자 2행(e2e spec 공유 규약) · `signInWithApp` JSDoc(UI 를 거치지 않는 이유) · 활성 키가 진입 조건 · 0 을 명시하는 이유 · 「잃지 않고 기다린다」 · 버스트를 0 에서 넣는 이유 · 스케일 아웃의 무조건성 · `--tail` 기본값 함정 · 「파드마다 하나 이상」으로 단정하지 않는 이유와 분석 id 로 맞추는 이유 · 「일을 해서 비웠다」.

**GitHub App · 인증 경계 축** (L 81)
- 모듈 머리 `//!` 9파일 — doc 주석 수준(정책). `github_app.rs` · `github_api.rs` 머리의 「상류 실패를 고정 문자열로 매핑한다(토큰·키가 로그에 닿지 않게)」는 비노출 불변식이 왜 그 자리에서 지켜져야 하는지다.
- `size_kb` 는 추정이지 접근 판단이 아니다 · `STUB_ACCESS` 미설정 = 다른 spec 이 기대는 저장소 셋 · stub 저장소 수와 `repository_count` 동기 · 설치 목록 엔드포인트가 이미 App 범위 · Setup URL `installation_id` 는 서명되지 않아 위조 가능(상류 동작) · stub 설치 id 를 이 파일에 두는 이유 · `iss` 는 client ID(상류 규약) — 각각 상류의 문서화되지 않은 동작과 stub 의 충실도 경계다.
- `github.rs` — Setup URL 도 단일 등록 origin 이라 state 에 PR 번호를 싣는다 · Setup URL 엔 state 가 오지 않아 best-effort CSRF · 로컬 행이 없을 때 두 번째 설치 제안 함정 · 연결 조회가 실패를 화면 실패로 올리지 않는 이유. `auth.rs` · `github_api.rs` 의 preview 콜백 origin · `redirect_uri` 정확 일치 · `?as=` 격리가 기대는 고유성 · 양수 i64 시프트.
- `backend/tests/github.rs` 「real 이라면 Forbidden」(stub 이 real 과 갈리는 지점) · 콜백이 착지하지 않은 사용자 상황 · 「한 번 채택하면 매 렌더 다시 묻지 않는다」. `backend/tests/auth.rs` 공유 DB 경로(시계 해상도 경합) · `/internal` 을 닫는 빈 토큰 · 태그는 자격이 아니다 · `Set-Cookie` 순서 무보장.
- `sc04-01` · `sc04-11` · `sc04-12` 머리(더블 배선 · 스텁 사용자 격리 · 기본 사용자 소유) — e2e spec 공유 규약. `sc04-12` 의 직전 쿠키 포착 이유.

**Web Push** (L 1)
- `backend/src/push.rs` 모듈 머리 — doc 주석 수준(정책)이고 데이터 모델 ERD 의 `의미` 링크 대상이다.

**자격증명 · LLM 키 경계 축** (L 42)
- `backend/tests/llmkey.rs` — 형식상 유효한 키를 쓰는 이유(거부가 키 검증이 아니라 지원 범위에서 와야 한다) · 더 오래된 OpenAI 키를 고른 이유 · 초 단위 타임스탬프 타이 · 등록 시점에 거부해야 하는 이유 · 게이트를 걷고 실측한 설정이라는 것 — 각각 테스트가 공허해지는 조건이다.
- `backend/src/audit.rs` 머리(`detail` 에 비밀 금지) · best-effort 기록 — 자격증명 비노출 불변식과 실패 격리.
- `backend/tests/security.rs` 빈 `/internal` 토큰 · e2e `sc04-03` · `sc04-04` · `sc04-05` · `sc04-13` 머리(더블 배선 · 스텁 사용자) — 공유 규약. 활성 키 표시의 0 해석 · 한 버튼이 등록과 진행을 겸함(둘) · 키 등록 화면으로 가는 조작이 없는 이유 · 센티널 평문 유일성 · reload 해야 서버가 되돌려 준 문서를 본다 — 각각 단정을 오독하게 만드는 함정이다.

**테스트 하네스 축** (L 35)
- `backend/tests/worker.rs` — 협조적 스케줄러에서는 경합이 증명되지 않는다 · 단계 4 게이트(위에서 고침) · stub 도 실물 모양의 설치 토큰을 발급한다 · 프로세스 전역 stub 스위치를 건드리지 않는 이유 · 접근 회수의 두 시점과 임대 경계 · 값 비교만으로 「둘 다 실렸다」를 못 잡는다.
- `backend/tests/common/mod.rs` — 모듈별 `dead_code` 허용 이유 · 같은 시계 틱의 경로 충돌(실제로 DB 파일을 공유했다).
- `scripts/e2e.sh` — 샤드 인자 · port-forward 감시 재기동(롤아웃 생존) · 그 로그를 출력하는 이유 · 워커 0 이어도 롤아웃을 기다리는 이유.
- `sc02-02` 임대 규약 1줄 · `is_test_path` 복제 결합 — 한쪽만 고치면 갈라진다.
