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
├── user-journey/         # 사용자 여정 — README + 플로우 4개
│   ├── README.md
│   ├── 01-onboarding-and-connect.md
│   ├── 02-discovery.md
│   ├── 03-feature-documents.md
│   └── 04-continuous-maintenance.md
├── wireframes/            # 10개 모바일 화면 정보 구조 (SVG)
│   ├── README.md
│   └── s01 ~ s10 *.svg
└── mockups/               # 10개 모바일 화면 목업 — 디자인 시스템 적용 HTML (단독 파일)
    ├── README.md
    └── s01 ~ s10 *.html

tools/
└── gen-wireframes.js      # wireframe SVG 일괄 생성 스크립트

backend/                   # axum 0.8 — /hello + S01 자격증명 API(GitHub App·LLM Key, 봉투 암호화) + SQLite + dist 정적 서빙
├── Cargo.toml
├── migrations/            # SQLite 스키마 (sqlx migrate — 바이너리에 임베드)
└── src/                   # lib(config·db·auth·github·llmkey·crypto·audit…) + main

frontend/                  # Vite 8 + React 19 — S01 Credentials Setup 화면 (디자인 시스템 토큰)
├── package.json
├── index.html
└── src/{App.tsx, CredentialsSetup.tsx, api.ts, main.tsx, index.css}

deploy/
├── k8s/                   # 정식 매니페스트 = kustomize 베이스 (deployment·service·pvc; secret은 외부 제공)
│   ├── kustomization.yaml
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── pvc.yaml
│   └── secret.yaml.example  # featuredoc-secrets 템플릿 — 복사·기입 후 적용 (secret.yaml은 gitignore)
└── e2e/                   # e2e 전용 오버레이 (featuredoc:dev + IfNotPresent + stub-mode secret 생성)
    ├── kustomization.yaml
    └── kind-cluster.yaml

e2e/                       # HTTP smoke (자격증명 평문 미노출 단언 포함) + Playwright 1개
├── smoke.sh
└── tests/s01.spec.ts

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

문서 외에 동작하는 수직 슬라이스가 함께 있습니다. 단일 axum 서비스가 `/hello`(프로브) + 자격증명 API(`/api/*`) + `dist/`(SPA)를 같은 오리진에서 서빙하고, 프론트는 디자인 시스템 토큰으로 S01 Credentials Setup 화면을 그립니다. 자격증명은 SQLite(PVC)에 봉투 암호화로 저장되고, GitHub/LLM 외부 경계는 `FEATUREDOC_MODE=stub`에서 테스트 더블로 대체됩니다.

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

### kind 기반 e2e (docker · kind · kubectl 필요)

```bash
bash scripts/e2e.sh
```

스크립트는 kind 클러스터 생성 → docker build → `kind load docker-image` → `kubectl apply -k deploy/e2e/` → rollout 대기 → `kubectl port-forward` → smoke.sh + Playwright 실행 → 종료 시 port-forward·클러스터 정리 순서로 한 명령으로 그린까지 갑니다. 클러스터를 남기려면 `KEEP_CLUSTER=1 bash scripts/e2e.sh`.

kind 노드 이미지는 `kindest/node:v1.34.3@sha256:08497ee1…dd48` digest로 핀. `deploy/k8s/`는 운영용 kustomize 베이스로, CI가 푸시하는 `ghcr.io/dlddu/featuredoc:latest`를 가리키고 `imagePullPolicy`를 명시하지 않아 `:latest` 태그 기본값인 `Always`로 동작합니다. e2e는 `deploy/e2e/` 오버레이가 이 베이스를 가져와 이미지를 로컬 빌드본(`featuredoc:dev`)으로 retag하고 `imagePullPolicy: IfNotPresent`로 패치하므로, kind에 `load`한 이미지를 GHCR에서 다시 받지 않고 그대로 씁니다.

### CI (GitHub Actions)

`.github/workflows/ci.yml` — 단일 워크플로, `ubuntu-24.04-arm` runner. 두 job:

- **`test`** — `cargo test` → kind+kubectl 설치 → `scripts/e2e.sh`(docker build + 클러스터 e2e). main 푸시·모든 PR에서 실행.
- **`push`** — `needs: test`로 test 그린 후에만. `docker/setup-buildx-action` + `docker/login-action` + `docker/metadata-action` + `docker/build-push-action@v6`(GHA 캐시) 조합으로 `ghcr.io/<owner>/featuredoc`에 푸시. 태그는 `<github.sha>` + `latest` 두 개.

## 문서 작성 원칙

- **가치 우선**: 모든 PRD/AC/테스트는 자신이 어떤 가치(V1~V8)를 달성하는지 명시합니다. 가치와 연결되지 않은 문서는 위험으로 진단됩니다.
- **AC 단위 작성**: PRD는 Acceptance Criteria 단위로 쪼개고, 각 AC는 1개 이상의 테스트 시나리오로 검증됩니다.
- **상태 추적**: 문서를 추가/수정할 때마다 [`docs/doc-tracker.md`](docs/doc-tracker.md)를 함께 갱신하여 끊어진 연결이 없는지 확인합니다.

## 현재 상태 (요약)

- 가치: **8개** / PRD: **4개** / AC: **23개** / 테스트 문서: **4개**
- 가치 미연결 AC: 0개 ✅
- 미검증 AC: 0개 ✅
- ⚠️ **제품 소유자 미지정** — 가장 우선 해결할 항목입니다. 자세한 내용은 [`docs/doc-tracker.md`](docs/doc-tracker.md)를 보세요.
