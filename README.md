# feature-doc

> GitHub 저장소에 연결되어, 그 저장소가 제공하는 **최종 사용자용 기능(feature)** 을 LLM의 도움으로 발견·표현·관리하기 쉽게 만드는 도구.

이 레포는 **제품 자체의 문서 체계**를 담고 있습니다. 가치 → PRD → Acceptance Criteria → 테스트 문서의 계층 구조를 따르며, 모든 AC는 가치와 테스트에 양방향으로 연결됩니다.

## 어디서부터 읽으면 될까

처음 보신다면 다음 순서를 추천합니다:

1. **[`docs/values.md`](docs/values.md)** — 제품이 제공하려는 8개 가치(V1~V8). 모든 판단의 기준입니다.
2. **[`docs/doc-tracker.md`](docs/doc-tracker.md)** — 문서 체계의 현재 상태와 위험 진단. 한눈에 전체 구조와 건강 상태를 봅니다.
3. **PRD 4종** — 핵심 기능 단위로 구체적인 요구사항을 정의합니다.
4. **테스트 문서 4종** — 각 AC의 검증 시나리오를 정의합니다.

## 문서 구조

```
docs/
├── values.md              # 가치 문서 (최상위)
├── doc-tracker.md         # 문서 체계 상태 추적
├── design-system.md       # UI 디자인 시스템 — 토큰·컴포넌트·원칙
├── prd/
│   ├── 01-analysis-pipeline.md       # 코드베이스 분석 파이프라인
│   ├── 02-feature-representation.md  # Feature 표현 (인수 테스트 + 종단 의존성)
│   ├── 03-doc-management.md          # Feature 단위 문서 관리 (LLM CRUD)
│   └── 04-platform.md                # 플랫폼 (k8s · 자격증명 · 모바일 우선)
├── test/
│   ├── 01-analysis-pipeline.md
│   ├── 02-feature-representation.md
│   ├── 03-doc-management.md
│   └── 04-platform.md
├── user-journey/          # 사용자 여정 — README + 여정 6개 (행동 축, JRN-/STP- 식별자)
│   ├── README.md                     # 페르소나 · 여정 지도 · 마찰점 · 연결 매트릭스
│   ├── JRN-connect-repo.md           # P1 — 저장소를 맡기고 첫 분석을 걸기
│   ├── JRN-discover-features.md      # P1 — 코드에서 기능 목록 뽑아내기
│   ├── JRN-review-feature.md         # P1 — 기능 하나의 표현이 맞는지 검수하기
│   ├── JRN-understand-feature.md     # P2 — 코드를 못 읽는 사람이 기능을 이해하기
│   ├── JRN-follow-code-change.md     # P1 — 코드가 바뀐 뒤 문서가 따라왔는지 확인하기
│   └── JRN-restore-history.md        # P1 — 잘못된 변경을 되짚어 되돌리기
├── wireframes/            # 10개 모바일 화면 정보 구조 (SVG)
│   ├── README.md
│   └── s01 ~ s10 *.svg
└── mockups/               # 목업 — 디자인 시스템 적용 HTML (단독 파일)
    ├── README.md                      # 여정 페이지 규약 · 매핑 · 이관 대기 원장
    ├── JRN-connect-repo.html          # 여정 목업 (S01~S03 흡수) — 5단계를 눌러 걸어본다
    └── s04 ~ s10 *.html               # 화면 단위 — 여정 페이지로 이관 대기

tools/
└── gen-wireframes.js      # wireframe SVG 일괄 생성 스크립트

backend/                   # axum 0.8 — /hello + S01 자격증명 API(GitHub App·LLM Key, 봉투 암호화) + SQLite + dist 정적 서빙
├── Cargo.toml             # [[bin]] 2개: featuredoc(API) · featuredoc-worker(분석 워커)
├── migrations/            # SQLite 스키마 (sqlx migrate — 바이너리에 임베드)
└── src/                   # lib(config·db·auth·github·llmkey·crypto·audit·pipeline·worker_api…) + main
    └── bin/worker.rs      # 분석 워커 — DB를 열지 않고 API의 /internal 큐로 claim·보고 (AC4.5)

frontend/                  # Vite 8 + React 19 — S01 Credentials Setup 화면 (디자인 시스템 토큰)
├── package.json
├── index.html
└── src/{App.tsx, CredentialsSetup.tsx, api.ts, main.tsx, index.css}

deploy/
├── k8s/                   # 정식 매니페스트 = kustomize 베이스 (API·워커 deployment·service·pvc; secret은 외부 제공)
│   ├── kustomization.yaml
│   ├── deployment.yaml
│   ├── worker-deployment.yaml  # 분석 워커 워크로드 — 같은 이미지, 커맨드만 다름, 볼륨 없음 (AC4.5)
│   ├── service.yaml
│   ├── pvc.yaml
│   └── secret.yaml.example  # featuredoc-secrets 템플릿 — 복사·기입 후 적용 (secret.yaml은 gitignore)
└── e2e/                   # e2e 전용 오버레이 (featuredoc:dev + IfNotPresent + stub-mode secret 생성)
    ├── kustomization.yaml
    └── kind-cluster.yaml

e2e/                       # HTTP smoke (자격증명 평문 미노출 단언 포함) + Playwright spec (AC 1개당 1파일)
├── smoke.sh
└── tests/                 # 각 spec 첫 줄에 `// 검증 AC: ACx.y` 를 정확히 1개 선언 (docs/doc-tracker.md "e2e 매핑")
    ├── ac1-1-repository-connect-and-trigger.spec.ts
    ├── ac4-1-github-app-connection.spec.ts
    ├── ac4-2-llm-key-lifecycle.spec.ts
    ├── ac4-3-credential-safety.spec.ts
    ├── ac4-5-worker-workload-separation.spec.ts  # kubectl로 워커를 0·2로 스케일해 API 가용성·드레인 확인
    └── ac4-8-signin-and-session.spec.ts

