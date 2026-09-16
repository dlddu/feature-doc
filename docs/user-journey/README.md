# 사용자 여정 (User Journey)

> FeatureDoc 사용자가 겪는 **행동의 흐름**을 여정 단위로 정리한 문서 묶음.
>
> [`values.md`](../values.md)가 *왜* 만드는지를, [`prd/`](../prd/)가 *무엇을* 만드는지를, [`mockups/`](../mockups/)가 *어떻게 보여줄지*를 다룬다면, 이 묶음은 **사용자가 무엇을 하려고 하는지**를 다룹니다.

## 이 묶음의 원칙

**단계는 화면이 아니라 행동으로 나눕니다.** 화면은 각 단계의 터치포인트 항목에 적습니다. 화면은 리뉴얼 때마다 바뀌지만 행동은 오래 유지되므로, 행동으로 나눠야 문서 수명이 깁니다.

**식별자는 순번을 쓰지 않습니다.** 여정은 `JRN-<슬러그>`, 단계는 `STP-<슬러그>`. 단계가 추가·삭제·재배치돼도 식별자는 그대로 살아남고, 목업·검증 쪽 연결이 조용히 어긋나지 않습니다. 기존 식별자는 바꾸지 않으며, 단계가 사라져도 식별자를 재사용하지 않고 각 문서의 변경 이력에 폐기 사실을 남깁니다.

**한 여정 = 한 페르소나의 한 목표.** 같은 화면을 두 페르소나가 다른 목적으로 쓰면 여정을 나눕니다.

## 1. 페르소나

### P1 — Pre (제품 소유 엔지니어)

자신이 운영하는 사이드 프로젝트나 회사 서비스 저장소를 들고 옵니다. 코드는 잘 알지만 "최종 사용자가 이 제품으로 무엇을 할 수 있는지"를 글로 정리한 적이 없습니다.

- **동기**: 기능 목록을 한 번 자동으로 뽑고, 누락과 의존성을 확인하고 싶다
- **기기**: 절반 이상 모바일. 출퇴근·짧은 휴식에 검토
- **위험 감각**: 저장소 접근 범위와 LLM Key 취급에 민감
- **연결 가치**: V1, V2, V4, V5, V6, V7, V8
- **여정**: `JRN-connect-repo` · `JRN-discover-features` · `JRN-review-feature` · `JRN-follow-code-change` · `JRN-restore-history`

### P2 — Roo (검토하는 비개발 동료 / PO)

코드를 읽지 못하지만 제품의 능력을 정확히 알아야 합니다. Pre가 분석을 끝내 둔 저장소를 공유받아 모바일에서 검토합니다.

- **동기**: 기능 하나를 1~2분 안에 이해하고, 남에게 설명할 수 있는 상태가 되고 싶다
- **기기**: 모바일 100%
- **위험 감각**: 개발자 용어가 섞이면 즉시 신뢰를 잃음
- **연결 가치**: V3, V5, V7
- **여정**: `JRN-understand-feature` (그리고 `JRN-follow-code-change`의 알림 수신자로만 등장)

P1과 P2는 같은 인스턴스를 공유하지만 **권한과 정보 깊이의 노출 정도가 다릅니다.** P2 진입 경로에는 자격증명 설정이 나타나지 않아야 합니다.

## 2. 여정 지도

| 여정 | 페르소나 | 트리거 | 완료 기준 | 단계 |
|---|---|---|---|---|
| [저장소를 맡기고 첫 분석을 걸기](./JRN-connect-repo.md) `JRN-connect-repo` | P1 | 기능 목록을 매번 코드에서 찾고 있다는 자각 | 분석 1건이 큐에 등록됨 | 5 |
| [코드에서 기능 목록 뽑아내기](./JRN-discover-features.md) `JRN-discover-features` | P1 | 단계 완료 알림 | 후보 전부 결정 완료 → 확정 목록 생성 | 5 |
| [기능 하나의 표현이 맞는지 검수하기](./JRN-review-feature.md) `JRN-review-feature` | P1 | 확정 목록 생성, 또는 표현이 미심쩍음 | 시나리오·의존성 확인 + 편집 결정 | 5 |
| [코드를 못 읽는 사람이 기능을 이해하기](./JRN-understand-feature.md) `JRN-understand-feature` | **P2** | 회의·문의 대응, 또는 P1의 공유 링크 | 시나리오 완독 (+ 모호하면 제안 1건) | 4 |
| [코드가 바뀐 뒤 문서가 따라왔는지 확인하기](./JRN-follow-code-change.md) `JRN-follow-code-change` | P1 | push 후 "N개 기능 갱신" 알림 | 변경분 diff 확인 + 미해소 충돌 0건 | 4 |
| [잘못된 변경을 되짚어 되돌리기](./JRN-restore-history.md) `JRN-restore-history` | P1 | "예전엔 이렇지 않았는데"라는 위화감 | 원인 특정 + 복원 또는 유지 결정 | 3 |

