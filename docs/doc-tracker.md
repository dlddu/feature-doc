# FeatureDoc 문서 체계 상태 추적

## 현재 상태 요약
- **마지막 검증 시점**: 2026-08-30 (여정 단위 목업 이관 슬라이스 1 — `JRN-connect-repo`)
- 정의된 가치: **8개** (V1 ~ V8)
- PRD: **4개** (analysis-pipeline, feature-representation, doc-management, platform)
- Acceptance Criteria: **24개** (가치 연결됨: 24 / 미연결: 0)
- 테스트 문서: **4개** (AC 커버됨: 24 / 미커버: 0)
- e2e spec (`e2e/tests/`): **8개** — AC 전용 1:1 매핑 8건 · 예외 0건 · 공백 16건 (아래 "e2e 매핑")
- 사용자 여정 문서: **7개** (README + 여정 6개) — 행동 축으로 정리, `JRN-`/`STP-` 안정 식별자 보유. 가치 8/8 · AC 23/24 커버 (미커버 AC4.5는 사용자에게 보이지 않는 운영 배경)
- 와이어프레임: **10개** (S01 ~ S10, 정보 구조 SVG)
- 목업: **7개 페이지** — 여정 페이지 **2개**(`JRN-connect-repo` S01~S03 흡수 · `JRN-discover-features` S04~S07 흡수) + 화면 단위 **5개**(S04·S07~S10). 화면 10개는 전부 살아 있고(와이어프레임과 1:1 대응 유지) 이관된 화면은 여정 페이지 안의 앵커가 공개 경로다. S04·S07 은 `JRN-follow-code-change` 가 아직 써서 원본 파일도 함께 남는다. 여정 6개 중 목업 페이지 보유 **2** · 이관 대기 **3** · 규칙 8 예외 **1**(`JRN-restore-history`). 매핑·원장·상한의 단일 소스는 [`mockups/README.md`](mockups/README.md)
- 디자인 시스템 §4 컴포넌트: **12개** (사용됨: 12 / 미사용: 0)
- 공개된 문서: **20개** (`docs/` 전체, 허브에서 도달 가능 20/20)
- **건강 상태**: ⚠️ **위험 있음 — 제품 소유자(Product Owner) 미지정** · 수용된 위험 2건(`JRN-restore-history` 전체 미시각화 · `STP-sign-in` 미인증 상태 **와이어프레임** 미표현) · 여정 단위 목업 이관 대기 4건(원장·상한으로 래칫)

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

UX 산출물의 연결. 10개 화면(S01~S10)이 각각 와이어프레임(정보 구조)과 목업(디자인 시스템 적용 최종 형태)을 모두 갖습니다. **목업 칸은 그 화면의 공개 경로**입니다 — 여정 페이지로 이관된 화면은 페이지 안의 단계 앵커가, 아직 이관되지 않은 화면은 화면 단위 파일이 공개 경로입니다. 화면별 AC·가치·여정 매핑의 단일 소스는 [`mockups/README.md`](mockups/README.md)입니다.

