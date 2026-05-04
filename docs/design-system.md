# Design System

> FeatureDoc 모바일 UX의 디자인 토큰·컴포넌트·원칙. 다크 톤, 단일 sans, 액센트 거의 없음.

이 문서는 `feature-doc` 모바일 UX의 단일 진실 원천이에요. 새 화면을 만들거나 기존 화면을 수정할 때 토큰·컴포넌트·원칙을 여기서 가져와 일관성을 유지합니다.

- **Audience** — 디자이너, 프론트엔드 엔지니어, LLM 어시스턴트
- **Status** — v0.1 (mockup-derived)
- **Theme** — Dark only

---

## 1. Foundations

### 1.1 Backgrounds

5단계 표면. 그림자·그라디언트 없이 색 차이로만 elevation을 표현합니다.

| Token              | Hex       | Usage                          |
| ------------------ | --------- | ------------------------------ |
| `--bg-base`        | `#0A0A0A` | 페이지 베이스                  |
| `--bg-elevated`    | `#0F0F0F` | 모바일 frame, 코드 헤더        |
| `--bg-card`        | `#141414` | card, repo, dep, scenario      |
| `--bg-input`       | `#181818` | input field, icon container    |
| `--bg-hover`       | `#1C1C1C` | hover 상태 (예약)              |

### 1.2 Borders

모든 보더는 1px hairline. 두께는 변경하지 말고 색의 강약으로만 구분.

| Token               | Hex       | Usage                            |
| ------------------- | --------- | -------------------------------- |
| `--border-subtle`   | `#1D1D1D` | 섹션·카드 분리, 점선 구분        |
| `--border-default`  | `#272727` | 버튼·태그·input·frame            |
| `--border-strong`   | `#333333` | 활성 상태, 강한 강조 1곳         |

### 1.3 Text

4단계 위계. `primary`는 본문·헤딩, `tertiary`는 메타·라벨, `quaternary`는 비활성·placeholder.

| Token                | Hex       | Usage                       |
| -------------------- | --------- | --------------------------- |
| `--text-primary`     | `#EDEDED` | 본문·헤딩·핵심 값           |
| `--text-secondary`   | `#9A9A9A` | 보조 본문·설명              |
| `--text-tertiary`    | `#6A6A6A` | 메타·라벨·아이콘            |
| `--text-quaternary`  | `#454545` | placeholder·dim             |

### 1.4 Status

채도를 끌어내려 거의 무채색에 가깝게 처리. **직접 텍스트나 배경에 쓰지 않고 작은 dot으로만 신호로 사용**합니다.

| Token       | Hex       | Usage                       |
| ----------- | --------- | --------------------------- |
| `--success` | `#86B89A` | 완료, 검증됨, 추가          |
| `--warning` | `#C9A868` | 주의, 충돌, 만료 임박       |
| `--danger`  | `#C98080` | 실패, 거부, 삭제            |
| `--info`    | `#8AA4C4` | 중립 정보, 힌트             |

### 1.5 Accent

화면 전체에서 1~2개의 정말 중요한 신호에만 사용. footer 레전드 등 표식용으로 정의는 유지하지만, 일반 UI에서는 거의 비사용입니다.

| Token             | Value                       | Usage                          |
| ----------------- | --------------------------- | ------------------------------ |
| `--accent`        | `#C8FF5E`                   | brand 표식 (거의 비사용)       |
| `--accent-dim`    | `rgba(200,255,94,.06)`      | 액센트 fill (예약)             |
| `--accent-border` | `rgba(200,255,94,.18)`      | 액센트 outline (예약)          |

---

## 2. Typography

2개 family만 사용해요. 디스플레이용 별도 폰트 (세리프·이탤릭 등)는 도입하지 않습니다 — 화면마다 다른 큰 헤드라인이 시각 산만함의 주 원인이었기 때문.

### 2.1 Families