### 여정 사이의 이어짐

```
JRN-connect-repo ──(분석 큐 등록)──> JRN-discover-features ──(확정 목록)──> JRN-review-feature
                                                                                    │
                                                              (P1이 링크 공유)      │
                                                                    ↓               │
                                                        JRN-understand-feature      │
                                                                                    │
   ┌────────────────(코드 push)──────────────────────────────────────────────────────┘
   ↓
JRN-follow-code-change ──(자동 갱신이 틀렸을 때)──> JRN-restore-history
   │
   └──(거부 항목을 이번엔 채택)──> JRN-review-feature
```

앞의 세 여정은 처음 한 번 순서대로 흐르지만, 그 뒤로는 순서가 없습니다. `JRN-review-feature`는 feature 개수만큼 반복되고, `JRN-follow-code-change`는 코드가 사는 한 비주기적으로 계속 재진입합니다.

## 3. End-to-end 시나리오: "Pre가 자기 사이드 프로젝트를 처음 분석하는 토요일 오후"

각 여정의 자체 시나리오는 해당 문서에 있습니다. 여기서는 여정들이 **하루 안에서 어떻게 이어지는지**만 봅니다.

```
14:02  로그인 → App 설치 → 키 등록          JRN-connect-repo (STP-sign-in ~ STP-register-llm-key)
14:05  repo·브랜치 지정, 비용 확인 후 시작   JRN-connect-repo (STP-pick-target ~ STP-confirm-cost)
14:06  앱 닫음                              [여정 완료 — 분석 큐 등록]
14:24  알림 받고 복귀, 진행 상태 그대로       JRN-discover-features (STP-leave-and-return)
14:25  횡단 관심사 확인                     JRN-discover-features (STP-review-landscape)
14:27  admin CLI 한 줄 보태고 전략 승인      JRN-discover-features (STP-tune-strategy)
14:30  후보 14개 중 12개 승인, 2개 거부      JRN-discover-features (STP-sift-candidates)
14:32  누락된 기능 1개 직접 추가             JRN-discover-features (STP-add-missing)
14:35  "비밀번호 재설정" 시나리오 검수        JRN-review-feature (STP-read-scenarios ~ STP-verify-evidence)
14:38  의존성 그래프 첫 체험                 JRN-review-feature (STP-trace-dependencies)
14:41  "에러 케이스 추가" 요청 → diff 승인    JRN-review-feature (STP-request-edit ~ STP-decide-diff)
14:43  앱 닫음 (12개 중 1개만 검수)

(월요일)
       Roo가 공유 링크로 같은 기능을 읽음     JRN-understand-feature

(1주일 뒤 금요일)
19:32  push 후 "3개 기능 갱신" 알림          JRN-follow-code-change (STP-notice-change)
19:34  충돌 발견 → LLM 보조 병합 선택        JRN-follow-code-change (STP-resolve-conflict)
```

능동 시간 합계 약 23분. 분석 백그라운드 대기 18분은 제외.

## 4. 마찰점 종합

각 마찰점은 특정 단계에 붙어 있습니다. 대응은 해당 단계의 "페인포인트 / 이탈 위험" 항목에 있습니다.

| # | 마찰점 | 붙는 단계 | 1차 대응 AC | 실측 지표 |
|---|---|---|---|---|
| F1 | 외부 App에 저장소 접근 허용이 심리적 장벽 | `JRN-connect-repo` / `STP-grant-repo-access` | AC4.1, AC4.3 | App 설치 이탈률 |
| F2 | 분석 시작 직전 비용 불확실 | `JRN-connect-repo` / `STP-confirm-cost` | AC4.6 | 비용 확인 후 이탈률 |
| F3 | 분석 단계 실패 시 처음부터 다시 | `JRN-discover-features` / `STP-leave-and-return` | AC1.5 | 단계 재시도 발생률 |
| F4 | LLM 비결정성으로 결과가 흔들림 | `JRN-discover-features` / `STP-review-landscape` | AC1.2 | 재분석 결과 변동률 |
| F5 | 시나리오에 개발자 용어 잔존 | `JRN-understand-feature` / `STP-grasp-behavior` (+ `JRN-review-feature` / `STP-read-scenarios`) | AC2.3 | 완독률, 중도 이탈 지점 분포 |
| F6 | 자동 분석이 사용자 편집을 덮어씀 | `JRN-follow-code-change` / `STP-resolve-conflict` | AC3.5 | 무단 덮어쓰기 발생 건수(목표 0) |
| F7 | 거부한 feature가 다시 잡힘 | `JRN-follow-code-change` / `STP-recheck-candidates` | AC1.4, AC3.3 | 거부 결정 반복 시간 |
| F8 | 모바일에서 편집이 3탭을 넘김 | `JRN-review-feature` / `STP-decide-diff` | AC3.1, AC4.4 | 3탭 준수율 |

