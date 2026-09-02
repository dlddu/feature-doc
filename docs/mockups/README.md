# Mockups

FeatureDoc 의 **목업** — 색·타이포·인터랙션·디자인 톤이 모두 결정된 최종 형태. [`design-system.md`](../design-system.md)의 토큰·컴포넌트·원칙을 그대로 표현한 결과물이에요. 같은 화면의 정보 구조 합의는 [`wireframes/`](../wireframes/)에 있습니다.

이 폴더에는 **여정 페이지**(`JRN-<슬러그>.html`)만 있습니다 — 사용자 여정 하나를 한 페이지에서 걸어보게 만든 것입니다. 한때 함께 있던 **화면 단위 파일**(`sNN-*.html`)은 아직 여정 페이지로 이관되지 않은 화면이었고, 2026-09-02 마지막 여정 이관으로 전부 흡수·삭제됐습니다. 목표 상태였던 「여정 페이지만 남는 것」에 도달했으며, 그 상태는 아래 [여정 페이지 규약](#여정-페이지-규약)의 상한 0 두 개로 붙들려 있습니다.

- **Format** — HTML, 393 × 844 (iOS 모바일 viewport), 다크 테마. 393 × 844 는 **상한**입니다 — 460px 이하 뷰포트에서는 프레임(보더·38px radius·상태바·홈 인디케이터)을 유지한 채 `min()` 으로 뷰포트에 맞춰 줄어듭니다. 실기기에서 열었을 때 가로 스크롤이 생기거나 프레임 안 스크롤과 페이지 스크롤이 겹치지 않게 하기 위한 것이고, 새 목업도 이 블록을 그대로 포함해야 합니다.
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

**⑤ 화면은 그 자체가 원본입니다** — 여정 페이지의 화면은 어딘가에서 복제해 온 스냅샷이 아닙니다. 원본 `sNN-*.html` 의 `<body>` 를 바이트 동일하게 옮기고 지문(`data-sha256`)으로 못 박는 방식은 **쓰지 않습니다** — 지문으로 화면을 고정하면 아래 ⑥(d) 의 실제 입력 요소와 ⑥(e) 의 상태 변형을 넣을 길이 구조적으로 막히기 때문입니다. 화면 단위 파일에서 이관할 때는 디자인 시스템 CSS 1벌과 시각 언어를 잇되, 본문은 **프로토타입으로 다시 씁니다**. 이관으로 흡수된 원본은 삭제되고 git 이력에 남습니다.

**⑥ 여정 페이지는 클릭되는 제품 프로토타입이어야 합니다** — 단계 메타를 늘어놓고 이전/다음으로 넘기는 문서 뷰어는 이 규약을 충족하지 않습니다.

- (a) **모든 단계 포함** — 여정 문서의 단계 집합이 페이지 안에 실재합니다.
- (b) **제품 화면이 지배면** — 문서 메타(단계 번호, 단계·여정 식별자, 터치포인트, 연결 AC, 행동 서술)를 제품 화면과 같은 평면에 상시 노출하지 않습니다. 기본으로 접힌 `<details data-meta="doc">` 보조 레이어에만 둡니다. 연 직후 보이는 것은 제품이어야 합니다. 스텝퍼도 두지 않습니다.
- (c) **화면 안의 행동으로 전진** — 모든 단계는 그 단계 화면 안의 주요 행동(`data-cta="advance"`)을 눌러 다음 단계에 도달합니다. 래퍼 네비게이션으로만 넘어가는 단계가 **하나라도** 있으면 위반입니다.
- (d) **실제 입력 요소** — 텍스트·선택·토글은 진짜 `<input>`/`<select>`/`<textarea>` 이고, 타이핑·선택이 관측 가능한 상태 변화(CTA 활성화, 검증 표시)를 일으킵니다. "입력처럼 보이는 비대화형 요소"(`.input > .val` 패턴)를 쓰지 않습니다.
- (e) **상태 변형** — 각 화면이 정상 경로 한 벌로 끝나지 않고, 여정 문서 §4 「분기·예외 흐름」의 상황에 프로토타입 안에서 실제로 도달합니다.
- (f) **딥링크** — `#STP-*` 로 그 단계에 바로 진입합니다.
- (g) **분기와 끝** — 분기를 선택할 수 있고 각 갈래의 끝(`END-*`)이 표현됩니다.
- (h) **정적 동작** — 빌드·번들·네트워크 없이 `file://` 로 열립니다. 폰트만 CDN 이며 오프라인에서는 시스템 폰트로 폴백됩니다.

**⑦ 여정 밖 분기는 이 여정의 끝입니다** — 여정 문서 §4 「분기·예외 흐름」의 「이어지는 단계」가
**다른 여정의 단계**를 가리키는 행이 있습니다. 그 갈래는 이 페이지 안에서 갈 곳이 없으므로 단계가
아니라 **갈래의 끝(handoff)** 으로 표현합니다 (수립: 2026-08-31, reconciler `rct_20260831-0003`).

- 그 끝 블록(`<section id="END-…">`)이 대상을 `data-goto-journey="JRN-<대상 여정>#STP-<대상 단계>"` 로
  **한 곳에서만** 선언합니다. 분기 목록의 버튼은 그 END 블록을 `data-goto` 로 가리킬 뿐이고,
  대상 식별자를 다시 적지 않습니다 — 두 곳에 적으면 조용히 어긋납니다.
- 대상 여정·단계가 실재하는지는 **여정 문서**에서 확인합니다(페이지가 자기 선언을 자기가 검증하면 공허합니다).
- 그 끝 블록은 실제로 넘어갈 수 있어야 합니다. 대상 여정에 페이지가 있으면 그 **단계 앵커**
  (`./JRN-<대상>.html#STP-<단계>`)로, 아직 없으면 여정 문서(`../reader.html?doc=user-journey/JRN-<대상>.md`)로
  링크합니다. 어느 쪽이 맞는지는 체커가 현재 상태를 보고 판정하므로, 대상 여정이 나중에 이관되면
  낡은 링크가 그대로 남지 않고 CI 에서 드러납니다.
- 그래서 **이관 순서에는 제약이 하나 있었습니다** — 대상 여정이 먼저 이관돼 있으면 강한 형태(단계 앵커)로
  바로 설 수 있습니다. 여정 밖 분기의 사슬은 `understand-feature → follow-code-change →
  review-feature → discover-features` 였고, `review-feature` 부터 역순으로 이관해 매번
  대상 페이지가 이미 존재하게 했습니다(2026-08-31 ~ 2026-09-02, 슬라이스 3~5). **사슬은 2026-09-02 에
  끝났고 판정 대상 5개가 모두 강한 형태로 서 있습니다** — 이 제약은 이제 새 여정이 추가될 때만 다시 걸립니다.

(c)(d)(e) 는 **정적 대조로 확인할 수 없습니다** — 파일을 읽어 속성만 세면 배선이 끊긴 버튼과 살아 있는 버튼이 구분되지 않습니다. 그래서 DOM 하네스([`tools/check-journey-prototype.js`](../../tools/check-journey-prototype.js), jsdom)가 실제 DOM 에서 굴려 집행하고 CI 게이트에 걸려 있습니다. **하네스가 없거나 CI 에 걸려 있지 않은 상태는 그 자체가 위반입니다.** 하네스의 기대값(단계 순서·분기 대상)은 페이지가 아니라 **여정 문서에서 파싱**합니다 — 페이지가 선언한 값으로 이동하는지를 그 페이지에서 읽어 단언하면 자기참조라 무엇을 바꿔도 통과합니다.

이 규약의 기계 검사는 [`tools/check-journey-mockup.py`](../../tools/check-journey-mockup.py)(정적 R0~R10)와 [`tools/check-journey-prototype.js`](../../tools/check-journey-prototype.js)(DOM P1~P7)가 [`docs — journey ↔ mockup`](../../.github/workflows/docs-journey-mockup.yml) 게이트에서 함께 수행합니다.

## 예시값 표기 규약 (`data-sample`)

목업이 그리는 문자열에는 성격이 다른 두 가지가 섞여 있습니다.

- **제품 카피** — 화면이 언제나 그대로 렌더하는 말. 제목·라벨·안내문·버튼 문구·빈 상태 문장.
  구현이 이것과 다르면 그것이 곧 정합성 이탈입니다.
- **예시값** — *「이 자리에 이런 종류의 값이 온다」* 를 보여 주려고 적어 둔 표본. 분석 결과
  항목명, 근거 파일 경로, 저장소 이름처럼 **실제로는 서버가 준 값이 렌더되는** 자리입니다.
  구현이 목업과 같은 문자열을 그리면 오히려 **거짓 화면**이 됩니다.

리터럴 카피 대조([`tools/check-mockup-render.py`](../../tools/check-mockup-render.py) 의 M3A·M3B)는
둘을 구분하지 못합니다. 그래서 **예시값을 담은 요소에 `data-sample` 을 붙여** 대조에서 뺍니다.

```html
<div class="ev">
  <span class="ename" data-sample>PostgreSQL 15 · 단일 인스턴스</span>
  <span class="esrc" data-sample>docker-compose.yml:12</span>
</div>
```

**규칙 세 가지.**

1. **값을 담은 잎(leaf) 요소에만 붙입니다.** 위 예에서 행(`div.ev`) 에 붙이면 같은 행에 있는
   제품 카피까지 함께 사라집니다 — 실제로 `근거 없음` 태그가 그렇게 묻힙니다. 숨김은 언제나
   최소 범위여야 합니다.
2. **제품 카피에는 붙일 수 없습니다.** 특히 상호작용 요소(`<a>`·`<button>`, 또는
   `data-goto`·`data-cta`·`id`·`href` 를 가진 요소)에는 금지입니다. CTA 문구를 예시값으로
   위장해 대조를 빠져나가는 길을 막기 위한 것이고, 이 금지는 게이트 규칙 **M6** 이 집행합니다.
3. **숫자·단위만으로 이루어진 표본에는 필요 없습니다.** `847 · 2.3 MB` · `~6 min` 같은 것은
   체커의 `is_copy()` 가 이미 카피로 보지 않습니다. `data-sample` 은 **글자로 된 예시값**
   (제품명·경로·문장)을 위한 장치입니다.

> 이 속성은 렌더링에 영향을 주지 않습니다(스타일 훅이 아닙니다). 목업을 `file://` 로 열었을 때
> 보이는 화면은 붙이기 전과 완전히 같습니다.

## Journeys — 여정 ↔ 목업 페이지 매핑

여정 문서에서 파싱한 값입니다. 「예외」는 [`doc-tracker.md`](../doc-tracker.md) 「수용된 위험」에 여정 단위로 등재되어 목업 페이지가 없어도 되는 여정입니다.

<!-- jmap:begin -->
| 여정 | 여정 문서 | 목업 페이지 | 단계 | 상태 |
| --- | --- | --- | --- | --- |
| `JRN-connect-repo` 저장소를 맡기고 첫 분석을 걸기 | [`JRN-connect-repo.md`](../user-journey/JRN-connect-repo.md) | [`JRN-connect-repo.html`](./JRN-connect-repo.html) | 5 | ✅ 이관 완료 |
| `JRN-discover-features` 코드에서 기능 목록 뽑아내기 | [`JRN-discover-features.md`](../user-journey/JRN-discover-features.md) | [`JRN-discover-features.html`](./JRN-discover-features.html) | 5 | ✅ 이관 완료 |
| `JRN-follow-code-change` 코드가 바뀐 뒤 문서가 따라왔는지 확인하기 | [`JRN-follow-code-change.md`](../user-journey/JRN-follow-code-change.md) | [`JRN-follow-code-change.html`](./JRN-follow-code-change.html) | 4 | ✅ 이관 완료 |
| `JRN-restore-history` 잘못된 변경을 되짚어 되돌리기 | [`JRN-restore-history.md`](../user-journey/JRN-restore-history.md) | — | 3 | 예외 (수용된 위험) |
| `JRN-review-feature` 기능 하나의 표현이 맞는지 검수하기 | [`JRN-review-feature.md`](../user-journey/JRN-review-feature.md) | [`JRN-review-feature.html`](./JRN-review-feature.html) | 5 | ✅ 이관 완료 |
| `JRN-understand-feature` 코드를 못 읽는 사람이 기능을 이해하기 | [`JRN-understand-feature.md`](../user-journey/JRN-understand-feature.md) | [`JRN-understand-feature.html`](./JRN-understand-feature.html) | 4 | ✅ 이관 완료 |
<!-- jmap:end -->

집계: 여정 **6**개 · 규칙 8 예외 **1**개 · 판정 대상 **5**개 · 이관 완료 **5**개 · 이관 대기 **0**개. **이관 대기 상한: 0** — 이관 대기 여정은 0개를 넘을 수 없습니다. 판정 대상 5개가 모두 이관돼 상한이 바닥에 닿았고, 앞으로 새 여정이 생기면 그 여정은 **이관 대기 상태로 존재할 수 없습니다** — 여정 문서와 같은 PR 에서 페이지까지 함께 만들어야 합니다(면제가 아니라 래칫이라, 되돌리면 CI 가 실패합니다).

### 이관 완료 — `JRN-connect-repo`

단계와 화면은 여정 문서에서 파싱한 것입니다. 화면 열은 그 단계의 터치포인트 줄에 등장하는 화면 ID 이며, 페이지의 `data-screens` 선언과 대조됩니다(R6).

<!-- steps:JRN-connect-repo:begin -->
| # | 단계 | 화면 | 공개 경로 |
| --- | --- | --- | --- |
| 1 | `STP-sign-in` 본인 확인하고 들어오기 | — (시각화 없음 · 수용된 위험) | [#STP-sign-in](./JRN-connect-repo.html#STP-sign-in) |
| 2 | `STP-grant-repo-access` 들여다볼 범위를 내가 정하기 | S01 | [#STP-grant-repo-access](./JRN-connect-repo.html#STP-grant-repo-access) |
| 3 | `STP-register-llm-key` 분석 비용을 낼 키 맡기기 | S01 | [#STP-register-llm-key](./JRN-connect-repo.html#STP-register-llm-key) |
| 4 | `STP-pick-target` 분석할 저장소와 브랜치 고르기 | S02 · S03 | [#STP-pick-target](./JRN-connect-repo.html#STP-pick-target) |
| 5 | `STP-confirm-cost` 비용을 확인하고 시작 누르기 | S03 | [#STP-confirm-cost](./JRN-connect-repo.html#STP-confirm-cost) |
<!-- steps:JRN-connect-repo:end -->

### 이관 완료 — `JRN-discover-features`

<!-- steps:JRN-discover-features:begin -->
| # | 단계 | 화면 | 공개 경로 |
| --- | --- | --- | --- |
| 1 | `STP-leave-and-return` 맡겨두고 떠났다가 돌아오기 | S04 | [#STP-leave-and-return](./JRN-discover-features.html#STP-leave-and-return) |
| 2 | `STP-review-landscape` 내 코드가 무엇 위에 서 있는지 확인하기 | S05 | [#STP-review-landscape](./JRN-discover-features.html#STP-review-landscape) |
| 3 | `STP-tune-strategy` 내가 아는 코드 구조를 전략에 보태기 | S06 | [#STP-tune-strategy](./JRN-discover-features.html#STP-tune-strategy) |
| 4 | `STP-sift-candidates` 진짜 기능과 아닌 것 가르기 | S07 | [#STP-sift-candidates](./JRN-discover-features.html#STP-sift-candidates) |
| 5 | `STP-add-missing` 빠진 기능 직접 추가하기 | S07 | [#STP-add-missing](./JRN-discover-features.html#STP-add-missing) |
<!-- steps:JRN-discover-features:end -->

### 이관 완료 — `JRN-follow-code-change`

<!-- steps:JRN-follow-code-change:begin -->
| # | 단계 | 화면 | 공개 경로 |
| --- | --- | --- | --- |
| 1 | `STP-notice-change` 무엇이 바뀌었는지 알기 | S04 | [#STP-notice-change](./JRN-follow-code-change.html#STP-notice-change) |
| 2 | `STP-scan-diff` 어느 기능의 무엇이 달라졌는지 훑기 | S07 · S08 · S09 | [#STP-scan-diff](./JRN-follow-code-change.html#STP-scan-diff) |
| 3 | `STP-resolve-conflict` 내 편집과 자동 결과 중 무엇을 살릴지 정하기 | S08 · S10 | [#STP-resolve-conflict](./JRN-follow-code-change.html#STP-resolve-conflict) |
| 4 | `STP-recheck-candidates` 후보 목록의 변화 확인하기 | S07 | [#STP-recheck-candidates](./JRN-follow-code-change.html#STP-recheck-candidates) |
<!-- steps:JRN-follow-code-change:end -->

### 이관 완료 — `JRN-review-feature`

<!-- steps:JRN-review-feature:begin -->
| # | 단계 | 화면 | 공개 경로 |
| --- | --- | --- | --- |
| 1 | `STP-read-scenarios` 사용자에게 무슨 일이 벌어지는지 읽기 | S08 | [#STP-read-scenarios](./JRN-review-feature.html#STP-read-scenarios) |
| 2 | `STP-verify-evidence` 근거 코드가 진짜 출처인지 확인하기 | S08 | [#STP-verify-evidence](./JRN-review-feature.html#STP-verify-evidence) |
| 3 | `STP-trace-dependencies` 이 기능이 무엇에 기대고 있는지 보기 | S09 | [#STP-trace-dependencies](./JRN-review-feature.html#STP-trace-dependencies) |
| 4 | `STP-request-edit` 어색한 표현을 자연어로 고쳐달라 하기 | S10 | [#STP-request-edit](./JRN-review-feature.html#STP-request-edit) |
| 5 | `STP-decide-diff` 제안된 변경을 승인하거나 버리기 | S10 | [#STP-decide-diff](./JRN-review-feature.html#STP-decide-diff) |
<!-- steps:JRN-review-feature:end -->

### 이관 완료 — `JRN-understand-feature`

<!-- steps:JRN-understand-feature:begin -->
| # | 단계 | 화면 | 공개 경로 |
| --- | --- | --- | --- |
| 1 | `STP-open-shared` 공유받은 기능 열기 | S02 | [#STP-open-shared](./JRN-understand-feature.html#STP-open-shared) |
| 2 | `STP-grasp-behavior` 위에서 아래로 읽어 이해하기 | S08 | [#STP-grasp-behavior](./JRN-understand-feature.html#STP-grasp-behavior) |
| 3 | `STP-check-scope` 이 기능이 무엇에 얽혀 있는지 훑기 | S09 | [#STP-check-scope](./JRN-understand-feature.html#STP-check-scope) |
| 4 | `STP-flag-ambiguity` 이상한 부분을 자연어로 넘기기 | S10 | [#STP-flag-ambiguity](./JRN-understand-feature.html#STP-flag-ambiguity) |
<!-- steps:JRN-understand-feature:end -->

## 화면 단위 잔여 — 이관 대기 원장

아직 여정 페이지로 이관되지 않은 화면 단위 파일입니다. **면제가 아니라 래칫입니다** — 체커는 (a) 이 원장에 없는 새 미선언 파일 (b) 이미 이관됐는데 원장에 남아 있는 공전 행 (c) 아래 상한 초과를 전부 실패로 만듭니다.

**원장은 2026-09-02 에 비었습니다.** 판정 대상 여정 5개가 모두 여정 페이지를 가지면서 `S01`~`S10` 전부의 마지막 소비 여정이 이관됐고, 화면 단위 파일은 하나도 남지 않았습니다. 아래 표가 빈 것과 상한이 0 인 것은 같은 사실의 두 표현입니다.

**화면 공유는 이관을 막지 않았습니다 — 원본 삭제만 미뤘습니다.** 공유가 실제로 가로막은 것은 *그 여정의 이관*이 아니라 *원본 `sNN` 파일의 삭제*뿐입니다 — 위 규약 ⑤ 대로 여정 페이지의 화면은 복제본이 아니라 그 자체가 원본이고, 「같은 화면이 여러 여정에 등장하는 것은 중복이 아니다」(여정마다 그 화면의 데이터와 다음 행동이 다르므로 각 여정 페이지에 그 여정의 맥락으로 각각 존재하는 것이 정상)가 이미 규약이기 때문입니다.

그래서 규칙은 이랬습니다 (확정: 2026-08-31, reconciler `rct_20260831-0001`). 원장이 빈 지금도 규칙 자체는 살아 있습니다 — 새 화면 단위 파일이 생기면 다시 여기에 등재하고 상한을 올려야 하며, 그 순간 CI 가 그 사실을 드러냅니다.

- **이관은 여정 단위로 진행합니다.** 공유 화면을 쓰는 여정도 다른 여정을 기다리지 않고, 그 화면을 자기 맥락으로 새로 써서 이관합니다.
- **원본 `sNN` 삭제는 마지막 소비 여정이 이관될 때 합니다.** 그때까지 원본은 아래 원장에 남고, 「쓰는 여정」 칸에서 이미 이관된 여정을 지워 남은 소비자가 몇인지 드러냅니다.

`S01`~`S03` 은 `JRN-connect-repo` 전용이라 그 여정을 이관하면서 함께 지웠고, `S05`·`S06` 은 `JRN-discover-features` 전용이라 같은 방식으로 지웠습니다. `JRN-review-feature` 이관(2026-08-31)으로는 **한 파일도 지우지 못했습니다** — `S08`~`S10` 을 다른 두 여정이 아직 쓰고 있었기 때문이고, 「쓰는 여정」 칸에서 소비자가 셋에서 둘로 줄어든 것이 그때의 진척이었습니다. `JRN-follow-code-change` 이관(2026-09-01)으로 `S04`·`S07` 의 마지막 소비자가 사라져 **두 파일을 지웠고**, `JRN-understand-feature` 이관(2026-09-02)으로 `S08`~`S10` 의 마지막 소비자까지 사라져 **남은 세 파일을 지웠습니다**.

**「쓰는 여정」 칸은 손으로 적는 값이 아닙니다** — 그 화면을 터치포인트로 쓰는 여정(여정 문서에서 파싱) 중 아직 이관되지 않고 규칙 8 예외도 아닌 것들이며, 체커가 그 파생을 실측과 대조합니다. 이 칸이 비면(= 소비자 0) 그 파일은 흡수·삭제할 차례이고, 남겨 두면 CI 가 실패합니다.

<!-- ledger:begin -->
| 파일 | 화면 | 쓰는 여정 | 해소 조건 |
| --- | --- | --- | --- |
<!-- ledger:end -->

**상한: 0** — 화면 단위 잔여 파일은 0개를 넘을 수 없습니다. 이관이 끝날 때마다 이 숫자를 함께 내렸고, 이제 바닥입니다. 새 화면을 화면 단위 파일로 추가하려면 이 숫자를 먼저 올려야 하며, 그 변경이 곧 「왜 여정 페이지가 아닌가」를 설명해야 하는 자리입니다.

## Mockup index — 디자인 시스템 사용 매핑

각 목업이 [`design-system.md`](../design-system.md)의 어떤 항목을 쓰는지. 모든 화면은 인라인 CSS를 통해 §1 Foundations(토큰)와 §6 Principles(원칙)를 공통으로 따르므로, 아래 **§4 컴포넌트** 열은 화면별로 두드러지게 쓰인 §4 Components 항목만 적습니다. §4 컴포넌트가 아닌 요소(§2 타이포 역할이나 화면 전용 일회성 요소)는 **§4 외 요소** 열에 따로 둡니다 — 이쪽은 디자인 시스템 §4 컴포넌트 커버리지 검증 대상이 아닙니다.

| ID  | 공개 경로 | 사용 §4 컴포넌트 | §4 외 요소 |
| --- | --------- | ---------------- | ---------- |
| S01 | [JRN-connect-repo #STP-grant-repo-access](./JRN-connect-repo.html#STP-grant-repo-access) | Input field · Button(primary·secondary) · Tag · Card · Icon container · Segment selector | — |
| S02 | [JRN-connect-repo #STP-pick-target](./JRN-connect-repo.html#STP-pick-target) · [JRN-understand-feature #STP-open-shared](./JRN-understand-feature.html#STP-open-shared) | Card · Tag(status badge) · Bottom tab bar · Metric grid · Progress bar | Section title (§2.2 타이포 역할) |
| S03 | [JRN-connect-repo #STP-confirm-cost](./JRN-connect-repo.html#STP-confirm-cost) | Input field · Card · Button(primary·ghost) · Tag(status badge) | — |
| S04 | [JRN-discover-features #STP-leave-and-return](./JRN-discover-features.html#STP-leave-and-return) · [JRN-follow-code-change #STP-notice-change](./JRN-follow-code-change.html#STP-notice-change) | Step(done·active·todo — active의 회전 ring 포함) · Card · Button(secondary) · Metric grid · Progress bar | Section title (§2.2 타이포 역할) |
| S05 | [JRN-discover-features #STP-review-landscape](./JRN-discover-features.html#STP-review-landscape) | Card · Tag(status badge) | Section title (§2.2 타이포 역할) · 근거 줄 (화면 전용) |
| S06 | [JRN-discover-features #STP-tune-strategy](./JRN-discover-features.html#STP-tune-strategy) | Input field · Card · Button(primary·secondary·ghost) | 전략 항목 줄 (화면 전용) |
| S07 | [JRN-discover-features #STP-sift-candidates](./JRN-discover-features.html#STP-sift-candidates) · [JRN-follow-code-change #STP-scan-diff](./JRN-follow-code-change.html#STP-scan-diff) | Input field · Card · Tag(status) · Button(primary·secondary·ghost) · Code block(kw) | 후보 카드 (화면 전용) |
| S08 | [JRN-review-feature #STP-read-scenarios](./JRN-review-feature.html#STP-read-scenarios) · [JRN-follow-code-change #STP-scan-diff](./JRN-follow-code-change.html#STP-scan-diff) · [JRN-understand-feature #STP-grasp-behavior](./JRN-understand-feature.html#STP-grasp-behavior) | Card · Tag · Button(primary·secondary·ghost) · Input field · Code block | Section title (§2.2 타이포 역할) · 시나리오 카드 (화면 전용) |
| S09 | [JRN-review-feature #STP-trace-dependencies](./JRN-review-feature.html#STP-trace-dependencies) · [JRN-understand-feature #STP-check-scope](./JRN-understand-feature.html#STP-check-scope) | Card · Tag · Input field · Button(primary) | 의존성 그래프 (화면 전용 inline SVG — 디자인 시스템 컴포넌트 아님) · 의존성 줄 (화면 전용) |
| S10 | [JRN-review-feature #STP-request-edit](./JRN-review-feature.html#STP-request-edit) · [JRN-understand-feature #STP-flag-ambiguity](./JRN-understand-feature.html#STP-flag-ambiguity) | Code block(diff add·del) · Card · Tag · Button(primary·secondary) · Input field | — |

> 이 표는 `doc-tracker.md` 검증의 입력입니다. 목업이 추가/변경되면 이 표와 위 Flows 표를 함께 갱신해야 연결 검증이 유효합니다. **§4 외 요소** 열의 항목은 §4 컴포넌트 커버리지(사용처 없는 컴포넌트 / 미정의 항목 사용) 검증에서 제외됩니다.

## Mockup vs Wireframe

- **Wireframe** ([`wireframes/`](../wireframes/)) — 정보 구조와 화면 흐름의 합의. SVG, 무채색, 디자인 톤 없음.
- **Mockup** (이 폴더) — 디자인 톤·색·폰트·인터랙션이 결정된 최종 형태. HTML, `design-system.md` 적용.

PRD가 변경되거나 화면 흐름을 재검토할 때는 와이어프레임을 먼저 갱신하고, 그 다음 이 목업을 디자인 시스템 토큰으로 다시 그립니다.

## 화면·여정을 추가할 때

**여정 페이지가 있는 여정에 단계를 추가하면** — 여정 문서에 `STP-<슬러그>` 헤딩과 터치포인트를 먼저 쓰고, 그 다음 여정 페이지에 같은 `data-step` 섹션을 더합니다. 순서를 지켜야 문서가 SSOT 로 남습니다.

**새 화면을 추가하면** — ① 같은 ID 의 와이어프레임이 먼저 있어야 합니다 (`wireframes/`) ② 기존 파일의 인라인 CSS 를 복사해 씁니다 — 임의의 색·radius·폰트를 새로 만들지 않습니다. 디자인 시스템이 바뀌면 모든 목업의 인라인 CSS 를 함께 갱신합니다 ③ 그 화면을 터치포인트로 쓰는 단계의 여정 페이지에 임베드하거나, 아직 이관 전이면 위 **이관 대기 원장**과 **화면별 매핑** 표에 행을 추가하고 상한을 올립니다 ④ 위 **Mockup index** 표에 행을 추가합니다 ⑤ [`doc-tracker.md`](../doc-tracker.md)의 연결 매트릭스와 변경 이력, 허브 [`../index.html`](../index.html)의 링크를 함께 갱신합니다.

**여정을 이관하면** — 여정 페이지를 만들고, 흡수한 화면 파일을 삭제하고, 위 **Journeys** 표·**이관 대기 원장**·상한·허브·`doc-tracker.md` 를 함께 갱신합니다. 체커가 넷의 불일치를 잡습니다.
