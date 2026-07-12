# FeatureDoc 문서 체계 상태 추적

## 현재 상태 요약
- **마지막 검증 시점**: 2026-05-27 (design-doc 정합성 검증)
- 정의된 가치: **8개** (V1 ~ V8)
- PRD: **4개** (analysis-pipeline, feature-representation, doc-management, platform)
- Acceptance Criteria: **24개** (가치 연결됨: 24 / 미연결: 0)
- 테스트 문서: **4개** (AC 커버됨: 24 / 미커버: 0)
- 사용자 여정 문서: **5개** (README + 플로우 4개) — 가치/AC/와이어프레임을 사용자 행동 시간 축으로 재구성
- 와이어프레임: **10개** (S01 ~ S10, 정보 구조 SVG)
- 목업: **10개** (S01 ~ S10, 디자인 시스템 적용 HTML) — 와이어프레임과 1:1 대응, 가치 연결됨: 10 / 미연결: 0
- 디자인 시스템 §4 컴포넌트: **12개** (사용됨: 12 / 미사용: 0)
- **건강 상태**: ⚠️ **위험 있음 — 제품 소유자(Product Owner) 미지정** · 수용된 위험 2건(S11 변경 이력 화면 미시각화 · S01 로그인 상태 미시각화)

> 가치 정의·문서화·검증 구조는 완비되었지만, 모든 가치의 책임자가 TBD인 상태이므로 "고아 가치(orphan value)" 위험이 존재합니다. 가장 우선 해결할 항목은 제품 소유자 지정입니다.

## 연결 매트릭스 — 가치 ↔ PRD ↔ AC ↔ 테스트

| 가치 | 연결 PRD | 연결 AC | 연결 테스트 | 상태 |
|------|----------|---------|-------------|------|
| V1: 코드에서 시작하는 feature 발견 가능성 | analysis-pipeline, doc-management | AC1.1, AC1.3, AC1.4, AC3.2 | analysis-pipeline, doc-management | ✅ 완전 |
| V2: 횡단·종단 양방향 이해 | analysis-pipeline, feature-representation | AC1.2, AC1.3, AC1.4, AC2.4 | analysis-pipeline, feature-representation | ✅ 완전 |
| V3: 최종 사용자 관점의 표현 | feature-representation, doc-management | AC2.1, AC2.2, AC2.3, AC3.1, AC3.2 | feature-representation, doc-management | ✅ 완전 |
| V4: 코드와 일치하는 살아있는 문서 | analysis-pipeline, feature-representation, doc-management | AC1.2, AC2.1, AC2.2, AC2.6, AC3.1, AC3.3, AC3.4, AC3.5 | 3개 테스트 문서 | ✅ 완전 |
| V5: feature 단위의 의존성 가시성 | feature-representation | AC2.4, AC2.5, AC2.6 | feature-representation | ✅ 완전 |
| V6: 사용자가 통제하는 자격증명·비용 | analysis-pipeline, platform | AC1.1, AC4.1, AC4.2, AC4.3, AC4.6, AC4.7, AC4.8 | analysis-pipeline, platform | ✅ 완전 |
| V7: 모바일에서 즉시 검토·수정 가능 | analysis-pipeline, doc-management, platform | AC1.5, AC3.1, AC3.5, AC4.4 | 3개 테스트 문서 | ✅ 완전 |
| V8: 운영 환경에서의 안정적 가용성 | analysis-pipeline, platform | AC1.5, AC4.3, AC4.5, AC4.6, AC4.7, AC4.8 | analysis-pipeline, platform | ✅ 완전 |

## 연결 매트릭스 — PRD ↔ 요구사항(원본 13개)

| 원본 요구사항 | 처리 위치 |
|---------------|-----------|
| #1 GitHub repository 단위 작동 | values 정체성 / AC1.1 |
| #2 feature(최종 사용자 노출 기능) 문서화 | values 정체성 / AC1.4, AC2.3 |
| #3 LLM API 호출로 문서 생성·수정 | AC1.2, AC2.1, AC3.1, AC4.2 |
| #4 코드베이스 종단·횡단 분석 | values 정체성 / V2 |
| #5 횡단 관심사 추출·문서화 | AC1.2 |
| #6 feature 탐색 전략 생성 | AC1.3 |
| #7 전략 기반 feature 목록 추출 | AC1.4 |
| #8 인수 테스트 형식 표현 (로직+테스트 코드) | AC2.1, AC2.2, AC2.3 |
| #9 feature 단위 종단 의존성 + 별도 데이터 기록 | AC2.4, AC2.5 |
| #10 LLM 활용 feature 문서 CRUD | PRD-3 전체 (AC3.1~3.5) |
| #11 k8s 서빙 | AC4.5, AC4.6 |
| #12 GitHub App 설치 + LLM API Key 등록 구조 | AC4.1, AC4.2, AC4.3 |
| #13 모바일 디바이스 우선 | AC4.4, V7 전반 |

