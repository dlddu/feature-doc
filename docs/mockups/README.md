# Mockups

FeatureDoc 의 **목업** — 색·타이포·인터랙션·디자인 톤이 모두 결정된 최종 형태. [`design-system.md`](../design-system.md)의 토큰·컴포넌트·원칙을 그대로 표현한 결과물이에요. 같은 화면의 정보 구조 합의는 [`wireframes/`](../wireframes/)에 있습니다.

이 폴더에는 두 종류의 파일이 있습니다. **여정 페이지**(`JRN-<슬러그>.html`)는 사용자 여정 하나를 한 페이지에서 걸어보게 만든 것이고, **화면 단위 파일**(`sNN-*.html`)은 아직 여정 페이지로 이관되지 않은 화면입니다. 목표 상태는 여정 페이지만 남는 것이며, 이관은 여정 하나씩 진행합니다 — 규약과 남은 분량은 아래 [여정 페이지 규약](#여정-페이지-규약)에 있습니다.

- **Format** — HTML, 393 × 844 (iOS 모바일 viewport), 다크 테마
- **Style** — `design-system.md` v0.1 그대로: 5단계 표면, 1px hairline, shadow·gradient 없음, 액센트 최소
- **Self-contained** — 각 파일은 디자인 시스템을 인코딩한 CSS 를 파일 안에 인라인으로 담은 단독 파일. 의존성 없이 브라우저로 바로 열면 그대로 렌더링됩니다 (와이어프레임 SVG 가 각각 단독 파일인 것과 동일).
- **Fonts** — Geist (본문) · JetBrains Mono (메타·코드), Google Fonts CDN. 오프라인일 때는 시스템 sans/mono 로 폴백됩니다.

## 여정 페이지 규약

여정 하나를 한 페이지에서 걸어보게 만드는 **여정 목업**의 규약입니다. 이 절이 여정 ↔ 목업 페이지 매핑의 단일 소스이고, 이관 슬라이스는 여기 정해진 경로·속성·추출 방식을 그대로 씁니다 (수립: 2026-08-30, reconciler `rct_20260830-0001`).

**① 경로와 파일명** — 여정 페이지는 `docs/mockups/JRN-<슬러그>.html` 1개 = 여정 1개. 흡수된 화면 단위 파일은 삭제합니다(내용은 여정 페이지 안에 바이트 동일하게 보존되고 원본은 git 이력에 남습니다).

> `docs/journeys/` 같은 별도 디렉터리에 두지 않는 이유가 있습니다. 이 산출물의 변경을 감지하는 쪽이 `docs/mockups` 트리 해시와 `docs/index.html` 을 보기 때문에, 그 밖에 두면 **여정 페이지를 아무리 고쳐도 변경이 감지되지 않습니다**. 목업 파일은 목업 폴더 안에 둡니다.

**② 선언 속성** — 여정 페이지는 루트 `<body>` 에 `data-journey="JRN-<슬러그>"` 를 **정확히 1개** 선언하고, 각 단계 섹션이 `id` 와 같은 값의 `data-step="STP-<슬러그>"` 를 선언합니다. `END-*` id 를 가진 블록은 갈래의 끝이며 **단계가 아닙니다**(`data-step` 을 달지 않습니다).

**③ 식별자는 여기서 정의하지 않습니다** — 여정 `JRN-*` 와 단계 `STP-*` 의 원천은 [`user-journey/`](../user-journey/) 의 여정 문서이고(각 문서의 `### \`STP-…\`` 헤딩), 이 README 와 여정 페이지는 그 값을 **인용**할 뿐입니다. 식별자 표를 여기에 따로 두면 SSOT 가 둘이 되어 조용히 어긋납니다. 기존 식별자는 바꾸지 않고, 단계가 사라져도 재사용하지 않습니다.

**④ 단계 → 화면 매핑은 파싱합니다** — 한 단계가 어떤 화면을 보여줄지는 창작하지 않고 그 단계의 `- **터치포인트**:` 줄에 등장하는 `S01`~`S10` 토큰을 그대로 씁니다. 화면이 0개인 단계(아직 시각화가 없는 단계)는 화면을 지어내지 않고 "시각화 없음" 자리로 남기며, 사유는 [`doc-tracker.md`](../doc-tracker.md) 「수용된 위험」에 등재합니다.

**⑤ 화면 본문은 손으로 다시 그리지 않습니다** — 원본 `sNN-*.html` 의 `<body>` 내용을 **바이트 동일**하게 옮기고(`<!--embed:SNN:begin-->`/`:end-->` 마커 사이가 그 구간이며 `data-sha256` 이 지문입니다), 스타일은 화면 목업의 `<style>` 1벌만 인라인합니다 — 화면 목업 10개의 CSS 는 `s02-home-repositories.html`(이관으로 삭제됨) 을 빼면 서로 바이트 동일하고 s02 만 `.section-action` 16줄이 더 있는 상위집합이라, 그 상위집합 1벌이 모든 화면을 덮습니다. 화면 안의 자체 진행 표기(예: S01 의 `Step 1 / 1`)는 원본 그대로 두고, 여정 진행은 페이지 크롬의 스텝퍼가 표시합니다.

**⑥ 여정 페이지가 만족해야 할 것** — (a) 그 여정의 모든 단계 포함 (b) 단계 전환 수단 (c) 현재 위치 표시 (d) 각 단계의 주요 행동을 눌러 전진 가능 (e) `#STP-*` 딥링크 (f) 분기 선택과 각 갈래의 끝 표현 (g) 빌드·네트워크 없이 열리는 정적 동작. 폰트만 CDN 이며 오프라인에서는 시스템 폰트로 폴백됩니다.

이 규약의 기계 검사는 [`tools/check-journey-mockup.py`](../../tools/check-journey-mockup.py) 가 CI 에서 수행합니다.

## Journeys — 여정 ↔ 목업 페이지 매핑

여정 문서에서 파싱한 값입니다. 「예외」는 [`doc-tracker.md`](../doc-tracker.md) 「수용된 위험」에 여정 단위로 등재되어 목업 페이지가 없어도 되는 여정입니다.

<!-- jmap:begin -->
| 여정 | 여정 문서 | 목업 페이지 | 단계 | 상태 |
| --- | --- | --- | --- | --- |
| `JRN-connect-repo` 저장소를 맡기고 첫 분석을 걸기 | [`JRN-connect-repo.md`](../user-journey/JRN-connect-repo.md) | [`JRN-connect-repo.html`](./JRN-connect-repo.html) | 5 | ✅ 이관 완료 |
| `JRN-discover-features` 코드에서 기능 목록 뽑아내기 | [`JRN-discover-features.md`](../user-journey/JRN-discover-features.md) | — | 5 | ⬜ 이관 대기 |
| `JRN-follow-code-change` 코드가 바뀐 뒤 문서가 따라왔는지 확인하기 | [`JRN-follow-code-change.md`](../user-journey/JRN-follow-code-change.md) | — | 4 | ⬜ 이관 대기 |
| `JRN-restore-history` 잘못된 변경을 되짚어 되돌리기 | [`JRN-restore-history.md`](../user-journey/JRN-restore-history.md) | — | 3 | 예외 (수용된 위험) |
| `JRN-review-feature` 기능 하나의 표현이 맞는지 검수하기 | [`JRN-review-feature.md`](../user-journey/JRN-review-feature.md) | — | 5 | ⬜ 이관 대기 |
| `JRN-understand-feature` 코드를 못 읽는 사람이 기능을 이해하기 | [`JRN-understand-feature.md`](../user-journey/JRN-understand-feature.md) | — | 4 | ⬜ 이관 대기 |
<!-- jmap:end -->

집계: 여정 **6**개 · 규칙 8 예외 **1**개 · 판정 대상 **5**개 · 이관 완료 **1**개 · 이관 대기 **4**개. **이관 대기 상한: 4** — 이관 대기 여정은 4개를 넘을 수 없습니다. 여정을 하나 이관할 때마다 이 숫자를 함께 내립니다(면제가 아니라 래칫이라, 되돌리면 CI 가 실패합니다).

### 이관 완료 — `JRN-connect-repo`

단계와 임베드 화면은 여정 문서에서 파싱한 것입니다.

<!-- steps:JRN-connect-repo:begin -->
| # | 단계 | 임베드 화면 | 공개 경로 |
| --- | --- | --- | --- |
| 1 | `STP-sign-in` 본인 확인하고 들어오기 | — (시각화 없음 · 수용된 위험) | [#STP-sign-in](./JRN-connect-repo.html#STP-sign-in) |
| 2 | `STP-grant-repo-access` 들여다볼 범위를 내가 정하기 | S01 | [#STP-grant-repo-access](./JRN-connect-repo.html#STP-grant-repo-access) |
| 3 | `STP-register-llm-key` 분석 비용을 낼 키 맡기기 | S01 | [#STP-register-llm-key](./JRN-connect-repo.html#STP-register-llm-key) |
| 4 | `STP-pick-target` 분석할 저장소와 브랜치 고르기 | S02 · S03 | [#STP-pick-target](./JRN-connect-repo.html#STP-pick-target) |
| 5 | `STP-confirm-cost` 비용을 확인하고 시작 누르기 | S03 | [#STP-confirm-cost](./JRN-connect-repo.html#STP-confirm-cost) |
<!-- steps:JRN-connect-repo:end -->

## 화면 단위 잔여 — 이관 대기 원장

아직 여정 페이지로 이관되지 않은 화면 단위 파일입니다. **면제가 아니라 래칫입니다** — 체커는 (a) 이 원장에 없는 새 미선언 파일 (b) 이미 이관됐는데 원장에 남아 있는 공전 행 (c) 아래 상한 초과를 전부 실패로 만듭니다.

이관을 막고 있는 것은 **화면 공유**입니다. `S04`·`S07` 은 두 여정이, `S08`~`S10` 은 세 여정이 터치포인트로 씁니다. 여정 페이지 하나가 화면을 흡수하면서 원본을 삭제하려면 그 화면을 쓰는 여정이 **전부** 이관돼야 하므로, 공유 화면의 처리 방식을 정하는 것이 다음 슬라이스의 선결 과제입니다. (`S01`~`S03` 은 `JRN-connect-repo` 전용이라 선결 판단 없이 이관할 수 있었습니다.)

<!-- ledger:begin -->
| 파일 | 화면 | 쓰는 여정 | 해소 조건 |
| --- | --- | --- | --- |
| [`s04-analysis-progress.html`](./s04-analysis-progress.html) | S04 | `JRN-discover-features` · `JRN-follow-code-change` | 이 화면을 쓰는 여정이 전부 여정 페이지를 가지면 흡수·삭제 |
| [`s05-cross-cutting-concerns.html`](./s05-cross-cutting-concerns.html) | S05 | `JRN-discover-features` | 이 화면을 쓰는 여정이 전부 여정 페이지를 가지면 흡수·삭제 |
| [`s06-discovery-strategy.html`](./s06-discovery-strategy.html) | S06 | `JRN-discover-features` | 이 화면을 쓰는 여정이 전부 여정 페이지를 가지면 흡수·삭제 |
| [`s07-feature-candidates.html`](./s07-feature-candidates.html) | S07 | `JRN-discover-features` · `JRN-follow-code-change` | 이 화면을 쓰는 여정이 전부 여정 페이지를 가지면 흡수·삭제 |
| [`s08-feature-acceptance.html`](./s08-feature-acceptance.html) | S08 | `JRN-review-feature` · `JRN-understand-feature` · `JRN-follow-code-change` | 이 화면을 쓰는 여정이 전부 여정 페이지를 가지면 흡수·삭제 |
| [`s09-feature-dependencies.html`](./s09-feature-dependencies.html) | S09 | `JRN-review-feature` · `JRN-understand-feature` · `JRN-follow-code-change` | 이 화면을 쓰는 여정이 전부 여정 페이지를 가지면 흡수·삭제 |
| [`s10-llm-edit.html`](./s10-llm-edit.html) | S10 | `JRN-review-feature` · `JRN-understand-feature` · `JRN-follow-code-change` | 이 화면을 쓰는 여정이 전부 여정 페이지를 가지면 흡수·삭제 |
<!-- ledger:end -->

**상한: 7** — 화면 단위 잔여 파일은 7개를 넘을 수 없습니다. 이관이 끝날 때마다 이 숫자를 함께 내립니다.

## 화면 단위 잔여 — 화면별 매핑

| ID  | Mockup | AC | Value | Purpose |
| --- | ------ | -- | ----- | ------- |
| S04 | [s04-analysis-progress.html](./s04-analysis-progress.html) | AC1.5 · AC4.6 | V7 · V8 | 분석 파이프라인 진행 상황 (5단계) |
| S05 | [s05-cross-cutting-concerns.html](./s05-cross-cutting-concerns.html) | AC1.2 | V2 · V4 | 추출된 횡단 관심사 (인프라·아키텍처·프레임워크·미들웨어) |
| S06 | [s06-discovery-strategy.html](./s06-discovery-strategy.html) | AC1.3 | V1 · V2 | LLM이 만든 탐색 전략 검토 및 승인 |
| S07 | [s07-feature-candidates.html](./s07-feature-candidates.html) | AC1.4 | V1 | feature 후보 목록 + 승인/거부/병합 |
| S08 | [s08-feature-acceptance.html](./s08-feature-acceptance.html) | AC2.1 · AC2.2 · AC2.3 | V3 · V4 | feature의 인수 시나리오 (Given-When-Then 4개) |
| S09 | [s09-feature-dependencies.html](./s09-feature-dependencies.html) | AC2.4 · AC2.5 | V5 | 종단 의존성 그래프 + 카테고리별 의존성 목록 |
| S10 | [s10-llm-edit.html](./s10-llm-edit.html) | AC3.1 | V3 · V4 · V7 | LLM 보조 편집 — 자연어 지시 → diff → 승인 |

## Mockup index — 디자인 시스템 사용 매핑

각 목업이 [`design-system.md`](../design-system.md)의 어떤 항목을 쓰는지. 모든 화면은 인라인 CSS를 통해 §1 Foundations(토큰)와 §6 Principles(원칙)를 공통으로 따르므로, 아래 **§4 컴포넌트** 열은 화면별로 두드러지게 쓰인 §4 Components 항목만 적습니다. §4 컴포넌트가 아닌 요소(§2 타이포 역할이나 화면 전용 일회성 요소)는 **§4 외 요소** 열에 따로 둡니다 — 이쪽은 디자인 시스템 §4 컴포넌트 커버리지 검증 대상이 아닙니다.

| ID  | 공개 경로 | 사용 §4 컴포넌트 | §4 외 요소 |
| --- | --------- | ---------------- | ---------- |
| S01 | [JRN-connect-repo #STP-grant-repo-access](./JRN-connect-repo.html#STP-grant-repo-access) | Input field · Button(primary·secondary) · Tag · Card · Icon container · Segment selector | — |
| S02 | [JRN-connect-repo #STP-pick-target](./JRN-connect-repo.html#STP-pick-target) | Card · Tag(status badge) · Bottom tab bar · Metric grid · Progress bar | Section title (§2.2 타이포 역할) |
| S03 | [JRN-connect-repo #STP-confirm-cost](./JRN-connect-repo.html#STP-confirm-cost) | Input field · Card · Button(primary·ghost) · Tag(status badge) | — |
| S04 | [s04-analysis-progress.html](./s04-analysis-progress.html) | Step(done·active·todo — active의 회전 ring 포함) · Card · Button(secondary) | Section title (§2.2 타이포 역할) |
| S05 | [s05-cross-cutting-concerns.html](./s05-cross-cutting-concerns.html) | Card | Section title (§2.2 타이포 역할) |
| S06 | [s06-discovery-strategy.html](./s06-discovery-strategy.html) | Code block(kw·str·com) · Tag · Card · Button(primary·secondary) | — |
| S07 | [s07-feature-candidates.html](./s07-feature-candidates.html) | Card · Tag(필터 chip·status) · Icon container | — |
| S08 | [s08-feature-acceptance.html](./s08-feature-acceptance.html) | Tabs · Card · Tag | Section title (§2.2 타이포 역할) |
| S09 | [s09-feature-dependencies.html](./s09-feature-dependencies.html) | Tabs · Card · Tag | 의존성 그래프 (화면 전용 inline SVG — 디자인 시스템 컴포넌트 아님) |
| S10 | [s10-llm-edit.html](./s10-llm-edit.html) | Code block(diff add·del) · Card · Tag · Button(primary·secondary) | — |

> 이 표는 `doc-tracker.md` 검증의 입력입니다. 목업이 추가/변경되면 이 표와 위 Flows 표를 함께 갱신해야 연결 검증이 유효합니다. **§4 외 요소** 열의 항목은 §4 컴포넌트 커버리지(사용처 없는 컴포넌트 / 미정의 항목 사용) 검증에서 제외됩니다.

## Mockup vs Wireframe

- **Wireframe** ([`wireframes/`](../wireframes/)) — 정보 구조와 화면 흐름의 합의. SVG, 무채색, 디자인 톤 없음.
- **Mockup** (이 폴더) — 디자인 톤·색·폰트·인터랙션이 결정된 최종 형태. HTML, `design-system.md` 적용.

PRD가 변경되거나 화면 흐름을 재검토할 때는 와이어프레임을 먼저 갱신하고, 그 다음 이 목업을 디자인 시스템 토큰으로 다시 그립니다.

## 화면·여정을 추가할 때

**여정 페이지가 있는 여정에 단계를 추가하면** — 여정 문서에 `STP-<슬러그>` 헤딩과 터치포인트를 먼저 쓰고, 그 다음 여정 페이지에 같은 `data-step` 섹션을 더합니다. 순서를 지켜야 문서가 SSOT 로 남습니다.

**새 화면을 추가하면** — ① 같은 ID 의 와이어프레임이 먼저 있어야 합니다 (`wireframes/`) ② 기존 파일의 인라인 CSS 를 복사해 씁니다 — 임의의 색·radius·폰트를 새로 만들지 않습니다. 디자인 시스템이 바뀌면 모든 목업의 인라인 CSS 를 함께 갱신합니다 ③ 그 화면을 터치포인트로 쓰는 단계의 여정 페이지에 임베드하거나, 아직 이관 전이면 위 **이관 대기 원장**과 **화면별 매핑** 표에 행을 추가하고 상한을 올립니다 ④ 위 **Mockup index** 표에 행을 추가합니다 ⑤ [`doc-tracker.md`](../doc-tracker.md)의 연결 매트릭스와 변경 이력, 허브 [`../index.html`](../index.html)의 링크를 함께 갱신합니다.

**여정을 이관하면** — 여정 페이지를 만들고, 흡수한 화면 파일을 삭제하고, 위 **Journeys** 표·**이관 대기 원장**·상한·허브·`doc-tracker.md` 를 함께 갱신합니다. 체커가 넷의 불일치를 잡습니다.