| 화면 | 여정 (터치포인트) | 가치 | 와이어프레임 | 목업 (공개 경로) | 상태 |
|------|-------------------|------|--------------|------------------|------|
| S01 Credentials Setup | `JRN-connect-repo` | V6 | ✅ | ✅ [여정 페이지 #STP-grant-repo-access](mockups/JRN-connect-repo.html#STP-grant-repo-access) · [#STP-register-llm-key](mockups/JRN-connect-repo.html#STP-register-llm-key) | ✅ 완전 |
| S02 Home · Repositories | `JRN-connect-repo` | V1·V8 | ✅ | ✅ [여정 페이지 #STP-pick-target](mockups/JRN-connect-repo.html#STP-pick-target) | ✅ 완전 |
| S03 Connect Repository | `JRN-connect-repo` | V6·V8 | ✅ | ✅ [여정 페이지 #STP-pick-target](mockups/JRN-connect-repo.html#STP-pick-target) · [#STP-confirm-cost](mockups/JRN-connect-repo.html#STP-confirm-cost) | ✅ 완전 |
| S04 Analysis in Progress | `JRN-discover-features` · `JRN-follow-code-change` | V7·V8 | ✅ | ✅ [여정 페이지 #STP-leave-and-return](mockups/JRN-discover-features.html#STP-leave-and-return) · 화면 단위 (follow-code-change 이관 대기) | ✅ 완전 |
| S05 Cross-cutting Concerns | `JRN-discover-features` | V2·V4 | ✅ | ✅ [여정 페이지 #STP-review-landscape](mockups/JRN-discover-features.html#STP-review-landscape) | ✅ 완전 |
| S06 Discovery Strategy | `JRN-discover-features` | V1·V2 | ✅ | ✅ [여정 페이지 #STP-tune-strategy](mockups/JRN-discover-features.html#STP-tune-strategy) | ✅ 완전 |
| S07 Feature Candidates | `JRN-discover-features` · `JRN-follow-code-change` | V1 | ✅ | ✅ [여정 페이지 #STP-sift-candidates](mockups/JRN-discover-features.html#STP-sift-candidates) · 화면 단위 (follow-code-change 이관 대기) | ✅ 완전 |
| S08 Feature · Acceptance | `JRN-review-feature` · `JRN-understand-feature` · `JRN-follow-code-change` | V3·V4 | ✅ | ✅ 화면 단위 (이관 대기) | ✅ 완전 |
| S09 Feature · Dependencies | `JRN-review-feature` · `JRN-understand-feature` · `JRN-follow-code-change` | V5 | ✅ | ✅ 화면 단위 (이관 대기) | ✅ 완전 |
| S10 LLM-assisted Edit | `JRN-review-feature` · `JRN-understand-feature` · `JRN-follow-code-change` | V3·V4·V7 | ✅ | ✅ 화면 단위 (이관 대기) | ✅ 완전 |

> 여정 열이 화면 소유를 보여줍니다 — **S01~S03(connect-repo)·S05~S06(discover-features)만 여정 하나의 전용 터치포인트**이고 나머지 5개는 2~3개 여정이 공유합니다. 공유가 막는 것은 *그 여정의 이관*이 아니라 *원본 `sNN` 파일의 삭제*뿐입니다 — 여정 페이지의 화면은 복제본이 아니라 그 자체가 원본이고, 같은 화면이 여러 여정에 각자의 맥락으로 등장하는 것은 중복이 아니기 때문입니다. 그래서 이관은 여정 단위로 진행하고, 원본 삭제만 마지막 소비 여정까지 미룹니다(확정: 2026-08-31, reconciler `rct_20260831-0001`). 이 열은 이관 순서의 제약이 아니라 **원본 삭제 시점의 제약**입니다.

목업은 디자인 시스템([`design-system.md`](design-system.md))의 토큰·컴포넌트를 각 HTML 파일에 인라인 CSS로 적용합니다 (와이어프레임 SVG처럼 화면별 단독 파일). 디자인 시스템 §4의 12개 컴포넌트(Button·Tag·Input·Card·Step·Tabs·Bottom tab bar·Code block·Icon container·Segment selector·Metric grid·Progress bar)는 모두 1개 이상의 목업에서 사용됩니다. 화면별 §4 컴포넌트 사용 매핑과 §4 외 요소(Section title 타이포 역할, 화면 전용 의존성 그래프)의 단일 소스는 [`mockups/README.md`](mockups/README.md)의 "Mockup index"입니다.

## e2e 매핑

AC(`docs/prd`) ↔ e2e spec(`e2e/tests/` 최상위 `*.spec.ts`)의 1:1 매핑과 그 예외·공백의 단일 소스.
reconciler 모델 `tbm_feature-doc-ac-e2e`가 이 절을 실제 파일 상태와 대조한다.

**규약**

- **매칭 단위**는 `e2e/tests/` 최상위 `*.spec.ts`다. `e2e/smoke.sh`(HTTP 스모크)·`playwright.config.ts`·`package.json`은 매칭 단위가 아니다.
- 각 spec은 첫 줄에 `// 검증 AC: ACx.y`를 **정확히 1개** 선언한다. 이 선언이 매핑의 확인 지점이다.
- 파일명은 `ac<major>-<minor>-<slug>.spec.ts` — 파일명과 헤더 선언이 서로를 교차 확인한다.
- spec은 각자 자기 stub 사용자로 로그인한다(`/api/auth/login?as=<handle>`). 설치·키는 사용자 단위 상태이고 spec 파일은 병렬 워커에서 한 배포를 공유하므로, 신원을 공유하면 한 spec이 다른 spec의 상태를 무너뜨린다. `ac4-8`만 UI의 "Sign in with GitHub" 버튼(기본 `stub` 사용자)을 소유한다.
- 셋업을 위해 다른 AC의 화면을 경유하는 것은 검증으로 세지 않는다 — 헤더에 선언된 AC만 그 파일의 검증 대상이다.
- **배포 전역 상태는 소유가 아니라 임대한다.** 사용자 단위 상태(App 설치·LLM 키)는 spec마다 자기 stub 신원으로 격리하지만, 분석 워커의 replica 수처럼 배포 전체에 걸린 것은 신원으로 나눌 수 없다. 그런 자원은 ① `deploy/e2e/`에서 **비활성(0)** 을 기본값으로 두고, ② 필요한 spec이 **자기 블록 안에서만** 켠 뒤 `finally`에서 반드시 0으로 되돌리며, ③ `playwright.config.ts`가 `workers: 1`로 spec 파일 간 동시 실행을 막는다. ④ 임차인은 **켜져 있는 동안 워커가 큐 전체를 드레인한다**는 잔여 효과를 받아들인다 — 그러므로 **자기가 만들지 않은 job에 대해서는 단정하지 않는다**(`ac4-5`가 자기 분석 id로만 로그를 세는 이유). scale·정착 대기 핸들은 `e2e/support/cluster.ts`에 한 벌만 두고 공유한다(`e2e/tests/` 최상위가 아니므로 매칭 단위가 아니다). 현재 임차인: `ac4-5`(토폴로지) · `ac1-5`(진행 표현). (도입: `rct_20260807-0002` — 워커가 켜져 있으면 큐를 수 초 안에 비워 `ac1-1`의 `Queued` 단정과 경합한다. 단독 소유 → 임대로 일반화: `rct_20260813-0001`.)

**매핑 (8건)**

| AC | 전용 spec | 이 spec이 검증하는 것 | 자동화 밖 잔여 |
|----|-----------|----------------------|----------------|
| AC1.1 저장소 연결·분석 트리거 | `ac1-1-repository-connect-and-trigger.spec.ts` | 홈 저장소·분석 목록 → 접근 범위 밖 타깃 거부(미큐잉·복구 경로) → pre-flight 추정 → 트리거 후 `Queued` | 실제 GitHub 저장소 fetch·브랜치 해석 |
| AC1.2 횡단 관심사 자동 추출·문서화 | `ac1-2-cross-cutting-concerns.spec.ts` | 워커 실행 전 문서 부재(404 — "아직 실행 안 됨"과 "찾은 게 없음"을 구분) → 워커가 2단계를 실행하면 AC1.2의 **5축이 모두** 있는 문서 생성 → **모든 항목이 분석된 저장소 안의 경로를 근거로 든다**(근거 유효성은 문서 자신이 아니라 스캔 대상 기준으로 판정) → S05가 그 문서를 그림 → S04→S05 진입 → **재분석 시 `unchanged` + 비교 대상 명시**(결정성 조항) | **LLM 산출물의 의미적 품질** — stub 모드에서 추출 결과는 결정적 더블이므로, "추출된 항목이 이 저장소를 잘 설명하는가"는 검증하지 않는다. 검증하는 것은 구조·근거·결정성이다. 실 프로바이더 호출 자체도 stub 경로 밖 |
| AC1.5 비동기 실행·진행 가시성 | `ac1-5-async-progress-and-partial-retry.spec.ts` | 워커 실행 전 S04가 5단계를 `대기 중`으로 표시 → 워커가 `fetch`·`cross_cutting`을 실행하면 `2 of 5`·40%·실측 `766 files · 2.2 MB` → **새로고침 후 동일**(시나리오 5) → S02 카드 `step 2 of 5` → S04 진입 → 실패 단계의 사유 표시와 `이 단계만 다시 시도` → 재시도 후 그 단계만 새 시각으로 재실행되고 나머지 단계·다른 분석은 불변(시나리오 6) | **재시도가 성공으로 끝나는 왕복** — stub의 실패 원인(없는 브랜치)은 결정적이라 2차 시도도 같은 이유로 실패한다. 재실행 사실은 타임스탬프 갱신으로만 단정한다. 이미 성공한 단계를 보존한 채 실패 단계만 리셋하는 계약은 `backend/tests/progress.rs`가 별도로 지킨다. **실측 누적 비용**은 AC4.6 미구현이라 화면에 없다 |
| AC4.1 GitHub App 설치를 통한 접근 연결 | `ac4-1-github-app-connection.spec.ts` | 설치 전 최소 권한(`contents:read`·`metadata:read`) 안내 → 연결 → 설치 상태·계정·접근 가능 저장소 수 | 실제 GitHub에서의 설치·저장소 범위 변경·설치 해제 라운드트립, 설치 토큰의 단기 만료 |
| AC4.2 LLM API Key 등록·교체·폐기 | `ac4-2-llm-key-lifecycle.spec.ts` | 검증 실패 키 거부 → 등록 → 다른 제공자로 교체 → 폐기 후 신규 호출 차단(`/api/llm-keys/preflight`) | 실제 제공자 호출 위임과 누적 호출/실측 비용 표시(분석 파이프라인 선행) |
| AC4.3 자격증명의 안전한 사용 정책 | `ac4-3-credential-safety.spec.ts` | 마스킹 식별자만 표시 · 렌더된 문서·자격증명 응답에 평문 부재 · 감사 이력(`llm_key.register`) 사용자 조회 | 운영 로그·오류 메시지의 평문 점검(클러스터 로그 수집 필요). 응답 본문 수준 평문 검사는 `e2e/smoke.sh`가 별도로 지킨다 |
| AC4.5 k8s 배포 및 워크로드 분리 | `ac4-5-worker-workload-separation.spec.ts` | 워커 0개에서 API 4개 엔드포인트 200 + 큐 보존(시나리오 7) → 워커 2개로 확장 시 두 파드 모두 기동, 대기 중이던 4건 전부 드레인, 각 건 **정확히 1회** 클레임, `fetch` 단계 실제 실행(시나리오 8) | 처리량의 **비례적** 증가 측정(부하 프로파일 필요). 동시 클레임의 분리성은 `backend/tests/worker.rs`(워커 6개 경합)가 별도로 지킨다 |
| AC4.8 GitHub OAuth 인증·세션 | `ac4-8-signin-and-session.spec.ts` | 미인증 시 보호 API 401 → 로그인 → `/api/me` 동일 사용자 · 재로그인 시 계정 미중복 → 로그아웃 시 세션 즉시 무효화 | 실제 GitHub OAuth 동의 화면 왕복 |

> "자동화 밖 잔여"는 **예외 등재가 아니다**. 해당 AC는 이미 전용 파일을 가지므로 1:1 매핑을 충족하며, 이 칸은 stub 모드 e2e가 닿지 못하는 부분을 숨기지 않고 남겨 둔 기록이다.

**예외 목록 (0건)**

현재 등재된 예외는 없다. 예외는 *e2e로 자동 검증하는 것이 곤란한* AC를 위한 것이며(실제 외부 계정 상태 왕복, 운영 로그 점검, 워커 강제 종료·수평 확장, LLM 산출물의 비결정적 품질 등), **"아직 구현되지 않았다"는 예외 사유가 아니다** — 미구현 AC는 아래 공백 목록에 남아 계속 gap으로 계수된다. 예외를 등재할 때는 AC별로 사유와 대체 검증 수단(수동 QA, `cargo test`, `e2e/smoke.sh` 등)을 함께 적는다.

**공백 (16건) — 구현 미착수, 위 "구현 수렴 로드맵"이 닫는다**

전용 spec을 쓸 화면·라우트가 아직 없어 비어 있는 AC. 자매 모델 `tbm_feature-doc-docs-impl`이 추적하는 미구현 16건과 정확히 같은 집합이며, 각 슬라이스가 구현을 얹을 때 그 슬라이스가 spec도 함께 만든다.

| 공백 AC | 닫을 슬라이스 |
|---------|---------------|
| AC1.3 탐색 전략 · AC1.4 feature 후보 · AC4.6 관측·비용 | 4b (AC4.6은 7에서 마감) |
| AC2.1~AC2.6 feature 표현·의존성 | 5 |
| AC3.1~AC3.5 문서 CRUD·이력·충돌 | 6 |
| AC4.4 모바일 우선 · AC4.7 분석 작업 격리 | 7 |

> 구현 시 예외 등재 후보(모델 정의가 예시로 든 것): ~~AC4.5의 워커 강제 종료·수평 확장~~(3a에서 자동화됨 — 등재하지 않았다), AC4.6의 비용·관측 수치, AC4.7의 격리 검증, ~~AC1.2~~(4a에서 구조·근거·결정성을 자동 검증했다 — 남는 것은 의미적 품질 한 갈래뿐이라 AC 전체를 면제하는 것은 과잉이고, 등재하지 않았다)·AC1.3~AC1.4의 LLM 산출물 품질. 판단은 각 슬라이스의 계획 단계에서 하고, 그 전에는 미리 등재하지 않는다.

**집계**

- AC: **24** · 예외: **0** · 매핑된 AC(전용 spec 보유): **8** · 공백: **16**
- 매칭 단위 파일: **8** — 전부 정확히 1개 AC 선언(중복 선언 0, 고아 파일 0). `e2e/support/`는 최상위가 아니므로 매칭 단위에 들지 않는다
- 항등식 `(AC 24 − 예외 0) = 매핑 파일 8 + 공백 16` ✅

기계 확인:

```bash
ls e2e/tests/*.spec.ts | wc -l                               # 매칭 단위 파일 수
grep -h '^// 검증 AC:' e2e/tests/*.spec.ts | sort            # 파일별 선언 (중복·누락 확인)
grep -rohE 'AC[0-9]+\.[0-9]+' docs/prd | sort -u | wc -l     # AC 총수
```

## 배포와 공개 범위

- **공개 URL**: https://dlddu.github.io/feature-doc/
- **배포 방식**: GitHub Pages — Deploy from a branch (`main` + `/docs`)
- **저장소 공개 여부**: public. `docs/` 아래 문서는 전부 이미 공개 상태였다
  (이번 작업으로 새로 노출된 문서는 **없다** — 읽을 수 있게 만들었을 뿐이다).
- **비공개 유지 문서**: 없음. 전 문서가 `docs/` 아래에 있고 공개 대상이다.

### 배포 골격

| 파일 | 역할 |
|------|------|
| `docs/index.html` | 진입점 허브. 문서 3구획 + 플로우별 목업 갤러리 |
| `docs/reader.html` | 마크다운 뷰어. `reader.html?doc=<경로>` 로 문서를 렌더링 |
| `docs/.nojekyll` | Jekyll 처리 우회 |

`.nojekyll` 이 있으면 Pages 는 `.md` 를 렌더링하지 않고 그대로 내보낸다.
그래서 마크다운을 `docs/` 에 두는 것만으로는 링크를 누른 사람에게 파일이 **내려받아질 뿐**이고,
`reader.html` 이 그 렌더링을 맡는다. 문서 링크는 항상 `reader.html?doc=` 를 거쳐야 한다.

### 리더 사용 시 유의점

- 리더는 `fetch()` 를 쓰므로 `file://` 로 직접 열면 브라우저 보안 정책에 막힌다.
  로컬 확인은 `cd docs && python3 -m http.server` → `http://localhost:8000/reader.html?doc=...`.
- **목업은 이 제약을 받지 않는다.** 여정 페이지(`mockups/JRN-*.html`)와 화면 단위 파일
  (`mockups/sNN-*.html`) 모두 단독 정적 파일이라 `file://` 로 그대로 열려야 하고,
  안 열리면 그건 진짜 결함이다.
- 리더의 `CONFIG.journeyPage` / `journeySection` / `flowHref` 는 **이관 여부에 따라 갈린다** —
  여정 페이지가 있는 여정(`JRN-connect-repo`·`JRN-discover-features`)은 그 페이지로, 아직 화면 단위인 여정은 허브의
  `#flow-NN` 구획으로 보낸다. 허브 구획에도 대응하지 않는 여정(`JRN-understand-feature` ·
  `JRN-follow-code-change` · `JRN-restore-history`)은 링크를 만들지 않는다. 이관이 끝나면
  `journeySection` 은 비고 `journeyPage` 만 남는다.
- 리더는 `### \`STP-xxx\` 단계명` 형태의 제목에서 백틱 안의 값을 그대로 앵커 id로 쓴다.
  그래서 `reader.html?doc=user-journey/JRN-connect-repo.md#STP-confirm-cost` 같은 **단계 딥링크**가
  목업 전환 이전에도 이미 동작한다.

## 위험 진단

### 🔴 고아 가치 (소유자 없는 가치)
- V1 ~ V8 **전체** — [`values.md`](values.md)의 "제품 소유자"가 TBD이므로 8개 가치 모두 책임자 미지정 상태입니다.
- **권장 조치**: 제품 소유자를 지정한 뒤 가치 문서를 갱신하세요.

### 🟡 검토가 필요한 의사결정
- **제품 명칭 미확정**: 현재 임시명 `FeatureDoc`을 사용 중. 확정 시 가치 문서/PRD/테스트 문서 파일명도 함께 갱신해야 합니다.
- **결정적 재현성의 한계**: AC1.2의 "결정적 재현 또는 차이 명시" 정책은 LLM 응답의 비결정성을 어디까지 허용할지에 대한 정책 결정이 필요합니다.
- **삭제된 feature의 보관 기간**: AC3.3에서 "일정 기간"으로만 표현됨. 운영 정책 결정 필요.

### 🟡 여정 단위 이관 진행 중 (공개 축)

- **결정: 여정 단위로 전환한다** (2026-08-30, reconciler `rct_20260830-0001`). 오래 미결정으로
  남아 있던 항목이다. 선행 조건이던 안정적 단계 식별자가 같은 날 여정 문서 재작성으로
  확보(`JRN-`/`STP-`)되면서 결정을 미룰 이유가 사라졌고, 방식은 **여정 하나씩 옮기고
  그때마다 검증**한다. 규약은 [`mockups/README.md`](mockups/README.md) 「여정 페이지 규약」,
  기계 검사는 `tools/check-journey-mockup.py`(정적 R0~R10) + `tools/check-journey-prototype.js`(DOM P1~P7).
- **진행: 2 / 5** — `JRN-connect-repo`(S01~S03 흡수, 화면 파일 3개 삭제) · `JRN-discover-features`(S04~S07 흡수, 전용 화면 파일 S05·S06 2개 삭제 — S04·S07 은 `JRN-follow-code-change` 가 아직 써서 원본이 남는다) 이관 완료. 이관은 화면을 옮겨 붙이는 일이 아니라 **프로토타입으로 다시 쓰는 일**이다 — 여정 페이지는 실제 입력 요소와 상태 변형을 갖고 모든 단계가 화면 안의 행동으로 전진해야 하며, 그 세 가지는 정적 검사로 판정할 수 없어 DOM 하네스가 CI 에서 굴린다.
  남은 4개는 [`mockups/README.md`](mockups/README.md) 의 **이관 대기 원장**(상한 7)에 있다.
  - 전환으로 얻은 것: 한 페이지에서 단계를 눌러 넘기는 **흐름 체험**, `#STP-*` 딥링크의
    목업 쪽 대응, 그리고 여정↔목업 정합성이 산문이 아니라 **CI 게이트**가 된 것.
  - 남은 전환의 선결 과제는 크기가 아니라 **화면 공유**다. `S04`·`S07` 은 두 여정이,
    `S08`~`S10` 은 세 여정이 터치포인트로 쓰므로, 흡수하며 원본을 삭제하는 규약을
    그대로 적용할 수 없다. 공유 화면을 여정마다 복제해 담을지(규칙상 중복이 아니다)
    다르게 다룰지가 다음 슬라이스의 첫 판단이다.
  - `JRN-restore-history` 는 소스 화면 자체가 없어 S11 제작이 먼저이며, 그때까지는
    아래 「수용된 위험」의 여정 단위 예외로 남는다.

- **문서 → 화면 단방향 (화면 단위 파일 5개에 한해 남음)**: 화면 단위 목업
  (`mockups/sNN-*.html`)에서 그 화면이 속한 여정 문서로 **돌아오는 링크는 없다**.
  목업을 먼저 연 사람은 "왜 이 화면이 이렇게 생겼는지"로 이동할 방법이 없다.
  이 7개는 2~3개 여정이 공유하므로 복귀 링크는 단일 링크가 아니라 여정 목록이 된다.
  - **여정 페이지에서는 해소됐다** — `mockups/JRN-*.html` 은 머리에 여정 문서·목업 인덱스·
    허브로 가는 링크를 갖고, 각 단계가 자기 터치포인트와 연결 AC 를 함께 보여준다.
    이관이 끝나면 이 항목 자체가 사라진다.
  - 남은 7개를 고치는 일은 이관과 겹치므로 별도 작업으로 넣지 않았다.

### 🟢 검증 커버리지
- 미정렬 문서: **없음** ✅
- 무가치 PRD: **없음** ✅
- AC 없는 PRD: **없음** ✅
- 미연결 AC: **없음** ✅
- 미검증 AC: **없음** ✅
- 고아 테스트: **없음** ✅
- 고아 여정 (가치 미참조): **없음** ✅ — 여정 6개 모두 유효한 가치 식별자 참조
- 고아 목업 (여정 미선언): **5건** — 화면 단위 파일 `s04`·`s07`~`s10` 이 `data-journey` 를 선언하지 않는다. 이 5개는 2~3개 여정이 공유하는 화면이라 "정확히 1개 여정"을 선언하는 것이 **원리적으로 불가능**하고, 그래서 화면 단위로 남아 있는 것 자체가 위반이다. [`mockups/README.md`](mockups/README.md) 의 **이관 대기 원장**(상한 5)에 등재되어 래칫으로 관리되며, 이관이 끝나면 0이 된다. 여정 페이지 `JRN-connect-repo.html`·`JRN-discover-features.html` 은 각각 `data-journey` 를 정확히 1개 선언 ✅
- 시각화 누락 화면 (목업 없는 와이어프레임): **없음** ✅ — 10개 와이어프레임 모두 목업 보유
- 목업 페이지 없는 여정: **4건** — `JRN-discover-features` · `JRN-review-feature` · `JRN-understand-feature` · `JRN-follow-code-change` (이관 대기, 원장 관리). `JRN-restore-history` 는 예외 등재라 위반이 아니다
- 시각화 누락 단계 (mockup 없는 여정 단계): **1건** — `JRN-restore-history` 의 세 단계 전부(이력 화면 S11 후보가 와이어프레임·목업 모두 없음). `JRN-connect-repo` / `STP-sign-in` 은 2026-08-30 프로토타입 이관으로 목업 쪽이 해소됐고 와이어프레임 공백만 남았다. → 아래 "수용된 위험" 참조
- 시각화 없는 가치 (목업 없는 가치): **없음** ✅ — V1~V8 모두 1개 이상 목업이 시각화
- 임의 스타일 목업 (디자인 시스템 미사용): **없음** ✅ — 전 목업이 인라인 CSS로 디자인 시스템 적용
- 사용처 없는 컴포넌트: **없음** ✅ — 디자인 시스템 §4의 12개 컴포넌트 모두 사용됨
- 공개 안 된 문서 (`docs/` 밖 문서): **없음** ✅ — 20개 문서 전부 `docs/` 아래
- 리더 부재: **없음** ✅ — `docs/reader.html` 설치됨
- 끊긴 문서 링크: **없음** ✅ — 허브의 문서 링크 20건 · 목업 링크(여정 페이지 1 + 화면 단위 7) 전부 실재 파일로 이어짐. `docs/` 안의 상대 링크 dangling 0 은 `tools/check-journey-mockup.py` 가 CI 에서 확인한다
- 허브 불일치: **없음** ✅ — `docs/` 아래 md 20개 전부 허브에서 도달 가능
- 미정의 항목 사용 (§4에 없는 컴포넌트 참조): **없음** ✅ — 2026-05-27 검증에서 발견된 6건을 해소함. 실 컴포넌트 3개(Segment selector·Metric grid·Progress bar)는 §4.10~4.12로 정식 추가, Section title(§2.2 타이포 역할)·진행 링(§4.5 Step의 일부)·의존성 그래프(화면 전용 inline SVG)는 `mockups/README.md` 인덱스에서 "§4 외 요소"로 재분류

## 목업↔구현 대조 범위

[`tools/check-mockup-render.py`](../tools/check-mockup-render.py) 가 CI 에서 읽는 표다. 목업이 시각의
SSOT 이므로 판정은 "구현이 목업과 어긋나는가"이고, 어긋남은 **구현을 고치거나 아래 편차 원장에
등재하거나** 둘 중 하나로만 닫힌다.

전 화면을 한꺼번에 대조하지 않는 이유는 대조 자체가 선결 결정을 요구하는 쌍이 있기 때문이다 —
구현이 목업 단계 2개를 한 화면으로 **병합**했거나(`CredentialsSetup`) 목업 단계 1개를 두 화면으로
**분할**한(`HomeRepositories` + `ConnectRepository`) 경우, 카피를 어느 화면에 두어야 하는지는
정합성 문제가 아니라 제품 결정이다. 그 결정이 서기 전까지는 「대조 보류」에 두고 **상한 래칫**으로
붙잡아 둔다(면제가 아니다 — 늘면 CI 가 실패한다).

### 활성 대조 대상

| 구현 화면 | 목업 | 대조 단계 |
|-----------|------|-----------|
| `frontend/src/ConnectRepository.tsx` | `docs/mockups/JRN-connect-repo.html` | `STP-pick-target` · `STP-confirm-cost` |

### 대조 보류

| 구현 화면 | 목업 | 대조 단계 | 보류 사유(선결 결정) | 해소 시점 |
|-----------|------|-----------|----------------------|-----------|
| `frontend/src/CredentialsSetup.tsx` | `docs/mockups/JRN-connect-repo.html` | `STP-grant-repo-access` · `STP-register-llm-key` | 구현이 목업의 두 단계를 한 화면으로 병합했다. 화면을 쪼갤지(=목업을 따를지) 합친 채로 둘지가 제품 결정이며, 그 결정 없이는 어느 카피가 어느 화면 몫인지 정할 수 없다 | 화면 분할 여부 결정 시 |
| `frontend/src/HomeRepositories.tsx` | `docs/mockups/JRN-connect-repo.html` | `STP-pick-target` (목록부) | 구현이 목업의 한 단계를 목록 화면과 연결 화면 둘로 분할했다. 위와 같은 결정의 뒷면이다 | 〃 |
| `frontend/src/AnalysisProgress.tsx` | `docs/mockups/s04-analysis-progress.html` | (화면 단위 목업 — 단계 없음) | 대응 목업이 아직 여정 페이지로 이관되지 않아 `data-step` 이 없다. 이관 전에는 단계 단위 대조가 성립하지 않는다 | `JRN-*` 이관 시 |

> 대조 보류 상한: **3**. 늘면 CI 실패 — 새 화면은 활성으로 들어오거나 보류 한 칸을 비워야 한다.

## 알려진 목업↔구현 편차

구현이 대응 목업과 어긋나는 지점을 **전부** 여기에 남겨 후속 슬라이스의 입력이 되게 한다. 숨기지 않는
것이 이 표의 목적이고, 등재는 면제가 아니라 부채 기록이다. 유형은 셋이다.

- **데이터 부재** — 목업이 전제하는 값이 아직 파이프라인에 없다. 지어내지 않고 미룬다.
- **제3자 소유** — 목업(여정 프로토타입)이 여정 완결성을 위해 그리지만 실제 화면은 GitHub 등
  외부가 소유한다. 구현은 리다이렉트하며, 구조적으로 수렴하지 않는다.
- **분할·병합 미결정** — 위 「대조 보류」의 선결 결정에 묶여 있다.

각 행의 **백틱 토큰만 기계 대조 대상**이다(나머지 산문은 사람용). 목업 측 토큰은 현재 목업에,
구현 측 토큰은 그 대상 파일에 실재해야 한다 — 삭제된 목업을 근거로 남은 **공전 행**을 이 규칙이
잡는다. `(단계 전체)` 가 붙은 행은 그 단계 전체를 대조에서 뺀다.

| 대상 | 목업이 표현하는 것 | 현재 구현 | 유형 | 사유 | 해소 시점 |
|------|--------------------|-----------|------|------|-----------|
| `docs/mockups/JRN-connect-repo.html` | `STP-grant-repo-access` (단계 전체) | 없음 — `CredentialsSetup.tsx` 가 `Connect GitHub App` 으로 github.com 설치 화면에 리다이렉트한다 | 제3자 소유 | GitHub App 설치 동의(범위 선택·저장소 체크·`Install & Authorize`)는 GitHub 이 소유하는 화면이다. 여정 프로토타입은 흐름이 끊기지 않게 그것까지 그리지만, 제품이 그 화면을 렌더링하면 그것이야말로 사실과 다르다. **구조적 편차이며 해소되지 않는다** | 해소되지 않음(구조적) |
| `docs/mockups/JRN-connect-repo.html` | `STP-sign-in` (단계 전체) | `CredentialsSetup.tsx` 의 미인증 분기(`Sign in with GitHub`) | 분할·병합 미결정 | 이 단계는 `data-screens` 가 비어 있어 대응 화면 ID 가 없다. 로그인 상태에 화면 ID(S11 이후)를 부여할지가 선결이며, 「수용된 위험」의 와이어프레임 공백 항목과 같은 뿌리다 | 로그인 화면 ID 부여 시 |
| `docs/mockups/JRN-connect-repo.html` | `STP-register-llm-key` (단계 전체) | `CredentialsSetup.tsx` 가 `STP-grant-repo-access` 와 한 화면으로 병합해 그린다 | 분할·병합 미결정 | 「대조 보류」의 `CredentialsSetup` 행과 같은 결정에 묶여 있다. 화면을 쪼개기로 하면 이 단계의 카피(`분석 비용을 낼 키` · `저장하고 계속` · `나중에 하기`)가 그대로 판정 대상이 된다 | 화면 분할 여부 결정 시 |
| `frontend/src/HomeRepositories.tsx` | `로그아웃` | 없음 — 화면에 로그아웃 액션이 없다 | 데이터 부재 | 백엔드는 `POST /api/auth/logout` 을 이미 갖고 있고(AC4.8 e2e 가 그 경로로 검증한다) **UI 진입점만 없다**. 목록 화면을 목업대로 되돌릴지가 정해지면 앱바에 붙는다 | 화면 분할 여부 결정 시 |
| `frontend/src/HomeRepositories.tsx` | `아직 분석한 저장소가 없어요. 아래에서 첫 저장소를 연결해 보세요.` | `아직 접근할 수 있는 저장소가 없어요. GitHub App을 연결하고 분석할 저장소를 고르면 여기에 나타납니다.` | 분할·병합 미결정 | 구현의 빈 상태는 "분석 이력 없음"이 아니라 "App 설치 범위에 저장소 없음"을 가리킨다 — 목록 화면이 분리돼 있어 안내가 달라졌다 | 화면 분할 여부 결정 시 |
| `frontend/src/HomeRepositories.tsx` | 없음 — 여정 페이지에는 메트릭 그리드가 없다 | 메트릭 그리드 `Repos` · `Analyses` · `Est. cost` | 분할·병합 미결정 | 2026-08-30 여정 이관으로 목업에서 메트릭 그리드가 사라졌다(구 `s02-*.html` 의 `Features`·`Spend` 는 파일과 함께 삭제). 구현의 그리드를 없앨지는 화면 분할 결정과 함께 판단한다 | 화면 분할 여부 결정 시 |
| `frontend/src/HomeRepositories.tsx` | 없음 — 여정 페이지에는 탭바가 없다 | 하단 탭바 4슬롯(`Repos` 현재 · `Keys` 이동 · `Activity` · `Settings` 비활성) | 분할·병합 미결정 | 위와 같은 뿌리. 탭바가 남는다면 비활성 2슬롯은 각 목적지 화면이 생기는 슬라이스에서 열린다 | 화면 분할 여부 결정 시 |
| `frontend/src/ConnectRepository.tsx` | `새 저장소 연결` · `얼마나 들까요` · `분석은 이 버튼으로만 시작돼요. 자동으로 시작되는 경로는 없습니다.` | `Connect a repository` — 한 화면이 두 단계를 덮으므로 제목이 하나뿐이다 | 분할·병합 미결정 | 목업은 `STP-pick-target` 과 `STP-confirm-cost` 를 각각 제 화면으로 그리고, 구현은 둘을 한 화면의 두 단계(추정 → 시작)로 합쳤다. 화면을 쪼개면 두 제목이 각자 자리를 얻는다 | 화면 분할 여부 결정 시 |
| `frontend/src/ConnectRepository.tsx` | `Pre-flight` | `New Repository` | 분할·병합 미결정 | 위와 같은 뿌리 — 앱바 제목도 하나뿐이다 | 화면 분할 여부 결정 시 |
| `frontend/src/ConnectRepository.tsx` | 브랜치 선택 목록 `main` · `develop` · `release/2026-08` | 자유 입력 + placeholder `기본 브랜치` | 데이터 부재 | 저장소의 실제 브랜치 목록을 주는 API 가 없다. 목업의 셋은 예시 값이므로 그대로 하드코딩하면 거짓 목록이 된다 | 브랜치 목록 API 가 생기는 시점 |
| `frontend/src/ConnectRepository.tsx` | `저장소 URL 형식이 올바르지 않아요.` · `형태로 입력해 주세요. 큐에 등록하지 않았습니다.` | 서버가 돌려준 오류 메시지를 그대로 표시 | 데이터 부재 | 클라이언트측 URL 형식 검증이 없다. 지금 문구를 심으면 검증 없이 문구만 생기므로 검증과 함께 넣는다 | 입력 검증을 붙이는 슬라이스 |
| `frontend/src/ConnectRepository.tsx` | `이 저장소에는 접근할 수 없어요 — App 설치 범위 밖입니다.` · `설치 범위에 추가하기` | `No access` 배지 + `App 설치 범위 관리` (문구에 대상 저장소 이름을 넣어 더 구체적) | 데이터 부재 | 의미는 같고 문구만 다르다. 구현 쪽이 저장소 이름을 포함해 더 구체적이라 어느 쪽으로 수렴할지는 카피 결정이 필요하다 | 카피 확정 시 |
| `frontend/src/ConnectRepository.tsx` | `분석을 시작하려면 LLM Key가 필요해요. 등록을 마치면 이 자리로 돌아옵니다.` · `키 등록하기` | 없음 — 키 미등록 상태 안내가 이 화면에 없다 | 데이터 부재 | 키 존재 여부를 이 화면에서 조회하지 않는다. `CredentialsSetup` 이 키를 요구하므로 지금은 도달 자체가 드물지만, 목업은 이 자리로 돌아오는 경로를 그린다 | 키 상태 조회를 붙이는 슬라이스 |
| `frontend/src/ConnectRepository.tsx` | `접근 가능한 저장소예요. 비용을 확인하고 시작할 수 있습니다.` | `Ready` 배지 + 추정 카드 자체 | 데이터 부재 | 같은 사실을 문장이 아니라 배지·카드로 표현한다 | 카피 확정 시 |
| `frontend/src/ConnectRepository.tsx` | `큐에 등록하지 못했어요 — 운영 측 문제입니다. 분석은 시작되지 않았고 비용도 발생하지 않았어요.` · `다시 시도` | `Error` 배지 + 서버 오류 메시지(전용 재시도 버튼 없음 — 같은 버튼을 다시 누른다) | 데이터 부재 | 큐 등록 실패를 다른 실패와 구분하는 오류 코드가 없다 | 오류 분류가 정의되는 시점 |
| `frontend/src/ConnectRepository.tsx` | `Save for later` 보조 액션 | `취소` — 저장을 뒷받침할 API 가 없어 가짜 동작을 만들지 않음 | 데이터 부재 | 구조적/데이터 사유는 앞 칸 참조 | 분석 초안 저장이 정의되는 시점 |
| `frontend/src/ConnectRepository.tsx` | 없음 — 여정 페이지에는 접근 여부 표시 칸이 없다 | `GitHub App` 칸의 `✓ has access` · `✕ no access` · `미확인` | 데이터 부재 | 구현은 pre-flight 를 눌러 확인하는 2단계라 "아직 확인 안 함" 상태가 존재한다. 목업 프로토타입에는 그 중간 상태가 없다 | 화면 분할 여부 결정 시 |
| `frontend/src/ConnectRepository.tsx` | 없음 | 진행 중 라벨 `확인 중…` · `시작하는 중…` | 데이터 부재 | 목업은 정적 프로토타입이라 네트워크 대기 상태를 그리지 않는다 | 목업에 로딩 상태를 넣는 시점 |
| `frontend/src/ConnectRepository.tsx` | 없음 | 화면 범례 `— discovery · connect repository`(앞에 `03` 배지) | 데이터 부재 | 목업의 화면 범례는 여정 페이지 이관 때 단계 머리말로 대체됐다. 구현에는 구 목업 시절의 범례가 남아 있다 | 범례 제거 여부 결정 시 |
| `frontend/src/AnalysisProgress.tsx` | `LLM Spend` · `Calls` — 값이 **실측** 누적(`$0.32` `of est. $0.80` · 호출 47회) | `Est. LLM Spend` · `Est. Calls`(pre-flight 추정치)만, "실측 누적은 아직 계측 전"을 함께 표기 | 데이터 부재 | 구조적/데이터 사유는 앞 칸 참조 | 실측 비용 회계(AC4.6, 슬라이스 7) |
| `frontend/src/AnalysisProgress.tsx` | 앱바 부제 `main · run #14` — 실행 회차 번호 | 부제에 브랜치 + 분석 id 앞 8자를 쓴다 — 분석에 회차 카운터가 없어 실제로 이 실행을 식별하는 값을 쓴다 | 데이터 부재 | 구조적/데이터 사유는 앞 칸 참조 | 회차 개념이 정의되는 시점(현재 계획 없음) |
| `frontend/src/AnalysisProgress.tsx` | 미완료 단계의 예상 소요(`~2m`) | 빈칸 — 단계별 소요 추정치가 없어 지어내지 않는다 | 데이터 부재 | 구조적/데이터 사유는 앞 칸 참조 | 파이프라인 단계가 실제로 구현되는 슬라이스 4~5 |

> 알려진 편차: **22건** · 미해소 편차 상한: **22**.
> 2026-08-31 `rct_20260831-0001` 에서 **현재 목업 기준으로 재기준선**했다 — 2026-08-30 여정 이관으로
> `s01`~`s03` 이 삭제되면서 구 표의 S02·S03 행 4건이 사라진 파일을 근거로 삼는 공전 행이 됐고,
> 행 수(7)와 캡션 집계(6)도 어긋나 있었다. 유형 구분과 기계 대조 형식은 이때 도입됐다.

## 수용된 위험

사용자가 인지하고 의도적으로 받아들인 위험. 검증 시 일반 위험 보고에서 제외하되 여기에 카운트로 남긴다.

| 위험 종류 | 대상 | 사유 | 수용 시점 |
|-----------|------|------|-----------|
| 🟡 시각화 누락 단계 | `JRN-restore-history` 전체 — 변경 이력 화면(S11 후보) | 변경 이력 화면은 후순위 작업으로 분류. 지금은 와이어프레임/목업을 제작하지 않고, 향후 제작 시 `wireframes/README.md`·`mockups/README.md`·본 추적 문서를 함께 갱신한다 (`JRN-restore-history.md` 상단 경고 및 `user-journey/README.md §6` 갱신 정책 참조). 2026-08-30 여정 재작성으로 대상 표기가 "플로우 4 시나리오 D"에서 여정 식별자로 바뀌었다 — 위험 자체는 동일하다. | 2026-05-27 |
| 🟡 시각화 누락 단계 | `JRN-connect-repo` / `STP-sign-in` — 로그인(미인증) 상태 | 구현(GitHub OAuth 게이트)과 여정 문서(`STP-sign-in`)에는 로그인 단계가 있으나, S01 와이어프레임·목업은 로그인 후 상태만 표현한다. 2026-08-30 프로토타입 이관으로 **목업 쪽 공백은 해소됐다** — `JRN-connect-repo` 페이지의 `STP-sign-in` 단계는 실제로 눌러 전진하는 로그인 화면을 갖는다(규칙 5c 가 모든 단계에 화면 내 행동을 요구하므로 "시각화 없음" 자리를 남길 수 없다). **남은 공백은 와이어프레임 쪽이다** — `gen-wireframes.js` 의 S01 정의는 여전히 로그인 후 상태만 표현한다. 후속 제작 시 S01 와이어프레임에 미인증 상태를 추가하고 `wireframes/README.md` 의 S01 행 AC 에 AC4.8 을 더한 뒤 본 문서에서 해제한다. | 2026-07-12 |

> 수용된 위험: **2건** (여정 단위 미시각화 1 · 와이어프레임 미표현 1).

## 변경 이력

| 시점 | 변경 내용 | 이전 상태 | 이후 상태 |
|------|-----------|-----------|-----------|
| 초기 생성 | 가치 문서 + PRD 4개 + 테스트 문서 4개 + 추적 문서 일괄 생성 | 문서 0개 | 가치 8, PRD 4, AC 23, 테스트 4 |
| user-journey 추가 | `docs/user-journey/` 추가 — README + 플로우 4개. 8개 가치 / 21개 AC / 10개 와이어프레임을 사용자 행동의 시간 축으로 재엮음 (신규 가치/AC 추가 없음) | 가치 8, PRD 4, AC 23, 테스트 4 | 좌동 + 사용자 여정 5개 |
| GitHub 인증 모델 변경 | GitHub PAT 등록 → GitHub App 설치 모델로 전환. AC4.1 재작성, AC1.1/AC4.3 보강, S01·S03 와이어프레임·테스트·사용자 여정 문서 동기화 (AC 수 변동 없음) | PAT 기반 자격증명 | GitHub App 설치 기반 |
| 목업 추가 | `docs/mockups/` 추가 — 화면별 단독 HTML 목업 10개(S01~S10, 디자인 시스템 CSS 인라인) + 인덱스 README. 디자인 시스템(`design-system.md`)이 참조하던 "별도 HTML mockup"의 실체를 구현. 신규 가치/AC/와이어프레임 추가 없음 | 와이어프레임 10, 목업 0 | 와이어프레임 10, 목업 10 |
| S02 진입점 보강 | 사용자 여정(`user-journey/01`)이 전제하던 S02의 "새 저장소 추가" 진입점이 와이어프레임·목업에 누락된 여정↔시각화 불일치를 해소. S02 섹션 헤더에 `+ New` 액션 추가 (목업 HTML + `gen-wireframes.js` S02 정의 + s02 SVG 재생성). 신규 화면/AC 추가 없음 | 여정↔S02 시각화 불일치 | 일치 ✅ |
| 2026-07-12 OAuth 인증 문서 편입 | 구현(#5·#6)으로 존재하던 GitHub OAuth 로그인·세션·계정을 PRD-4 범위·AC4.8로 편입, `test/04` 시나리오 11·12 추가, 여정 플로우 1(S01)에 로그인 단계 반영, README §5 매트릭스·카운트 갱신. "사용자 본인 인증 방식 미정의" 의사결정 해소(결정: GitHub OAuth, PRD-4 확장). 구현 수렴 로드맵(슬라이스 2~7) 신설. S01 로그인(미인증) 상태 미시각화는 수용된 위험으로 등재 (reconciler `rct_20260712-0001`) | AC 23 · 인증 비범위(문서↔구현 모순) | AC 24 · 인증 범위 편입(모순 해소) |
| 2026-08-02 슬라이스 2a(AC1.1 백엔드) 착수 기록 | AC1.1 저장소 연결·분석 트리거의 백엔드 enqueue 계약(`/api/repositories` · `POST /api/analyses/preflight` · `POST`·`GET /api/analyses`; `queued` 행 적재 · 범위 밖 타깃은 명확한 오류로 거부하며 미큐잉)이 PR #8(`dd5e6ee`, 2026-07-22 머지, reconciler `rct_20260715-0001`, `cargo test` + kind e2e 게이트)로 구현됨을 로드맵에 반영. 슬라이스 2를 2a(백엔드·완료)/2b(S02·S03 프론트·다음)로 분할. to-be(24개 AC)는 불변 — 메타 추적을 구현 현실에 일치시킨 갱신일 뿐 목표를 낮추지 않음 (reconciler `rct_20260802-0001`) | 로드맵 슬라이스 2 미착수 표기 (2a 구현 후 ~11일 미반영) | 2a 완료 · 2b 다음, AC1.1 백엔드 완료·프론트 잔여 |
| 2026-08-07 슬라이스 2b(AC1.1 프론트) 완료 | AC1.1의 프론트엔드 절반(S02 Home·Repositories, S03 Connect Repository)을 2a 백엔드 API 위에 구현. 여정 플로우 1(S01 → S02 → S03) 라우팅, pre-flight 추정을 트리거 전 필수 단계로 배치, 접근 범위 밖 저장소의 오류·복구 경로·미큐잉을 stub 모드 Playwright 스펙(`e2e/tests/s02-s03.spec.ts`)으로 검증. 목업이 전제하나 파이프라인 데이터가 없어 표현하지 못한 4건을 "알려진 목업↔구현 편차"로 등재. to-be(24개 AC) 불변 (PR #11, reconciler `rct_20260807-0001`) | AC1.1 백엔드 완료 · 프론트 잔여, 미구현 AC 20건 | AC1.1 완료, 미구현 AC 19건 · 다음 슬라이스 3 |
| 2026-08-07 슬라이스 3a(AC4.5 워크로드 분리) 완료 | 분석 큐를 리스 기반 클레임으로 만들고 `featuredoc-worker`를 **별도 k8s Deployment**로 분리(같은 이미지, 커맨드만 다름). 워커는 데이터베이스를 열지 않고 API의 `/internal` 라우트로 claim·heartbeat·단계 보고·finish 한다 — SQLite가 요구하는 단일 writer(`replicas:1` + `Recreate`)를 깨지 않으면서 워커만 수평 확장할 수 있게 하기 위한 선택이다. enqueue가 S04 목업의 5단계를 `analysis_stages`에 시드하고, 워커는 LLM이 필요 없는 1단계 `fetch`(저장소 트리 실측 → `766 files · 2.2 MB`)만 실행한 뒤 작업을 `awaiting_pipeline`으로 닫는다 — 2~5단계는 `pending`으로 남아 슬라이스 4~5가 채운다(미구현 단계를 완료로 표기하지 않음). test/04 시나리오 7·8을 `ac4-5-worker-workload-separation.spec.ts`가 kind에서 자동 검증(워커 0 → API 200·큐 보존 / 워커 2 → 전량 드레인·건당 정확히 1회 클레임). 트래커가 예외 후보로 적어 둔 "워커 강제 종료·수평 확장"은 자동화됐으므로 **예외로 등재하지 않았다**. 부수: 병렬 테스트가 같은 임시 DB 경로를 뽑아 간헐 실패하던 `tests/common` 결함 수정(`cargo test` 27건 → 44건). to-be(24개 AC) 불변 (reconciler `rct_20260807-0002`) | AC1.1 완료, 미구현 19건 · 매핑 5 · 공백 19 | AC1.1·AC4.5 완료, 미구현 18건 · 매핑 6 · 공백 18 · 다음 슬라이스 3b |
| 2026-08-13 슬라이스 3b(AC1.5 S04 진행 화면) 완료 | 3a가 적재해 둔 단계 데이터를 사용자에게 표현하고 실패 단계의 부분 재시도를 붙여 **AC1.5를 구현 완료**했다. 백엔드는 읽기 면(`GET /api/analyses/{id}` — 분석 + 단계 행, 소유자 스코프)과 재시도(`POST /api/analyses/{id}/stages/{key}/retry`)를 더했고, 재시도는 **큐 연산**으로 표현했다 — 실패한 단계 행만 `pending`으로 되돌리고 분석을 `queued`로 되돌리면 기존 claim/lease 경로가 재실행을 수행하므로 형제 단계의 측정값이 그대로 보존된다(실행 중인 lease 아래에서는 409로 거부). 프론트는 S04(`AnalysisProgress.tsx`, 진행 링·5단계·추정 비용)와 `#/analyses/{id}` 해시 라우트를 추가해 화면을 **주소 지정 가능**하게 만들었다 — 진행을 클라이언트에 두지 않으므로 새로고침이 곧 "앱 종료 후 복귀"(test/01 시나리오 5)의 관측이다. 부수: stub의 `repo_scan`이 브랜치를 무시하고 항상 성공하던 충실도 결함을 고쳐(없는 ref는 real 경로와 같은 `github tree rejected (404)`) 실패 경로가 사용자 입력만으로 도달 가능해졌고, 워커 replica 규약을 **단독 소유 → 임대**로 일반화했다(`e2e/support/cluster.ts`). 목업이 전제하나 데이터가 없는 3건(실측 비용·회차 번호·단계별 예상 소요)은 지어내지 않고 편차로 등재. to-be(24개 AC) 불변 (reconciler `rct_20260813-0001`) | AC1.1·AC4.5 완료, 미구현 18건 · 매핑 6 · 공백 18 | AC1.1·AC1.5·AC4.5 완료, 미구현 17건 · 매핑 7 · 공백 17 · 다음 슬라이스 4 |
| 2026-08-07 e2e 매핑 규약 도입 | AC ↔ e2e spec의 1:1 매핑을 문서·파일 양쪽에 확립. spec 헤더 `// 검증 AC: ACx.y` 선언 규약과 `ac<major>-<minor>-<slug>.spec.ts` 파일명 규약을 도입하고, AC4.8·AC4.1·AC4.2·AC4.3을 한 파일에 묶고 있던 `e2e/tests/s01.spec.ts`를 AC별 전용 spec 4개로 분리, `s02-s03.spec.ts`를 `ac1-1-repository-connect-and-trigger.spec.ts`로 리네임·선언. 본 문서에 "e2e 매핑" 절(매핑 표·예외 정책·공백 목록·집계·기계 확인 recipe)을 신설. 예외는 0건으로 두고 "미구현은 예외 사유가 아니다"를 명문화해 미구현 19건이 계속 gap으로 계수되게 했다. to-be(24개 AC) 불변 — e2e 테스트만 재배치하고 구현·PRD는 건드리지 않음 (reconciler `rct_20260807-0001`, 모델 `tbm_feature-doc-ac-e2e`) | spec 2개(AC 선언 0, 4-AC 묶음 1) · 매핑 문서 없음 | spec 5개(각 1 AC 선언) · 매핑 5 · 예외 0 · 공백 19 |
| 2026-08-14 슬라이스 4a(AC1.2 횡단 관심사 추출) 완료 | 3a가 시드하고 3b가 표현만 해 두었던 파이프라인 **2단계 `cross_cutting`을 실제로 실행**시켜 **AC1.2를 구현 완료**했다. 핵심은 **LLM 호출 경계 신설**(`backend/src/llm.rs`) — 나머지 코드베이스가 이미 쓰는 `Mode::Stub`/`Mode::Real` 이분법을 그대로 따르며, 4b의 AC1.3·AC1.4가 재사용할 자산이다. 실 경로는 Anthropic Messages API를 쓰되 **프리필 대신 structured outputs**(`output_config.format`)로 응답 모양을 강제하고 **sampling parameter를 보내지 않는다** — 현행 모델은 `temperature`를 400으로 거부하므로 "temperature 0으로 결정성 확보"는 성립하지 않는다. 결정성은 대신 **관측**한다: 정렬·절단된 경로 목록이라는 고정 입력 + 산출물의 `content_hash`를 직전 동일 대상 분석과 대조해 `first`/`unchanged`/`changed`를 명시한다(AC1.2의 "결정적으로 재현되거나 차이가 명시된다"). 1단계가 이미 읽고도 버리던 **파일 경로 목록을 산출물로 승격**해 2단계의 근거 원천으로 삼았고(블롭 본문을 받지 않는다), 산출물은 `analysis_documents`에 (분석,종류)당 1행으로 영속화된다 — 단계 재시도는 자기 행을 덮어쓴다. LLM 키는 `installation_token` 선례대로 claim 응답에 단명으로 실려 워커에 전달되며 저장·로깅되지 않는다(AC4.1·AC4.3). 프론트는 S05(`CrossCuttingConcerns.tsx`)와 `#/analyses/{id}/cross-cutting` 해시 라우트를 더해 문서를 **주소 지정 가능**하게 만들었다. **목업은 4범주지만 AC는 5축**이라 AC를 따랐고(PRD가 SSOT), 그 차이를 편차로 등재했다 — 편차 표에서 **구현이 아니라 목업이 뒤처진 첫 항목**이다. 부수: 재현성 비교가 `created_at`(초 단위)만으로 정렬해 **같은 초에 트리거된 재분석을 첫 분석으로 오판**하던 결함을 `rowid` 타이브레이커로 수정(`backend/tests/documents.rs`가 잡았다). 기존 단정 변경은 2건뿐이며 둘 다 이 슬라이스가 단계 수를 늘린 직접 귀결이다(`executableStages`, `1 of 5`→`2 of 5`). `cargo test` 53건 → 71건. to-be(24개 AC) 불변 (reconciler `rct_20260814-0001`) | AC1.1·AC1.5·AC4.5 완료, 미구현 17건 · 매핑 7 · 공백 17 | AC1.1·AC1.2·AC1.5·AC4.5 완료, 미구현 16건 · 매핑 8 · 공백 16 · 다음 슬라이스 4b |
| 2026-08-27 문서 포털 공개 | `docs/` 를 목업 갤러리에서 **문서 포털**로 확장. `.nojekyll` 이 있으면 Pages 가 `.md` 를 렌더링하지 않고 그대로 내보내므로, 지금까지 18개 문서는 링크를 눌러도 파일이 내려받아질 뿐 읽히지 않았다 — 이걸 해소하는 게 이번 변경의 전부다. ① `docs/reader.html` 추가(단일 파일 마크다운 뷰어, `design-system.md` v0.1 토큰으로 테마 적용, 문서 안의 `.md` 상대 링크를 리더 경유로 재작성, 플로우 문서에서 허브의 `#flow-NN` 구획으로 가는 링크 제공) ② `docs/.nojekyll` 추가 ③ 허브에 문서 3구획(제품 문서 9 · 사용자 여정 5 · 디자인 시스템/인덱스 4) 신설, 각 플로우 제목 옆에 여정 문서 링크를 붙여 읽는 경로와 눌러보는 경로를 같은 줄에 배치, hero 카피·요약 수치 갱신. **문서를 옮기지 않았고 새로 공개된 문서도 없다** — 전부 이미 `docs/` 아래 public 이었다. 목업의 여정 단위 전환과 목업→문서 복귀 링크는 범위에서 제외하고 위험으로 등재 | md 18개 공개되나 렌더링 불가 · 허브에서 도달 가능 0개 | md 18개 렌더링 가능 · 허브에서 도달 가능 18개 |
| 2026-05-27 design-doc 정합성 검증 | 4종 문서(가치/여정/디자인 시스템/목업) 정합성 검증. 위험 2건 발견 후 처리: ① 🟡 시각화 누락 단계(S11 변경 이력 화면) → "수용된 위험"으로 기록 ② 🟢 미정의 항목 사용 6건 → 실 컴포넌트 3개를 `design-system.md` §4.10~4.12로 추가하고 §4 헤더를 "12개"로 정정, 나머지 3개는 `mockups/README.md` 인덱스에서 "§4 외 요소"로 재분류. 부수 정정: `featuredoc-values.md`→`values.md` 오기, `user-journey/README.md §5` AC 커버리지(22→21개, 예외 AC4.5·AC4.7→AC3.2·AC4.4), `wireframes/README.md` AC 칸의 가치 표기 제거. `.claude/skills/`에 주입형 스킬 2개(ui-with-design-system, screen-with-mockup-and-design-system) 추가 | §4 컴포넌트 9 표기·미정의 참조 6건·검증 위험 미기록 | §4 컴포넌트 12 정합·미정의 참조 0·수용된 위험 1건 기록 |
| 2026-08-30 사용자 여정 행동 축 재작성 | 여정 문서 5개(README + 플로우 4개)를 **폐기하고** 7개(README + 여정 6개)로 재작성. 이전 판은 본문 뼈대가 `## 화면별 상세` 였고 단위 키가 `S01`~`S10`(와이어프레임 ID)이라, 화면을 합치거나 나누면 여정 문서 구조가 함께 깨지는 상태였다 — 본 문서가 2026-05-27부터 기록해 온 "안정적 단계 식별자 없음"이 그것이다. ① 단계를 화면이 아닌 **사용자 행동**으로 재분할하고 `JRN-<슬러그>`/`STP-<슬러그>` 식별자 부여 ② P1·P2 여정 분리 — 구 `03-feature-documents.md` 에 "페르소나 메모"로 묻혀 있던 P2 흐름을 `JRN-understand-feature` 로 독립 ③ 구 플로우 4를 트리거가 다른 두 여정(`JRN-follow-code-change` · `JRN-restore-history`)으로 분할 ④ 여정마다 진입 맥락·트리거·완료 기준(관찰 가능 이벤트)·분기표·**측정 지표**·변경 이력 신설 — 이전 판에는 지표가 5개 문서 통틀어 0건이었다 ⑤ 행동 축으로 자르니 **AC3.2**(자동 추출되지 않은 feature 직접 추가)가 `STP-add-missing` 으로 자연히 편입 — 이전 판에서 "본 여정이 다루지 않는 사용자 행동"으로 제외돼 있던 항목이다. 부수: 허브 문서 목록·플로우별 여정 링크, `reader.html` 의 `flowFromPath`/`journeySection`, 루트 README 트리, `mockups/README.md` 의 여정 참조 문구 갱신. **to-be(가치 8 · AC 24) 불변** — 신규 가치/AC/화면을 만들지 않고 같은 재료를 사용자 행동 축으로 다시 엮었다. 수용된 위험 2건은 대상 표기만 여정 식별자로 옮겼고 위험 자체는 그대로다 | 여정 5개 · 화면 축 · 단계 식별자 없음 · 지표 0건 · AC 커버 22/24(매트릭스 직접 기재 20) | 여정 7개 · 행동 축 · `JRN-`/`STP-` 식별자 · 지표 39건 · AC 커버 23/24 |
| 2026-08-30 여정 단위 목업 이관 슬라이스 1 | 오래 미결정이던 "목업을 여정 단위로 전환하는가"를 **전환한다**로 결정하고, `JRN-connect-repo` 하나를 **클릭되는 제품 프로토타입**으로 이관해 규약을 실증. ① 「여정 페이지 규약」을 `mockups/README.md` 에 신설 — 경로 `docs/mockups/JRN-<슬러그>.html`(별도 디렉터리에 두면 목업 변경 감지가 트리 해시를 보므로 잡히지 않는다), `data-journey` 1개 + `data-step` 선언, **식별자는 여정 문서가 원천이고 README 는 인용만 한다**(SSOT 이중화 금지), 단계→화면 매핑은 각 단계의 `터치포인트` 줄에서 파싱해 `data-screens` 와 대조 ② `docs/mockups/JRN-connect-repo.html` 신설 — 5단계 전부가 **화면 안의 행동으로 전진**하고, 텍스트·선택이 진짜 `<input>`/`<select>` 이며, 여정 문서 §4 분기표 7행의 상태(범위 밖 저장소·URL 오타·키 미등록·큐 등록 실패·재방문·이탈 후 이어하기·로그아웃)에 프로토타입 안에서 실제로 도달한다. 문서 메타는 기본 닫힌 보조 레이어로 내려 **연 직후 보이는 것이 제품**이 되게 했다. 화면 본문을 원본과 바이트 동일하게 고정하는 방식은 쓰지 않는다 — 지문으로 화면을 못 박으면 입력과 상태 변형을 넣을 길이 구조적으로 막힌다. 여정 페이지의 화면은 복제본이 아니라 **그 자체가 원본**이다 ③ 흡수된 `s01`~`s03` 삭제 — 이 셋은 `JRN-connect-repo` **전용** 터치포인트라 다른 여정의 이관을 막지 않는다 ④ 남은 화면 단위 7개를 **이관 대기 원장 + 상한 7 래칫**으로 등재 (면제가 아니라 래칫 — 원장 밖 위반·공전 행·상한 초과가 전부 CI 실패) ⑤ 정합성이 산문에서 **기계 게이트**가 됨 — 정적 체커 `tools/check-journey-mockup.py`(R0~R10)와 DOM 하네스 `tools/check-journey-prototype.js`(jsdom, P1~P7)를 전용 워크플로에서 함께 돌린다. 배선·입력·상태는 파일을 읽어서는 판정할 수 없어(끊긴 버튼과 살아 있는 버튼이 구분되지 않는다) 실제 DOM 에서 굴리며, 하네스의 기대값은 페이지가 아니라 **여정 문서에서 파싱**한다 ⑥ 허브·리더·프런트 주석의 경로 갱신, 허브 요약 수치의 자기모순 정정(`18 Documents` ↔ 문서 구획 `20 documents`). **여정·단계·AC·가치는 하나도 바뀌지 않았다** — 같은 흐름의 공개 경로를 여정 단위로 옮기고 눌러볼 수 있게 만들었을 뿐이다 (reconciler `rct_20260830-0001`) | 목업 10개 전부 화면 단위 · 여정별 목업 페이지 0 · `data-journey`/`data-step` 0건(정합성 기계 판정 불가) · 목업 전체에 폼 요소 0개 | 여정 페이지 1(프로토타입) + 화면 단위 7 · 이관 1/5 · 예외 1 · 규약과 원장과 프로토타입 충실도가 CI 게이트 |
| 2026-08-31 목업↔구현 대조를 기계 게이트로 | 이 레포의 목업↔구현 정합성은 지금까지 **산문**이었다 — 원장 표 하나가 전부였고, 그 표가 맞는지 확인하는 장치가 없었다. 2026-08-30 여정 이관(#22)으로 `s01`~`s03` 이 삭제되자 그 사실이 드러났다: 표의 S02·S03 행 4건이 **사라진 파일을 근거로** 남았고(공전 행), 행 수(7)와 캡션 집계(6)도 어긋나 있었다. ① 정적 체커 [`tools/check-mockup-render.py`](../tools/check-mockup-render.py)(python3 stdlib, 의존성 0) 신설 — M0 파싱 · M1 매핑 완비 · M2 토큰 1:1 · M3 카피 양방향 대조 · M4 원장 무결성(공전 행 0 · 캡션=행 수) · M5 래칫. 판정 대상 화면은 하드코딩하지 않고 **상단 주석에 목업 매핑을 가진 tsx** 로 발견하므로 새 화면이 생기면 자동 편입되고, 등재되지 않으면 M1 이 실패한다 ② 「목업↔구현 대조 범위」 신설 — 전 화면을 한꺼번에 대조하지 않는다. 구현이 목업 단계를 **병합**(`CredentialsSetup`)하거나 **분할**(`HomeRepositories`+`ConnectRepository`)한 쌍은 카피 배치가 정합성이 아니라 제품 결정이므로 「대조 보류」에 두고 **상한 3 래칫**으로 붙잡는다(면제가 아니다 — 늘면 실패) ③ 원장을 현재 목업 기준으로 재기준선 — 공전 행 제거, 유형 3종(데이터 부재 / 제3자 소유 / 분할·병합 미결정) 도입, 6칸 기계 판독 형식, 백틱 토큰만 대조 대상. `STP-grant-repo-access` 는 **GitHub 이 소유하는 설치 동의 화면**이라 구조적으로 수렴하지 않음을 처음으로 명시했다 ④ 제품 결정이 필요 없는 **무전제 수렴 5건**만 구현에 반영 — `기능 후보`(목업과 한 글자 차이였다) · CTA `비용 확인하기` · 추정 카드 캡션 `Target` 과 목업 순서(대상 → 비용·소요 → 스캔 수치) · 섹션 제목 `연결된 저장소`. **AC·여정·목업은 하나도 바뀌지 않았다** — 목업이 SSOT 이므로 움직인 것은 구현과 기록뿐이다 (reconciler `rct_20260831-0001`, 모델 `tbm_feature-doc-mockup-render`) | 편차 원장 7행 ↔ 캡션 6건(모순) · 공전 행 4 · 기계 판정 장치 없음 · 대조 범위 정의 없음 | 편차 원장 22행 = 캡션 22건 · 공전 행 0 · CI 게이트 6규칙 · 활성 대조 1 · 대조 보류 3(상한 래칫) |
| 2026-08-31 여정 단위 목업 이관 슬라이스 2 | `JRN-discover-features` 를 규칙 5 프로토타입으로 이관(5단계 · §4 분기 6행 · 갈래의 끝 2개)하고, 전용 화면 `s05`·`s06` 2개를 흡수·삭제했다. **선결 과제였던 「공유 화면」을 결정으로 닫았다** — 공유가 막는 것은 그 여정의 이관이 아니라 원본 `sNN` 파일의 삭제뿐이므로(규약 ⑤ + 「같은 화면이 여러 여정에 등장하는 것은 중복이 아니다」), 이관은 여정 단위로 진행하고 원본 삭제만 마지막 소비 여정까지 미룬다. `S04`·`S07` 은 `JRN-follow-code-change` 가 아직 써서 원장에 남는다. **실질 블로커는 페이지가 아니라 하네스였다** — `tools/check-journey-prototype.js` 의 P3(실제 입력)과 마지막 P5(두 번째 갈래의 끝)가 `JRN-connect-repo` 전용 id 에 여정 조건 없이 묶여 있어, 두 번째 여정 페이지가 생기는 순간 null 에 `.value` 를 대입하고 uncaught TypeError 로 통째 죽었다(단언 보고조차 못 한다). 여정별 등록부를 `INPUT_PROBE`·`SECOND_END` 로 추가해 4종으로 늘리고 P0 의 미등록 검사를 넷 전부로 넓혔다 — connect-repo 의 단언 80건은 회귀 없이 그대로다. 래칫 2종을 함께 내렸다(이관 대기 4→3 · 화면 단위 잔여 7→5). to-be(가치 8 · AC 24) 불변 — 문서·정적 HTML·도구 스크립트만 바뀌고 실행 코드 동작 변화 0 (reconciler `rct_20260831-0001`) | 여정 페이지 1 · 화면 단위 7 · 이관 대기 4 · 하네스 단언 80건(여정 1개) | 여정 페이지 2 · 화면 단위 5 · 이관 대기 3 · 하네스 단언 160건(여정 2개) |

## 다음 단계 권장

1. **제품 소유자 지정** — 모든 가치의 고아 상태를 해소합니다.
2. **제품 명칭 확정** — 파일명·문서 내 식별자 일괄 변경.
3. **운영 정책 수치화** — 보관 기간, 동시 분석 한도, LLM 호출 한도 등.
4. **PRD 검토 워크숍** — 4개 PRD를 소유자/엔지니어와 함께 검토하여 누락된 AC가 있는지 확인.
5. **남은 여정 4개의 목업 이관** — 전환 여부는 2026-08-30 에 "전환한다"로 결정됐고 `JRN-connect-repo` 로 규약이 실증됐다(위 위험 진단 참조). 남은 것은 `JRN-discover-features` · `JRN-review-feature` · `JRN-understand-feature` · `JRN-follow-code-change` 이며, 첫 판단은 **공유 화면**(S04·S07 은 2개, S08~S10 은 3개 여정이 씀)을 여정마다 복제해 담을지 정하는 것이다. `JRN-restore-history` 는 소스 화면이 없어 S11 제작(7번)이 먼저다.
6. **화면 단위 목업 7개의 여정 문서 복귀 링크** — 여정 페이지는 이미 복귀 링크를 갖는다(2026-08-30). 남은 `sNN-*.html` 7개는 여전히 단방향이라 목업을 먼저 연 사람이 근거 문서로 갈 수 없다. 화면 1개가 2~3개 여정에 걸치므로 링크는 목록 형태가 되며, 이관이 끝나면 항목 자체가 사라진다.
7. **S11 변경 이력 화면 제작** — 현재 "수용된 위험"으로 보류 중. `JRN-restore-history` 는 이 화면이 없으면 세 단계 전부가 시각화 없는 여정으로 남는다. 제작 시 와이어프레임(`gen-wireframes.js` S11 entry)→목업 순서로 만들고 본 추적 문서의 수용된 위험에서 해제.

8. **여정 지표의 계측 착수** — 2026-08-30 재작성으로 여정 6개에 지표 39건이 정의됐으나 전부 `TBD` 또는 `목표(예시)`이고 실제 계측은 없다. 목표치가 없는 지표는 마찰점(F1~F8)이 실제로 사용자를 멈춰 세우는지 확인할 수 없게 만든다. 우선 계측할 3건: **무단 덮어쓰기 발생 건수**(F6, 목표 0건 — 어기면 편집 기능 전체가 죽는다) · **완독률**(F5, V3의 핵심 실측치) · **3탭 준수율**(F8). AC4.6의 관측 기반이 깔리는 슬라이스 7과 함께 다루는 게 자연스럽다.
9. **여정 담당자 지정** — 여정 6개의 담당자가 전부 "미지정"이다. 1번(제품 소유자 지정)과 함께 처리한다.

> 참고: `.claude/skills/`에 디자인 시스템 보조 스킬 2개(`ui-with-design-system`, `screen-with-mockup-and-design-system`)가 설치되어 있습니다. 이 레포에서 UI/화면 코드 작업 시 디자인 시스템·목업을 자동으로 참조합니다. `.gitignore`로 제외하지 마세요.

## 구현 수렴 로드맵 (권장 순서)

미구현 AC 16건(AC1.3~1.4 · AC2.1~2.6 · AC3.1~3.5 · AC4.4 · AC4.6 · AC4.7)을 문서 → 구현으로 수렴시키는 권장 슬라이스 순서. 순서 근거는 **사용자 여정 순서**(연결 → 발견 → 문서)와 **기술 의존성**(분석 파이프라인은 큐·워커 기질이, feature 표현은 분석 산출물이, 문서 관리는 표현이 먼저 필요)이다. AC4.4(모바일 우선)는 각 슬라이스의 화면마다 적용하고, AC4.6(관측·비용)·AC4.7(격리)은 파이프라인 구축 시 내장한 뒤 마지막 슬라이스에서 마감 검증한다. (수립: 2026-07-12 `rct_20260712-0001`; 진행 갱신: 2026-08-02 `rct_20260802-0001`, 2026-08-07 `rct_20260807-0001`, 2026-08-07 `rct_20260807-0002`, 2026-08-13 `rct_20260813-0001`, 2026-08-14 `rct_20260814-0001`)

> **진행 현황 (2026-08-13):** 슬라이스 2가 **2a(백엔드 enqueue 계약, PR #8 `dd5e6ee`, `rct_20260715-0001`) + 2b(S02·S03 프론트, PR #11, `rct_20260807-0001`)** 로 완료되어 **AC1.1이 구현 완료**됐고, 슬라이스 3도 **3a(AC4.5 워크로드 분리, `rct_20260807-0002`) + 3b(AC1.5 S04 진행 화면·부분 재시도, `rct_20260813-0001`)** 로 모두 완료되어 **AC4.5·AC1.5가 구현 완료**됐다. 슬라이스 4도 **4a(AC1.2 횡단 관심사 추출·S05, `rct_20260814-0001`)** 로 절반이 끝나 **AC1.2가 구현 완료**됐다.
> 아직 미구현으로 남은 것은 **AC1.3~1.4 · AC2.1~2.6 · AC3.1~3.5 · AC4.4 · AC4.6 · AC4.7**. (다음 권장 슬라이스 = 4b)

| 슬라이스 | 대상 AC | 화면 | 내용 |
|----------|---------|------|------|
| 1 (이 변경) | AC4.8 문서화 | — | OAuth 인증을 PRD-4 범위·AC로 편입, 로드맵 확정 |
| 2a ✅ | AC1.1 (백엔드) | — | 저장소 열거(`/api/repositories`)·pre-flight 비용 추정(`/api/analyses/preflight`)·분석 트리거→`queued`(`POST /api/analyses`)·홈 목록(`GET /api/analyses`)·범위 밖 거부(미큐잉)의 백엔드 enqueue 계약. **완료: PR #8 `dd5e6ee`** (`rct_20260715-0001`) |
| 2b ✅ | AC1.1 (프론트) | S02 · S03 | 홈 저장소·분석 목록, 저장소 연결, 분석 트리거 UI, pre-flight 비용 안내 — 위 2a 백엔드 API를 소비. **완료: PR #11** (`rct_20260807-0001`) |
| 3a ✅ | AC4.5 | — | 큐 클레임(리스 기반)과 `featuredoc-worker` Deployment 분리, 단계 레코드 적재, 1단계 `fetch` 실행. 워커는 DB를 열지 않고 API의 `/internal`로 클레임한다 — SQLite 단일 writer 불변식을 유지한 채 수평 확장. **완료: `rct_20260807-0002`** |
| 3b ✅ | AC1.5 | S04 | 적재된 단계·진행률·추정 비용을 사용자에게 표현하고(`GET /api/analyses/{id}` + 주소 지정 가능한 S04), 실패 단계만 큐로 되돌리는 부분 재시도를 제공. 실측 비용은 AC4.6으로 남긴다. **완료: `rct_20260813-0001`** |
| 4a | AC1.2 | S05 | LLM 호출 경계 신설(`backend/src/llm.rs`) + 파이프라인 2단계 `cross_cutting` 실행 + 산출 문서 영속화·재현성 판정 + S05. **완료** (`rct_20260814-0001`) |
| 4b | AC1.3 → AC1.4 | S06 · S07 | 탐색 전략 생성·승인 → feature 후보 추출, 4a의 LLM 경계를 재사용. AC4.6 계측 내장. **다음 권장 슬라이스** |
| 5 | AC2.1~2.6 | S08 · S09 | 인수 기준 도출·보강·문서화, 의존성 추출·저장·재계산 |
| 6 | AC3.1~3.5 | S10 | LLM 문서 수정·직접 추가·삭제·이력·충돌 처리 |
| 7 | AC4.4 · AC4.6 · AC4.7 | 전 화면 | 모바일 우선 전 화면 확인, 사용자 노출 비용 완성, 격리 검증 |

> 각 슬라이스는 reconciler 정합성 루프의 개별 task로 계획·실행되며, 슬라이스 완료마다 본 문서의 미구현 AC 잔여를 갱신한다.
