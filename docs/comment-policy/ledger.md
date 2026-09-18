# 주석 판정 원장

정책 본문은 [README.md](README.md), 각 행의 근거 상세는 [passes/](passes/)에 있다. 이 파일은
판정 결과의 표면이다 — 규칙을 여기에 다시 쓰지 않고, 근거 상세를 여기에 펴지 않는다.

## 원장 읽는 법

- 한 행 = 판정한 범위(파일 단위). **지문**은 그 범위의 주석 줄을 정규화·정렬해 해싱한 값으로,
  주석이 추가·수정·삭제될 때만 변한다(본문 「지문과 사각지대」).
- 같은 범위의 증분 재판정은 새 행을 만들지 않고 원래 행의 결과 칸을 갱신하며, 패스 상세는
  원래 패스 파일에 절을 더한다.
- 결과 칸의 요약 뒤에는 해당 패스 파일을 링크한다.

| 판정 범위 | 현재 주석 줄 수 | 지문 | 결과 요약 |
|---|---|---|---|
| `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일) | 537 | `60954a7f865e9449bfe9a367cebbe0400ccf8f86f52358a7d22eb3edaffad401` | 순 제거 58행(구분선 16 · 빈 주석 행 3 · 문서·선언 재진술·작업 흔적 39 — 총 62행 제거 중 불변식 보존 2행 재작성) · 유지 507행 · 판단 갈림 3건 — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) · **증분 재판정 ①**(2026-09-18): #55가 더한 `llm.rs` 21행은 stub↔real 충실도 경계라 **전건 유지**, 제거 후보 3행(테스트 case 라벨)은 `llm.rs`가 #49와 경합이라 보류 · **증분 재판정 ②**(2026-09-18): #43이 `analysis.rs`·`worker_api.rs`에 더한 40행을 판정해 **순 제거 10행**(AC 조항 재진술 · 호출자·구조체 본문 재진술 · rustdoc 링크만의 교차 참조) · 유지 30행(리스 계약 · 게이트는 큐의 성질 · `acceptance_pending`의 술어 함정 · `work_remains`의 경합) |
| `tools/check-journey-prototype.js` · `backend/src/config.rs` (열린 통합 PR 무접촉 2파일) | 140 | `ddacce0b4fcb83672a018383ad5845e7a63cbce655d520c31e06643debf6329f` | 순 제거 35행(파일 머리 되풀이 인라인 마커 8 · 등록부 존재 이유 되풀이 · 같은 문구 5회 반복 5 · 절 제목 5 · 선언 재진술 — diff 기준 56행 삭제 · 4행 재작성, 차이는 블록 주석 본문이 지문에 안 보이기 때문) · 유지 130행 · 판단 갈려 남긴 것 11건 — [passes/2026-09-18-uncontested-harness-config.md](passes/2026-09-18-uncontested-harness-config.md) · **증분 재판정 ①**(2026-09-18): #60이 `config.rs`에 `Mode`·`Doubles` doc으로 더한 23행을 판정해 **순 제거 13행**(variant·시그니처 재진술 · 경계 식별자 필드 doc 6 · 정책 문서 인용 2) · 유지 10행(경계별 선택 불변식 · 안전 기본값) |
| `e2e/tests/sc01-01-full-pipeline-run.spec.ts` · `e2e/tests/sc01-06-partial-retry.spec.ts` · `e2e/support/cluster.ts` · `e2e/smoke.sh` · `e2e/playwright.config.ts` (e2e 하네스 비경합 5파일) | 191 | `a736f329faca47d85dff2c0f6560a6bddff15ec482a4ab9f748767d66a9cf126` | 순 제거 49행(시나리오 문서 인용 10 · 절 제목 18 · 제목+AC 2 · 작업 흔적 2 · `finally` 재진술 2 · smoke 머리·인라인 6 · 나머지 선언 재진술 — diff 기준 52행 삭제 · 3행 재작성) · 유지 142행(`playwright.config.ts`는 전건 유지) · 기계 판독 `// 검증 시나리오:` 선언 2개 보존 — [passes/2026-09-18-e2e-uncontested.md](passes/2026-09-18-e2e-uncontested.md) |
| `backend/src/acceptance.rs` · `backend/tests/acceptance.rs` · `e2e/support/acceptance.ts` · `e2e/tests/sc02-01-acceptance-from-logic.spec.ts` · `e2e/tests/sc02-04-user-facing-acceptance-doc.spec.ts` · `frontend/src/FeatureAcceptance.tsx` (인수 축 비경합 6파일) | 141 | `e38dcf201e4bd74600fd17c664a64a9aa4cb81003fd7f0b8652435e88d6864d0` | 순 제거 127행(PRD-2 AC2.1~AC2.3 조항 재진술 · doc-tracker 등재 편차 재진술 10 · 절 제목 9 · 목업 카피·페인포인트 인용 · 단언 재진술 · 이름 재진술 — diff 기준 180행 삭제 · 52행 재작성) · 유지 141행 · 판단이 갈려 남긴 것 4건 · 기계 판독 `// 검증 시나리오:` 2건과 `mock-exception:` 2건 보존 — [passes/2026-09-18-acceptance-axis.md](passes/2026-09-18-acceptance-axis.md) |
| `backend/src/bin/worker.rs` · `deploy/e2e/kustomization.yaml` · `deploy/k8s/deployment.yaml` · `deploy/k8s/secret.yaml.example` · `deploy/k8s/worker-deployment.yaml` · `e2e/tests/sc04-07-api-availability-without-workers.spec.ts` · `e2e/tests/sc04-08-worker-horizontal-scale.spec.ts` (워커 · 더블 배선 축 비경합 7파일) | 185 | `96569bfb7a4c6bc102f7c3c975841eb01b365fca9a5576707b22eaa588b1ca2d` | 순 제거 122행(5단계 열거·선언 재진술 · 시나리오 본문 인용 12 · 더블 배선 설명 14 · AC 조항 인용 9 · 3중 복제된 경위 서술 19 · 작업 흔적 8 · 절 제목 3 · 세 벌 중복 중 두 벌 — diff 기준 185행 삭제 · 63행 재작성) · 유지 185행 · 판단이 갈려 남긴 것 3건 · 기계 판독 `// 검증 시나리오:` 2건 보존 — [passes/2026-09-18-worker-double-axis.md](passes/2026-09-18-worker-double-axis.md) |

