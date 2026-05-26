# FeatureDoc 문서 체계 상태 추적

## 현재 상태 요약
- 정의된 가치: **8개** (V1 ~ V8)
- PRD: **4개** (analysis-pipeline, feature-representation, doc-management, platform)
- Acceptance Criteria: **23개** (가치 연결됨: 23 / 미연결: 0)
- 테스트 문서: **4개** (AC 커버됨: 23 / 미커버: 0)
- 사용자 여정 문서: **5개** (README + 플로우 4개) — 가치/AC/와이어프레임을 사용자 행동 시간 축으로 재구성
- 와이어프레임: **10개** (S01 ~ S10, 정보 구조 SVG)
- 목업: **10개** (S01 ~ S10, 디자인 시스템 적용 HTML) — 와이어프레임과 1:1 대응, 가치 연결됨: 10 / 미연결: 0
- **건강 상태**: ⚠️ **위험 있음 — 제품 소유자(Product Owner) 미지정**

> 가치 정의·문서화·검증 구조는 완비되었지만, 모든 가치의 책임자가 TBD인 상태이므로 "고아 가치(orphan value)" 위험이 존재합니다. 가장 우선 해결할 항목은 제품 소유자 지정입니다.

## 연결 매트릭스 — 가치 ↔ PRD ↔ AC ↔ 테스트

| 가치 | 연결 PRD | 연결 AC | 연결 테스트 | 상태 |
|------|----------|---------|-------------|------|
| V1: 코드에서 시작하는 feature 발견 가능성 | analysis-pipeline, doc-management | AC1.1, AC1.3, AC1.4, AC3.2 | analysis-pipeline, doc-management | ✅ 완전 |
| V2: 횡단·종단 양방향 이해 | analysis-pipeline, feature-representation | AC1.2, AC1.3, AC1.4, AC2.4 | analysis-pipeline, feature-representation | ✅ 완전 |
| V3: 최종 사용자 관점의 표현 | feature-representation, doc-management | AC2.1, AC2.2, AC2.3, AC3.1, AC3.2 | feature-representation, doc-management | ✅ 완전 |
| V4: 코드와 일치하는 살아있는 문서 | analysis-pipeline, feature-representation, doc-management | AC1.2, AC2.1, AC2.2, AC2.6, AC3.1, AC3.3, AC3.4, AC3.5 | 3개 테스트 문서 | ✅ 완전 |
| V5: feature 단위의 의존성 가시성 | feature-representation | AC2.4, AC2.5, AC2.6 | feature-representation | ✅ 완전 |
| V6: 사용자가 통제하는 자격증명·비용 | analysis-pipeline, platform | AC1.1, AC4.1, AC4.2, AC4.3, AC4.6, AC4.7 | analysis-pipeline, platform | ✅ 완전 |
| V7: 모바일에서 즉시 검토·수정 가능 | analysis-pipeline, doc-management, platform | AC1.5, AC3.1, AC3.5, AC4.4 | 3개 테스트 문서 | ✅ 완전 |
| V8: 운영 환경에서의 안정적 가용성 | analysis-pipeline, platform | AC1.5, AC4.3, AC4.5, AC4.6, AC4.7 | analysis-pipeline, platform | ✅ 완전 |

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

목업은 디자인 시스템([`design-system.md`](design-system.md))의 토큰·컴포넌트를 각 HTML 파일에 인라인 CSS로 적용합니다 (와이어프레임 SVG처럼 화면별 단독 파일). 디자인 시스템 §4의 9개 컴포넌트(Button·Tag·Input·Card·Step·Tabs·Bottom tab bar·Code block·Icon container)는 모두 1개 이상의 목업에서 사용됩니다.

## 위험 진단

### 🔴 고아 가치 (소유자 없는 가치)
- V1 ~ V8 **전체** — `featuredoc-values.md`의 "제품 소유자"가 TBD이므로 8개 가치 모두 책임자 미지정 상태입니다.
- **권장 조치**: 제품 소유자를 지정한 뒤 가치 문서를 갱신하세요.

