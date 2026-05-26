# Wireframes

10개 모바일 화면의 정보 구조 와이어프레임. 색상·실제 폰트·디테일은 모두 빠지고 **레이아웃과 정보 위계만** 표현돼요. 디자인 톤은 [`design-system.md`](../design-system.md)에서 별도로 정의됩니다.

- **Format** — SVG, 393 × 844 (iOS 모바일 viewport)
- **Style** — 흰 배경, 1px 회색 hairline, 회색 텍스트 (status 색 없음)
- **Source** — `gen-wireframes.js`로 일괄 생성 가능

## Flows

### 01 — Onboarding & Connect
| ID  | Wireframe | AC                    | Purpose                                          |
| --- | --------- | --------------------- | ------------------------------------------------ |
| S01 | [s01-credentials-setup.svg](./s01-credentials-setup.svg) | AC4.1 · AC4.2 · AC4.3 | GitHub App 연결 + LLM 자격증명 등록              |
| S02 | [s02-home-repositories.svg](./s02-home-repositories.svg) | AC1.1 · AC1.5         | 연결된 저장소 목록과 분석 상태                  |
| S03 | [s03-connect-repository.svg](./s03-connect-repository.svg) | AC1.1 · AC4.6         | 새 저장소 연결 + 분석 pre-flight 비용 안내     |

### 02 — Discovery
| ID  | Wireframe | AC                    | Purpose                                          |
| --- | --------- | --------------------- | ------------------------------------------------ |
| S04 | [s04-analysis-progress.svg](./s04-analysis-progress.svg) | AC1.5 · AC4.6         | 분석 파이프라인 진행 상황 (5단계)              |
| S05 | [s05-cross-cutting-concerns.svg](./s05-cross-cutting-concerns.svg) | AC1.2 · V2            | 추출된 횡단 관심사 (인프라·아키텍처·프레임워크·미들웨어) |
| S06 | [s06-discovery-strategy.svg](./s06-discovery-strategy.svg) | AC1.3 · V1·V2         | LLM이 만든 탐색 전략 검토 및 승인              |
| S07 | [s07-feature-candidates.svg](./s07-feature-candidates.svg) | AC1.4 · V1            | feature 후보 목록 + 승인/거부/병합              |

### 03 — Feature Documents
| ID  | Wireframe | AC                    | Purpose                                          |
| --- | --------- | --------------------- | ------------------------------------------------ |
| S08 | [s08-feature-acceptance.svg](./s08-feature-acceptance.svg) | AC2.1 · AC2.2 · AC2.3 | feature의 인수 시나리오 (Given-When-Then 4개)  |
| S09 | [s09-feature-dependencies.svg](./s09-feature-dependencies.svg) | AC2.4 · AC2.5 · V5    | 종단 의존성 그래프 + 카테고리별 의존성 목록    |
| S10 | [s10-llm-edit.svg](./s10-llm-edit.svg) | AC3.1 · V3·V4·V7      | LLM 보조 편집 — 자연어 지시 → diff → 승인     |

## Why wireframes (separate from mockups)

- **Mockup** — 디자인 톤·색·폰트·인터랙션이 모두 결정된 최종 형태. 디자인 시스템의 표현 결과물.
- **Wireframe** — 정보 구조와 화면 흐름의 합의. 디자인 톤이 바뀌어도 기능 변경이 없으면 그대로 유지됨.

따라서 PRD가 변경되거나 화면 흐름을 재검토할 때 와이어프레임을 먼저 갱신하고, 그 다음 mockup을 디자인 시스템 토큰으로 다시 그립니다.

## Regenerating

와이어프레임은 단일 Node.js 스크립트 [`tools/gen-wireframes.js`](../../tools/gen-wireframes.js)에서 생성됩니다. 화면 추가 시 `screens` 객체에 entry를 추가하고 스크립트를 다시 실행하면 같은 컨벤션으로 일관된 SVG가 만들어져요.

```bash
cd tools
node gen-wireframes.js
```
