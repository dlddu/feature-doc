# Mockups

10개 모바일 화면의 **목업** — 색·타이포·인터랙션·디자인 톤이 모두 결정된 최종 형태. [`design-system.md`](../design-system.md)의 토큰·컴포넌트·원칙을 그대로 표현한 결과물이에요. 같은 화면의 정보 구조 합의는 [`wireframes/`](../wireframes/)에 있습니다.

> **이관 중**: 목업의 매칭 단위를 **화면 1개 = 파일 1개**에서 **여정 1개 = 페이지 1개**로 옮기는 중입니다. 플로우 1은 [`../journeys/flow-01-onboarding-and-connect.html`](../journeys/flow-01-onboarding-and-connect.html)로 이관 완료(S01~S03 흡수), 플로우 2·3은 아직 화면 단위 파일 7개로 남아 있고, 플로우 4는 전용 페이지를 아직 만들지 않았습니다. 규약과 진행 상태는 아래 [여정 페이지 규약](#여정-페이지-규약)과 [`doc-tracker.md`](../doc-tracker.md)의 "여정 단위 목업 마이그레이션"을 보세요.

## 여정 페이지 규약

여정 하나를 한 페이지에서 걸어보게 만드는 **여정 목업**의 규약. 이 절이 여정 ↔ 목업 페이지 매핑의 단일 소스이며, 이관 슬라이스는 여기 정해진 경로·속성·식별자를 그대로 씁니다 (수립: 2026-08-15, reconciler `rct_20260815-0001`).

**경로와 파일명** — 여정 페이지는 `docs/journeys/flow-0N-<슬러그>.html` 1개 = 여정 1개. 화면 단위 파일(`docs/mockups/sNN-*.html`)은 이 단위로 흡수될 대상이며, 흡수된 화면 파일은 삭제합니다(내용은 여정 페이지 안에 그대로 보존되고 원본은 git 이력에 남습니다).

**선언 속성** — 여정 페이지는 루트 `<body>`에 `data-journey="JRN-<슬러그>"`를 **정확히 1개** 선언하고, 각 단계 섹션이 `id`와 같은 값의 `data-step="STP-<슬러그>"`를 선언합니다. 아직 이관되지 않은 화면 단위 파일도 자기 `<body>`에 `data-journey`/`data-step`을 1개씩 달아, 이관 전에도 단계 집합을 기계적으로 대조할 수 있게 합니다. `END-*` id를 가진 요소는 갈래의 끝을 표현하는 안내 블록이며 **단계가 아닙니다**(`data-step`을 달지 않습니다).

**식별자 레지스트리** — 단계 식별자는 전역 순번(`S01`…)에서 슬러그로 이관합니다. 순번은 화면이 중간에 추가되면 뒤 번호가 밀려 목업·인덱스·추적 문서 세 곳이 동시에 어긋나기 때문입니다. 아래 표가 정식 식별자이고, 순번은 기존 문서·와이어프레임과의 대조를 위한 별칭으로만 남깁니다. **한 번 등재된 식별자는 재사용하지 않습니다**(단계가 사라지면 폐기 사실을 여정 문서 변경 이력에 남기고 식별자는 비워 둡니다).

| 여정 | 여정 식별자 | 여정 문서 | 단계 식별자 (별칭) |
| --- | --- | --- | --- |
| 1 진입과 연결 | `JRN-onboarding-and-connect` | [`01-onboarding-and-connect.md`](../user-journey/01-onboarding-and-connect.md) | `STP-credentials-setup` (S01) · `STP-home-repositories` (S02) · `STP-connect-repository` (S03) |
| 2 자동 발견 | `JRN-discovery` | [`02-discovery.md`](../user-journey/02-discovery.md) | `STP-analysis-progress` (S04) · `STP-cross-cutting-concerns` (S05) · `STP-discovery-strategy` (S06) · `STP-feature-candidates` (S07) |
| 3 Feature 문서 | `JRN-feature-documents` | [`03-feature-documents.md`](../user-journey/03-feature-documents.md) | `STP-feature-acceptance` (S08) · `STP-feature-dependencies` (S09) · `STP-llm-edit` (S10) |
| 4 지속적 유지 | `JRN-continuous-maintenance` | [`04-continuous-maintenance.md`](../user-journey/04-continuous-maintenance.md) | `STP-reanalysis-diff` (시나리오 A) · `STP-edit-conflict` (B) · `STP-rediscovered-feature` (C) · `STP-change-history` (D, S11 후보) |

플로우 4의 단계는 화면이 아니라 **재진입 시나리오**입니다 — 같은 화면이라도 여정마다 데이터와 다음 행동이 다르므로 그 여정의 맥락으로 각각 존재하는 것이 정상이고, 따라서 플로우 4는 예외 등재가 아니라 전용 페이지를 갖습니다(결정 근거는 [`doc-tracker.md`](../doc-tracker.md) "여정 단위 목업 마이그레이션"). `STP-change-history`는 화면이 아직 없어 **수용된 위험**으로 남아 있으며, 페이지에서는 진입점만 표현합니다.

**여정 페이지가 만족해야 할 것** — (a) 그 여정의 모든 단계 포함 (b) 단계 전환 수단 (c) 현재 위치 표시 (d) 각 단계의 주요 행동을 눌러 전진 가능 (e) `#<step-id>` 딥링크 (f) 분기 선택과 각 갈래의 끝 표현 (g) 빌드·네트워크 없이 열리는 정적 동작. 폰트만 CDN이며 오프라인에서는 시스템 sans/mono로 폴백됩니다.

**만드는 방법** — 화면 본문을 손으로 다시 그리지 않습니다. 기존 `sNN-*.html`의 `<body>` 본문을 그대로 옮기고 스타일 블록 1벌을 공유한 뒤, 여정 크롬(스텝퍼·분기·딥링크·전진 배선)만 덧붙입니다. 10개 목업의 인라인 CSS는 `s02`(`.section-action` 16줄이 더 있는 상위집합)를 빼면 바이트 동일하므로 한 벌로 덮입니다.

- **Format** — HTML, 393 × 844 (iOS 모바일 viewport), 다크 테마
- **Style** — `design-system.md` v0.1 그대로: 5단계 표면, 1px hairline, shadow·gradient 없음, 액센트 최소
- **Self-contained** — 각 `sNN-*.html`과 각 여정 페이지는 디자인 시스템을 인코딩한 CSS를 파일 안에 인라인으로 담은 단독 파일. 의존성 없이 브라우저로 바로 열면 그대로 렌더링됩니다 (와이어프레임 SVG가 각각 단독 파일인 것과 동일).
- **Fonts** — Geist (본문) · JetBrains Mono (메타·코드), Google Fonts CDN. 오프라인일 때는 시스템 sans/mono로 폴백됩니다.

## Flows

화면 흐름·플로우 정의는 [`user-journey/`](../user-journey/)와 같습니다. 목업은 와이어프레임과 1:1로 대응해요. 여정 페이지로 이관된 화면은 페이지 안의 앵커(`#STP-*`)가 그 화면의 공개 경로입니다.

### 01 — Onboarding & Connect · 여정 페이지 [`journeys/flow-01-onboarding-and-connect.html`](../journeys/flow-01-onboarding-and-connect.html)

| ID  | 단계 식별자 | Mockup | AC | Value | Purpose |
| --- | ----------- | ------ | -- | ----- | ------- |
| S01 | `STP-credentials-setup` | [flow-01 #STP-credentials-setup](../journeys/flow-01-onboarding-and-connect.html#STP-credentials-setup) | AC4.1 · AC4.2 · AC4.3 | V6 | GitHub App 연결 + LLM 자격증명 등록 |
| S02 | `STP-home-repositories` | [flow-01 #STP-home-repositories](../journeys/flow-01-onboarding-and-connect.html#STP-home-repositories) | AC1.1 · AC1.5 | V1 · V8 | 연결된 저장소 목록과 분석 상태 |
| S03 | `STP-connect-repository` | [flow-01 #STP-connect-repository](../journeys/flow-01-onboarding-and-connect.html#STP-connect-repository) | AC1.1 · AC4.6 | V6 · V8 | 새 저장소 연결 + 분석 pre-flight 비용 안내 |

분기(여정 문서 `## 화면 흐름`): **첫 방문** `STP-credentials-setup` → `STP-connect-repository` 직행 · **재방문** `STP-home-repositories`에서 시작해 새 저장소 추가(→ `STP-connect-repository`) 또는 기존 분석 진입(→ 플로우 2/3 인계). 두 갈래의 끝은 페이지의 `END-analysis-queued` · `END-existing-analysis` 블록이 표현합니다.

### 02 — Discovery · 여정 페이지 미이관 (화면 단위 4개)

| ID  | 단계 식별자 | Mockup | AC | Value | Purpose |
| --- | ----------- | ------ | -- | ----- | ------- |
| S04 | `STP-analysis-progress` | [s04-analysis-progress.html](./s04-analysis-progress.html) | AC1.5 · AC4.6 | V7 · V8 | 분석 파이프라인 진행 상황 (5단계) |
| S05 | `STP-cross-cutting-concerns` | [s05-cross-cutting-concerns.html](./s05-cross-cutting-concerns.html) | AC1.2 | V2 · V4 | 추출된 횡단 관심사 (인프라·아키텍처·프레임워크·미들웨어) |
| S06 | `STP-discovery-strategy` | [s06-discovery-strategy.html](./s06-discovery-strategy.html) | AC1.3 | V1 · V2 | LLM이 만든 탐색 전략 검토 및 승인 |
| S07 | `STP-feature-candidates` | [s07-feature-candidates.html](./s07-feature-candidates.html) | AC1.4 | V1 | feature 후보 목록 + 승인/거부/병합 |

### 03 — Feature Documents · 여정 페이지 미이관 (화면 단위 3개)

| ID  | 단계 식별자 | Mockup | AC | Value | Purpose |
| --- | ----------- | ------ | -- | ----- | ------- |
| S08 | `STP-feature-acceptance` | [s08-feature-acceptance.html](./s08-feature-acceptance.html) | AC2.1 · AC2.2 · AC2.3 | V3 · V4 | feature의 인수 시나리오 (Given-When-Then 4개) |
| S09 | `STP-feature-dependencies` | [s09-feature-dependencies.html](./s09-feature-dependencies.html) | AC2.4 · AC2.5 | V5 | 종단 의존성 그래프 + 카테고리별 의존성 목록 |
| S10 | `STP-llm-edit` | [s10-llm-edit.html](./s10-llm-edit.html) | AC3.1 | V3 · V4 · V7 | LLM 보조 편집 — 자연어 지시 → diff → 승인 |

### 04 — Continuous Maintenance · 여정 페이지 미제작

전용 화면 없이 플로우 2·3의 화면에 재진입하는 흐름입니다. 재진입 맥락(거부 사유 표시·충돌 배너·병합 diff·이력 진입점)이 여정마다 다르므로 **전용 여정 페이지를 만드는 것이 이 흐름의 수렴 방향**이며(결정: 2026-08-15, [`doc-tracker.md`](../doc-tracker.md)), 시나리오 D의 변경 이력 화면(`STP-change-history` · S11 후보)은 기존 **수용된 위험**으로 남습니다.

## Mockup index — 디자인 시스템 사용 매핑

각 목업이 [`design-system.md`](../design-system.md)의 어떤 항목을 쓰는지. 모든 화면은 인라인 CSS를 통해 §1 Foundations(토큰)와 §6 Principles(원칙)를 공통으로 따르므로, 아래 **§4 컴포넌트** 열은 화면별로 두드러지게 쓰인 §4 Components 항목만 적습니다. §4 컴포넌트가 아닌 요소(§2 타이포 역할이나 화면 전용 일회성 요소)는 **§4 외 요소** 열에 따로 둡니다 — 이쪽은 디자인 시스템 §4 컴포넌트 커버리지 검증 대상이 아닙니다.

| ID  | 사용 §4 컴포넌트 | §4 외 요소 |
| --- | ---------------- | ---------- |
| S01 | Input field · Button(primary·secondary) · Tag · Card · Icon container · Segment selector | — |
| S02 | Card · Tag(status badge) · Bottom tab bar · Metric grid · Progress bar | Section title (§2.2 타이포 역할) |
| S03 | Input field · Card · Button(primary·ghost) · Tag(status badge) | — |
| S04 | Step(done·active·todo — active의 회전 ring 포함) · Card · Button(secondary) | Section title (§2.2 타이포 역할) |
| S05 | Card | Section title (§2.2 타이포 역할) |
| S06 | Code block(kw·str·com) · Tag · Card · Button(primary·secondary) | — |
| S07 | Card · Tag(필터 chip·status) · Icon container | — |
| S08 | Tabs · Card · Tag | Section title (§2.2 타이포 역할) |
| S09 | Tabs · Card · Tag | 의존성 그래프 (화면 전용 inline SVG — 디자인 시스템 컴포넌트 아님) |
| S10 | Code block(diff add·del) · Card · Tag · Button(primary·secondary) | — |

> 이 표는 `doc-tracker.md` 검증의 입력입니다. 목업이 추가/변경되면 이 표와 위 Flows 표를 함께 갱신해야 연결 검증이 유효합니다. **§4 외 요소** 열의 항목은 §4 컴포넌트 커버리지(사용처 없는 컴포넌트 / 미정의 항목 사용) 검증에서 제외됩니다.
>
> S01~S03 행의 컴포넌트 사용처는 여정 페이지 `journeys/flow-01-onboarding-and-connect.html`의 해당 `#STP-*` 섹션입니다 — 화면 본문이 그대로 옮겨졌을 뿐이라 §4 커버리지는 이관 전후로 변하지 않습니다. 여정 페이지가 덧붙인 크롬(스텝퍼·분기·안내 카드)은 그 페이지 전용 레이어이며 §4 컴포넌트 커버리지 대상이 아닙니다.

## Mockup vs Wireframe

- **Wireframe** ([`wireframes/`](../wireframes/)) — 정보 구조와 화면 흐름의 합의. SVG, 무채색, 디자인 톤 없음.
- **Mockup** (이 폴더) — 디자인 톤·색·폰트·인터랙션이 결정된 최종 형태. HTML, `design-system.md` 적용.

PRD가 변경되거나 화면 흐름을 재검토할 때는 와이어프레임을 먼저 갱신하고, 그 다음 이 목업을 디자인 시스템 토큰으로 다시 그립니다.

## 화면 추가 시

1. 같은 ID의 와이어프레임이 먼저 있어야 합니다 (`wireframes/`).
2. 그 화면이 속한 여정이 이미 여정 페이지를 가지면 **새 파일을 만들지 말고** 그 페이지에 `data-step` 섹션을 추가합니다. 아직 이관 전이면 기존 `sNN-*.html` 의 인라인 CSS(디자인 시스템 토큰·컴포넌트)를 복사해 새 `sNN-*.html` 을 작성합니다 — 임의의 색·radius·폰트를 새로 만들지 않습니다. 디자인 시스템이 바뀌면 목업 파일 전부(현재 여정 페이지 1개 + 화면 단위 7개)의 인라인 CSS를 함께 갱신합니다.
3. 위 [여정 페이지 규약](#여정-페이지-규약)의 식별자 레지스트리에 `STP-<슬러그>` 행을 추가하고, **Flows** 표와 **Mockup index** 표에 행을 추가합니다.
4. [`doc-tracker.md`](../doc-tracker.md)의 연결 매트릭스와 변경 이력을 갱신하고, 허브 [`../index.html`](../index.html)의 링크 목록을 이 문서의 매핑과 일치시킵니다.