## 연결 매트릭스 — 화면 ↔ 와이어프레임 ↔ 목업

UX 산출물의 연결. 10개 화면(S01~S10)이 각각 와이어프레임(정보 구조)과 목업(디자인 시스템 적용 최종 형태)을 모두 갖습니다. 화면별 AC·가치·플로우 매핑의 단일 소스는 [`mockups/README.md`](mockups/README.md)입니다.

| 화면 | 플로우 | 가치 | 와이어프레임 | 목업 | 상태 |
|------|--------|------|--------------|------|------|
| S01 Credentials Setup | 1 진입과 연결 | V6 | ✅ | ✅ | ✅ 완전 |
| S02 Home · Repositories | 1 진입과 연결 | V1·V8 | ✅ | ✅ | ✅ 완전 |
| S03 Connect Repository | 1 진입과 연결 | V6·V8 | ✅ | ✅ | ✅ 완전 |
| S04 Analysis in Progress | 2 자동 발견 | V7·V8 | ✅ | ✅ | ✅ 완전 |
| S05 Cross-cutting Concerns | 2 자동 발견 | V2·V4 | ✅ | ✅ | ✅ 완전 |
| S06 Discovery Strategy | 2 자동 발견 | V1·V2 | ✅ | ✅ | ✅ 완전 |
| S07 Feature Candidates | 2 자동 발견 | V1 | ✅ | ✅ | ✅ 완전 |
| S08 Feature · Acceptance | 3 Feature 문서 | V3·V4 | ✅ | ✅ | ✅ 완전 |
| S09 Feature · Dependencies | 3 Feature 문서 | V5 | ✅ | ✅ | ✅ 완전 |
| S10 LLM-assisted Edit | 3 Feature 문서 | V3·V4·V7 | ✅ | ✅ | ✅ 완전 |

목업은 디자인 시스템([`design-system.md`](design-system.md))의 토큰·컴포넌트를 각 HTML 파일에 인라인 CSS로 적용합니다 (와이어프레임 SVG처럼 화면별 단독 파일). 디자인 시스템 §4의 12개 컴포넌트(Button·Tag·Input·Card·Step·Tabs·Bottom tab bar·Code block·Icon container·Segment selector·Metric grid·Progress bar)는 모두 1개 이상의 목업에서 사용됩니다. 화면별 §4 컴포넌트 사용 매핑과 §4 외 요소(Section title 타이포 역할, 화면 전용 의존성 그래프)의 단일 소스는 [`mockups/README.md`](mockups/README.md)의 "Mockup index"입니다.

## 위험 진단

### 🔴 고아 가치 (소유자 없는 가치)
- V1 ~ V8 **전체** — [`values.md`](values.md)의 "제품 소유자"가 TBD이므로 8개 가치 모두 책임자 미지정 상태입니다.
- **권장 조치**: 제품 소유자를 지정한 뒤 가치 문서를 갱신하세요.

### 🟡 검토가 필요한 의사결정
- **제품 명칭 미확정**: 현재 임시명 `FeatureDoc`을 사용 중. 확정 시 가치 문서/PRD/테스트 문서 파일명도 함께 갱신해야 합니다.
- **결정적 재현성의 한계**: AC1.2의 "결정적 재현 또는 차이 명시" 정책은 LLM 응답의 비결정성을 어디까지 허용할지에 대한 정책 결정이 필요합니다.
- **삭제된 feature의 보관 기간**: AC3.3에서 "일정 기간"으로만 표현됨. 운영 정책 결정 필요.

### 🟢 검증 커버리지
- 미정렬 문서: **없음** ✅
- 무가치 PRD: **없음** ✅
- AC 없는 PRD: **없음** ✅
- 미연결 AC: **없음** ✅
- 미검증 AC: **없음** ✅
- 고아 테스트: **없음** ✅
- 고아 여정 (가치 미참조): **없음** ✅ — 플로우 1~4 모두 유효한 가치 식별자 참조
- 고아 목업 (가치/여정 미연결): **없음** ✅ — 10개 목업 모두 화면·플로우·가치에 매핑됨
- 시각화 누락 화면 (목업 없는 와이어프레임): **없음** ✅ — 10개 와이어프레임 모두 목업 보유
- 시각화 누락 단계 (mockup 없는 여정 단계): **2건** — 플로우 4 시나리오 D(변경 이력 회고)의 변경 이력 화면(S11 후보)이 와이어프레임·목업 모두 없음 · 플로우 1 S01의 로그인(미인증) 상태가 와이어프레임·목업에 미표현. → 아래 "수용된 위험" 참조
- 시각화 없는 가치 (목업 없는 가치): **없음** ✅ — V1~V8 모두 1개 이상 목업이 시각화
- 임의 스타일 목업 (디자인 시스템 미사용): **없음** ✅ — 전 목업이 인라인 CSS로 디자인 시스템 적용
- 사용처 없는 컴포넌트: **없음** ✅ — 디자인 시스템 §4의 12개 컴포넌트 모두 사용됨
- 미정의 항목 사용 (§4에 없는 컴포넌트 참조): **없음** ✅ — 2026-05-27 검증에서 발견된 6건을 해소함. 실 컴포넌트 3개(Segment selector·Metric grid·Progress bar)는 §4.10~4.12로 정식 추가, Section title(§2.2 타이포 역할)·진행 링(§4.5 Step의 일부)·의존성 그래프(화면 전용 inline SVG)는 `mockups/README.md` 인덱스에서 "§4 외 요소"로 재분류