| Family            | Weights              | Usage                                     |
| ----------------- | -------------------- | ----------------------------------------- |
| **Geist**         | 300 / 400 / 500 / 600 | 본문 모든 곳. `ss01`, `cv11` 활성화      |
| **JetBrains Mono** | 400 / 500            | 메타·라벨·숫자·경로·코드 전용            |

### 2.2 Scale

실제 mockup에서 쓰인 크기들. 스케일 폭이 좁음 — 한 화면 안의 시각적 도약을 줄이기 위함.

| Role               | Size / Weight | Family | Notes                                     |
| ------------------ | ------------- | ------ | ----------------------------------------- |
| Page H1            | 28 / 500      | sans   | letter-spacing -.015em, line-height 1.2   |
| Display            | 20 / 500      | sans   | 화면 핵심 헤딩 (`.h-display`)             |
| App bar title      | 18 / 500      | sans   | 화면 상단 타이틀                          |
| Metric             | 18 / 500      | mono   | 숫자·금액·카운트                          |
| Body               | 14 / 400      | sans   | 1.55 line-height                          |
| Body small         | 13 / 400      | sans   | secondary 색, 시나리오·인수기준·설명문    |
| Sub                | 12.5 / 400    | sans   | tertiary 색, display 부속 (`.h-display-sub`) |
| Section title      | 11 / 400      | mono   | UPPER, tracking .14em, tertiary           |
| Caps label         | 10 / 400      | mono   | UPPER, tracking .16em, input·dep 헤더     |

### 2.3 Hierarchy rule

**한 화면당 핵심 헤딩 1개**. 20px `.h-display` 하나만 사용하고, 부가 설명은 12.5px `.h-display-sub`. 화면마다 다른 큰 헤드라인 (다른 폰트·다른 표현)을 만들지 않습니다.

```html
<h3 class="h-display">Discovery strategy</h3>
<p class="h-display-sub">LLM이 횡단 관심사를 토대로 만든 탐색 전략이에요.</p>
```

---

## 3. Layout

### 3.1 Spacing scale

8px 기본 그리드. systematic scale 없이 시각적 호흡으로 결정합니다.

| Value | Usage                                    |
| ----- | ---------------------------------------- |
| 4     | 라벨·아이콘 마진                         |
| 6     | 작은 chip 간 gap                         |
| 8     | 컴포넌트 간격                            |
| 10    | 카드 사이                                |
| 14    | section 내부 여백                        |
| 18    | 카드 grouping 사이                       |
| 22    | 화면 좌우 패딩 (모바일)                  |
| 28    | subsection 사이                          |
| 38    | mock frame radius                        |
| 56    | section 사이 (page level)                |

### 3.2 Radius

5단계. 작은 것은 명확히 작게, 큰 것은 frame처럼 명확히 크게. 어중간한 값은 회피합니다.

| Radius | Usage                       |
| ------ | --------------------------- |
| 4      | tag                         |
| 6      | icon container              |
| 9      | button                      |
| 11     | input                       |
| 14     | card                        |
| 38     | mobile frame                |
| 50%    | dot, status indicator       |

**규칙**: 크기와 라디우스가 함께 자람. 작은 요소엔 작은 라디우스, 큰 컨테이너에만 큰 라디우스.

### 3.3 Borders & depth

- 모든 보더는 **1px**
- `box-shadow`, `text-shadow`, `filter: drop-shadow()`, `gradient` 사용 금지
- 깊이는 **background 톤 차이**로만 표현 (4단계: base / elevated / card / input)

---

## 4. Components

12개 primitive. 모두 토큰의 조합으로 만들어졌고, **변형 (variants)은 색이 아니라 채워짐·채움 부재로 구분**됩니다.

### 4.1 Button

3 variants. **Primary는 화면당 1개 원칙** — 가장 중요한 액션 하나에만.

| Variant     | Class             | Style                                     |
| ----------- | ----------------- | ----------------------------------------- |
| Primary     | `.btn-primary`    | 흰색 fill, dark 텍스트                    |
| Secondary   | `.btn-secondary`  | 보더만, primary 텍스트                    |
| Ghost       | `.btn-ghost`      | 텍스트만, tertiary 색                     |