**합계**: 판정 24파일 · 판정 대상 누적 1,582행 · 순 제거 **414행** · 현재 1,145행.

**미판정 잔여**: **73파일 / 1,775행**
(이 패스 병합 후 트리 기준 — 전역 `lines=2920 files=97`에서 판정 24파일 / 1,145행을 뺀 값).
셋으로 갈린다.

- **`backend/migrations/*.sql` 7파일 / 104행** — 본문 「적용된 마이그레이션」 절의 전용 PR ·
  수동 repair · 사람 승인 게이트를 거치는 **별도 패스**다. 다른 정리와 섞지 않는다.
- **열린 PR 접촉(경합)** — `e2e/tests/sc02-02-acceptance-from-tests.spec.ts` 31행(**#67**,
  `sc02-03` 분할) · `tools/check-mockup-render.py` 71행(**#69**·**#26** draft) ·
  `frontend/src/{CrossCuttingConcerns,DiscoveryStrategy,FeatureCandidates}.tsx`(**#69**) ·
  `frontend/src/{ConnectRepository,CredentialsSetup,HomeRepositories}.tsx` · `index.css`
  100행(**#17** draft) · `tools/check-data-format-change.py`(**#64**). 각 PR이 닫힌 뒤 집는다.
- **자유 풀 나머지** — 지금 판정 가능한 범위다. 직전 판이 자매 모델
  `tbm_feature-doc-e2e-mock-policy`의 PR(#59·#60) 뒤로 미뤄 둔 12파일은 **#60이 `ebe8657`로
  머지되며 전부 자유로워졌고**, 그중 워커·더블 배선 축 7파일은 이 패스가 집었다. 남은 후보:
  `frontend/src/api.ts` 72 · `backend/src/feature_candidates.rs` 63 ·
  `tools/check-journey-mockup.py` 55 · `e2e/tests/sc01-07-candidate-rejection-carryover.spec.ts` 49 ·
  `backend/src/repo_scan.rs` 49 · `e2e/tests/sc01-04-strategy-edit-and-approve.spec.ts` 47 ·
  `backend/src/discovery_strategy.rs` 46 · `e2e/tests/sc01-03-cross-cutting-determinism.spec.ts` 45 ·
  `tools/check-scenario-e2e.py` 44 · `backend/tests/candidates.rs` 44.

> **①의 경합 분류는 이 패스를 준비하는 동안 무효가 됐다 — 다음 감지가 다시 나눈다.** 직전 판까지
> 잔여를 가르던 기준은 「열린 draft PR #43·#49 가 건드리는 46파일 / 1,440행」이었는데, **#49는
> #43 브랜치로, #43은 main으로 머지됐다**(`6484e215`, 2026-09-18T15:47Z). 그 46파일은 더 이상
> 경합이 아니므로 위 잔여에는 **경합/비경합 분할을 적지 않았다** — 전체 수치만 실측으로 갱신하고,
> 새 분할은 재감지가 연다. 이 패스의 5파일은 #43과 한 파일도 겹치지 않아 판정·지문 모두 영향이
> 없다.

> **직전 판이 「이 축의 실질 병목」으로 지목한 #43·#49가 그 사이 둘 다 풀렸다.** 직전 패스가
> 다음 후보로 적어 둔 `tools/` 체커 3개와 backend 모듈 4개는 #49 갱신으로 전부 경합에 묶여
> 있었는데, 그 머지로 **일곱 개 모두 다시 자유로워졌다**. 다음 패스의 후보 풀은 이 축에서 다시
> 열린다. 다만 #43은 **이미 판정한 1행 4파일(`analysis.rs`·`llm.rs`·`worker_api.rs`·`llmkey.rs`)에
> 주석 40행을 더했다**(507행 → 547행) — 1행은 그만큼 증분 재판정 대상이며, 이 패스의 범위 밖이다.

> 1행의 지문은 2026-09-17 판정 시점 트리(5c28852d) 기준이며, 그 패스가 병합돼
> `aa1931952efd07b621726cc6a524f742085bbe5ebbf3e94b6e3b3c210ab375ee`로 실제로 움직인 것을
> 확인했다. 그 뒤 #55가 `llm.rs`에 21행을 더해 현재 값은
> `83758c065a2a2e2cadeca3b3e93753202625c38559cd4339e2ea94aab4e8e1ca`(507행)이다 — 위 증분
> 재판정 ①이 그 이동을 판정한 결과다. 2행의 지문은 2026-09-18 판정 시점 트리(313750f) 기준이고,
> 그 패스가 병합돼 `3b985312228899e6111a8b7f87b516ca9d3ed2c88c01c58e07a141897d52a843`로 움직인
> 것을 현재 main 트리에서 재현했다. 3행의 지문은 2026-09-18 판정 시점 트리(84f2734) 기준이고,
> 이 패스가 병합되면 범위 지문은
> `fb27e0dba9370d129dc0d7972da601977fdcb2726397c9ee78f9a95b128b12ef`(142행)로 움직여야 한다.
> 이 값은 판정 시점 이후 베이스가 `84f2734` → `42e63fa`(#62, `.github/workflows/` 전용) →
> `6484e215`(#43)로 두 칸 움직인 뒤에도 **그대로다** — 두 커밋 모두 이 5파일에 무접촉이라
> 부모 기준을 옮겨 재현해도 191행 / `a736f329…` 그대로였다.
>
> **전역 as-is 예고값은 그 베이스 이동으로 갱신됐다.** 판정 시점 예고는
> `lines=2717 files=90` / `7b125a92…` 였는데, 그 값은 `84f2734`를 부모로 둔 것이라 #43이 더한
> 주석(신규 7파일 · 383행)을 담지 않는다. **머지 후 실측 기준은
> `lines=3100 files=97` / `a081f4e1aade89a7f727d3ec36b27cf337af65d3892bbf0871f09fd8187f4e4a`**
> 이다(부모 main `6484e215` = `lines=3149 files=97` / `7c5e57e9…` 에서 이 패스가 49행 순 제거).
> 순 제거 49행이라는 이 패스의 기여는 두 판 모두에서 같다.
> 지문 계산은 경로 접두사와 후행 개행을 포함한다 — 후행 개행 없이 재면 값이 달라진다.

> **4차 패스 기준 갱신 (2026-09-18 · `rct_20260918-0004`).** 위 세 블록의 수치는 각 패스의
> 판정 시점 값이라 그대로 두고, 현재 기준만 여기 적는다.
>
> - **직전 판의 전역 예고는 이미 지났다.** 3차 패스가 적은 머지 후 실측 기준
>   `lines=3100` / `a081f4e1…`은 **부모 `6484e215`를 기준으로 계산해 #59의 +1행을 담지
>   못했고**, 실제 머지 결과는 `lines=3101 files=97` / `3526b836…`였다. 그 뒤 #66이
>   `deploy/k8s/*.yaml`에 +2(→`lines=3103` / `764e05b6…`), #65·#60이 +89(→**`lines=3192
>   files=97` / `f724aab9…`** = 이 패스의 부모 `ebe8657`)를 더했다. 원장 3행의 **행수·지문은
>   전건 일치**했으므로 판정의 오류가 아니라 전역 합계 줄의 신선도 문제였다.
> - **이 패스가 병합되면 전역 지문은 `lines=3055 files=97` /
>   `b1f902629f76dd10e60cb3072b4a1f2e8aa222d8f44769bcaf522ce5c0578d5e`로 움직여야 한다**
>   (부모 `ebe8657` 3,192행에서 순 제거 137행 = 인수 축 127 + 증분 재판정 ② 10).
> - **원장 1행의 「507행 → 547행」 증분은 이 패스가 닫았다** — 위 증분 재판정 ②. 1행의 현재
>   값은 537행 / `60954a7f…`다.
> - **원장 2행은 #60이 `config.rs`에 더한 23행만큼 움직였고 그 증분은 아직 미판정이다**
>   (130행 / `3b985312…` → 153행 / `ea29e09f…`). 다음 패스의 첫 항목 중 하나다.
> - 지문 계산 규약은 위와 같다: 원장의 **범위 지문**은 후행 개행을 **포함**하고
>   (`echo "$HITS" | sha256sum`), 모델의 **전역 지문**은 versionScript 그대로 후행 개행을
>   **제외**한다(`printf '%s'`). 같은 입력에도 두 값은 다르다.

> **5차 패스 기준 갱신 (2026-09-18 · `rct_20260918-0005`).** 위 블록들의 수치는 각 패스의 판정
> 시점 값이라 그대로 두고, 현재 기준만 여기 적는다.
>
> - **4차 패스의 전역 예고는 정확히 맞았다.** #68이 머지된 main tip `fae3e17`의 실측은
>   `lines=3055 files=97` / `b1f90262…`로 예고값과 전건 일치했고, 그 뒤 이 패스의 부모까지
>   main은 움직이지 않았다. 원장 1·3·4행의 행수·지문도 tip에서 재현해 **전건 일치**했다
>   (537 / `60954a7f…` · 142 / `fb27e0db…` · 141 / `e38dcf20…`).
> - **이 패스가 병합되면 전역 지문은 `lines=2920 files=97` /
>   `d4c000e483d4ad39c88a4c8e57e2cecf756cdc35a0041d0e9ae0e71d87db324c`로 움직여야 한다**
>   (부모 `fae3e17` 3,055행에서 순 제거 135행 = 워커·더블 축 122 + 증분 재판정 13).
> - **원장 2행의 「미판정 증분 23행」은 이 패스가 닫았다** — 위 증분 재판정 ①. 2행의 현재 값은
>   140행 / `ddacce0b…`다. 이제 원장 안에 미판정으로 남은 증분은 **없다**.
> - **#60이 푼 12파일 블록은 이 패스가 그 중심 7파일을 집으며 절반 이상 닫혔다.** 남은 조각
>   (`sc04-04`·`sc04-12`·`sc04-13`의 머리 개명 추종 8행 등)은 자유 풀에 있고, 그 8행은
>   `FEATUREDOC_MODE=stub` → `FEATUREDOC_DOUBLE_*=stub` **개명 추종이지 새 명제가 아니다**.
> - **열린 PR 집합이 하나 늘었다** — #69(앱바 슬롯 축)가 `tools/check-mockup-render.py`와
>   frontend 3화면을 잡고 있다. 위 경합 목록은 그 기준으로 다시 적었다.