번호는 이전 판(F1~F8)과 동일하게 유지했습니다 — [`doc-tracker.md`](../doc-tracker.md)와 외부 참조가 깨지지 않도록.

> 이 표는 `doc-tracker.md`의 위험 진단과 별개입니다. 그쪽은 *문서 체계의 위험*을, 이 표는 *사용자 행동상의 마찰*을 다룹니다.

## 5. 연결 매트릭스 — 여정 ↔ 가치 ↔ AC

| 여정 | 가치 | AC |
|---|---|---|
| [`JRN-connect-repo`](./JRN-connect-repo.md) | V1, V6, V8 | AC1.1, AC4.1, AC4.2, AC4.3, AC4.6, AC4.8 |
| [`JRN-discover-features`](./JRN-discover-features.md) | V1, V2, V4, V8 | AC1.2, AC1.3, AC1.4, AC1.5, AC3.2, AC4.6 |
| [`JRN-review-feature`](./JRN-review-feature.md) | V3, V4, V5, V7 | AC2.1, AC2.2, AC2.3, AC2.4, AC2.5, AC3.1, AC3.4, AC4.4 |
| [`JRN-understand-feature`](./JRN-understand-feature.md) | V3, V5, V7 | AC2.3, AC2.4, AC3.1, AC3.4, AC4.7, AC4.8 |
| [`JRN-follow-code-change`](./JRN-follow-code-change.md) | V4, V5, V7 | AC1.4, AC2.6, AC3.3, AC3.5 |
| [`JRN-restore-history`](./JRN-restore-history.md) | V4, V7 | AC3.4 |

표의 AC 열은 각 여정의 **단계에 `연결 AC`로 붙은 것**만 적습니다. 분기·예외표에서 다른 여정 소유의 AC를 참조하는 경우(예: `JRN-connect-repo` 분기표의 AC1.5)는 세지 않습니다 — 한 AC가 여러 여정에 흩어져 보이면 어느 여정이 그 AC를 책임지는지 흐려지기 때문입니다.

**커버리지 점검**

- 가치 8개 중 등장: **8개** ✅ (V1~V8)
- AC 24개 중 등장: **23개**. 미등장 1개는 **AC4.5**(k8s 배포·워크로드 분리) — 사용자에게 보이지 않는 운영 배경이라 어느 행동에도 붙지 않습니다. AC4.4(모바일 우선)는 모든 여정에 깔리는 횡단 약속이지만, 3탭 룰이라는 관찰 가능한 형태로 `STP-decide-diff`에 앵커했습니다
- **시각화 공백 1건**: `JRN-restore-history` 전체 (이력 화면 미제작). [`doc-tracker.md`](../doc-tracker.md)에 수용된 위험으로 등재되어 있습니다. `JRN-connect-repo` / `STP-sign-in`의 미인증 상태는 여정 페이지가 실제 로그인 화면을 가지면서 해소됐습니다
- **목업 연결 5건**: 판정 대상 여정 5개가 각각 목업 페이지 하나(`mockups/JRN-<슬러그>.html`)를 갖습니다. `JRN-restore-history`는 위 공백 1건으로 예외 등재. 매핑의 단일 소스는 [`mockups/README.md`](../mockups/README.md)입니다

## 6. 갱신 정책

다음 경우에 이 묶음을 함께 갱신합니다.

1. **새 AC 추가로 사용자 행동이 늘거나 바뀜** → 해당 여정에 단계를 추가하고(새 `STP-` 부여) §5 매트릭스 갱신. 어느 여정에도 안 붙으면 새 여정이 필요하다는 신호
2. **화면 추가·제거** → 영향받는 단계의 터치포인트를 갱신. 화면은 이름을 가진 단위가 아니라 단계 안의 자리이므로 여정 문서의 구조는 바뀌지 않습니다
3. **새 페르소나 정의** → §1과, 그 페르소나의 목표에 해당하는 새 여정 문서 추가
4. **새 여정 추가** → 여정 문서와 같은 PR에서 목업 페이지(`mockups/JRN-<슬러그>.html`)까지 만들고, 각 여정의 `연결 문서` 칸과 §5 매트릭스를 함께 갱신 (이관 대기 상한이 0이라 페이지 없는 여정은 CI를 통과하지 못합니다)

문서를 고칠 때는 기존 식별자를 유지한 채 해당 섹션만 바꾸고, 그 문서의 변경 이력에 한 줄을 추가하며 버전을 올립니다(내용 변경 0.1 단위, 팀 확정 시 1.0). 단계를 추가·삭제했다면 목업 연결이 깨질 수 있으므로 `doc-tracker.md`에도 남깁니다.

문서 체계 전체의 일관성 점검은 [`doc-tracker.md`](../doc-tracker.md)에서 통합 추적합니다.