scripts/
└── e2e.sh                 # kind 생성 → build → load → apply → port-forward → e2e

.github/workflows/
└── ci.yml                 # 단일 워크플로 (ARM runner) — test + e2e + ghcr 푸시

Dockerfile                 # 멀티스테이지: node 22 → rust 1.94 → debian slim
```

각 PRD에는 동일 번호의 테스트 문서가 1:1로 대응합니다. UX 디자인 산출물(`design-system.md`, `wireframes/`, `mockups/`)은 PRD가 정의한 acceptance criteria를 어떻게 화면으로 전달할지 결정하며, PRD가 변경되면 wireframe을 먼저 갱신하고 디자인 시스템 토큰으로 mockup을 다시 그리는 순서를 따릅니다.

## PRD ↔ 다루는 문제

| PRD | 다루는 핵심 문제 |
|---|---|
| [01 분석 파이프라인](docs/prd/01-analysis-pipeline.md) | 저장소 연결 → 횡단 관심사 추출 → feature 탐색 전략 → feature 후보 추출 |
| [02 Feature 표현](docs/prd/02-feature-representation.md) | 확정된 feature를 인수 테스트로 표현하고 종단 의존성을 데이터로 보존 |
| [03 문서 관리](docs/prd/03-doc-management.md) | 사용자가 LLM의 도움으로 feature 문서를 추가·수정·삭제 |
| [04 플랫폼](docs/prd/04-platform.md) | k8s 운영, 사용자 자격증명(GitHub App 설치 / LLM API Key), 모바일 우선 UX |

## Walking skeleton 실행

문서 외에 동작하는 수직 슬라이스가 함께 있습니다. axum API가 `/hello`(프로브) + 자격증명 API(`/api/*`) + `dist/`(SPA)를 같은 오리진에서 서빙하고, 그 옆에서 **별도 워크로드인 분석 워커**가 큐를 비웁니다(AC4.5 — 워커는 데이터베이스를 열지 않고 API의 `/internal` 라우트로 작업을 claim 하므로, SQLite는 계속 writer가 하나입니다). 프론트는 디자인 시스템 토큰으로 S01 Credentials Setup 화면을 그립니다. 자격증명은 SQLite(PVC)에 봉투 암호화로 저장되고, GitHub/LLM 외부 경계는 `FEATUREDOC_MODE=stub`에서 테스트 더블로 대체됩니다.

### 로컬 (k8s 없이)

```bash
# 1) 프론트 빌드
( cd frontend && npm install && npm run build )

# 2) 백엔드 실행 (frontend/dist 서빙) — S01 흐름을 외부 연동 없이 보려면 stub 모드
( cd backend && STATIC_DIR=../frontend/dist FEATUREDOC_MODE=stub cargo run --release )

# 3) 확인
curl http://localhost:8080/hello
# → {"message":"Hello from FeatureDoc backend"}
open http://localhost:8080
```

dev 모드(`cd frontend && npm run dev`)는 `/hello`를 `localhost:8080`으로 프록시합니다.

분석 큐가 실제로 비워지는 것까지 보려면 워커를 두 번째 프로세스로 띄웁니다. **두 프로세스가 같은 `FEATUREDOC_WORKER_TOKEN`을 보아야** 합니다 — API는 그 값으로 `/internal`을 열고, 워커는 그 값으로 인증합니다.

```bash
# 터미널 1 — API (위 2번 명령에 워커 토큰을 더한 것)
( cd backend && STATIC_DIR=../frontend/dist FEATUREDOC_MODE=stub \
    FEATUREDOC_WORKER_TOKEN=dev-token cargo run --release )

# 터미널 2 — 분석 워커
( cd backend && FEATUREDOC_MODE=stub FEATUREDOC_WORKER_TOKEN=dev-token \
    FEATUREDOC_API_BASE=http://127.0.0.1:8080 WORKER_ID=dev-worker \
    cargo run --release --bin featuredoc-worker )
```

토큰을 주지 않으면 API는 `/internal` 전체를 401로 닫고 워커는 아예 기동을 거부합니다 — 기본값이 "열림"이 아니라 "닫힘"입니다.

### kind 기반 e2e (docker · kind · kubectl 필요)

```bash
bash scripts/e2e.sh
```

스크립트는 kind 클러스터 생성 → docker build → `kind load docker-image` → `kubectl apply -k deploy/e2e/` → rollout 대기 → `kubectl port-forward` → smoke.sh + Playwright 실행 → 종료 시 port-forward·클러스터 정리 순서로 한 명령으로 그린까지 갑니다. 클러스터를 남기려면 `KEEP_CLUSTER=1 bash scripts/e2e.sh`.

kind 노드 이미지는 `kindest/node:v1.34.3@sha256:08497ee1…dd48` digest로 핀. `deploy/k8s/`는 운영용 kustomize 베이스로, CI가 커밋마다 고정해 넣는 불변 태그 `ghcr.io/dlddu/featuredoc:<commit sha>`를 가리킵니다. 태그가 불변이라 `imagePullPolicy`는 기본값 `IfNotPresent`가 맞고, 배포는 오직 그 태그 줄이 바뀔 때만 일어납니다. e2e는 `deploy/e2e/` 오버레이가 이 베이스를 가져와 이미지를 로컬 빌드본(`featuredoc:dev`)으로 retag하고 `imagePullPolicy: IfNotPresent`로 패치하므로, kind에 `load`한 이미지를 GHCR에서 다시 받지 않고 그대로 씁니다.

### CI (GitHub Actions)

`.github/workflows/ci.yml` — 단일 워크플로, `ubuntu-24.04-arm` runner. 세 job:

- **`test`** — `cargo test` → kind+kubectl 설치 → `scripts/e2e.sh`(docker build + 클러스터 e2e). main 푸시·모든 PR에서 실행.
- **`push`** — `needs: test`로 test 그린 후에만. `docker/setup-buildx-action` + `docker/login-action` + `docker/metadata-action` + `docker/build-push-action@v6`(GHA 캐시) 조합으로 `ghcr.io/<owner>/featuredoc`에 푸시. **태그는 커밋 SHA 하나뿐입니다** — `latest`도, 브랜치 롤링 태그도 만들지 않습니다.
- **`pin`** — `needs: push`, **main 푸시에서만**. `deploy/k8s/deployment.yaml`과 `worker-deployment.yaml`의 이미지 태그를 방금 빌드한 커밋 SHA로 바꿔 `chore(deploy): ... [skip ci]` 커밋으로 main에 되돌려 놓습니다(`contents: write`는 이 job에만 부여). 기본 `GITHUB_TOKEN`으로 나가는 푸시는 새 워크플로를 트리거하지 않으므로 재귀 빌드가 생기지 않고, `[skip ci]`가 이중 안전장치입니다.

> **태그 = 배포 상태.** 지금 운영에 무엇이 떠 있는지는 `deploy/k8s/deployment.yaml`의 태그 한 줄이 그대로 말해줍니다. PR 빌드도 이미지는 `<head sha>`로 올라가지만(preview 환경이 이 태그를 씁니다) 매니페스트를 건드리지 않으므로 **머지 전에는 운영에 닿을 수 없습니다.** 롤백은 그 줄을 되돌릴 커밋 SHA로 바꾸는 것이고, 이미지는 이미 GHCR에 있으니 재빌드가 필요 없습니다.

## 마이그레이션 규칙

**한 번 배포된 `backend/migrations/*.sql`은 한 글자도 고치지 않습니다 — 주석도 포함해서.** sqlx가 파일 전체 바이트를 SHA-384로 해싱해 `_sqlx_migrations.checksum`에 저장하고 부팅 때마다 대조하므로, 주석 한 글자만 달라져도 `VersionMismatch`로 부팅이 실패합니다. sqlx에는 Flyway `repair`에 해당하는 명령이 없습니다.

CI는 이 사고를 잡지 못합니다 — `cargo test`도 kind e2e도 매번 새 DB에서 돌아 체크섬 대조가 아예 일어나지 않기 때문입니다. 실패는 운영 볼륨에 롤아웃될 때 처음 드러납니다.

그래서 마이그레이션 주석에는 **그 시점의 기록**만 남기고, 나중에 바뀔 이름은 쓰지 않습니다. 파일 경로·목업 이름은 물론이고 AC 번호·화면 ID·여정 ID 처럼 **제품 문서 쪽에 사는 식별자도 넣지 않습니다** — 문서는 언제든 개편되는데 마이그레이션 파일만 따라갈 수 없기 때문입니다. 번호를 가리키는 대신 그 번호가 요구하는 바를 자체 완결적으로 서술합니다. 컬럼의 *현재* 의미는 코드 옆 rustdoc에 둡니다.

전체 규칙과 부득이한 repair 절차는 [`backend/migrations/README.md`](backend/migrations/README.md)에 있습니다.

## 문서 작성 원칙

- **가치 우선**: 모든 PRD/AC/테스트는 자신이 어떤 가치(V1~V8)를 달성하는지 명시합니다. 가치와 연결되지 않은 문서는 위험으로 진단됩니다.
- **AC 단위 작성**: PRD는 Acceptance Criteria 단위로 쪼개고, 각 AC는 1개 이상의 테스트 시나리오로 검증됩니다.
- **상태 추적**: 문서를 추가/수정할 때마다 [`docs/doc-tracker.md`](docs/doc-tracker.md)를 함께 갱신하여 끊어진 연결이 없는지 확인합니다.

## 현재 상태 (요약)

- 가치: **8개** / PRD: **4개** / AC: **23개** / 테스트 문서: **4개**
- 가치 미연결 AC: 0개 ✅
- 미검증 AC: 0개 ✅
- ⚠️ **제품 소유자 미지정** — 가장 우선 해결할 항목입니다. 자세한 내용은 [`docs/doc-tracker.md`](docs/doc-tracker.md)를 보세요.