## 수용된 위험

사용자가 인지하고 의도적으로 받아들인 위험. 검증 시 일반 위험 보고에서 제외하되 여기에 카운트로 남긴다.

| 위험 종류 | 대상 | 사유 | 수용 시점 |
|-----------|------|------|-----------|
| 🟡 시각화 누락 단계 | 플로우 4 시나리오 D — 변경 이력 화면(S11 후보) | 변경 이력 화면은 후순위 작업으로 분류. 지금은 와이어프레임/목업을 제작하지 않고, 향후 제작 시 `wireframes/README.md`·`mockups/README.md`·본 추적 문서를 함께 갱신한다 (`user-journey/04` 시나리오 D의 "누락 항목" 메모 및 `user-journey/README.md §6` 갱신 정책 참조). | 2026-05-27 |
| 🟡 시각화 누락 단계 | 플로우 1 S01 — 로그인(미인증) 상태 | 구현(GitHub OAuth 게이트)과 여정 문서(플로우 1 S01 주요 행동 (0))에는 로그인 단계가 있으나, S01 와이어프레임·목업은 로그인 후 상태만 표현한다. 후속 제작 시 `gen-wireframes.js`의 S01 정의와 `mockups/s01-*.html`에 미인증 상태를 추가하고 `wireframes/README.md`·`mockups/README.md`의 S01 행 AC에 AC4.8을 더한 뒤 본 문서에서 해제한다 (그 전까지 두 README의 S01 행은 아티팩트가 실제 표현하는 AC4.1~4.3만 유지). | 2026-07-12 |

> 수용된 위험: **2건**.

## 변경 이력

