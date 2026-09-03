# Mockups

FeatureDoc 의 **목업** — 색·타이포·인터랙션·디자인 톤이 모두 결정된 최종 형태. [`design-system.md`](../design-system.md)의 토큰·컴포넌트·원칙을 그대로 표현한 결과물이에요. 각 화면이 어떤 흐름의 어느 자리에 놓이는지는 [`user-journey/`](../user-journey/)의 여정 문서가 정합니다.

이 폴더에는 **여정 페이지**(`JRN-<슬러그>.html`)만 있습니다 — 사용자 여정 하나를 한 페이지에서 걸어보게 만든 것입니다. 한때 함께 있던 **화면 단위 파일**(`sNN-*.html`)은 아직 여정 페이지로 이관되지 않은 화면이었고, 2026-09-02 마지막 여정 이관으로 전부 흡수·삭제됐습니다. 목표 상태였던 「여정 페이지만 남는 것」에 도달했으며, 그 상태는 아래 [여정 페이지 규약](#여정-페이지-규약)의 상한 0 두 개로 붙들려 있습니다.

- **Format** — HTML, 393 × 844 (iOS 모바일 viewport), 다크 테마. 393 × 844 는 **상한**입니다 — 460px 이하 뷰포트에서는 프레임(보더·38px radius·상태바·홈 인디케이터)을 유지한 채 `min()` 으로 뷰포트에 맞춰 줄어듭니다. 실기기에서 열었을 때 가로 스크롤이 생기거나 프레임 안 스크롤과 페이지 스크롤이 겹치지 않게 하기 위한 것이고, 새 목업도 이 블록을 그대로 포함해야 합니다.
- **Style** — `design-system.md` v0.1 그대로: 5단계 표면, 1px hairline, shadow·gradient 없음, 액센트 최소
- **Self-contained** — 각 파일은 디자인 시스템을 인코딩한 CSS 를 파일 안에 인라인으로 담은 단독 파일. 의존성 없이 브라우저로 바로 열면 그대로 렌더링됩니다.
- **Fonts** — Geist (본문) · JetBrains Mono (메타·코드), Google Fonts CDN. 오프라인일 때는 시스템 sans/mono 로 폴백됩니다.

## 여정 페이지 규약

여정 하나를 한 페이지에서 걸어보게 만드는 **여정 목업**의 규약입니다. 이 절이 여정 ↔ 목업 페이지 매핑의 단일 소스이고, 이관 슬라이스는 여기 정해진 경로·속성·추출 방식을 그대로 씁니다 (수립: 2026-08-30, reconciler `rct_20260830-0001`).

**① 경로와 파일명** — 여정 페이지는 `docs/mockups/JRN-<슬러그>.html` 1개 = 여정 1개. 흡수된 화면 단위 파일은 삭제합니다(내용은 여정 페이지 안에 바이트 동일하게 보존되고 원본은 git 이력에 남습니다).

> `docs/journeys/` 같은 별도 디렉터리에 두지 않는 이유가 있습니다. 이 산출물의 변경을 감지하는 쪽이 `docs/mockups` 트리 해시와 `docs/index.html` 을 보기 때문에, 그 밖에 두면 **여정 페이지를 아무리 고쳐도 변경이 감지되지 않습니다**. 목업 파일은 목업 폴더 안에 둡니다.

**② 선언 속성** — 여정 페이지는 루트 `<body>` 에 `data-journey="JRN-<슬러그>"` 를 **정확히 1개** 선언하고, 각 단계 섹션이 `id` 와 같은 값의 `data-step="STP-<슬러그>"` 를 선언합니다. `END-*` id 를 가진 블록은 갈래의 끝이며 **단계가 아닙니다**(`data-step` 을 달지 않습니다).

**③ 식별자는 여기서 정의하지 않습니다** — 여정 `JRN-*` 와 단계 `STP-*` 의 원천은 [`user-journey/`](../user-journey/) 의 여정 문서이고(각 문서의 `### \`STP-…\`` 헤딩), 이 README 와 여정 페이지는 그 값을 **인용**할 뿐입니다. 식별자 표를 여기에 따로 두면 SSOT 가 둘이 되어 조용히 어긋납니다. 기존 식별자는 바꾸지 않고, 단계가 사라져도 재사용하지 않습니다.

**④ 화면은 이름을 갖지 않습니다** — 한 단계가 무엇을 보여줄지는 그 단계의 `- **터치포인트**:` 줄이 산문으로 적습니다. 화면에 `S01` 같은 식별자를 붙이던 층은 2026-09-03 에 폐지했습니다(이유는 아래 [화면 ID 를 폐지한 이유](#화면-id-를-폐지한-이유)). 화면을 가리키는 유일한 주소는 **단계 앵커**(`JRN-<슬러그>.html#STP-<슬러그>`)이며, 같은 UI 가 여러 여정에 등장하면 그 여정 수만큼 앵커를 갖습니다 — 그것이 중복이 아니라 정상입니다.

**⑤ 화면은 그 자체가 원본입니다** — 여정 페이지의 화면은 어딘가에서 복제해 온 스냅샷이 아닙니다. 다른 파일의 `<body>` 를 바이트 동일하게 옮기고 지문(`data-sha256`)으로 못 박는 방식은 **쓰지 않습니다** — 지문으로 화면을 고정하면 아래 ⑥(d) 의 실제 입력 요소와 ⑥(e) 의 상태 변형을 넣을 길이 구조적으로 막히기 때문입니다. 화면을 옮겨올 때는 디자인 시스템 CSS 1벌과 시각 언어를 잇되, 본문은 **프로토타입으로 다시 씁니다**.

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

## 제3자 소유 화면 표기 규약 (`data-owner`)

여정에는 **우리가 그리지 않는 화면**이 섞입니다. `JRN-connect-repo` 의 `STP-grant-repo-access`
(GitHub App 설치 동의)가 그렇습니다 — 범위 선택·저장소 체크·`Install & Authorize` 는 github.com 이
소유·렌더링하고, 권한 부여가 GitHub 도메인에서 일어나는 것은 우회할 대상이 아니라 보안 속성입니다.

여정 프로토타입은 흐름이 끊기지 않게 그 화면까지 그립니다(규약 ⑥(a)). 그러나 **제품 화면과 같은
모양으로 그리면 거짓**이 됩니다 — 목업이 SSOT 이므로, 그 모양은 "이것도 우리가 만들 화면"이라는
지시로 읽힙니다. 실제로 구현이 그 화면을 렌더링하면 그것이야말로 사실과 다릅니다
([`doc-tracker.md`](../doc-tracker.md) 「알려진 목업↔구현 편차」의 유형 **「제3자 소유」**).

그래서 소유자가 우리가 아닌 화면은 **둘 다** 합니다.

1. **선언** — 그 단계 `<section>` 에 `data-owner="<호스트>"` 를 붙입니다(예: `data-owner="github.com"`).
   기계 판독용이며, 소유자를 산문에만 적어 두면 조용히 낡습니다.
2. **표시** — 제품의 `.appbar` 대신 **브라우저 크롬(`.extbar`)** 을 씁니다. 잠금 아이콘과 호스트명을
   띄우고, 왼쪽은 닫기(`✕`)로 취소 갈래에 연결합니다. 바로 아래 `.ext-note` 한 줄로 누가 이 화면을
   소유하는지, 우리가 무엇을 하는지(넘겨주고 돌아온다)를 말로도 적습니다.

> 목업 카피 대조(M3)는 이 단계를 보지 않습니다 — 대응 구현 화면이 **없는** 것이 정상이기 때문입니다.
> 그래서 표기가 없으면 어떤 게이트도 이 자리를 지적하지 않고, 화면의 모양만 남아 잘못된 지시가 됩니다.

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

단계 순서와 식별자는 여정 문서에서 파싱한 것입니다(R9).

<!-- steps:JRN-connect-repo:begin -->
| # | 단계 | 공개 경로 |
| --- | --- | --- |
| 1 | `STP-sign-in` 본인 확인하고 들어오기 | [#STP-sign-in](./JRN-connect-repo.html#STP-sign-in) |
| 2 | `STP-grant-repo-access` 들여다볼 범위를 내가 정하기 | [#STP-grant-repo-access](./JRN-connect-repo.html#STP-grant-repo-access) |
| 3 | `STP-register-llm-key` 분석 비용을 낼 키 맡기기 | [#STP-register-llm-key](./JRN-connect-repo.html#STP-register-llm-key) |
| 4 | `STP-pick-target` 분석할 저장소와 브랜치 고르기 | [#STP-pick-target](./JRN-connect-repo.html#STP-pick-target) |
| 5 | `STP-confirm-cost` 비용을 확인하고 시작 누르기 | [#STP-confirm-cost](./JRN-connect-repo.html#STP-confirm-cost) |
<!-- steps:JRN-connect-repo:end -->

### 이관 완료 — `JRN-discover-features`

<!-- steps:JRN-discover-features:begin -->
| # | 단계 | 공개 경로 |
| --- | --- | --- |
| 1 | `STP-leave-and-return` 맡겨두고 떠났다가 돌아오기 | [#STP-leave-and-return](./JRN-discover-features.html#STP-leave-and-return) |
| 2 | `STP-review-landscape` 내 코드가 무엇 위에 서 있는지 확인하기 | [#STP-review-landscape](./JRN-discover-features.html#STP-review-landscape) |
| 3 | `STP-tune-strategy` 내가 아는 코드 구조를 전략에 보태기 | [#STP-tune-strategy](./JRN-discover-features.html#STP-tune-strategy) |
| 4 | `STP-sift-candidates` 진짜 기능과 아닌 것 가르기 | [#STP-sift-candidates](./JRN-discover-features.html#STP-sift-candidates) |
| 5 | `STP-add-missing` 빠진 기능 직접 추가하기 | [#STP-add-missing](./JRN-discover-features.html#STP-add-missing) |
<!-- steps:JRN-discover-features:end -->

### 이관 완료 — `JRN-follow-code-change`

<!-- steps:JRN-follow-code-change:begin -->
| # | 단계 | 공개 경로 |
| --- | --- | --- |
| 1 | `STP-notice-change` 무엇이 바뀌었는지 알기 | [#STP-notice-change](./JRN-follow-code-change.html#STP-notice-change) |
| 2 | `STP-scan-diff` 어느 기능의 무엇이 달라졌는지 훑기 | [#STP-scan-diff](./JRN-follow-code-change.html#STP-scan-diff) |
| 3 | `STP-resolve-conflict` 내 편집과 자동 결과 중 무엇을 살릴지 정하기 | [#STP-resolve-conflict](./JRN-follow-code-change.html#STP-resolve-conflict) |
| 4 | `STP-recheck-candidates` 후보 목록의 변화 확인하기 | [#STP-recheck-candidates](./JRN-follow-code-change.html#STP-recheck-candidates) |
<!-- steps:JRN-follow-code-change:end -->

### 이관 완료 — `JRN-review-feature`

<!-- steps:JRN-review-feature:begin -->
| # | 단계 | 공개 경로 |
| --- | --- | --- |
| 1 | `STP-read-scenarios` 사용자에게 무슨 일이 벌어지는지 읽기 | [#STP-read-scenarios](./JRN-review-feature.html#STP-read-scenarios) |
| 2 | `STP-verify-evidence` 근거 코드가 진짜 출처인지 확인하기 | [#STP-verify-evidence](./JRN-review-feature.html#STP-verify-evidence) |
| 3 | `STP-trace-dependencies` 이 기능이 무엇에 기대고 있는지 보기 | [#STP-trace-dependencies](./JRN-review-feature.html#STP-trace-dependencies) |
| 4 | `STP-request-edit` 어색한 표현을 자연어로 고쳐달라 하기 | [#STP-request-edit](./JRN-review-feature.html#STP-request-edit) |
| 5 | `STP-decide-diff` 제안된 변경을 승인하거나 버리기 | [#STP-decide-diff](./JRN-review-feature.html#STP-decide-diff) |
<!-- steps:JRN-review-feature:end -->

### 이관 완료 — `JRN-understand-feature`

<!-- steps:JRN-understand-feature:begin -->
| # | 단계 | 공개 경로 |
| --- | --- | --- |
| 1 | `STP-open-shared` 공유받은 기능 열기 | [#STP-open-shared](./JRN-understand-feature.html#STP-open-shared) |
| 2 | `STP-grasp-behavior` 위에서 아래로 읽어 이해하기 | [#STP-grasp-behavior](./JRN-understand-feature.html#STP-grasp-behavior) |
| 3 | `STP-check-scope` 이 기능이 무엇에 얽혀 있는지 훑기 | [#STP-check-scope](./JRN-understand-feature.html#STP-check-scope) |
| 4 | `STP-flag-ambiguity` 이상한 부분을 자연어로 넘기기 | [#STP-flag-ambiguity](./JRN-understand-feature.html#STP-flag-ambiguity) |
<!-- steps:JRN-understand-feature:end -->

## 화면 ID 를 폐지한 이유

`S01`~`S10` 이라는 화면 식별자는 와이어프레임 시절의 유산이었습니다. 그때는 화면 하나가 파일 하나(`s01-*.svg`)였고, ID 는 그 파일의 이름이었습니다. 2026-09-02 여정 이관이 끝나면서 그 대응이 끊겼습니다 — 화면은 더 이상 자기 파일을 갖지 않고 여정 페이지 안의 단계가 됐고, ID 는 **아무 산출물도 가리키지 않는 이름**으로 남았습니다.

남겨 둘 수도 있었고 그 편이 짧게 쓰이긴 했습니다. 화면 10개 중 6개를 여정 2~3개가 공유하므로, ID 하나가 단계 앵커 여러 개를 대신해 주기 때문입니다. 폐지를 택한 근거는 **두 이름이 서로를 검증하지 못한다**는 쪽이었습니다.

- ID 의 정의를 담은 곳이 사라졌습니다. 이름·목적·AC 를 적어 두던 표는 와이어프레임 README 였고, 그 파일과 함께 없어졌습니다.
- ID 는 기계가 붙들지 못합니다. 단계 앵커는 `id` 속성으로 실재해 링크가 깨지면 체커가 잡지만, `S08` 이 무엇인지는 산문만 압니다.
- 그래서 같은 화면을 두 이름으로 부르는 상태가 됐고, 둘이 어긋나도 아무 게이트도 울리지 않았습니다.

**지금 화면을 가리키는 유일한 주소는 단계 앵커입니다.** 같은 UI 가 여러 여정에 등장하면 앵커도 그 수만큼 있고, 그것이 중복이 아니라 정상입니다 — 같은 화면이라도 여정마다 데이터와 다음 행동이 다르기 때문입니다(규약 ⑤).

## 여정 페이지 ↔ 디자인 시스템 §4 사용 매핑

각 여정 페이지가 [`design-system.md`](../design-system.md)의 어떤 §4 컴포넌트를 쓰는지. 모든 페이지는 인라인 CSS 로 §1 Foundations(토큰)와 §6 Principles(원칙)를 공통으로 따르므로, 아래는 두드러지게 쓰인 §4 항목만 적습니다. §4 컴포넌트가 아닌 요소(§2 타이포 역할이나 화면 전용 일회성 요소)는 **§4 외 요소** 열에 따로 두며, 커버리지 검증 대상이 아닙니다.

| 여정 페이지 | 사용 §4 컴포넌트 | §4 외 요소 |
| --- | --- | --- |
| [`JRN-connect-repo`](./JRN-connect-repo.html) | Input field · Button(primary·secondary·ghost) · Tag(status badge 포함) · Card · Icon container · Segment selector · Bottom tab bar · Metric grid · Progress bar | Section title (§2.2 타이포 역할) |
| [`JRN-discover-features`](./JRN-discover-features.html) | Step(done·active·todo — active 의 회전 ring 포함) · Card · Button(primary·secondary·ghost) · Tag(status badge) · Input field · Code block(kw) · Metric grid · Progress bar | Section title · 근거 줄 · 전략 항목 줄 · 후보 카드 (모두 화면 전용) |
| [`JRN-review-feature`](./JRN-review-feature.html) | Card · Tag · Button(primary·secondary·ghost) · Input field · Code block(diff add·del 포함) | Section title · 시나리오 카드 · 의존성 그래프(화면 전용 inline SVG) · 의존성 줄 |
| [`JRN-follow-code-change`](./JRN-follow-code-change.html) | Step · Card · Tag(status) · Button(primary·secondary·ghost) · Input field · Code block(kw · diff) · Metric grid · Progress bar | Section title · 후보 카드 · 시나리오 카드 · 의존성 그래프 |
| [`JRN-understand-feature`](./JRN-understand-feature.html) | Card · Tag(status badge) · Button(primary·secondary·ghost) · Input field · Code block(diff) · Bottom tab bar · Metric grid · Progress bar | Section title · 시나리오 카드 · 의존성 그래프 · 의존성 줄 |

> 이 표는 화면 단위(`S01`~`S10`) 매핑을 여정 페이지 단위로 접은 것입니다. 화면 ID 폐지 전의 값을 합집합으로 옮겼을 뿐, 새로 판정하지 않았습니다.
>
> **커버리지: §4 컴포넌트 12개 중 11개 사용 · 미사용 1개(`Tabs` §4.6).** `Tabs` 는 전 목업의 인라인 CSS 에 정의만 있고 마크업에서 한 번도 쓰이지 않습니다. 화면 단위 매핑 표에도 처음부터 등장한 적이 없었고, 2026-09-03 폐지 작업에서 드러났습니다. [`doc-tracker.md`](../doc-tracker.md) 「위험 진단」 참조.

## 흐름의 합의는 어디서 하나

정보 구조를 무채색 SVG 로 먼저 합의하던 와이어프레임 층은 2026-09-03 에 제거됐습니다. 여정 페이지가 그 역할을 흡수했기 때문입니다 — 화면 하나를 따로 놓고 보는 대신, 그 화면이 실제로 놓이는 여정 안에서 눌러 보며 흐름을 확인합니다.

그래서 합의의 순서가 이렇게 바뀌었습니다.

- **흐름** — [`user-journey/`](../user-journey/) 의 여정 문서에서 단계와 터치포인트로 합의합니다.
- **화면** — 이 폴더의 여정 페이지에서 `design-system.md` 토큰을 입혀 그립니다.

PRD가 변경되거나 화면 흐름을 재검토할 때는 여정 문서의 단계·터치포인트를 먼저 갱신하고, 그 다음 해당 여정 페이지를 디자인 시스템 토큰으로 다시 그립니다.

## 화면·여정을 추가할 때

**여정 페이지가 있는 여정에 단계를 추가하면** — 여정 문서에 `STP-<슬러그>` 헤딩과 터치포인트를 먼저 쓰고, 그 다음 여정 페이지에 같은 `data-step` 섹션을 더합니다. 순서를 지켜야 문서가 SSOT 로 남습니다.

**새 화면을 추가하면** — ① 그 화면을 터치포인트로 쓰는 단계가 여정 문서([`user-journey/`](../user-journey/))에 먼저 있어야 합니다 ② 기존 파일의 인라인 CSS 를 복사해 씁니다 — 임의의 색·radius·폰트를 새로 만들지 않습니다. 디자인 시스템이 바뀌면 모든 목업의 인라인 CSS 를 함께 갱신합니다 ③ 그 화면을 터치포인트로 쓰는 단계의 여정 페이지에 임베드하거나, 아직 이관 전이면 위 **이관 대기 원장**과 **화면별 매핑** 표에 행을 추가하고 상한을 올립니다 ④ 위 **Mockup index** 표에 행을 추가합니다 ⑤ [`doc-tracker.md`](../doc-tracker.md)의 연결 매트릭스와 변경 이력, 허브 [`../index.html`](../index.html)의 링크를 함께 갱신합니다.

**여정을 이관하면** — 여정 페이지를 만들고, 흡수한 화면 파일을 삭제하고, 위 **Journeys** 표·**이관 대기 원장**·상한·허브·`doc-tracker.md` 를 함께 갱신합니다. 체커가 넷의 불일치를 잡습니다.
