# Mockups

10개 모바일 화면의 **목업** — 색·타이포·인터랙션·디자인 톤이 모두 결정된 최종 형태. [`design-system.md`](../design-system.md)의 토큰·컴포넌트·원칙을 그대로 표현한 결과물이에요. 같은 화면의 정보 구조 합의는 [`wireframes/`](../wireframes/)에 있습니다.

- **Format** — HTML, 393 × 844 (iOS 모바일 viewport), 다크 테마
- **Style** — `design-system.md` v0.1 그대로: 5단계 표면, 1px hairline, shadow·gradient 없음, 액센트 최소
- **Self-contained** — 각 `sNN-*.html` 은 디자인 시스템을 인코딩한 CSS를 파일 안에 인라인으로 담은 단독 파일. 의존성 없이 브라우저로 바로 열면 그대로 렌더링됩니다 (와이어프레임 SVG가 각각 단독 파일인 것과 동일).
- **Fonts** — Geist (본문) · JetBrains Mono (메타·코드), Google Fonts CDN. 오프라인일 때는 시스템 sans/mono로 폴백됩니다.

각 `sNN-*.html` 은 브라우저로 바로 열면 단독으로 렌더링됩니다.

## Flows

화면 흐름·플로우 정의는 [`user-journey/`](../user-journey/)와 같습니다. 목업은 와이어프레임과 1:1로 대응해요.

### 01 — Onboarding & Connect

| ID  | Mockup | AC | Value | Purpose |
| --- | ------ | -- | ----- | ------- |
| S01 | [s01-credentials-setup.html](./s01-credentials-setup.html) | AC4.1 · AC4.2 · AC4.3 | V6 | GitHub App 연결 + LLM 자격증명 등록 |
| S02 | [s02-home-repositories.html](./s02-home-repositories.html) | AC1.1 · AC1.5 | V1 · V8 | 연결된 저장소 목록과 분석 상태 |
| S03 | [s03-connect-repository.html](./s03-connect-repository.html) | AC1.1 · AC4.6 | V6 · V8 | 새 저장소 연결 + 분석 pre-flight 비용 안내 |

### 02 — Discovery

| ID  | Mockup | AC | Value | Purpose |
| --- | ------ | -- | ----- | ------- |
| S04 | [s04-analysis-progress.html](./s04-analysis-progress.html) | AC1.5 · AC4.6 | V7 · V8 | 분석 파이프라인 진행 상황 (5단계) |
| S05 | [s05-cross-cutting-concerns.html](./s05-cross-cutting-concerns.html) | AC1.2 | V2 · V4 | 추출된 횡단 관심사 (인프라·아키텍처·프레임워크·미들웨어) |
| S06 | [s06-discovery-strategy.html](./s06-discovery-strategy.html) | AC1.3 | V1 · V2 | LLM이 만든 탐색 전략 검토 및 승인 |
| S07 | [s07-feature-candidates.html](./s07-feature-candidates.html) | AC1.4 | V1 | feature 후보 목록 + 승인/거부/병합 |

### 03 — Feature Documents

| ID  | Mockup | AC | Value | Purpose |
| --- | ------ | -- | ----- | ------- |
| S08 | [s08-feature-acceptance.html](./s08-feature-acceptance.html) | AC2.1 · AC2.2 · AC2.3 | V3 · V4 | feature의 인수 시나리오 (Given-When-Then 4개) |
| S09 | [s09-feature-dependencies.html](./s09-feature-dependencies.html) | AC2.4 · AC2.5 | V5 | 종단 의존성 그래프 + 카테고리별 의존성 목록 |
| S10 | [s10-llm-edit.html](./s10-llm-edit.html) | AC3.1 | V3 · V4 · V7 | LLM 보조 편집 — 자연어 지시 → diff → 승인 |

## Mockup index — 디자인 시스템 사용 매핑

각 목업이 [`design-system.md`](../design-system.md)의 어떤 컴포넌트를 쓰는지. 모든 화면은 인라인 CSS를 통해 §1 Foundations(토큰)와 §6 Principles(원칙)를 공통으로 따르므로, 아래는 §4 Components 중 화면별로 두드러지게 쓰인 항목만 적습니다.

| ID  | 사용 컴포넌트 (design-system §4) |
| --- | -------------------------------- |
| S01 | Input field · Button(primary·secondary) · Tag · Card · Icon container · 세그먼트 선택자 |
| S02 | Card · Tag(status badge) · Bottom tab bar · 메트릭 그리드 · 진행 바 · Section title(+New 액션) |
| S03 | Input field · Card · Button(primary·ghost) · Tag(status badge) |
| S04 | Step(done·active·todo) · Card · Button(secondary) · 진행 링 · Section title |
| S05 | Card · Section title(caps) |
| S06 | Code block(kw·str·com) · Tag · Card · Button(primary·secondary) |
| S07 | Card · Tag(필터 chip·status) · Icon container |
| S08 | Tabs · Card · Tag · Section title(caps) |
| S09 | Tabs · Card · Tag · 의존성 그래프(inline SVG) |
| S10 | Code block(diff add·del) · Card · Tag · Button(primary·secondary) |

> 이 표는 `doc-tracker.md` 검증의 입력입니다. 목업이 추가/변경되면 이 표와 위 Flows 표를 함께 갱신해야 연결 검증이 유효합니다.

## Mockup vs Wireframe

- **Wireframe** ([`wireframes/`](../wireframes/)) — 정보 구조와 화면 흐름의 합의. SVG, 무채색, 디자인 톤 없음.
- **Mockup** (이 폴더) — 디자인 톤·색·폰트·인터랙션이 결정된 최종 형태. HTML, `design-system.md` 적용.

PRD가 변경되거나 화면 흐름을 재검토할 때는 와이어프레임을 먼저 갱신하고, 그 다음 이 목업을 디자인 시스템 토큰으로 다시 그립니다.

## 화면 추가 시

1. 같은 ID의 와이어프레임이 먼저 있어야 합니다 (`wireframes/`).
2. 기존 `sNN-*.html` 의 인라인 CSS(디자인 시스템 토큰·컴포넌트)를 복사해 새 `sNN-*.html` 을 작성합니다 — 임의의 색·radius·폰트를 새로 만들지 않습니다. 디자인 시스템이 바뀌면 10개 파일의 인라인 CSS를 함께 갱신합니다.
3. 위 **Flows** 표와 **Mockup index** 표에 행을 추가합니다.
4. [`doc-tracker.md`](../doc-tracker.md)의 연결 매트릭스와 변경 이력을 갱신합니다.
