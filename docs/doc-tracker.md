# FeatureDoc 문서 체계 상태 추적

## 현재 상태 요약
- 정의된 가치: **8개** (V1 ~ V8)
- PRD: **4개** (analysis-pipeline, feature-representation, doc-management, platform)
- Acceptance Criteria: **23개** (가치 연결됨: 23 / 미연결: 0)
- 테스트 문서: **4개** (AC 커버됨: 23 / 미커버: 0)
- 사용자 여정 문서: **5개** (README + 플로우 4개) — 가치/AC/와이어프레임을 사용자 행동 시간 축으로 재구성
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
| #12 사용자 GitHub Token + LLM API Key 등록 구조 | AC4.1, AC4.2, AC4.3 |
| #13 모바일 디바이스 우선 | AC4.4, V7 전반 |

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

## 변경 이력

| 시점 | 변경 내용 | 이전 상태 | 이후 상태 |
|------|-----------|-----------|-----------|
| 초기 생성 | 가치 문서 + PRD 4개 + 테스트 문서 4개 + 추적 문서 일괄 생성 | 문서 0개 | 가치 8, PRD 4, AC 23, 테스트 4 |
| user-journey 추가 | `docs/user-journey/` 추가 — README + 플로우 4개. 8개 가치 / 22개 AC / 10개 와이어프레임을 사용자 행동의 시간 축으로 재엮음 (신규 가치/AC 추가 없음) | 가치 8, PRD 4, AC 23, 테스트 4 | 좌동 + 사용자 여정 5개 |

## 다음 단계 권장

1. **제품 소유자 지정** — 모든 가치의 고아 상태를 해소합니다.
2. **제품 명칭 확정** — 파일명·문서 내 식별자 일괄 변경.
3. **본인 인증 방식 결정** — PRD-4 확장 또는 별도 PRD 추가.
4. **운영 정책 수치화** — 보관 기간, 동시 분석 한도, LLM 호출 한도 등.
5. **PRD 검토 워크숍** — 4개 PRD를 소유자/엔지니어와 함께 검토하여 누락된 AC가 있는지 확인.