### 4.2 Tag

모든 태그가 동일한 회색 outline. 상태는 작은 dot의 색으로만 신호. **채워진 라임/그린/오렌지 배지는 만들지 않음**.

```html
<span class="tag">repo</span>
<span class="tag accent"><span class="dot"></span>DRAFT v2</span>
<span class="tag success"><span class="dot"></span>VERIFIED</span>
<span class="tag warn"><span class="dot"></span>CONFLICT</span>
<span class="tag danger"><span class="dot"></span>FAILED</span>
```

- 4 / 9 / 10 padding · mono · UPPER 또는 lower
- 모디파이어: `.accent` `.success` `.warn` `.danger` (dot 색만 변함)

### 4.3 Input field

caps mono 라벨 + mono value. 한 컴포넌트가 라벨·값·부가 액션 (검증 상태 등)을 모두 담음.

```html
<div class="input">
  <span class="lbl">Repository URL</span>
  <span class="val">github.com/dlddu/payments-api</span>
</div>
```

- bg-input · 11px radius · 13/15 padding

### 4.4 Card

일반 컨테이너. **모든 카드는 같은 스타일** — 보더 1px, bg-card, 14px radius. 강조 카드 (다른 색·다른 보더)는 만들지 않습니다.

- 14px radius · 14/16 padding · subtle border

### 4.5 Step

파이프라인 진행 표현. 3가지 상태:

- `.done` — 채워진 dot + check 아이콘
- `.active` — 보더 + 회전 dashed ring
- `.todo` — 보더만, 빈 원

22px ic · 13.5 sans label · mono sub/time

### 4.6 Tabs

컨테이너 1px 보더 + 활성 탭만 dark fill + ring. 화면 내 sub-navigation에 사용.

```html
<div class="tabs">
  <div class="t active">Acceptance · 4</div>
  <div class="t">Dependencies</div>
  <div class="t">History</div>
</div>
```

활성 탭은 색이 아닌 fill (bg-base) + 보더 ring으로 표현.

### 4.7 Bottom tab bar

화면 내비게이션. 4개 슬롯 고정, 활성은 색만 흰색 (액센트 사용 금지).

- 9px mono caps · UPPER · tracking .1em

### 4.8 Code block

YAML·diff·구성 표시. **syntax 강조는 거의 무채색** — 구조 식별이 목적이지 화려함이 아님.

| Class    | Color    | Usage                       |
| -------- | -------- | --------------------------- |
| `.kw`    | primary  | YAML 키, 식별자             |
| `.str`   | secondary | 문자열                     |
| `.com`   | quaternary | 주석                      |
| `.add`   | desat green tint | diff +라인          |
| `.del`   | desat red tint   | diff -라인          |

- 11.5/1.6 mono · `#070707` 배경

### 4.9 Icon container

아이콘 옆에 의미 부여할 때 — 채움·테두리·돔 색은 모두 동일한 회색.

| Size | Radius | Usage                       |
| ---- | ------ | --------------------------- |
| 24   | 6      | 인라인 아이콘 보조          |
| 28   | 7      | 카드 내 아이콘              |
| 36   | 10     | 앱바 icon-btn               |

---

## 5. CSS tokens

바로 복사해서 다른 화면에 적용할 수 있는 변수 정의입니다. 폰트 임포트와 `:root` 블록을 함께 두면 어떤 컴포넌트도 같은 톤으로 만들어져요.

### 5.1 Fonts

```html
<link
  href="https://fonts.googleapis.com/css2?family=Geist:wght@300;400;500;600&family=JetBrains+Mono:wght@400;500&display=swap"
  rel="stylesheet"
>
```

### 5.2 :root