### 🟡 검토가 필요한 의사결정
- **제품 명칭 미확정**: 현재 임시명 `FeatureDoc`을 사용 중. 확정 시 가치 문서/PRD/테스트 문서 파일명도 함께 갱신해야 합니다.
- **사용자 본인 인증 방식 미정의**: PRD-4에서 사용자별 격리(AC4.7)를 요구하지만 사용자 자신의 인증(SSO/이메일/소셜 로그인 등)은 비범위로 두었습니다. 별도 PRD 추가가 필요할 수 있습니다.
- **결정적 재현성의 한계**: AC1.2의 "결정적 재현 또는 차이 명시" 정책은 LLM 응답의 비결정성을 어디까지 허용할지에 대한 정책 결정이 필요합니다.
- **삭제된 feature의 보관 기간**: AC3.3에서 "일정 기간"으로만 표현됨. 운영 정책 결정 필요.

### 🟢 검증 커버리지
- 미정렬 문서: **없음** ✅
- 무가치 PRD: **없음** ✅
- AC 없는 PRD: **없음** ✅
- 미연결 AC: **없음** ✅
- 미검증 AC: **없음** ✅
- 고아 테스트: **없음** ✅
- 고아 목업 (가치/여정 미연결): **없음** ✅ — 10개 목업 모두 화면·플로우·가치에 매핑됨
- 시각화 누락 화면 (목업 없는 와이어프레임): **없음** ✅ — 10개 와이어프레임 모두 목업 보유
- 시각화 없는 가치 (목업 없는 가치): **없음** ✅ — V1~V8 모두 1개 이상 목업이 시각화
- 임의 스타일 목업 (디자인 시스템 미사용): **없음** ✅ — 전 목업이 인라인 CSS로 디자인 시스템 적용
- 사용처 없는 컴포넌트: **없음** ✅ — 디자인 시스템 §4의 9개 컴포넌트 모두 사용됨

## 변경 이력

| 시점 | 변경 내용 | 이전 상태 | 이후 상태 |
|------|-----------|-----------|-----------|
| 초기 생성 | 가치 문서 + PRD 4개 + 테스트 문서 4개 + 추적 문서 일괄 생성 | 문서 0개 | 가치 8, PRD 4, AC 23, 테스트 4 |
| user-journey 추가 | `docs/user-journey/` 추가 — README + 플로우 4개. 8개 가치 / 22개 AC / 10개 와이어프레임을 사용자 행동의 시간 축으로 재엮음 (신규 가치/AC 추가 없음) | 가치 8, PRD 4, AC 23, 테스트 4 | 좌동 + 사용자 여정 5개 |
| GitHub 인증 모델 변경 | GitHub PAT 등록 → GitHub App 설치 모델로 전환. AC4.1 재작성, AC1.1/AC4.3 보강, S01·S03 와이어프레임·테스트·사용자 여정 문서 동기화 (AC 수 변동 없음) | PAT 기반 자격증명 | GitHub App 설치 기반 |
| 목업 추가 | `docs/mockups/` 추가 — 화면별 단독 HTML 목업 10개(S01~S10, 디자인 시스템 CSS 인라인) + 인덱스 README. 디자인 시스템(`design-system.md`)이 참조하던 "별도 HTML mockup"의 실체를 구현. 신규 가치/AC/와이어프레임 추가 없음 | 와이어프레임 10, 목업 0 | 와이어프레임 10, 목업 10 |
| S02 진입점 보강 | 사용자 여정(`user-journey/01`)이 전제하던 S02의 "새 저장소 추가" 진입점이 와이어프레임·목업에 누락된 여정↔시각화 불일치를 해소. S02 섹션 헤더에 `+ New` 액션 추가 (목업 HTML + `gen-wireframes.js` S02 정의 + s02 SVG 재생성). 신규 화면/AC 추가 없음 | 여정↔S02 시각화 불일치 | 일치 ✅ |

## 다음 단계 권장

1. **제품 소유자 지정** — 모든 가치의 고아 상태를 해소합니다.
2. **제품 명칭 확정** — 파일명·문서 내 식별자 일괄 변경.
3. **본인 인증 방식 결정** — PRD-4 확장 또는 별도 PRD 추가.
4. **운영 정책 수치화** — 보관 기간, 동시 분석 한도, LLM 호출 한도 등.
5. **PRD 검토 워크숍** — 4개 PRD를 소유자/엔지니어와 함께 검토하여 누락된 AC가 있는지 확인.
