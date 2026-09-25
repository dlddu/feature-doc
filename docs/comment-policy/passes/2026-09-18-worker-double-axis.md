# 판정 상세 — 워커 · 테스트 더블 배선 축 (비경합 7파일)

- **판정일**: 2026-09-18
- **판정 범위**: `backend/src/bin/worker.rs` · `deploy/e2e/kustomization.yaml` ·
  `deploy/k8s/deployment.yaml` · `deploy/k8s/secret.yaml.example` ·
  `deploy/k8s/worker-deployment.yaml` ·
  `e2e/tests/sc04-07-api-availability-without-workers.spec.ts` ·
  `e2e/tests/sc04-08-worker-horizontal-scale.spec.ts`
  (+ 원장 2행의 **증분 재판정** `backend/src/config.rs`)
- **기준 트리**: `fae3e17` (main, #68 머지 직후)
- **reconciler task**: `tbm_feature-doc-comment-redundancy/rct_20260918-0005`

규칙은 [../README.md](../README.md), 판정 결과의 표면은 [../ledger.md](../ledger.md)에 있다.
이 파일은 이번 범위의 **근거**만 담는다.

## 범위를 이 축으로 고른 이유

**⑴ 원장이 이름으로 걸어 둔 인계 조건이 이 트리거로 풀렸다.** 원장 ②가 「자매 모델
`tbm_feature-doc-e2e-mock-policy`의 PR(#59·#60) 뒤로 미룬다」고 파킹한 12파일 / 296행은
**#60이 `ebe8657`로 머지되며 전부 자유로워졌다**. 이번 범위는 그 블록의 중심 — 워커 워크로드와
경계별 더블 배선 — 을 집는다. 파킹을 푼 커밋이 곧 이 task의 트리거다.

**⑵ 트리거가 들인 주석이 대부분 여기 생존해 있었다.** #60·#65가 더한 117행 중 #68이 집어간 것은
`FeatureAcceptance.tsx` 9행뿐이고, 남은 108행 중 **자유 풀 72행**이 이 축에 몰려 있다
(`kustomization.yaml` 21 · `worker.rs` 16 · `sc04-08` 11 · `sc04-07` 11 · `deployment.yaml` 3 ·
`secret.yaml.example` 2 · spec 머리 개명 8). 새로 들어온 주석을 가장 싸게 판정할 수 있는 시점이다.

**⑶ 같은 명제가 층을 넘어 복제된 것이 이 축의 특징이다.** 「keyless job이 LLM double 아래에서
기본 프로바이더로 폴백하던 관대함은 사라졌다」는 경위가 `worker.rs`·`sc04-07`·`sc04-08`에 거의
축자로 **3중 복제**돼 있었다. 세 파일을 한 패스에 넣지 않으면 한 곳만 지우고 나머지 둘이 남는다.

**⑷ 열린 PR 전수 대조 — 이번 7파일은 전부 무접촉.** 판정 시점 열린 PR 5건: **#69**
(`tools/check-mockup-render.py` · `frontend/src/{CrossCuttingConcerns,DiscoveryStrategy,FeatureCandidates}.tsx`) ·
**#67**(`e2e/tests/sc02-02`·`sc02-03`) · **#64**(`tools/check-data-format-change.py`) ·
**#26**(draft, `tools/check-mockup-render.py`) · **#17**(draft, `frontend/src/{ConnectRepository,CredentialsSetup,HomeRepositories}.tsx`·`index.css`).
이번 범위는 다섯 모두와 겹치지 않는다.

**⑸ 원장 2행의 미판정 증분 23행을 같은 패스에 묶었다.** #60이 `config.rs`에 `Mode`·`Doubles` doc으로
들인 23행은 #68의 원장이 「미판정 증분」으로 명시 등재했을 뿐 아무 task도 들고 있지 않았다. 그 23행이
서술하는 것이 바로 이 축의 불변식(경계별 선택)이라, 같은 눈으로 한 번에 보는 편이 정합이다. 원장 규약대로
**새 행을 만들지 않고** 2행의 결과 칸을 갱신하고, 상세는
[2026-09-18-uncontested-harness-config.md](2026-09-18-uncontested-harness-config.md)에 절을 더했다.

## 집계

| 파일 | 판정 전(지문 기준) | 판정 후 | 지문 기준 감소 | diff |
|---|---|---|---|---|
| `backend/src/bin/worker.rs` | 99 | 55 | 44 | 65행 삭제 · 21행 재작성 |
| `e2e/tests/sc04-08-worker-horizontal-scale.spec.ts` | 64 | 41 | 23 | 41행 삭제 · 18행 재작성 |
| `e2e/tests/sc04-07-api-availability-without-workers.spec.ts` | 56 | 33 | 23 | 32행 삭제 · 9행 재작성 |
| `deploy/e2e/kustomization.yaml` | 35 | 13 | 22 | 32행 삭제 · 10행 재작성 |
| `deploy/k8s/secret.yaml.example` | 21 | 15 | 6 | 9행 삭제 · 3행 재작성 |
| `deploy/k8s/deployment.yaml` | 16 | 13 | 3 | 3행 삭제 · 재작성 0 |
| `deploy/k8s/worker-deployment.yaml` | 16 | 15 | 1 | 3행 삭제 · 2행 재작성 |
| **합계 (이 패스의 새 원장 행)** | **307** | **185** | **122** | **185행 삭제 · 63행 재작성** |
| `backend/src/config.rs` (원장 2행 증분) | 64 | 51 | 13 | 23행 삭제 · 9행 재작성 |
| **총계** | **371** | **236** | **135** | **208행 삭제 · 72행 재작성** |

**코드는 한 글자도 바뀌지 않았다.** 8파일 모두 주석 줄을 들어낸 뒤 나머지를 해싱해 대조했고
(`grep -v` 주석 패턴 → `sha256sum`), 판정 전후 값이 **전건 일치**했다. `.sql`은 이 범위에 없다.

## 지우지 않은 것부터 — 기계가 읽는 주석

- `// 검증 시나리오: 04-platform.md#시나리오 7`(sc04-07) ·
  `// 검증 시나리오: 04-platform.md#시나리오 8`(sc04-08) — `tools/check-scenario-e2e.py`가 파싱하는
  매핑 선언이다. 지문·판정 모두에서 제외되며 **양쪽 다 그대로 보존**했다.
- `mock-exception:` 주석은 이 범위에 0건이다. 정책이 「활성 배선은 모킹 지점이 아니므로 예외 주석을
  달지 않는다」고 못박은 자리들이라, 애초에 붙어 있지 않다.

## 제거 — 복원 경로별

### ① 코드 자체에서 복원되는 것

- **`worker.rs` 모듈 머리의 5단계 열거 9행** — `fetch` · `cross_cutting` · `discovery_strategy` ·
  `feature_candidates` · `acceptance_dependencies`가 무엇을 하는지는 같은 파일의 `run()` 분기와
  각 스테이지 함수 이름이 그대로 말한다.
- **`/// Runs the implemented stages of one claimed job.`(`run`) · `/// Wait between polls when the
  queue is empty.`(`IDLE_POLL`) · `/// One environment variable per boundary`(`Doubles::from_env`)** —
  이름 재진술 3행.
- **`// k8s projects the pod name here; falls back to the hostname.`** — 바로 아래 두 줄
  (`env::var("WORKER_ID")` → `read_to_string("/etc/hostname")`)이 그 자체다. 매니페스트 쪽의
  `fieldRef: metadata.name`은 `worker-deployment.yaml`에 남아 있다.
- **`// Stage 3 (AC1.3) needs stage 2's document as its input, so it runs only when stage 2 actually
  produced one in this pass.`** — 바로 아래가 `if let Some(landscape) = cross_cutting_doc.as_ref()`다.
- **`Mode`의 `Real`/`Stub` 동작 설명 4행 · `Mode::from_env`의 「`stub` selects the double」 1행** —
  variant 이름과 `eq_ignore_ascii_case("stub")`이 말한다. 「unset이면 real」이라는 **안전 기본값**
  문장은 남겼다(아래 유지 목록).
- **`Doubles`의 필드 doc 4행(`EXT-01`·`EXT-02`·`EXT-04`) · `WorkerDoubles`의 필드 doc 2행
  (`EXT-03`·`LLM-01`)** — 필드 이름이 경계를 말하고, 식별자↔env↔프로세스 대응표는
  `docs/e2e-mocking-policy.md`에 있다(②로도 복원된다).
- **`kustomization.yaml`의 「Shared by the API … and the worker」 1행** — 같은 명제가
  `secret.yaml.example`과 `worker-deployment.yaml`에 더 온전한 형태로 있다. **세 벌 중 한 벌만
  남겼다**(가장 온전한 `secret.yaml.example` 쪽).
- **`deployment.yaml`의 「Non-secret runtime config」·「Secrets only: KEK + GitHub App credentials.」
  2행** — 바로 아래 `env:` / `envFrom: secretRef:` 블록이 그대로 말한다.
- **`secret.yaml.example`의 「Non-secret config … is plain env on the Deployment, not here」 4행** —
  `deployment.yaml`을 되풀이한다. 「`secret.yaml`은 gitignore돼 있다」 1행도 `.gitignore`에서 읽힌다.

### ② 저장소 문서에서 복원되는 것

- **spec 머리의 제목·본문 인용 12행** — `sc04-07`의 「시나리오 7 — 워커 파드를 모두 강제 종료해도 …」
  2행과 `sc04-08`의 「다수 분석 요청이 큐에 적재된 상태에서 …」 2행은 `docs/test/04-platform.md`
  시나리오 7·8의 **기대 결과 문장 그대로**다. 제목 재진술 2행(「「워커 다운 시 API 가용성」 전용 spec」
  등)도 같다. 기계 판독 선언 `// 검증 시나리오:`가 이미 그 문서의 절을 **가리키고 있어** 복원 경로가
  파일 안에 남는다.
- **`sc04-08`의 「API/워커 간 결합 없이 확장된다」 인용** — 같은 문서의 기대 결과 인용.
- **`kustomization.yaml` API 패치의 배선 설명 9행** — 「EXT-01 · EXT-02 · EXT-04를 켠다」,
  「등재 지점은 그것이 선택하는 코드 분기다」, 「스위치 지점에는 예외 주석을 달지 않는다」는 셋 다
  `docs/e2e-mocking-policy.md`의 허용목록 표와 표기 규약 절이 **문면으로** 규정한다.
- **워커 패치의 더블 설명 5행** — 같은 표의 EXT-03 · LLM-01 행.
- **`config.rs`의 「The allow-list for these doubles is `docs/e2e-mocking-policy.md`; every branch
  they select carries a matching `mock-exception:` comment.」 2행** — 그 문서를 가리키는 문장이
  그 문서에 있는 규칙을 되풀이한다. 정책이 「rustdoc 링크를 위해서만 문장을 남기지 않는다」고 한 자리다.
- **`kustomization.yaml` replicas 주석의 「See docs/doc-tracker.md "e2e 매핑"」** · **spec 머리의
  화면 소유자 열거(`ac1-1 / ac4-1 / sc04-03`, `sc01-02 / sc04-01 / sc04-03`)** — doc-tracker의
  e2e 매핑 표에서 복원된다.
- **AC 식별자 인용 9건** — `(AC4.5)` 4회 · `(AC1.3)` 2회 · `(AC1.4)` 2회 · `(AC1.2)`·`(AC2.1~AC2.3)`·
  `(AC1.5)`. 전부 `docs/prd/`와 `docs/test/`의 조항 번호다.

### ③④ PR · 커밋 메시지에서 복원되는 것

- **`provider_for`의 경위 서술 7행** — 「This used to fall back to a default provider … that leniency
  is gone」은 **#60의 PR 본문과 커밋 메시지**가 담고 있고, `docs/e2e-mocking-policy.md`의 변경 이력
  표(2026-09-18 · `rct_20260918-0002` 행)에도 같은 문장이 있다. 복원 경로 셋이 겹친다.
- **`sc04-07`·`sc04-08`의 같은 경위 6행씩** — 위 서술의 **축자 복제**다. 세 벌 모두에서 경위를 들어내고,
  각 자리에는 「활성 키는 stub 경로에서도 분석의 진입 조건이다」라는 **충실도 경계 1문장**만 남겼다
  (유지 근거는 아래).
- **`sc04-07` 머리의 `rct_20260916-0002`가 닫았다 3행 · `sc04-08` 머리의 「규칙 2 상 분리 대상으로
  등재됐다가 이 파일로 옮겨왔다」 5행** — 파일 분리의 경위이자 reconciler task id를 담은 **작업 흔적**.
  정책이 ③의 자리로 이름 붙인 유형이다.

### 구분선 · 절 제목

- `sc04-07` 2행 · `sc04-08` 1행 — `// ── 시나리오 7: 워커를 전부 내린다 ─────` 류. 아래 코드가
  스스로 말하는 절 제목이며, 앞선 세 패스가 같은 유형으로 총 43행을 지웠다.

## 지우지 않은 것 — 복원 불가능한 지식

- **PID 1 시그널 함정 5행**(`worker.rs`) — 「컨테이너의 PID 1은 핸들러가 없는 시그널을 커널이 버리므로
  핸들러 없이는 scale-down의 SIGTERM이 무시되고 kubelet이 grace period를 다 기다린다」. 실패 모드의
  함정이고 코드 어디에도 없다.
- **claim/lease 계약 · 중단 시점 6행**(`worker.rs`) — 「stop은 claim 사이에서만 존중된다 · 진행 중인
  claim을 취소하면 job이 LEASE_SECONDS 동안 고아가 된다」. 정책이 이름 붙인 동시성 계약.
- **lease 갱신 자리 4행**(`worker.rs`) — 「네트워크 호출 **직전에** 연장해야 느린 GitHub 응답이 재청구를
  부르지 않는다」.
- **버전 스큐 완충 3행**(`worker.rs`) — 「구 워커 × 신 API(또는 반대)가 모르는 스테이지를 보고하는 대신
  조용히 멈추도록」.
- **`awaiting_pipeline`인 이유 3행**(`worker.rs`) — 「`succeeded`가 아니다: 사람 게이트 뒤의 스테이지가
  아직 차례를 못 받았을 수 있다」. 상태 이름만으로는 복원되지 않는 판단.
- **`provider_for`의 충실도 경계 3행** — 「활성 키는 LLM double이 켜져도 진입 조건으로 남는다 — 여기서
  거부하는 것이 stub 경로가 real보다 관대해지지 않게 한다」. 정책이 유지로 못박은 **stub↔real 충실도
  경계**다. 경위(③④)만 들어내고 경계 자체는 남겼다.
- **`Doubles`·`WorkerDoubles`의 경계별 선택 불변식 각 3행** — 「한 프로세스가 자기가 돌리지 않는 double을
  켤 수 없다」. #60이 만든 설계 불변식이고, 코드는 그 결과만 보일 뿐 **왜 그렇게 나눴는지**는 말하지 않는다.
- **`Mode::from_env`의 안전 기본값 2행** — 「말하지 않은 배포는 아무것도 stub되지 않는다」.
- **`kustomization.yaml`의 `FEATUREDOC_STUB_LLM_FAIL` **부재의 이유** 3행** — 「여기 두지 않는다:
  sc01-06이 자기 워커 리스 안에서 주입하고 되돌린다(`e2e/support/cluster.ts setWorkerEnv`)」.
  **없는 것**에 대한 설명이라 어떤 복원 경로에도 없다.
- **워커 리스 계약 5행**(`kustomization.yaml`의 `replicas: 0`) — 「쉬는 상태가 0인 이유 · 리스 방식 ·
  `workers: 1`이 겹침을 막는 이유」. 스펙 머리의 Isolation 문단과 명제가 겹치지만, **매니페스트가
  왜 0인가**는 매니페스트 쪽에만 있는 질문이라 남겼다(doc-tracker 참조와 spec 이름 열거만 들어냄).
- **`kind load` 함정 3행**(`kustomization.yaml`) — 「IfNotPresent를 강제하지 않으면 노드가 적재된
  이미지를 무시하고 GHCR에서 당긴다」.
- **`imagePullPolicy`의 의도된 부재 3행**(`deployment.yaml`) — 「태그가 불변 커밋 SHA라 기본값이 맞다」.
- **운영 이미지 pin 경고 4행**(`deployment.yaml`) — 「CI가 관리하는 줄 · 운영 값은 deploy 브랜치 ·
  손으로 고치지 말 것」. 워크플로에서 *읽어낼 수는* 있으나, 이것은 **그 줄을 고치려는 사람이 바로 그
  자리에서 읽어야 하는 경고**다. 「애매하면 남긴다」를 적용했다.
- **Recreate 전략의 이유**(`deployment.yaml`) · **RollingUpdate가 안전한 이유**(`worker-deployment.yaml`) —
  RWO 볼륨에 두 파드를 붙이지 않는다 / 워커는 볼륨이 없고 리스가 만료되면 job이 큐로 돌아온다.
- **워커에 DATABASE_URL·볼륨이 **없는** 이유 4행**(`worker-deployment.yaml`) — 단일 writer 제약.
- **`worker_token` 빈 값의 fail-closed 3행**(`secret.yaml.example`) — 「비워 두면 API가 모든
  `/internal`을 거부하고 워커는 시작하지 않는다」.
- **KEK 1행**(`secret.yaml.example`) — 「레코드별 DEK를 감싸는 키」. 봉투 암호화 불변식.
- **`/hello`가 마이그레이션 이후에만 답한다 1행**(`deployment.yaml`) — readinessProbe가 왜 그 경로인지.

## 판단이 갈려 남긴 것 (3건)

1. **`sc04-07`·`sc04-08` 머리의 Isolation 문단(각 10행)** — `kustomization.yaml`의 `replicas: 0`
   주석과 명제가 상당 부분 겹친다. 그쪽을 남겼으니 이쪽은 ①로 복원된다고 볼 여지가 있으나, spec을
   읽는 사람이 「왜 이 파일만 `scaleWorkers`를 부르는가」를 매니페스트까지 가서 읽어야 한다면 그건
   원본이 부실한 쪽이다. **양쪽 다 남기고 다음 패스의 재판정 후보로 적는다.**
2. **`sc04-08`의 「Every line, not the default --tail=10」** — `workerLogs()` 구현을 보면 복원되지만,
   그 함수가 왜 전량을 가져오는지는 **호출자의 단정**(claim 횟수 세기)에서만 설명된다. 남겼다.
3. **`worker-deployment.yaml`의 「API와 같은 이미지, 같은 SHA」 2행** — `image:` 두 줄을 비교하면
   복원되지만, pin job이 두 파일을 **함께** 바꾼다는 보장은 워크플로에만 있다. 남겼다.

## 검증

- **코드 무접촉** — 8파일에서 주석 줄을 제거한 나머지의 sha256이 판정 전후 동일.
- **범위 지문** — 이 패스의 7파일: `b03ef6d3…`(307행) → `96569bfb…`(185행).
  원장 2행(`check-journey-prototype.js` + `config.rs`): `ea29e09f…`(153행) → `ddacce0b…`(140행).
- **전역 지문 예고** — 부모 `fae3e17` = `lines=3055 files=97` / `b1f90262…`에서 순 제거 135행
  (이 범위 122 + 증분 재판정 13) → **`lines=2920 files=97` /
  `d4c000e483d4ad39c88a4c8e57e2cecf756cdc35a0041d0e9ae0e71d87db324c`**.
- 지문 계산 규약: 원장의 **범위 지문**은 후행 개행을 **포함**하고(`echo "$HITS" | sha256sum`),
  모델의 **전역 지문**은 versionScript 그대로 후행 개행을 **제외**한다(`printf '%s'`). 같은 입력에도
  두 값은 다르다.
- **허브(`docs/index.html`)** — 새 패스 파일 하나가 `docs/`에 늘었으므로 `tools/check-journey-mockup.py`
  R9가 요구하는 대로 요약 `Documents` 26 → 27로 올리고 문서 링크 행을 더했다. `doc-tracker.md`는
  손대지 않았다 — R9가 그 파일에서 읽는 것은 목업 집계뿐이고, 이 패스는 목업을 건드리지 않는다.
  (그 파일은 열린 PR **#67**·**#69** 둘 다와 경합이기도 하다.)

## 범위 밖으로 남긴 것 (다음 패스가 받는다)

이 패스 뒤의 미판정 잔여는 **73파일 / 1,775행**이다(전역 2,920 − 판정 24파일 / 1,145).

1. **열린 PR 경합** — `sc02-02` 31행(**#67**) · `tools/check-mockup-render.py` 71행(**#69**·**#26**) ·
   `frontend/src/{CrossCuttingConcerns,DiscoveryStrategy,FeatureCandidates}.tsx`(**#69**) ·
   `frontend/src/{ConnectRepository,CredentialsSetup,HomeRepositories}.tsx`·`index.css` 100행(**#17**) ·
   `tools/check-data-format-change.py`(**#64**). 각 PR이 닫힌 뒤 집는다.
2. **`backend/migrations/*.sql` 7파일 / 104행** — 본문 「적용된 마이그레이션」 절의 전용 PR · 수동
   repair · **사람 승인 게이트**를 거치는 별도 패스다. 다섯 패스 연속 이월 중이며, 함대의
   `awaiting_review` 백로그가 병목인 동안에는 여는 시점을 사람 판단에 맡긴다.
3. **자유 풀 나머지** — 밀도 높은 후보(tip 기준): `frontend/src/api.ts` 72 ·
   `backend/src/feature_candidates.rs` 63 · `tools/check-journey-mockup.py` 55 ·
   `e2e/tests/sc01-07-…` 49 · `backend/src/repo_scan.rs` 49 · `e2e/tests/sc01-04-…` 47 ·
   `backend/src/discovery_strategy.rs` 46 · `e2e/tests/sc01-03-…` 45 · `tools/check-scenario-e2e.py` 44 ·
   `backend/tests/candidates.rs` 44. **#60이 푼 12파일 블록 중 이번에 집지 않은 잔여**
   (`sc04-04`·`sc04-12`·`sc04-13`의 머리 블록 등, 개명 추종 8행 포함)도 여기 있다 — 그 8행은
   `FEATUREDOC_MODE=stub` → `FEATUREDOC_DOUBLE_*=stub` **개명 추종이지 새 명제가 아니다.**
4. **위 「판단이 갈려 남긴 것」 1번** — spec 머리 Isolation 문단과 `kustomization.yaml` replicas 주석의
   중복. 둘 중 어느 쪽이 원본인지는 다음 패스가 정한다.
5. **`doc-tracker.md` L14의 「공개된 문서 22개 … 22/22」** — 3차 패스가 남긴 한 줄 부채. 실측은 이제
   27편이다. #67·#69가 닫힌 뒤 한 줄로 고친다.

## 증분 재판정 ② — `#92` 가 `deploy/e2e/kustomization.yaml` 에 더한 2행 (2026-09-21 · `rct_20260921-0007`)

`#92`(`51daa9c`, 슬라이스 6a)가 API Deployment 패치의 env 목록에 `FEATUREDOC_DOUBLE_LLM: "stub"`
을 더하며 그 위에 쓴 주석 2행 「편집 제안(AC3.1)은 워커가 아니라 이 프로세스가 모델을 부른다 —
파이프라인 단계와 같은 이름의 변수를 여기서도 읽는다」를 판정해 **전건 제거**했다.

- 「편집 제안은 워커가 아니라 이 프로세스가 모델을 부른다」 — ② doc-tracker `2026-09.md`
  슬라이스 6a 행 「**제안은 큐를 타지 않는다** — … API 프로세스가 직접 한 번 부른다」 · ③ PR #92
  「설계 판단 1」 — `backend/src/doc_edit.rs` 머리의 같은 문단도 이 패스가 같은 근거로 걷었다.
  AC 꼬리표.
- 「파이프라인 단계와 같은 이름의 변수를 여기서도 읽는다」 — ① 같은 파일 아래 워커 패치의
  `FEATUREDOC_DOUBLE_LLM` · `backend/src/config.rs` `Doubles` doc 「the analysis worker holds its own
  set」(원장 2행 유지분).

결과: 5행의 줄 수·지문이 #92 이전 값 **190 / `632b0475…`** 으로 되돌아왔다(부모 `19d58fa`
재계산과 바이트 동일). 매니페스트는 주석 제거 후 부모와 바이트 동일(stripper md5 `6d2a26da`) —
env 항목 자체는 그대로다. 새 파일 쪽 판정은 [2026-09-21-doc-edit-axis.md](2026-09-21-doc-edit-axis.md).

## 증분 재판정 ③ — `#107` 이 `deploy/e2e/kustomization.yaml` 에 더한 2행 (2026-09-21 · `rct_20260921-0010`)

`#107`(`89a1625`, 슬라이스 6b)이 API 컨테이너 env 에 `FEATUREDOC_DOUBLE_REPO_SCAN: "stub"` 을 더하며 그 위에
쓴 2행 「빠진 기능 직접 추가(AC3.2)의 근거 찾기는 이 프로세스가 트리를 한 번 더 스캔한다 — 워커의
1단계와 같은 이름의 변수를 여기서도 읽는다」를 판정해 **전건 제거**했다 — ② 가 `FEATUREDOC_DOUBLE_LLM`
위의 같은 모양을 걷은 판정 그대로다. 복원 경로: ② doc-tracker `2026-09.md` 슬라이스 6b 행(「`FEATUREDOC_DOUBLE_LLM`
을 API 도 읽는 것과 같은 방식으로 저장소 트리 스캔의 더블 이름 `FEATUREDOC_DOUBLE_REPO_SCAN` 도 API 가
읽는다」) · ② `docs/e2e-mocking-policy.md` EXT-03 배선 표 · ③ PR #107 본문 「API 컨테이너
`FEATUREDOC_DOUBLE_REPO_SCAN=stub`」 · ① `config.rs` `Doubles::from_env` 와 `bin/worker.rs` 가 같은 이름을 읽는
코드 — AC 꼬리표.

결과: 5행의 줄 수·지문이 #107 이전 값 **190 / `632b0475…`** 으로 되돌아왔다(부모 `24f488d` 재계산과
바이트 동일). 파일은 주석 제거 후 부모와 바이트 동일(stripper md5 `e39333f3`). 새 파일 쪽 판정은
[2026-09-21-feature-add-axis.md](2026-09-21-feature-add-axis.md).

## 증분 재판정 ④ — `#108` 이 `bin/worker.rs` 에 더한 7행 (2026-09-22 · `rct_20260922-0001`)

사람 PR **#108**(AC4.9 출력 언어 설정)이 `backend/src/bin/worker.rs` 에 **7행**을 들여왔다. 판정 맥락은
[2026-09-22-output-language-axis.md](2026-09-22-output-language-axis.md). **전건 제거 7 · 유지 0.**

- **`Claim.llm_language` 필드 doc 2행** (「The output language snapshotted onto this analysis when it was
  triggered. / Absent for an analysis triggered before the setting existed.」) — 앞 문장은 스냅숏 명제의 사본
  (정본은 `analysis.rs::create` 의 복사 지점), 뒤 문장은 이 PR 이 일곱 자리에 적은 명제의 사본(정본은
  `settings.rs::analysis_language`). 필드 이름 `llm_language` 와 `Option` 이 나머지를 복원한다(①).
- **`language_for` doc 5행** (「Which language this job's prose is written in. Read from the job, not the user,
  for the same reason as [`Self::provider_for`]: the stages of one analysis are claimed separately, across
  approval gates, and must not end up half in one language and half in another because the setting moved
  between them.」) —
  - 첫 문장은 fn 이름·시그니처의 재진술(①).
  - 「Read from the job, not the user」는 본문이 문자 그대로 `job.llm_language` 를 읽는다(①).
  - 이어지는 이유절은 **바로 위 `provider_for` 의 doc**(「Shared by every LLM-backed stage so they cannot
    disagree about it mid-job」)이 같은 패턴에 대해 이미 말하는 것을 다시 편 것이고, 불변식을 만드는 쪽의
    정본은 `analysis.rs::create` 다. 「for the same reason as [`Self::provider_for`]」만 남기는 선택지는 정책이
    막는다 — 링크를 위해서만 문장을 남기지 않는다.

지문: **197행 `e3005ab0…` → 190행 `632b0475182bf04b9ec8c21f3b69c5b91172ecee41e417f7a58b98cdb15ba66d`** —
**이 패스 직전 원장 값으로 바이트 그대로 복귀**했다(#108 이 이 행에 들여온 주석이 전건 제거로 판정됐고, 편집이
다른 줄을 건드리지 않았다는 독립 증거).

## 증분 재판정 ⑤ — `#121` · `#137` 이 `bin/worker.rs` 에 연 5행 (2026-09-24 · `rct_20260924-0001`)

원장 5행이 「판정하지 않고 다음 감지에 넘긴다」로 등재해 둔 **#121 3행**(「자매 착지 재실측」
2026-09-22)과, 그 뒤 착지한 **#137**(`fd6cdad`)의 **2행**을 함께 판정한다. 다섯 줄 전부
`backend/src/bin/worker.rs` 한 파일이다.

### 제거 2행 — 같은 명제의 두 벌째 (#121)

```rust
// Seeded from the claim when stage 3 re-runs without stage 2 (AC1.5): the
// landscape it plans over is the one already stored, left untouched.
let mut cross_cutting_doc: Option<serde_json::Value> = job.cross_cutting_document.clone();
```

- 「Seeded from the claim when stage 3 re-runs without stage 2」는 **같은 PR 이 40행에 더한 필드 doc**
  (`/// Stage 2's stored document, present when stage 3 is re-run on its own.`)의 두 벌째다. 정본은
  그 필드 쪽이다 — 값이 언제 실리는지는 API↔워커 와이어 계약이고 `Claim` 이 그 계약의 자리다.
  「복제된 명제는 강제하는 쪽 한 벌만」(원장 1행 증분 재판정 ⑤·⑨·⑪, 9행 ③)과 같은 잣대.
- `(AC1.5)` 는 **AC 꼬리표**다(이 패스의 「③④」 절이 같은 형태로 8행을 걷었다).
- 「left untouched」는 바로 그 줄의 `.clone()` 이 말한다(①).

### 유지 1행 — 필드 doc (#121)

`/// Stage 2's stored document, present when stage 3 is re-run on its own.` 는 남긴다. **언제 Some 인가**는
`Option` 이 말하지 않고, `Claim` 의 자매 필드 세 개가 이 패스 이후 모두 같은 모양으로 살아남았다
(`/// Empty until the reviewer approves — which is also when stage 4 is not offered.` 외 2). 이 구조체에서
「이 필드가 언제 채워지는가」 한 줄은 유지 규약이다.

### 2행 → 1행 재작성 — 임대 갱신 (#137)

```rust
// Reading a handful of files is several round trips; renew again so the
// model call starts with a full lease.
```

- 뒤 절 「renew again so the model call starts with a full lease」는 **626행이 이미 말한다**
  (`// The model call is the long one in this job; renew before it as `fetch` does.`) — 같은 함수 안의
  두 벌째다. 이 패스가 「13벌까지 복제된 워커 임대 문단」을 걷어 7개 호출부 중 3곳만 남긴 그 기준을
  그대로 적용한다.
- 앞 절 「Reading a handful of files is several round trips」는 **왜 한 번 더 갱신하는가**이고, 그것만
  남으면 626행과 겹치지 않는다. `#137` PR 본문이 같은 말을 적지만(③), 임대 staleness 는 정책 본문이
  **유지 대상으로 명시 열거한 「워커 임대·claim/lease 동시성 계약」**이라 「애매하면 남긴다」를 따른다.
- 결과: `// Reading a handful of files is several round trips of its own.`

### 값

판정 5행 · **순 제거 3행 · 유지 2행**. 195(#137 착지 후) → **192 /
`b36ee596e5bf5353de6ca3cb8ea3dac138ca58ec2370c5d0ca493a4968e2dcc3`**.
이 행에 **미판정 증분은 남지 않는다.**

**자매 착지 재실측(2026-09-25 · #141)** — 머지 직전에 #141(`c405ad7`, 지문 사각지대 축)이 착지해
`e2e/tests/sc04-07-api-availability-without-workers.spec.ts` · `sc04-08-worker-horizontal-scale.spec.ts`
의 주석을 **제자리 수정**했다 — 5행의 줄 수는 195 → 193 이고 지문만 갈렸다. 원장 「자매 착지 재실측」
규약대로 **판정을 다시 하지 않고 줄 수·지문만 재고정**한다:
**190 / `80748ab3041408f76823e2cac502b043fef3ebe055628efc883742b36ae593a5`**.
이 패스의 순 제거 −3행은 불변이다(193 → 190).
이 시점의 전건 산술: 전역 2732 → **2672 /
`2ed67e589879ca7fd094e68239f6da7d998ccc8ba2d3ba3e75db253102d65897`** · 행 합 2523 → 2463 ·
잔여 **209 불변** ⇒ **행 합 −60 == 전역 −60, 잔차 0**.

## 증분 재판정 ⑥ — `#148` 이 `bin/worker.rs` · `deploy/e2e/kustomization.yaml` 에 더한 5행 (2026-09-25 · `rct_20260925-0013`)

`#148`(`67c15ad`, e2e 폴링 단축)이 워커 유휴 폴링을 env 로 덮을 수 있게 만들면서 이 행에 **순 +5행**을
열었다(190 → 195 / `2394fbc09d8f799417e9efb81f5e4a97badd995430f2b1955e81c136b4009c27`). 두 파일에
걸치지만 **하나의 명제가 두 벌로 적힌 것**이라 함께 본다.

### 제거 3행 — 비공개 const 의 `///` (`bin/worker.rs`)

```rust
/// Default pause after an empty claim. `FEATUREDOC_WORKER_IDLE_POLL_MS` overrides
/// it — the e2e overlay lowers it because every queue hand-off in a spec otherwise
/// waits out up to this long.
const IDLE_POLL: Duration = Duration::from_secs(2);
```

- **「`pub` 항목의 `///` 요약 1줄 유지」 조항의 대상이 아니다** — `IDLE_POLL` 은 **비공개 const** 다.
- 1행 「Default pause after an empty claim」은 ① 이다: 이름 `IDLE_POLL` + `Duration::from_secs(2)` +
  claim 루프의 `Ok(None) => idle_poll` 한 줄이 「빈 claim 뒤 이만큼 쉰다」를 그대로 말한다.
- 2행 「`FEATUREDOC_WORKER_IDLE_POLL_MS` overrides it」도 ① 이다 — **같은 파일 아래의 신설
  `fn idle_poll()`** 이 그 env 를 읽어 파싱하고 실패 시 `IDLE_POLL` 로 떨어지는 코드 그 자체다.
  이름을 주석이 한 번 더 적을 뿐이다.
- 2~3행 「the e2e overlay lowers it because every queue hand-off in a spec otherwise waits out up to
  this long」은 ①(overlay 가 `FEATUREDOC_WORKER_IDLE_POLL_MS: "200"` 으로 값까지 보인다)과
  ③(PR #148 본문 §변경 1항 「**e2e overlay 에서만** 200ms. 운영 기본값은 그대로」 · 2항 「큐 hand-off 가
  세 번이라 hop 마다 두 폴링 대기가 겹쳤습니다」)로 이중 복원된다.

**바로 아래 `ERROR_BACKOFF` 의 `///` 1행을 유지한 것과 어긋나지 않는다.** 그 줄
(`/// Back-off when the API is unreachable, so a restarting API is not hammered.`)은 이 패스가 판정해
**유지**한 쌍둥이지만, 그 한 줄이 담은 **왜**(재기동 중인 API 를 두들기지 않으려고)는 이름에도
호출부에도 다른 문서에도 없다 — 네 경로 어디에서도 복원되지 않는다. `IDLE_POLL` 의 **왜**는 ③ 본문에
축자로 있다. 같은 규칙에 술어 값이 다를 뿐이라 **선례를 뒤집는 것이 아니다.**

### 제거 2행 — 같은 명제의 두 벌째 (`deploy/e2e/kustomization.yaml`)

```yaml
                  # 기본 2 s 면 spec 의 큐 hand-off(claim → 전략 승인 → 후보 확정)마다
                  # 최대 2 s 씩 쉰다. 클러스터에 이 spec 하나만 도는 e2e 에선 짧아도 된다.
                  - name: FEATUREDOC_WORKER_IDLE_POLL_MS
                    value: "200"
```

- 위 `bin/worker.rs` 3행과 **같은 명제의 두 벌째**다. 이 행이 이미 두 번 같은 자리를 같은 이유로
  닫았다 — **증분 재판정 ②**(#92, 이 파일의 API env 위 2행 · 전건 제거)와 **③**(#107, 같은 파일
  API env 위 2행 · 전건 제거). `#148` 이 더한 것은 **또 새 env 위의 2행**이고, 이번에도 ③(PR 본문)이
  그 명제를 축자로 갖는다.
- 남는 자리를 따로 세우지 않는다 — 두 벌 중 어느 쪽도 네 경로 밖의 지식을 담고 있지 않아, 「정본을
  어디에 둘 것인가」가 아니라 **둘 다 복원 가능**한 경우다.

### 값

판정 5행 · **순 제거 5행 · 유지 0행**. 195 → **190 /
`80748ab3041408f76823e2cac502b043fef3ebe055628efc883742b36ae593a5`** 로, ②·③·④ 와 같이
**#148 이전 값으로 바이트 동일 복귀**한다. 이 행에 **미판정 증분은 남지 않는다.**

> 같은 창(`#148`)이 들여온 신설 파일 `e2e/support/clock.ts` 9행은 이 행이 아니라 **행 밖 잔여**이며
> [2026-09-25-e2e-clock-helper.md](2026-09-25-e2e-clock-helper.md) 가 **전건 유지**로 판정했다.