```css
:root {
  /* surfaces */
  --bg-base:        #0a0a0a;
  --bg-elevated:    #0f0f0f;
  --bg-card:        #141414;
  --bg-input:       #181818;
  --bg-hover:       #1c1c1c;

  /* borders — always 1px */
  --border-subtle:  #1d1d1d;
  --border-default: #272727;
  --border-strong:  #333333;

  /* text — 4 levels */
  --text-primary:    #ededed;
  --text-secondary:  #9a9a9a;
  --text-tertiary:   #6a6a6a;
  --text-quaternary: #454545;

  /* status — desaturated, used as small dot only */
  --success:  #86b89a;
  --warning:  #c9a868;
  --danger:   #c98080;
  --info:     #8aa4c4;

  /* accent — used very sparingly */
  --accent:        #c8ff5e;
  --accent-dim:    rgba(200, 255, 94, 0.06);
  --accent-border: rgba(200, 255, 94, 0.18);

  /* fonts */
  --font-sans: 'Geist', -apple-system, BlinkMacSystemFont, sans-serif;
  --font-mono: 'JetBrains Mono', 'SF Mono', Menlo, monospace;
}
```

### 5.3 Base reset

```css
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

html, body {
  background: var(--bg-base);
  color: var(--text-primary);
  font-family: var(--font-sans);
  font-feature-settings: "ss01", "cv11";
}
```

---

## 6. Principles

새 화면을 만들 때 지키면 톤이 흐트러지지 않는 8개 원칙. 각각이 mockup의 v1 → v2 변화에서 도출된 결정이에요. **의심될 때 이 8개를 다시 점검**하세요.

### 1. 하나의 폰트, 하나의 액센트

본문은 Geist, 메타·코드·숫자는 JetBrains Mono. 디스플레이용 별도 폰트 (세리프·이탤릭 등)는 도입하지 않음. 액센트 색은 정의만 두고 화면 안에서는 사실상 안 씀.

### 2. 상태는 색이 아니라 라벨로

danger를 빨강, success를 초록, warning을 주황으로 칠하지 않음. 텍스트는 항상 secondary/tertiary, 차이는 작은 dot 색만. 의미는 `"VERIFIED"` · `"CONFLICT"` 같은 라벨이 전달.

### 3. shadow·glow·gradient 없음

모든 면은 단색. 깊이는 5단계 background 톤 차이로만 표현. `drop-shadow`나 `box-shadow`로 카드를 띄우지 않고, gradient로 강조하지 않음.

### 4. 1px hairline, 그 이상은 없음

모든 보더는 1px. 카드·버튼·input·frame 전부. 두께를 늘려 강조하는 대신 색의 강약 (subtle / default / strong) 3단계로 차이.

### 5. 화면당 핵심 헤딩 1개

20px `.h-display` 하나만. 부가 설명은 12.5px `.h-display-sub`. 화면마다 다른 큰 헤드라인 (다른 폰트·다른 표현)을 만들지 않음. 시각적 운율은 일관되게.

### 6. Primary 액션은 화면당 1개, 흰색

가장 중요한 액션 하나만 흰색 fill. 나머지 보조 액션은 secondary (보더만) 또는 ghost (텍스트만). primary가 두 개 이상 있으면 어느 쪽도 primary가 아님.

### 7. Mono는 메타 정보만

숫자·금액·경로·ID·라벨·타임스탬프·코드. 본문 자연어는 sans. 메타와 본문이 시각적으로 분리되면 정보 위계가 자동으로 잡힘.

### 8. 여백이 위계를 만든다

같은 폰트·같은 색이라도 위/아래 여백이 다르면 관계가 다름. 큰 폰트로 강조하지 말고 22px (화면 좌우) · 28px (subsection) · 56px (section) 단계로 그루핑.

---

## 7. References

- **Mockups** — 이 시스템이 도출된 모바일 UX는 별도 HTML mockup으로 보존되어 있어요
- **Wireframes** — 정보 구조 검토용 SVG 와이어프레임은 [`docs/wireframes/`](./wireframes/)
- **PRDs** — 각 화면이 지원하는 acceptance criteria는 [`docs/prd/`](./prd/)