| 시점 | 변경 내용 | 이전 상태 | 이후 상태 |
|------|-----------|-----------|-----------|
| 초기 생성 | 가치 문서 + PRD 4개 + 테스트 문서 4개 + 추적 문서 일괄 생성 | 문서 0개 | 가치 8, PRD 4, AC 23, 테스트 4 |
| user-journey 추가 | `docs/user-journey/` 추가 — README + 플로우 4개. 8개 가치 / 21개 AC / 10개 와이어프레임을 사용자 행동의 시간 축으로 재엮음 (신규 가치/AC 추가 없음) | 가치 8, PRD 4, AC 23, 테스트 4 | 좌동 + 사용자 여정 5개 |
| GitHub 인증 모델 변경 | GitHub PAT 등록 → GitHub App 설치 모델로 전환. AC4.1 재작성, AC1.1/AC4.3 보강, S01·S03 와이어프레임·테스트·사용자 여정 문서 동기화 (AC 수 변동 없음) | PAT 기반 자격증명 | GitHub App 설치 기반 |
| 목업 추가 | `docs/mockups/` 추가 — 화면별 단독 HTML 목업 10개(S01~S10, 디자인 시스템 CSS 인라인) + 인덱스 README. 디자인 시스템(`design-system.md`)이 참조하던 "별도 HTML mockup"의 실체를 구현. 신규 가치/AC/와이어프레임 추가 없음 | 와이어프레임 10, 목업 0 | 와이어프레임 10, 목업 10 |
| S02 진입점 보강 | 사용자 여정(`user-journey/01`)이 전제하던 S02의 "새 저장소 추가" 진입점이 와이어프레임·목업에 누락된 여정↔시각화 불일치를 해소. S02 섹션 헤더에 `+ New` 액션 추가 (목업 HTML + `gen-wireframes.js` S02 정의 + s02 SVG 재생성). 신규 화면/AC 추가 없음 | 여정↔S02 시각화 불일치 | 일치 ✅ |
| 2026-07-12 OAuth 인증 문서 편입 | 구현(#5·#6)으로 존재하던 GitHub OAuth 로그인·세션·계정을 PRD-4 범위·AC4.8로 편입, `test/04` 시나리오 11·12 추가, 여정 플로우 1(S01)에 로그인 단계 반영, README §5 매트릭스·카운트 갱신. "사용자 본인 인증 방식 미정의" 의사결정 해소(결정: GitHub OAuth, PRD-4 확장). 구현 수렴 로드맵(슬라이스 2~7) 신설. S01 로그인(미인증) 상태 미시각화는 수용된 위험으로 등재 (reconciler `rct_20260712-0001`) | AC 23 · 인증 비범위(문서↔구현 모순) | AC 24 · 인증 범위 편입(모순 해소) |
| 2026-05-27 design-doc 정합성 검증 | 4종 문서(가치/여정/디자인 시스템/목업) 정합성 검증. 위험 2건 발견 후 처리: ① 🟡 시각화 누락 단계(S11 변경 이력 화면) → "수용된 위험"으로 기록 ② 🟢 미정의 항목 사용 6건 → 실 컴포넌트 3개를 `design-system.md` §4.10~4.12로 추가하고 §4 헤더를 "12개"로 정정, 나머지 3개는 `mockups/README.md` 인덱스에서 "§4 외 요소"로 재분류. 부수 정정: `featuredoc-values.md`→`values.md` 오기, `user-journey/README.md §5` AC 커버리지(22→21개, 예외 AC4.5·AC4.7→AC3.2·AC4.4), `wireframes/README.md` AC 칸의 가치 표기 제거. `.claude/skills/`에 주입형 스킬 2개(ui-with-design-system, screen-with-mockup-and-design-system) 추가 | §4 컴포넌트 9 표기·미정의 참조 6건·검증 위험 미기록 | §4 컴포넌트 12 정합·미정의 참조 0·수용된 위험 1건 기록 |

## 다음 단계 권장

1. **제품 소유자 지정** — 모든 가치의 고아 상태를 해소합니다.
2. **제품 명칭 확정** — 파일명·문서 내 식별자 일괄 변경.
3. **운영 정책 수치화** — 보관 기간, 동시 분석 한도, LLM 호출 한도 등.
4. **PRD 검토 워크숍** — 4개 PRD를 소유자/엔지니어와 함께 검토하여 누락된 AC가 있는지 확인.
5. **S11 변경 이력 화면 제작** — 현재 "수용된 위험"으로 보류 중. 제작 시 와이어프레임(`gen-wireframes.js` S11 entry)→목업 순서로 만들고 본 추적 문서의 수용된 위험에서 해제.

> 참고: `.claude/skills/`에 디자인 시스템 보조 스킬 2개(`ui-with-design-system`, `screen-with-mockup-and-design-system`)가 설치되어 있습니다. 이 레포에서 UI/화면 코드 작업 시 디자인 시스템·목업을 자동으로 참조합니다. `.gitignore`로 제외하지 마세요.

## 구현 수렴 로드맵 (권장 순서)

미구현 AC 20건(AC1.1~1.5 · AC2.1~2.6 · AC3.1~3.5 · AC4.4~4.7)을 문서 → 구현으로 수렴시키는 권장 슬라이스 순서. 순서 근거는 **사용자 여정 순서**(연결 → 발견 → 문서)와 **기술 의존성**(분석 파이프라인은 큐·워커 기질이, feature 표현은 분석 산출물이, 문서 관리는 표현이 먼저 필요)이다. AC4.4(모바일 우선)는 각 슬라이스의 화면마다 적용하고, AC4.6(관측·비용)·AC4.7(격리)은 파이프라인 구축 시 내장한 뒤 마지막 슬라이스에서 마감 검증한다. (수립: 2026-07-12, reconciler `rct_20260712-0001`)

| 슬라이스 | 대상 AC | 화면 | 내용 |
|----------|---------|------|------|
| 1 (이 변경) | AC4.8 문서화 | — | OAuth 인증을 PRD-4 범위·AC로 편입, 로드맵 확정 |
| 2 | AC1.1 | S02 · S03 | 저장소 연결·홈 목록·분석 트리거·pre-flight 비용 안내 |
| 3 | AC4.5 · AC1.5 | S04 | 큐 + API/워커 워크로드 분리, 비동기 진행 가시성 |
| 4 | AC1.2 → AC1.3 → AC1.4 | S05 · S06 · S07 | 분석 파이프라인(횡단 관심사 → 탐색 전략 → feature 후보), AC4.6 계측 내장 |
| 5 | AC2.1~2.6 | S08 · S09 | 인수 기준 도출·보강·문서화, 의존성 추출·저장·재계산 |
| 6 | AC3.1~3.5 | S10 | LLM 문서 수정·직접 추가·삭제·이력·충돌 처리 |
| 7 | AC4.4 · AC4.6 · AC4.7 | 전 화면 | 모바일 우선 전 화면 확인, 사용자 노출 비용 완성, 격리 검증 |

> 각 슬라이스는 reconciler 정합성 루프의 개별 task로 계획·실행되며, 슬라이스 완료마다 본 문서의 미구현 AC 잔여를 갱신한다.
