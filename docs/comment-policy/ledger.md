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
| `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일) | 590 | `9757b6b5a0251379c55161ab6768da3b3f05548f1866b2a98f82f98b4f4dea87` | 순 제거 58행(구분선 16 · 빈 주석 행 3 · 문서·선언 재진술·작업 흔적 39 — 총 62행 제거 중 불변식 보존 2행 재작성) · 유지 507행 · 판단 갈림 3건 — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) · **증분 재판정 ①**(2026-09-18): #55가 더한 `llm.rs` 21행은 stub↔real 충실도 경계라 **전건 유지**, 제거 후보 3행(테스트 case 라벨)은 `llm.rs`가 #49와 경합이라 보류 · **증분 재판정 ②**(2026-09-18): #43이 `analysis.rs`·`worker_api.rs`에 더한 40행을 판정해 **순 제거 10행**(AC 조항 재진술 · 호출자·구조체 본문 재진술 · rustdoc 링크만의 교차 참조) · 유지 30행(리스 계약 · 게이트는 큐의 성질 · `acceptance_pending`의 술어 함정 · `work_remains`의 경합) · **증분 재판정 ③**(2026-09-19): #71이 `worker_api.rs`에 더한 37행을 판정해 **순 제거 18행**(라우트 선택 근거의 세 벌째 · AC 꼬리표 · 함수 이름 재진술 · SQL이 이미 말하는 「승인된 후보만」 · rustdoc 링크만의 교차 참조) · 유지 19행(요청 행이 곧 게이트 · 실패는 재시도되지 않음 · 같은 트랜잭션에서 행 교체) — [passes/2026-09-19-dependencies-axis.md](passes/2026-09-19-dependencies-axis.md) · **증분 재판정 ④**(2026-09-19): #71이 `analysis.rs`에 더한 36행과 #75가 더한 24행, 합 **60행**을 판정해 **순 제거 29행**(절 제목 · AC 조항 재진술 6 · 여정·목업 인용 · 선언 재진술 · 라우트 테이블과 핸들러 doc 의 두 벌 중 한 벌) · 유지 31행(`(created_at, rowid)` 정렬 이유 · 「묻지 않은 것 ≠ 없던 것」과 0008 의 두 테이블 분리 · 재큐잉 없는 요청은 아무도 돌리지 않는다는 한 트랜잭션 계약 · 실패 후 재요청이 행을 `queued` 로 되돌린다) — [passes/2026-09-19-reanalysis-diff-axis.md](passes/2026-09-19-reanalysis-diff-axis.md) · **증분 재판정 ⑤**(2026-09-21): #93이 `llm.rs`에 더한 `assert_strict_schema` doc 5행을 **명제 단위로** 판정해 **순 제거 2행**(함수 본문의 두 `assert` 재진술 · 단정 메시지가 이미 말하는 「optional 대신 nullable」과 스키마 리터럴 `["string","null"]` 재진술 — 5행 → 3행 재작성) · 유지 3행(`pub(crate)` 요약 1줄 · 「`required` 누락은 모델 실행 전 400」의 상류 거부 조건 · stub 은 스키마를 어디에도 보내지 않는다는 충실도 경계). **이 자리가 「required-but-nullable」 명제의 정본이다** — 강제하는 코드 `assert_strict_schema` 옆(4·6·8행의 같은 명제 사본 4행은 그래서 걷었다) — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) · **미판정 증분 없음** |
| `tools/check-journey-prototype.js` · `backend/src/config.rs` (열린 통합 PR 무접촉 2파일) | 140 | `ddacce0b4fcb83672a018383ad5845e7a63cbce655d520c31e06643debf6329f` | 순 제거 35행(파일 머리 되풀이 인라인 마커 8 · 등록부 존재 이유 되풀이 · 같은 문구 5회 반복 5 · 절 제목 5 · 선언 재진술 — diff 기준 56행 삭제 · 4행 재작성, 차이는 블록 주석 본문이 지문에 안 보이기 때문) · 유지 130행 · 판단 갈려 남긴 것 11건 — [passes/2026-09-18-uncontested-harness-config.md](passes/2026-09-18-uncontested-harness-config.md) · **증분 재판정 ①**(2026-09-18): #60이 `config.rs`에 `Mode`·`Doubles` doc으로 더한 23행을 판정해 **순 제거 13행**(variant·시그니처 재진술 · 경계 식별자 필드 doc 6 · 정책 문서 인용 2) · 유지 10행(경계별 선택 불변식 · 안전 기본값) |
| `e2e/tests/sc01-01-full-pipeline-run.spec.ts` · `e2e/tests/sc01-06-partial-retry.spec.ts` · `e2e/support/cluster.ts` · `e2e/smoke.sh` · `e2e/playwright.config.ts` (e2e 하네스 비경합 5파일) | 142 | `93be69ea2cff032c6bac2c4d0585d801aa22e392856891e1ce2e42f4d9aa243a` | 순 제거 49행(시나리오 문서 인용 10 · 절 제목 18 · 제목+AC 2 · 작업 흔적 2 · `finally` 재진술 2 · smoke 머리·인라인 6 · 나머지 선언 재진술 — diff 기준 52행 삭제 · 3행 재작성) · 유지 142행(`playwright.config.ts`는 전건 유지) · 기계 판독 `// 검증 시나리오:` 선언 2개 보존 — [passes/2026-09-18-e2e-uncontested.md](passes/2026-09-18-e2e-uncontested.md) · **증분 재판정 ①**(2026-09-20): #79가 `sc01-01`에 더한 순증 3행을 판정해 **순 제거 3행**(슬라이스 번호를 단 작업 흔적 2행 제거 · 목업 인용을 뺀 단정 의미 1행만 유지) · 범위는 145행 → 142행으로 돌아왔으나 **지문은 `fb27e0db…`가 아니라 `0e5c3d31…`**이다(살아남은 줄의 문면이 #79 이전과 다르다) — [passes/2026-09-20-pipeline-cross-cutting-axis.md](passes/2026-09-20-pipeline-cross-cutting-axis.md) · **증분 재판정 ②**(2026-09-20): #83 이 `sc01-01` 에 연 순증 2행을 판정해 **순 제거 2행**(슬라이스 번호를 단 작업 흔적 · 화면 전이 서술 · 교차 참조 `(선례: sc01-05)`) · 유지 2행(「셋업을 API 로 끝내도 로드는 자격증명 화면에서 시작한다」는 상태 머신 함정) · 범위는 144행 → 142행이지만 **지문은 `0e5c3d31…` 가 아니라 `93be69ea…`** 다 — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) · **증분 재판정 ③**(2026-09-21): #99(`899800e`)가 `sc01-01`(+5) · `sc01-06`(+5) 에 더한 10행을 판정해 **순 제거 10행**(원장 항목 번호 ⑼⒃㉑⑿ 를 현재형으로 인용하는 작업 흔적 5 — 인용 대상 4행은 같은 커밋이 doc-tracker 에서 닫아 지웠다 · PR #99 계획 1·2·3 과 doc-tracker 변경 이력의 축자 · 시나리오 6 기대 결과 재진술 · 바로 아래 단정이 그 문장 자체인 것) · 유지 0행 · 줄 수와 지문이 **둘 다 #99 이전 값으로 되돌아왔다**(142 / `93be69ea…` — 부모 `3567755` 실측과 바이트 동일) — [passes/2026-09-18-e2e-uncontested.md](passes/2026-09-18-e2e-uncontested.md) 「증분 재판정 ③」 · **미판정 증분 없음** |
| `backend/src/acceptance.rs` · `backend/tests/acceptance.rs` · `e2e/support/acceptance.ts` · `e2e/tests/sc02-01-acceptance-from-logic.spec.ts` · `e2e/tests/sc02-04-user-facing-acceptance-doc.spec.ts` · `frontend/src/FeatureAcceptance.tsx` (인수 축 비경합 6파일) | 141 | `e38dcf201e4bd74600fd17c664a64a9aa4cb81003fd7f0b8652435e88d6864d0` | 순 제거 127행(PRD-2 AC2.1~AC2.3 조항 재진술 · doc-tracker 등재 편차 재진술 10 · 절 제목 9 · 목업 카피·페인포인트 인용 · 단언 재진술 · 이름 재진술 — diff 기준 180행 삭제 · 52행 재작성) · 유지 141행 · 판단이 갈려 남긴 것 4건 · 기계 판독 `// 검증 시나리오:` 2건과 `mock-exception:` 2건 보존 — [passes/2026-09-18-acceptance-axis.md](passes/2026-09-18-acceptance-axis.md) · **증분 재판정 ①**(2026-09-21): #93이 `acceptance.rs`에 더한 1행(`// Required-but-nullable; see feature_candidates::schema.`)은 두 줄 위 `required` 배열과 바로 아래 `["string","null"]` 이 축자로 말하는 것(①) + 교차 참조뿐이라 **제거 1행** — 줄 수·지문이 #93 이전 값(141 / `e38dcf20…`)으로 되돌아왔다 — [passes/2026-09-18-acceptance-axis.md](passes/2026-09-18-acceptance-axis.md) 「증분 재판정 ①」 · **미판정 증분 없음** |
| `backend/src/bin/worker.rs` · `deploy/e2e/kustomization.yaml` · `deploy/k8s/deployment.yaml` · `deploy/k8s/secret.yaml.example` · `deploy/k8s/worker-deployment.yaml` · `e2e/tests/sc04-07-api-availability-without-workers.spec.ts` · `e2e/tests/sc04-08-worker-horizontal-scale.spec.ts` (워커 · 더블 배선 축 비경합 7파일) | 190 | `632b0475182bf04b9ec8c21f3b69c5b91172ecee41e417f7a58b98cdb15ba66d` | 순 제거 122행(5단계 열거·선언 재진술 · 시나리오 본문 인용 12 · 더블 배선 설명 14 · AC 조항 인용 9 · 3중 복제된 경위 서술 19 · 작업 흔적 8 · 절 제목 3 · 세 벌 중복 중 두 벌 — diff 기준 185행 삭제 · 63행 재작성) · 유지 185행 · 판단이 갈려 남긴 것 3건 · 기계 판독 `// 검증 시나리오:` 2건 보존 — [passes/2026-09-18-worker-double-axis.md](passes/2026-09-18-worker-double-axis.md) · **증분 재판정 ①**(2026-09-19): #71이 `bin/worker.rs`에 더한 12행을 판정해 **순 제거 7행**(같은 문장의 네 번째 벌 · AC 꼬리표 · 라우트 선택 근거) · 유지 5행(한 feature 의 실패가 잡을 죽이지 않는다는 격리 계약) — [passes/2026-09-19-dependencies-axis.md](passes/2026-09-19-dependencies-axis.md) |
| `backend/src/dependencies.rs` · `backend/tests/dependencies.rs` · `frontend/src/FeatureDependencies.tsx` · `e2e/support/dependencies.ts` · `e2e/tests/sc02-05-dependency-extraction.spec.ts` · `e2e/tests/sc02-06-dependency-reverse-query.spec.ts` · `e2e/tests/sc02-07-dependency-export.spec.ts` (의존성 축 비경합 7파일) | 112 | `e2ff313a778df1e7388fc7bbb07bd72c323ead751d854c5db743c3c2eee133e9` | 순 제거 137행(PRD-2 AC2.4·AC2.5 조항 재진술 17 · 목업·여정·doc-tracker 인용 21 · 시나리오 문서 인용 22 · 테스트 이름을 다시 쓴 doc 11 · 단정 재진술 18 · 선언 재진술 19 · 구분선 3 · 세 벌 중복 중 두 벌 — diff 기준 189행 삭제 · 50행 재작성) · 유지 112행 · 판단이 갈려 남긴 것 6건 · 기계 판독 `// 검증 시나리오:` 3건과 `mock-exception:` 1건 보존 — [passes/2026-09-19-dependencies-axis.md](passes/2026-09-19-dependencies-axis.md) · **증분 재판정 ①**(2026-09-21): #93이 `dependencies.rs`에 더한 1행(4행과 같은 문장의 두 번째 벌)은 같은 근거로 **제거 1행** — 줄 수·지문이 #93 이전 값(112 / `e2ff313a…`)으로 되돌아왔다 — [passes/2026-09-19-dependencies-axis.md](passes/2026-09-19-dependencies-axis.md) 「증분 재판정 ①」 · **미판정 증분 없음** |
| `backend/src/diff.rs` · `backend/tests/diff.rs` · `frontend/src/AnalysisDiff.tsx` · `e2e/tests/sc02-08-reanalysis-diff.spec.ts` · `backend/src/repo_scan.rs` · `frontend/src/AnalysisProgress.tsx` · `backend/src/lib.rs` (재분석 diff 축 비경합 7파일) | 135 | `47fdf929108fc66c0e5671797aca525c1d6587cf47c2750ca7830533c79a5188` | 순 제거 168행(화면 두 개의 머리 주석 50 — 목업·여정·doc-tracker 인용과 「이 슬라이스가 그리지 않는 것」 목록이 원장에 이미 있다고 주석 스스로 적는다 · 시나리오 문서 인용 12 · 선언·시그니처 재진술 41(rustdoc 링크만의 교차 참조와 이미 낡은 `(future)` 포함) · 테스트 이름을 다시 쓴 doc 27 · 네 벌까지 복제된 명제 중 잉여 18 · 구분선 8 · AC 꼬리표) · 유지 132행 · 판단이 갈려 남긴 것 5건 · 기계 판독 `// 검증 시나리오:` 1건 · `mock-exception:` 1건 · **화면 머리의 목업 매핑 2건**(M1 이 읽는다) 보존 — [passes/2026-09-19-reanalysis-diff-axis.md](passes/2026-09-19-reanalysis-diff-axis.md) · **증분 재판정 ①**(2026-09-21): #99(`899800e`)가 `AnalysisProgress.tsx` 에 더한 4행을 판정해 **순 제거 3행**(`subOf` 머리의 「서버 사유를 그리지 않는다」 3행 — doc-tracker 변경 이력 ⑿ · PR #99 계획 3 · 바로 아래 `return '실패했어요'` 와 `stage-failed` 안내의 삼중) · 유지 1행(「한 번에 실패하는 단계는 하나 — 파이프라인이 거기서 멈춘다」 — `stages.find` 의 전제인 백엔드 불변식인데 `docs/` · `pipeline.rs` 어디에도 문장으로 없다, 판단이 갈려 남김) · 지문 밖 JSX 블록 연속행 3행(「Stage 3 is deliberately not one of them …」 — doc-tracker ⒃ · PR 계획 2 축자)도 함께 제거 · 132행 → 133행(판정 시점 `4aa7a8eb…`) — [passes/2026-09-19-reanalysis-diff-axis.md](passes/2026-09-19-reanalysis-diff-axis.md) 「증분 재판정 ①」 · **#101(`3d147d6`, 슬라이스 ⑪) 유입 2행 미판정** — 이 재판정의 머지 직전에 #101 이 먼저 머지돼 `STAGE_TITLES` 머리 주석 2행이 들어왔다. 위 줄 수·지문(**135** / `47fdf929…`)은 병합 트리 재실측값이라 그 2행을 **포함**한다(판정 완료는 133) · 후속 재감지 몫 |
| `backend/src/discovery_strategy.rs` · `backend/src/feature_candidates.rs` · `backend/tests/strategy.rs` · `backend/tests/candidates.rs` · `frontend/src/DiscoveryStrategy.tsx` · `frontend/src/FeatureCandidates.tsx` · `e2e/tests/sc01-04-strategy-edit-and-approve.spec.ts` · `e2e/tests/sc01-07-candidate-rejection-carryover.spec.ts` (후보·전략 축 비경합 8파일) | 141 | `a431e905c7f2b2e08b033c28b6776705c4995a25a6e21bfb3034288900842d70` | 순 제거 222행(파일 머리 넷의 AC1.3·AC1.4 검증 방법 재진술 56 · 목업·여정·doc-tracker 인용 27 · 구분선 21 · AC 꼬리표 · 선언·시그니처 재진술 · 테스트 이름을 다시 쓴 doc · **여섯 벌까지 복제된 명제 중 잉여** — diff 기준 290행 삭제 · 65행 재작성) · 유지 140행 · 판단이 갈려 남긴 것 6건 · 기계 판독 `// 검증 시나리오:` 2건 · `mock-exception:` 3건 · **화면 머리의 목업 매핑 2건**(M1 이 읽는다) 보존 — [passes/2026-09-19-candidate-strategy-axis.md](passes/2026-09-19-candidate-strategy-axis.md) · **증분 재판정 ①**(2026-09-21): #93이 `feature_candidates.rs`에 더한 2행(「required-but-nullable — OpenAI `strict` 는 `required` 누락을 400 으로 거부」)은 같은 명제의 정본이 `llm.rs::assert_strict_schema` 옆에 있고(1행 재판정 ⑤) 같은 파일의 테스트 `schema_is_accepted_by_openai_strict_mode` 가 이름으로 그 자리를 가리키므로 **제거 2행**(12차 패스의 「복제된 명제는 강제하는 코드 옆 한 벌만」) — [passes/2026-09-19-candidate-strategy-axis.md](passes/2026-09-19-candidate-strategy-axis.md) 「증분 재판정 ①」 · **증분 재판정 ②**(2026-09-21): #99(`899800e`)가 `sc01-04`(+5) · `sc01-07`(+1) · `DiscoveryStrategy.tsx`(+5) 에 더한 11행을 판정해 **순 제거 10행**(원장 번호 ⑼⒃ 를 현재형으로 인용하는 작업 흔적 · 바로 아래 세 단정이 그 문장 자체인 것 · doc-tracker 변경 이력 ⑼ 를 축자로 옮긴 JSX 머리 · 빈 `catch` + `setInterval` 이 그 문장 자체 · `POLL_MS` 이름·값만의 교차 참조 · 「승인이 4단계를 재큐잉한다」 — diff 기준 11행 삭제 · 1행 재작성) · 유지 1행(폴링을 택한 이유 — `docs/` 어디에도 「왜 한 번 읽지 않고 폴링하는가」 가 없다, 재작성한 1행) · 지문 밖 JSX 블록 연속행 3행도 함께 제거 · 151행 → **141행** — [passes/2026-09-19-candidate-strategy-axis.md](passes/2026-09-19-candidate-strategy-axis.md) 「증분 재판정 ②」 · **미판정 증분 없음** |
| `backend/src/pipeline.rs` · `backend/src/cross_cutting.rs` · `backend/tests/documents.rs` · `backend/tests/progress.rs` · `frontend/src/CrossCuttingConcerns.tsx` · `e2e/tests/sc01-02-repo-out-of-scope.spec.ts` · `e2e/tests/sc01-03-cross-cutting-determinism.spec.ts` · `e2e/tests/sc01-05-resume-after-app-exit.spec.ts` · `e2e/tests/sc02-03-contradiction-separation.spec.ts` (파이프라인 · 횡단 관심사 축 비경합 9파일) | 140 | `03babd782b097f5022f5b81b1954b83ab20584408db686318656aa4cc8b1142a` | 순 제거 198행(AC1.2 조항·시나리오 2·3·5 본문 재진술 · **13벌까지 복제된 워커 임대 문단** · 목업·여정 인용 · 작업 흔적(PR·슬라이스·task id) · 선언·시그니처 재진술 · 구분선 25 · 단정을 산문으로 옮긴 주석 6) · 유지 139행 · 판단이 갈려 남긴 것 5건 · **낡아서 거짓이 된 주석 2건 적발**(`cross_cutting.rs`의 「목업은 4축만 그린다 · 편차 등재됨」 — 목업은 5축을 그리고 doc-tracker는 「등재하지 않는다」고 적는다 / `sc01-05`의 「3 of 5」 — 같은 트리의 `pipeline.rs`는 「다섯 단계 모두 돈다」고 적는다) · 기계 판독 `// 검증 시나리오:` 4건 · `mock-exception:` 2건 · **화면 머리의 목업 매핑 1건**(M1 이 읽는다) 보존 — [passes/2026-09-20-pipeline-cross-cutting-axis.md](passes/2026-09-20-pipeline-cross-cutting-axis.md) · **증분 재판정 ①**(2026-09-20): #83 이 `sc01-02`(+1) · `sc01-05`(+2) 에 연 순증 3행을 판정해 **순 제거 2행**(목업 카피 인용 · 슬라이스 ⑦ 작업 흔적) · 유지 2행(같은 버튼을 연달아 두 번 누르는 코드가 복사 실수로 읽히지 않게 하는 한 줄씩) · 139행이 아니라 **140행**으로 내려온다 — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) · **증분 재판정 ②**(2026-09-20): #85(제품 fix)가 `sc01-02` 에 연 순증 1행을 판정해 **순 제거 1행**(목업 카피 인용 · 바로 아래 세 줄의 단정이 그 문장 자체) · 유지 0행 · 줄 수와 지문이 **둘 다 #85 이전 값으로 되돌아왔다**(140 / `03babd78…` — #85 의 부모 `e340bdc` 실측과 바이트 동일) — [passes/2026-09-20-tools-checker-axis.md](passes/2026-09-20-tools-checker-axis.md) · **미판정 증분 없음** |
| `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일) | 81 | `af9fbb9d706cb48c62e20792bb052363fd2388d88dc43203a5a222a12fddba61` | 순 제거 260행(절 제목 33 + `api.ts` AC 꼬리표 절 제목 5 · 선언·시그니처 재진술 · AC 조항·시나리오·목업 카피 인용 · 작업 흔적(슬라이스 ⑥⑦ · task id · 판정일) · **4벌까지 복제된 인계 계약과 404 명제 중 잉여** — diff 기준 367행 삭제 · 57행 재작성) · 유지 80행 · 판단이 갈려 남긴 것 12건 · **낡아서 거짓이 된 주석 1건 적발**(`SignIn.tsx` 머리가 #83 이 지운 `CredentialsSetup.tsx` 를 현재형으로 서술) · `format.ts` 는 전건 제거로 주석 0행이 되어 지문의 파일 집합에서 빠졌다(8파일 판정 → 지문 `files=7`) · **화면 머리의 목업 매핑 4건**(M1 이 읽는다) 보존 — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) · **증분 재판정 ①**(2026-09-20): #85(제품 fix)가 `HomeRepositories.tsx`(+5) · `index.css`(+1) 에 연 순증 6행을 판정해 **순 제거 6행**(`pick()` JSDoc 4 — 본문 한 줄과 여섯 줄 위 `edit()` 의 재진술 + 목업 `renderHome()` 인용 · `stopPropagation` 인라인 1 · CSS 선택자 재진술 1) · 유지 0행 · 줄 수와 지문이 **둘 다 #85 이전 값으로 되돌아왔다**(80 / `641e9457…` — 부모 `e340bdc` 실측과 바이트 동일) — [passes/2026-09-20-tools-checker-axis.md](passes/2026-09-20-tools-checker-axis.md) · **증분 재판정 ②**(2026-09-21): #91(`3567755`, 반응형 레이아웃)이 `index.css` 의 반응형 블록에 연 순증 5행(물리 9줄)을 판정해 **순 제거 4행**(절 제목 겸 §3.4 축자 재진술 블록 1 — 물리 6줄 · 선택자 재진술 1 · §4.7 축자 1 · §3.4 불릿 축자 1 — 전부 같은 커밋이 신설한 `docs/design-system.md` §3.4·§4.7·§5.4 로 복원) · **유지 1행**(`.screen > .tabbar { animation: none }` 위의 함정 — 진입 애니메이션 `rise` 의 `transform` 이 탭바의 `translateX(-50%)` 를 덮어쓴다; PR #91 본문에도 있으나 「실패 모드의 함정」이라 판단이 갈려 남긴 것 **13건째**) · 유지분이 남아 줄 수·지문은 #91 이전 값(80 / `641e9457…`)으로 **돌아가지 않는다**(81 / `af9fbb9d…`) · 비주석 코드 무접촉(빌드 CSS 산출물 sha256 부모와 동일) — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) 「증분 재판정 — 원장 10행에 #91 이 연 +5행」 · **미판정 증분 없음** |
| `tools/check-mockup-render.py` · `tools/check-journey-mockup.py` · `tools/check-scenario-e2e.py` (`tools/` 체커 축 — 정적 게이트 3파일) | 112 | `8b41c689f8bb34951e205c72d855e4354a6f07fc9569e209ab481bcc1bc41ab7` | 순 제거 101행(형제 게이트 셋이 각자 한 벌씩 적은 SSOT·면제 통로 서술 14 · `data-sample`·`data-variant` 규약 전문 22 — 주석 스스로 `docs/mockups/README.md` 를 복원처로 지목하고 그 포인터의 절 제목마저 이미 낡아 있었다 · 인라인 규칙 마커 21(M0~M7 · R0~R11 · S0~S5 — 파일 머리 목록과 `fail()`·`print()` 문면의 세 번째 벌) · 절 제목 13 · 여정 밖 분기 설명의 다섯 벌 중 셋 7 · M7 블록 머리의 두 번째 벌 5 · 의존성 0·`--verbose` 사용법 5 · R6·R10 규칙 재진술 4 — diff 기준 152행 삭제 · 6행 재작성, 차이는 파이썬 docstring 51행이 지문에 안 보이기 때문) · 유지 112행 · 판단이 갈려 남긴 것 17건 · **파일 머리의 규칙 목록 M0~M7·R0~R11·S0~S5 는 원본이라 보존**(워크플로 셋이 「무엇을 검사하는지는 이 헤더에 있다」로 가리킨다 — 2026-09-18 판정이 `check-journey-prototype.js` 의 P1~P7 을 남긴 것과 같은 결정) · 기계 판독 `// 검증 시나리오:` 선언 1건(S1 항목) 보존 · **AST 동일 3/3** — [passes/2026-09-20-tools-checker-axis.md](passes/2026-09-20-tools-checker-axis.md) |
| `backend/src/github_app.rs` · `backend/src/github.rs` · `backend/src/github_api.rs` · `backend/src/github_tokens.rs` · `backend/src/auth.rs` · `backend/src/session.rs` · `backend/src/cookies.rs` · `backend/src/installations.rs` · `backend/src/users.rs` · `backend/tests/github.rs` · `backend/tests/auth.rs` · `e2e/tests/sc04-01-app-install-and-scope.spec.ts` · `e2e/tests/sc04-11-unauthenticated-block-and-signin.spec.ts` · `e2e/tests/sc04-12-logout-session-invalidation.spec.ts` (GitHub App · 인증 경계 축 14파일) | 71 | `261ad97902dd63b0f049c13eed08d8c02118e7c2074b965ebeb7a99136967259` | 순 제거 152행(선언·시그니처만 영어로 풀어 쓴 `///` 요약 19건 · AC 꼬리표 `(AC4.1)`·`(AC4.3)`·`(AC4.7)`·`(AC4.8)` 전건 · 시나리오 원문 축자 인용과 본문 절 제목 (AC4.8 검증 방법의 복사) · 작업 흔적(분리 이력 · 정정 이력 · `rct_20260916-0002`) · **네 벌까지 복제된 명제 중 잉여** — Setup URL `installation_id` 스푸핑 4벌 · OAuth 토큰 보관 사유 3벌 · 미리보기 state 태깅 3벌 · adoption best-effort 2벌) · 유지 71행 · 판단이 갈려 남긴 것 2건 · **낡아서 거짓이 된 주석 2건 적발**(`github.rs` 의 「distinct users → distinct ids」 — 식은 `rem_euclid(90_000)` 이라 충돌한다 / `tests/auth.rs` 머리가 테스트 넷을 열거하는데 파일에는 여섯 개다) · 기계 판독 `// 검증 시나리오:` 3건 · `mock-exception:` 6건 보존 · **주석 제거 후 부모와 바이트 동일 14/14** — [passes/2026-09-20-github-app-auth-axis.md](passes/2026-09-20-github-app-auth-axis.md) |
| `backend/tests/llmkey.rs` · `e2e/tests/sc04-13-unsupported-provider-rejection.spec.ts` · `e2e/tests/sc04-04-revoked-key-blocks-calls.spec.ts` · `e2e/tests/sc04-03-llm-key-registration.spec.ts` · `e2e/tests/sc04-05-credential-log-exposure.spec.ts` · `backend/tests/security.rs` · `backend/src/audit.rs` · `backend/tests/crypto.rs` (자격증명 · LLM 키 경계 축 8파일 — `backend/src/crypto.rs` 는 **D2 사람 게이트 풀**로 보류) | 42 | `96636e7917d30bcacf740813a2cafe519bba1d3638f2644f38f628029856a22a` | 순 제거 104행(단정·선언 재진술 — 주석 바로 아래 한두 줄이 그 문장 자체인 것 · 시그니처만 영어로 풀어 쓴 `///` 1건 · **테스트 이름을 다시 쓴 파일 머리 `//!` 3건** · 시나리오 원문 축자 인용 2건(시나리오 4·13) · **「자동화 밖 잔여」 4건 전건**(정본은 doc-tracker 「e2e 매핑」의 마지막 열이고 주석 스스로 그렇게 적는다) · AC 꼬리표 전건 · 작업 흔적(분리 이력 `rct_20260916-0002` 3벌 · 슬라이스 ⑦ 3건 · 목업 식별자 인용) · **4벌까지 복제된 명제 중 잉여** — 「App 연결이 선행돼야 한다」 4벌 중 3벌 제거하고 `sc04-03` 을 정본으로 · backdate 사유 2벌 중 뒤의 벌 압축 — diff 기준 125행 삭제 · 18행 재작성, 차이는 불변식·함정 주석을 지우지 않고 되풀이된 절반만 걷어 다시 썼기 때문) · 유지 42행 · 판단이 갈려 남긴 것 2건 · **낡아서 거짓이 된 주석 1건 적발**(`tests/crypto.rs` 머리가 테스트를 셋 열거하는데 파일에는 다섯이다 — `cargo test --test crypto` 5 passed 로 실측; 12차 패스의 `tests/auth.rs` 와 같은 유형의 두 번째 사례) · 기계 판독 `// 검증 시나리오:` 4건 보존(spec 당 정확히 1개) · `backend/tests/crypto.rs` 는 전건 제거로 주석 0행이 되어 지문의 파일 집합에서 빠졌다(8파일 판정 → 지문 `files=7`) · **주석 제거 후 부모와 바이트 동일 8/8** · **필수 status 판정기 `✅ 변경 없음`**(9파일 트리는 D2 로 `⚠️` 였다 — `crypto.rs` 13행은 판정만 마치고 잔여에 남긴다) · `cargo test` 16 passed · 문서 게이트 3종 rc=0 — [passes/2026-09-20-credential-llm-key-axis.md](passes/2026-09-20-credential-llm-key-axis.md) |
| `backend/tests/worker.rs` · `backend/tests/common/mod.rs` · `backend/tests/analyses.rs` · `scripts/e2e.sh` · `e2e/tests/sc02-02-acceptance-from-tests.spec.ts` (테스트 하네스 축 5파일) | 20 | `8b8764df56b28c69c1419f82a0f65c23c0316815315a37e6b0e9d09af544fc9c` | 순 제거 68행(테스트 이름이 그대로 말하는 `///` 10 · 절 제목 7 · 인라인 단정 재진술 5 · 시그니처·이름 재진술 5 · **테스트 이름을 열거한 파일 머리 `//!` 2건 11행**(12·13차가 같은 유형의 낡은 주석을 둘 적발했다) · 시나리오 2 기대 결과 **축자 인용 2벌 4행**(`검증 시나리오:` 마커가 이미 가리킨다) · AC2.2 제목 축자 · `sc02-03` 교차 참조 · spec 11개 열거 · 슬라이스 번호 · **워커 임대 문단 4행 → 1행**(정본은 `e2e/support/cluster.ts`)) · 유지 20행 · 판단이 갈려 남긴 것 **0건** · 기계 판독 `// 검증 시나리오:` 1건 보존 · `backend/tests/analyses.rs` 는 전건 제거로 주석 0행이 되어 지문의 파일 집합에서 빠졌다(5파일 판정 → 지문 `files=4`) · **주석 제거 후 부모와 동일 5/5** · `cargo test --tests` 159 passed(3회 연속) · 문서 게이트 3종 rc=0 · **판정기 `✅ 변경 없음`** · **원장 정정: 후보 ① `tools/check-data-format-change.py` 는 D6 로, `deploy/k8s/pvc.yaml` 은 D5 로 경로 매칭돼 무인 머지 경로가 없다**(음성 대조 실측) — [passes/2026-09-20-test-harness-axis.md](passes/2026-09-20-test-harness-axis.md) |
| `deploy/k8s/kustomization.yaml` · `backend/src/util.rs` · `backend/src/main.rs` · `backend/src/error.rs` · `backend/src/state.rs` (API 셸 · 배포 베이스 축 5파일 — 잔여의 무인 자유 풀 전량) | 19 | `3b0735c725531481568605ca1a47b0573a706afce46b723edfcc66ab382db4e7` | 순 제거 19행(`kustomization.yaml` 머리 **15행 전건** — `README.md` §배포·§CI 가 문장 단위로 되풀이하는 것(핀 파이프라인 6 · 두 워크로드 3 · secret 외부 제공 4)과 빈 주석 행 2 · 이름·시그니처를 영어로 옮긴 `///` 3행(`util.rs`) · `error.rs` 모듈 머리의 **동작 서술 3행 → 불변식 2행 재작성**) · 유지 19행(PID 1 시그널 함정 5 · OAuth `state` 접두사의 CSRF 불변식 8 · 모듈 머리 셋 4 · `error.rs` 불변식 2) · 판단이 갈려 남긴 것 **1건**(`main.rs` 의 PID 1 함정은 `bin/worker.rs:118-122` 와 같은 명제의 두 번째 벌 — **4차 패스가 그 벌을 「유지」로 닫았고**, 복제 정리는 복원 경로 넷에 없는 재량이며 두 바이너리는 독립 표면이라 전건 유지. 뒤집으려면 `worker.rs` 를 포함한 증분 재판정으로 한 번에) · `deploy/k8s/kustomization.yaml` 은 전건 제거로 주석 0행이 되어 지문의 파일 집합에서 빠졌다(5파일 판정 → 지문 `files=4`) · **주석 제거 후 부모와 바이트 동일 3/3** · 비주석 diff 0줄 · `cargo test --tests` 159 passed · 문서 게이트 4종 rc=0(`check-journey-mockup` · `check-mockup-render` · `check-scenario-e2e` · `check-journey-prototype`) · 허브 `Documents` 40 → 41 · **판정기 `✅ 변경 없음`** · **이 패스 뒤 무인 자유 풀은 0** — 남는 242행은 전부 사람 게이트 3몫이다(단 #92·#93 이 머지되면 새 주석이 들어와 다시 열린다) — [passes/2026-09-20-api-shell-deploy-base-axis.md](passes/2026-09-20-api-shell-deploy-base-axis.md) |

**합계**: 판정 **98파일**(지문의 파일 집합 기준으로는 94 — `format.ts` · `backend/tests/crypto.rs` ·
`backend/tests/analyses.rs` · `deploy/k8s/kustomization.yaml` 이 전건 제거로 주석 0행이 되어 빠졌다.
**파일 수는 지문의 `files` 가 아니라 행의 목록 길이로 센다**) · 순 제거 누적 **1,944행**(직전 1,940 +
증분 재판정 4 — 새 범위 없음, #91 의 5행 중 제거 4 · 유지 1) · 판정 범위의 현재 합계 **2,076행**
(전역 `lines=2318` − 잔여 242). **그중 판정 완료는 2,074행**(직전 2,073 + #91 유입 5 − 제거 4)이고
**미판정 증분 2행**이 판정 완료 범위 안에 들어와 있다 — **#101(`3d147d6`)이 7행 `AnalysisProgress.tsx`
에 더한 2행**(#99 재판정의 머지 직전에 들어와 7행 계수에는 포함됐으나 미판정 — 후속 재감지 몫).
**#91(`3567755`)이 10행 `frontend/src/index.css` 에 더한 5행**은 이 재판정(증분 재판정 ②, 제거 4 · 유지 1)이
닫았고 10행의 줄 수·지문은 80/`641e9457…` → 81/`af9fbb9d…` 로 갱신됐다. #99 의 25행은 3행(`sc01-01` 5 ·
`sc01-06` 5 = 10) · 7행(`AnalysisProgress.tsx` 4) · 8행(`sc01-04` 5 · `sc01-07` 1 · `DiscoveryStrategy.tsx`
5 = 11)에 걸쳐 있었고(그 직전 판의 「3행 14 · 8행 11」은 7행 몫 4 를 3행에 잘못 얹은 것이다) 직전 재판정이
셋을 한 패스로 닫았다. 행 열의 합은 2,076 = 판정 완료 2,074 + 7행 안의 #101 미판정 2(7행은 그 병합 트리
재실측값으로 이미 +2 됐다).

**미판정 잔여**: **14파일 / 242행**
(이 재판정 병합 후 트리 기준 — 전역 `lines=2318 files=108`). 전역 지문의 파일 목록에서 판정 98파일을
**집합으로 뺀** 값이며, 뺄셈과도 일치한다(242 == 2,318 − 2,076). #93·#91·#99·#101 은 새 파일을 들여오지
않았으므로 이 잔여는 15차 패스 이후 한 줄도 움직이지 않았다.

**잔여(행 없는 파일)는 세 몫뿐이고, 셋 다 사람 게이트다**(8 + 4 + 2 = 14파일 · 139 + 44 + 59 = 242행).
**무인 자유 풀은 판정 완료 범위 안의 미판정 증분 2행뿐이다**(위 합계 — 15차 패스가 「#92·#93 이
머지되면 다시 열린다」고 적어 둔 조건이 #93·#99·#91 로 발효됐고, 그 9행·25행·5행은 세 재판정이 닫았다.
이 조건은 PR 번호에 묶이지 않는다 — **자매 모델의 수렴 슬라이스가 e2e·프런트를 다시 쓸 때마다** 판정
완료 범위 안에 증분이 생기고, #91·#99 가 그 형태였다).

- **`backend/migrations/*.sql` 8파일 / 139행** — 본문 「적용된 마이그레이션」 절의 전용 PR ·
  수동 repair · 사람 승인 게이트를 거치는 **별도 패스**다. 다른 정리와 섞지 않는다. 12차 패스가
  `0002_github_tokens.sql:3` 에서 「Setup URL 의 `installation_id` 는 스푸핑 가능」의 **4벌째**를
  확인했으므로, 그 패스는 12차가 정한 정본(`github_app.rs`)을 이어받으면 된다.
- **D2 「저장 계층 핵심」 4파일 / 44행 — `backend/src/db.rs` 17 · `backend/src/crypto.rs` 13 ·
  `backend/tests/migrations.rs` 13 · `backend/src/models.rs` 1** (다섯 번째인
  `backend/src/pipeline.rs` 는 범위 안 주석이 0행이라 잔여에 없다). `crypto.rs` 13행은 13차 패스가
  **판정까지 마쳐 뒀다**(제거 6 · 유지 7 —
  [passes/2026-09-20-credential-llm-key-axis.md](passes/2026-09-20-credential-llm-key-axis.md)
  「보류분의 판정 결과」).
- **D6 · D5 경로 규칙 2파일 / 59행 — `tools/check-data-format-change.py` 58 ·
  `deploy/k8s/pvc.yaml` 1.** 판정기 자신의 **D6**(`SELF_PATHS`)과 **D5** `is_pvc()` 에 **경로로**
  걸린다. D3 와 달리 이 규칙들에는 「주석 아닌 줄」 예외가 없어 **주석 한 줄만 고쳐도**
  `needs_review=true` 가 되고 필수 체크 `review/data-format` 이 붙지 않는다 — D2 와 같은 벽이다.
  14차 패스가 음성 대조로 실측했다(둘 다 `⚠️ 사람 리뷰 필요`, 같은 트리의 5파일은 `✅ 변경 없음`).

**다음 무인 패스의 후보 풀은 위 미판정 증분 2행이다**(#101 2행 → 7행, #99 재판정 직전에 머지돼
다음 재감지가 연다 — #91 의 5행은 이 재판정이 닫았다).
열린 PR #92 는 `backend/src/doc_edit.rs` · `backend/tests/doc_edit.rs` 등 **새 파일**을
들여놓으므로 머지되면 잔여(행 없는 파일)도 다시 늘어난다. **「자유 풀」의 수치는 이 트리에서만 참인
조건부 진술이다** — 다음 감지는 절대 수치를 믿지 말고 **전역 지문의 파일 목록에서 위 98파일을
집합으로 빼서** 잔여를, 행 열의 합과 「전역 − 잔여」의 차로 미판정 증분을 다시 계산해야 한다.
증분 밖에서 진짜로 남은 일은 **마이그레이션 축의 사람 게이트 패스**다(139행, 위 첫 몫 — PR #98).

**슬라이스 전 필수 절차 — 판정기를 돌린다.** 원장이 다음 축을 파일·행수까지 지목해도, 집기 전에
후보 트리에서 `python3 tools/check-data-format-change.py --base <main tip> --head <probe> --verbose`
를 돌려 **`✅ 변경 없음`** 을 확인한다. 경로 규칙은 D2 하나가 아니다(D1 · D2 · D5-pvc · D6) —
「D2 목록에 없다」는 충분한 근거가 아니고, 판정기 출력만이 충분하다. 14차 패스가 원장의 후보 ①에서
이것으로 벽을 미리 찾아냈다.

**경합 0 · 열린 PR 3건** (2026-09-20T16:2xZ `/pulls?state=open` 전수 재실측 — #91 반응형 레이아웃 ·
#92 슬라이스 6a · #93 OpenAI strict fix, 셋 다 base `f5a2937`). 각 PR 의 `/pulls/<n>/files` 를
판정 5파일과 대조해 **겹침 0**. ⚠️ `#92` 의 `deploy/e2e/kustomization.yaml` 은 이 패스의
`deploy/k8s/kustomization.yaml` 과 **다른 파일**이다. 셋 다 in-scope 주석 파일을 건드리므로 **먼저
머지되면 전역 지문의 절대값은 움직인다** — 그래서 이 패스의 완료 기준도 절대 지문이 아니라
**「부모 `445ec57` 대비 순 제거 19행」**이고, 위 15행의 범위 지문은 이 5파일만의 값이라 자매 머지에
무관하다.

그래서 **`backend/src` 도 `deploy/k8s` 도 닫히지 않았다.** `backend/src` 에는
`db.rs`·`crypto.rs`·`models.rs`(D2)가, `deploy/k8s` 에는 `pvc.yaml`(D5)이 남아 있으며, 열린 PR
#92·#93 이 `backend/src` 에 파일과 주석을 더하는 중이다. **디렉터리 단위 종료를 선언하지 않는다** —
11차 패스가 `tools/` 에서, 14차가 `backend/tests` 에서 배운 그대로다.

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

> **6차 패스 기준 갱신 (2026-09-19 · `rct_20260919-0002`).** 위 블록들의 수치는 각 패스의 판정
> 시점 값이라 그대로 두고, 현재 기준만 여기 적는다.
>
> - **5차 패스의 전역 예고는 빗나갔고, 그것은 패스의 오류가 아니다.** 예고 `lines=2920 files=97` /
>   `d4c000e4…`는 부모 `fae3e17` 기준이었는데 그 뒤 #67·#69·#71·#73·#74가 끼었다. 이 패스의 부모
>   `66bb7a5`에서 재측정한 실측은 **`lines=3365 files=106` / `a43e22ba…`**이고, 5차 패스 자신의
>   기여(−135행)는 예고와 정확히 일치한다. **#74는 in-scope 주석에 무접촉**이라 `f357f12`와 지문이
>   바이트 동일이다.
> - **이 패스의 완료 기준은 절대 지문이 아니라 「부모 대비 순 제거 **162행**」이다**
>   (의존성 축 137 + `worker_api.rs` 증분 18 + `bin/worker.rs` 증분 7). 판정 시점 부모 `66bb7a5`
>   기준 절대값은 `lines=3203 files=106` /
>   `afd4b8e3ca44ec129b833a18c8c9d506f664148021b50ef990eb88328f398714`이지만, 열린 #75·#72가 먼저
>   머지되면 절대값은 그만큼 움직인다 — 그때도 순 제거 162행은 그대로다.
> - **원장 1행의 증분은 절반만 닫혔다.** #71이 더한 73행 중 `worker_api.rs` 37행은 위 증분 재판정
>   ③이 닫았고, `analysis.rs` 36행은 **#75 경합**이라 열려 있다. 1행의 현재 값은 592행 /
>   `b03aac73…`이며, 이 36행이 판정될 때까지 1행은 「증분 미판정」 상태다.
> - **원장 5행의 12행 증분은 이 패스가 닫았다** — 위 증분 재판정 ①. 5행의 현재 값은 190행 /
>   `632b0475…`다.
> - **원장 2·3·4행은 현재 트리 재계산에서 바이트 동일**이라 재판정이 필요 없다
>   (140 / `ddacce0b…` · 142 / `fb27e0db…` · 141 / `e38dcf20…`).
> - 지문 계산 규약은 앞 블록과 같다: 원장의 **범위 지문**은 후행 개행을 **포함**하고
>   (`echo "$HITS" | sha256sum`), 모델의 **전역 지문**은 versionScript 그대로 후행 개행을
>   **제외**한다(`printf '%s'`). 같은 입력에도 두 값은 다르다.

> **7차 패스 기준 갱신 (2026-09-19 · `rct_20260919-0003`).** 위 블록들의 수치는 각 패스의 판정
> 시점 값이라 그대로 두고, 현재 기준만 여기 적는다.
>
> - **6차 패스의 기여는 예고와 정확히 일치했다.** 순 제거 162행 — 부모 `66bb7a5` 에서 재측정한
>   `lines=3365 files=106` 이 #72(+25) · #75(+236 · +4파일) · #76(−162) 를 거쳐 main tip `f298ecb`
>   에서 `lines=3464 files=110` / `0f8cdcef…` 다. 절대 예고값 `lines=3203` 은 자매 둘이 먼저
>   머지돼 지났지만, **순 제거로 적은 완료 기준은 그대로 맞았다**.
> - **이 패스의 완료 기준도 절대 지문이 아니라 「부모 `f298ecb` 대비 순 제거 **197행**」이다**
>   (재분석 diff 축 168 + `analysis.rs` 증분 재판정 29). 판정 시점 절대값은
>   `lines=3267 files=110` / `fdb14462ceb06069dd97062ff9095b6338e03fe63af165ade0111c4c91750a78`
>   이지만, 열린 #77·#64·#26·#17 중 어느 것이 먼저 머지되면 그만큼 움직인다.
> - **원장 안의 미판정 증분이 0이 됐다.** 1행의 `analysis.rs` 60행을 증분 재판정 ④ 가 닫았고,
>   2·3·4·5·6행은 현재 트리 재계산에서 **바이트 동일**이라 재판정이 필요 없다
>   (140 / `ddacce0b…` · 142 / `fb27e0db…` · 141 / `e38dcf20…` · 190 / `632b0475…` ·
>   112 / `e2ff313a…`).
> - **지문에 보이는 기계 판독 주석이 하나 더 있다는 것을 이번 패스가 찾았다** —
>   `frontend/src/*.tsx` 머리의 `docs/mockups/<파일>.html#STP-<앵커>` 매핑 줄은
>   `tools/check-mockup-render.py::discover_screens()` 가 앞 2,000자에서 읽는 **M1 의 입력**이다.
>   `// 검증 시나리오:`·`mock-exception:` 과 달리 모델의 제외 패턴에 없어 지문에 들어와 있고,
>   지우면 그 화면이 M1 의 발견 집합에서 빠져 게이트가 붉어진다. 현재 매핑을 가진 tsx 는 10개.
>   **제거 후보로 보지 않는다.** 제외 패턴에 더할지는 모델 정의를 바꾸는 control plane 작업이라
>   data plane 패스가 단독으로 하지 않는다 —
>   [passes/2026-09-19-reanalysis-diff-axis.md](passes/2026-09-19-reanalysis-diff-axis.md) 「발견」.
> - 지문 계산 규약은 앞 블록과 같다: 원장의 **범위 지문**은 후행 개행을 **포함**하고
>   (`echo "$HITS" | sha256sum`), 모델의 **전역 지문**은 versionScript 그대로 후행 개행을
>   **제외**한다(`printf '%s'`). 같은 입력에도 두 값은 다르다.

> **8차 패스 기준 갱신 (2026-09-19 · `rct_20260919-0004`).** 위 블록들의 수치는 각 패스의 판정
> 시점 값이라 그대로 두고, 현재 기준만 여기 적는다.
>
> - **7차 패스의 기여는 예고와 정확히 일치했다.** 순 제거 197행 — 부모 `f298ecb` 의
>   `lines=3464 files=110` 이 #78 머지 뒤 main tip `e963a5f` 에서 `lines=3289 files=111` /
>   `bc68c231…` 이다. 차이 175행은 **#77(목업 수렴 ⑤)이 먼저 머지돼 +22행 +1파일
>   (`SignIn.tsx`)을 더했기 때문**이고, 7차 패스 자신의 −197 은 그대로다. 절대 예고값
>   `lines=3267` 은 그 자매 머지로 지났다 — **순 제거로 적은 완료 기준만 두 번 다 맞았다.**
> - **이 패스의 완료 기준도 절대 지문이 아니라 「부모 `e963a5f` 대비 순 제거 **222행**」이다.**
>   판정 시점 절대값은 `lines=3067 files=111` /
>   `df0f9019d9423520ff0c4189e5fd43a5b2e5a9d37de905f6d86c7c0b0728c403` 이지만, 열린
>   #79·#64·#26·#17 중 어느 것이 먼저 머지되면 그만큼 움직인다.
> - **원장 안의 미판정 증분은 여전히 0이다.** 1~7행 전건을 `e963a5f` 에서 재계산해 줄수·지문이
>   **7/7 바이트 동일**임을 확인했다(587 / `6b77b15a…` · 140 / `ddacce0b…` · 142 / `fb27e0db…` ·
>   141 / `e38dcf20…` · 190 / `632b0475…` · 112 / `e2ff313a…` · 132 / `a0354a08…`).
> - **다음 증분은 #79 가 연다.** #79(목업↔구현 수렴 ⑥)가 `e2e/tests/sc01-01-full-pipeline-run.spec.ts`
>   를 `+6 −4` 로 건드리므로, 머지되면 **원장 3행에 미판정 증분이 열린다**. 이 패스의 범위 밖이고
>   다음 감지가 그 이동을 발화시킨다.
> - **경합 집합은 감지 브리프와 다르다.** detector 가 넘긴 「경합 5파일 / 212행」은 #79 가 열리기
>   전 값이다. 계획 시점 전수 실측은 **7파일 / 296행**이다 — 열린 PR 목록은 계획 직전에 다시
>   재는 것이 규칙이다.
> - 지문 계산 규약은 앞 블록과 같다: 원장의 **범위 지문**은 후행 개행을 **포함**하고
>   (`echo "$HITS" | sha256sum`), 모델의 **전역 지문**은 versionScript 그대로 후행 개행을
>   **제외**한다(`printf '%s'`). 같은 입력에도 두 값은 다르다.

> **9차 패스 기준 갱신 (2026-09-20 · `rct_20260919-0005`).** 위 블록들의 수치는 각 패스의 판정
> 시점 값이라 그대로 두고, 현재 기준만 여기 적는다.
>
> - **8차 패스의 기여는 예고와 정확히 일치했다.** 순 제거 222행 — 부모 `e963a5f` 의
>   `lines=3289 files=111` 이 #80 머지 뒤 `lines=3067 files=111` / `df0f9019…` 로, 8차 패스가
>   적은 절대 예고값과 **바이트 동일**하게 착지했다. 그 뒤 #81(+7 · `check-journey-mockup.py`)과
>   **#79(목업↔구현 수렴 ⑥)** 가 끼어 이 패스의 부모 `48a4a87` 에서는
>   **`lines=3086 files=110` / `701de5be…`** 다. 파일이 하나 줄어든 것은 #79 가
>   `ConnectRepository.tsx` 를 `HomeRepositories.tsx` 로 병합했기 때문이다.
> - **이 패스의 완료 기준도 절대 지문이 아니라 「부모 `48a4a87` 대비 순 제거 **201행**」이다**
>   (새 범위 9파일 198 + 원장 3행 증분 재판정 3). 판정 시점 절대값은 `lines=2885 files=110` /
>   `d9be3f73e9d286504b208e95cb6a01294f3ebbc3f8eb400baf56e7b20aea4bac` 이지만, 열린
>   #64·#26·#17 중 어느 것이 먼저 머지되면 그만큼 움직인다.
> - **8차 패스가 예고한 「#79 가 여는 원장 3행 증분」이 실제로 열렸고, 이 패스가 닫았다.**
>   3행은 142행 → **145행**(#79 의 순증 3) → 증분 재판정 ① 로 **142행**이다. **지문은 #79
>   이전 값(`fb27e0db…`)으로 돌아가지 않는다** — `0e5c3d31…` 다. 줄 수 일치를 원상 복구로
>   읽지 말 것.
> - **원장 1·2·4·5·6·7·8행은 부모 트리 재계산에서 바이트 동일**이라 재판정이 필요 없었다
>   (587 / `6b77b15a…` · 140 / `ddacce0b…` · 141 / `e38dcf20…` · 190 / `632b0475…` ·
>   112 / `e2ff313a…` · 132 / `a0354a08…` · 140 / `d46c9262…`).
> - **`docs/**.md` 를 더하는 패스는 허브 등재가 함께 가야 한다.** 이 패스가 `passes/` 에 판정
>   상세를 하나 더하면서 `docs/index.html` 에 링크를 넣고 `Documents` 집계를 **34 → 35** 로
>   올렸다. 빠뜨리면 `check-journey-mockup.py` 의 **R8·R9 가 3건 실패**한다(실제로 한 번
>   관측하고 고쳤다). 허브는 주석 지문 범위 밖이라 판정 수치에는 영향이 없다.
> - **낡아서 거짓이 된 주석이 두 건 나왔다** — 제거 근거를 강화하는 유형이다.
>   `cross_cutting.rs` 머리는 「목업이 5축 중 4축만 그리고 그 편차가 등재돼 있다」고 적었지만
>   목업은 5축을 그리고(`JRN-discover-features.html:1048`) `docs/doc-tracker/2026-08.md` 는
>   「이 항목은 원장에 등재하지 않는다」고 적는다. `sc01-05` 는 「3 of 5」라 적었는데 같은
>   트리의 `pipeline.rs` 는 「다섯 단계 모두 돈다」고 적는다.
> - **`.rs` 파일의 목업 URL 은 게이트 입력이 아니다.** `check-mockup-render.py::discover_screens()`
>   는 `frontend/src/*.tsx` 만 훑으므로(`SRC_DIR`), `pipeline.rs` 머리의
>   `docs/mockups/…#STP-leave-and-return` 인용은 지워도 M1 이 영향을 받지 않는다. tsx 머리의
>   매핑은 **여전히 M1 의 입력이므로 제거 후보가 아니다**.
> - 지문 계산 규약은 앞 블록과 같다: 원장의 **범위 지문**은 후행 개행을 **포함**하고
>   (`echo "$HITS" | sha256sum`), 모델의 **전역 지문**은 versionScript 그대로 후행 개행을
>   **제외**한다(`printf '%s'`). 같은 입력에도 두 값은 다르다.

> **10차 패스 기준 갱신 (2026-09-20 · `rct_20260920-0002`).** 위 블록들의 수치는 각 패스의 판정
> 시점 값이라 그대로 두고, 현재 기준만 여기 적는다.
>
> - **9차 패스의 기여는 예고와 정확히 일치했다.** 순 제거 201행 — 부모 `48a4a87` 의
>   `lines=3086 files=110` 이 #82 머지 뒤 `lines=2885 files=110` / `d9be3f73…` 로, 9차 패스가
>   적은 절대 예고값과 **바이트 동일**하게 착지했다. 그 뒤 **#83(목업↔구현 수렴 ⑦)** 이 끼어
>   이 패스의 부모 `79c1cb4` 에서는 **`lines=2944 files=111` / `60ff1f0f…`** 다. 파일이 하나
>   늘어난 것은 #83 이 `CredentialsSetup.tsx` 를 `GrantRepoAccess.tsx` + `RegisterLlmKey.tsx`
>   **둘로 쪼갰기** 때문이다(순증 59행 = 추가 107 − 제거 48).
> - **이 패스의 완료 기준도 절대 지문이 아니라 「부모 `79c1cb4` 대비 순 제거 **264행**」이다**
>   (새 범위 8파일 260 + 원장 3행 증분 2 + 원장 9행 증분 2). 판정 시점 절대값은
>   `lines=2680 files=110` /
>   `fc595b135d11891f70047d2232698ef69d0f3336163cf469002f9e7c59cfffff` 이지만, 열린 #64 가 먼저
>   머지되면 절대값은 그만큼 움직인다 — 그때도 순 제거 264행은 그대로다.
> - **9차 패스가 예고하지 않은 증분이 둘 열렸고, 이 패스가 둘 다 닫았다.** #83 은 `sc01-01`
>   (원장 3행, +2)과 `sc01-02`·`sc01-05`(원장 9행, +3)를 함께 건드렸다. **두 행 모두 판정 전
>   줄 수로 돌아가지 않는다** — 3행은 142행이되 지문이 `0e5c3d31…` → `93be69ea…` 이고, 9행은
>   139행이 아니라 **140행** / `03babd78…` 이다.
> - **원장 1·2·4·5·6·7·8행은 부모 트리 재계산에서 바이트 동일**이라 재판정이 필요 없었다
>   (587 / `6b77b15a…` · 140 / `ddacce0b…` · 141 / `e38dcf20…` · 190 / `632b0475…` ·
>   112 / `e2ff313a…` · 132 / `a0354a08…` · 140 / `d46c9262…`).
> - **판정 범위가 한 디렉터리를 처음으로 닫았다.** `frontend/src` 는 이 패스로 전량 판정
>   완료다(주석 0행인 `main.tsx`, 기계 판독뿐인 `vite-env.d.ts` 포함). 6·9차 패스가 예고한
>   「프런트 데이터 계층」 후보 6파일에 `SignIn.tsx`(12) · `format.ts`(5)를 더해 17행을 더
>   넣은 결과이고, 그 덕에 다음 패스의 범위가 「나머지 조각」이 되지 않는다.
> - **`format.ts` 는 판정 후 주석이 0행이 되어 지문의 파일 집합에서 빠진다.** 8파일을 판정했는데
>   원장 행의 지문은 `files=7` 이다. **판정 파일 수와 지문 파일 수를 같은 값으로 읽지 말 것.**
> - **`docs/**.md` 를 더하는 패스는 허브 등재가 함께 간다.** 이 패스가 `passes/` 에 판정 상세를
>   하나 더하면서 `docs/index.html` 에 링크를 넣고 `Documents` 집계를 **35 → 36** 으로 올렸다.
>   빠뜨리면 `check-journey-mockup.py` 의 R8·R9 가 실패한다. 허브는 주석 지문 범위 밖이라 판정
>   수치에는 영향이 없다.
> - **`index.css:48` 의 `* {`(전역 선택자)는 지문이 주석으로 세는 오탐이다.** 패턴
>   `\*([[:space:]]|$)` 에 걸린다. 결정적이라 무해하고 **코드이므로 건드리지 않았다** — 이
>   패스 뒤 `index.css` 의 지문 3행 중 한 행이 이 줄이다.
> - 지문 계산 규약은 앞 블록과 같다: 원장의 **범위 지문**은 후행 개행을 **포함**하고
>   (`echo "$HITS" | sha256sum`), 모델의 **전역 지문**은 versionScript 그대로 후행 개행을
>   **제외**한다(`printf '%s'`). 같은 입력에도 두 값은 다르다.

> **11차 패스 기준 갱신 (2026-09-20 · `rct_20260920-0003`).** 위 블록들의 수치는 각 패스의 판정
> 시점 값이라 그대로 두고, 현재 기준만 여기 적는다.
>
> - **10차 패스의 기여는 예고와 정확히 일치했다.** 순 제거 264행 — 부모 `79c1cb4` 의
>   `lines=2944 files=111` 이 #84 머지 뒤 `lines=2680 files=110` / `fc595b13…` 로, 10차 패스가
>   적은 절대 예고값과 **바이트 동일**하게 착지했다. 그 뒤 **#86**(목업↔구현 수렴 ⑧, `tools/
>   check-mockup-render.py` +16)과 **#85**(제품 fix, +7)가 끼어 이 패스의 부모 `7a232f9` 에서는
>   **`lines=2703 files=110` / `c7f7eb98…`** 다. 자매 둘 다 to-be(`docs/comment-policy/`)에는
>   무접촉이라 as-is 만 움직였다.
> - **이 패스의 완료 기준도 절대 지문이 아니라 「부모 대비 순 제거 **108행**」이다**
>   (새 범위 3파일 101 + 원장 9행 증분 1 + 원장 10행 증분 6).
> - **그 「절대값이 아니라 순 제거」가 이번에 실제로 값을 구했다.** 계획 중에 **#64 가 머지돼**
>   부모가 `7a232f9` → `b2724da` 로 움직였고, 계획 시점에 적었던 절대 예고값
>   `lines=2595 files=110` / `b7421e6a…` 는 그 순간 지났다. 리베이스 후 재측정한 값은
>   부모 `b2724da` `lines=2761 files=111` / `75496d07…` → 이 패스 뒤
>   **`lines=2653 files=111` / `15cb6b241b0c348328fa63c9b01ddd6cc6635651062d0a4393d0658cb59ac479`**
>   이고, **차는 정확히 108행**이다. 예고했던 이동폭(+59)도 한 줄 틀렸다 — 실제는 **+58** 이다.
>   신규 파일의 shebang 이 모델의 제외 패턴(`:#!/`)에 걸려 지문에서 빠지기 때문이고, 그
>   패턴은 **경로 접두사를 포함한 줄**에 대고 맞추므로 파일 내용만 보고 세면 한 줄이 남는다.
> - **원장 1~8행은 부모 트리 재계산에서 8/8 바이트 동일**이라 재판정이 필요 없었다
>   (587 / `6b77b15a…` · 140 / `ddacce0b…` · 142 / `93be69ea…` · 141 / `e38dcf20…` ·
>   190 / `632b0475…` · 112 / `e2ff313a…` · 132 / `a0354a08…` · 140 / `d46c9262…`).
> - **9·10행은 줄 수와 지문이 *둘 다* 판정 전 값으로 되돌아왔다 — 이 원장에서 처음이다.**
>   9·10차 패스는 「줄 수 일치를 원상 복구로 읽지 말 것」이라 적었고 그 경고는 여전히 옳다.
>   이번에 복구라고 단정할 수 있는 근거는 줄 수가 아니라 **#85 의 부모 `e340bdc` 에서 같은 범위를
>   재계산한 값과의 바이트 동일**이다(140 / `03babd78…` · 80 / `641e9457…`). #85 가 그 두 범위에서
>   건드린 것이 주석 7행뿐이었고 그 7행이 전건 제거됐기 때문이다. **판정 전 값과 같아 보이면
>   부모 트리에서 재측정해 확증할 것** — 줄 수만 보고 판단하지 말 것.
> - **「디렉터리를 닫았다」는 그 시점 트리에서만 참이다.** 이 패스는 `tools/` 의 판정 대상
>   4파일 전량을 닫았지만 **같은 사이클에 #64 가 다섯 번째 파일을 들여놓아 잔여에 남았다**
>   (58행). 10차 패스가 `frontend/src` 에 적은 「디렉터리 단위 종료」도 같은 조건부로 읽어야
>   한다 — 다음 패스가 「이미 닫힌 디렉터리」로 건너뛰면 그 사이 들어온 파일을 놓친다.
> - **파이썬 `"""docstring"""` 은 지문에 보이지 않지만 판정 대상이다.** 모델의 추출 패턴
>   (`^\s*(//|#|/\*|\*(\s|$)|\{/\*)`)이 `"""` 로 시작하는 줄을 잡지 못하므로, 이 패스가 지운
>   docstring 51행은 지문 감소(101)에 한 줄도 기여하지 않는다. 지문이 못 보는 자리라고 남기면
>   같은 중복이 그 자리에 계속 쌓이므로 판정해 지웠고, 그 차이를 패스 상세의 집계 표에 적었다.
>   **주석 정리의 검증을 「비주석 diff 0줄」로 하지 말 것** — 주석에 딸려 지워진 선언을 놓친다.
>   이 패스는 **AST 동일**(`ast.dump`, docstring Expr 만 정규화)로 세 파일을 확인했다.
> - **`docs/**.md` 를 더하는 패스는 허브 등재가 함께 간다.** 이 패스가 `passes/` 에 판정 상세를
>   하나 더하면서 `docs/index.html` 에 링크를 넣고 `Documents` 집계를 **36 → 37** 로 올렸다.
>   빠뜨리면 `check-journey-mockup.py` 의 R8·R9 가 실패한다. 허브는 주석 지문 범위 밖이라 판정
>   수치에는 영향이 없다.
> - 지문 계산 규약은 앞 블록과 같다: 원장의 **범위 지문**은 후행 개행을 **포함**하고
>   (`echo "$HITS" | sha256sum`), 모델의 **전역 지문**은 versionScript 그대로 후행 개행을
>   **제외**한다(`printf '%s'`). 같은 입력에도 두 값은 다르다.

> **12차 패스 기준 갱신 (2026-09-20 · `rct_20260920-0005`).** 위 블록들의 수치는 각 패스의 판정
> 시점 값이라 그대로 두고, 현재 기준만 여기 적는다.
>
> - **11차 패스 이후 main 이 움직이지 않았다 — 이 원장에서 처음이다.** 부모 `f5a2937` 에서 잰
>   전역 지문이 11차 패스가 적어 둔 착지값과 **바이트 동일**하다
>   (`lines=2653 files=111` / `15cb6b24…`). 그래서 원장 1~11행 전건이 재계산에서 **11/11 바이트
>   동일**이고 증분 재판정이 한 건도 없었다. 다음 패스가 같은 상황을 기대해서는 안 된다 —
>   **열린 PR 이 0건이 아니라 3건**이고, 셋 다 in-scope 주석 파일을 건드린다.
> - **이 패스의 완료 기준도 절대 지문이 아니라 「부모 대비 순 제거 **152행**」이다**
>   (전부 새 범위 14파일, 증분 재판정 0). 착지값은
>   **`lines=2501 files=111` / `4e81c916e413f7badaa79eac340f5cd6685857ec2e5e90b0271e0a36ccac8dd8`**
>   이지만, #91·#92·#93 중 하나라도 먼저 머지되면 그 절대값은 지난다. **순 제거 152 는 그래도
>   불변이다** — 셋 다 이 14파일과 겹침이 0 이다.
> - **지문 감소와 diff 삭제 줄 수가 이번엔 같다(둘 다 152).** 11차 패스에서 둘이 갈렸던 이유
>   (파이썬 docstring 이 지문에 안 보인다)가 이 범위엔 없다 — Rust·TypeScript 뿐이고 제거·재작성이
>   전부 줄머리 `//`·`///`·`//!` 이다. **파일 수도 111 로 불변이다** — 주석이 0행이 된 파일이
>   없어 10차 패스의 `format.ts` 같은 어긋남이 이번엔 생기지 않았다.
> - **검증은 AST 가 아니라 「주석 제거 후 바이트 동일」로 했다.** 이 범위엔 파이썬이 없어
>   `ast.dump` 를 쓸 수 없다. 문자열·문자 리터럴을 인식하는 stripper 로 줄 주석(`//`·`///`·`//!`)과
>   블록 주석을 걷어낸 뒤 부모와 바이트 비교해 **14/14 IDENTICAL** 을 얻었다. 「비주석 diff 0줄」은
>   주석 재작성에 딸려 사라진 선언을 놓치므로 쓰지 않았다(10차 패스의 경고).
> - **복제된 명제는 「정본을 어디에 둘 것인가」로 판정했다.** 이 축에서 네 벌까지 복제된 명제가
>   여섯 건 나왔고(Setup URL 스푸핑 4 · OAuth 토큰 보관 사유 3 · 미리보기 state 태깅 3 ·
>   adoption best-effort 2 · 두 번째 설치 유도 2 · 기본 stub 사용자 소유 3), 전부 **그 명제를
>   강제하는 코드 옆**을 정본으로 골랐다. 테스트 쪽 복사본은 대개 fn 이름이 이미 복원한다.
> - **딱 한 명제만 두 벌을 일부러 남겼다** — 「상류 실패를 고정 문자열로 사상하고 끼워 넣지
>   않는다」. `github_app.rs` 와 `github_api.rs` 는 각자 자기 `map_err` 에서 다른 비밀을 버리므로
>   (App JWT·설치 토큰·개인키 / client secret·OAuth code·access token), 정책이 유지 대상으로
>   이름 붙인 「불변식이 **왜 그 자리에서** 지켜져야 하는지」에 해당한다.
> - **선례와 어긋나는 판정은 하지 않았다.** 세 spec 의 「Runs against the e2e deployment
>   (FEATUREDOC_DOUBLE_*=stub)」 블록은 `docs/e2e-mocking-policy.md` 의 env 표로 걷을 여지가
>   있었지만, **9차 패스가 `sc01-02` 에서 같은 모양을 「유지」로 닫아 두었다.** 같은 명제를
>   패스마다 반대로 판정하면 그 자체가 drift 다. **뒤집으려면 `sc01-02` 를 포함한 증분 재판정으로
>   한 번에 해야 한다.**
> - **`docs/**.md` 를 더하는 패스는 허브 등재가 함께 간다.** 이 패스가 `passes/` 에 판정 상세를
>   하나 더하면서 `docs/index.html` 에 링크를 넣고 `Documents` 집계를 **37 → 38** 로 올렸다.
>   빠뜨리면 `check-journey-mockup.py` 의 R8·R9 가 실패한다. 허브는 주석 지문 범위 밖이라 판정
>   수치에는 영향이 없다.
> - **범위 밖에서 적발한 것 하나 — `backend/src/llm.rs:494-499` 의 주석이 거짓이고, 그 거짓이
>   실제 flake 를 덮고 있다.** 「no other test reads this variable, so parallel test runs cannot
>   race on it」이라 적혀 있는데 같은 파일 449행의 `stub_answer` 가 `FEATUREDOC_STUB_LLM_FAIL` 을
>   읽고 `stub_is_deterministic_for_the_same_ask` 가 그 경로를 탄다. 실측 **15회 중 1회** 그
>   테스트가 실패한다(부모 `f5a2937` 에서도 재현 — 이 패스와 무관하다). `llm.rs` 는 원장 1행의
>   범위라 **증분 재판정**이고, 주석만 고쳐서는 flake 가 남으므로 **테스트 수정이 따라붙는 별개
>   작업**이다. 여기서는 등재만 한다.
> - 지문 계산 규약은 앞 블록과 같다: 원장의 **범위 지문**은 후행 개행을 **포함**하고
>   (`echo "$HITS" | sha256sum`), 모델의 **전역 지문**은 versionScript 그대로 후행 개행을
>   **제외**한다(`printf '%s'`). 같은 입력에도 두 값은 다르다.
