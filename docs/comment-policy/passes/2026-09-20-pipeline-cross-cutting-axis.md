# 판정 상세 — 파이프라인 · 횡단 관심사 축 비경합 9파일 (2026-09-20)

판정 범위: `backend/src/pipeline.rs` · `backend/src/cross_cutting.rs` ·
`backend/tests/documents.rs` · `backend/tests/progress.rs` ·
`frontend/src/CrossCuttingConcerns.tsx` ·
`e2e/tests/sc01-02-repo-out-of-scope.spec.ts` ·
`e2e/tests/sc01-03-cross-cutting-determinism.spec.ts` ·
`e2e/tests/sc01-05-resume-after-app-exit.spec.ts` ·
`e2e/tests/sc02-03-contradiction-separation.spec.ts`

판정 시점 트리: `48a4a87`(main tip). 범위의 주석 **337행 → 139행**, 순 제거 **198행**.
여기에 원장 3행의 **증분 재판정 ①**(순 제거 3행)을 더해 이 패스의 총 순 제거는 **201행**이다.

규칙은 [README.md](../README.md), 결과 표면은 [ledger.md](../ledger.md)에 있다.

## 이 범위를 고른 이유

**`#79`(목업↔구현 수렴 슬라이스 ⑥)가 머지되면서 이 축이 통째로 열렸다.** 직전 패스(후보·전략 축)가
「`#79`가 머지되면 `api.ts`·`App.tsx`를 함께 집는 것이 다음 패스의 첫 후보」라고 적어 뒀지만,
`#79`는 그 둘 말고도 `sc01-02`를 자유 풀로 돌려보냈고 동시에 **원장 3행에 미판정 증분을 열었다**.
후자가 우선순위를 바꿨다 — 원장 안의 미판정 증분은 5·6·7·8차 패스가 매번 그 패스 안에서 닫아
왔고, 열어 둔 채 다음으로 넘기면 원장이 「현재 상태의 표면」이기를 그만둔다.

그래서 이번 범위는 **증분이 열린 그 파이프라인 축**으로 잡았다. 한 사용자 동선의 전 층을 한 범위에
넣어 같은 문장이 층마다 몇 벌로 복제됐는지 한 번에 보는 기준은 앞 패스들과 같다:

- **단계 정의** `pipeline.rs` — 파이프라인이 무엇으로 이루어졌는지의 한 자리
- **2단계 구현** `cross_cutting.rs` — 그 단계 중 첫 LLM 경유 단계
- **계약 테스트** `documents.rs`(문서 저장·재현성) · `progress.rs`(진행·부분 재시도)
- **화면** `CrossCuttingConcerns.tsx`
- **e2e** `sc01-02`(진입 거부) · `sc01-03`(결정적 재현) · `sc01-05`(앱 종료 후 복귀) ·
  `sc02-03`(모순 분리)

`sc02-03`은 인수 축 소속으로 보일 수 있으나 `sc01-03`·`sc01-05`와 **워커 임대 블록을 글자 그대로
공유**하는 파일이라 같이 집었다(아래 ②). 이 축을 한 범위로 묶은 덕에 **13벌까지 복제된 문단**이
드러났다.

### 경합 집합은 감지 브리프 이후 또 바뀌었다 — 이번엔 줄었다

detector가 넘긴 브리프는 경합을 **7파일 / 296행**으로 적었다. 그 값은 `072d8b6` 시점이고,
계획 시점(`48a4a87`)에 `/pulls` 를 다시 전수 실측하니 **4파일 / 217행**이다.

| 파일 | 행 | 잡은 PR |
|---|---|---|
| `tools/check-mockup-render.py` | 91 | `#26` |
| `frontend/src/index.css` | 44 | `#17` |
| `frontend/src/HomeRepositories.tsx` | 42 | `#17` |
| `frontend/src/CredentialsSetup.tsx` | 40 | `#17` |

`#79`가 머지돼 열린 PR에서 빠지면서 `App.tsx`(53) · `sc01-02`(35)가 자유 풀로 돌아왔고,
`ConnectRepository.tsx`는 `#79`가 `HomeRepositories.tsx`로 **병합하며 사라졌다**(in-scope 주석
파일이 111 → 110개가 된 이유가 이것이다). `#64`는 in-scope 주석 파일이 **신규 추가뿐**이라
여전히 경합이 아니다.

이 패스의 9파일은 `#64`·`#26`·`#17` 세 PR의 파일 목록과 **한 파일도 겹치지 않는다.**

## 제거한 것 — 복원 경로별

### ① AC 조항·시나리오 본문 재진술 (docs/prd · docs/test)

가장 큰 덩어리다. `docs/prd/01-analysis-pipeline.md` AC1.2와
`docs/test/01-analysis-pipeline.md` 시나리오 2·3·5가 주석 안에 거의 그대로 다시 적혀 있었다.

- `cross_cutting.rs` 머리 — AC1.2의 **다섯 축을 한국어로 열거**하고(`AXES` 상수가 바로 아래에서
  같은 것을 열거한다) 「모든 항목이 파일 경로/심볼 참조를 달아야 한다」는 검증 방법을 재진술했다.
  **두 겹으로 복원 가능하다**: 코드(①)와 PRD(②).
- `sc01-03` 머리 — 「docs/test/01 시나리오 3을 그대로 따라간다:」로 시작해 기대 결과 세 줄을
  **인용 블록으로 옮겨 적었다**. 파일 1행의 `// 검증 시나리오:` 선언이 이미 그 문서를 가리킨다.
- `sc01-02` 머리 — 「repo URL 입력 → 분석 시작 → 큐 등록」과 거부 절반의 기대 결과가 AC1.1의
  검증 방법 문장 그대로다.
- `documents.rs` 의 `a_reanalysis_reports_whether_the_result_reproduced` doc — 「AC1.2의 결정성
  조항, 양 절반」. 함수 이름이 이미 말하고, 조항 본문은 PRD에 있다.
- `cross_cutting.rs` 단위 테스트 둘의 doc — 「AC1.2's verification method: …」·「The determinism
  clause: …」. 함수 이름(`every_extracted_item_cites_a_path_from_the_tree` ·
  `the_same_tree_reproduces_the_same_document`)이 같은 말을 한다.
- AC 꼬리표만 남은 것들 — `pipeline.rs` 의 `(AC1.2)`·`(AC1.3)`·`(AC1.4)`·`(AC2.1~AC2.3)`,
  `progress.rs`·`documents.rs` 의 `(AC4.7)`·`(AC1.5)`.

### ② 층·파일을 가로질러 복제된 명제 — **최대 13벌**

- **워커 임대 문단.** 「Isolation: this spec *leases* the analysis worker (see
  `e2e/support/cluster.ts`). It scales the Deployment to 1 … `playwright.config.ts` pins
  `workers: 1` … it drains every queued job …」 — 6줄짜리 이 문단이 **`e2e/tests/` 의 13개 spec에
  글자 그대로 복제돼 있다.** 그런데 그 문단 자신이 가리키는 `e2e/support/cluster.ts:1-11` 에
  **더 정확하고 긴 정본**이 있다(원장 3행이 이미 유지로 판정해 보존한 바로 그 주석이다).
  이 패스의 3개 spec(`sc01-03`·`sc01-05`·`sc02-03`)에서 그 6줄을
  **`// Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.` 한 줄로 줄였다.**
  나머지 10개는 이 범위 밖이라 그대로 뒀다 — **후속 패스의 명시적 후보다**(아래 「범위 밖」).
- **`finally` 의 「The worker is leased, not owned — hand it back whatever happened above.」**
  — 바로 위 문단의 두 벌째이자 `scaleWorkers(0)` 자신의 재진술.
- **「기대값을 상수로 박지 않는다」 문단** — `sc01-03` 머리에 있고, 본문 두 곳
  (「근거의 유효성은 … 분석된 저장소를 기준으로 본다」·「화면은 API가 내려준 값과 대조한다」)에
  다시 풀어 썼다. **머리 한 벌만 남겼다.**
- **「stub 더블이 정확히 한 번 다툰다」** — `sc02-03` 의 사전 조건 블록 5줄이 `acceptance.rs`
  단위 테스트의 고정 성질을 서술하는데, 바로 아래 단정
  `expect(doc.contradictions.length, '이 저장소는 정확히 한 번 다툰다').toBe(1)` 이 같은 말을
  메시지까지 달아 한다. **한 줄로 줄였다.**

### ③ 목업 · 여정 · doc-tracker 인용

- `pipeline.rs` 머리의 `docs/mockups/JRN-discover-features.html#STP-leave-and-return` 인용과
  「("Pipeline · 3 of 5")」 — 목업 자신에서 복원된다. **이 인용은 M1의 입력이 아니다**:
  `check-mockup-render.py::discover_screens()` 는 `frontend/src/*.tsx` 만 훑으므로
  `.rs` 파일의 목업 URL은 어떤 게이트도 읽지 않는다(직접 `SRC_DIR` 를 확인했다).
- `CrossCuttingConcerns.tsx` 머리의 목업 편차 서술 — 「목업의 항목별 근거는 경로 하나다」 등.
  **매핑 줄 자체는 남기고**(아래 「유지」) 편차 서술만 한 문장으로 줄였다.
- `sc01-02` 본문의 「메트릭 그리드는 목업에 없어 슬라이스 ⑥ 이 제거했다 …」 ·
  「슬라이스 ⑥ 이 그 문면을 목업 카피로 수렴시켰다(대상 저장소 이름을 끼워 넣던 구현 문구 →
  목업의 고정 문장)」 — 목업 카피 인용 + 작업 흔적.
- `CrossCuttingConcerns.tsx` 의 `Appbar` doc 5줄 — 「목업의 `STP-review-landscape` 처럼 세
  슬롯 …」. **슬롯 수는 M7이 목업에서 직접 세지 주석에서 읽지 않는다.**

### ④ 작업 흔적 — PR 번호 · 슬라이스 번호 · task id · 정정 이력

- `sc01-03` 머리의 「(종전 헤더는 시나리오 2를 인용했으나 오기였다 — …)」 — **정정 이력**은
  커밋 메시지(④)의 자리다.
- `sc01-05` 머리의 「한 파일에 있던 두 시나리오를 분리한 것은 `rct_20260916-0002`가 닫았다」 —
  reconciler task id.
- `CrossCuttingConcerns.tsx` 의 「목업이 AC1.2의 다섯 축을 따라잡은 것은 2026-09-18 (#57)이고 이
  화면은 뒤이은 수렴 슬라이스에서 한국어 캡션을 받았다 — **그래서 지금은 양쪽이 같다**」 —
  PR 번호 + 날짜 + 「지금은 차이가 없다」. **차이가 없다는 말은 현재 상태에 대해 아무것도 더하지
  않는다.**
- `sc01-05` 의 「(The failing-stage half that used to share this file now lives in
  sc01-06-partial-retry.spec.ts.)」 · `sc01-02` 의 슬라이스 ⑥ 언급 2곳.

### ⑤ 낡아서 **거짓이 된** 주석 2건 — 제거 근거를 강화한다

모델 정의의 「되풀이된 주석이 낡아 틀려 있으면 그것은 제거 근거를 강화한다」에 해당하는 실례가
이 범위에서 둘 나왔다.

1. **`cross_cutting.rs` 머리** — 「Cross-cutting Concerns 목업은 다섯 축 중 **넷만 그리고**
   (저장소 구조를 뺀다) … 그 차이는 **알려진 목업↔구현 편차로 등재돼 있다**」.
   **두 문장 다 거짓이다.** 목업은 저장소 구조를 그린다
   (`docs/mockups/JRN-discover-features.html:1048` 에 `<span>저장소 구조</span>` 가 있다),
   그리고 `docs/doc-tracker/2026-08.md` 는 그 편차가 머지 직전 rebase 시점에 **이미 해소돼
   있었으므로 「이 항목은 원장에 등재하지 않는다」** 고 명시한다. 같은 레포의
   `CrossCuttingConcerns.tsx` 는 (제거 대상이던 ④ 문단에서) **반대로** 「목업이 따라잡았다」고
   적고 있었다 — 두 주석이 서로 모순이었고, 틀린 쪽이 이것이다.
2. **`sc01-05` 본문** — 「Three stages are implemented as of slice 4b-1 …; stages 4-5 stay
   pending, so the run lands at 3 of 5. This number … moves every time a slice implements one
   more stage」. 그런데 `pipeline.rs` 머리는 같은 트리에서 「**All five stages run today**」라고
   적는다. 어느 쪽이 맞든 **한쪽은 낡았고**, 단계 수라는 사실의 SSOT는 `STAGES` 상수와
   `docs/doc-tracker/` 변경 이력이지 spec 본문 주석이 아니다.

### ⑥ 선언 · 시그니처 재진술

- `pipeline.rs`: `Stage`(「One step of the pipeline as the user sees it」) ·
  `title`(「Label rendered by Analysis Progress」) · `stage()`(「Looks a stage up by its wire
  key」) · `QUEUED`(「Enqueued, waiting for a worker to claim it」) ·
  `is_reportable()`(「Whether a worker-reported stage status is one we accept」 — 본문이
  `matches!(s, RUNNING | SUCCEEDED | FAILED)` 한 줄이다) · `CROSS_CUTTING`·`DISCOVERY_STRATEGY`
  의 doc 전체.
- `cross_cutting.rs`: `input_paths`(「Sorted, truncated view of the tree」 — 본문이
  `sort` · `dedup` · `truncate` 세 줄이다) · `extract`(「Runs stage 2 and returns the document to
  persist」) · `stub_answer` 의 요약 1줄.
- `documents.rs`: `submit`(「Submits a document the way the worker does, and asserts it was
  accepted」) · `a_worker_without_the_lease_cannot_store_a_document` ·
  `another_users_document_is_not_readable` 의 doc — **함수 이름이 문장 그대로다**.
- `progress.rs`: `stage()` · `run_until_second_stage_fails` ·
  `retry_resets_only_the_failed_stage_and_requeues_the_job` 의 doc.
- `CrossCuttingConcerns.tsx`: `reproText` · `onBack` · `onOpenDiscoveryStrategy` 의 JSDoc.
- `progress.rs` 의 단정 나열 주석 4줄(「The retried stage is clean again…」 →
  「…the stage that already succeeded is untouched…」 → 「…the stages that never ran are still
  waiting…」 → 「…and the job is queued again」)과 「The refusals changed nothing.」 ·
  「The owner's job is untouched by the refused retry.」 — 전부 **바로 아래 단정의 산문 번역**이다.

### ⑦ 구분선 · 절 제목

`sc01-03` 9건 · `sc01-05` 6건 · `sc01-02` 5건 · `sc02-03` 3건 · `progress.rs` 2건.
(`// ── setup: this spec's own user, App installation, LLM key ──`,
`// ── read: Analysis Progress's progress ──`, `// ── partial retry ──` 류.)

## 유지한 것 — 복원 불가능한 지식

- **기계가 읽는 주석**(정책 대상 밖): `sc01-02:1`·`sc01-03:1`·`sc01-05:1`·`sc02-03:1` 의
  `// 검증 시나리오:` 4건, `cross_cutting.rs` 의 `mock-exception: LLM-01` 2건.
- **`CrossCuttingConcerns.tsx` 머리의 목업 매핑 줄** —
  `docs/mockups/JRN-discover-features.html#STP-review-landscape` 는
  `check-mockup-render.py::discover_screens()` 가 앞 2,000자에서 읽는 **M1의 입력**이다.
  꼬리표 `(Cross-cutting Concerns, AC1.2)` 와 앞 줄의 산문만 떼고 **URL은 그대로** 뒀다.
  판정 뒤 `MAPPING_REF` 를 직접 돌려 이 화면이 발견 집합에 남아 있음을 확인했고, M7이
  `[frontend/src/CrossCuttingConcerns.tsx ↔ STP-review-landscape] 슬롯 3` 으로 통과한다.
- **`acceptance_dependencies` 의 이름–동작 불일치** — 「wire key는 로드맵이 한 슬라이스에 묶었던
  두 절반을 아직 이름에 달고 있다. 키는 wire 계약(`/internal/.../stages/{key}`,
  `analysis_documents.kind`)이라 그대로 두고, 무엇이 실제로 도는지는 title이 말한다.」
  이 주석이 없으면 **키를 고쳐야 할 오타로 읽힌다.**
- **게이트가 큐의 성질이라는 설계** — `FEATURE_CANDIDATES`·`ACCEPTANCE_DEPENDENCIES` 의
  「승인 전까지 큐가 내주지 않는다 — 각 워커가 기억하는 규칙이 아니다」. 다만 **같은 문장이 두
  상수에 두 벌**이어서 두 번째의 「Same mechanism as AC1.3's gate」는 지웠다.
- **`AWAITING_PIPELINE` 의 명명 근거** — 「일부러 `succeeded`가 아니다 — 분석은 완결되지 않았고,
  그렇게 말하면 실제로 돈 것을 과장한다.」
- **상류 API 거부 조건** — `cross_cutting.rs` 의 「스키마는 프로바이더에 그대로 전달되므로 stub
  답이 그 안에 있으면 안 된다 — OpenAI는 `strict` 아래에서 모르는 키워드를 거부한다.」
- **프롬프트 결정성 함정** — `MAX_PATHS` 의 「샘플이 아니라 정렬된 목록의 *앞* N개를 취하는 것이
  결정성을 준다.」
- **블롭을 받지 않는다는 경계** — `cross_cutting.rs` 머리. 경로만으로 근거가 충분하다는 판단은
  코드에서 읽히지 않는다.
- **하네스 충실도 불변식** — `progress.rs`·`documents.rs` 머리의 「단계 전이는 워커 자신의
  `/internal` 경로로 몰아간다 — 손으로 쓴 `analysis_stages` 행은 워커가 실제로 만드는 것과
  어긋날 수 있다.」
- **동시성 · 임대 계약** — `progress.rs::claim` 의 「임대를 쥐는 유일한 방법」,
  `retry_is_refused_while_a_worker_still_holds_the_job` 의 「살아 있는 임대 아래에서 재큐잉하면
  보유자가 방금 리셋된 런에 계속 보고하게 된다」, 「Deliberately no `finish`: 잡은 아직 만료되지
  않은 임대 아래 `running` 이다」.
- **보안 불변식** — 「`404`이지 `403`이 아니다 — API는 내주지 않을 id의 실재를 확인해 주지
  않는다.」
- **「없음」과 「빈 것」의 구분** — `sc01-03` 의 「404, not an empty document: 『아직 실행되지
  않음』과 『실행했고 아무것도 못 찾음』은 사용자에게 다른 상태다」와 `documents.rs` 의 대응 주석.
  **이 둘은 같은 명제의 두 벌이지만 층이 다르고**(e2e ↔ 계약 테스트) 각각이 자기 층에서 단정을
  설명하므로 둘 다 남겼다.
- **테스트 격리 함정** — `sc01-03`·`sc01-05` 의 「오버레이가 이미 0이지만 전제를 명시해 큐에
  있던 잡이 『아직 아무것도 안 돌았다』 단정 전에 드레인되지 않게 한다.」
- **더블의 고정 경계** — `sc01-05` 의 「스텁 저장소는 2300 KiB ⇒ 766 files · 2.2 MB
  (`repo_scan::stub_scan`); 이 수는 워커의 측정값이지 이 파일의 픽스처가 아니다」와
  「Cost is still the pre-flight estimate — measured spend is AC4.6」.
- **화면이 비어 있는 이유** — `CrossCuttingConcerns.tsx` 의 「목업이 대기 카피를 그리지 않으므로
  대기는 이 화면이 지어낸 문장이 아니라 빈자리다」와 「문서가 통째로 뺀 축도 제목은 받는다 —
  조용히 빠진 제목은 『이 축은 요구되지 않았다』로 읽힌다」.
- **읽기가 갈린 이유** — 「별도의 읽기라서 그 실패가 이 화면이 존재하는 이유인 페이지를 막지
  않는다.」
- **`sc01-02` 의 스텁 배선** — 「e2e 배포(`FEATUREDOC_DOUBLE_GITHUB_APP=stub`)의 App 설치가 닿는
  저장소가 정확히 셋이다」 — `toHaveCount(3)` 의 3이 어디서 오는지는 코드에 없다.
- **사용자 단위 격리** — 「App 설치·키 상태는 *사용자* 단위이므로 spec마다 자기 스텁 사용자로
  로그인한다 — 정체성을 공유하면 한 spec이 다른 spec 밑에서 App을 설치해 버린다.」
- **두 번 누르는 `continue`** — 「Continue confirms readiness, then carries the user into Home.」
  같은 요소를 연속으로 두 번 클릭하는 코드는 이 한 줄이 없으면 버그로 읽힌다.
- **`sc01-02` 의 범위 선언** — 「앞부분의 홈 목록 → pre-flight → `Queued` 는 셋업이지 이 파일의
  선언 대상이 아니다 — 그 절반은 시나리오 1 전용 spec이 신설될 때 이어받는다.」
  등재처인 doc-tracker 「e2e 매핑」의 **미매핑 잔여 표는 그 spec이 생기면 사라지는 임시 구조**라
  복원 경로로 세지 않았다.
- **`sc02-03` 의 범위 밖 선언** — 「원문의 기대 결과 중 『사용자는 어느 쪽을 정설로 채택할지
  결정할 수 있다』는 구현이 없어 e2e가 관측할 대상 자체가 없다.」 같은 이유로 유지.
- **`AXES` 의 PRD 순서 의존** — 상수의 **삽입 순서**가 곧 렌더 순서라는 사실은 배열만 보면
  읽히지 않는다. `cross_cutting.rs` 와 `CrossCuttingConcerns.tsx` 양쪽에 한 줄씩 남겼다
  (두 벌이지만 각각 자기 파일의 **순서 의존**을 지킨다).

## 판단이 갈려 남긴 것 (5건)

정책의 「애매하면 남긴다」를 적용했다.

1. **`sc01-03` 의 `/** Must match `backend/src/cross_cutting.rs` AXES. */`** — 원래 문장
   (「The five axes AC1.2 enumerates (backend/src/cross_cutting.rs AXES)」)에서 AC 재진술만
   떼고 **교차 참조는 남겼다.** 두 곳에 복제된 상수는 어느 쪽이 정본인지가 코드에 없다.
2. **`documents.rs::claim_offers_the_cross_cutting_stage` 의 doc** — 「큐는 워커에게 2단계에
   필요한 키를 내주고 그 단계를 실행 가능으로 지목해야 한다 — 아니면 워커는 `fetch` 뒤에
   멈춘다」. 앞 절은 단정이 보이지만 **「아니면 멈춘다」는 실패 모드**는 어느 단정에도 없다.
3. **`progress.rs` 의 「The home list carries the same fraction, from the same rows.」** —
   원래 문장이 인용하던 `"1 of 5"` 는 지웠다(화면 카피이고 단계 수에 따라 움직인다). 남은
   절반은 **왜 이 테스트가 홈 목록까지 확인하는가**를 말하므로 유지.
4. **`cross_cutting.rs::detail` 의 「Shaped to match stage 1's own detail line」** — 원문의
   `"766 files · 2.2 MB"` 인용은 지웠지만, **두 단계의 detail 형식이 일부러 같다**는 계약은
   `format!` 한 줄에서 읽히지 않는다.
5. **`CrossCuttingConcerns.tsx` 의 「목업 대조 상대가 없는 두 자리」** — 편차 서술을 한 문장으로
   줄이며 **항목별 근거 다중 경로**와 **재현성 줄**이 목업에 없다는 사실만 남겼다. 이 둘은
   M3B 카피 대조가 **구조적으로 볼 수 없는 자리**라, 지우면 다음 수렴 패스가 같은 질문을
   처음부터 다시 한다.

## 원장 3행 증분 재판정 ① — `#79` 가 연 3행

`#79`가 `sc01-01`에 남긴 순증 3행(수정 1줄 → 2줄, 추가 2줄)을 판정했다.

- **제거**: 「저장소 연결 폼은 홈과 한 화면이다(슬라이스 ⑥ 병합) — … 별도의 연결 화면으로
  건너뛰는 단계가 없어졌다.」 2줄. **슬라이스 번호는 작업 흔적**(③·④)이고, 「별도 화면으로
  건너뛰지 않는다」는 **그 diff 자신**에서 복원된다. 폼이 홈과 한 화면이라는 사실은 바로 아래
  코드가 네비게이션 없이 `repo-url` 을 채우는 것으로 보인다.
- **압축**: 「시작 전에는 어떤 저장소도 실행 이력이 없다(메트릭 그리드는 목업에 없어
  제거됐으므로 같은 사실을 카드의 실행 상태로 단정한다 — 상태 배지가 하나도 없다).」 2줄 →
  **「시작 전에는 어떤 저장소도 실행 이력이 없다.」 1줄.** 괄호 안은 목업 인용 + 제거 이력이고,
  앞 절은 `badge).toHaveCount(0)` 의 **의미**라 단정만으로 복원되지 않는다.

순 제거 **3행**. 3행의 범위는 **145행 → 142행**으로 돌아왔지만 **지문은 `fb27e0db…` 가 아니라
`0e5c3d31…`** 이다 — 살아남은 줄의 문면이 `#79` 이전과 다르기 때문이다. 줄 수만 보고 「원상
복구」로 읽지 말 것.

## 이 패스가 병합되면

- **범위 9파일 지문**: **139행** /
  `b313af22be5ab70962138ca67e50503c5a39a21699c55a0642c58457005b8f01`
- **원장 3행(갱신)**: **142행** /
  `0e5c3d31474b356f84960e1be87169d5c72d693b41ac26263ddec5deebec5e4f`
- **전역**(모델 versionScript, 후행 개행 제외): 부모 `48a4a87` 의 `lines=3086 files=110` /
  `701de5be…` 에서 **순 제거 201행** → `lines=2885 files=110` /
  `d9be3f73e9d286504b208e95cb6a01294f3ebbc3f8eb400baf56e7b20aea4bac`.

전역 절대값은 열린 PR(`#64`·`#26`·`#17`)이 먼저 머지되면 그만큼 움직인다 — 그때도
**순 제거 201행**은 그대로다. **완료 기준은 절대 지문이 아니라 이 순 제거다**(6·7·8차 모두
절대값은 자매 머지로 지났고 순 제거만 맞았다).

## 검증 (판정 시점 로컬 실측)

- **코드 무변경을 「비주석 diff 0줄」보다 강하게 증명했다.** 바뀐 10파일 각각에 대해
  **주석을 전부 제거한 소스**(줄 주석 · 블록 주석 · JSX 주석 · 빈 줄)를 `HEAD` 판과 대조해
  **10/10 바이트 동일**임을 확인했다. 단정·문자열·제어 흐름은 한 글자도 바뀌지 않았다.
- **이 검사가 실제로 결함을 하나 잡았다.** 머리 주석을 재작성하는 과정에서
  `backend/tests/progress.rs`·`backend/tests/documents.rs` 의 **`mod common;` 선언이 함께
  지워졌다**(둘 다 `cargo test` 컴파일이 깨졌을 것이다). 눈대중 diff 와 「주석 줄만 바뀌었나」
  검사는 둘 다 이것을 **삭제된 비주석 줄**로만 보여 주는데, 위 대조는 복원 여부까지 판정한다.
  복원 후 재실행해 10/10 동일을 확인했다.
- **문서 게이트 3종 rc=0** — `check-mockup-render.py`(M1~M7) · `check-scenario-e2e.py`(S1~S3) ·
  `check-journey-mockup.py`(R0~R11).
- **기계 판독 주석 보존**(전역 계수) — `// 검증 시나리오:` **27건** · `mock-exception:` **18건**
  (둘 다 부모와 동일) · `frontend/src/*.tsx` 목업 매핑 **10건**. 매핑이 부모의 11건이 아니라
  10건인 것은 **이 패스와 무관하다** — `#79` 가 `ConnectRepository.tsx` 를 삭제했다.
- **허브 등재** — `docs/**.md` 가 하나 늘었으므로 R9가 요구하는 대로 `docs/index.html` 에 이
  파일 링크를 더하고 `Documents` 집계를 **34 → 35** 로 올렸다. (이 절차를 빠뜨리면 R8·R9가
  3건 실패한다 — 실제로 한 번 관측하고 고쳤다.)
- `cargo test` 와 `check-journey-prototype.js`(node)는 호스트에 cargo·node 가 없어 로컬에서
  돌릴 수 없다 — **CI 가 유일한 집행자**다. 위 「주석만 바뀌었다」 대조가 그 공백을 메우는
  근거다.

## 범위 밖 (후속)

- **워커 임대 문단이 남은 10개 spec** — `sc01-01`·`sc01-04`·`sc01-06`·`sc01-07`·`sc02-01`·
  `sc02-02`·`sc02-04`·`sc02-05`·`sc02-06`·`sc02-07`. 이 패스가 3개에서만 줄였다.
  그중 `sc01-01`·`sc01-06`(원장 3행) 등 **이미 판정이 끝난 행에 속한 것들은 증분 재판정이
  아니라 「판정 기준의 변경」**이므로, 한 패스에서 10개를 한꺼번에 다루기보다 각 행을 집는
  패스에서 같은 기준을 적용하는 편이 원장과 어긋나지 않는다. **다음 패스가 가장 먼저 저울질할
  항목으로 여기 남긴다.**
- `backend/migrations/*.sql` **8파일 / 139행** — 본문 「적용된 마이그레이션」 절의 전용 PR ·
  수동 repair · 사람 승인 게이트를 거치는 별도 패스. 이번 창에서 무이동이다.
- **열린 PR 접촉(경합) 4파일 / 217행** — `tools/check-mockup-render.py` 91(`#26`) ·
  `frontend/src/index.css` 44(`#17`) · `frontend/src/HomeRepositories.tsx` 42(`#17`) ·
  `frontend/src/CredentialsSetup.tsx` 40(`#17`).
- **자유 풀 나머지 43파일 / 806행**. 큰 후보: `frontend/src/api.ts` 96 ·
  `tools/check-journey-mockup.py` 62 · `frontend/src/App.tsx` 53 ·
  `tools/check-scenario-e2e.py` 44 · `backend/tests/worker.rs` 35 ·
  `backend/src/github_app.rs` 35 · `backend/src/cross_cutting.rs` 는 이제 판정 완료.
  **`api.ts` + `App.tsx` 는 이제 둘 다 비경합**이라 직전 패스가 예고한 「프런트 데이터 계층을 한
  범위로」가 드디어 가능하다.
- **`tools/check-journey-mockup.py` 의 `#81` 증분 7행** — detector 가 「미리 판정하지 않았다」고
  넘긴 항목이다. 그 파일은 자유 풀에 있고 이 패스의 범위 밖이라 **판정하지 않았다.** 구분선
  `# ── R11 · 허브 단계 주석 ───` 은 형태상 유형 ⑦이고 나머지는 R11 설계 근거 서술이라
  `#81` 본문(복원 경로 ③)과의 대조가 필요하다 — `tools/` 체커 축을 집는 패스의 몫이다.
- **모델 제외 패턴 확장 판단**(`docs/mockups/.*\.html#STP-` 를 지문에서 빼는 안)은 여전히
  **control plane** 의 몫이다. 이 패스도 data plane 이라 모델 정의를 건드리지 않았다.
  다만 이 패스가 **그 패턴이 `.rs` 파일에서는 게이트 입력이 아니라는 사실**을 실측으로 좁혔다
  (`discover_screens()` 는 `frontend/src/*.tsx` 만 훑는다) — 확장할 때 범위를 tsx 로 한정할
  근거가 된다.

## 증분 재판정 ③ — 원장 9행에 #134 · #132 가 연 +6행 (2026-09-24 · `rct_20260922-0009`)

reconciler task `rct_20260922-0009`. **순 제거 5행 · 유지 1행**(3행 → 1행 재작성 포함).

판정 전 재현으로 먼저 고정한 것: 행 기재값 142 / `5a4aea6e…` 는 **트리거 커밋 `27d9b81` 에서
바이트 일치**하고, HEAD `074c325` 에서 148 / `26eaec9f…` 로 갈린다. 증분 `+6` 이 전역 지문
`2646 → 2654` 의 `+8` 중 6을 설명하고 나머지 2는 원장 10행(`index.css`)이라 **잔차 0** 이다
(행 지문은 「행 지문을 재현하는 법」대로 개행 포함 해시, 전역은 미포함 — 두 규약을 모두 돌려
행 규약 쪽이 맞는 것을 확인했다).

### 제거 — `backend/src/cross_cutting.rs` `AXIS_GUIDE` doc 2행 (#134)

- 「What each axis asks for, as the model reads it」 — 상수 이름 `AXIS_GUIDE` 와 바로 아래 리터럴이
  그대로 말하는 **선언 재진술**(①).
- 「the labels in [`AXES`] are screen copy」 — `AXES` 자신의 doc 「In PRD order — the screen renders
  the axes in this order」(①)와 PR #134 본문 「한글 라벨(`AXES`)은 화면 카피라 건드리지 않았다」(③)의
  **두 벌째**. rustdoc 링크만의 교차 참조를 위해 문장을 남기지 않는다는 9행의 기존 잣대도 같은 방향이다.
- 「Examples name kinds of evidence, never concrete paths」 — **열 줄 아래 `prompt()` 리터럴이 모델에게
  그대로 말하는 문장**(`The examples describe kinds of evidence, not files in this repository: cite only
  paths from the list above.`)의 두 벌째다. 증분 재판정 ⑤·⑨·⑪ 의 「복제된 명제는 **강제하는 코드 옆**
  한 벌만」을 적용하면, 실제로 모델에 전달되어 규칙을 *강제하는* 쪽은 프롬프트 리터럴이므로 그쪽을
  정본으로 둔다(①).

### 제거 — `frontend/src/CrossCuttingConcerns.tsx` 3행 (#132)

- 근거 줄 `' · '` join 실패 모드 2행(「joined with `' · '` they wrapped mid-path and the separators
  landed at the start of the next line」) — PR #132 본문 증상절이 「경로를 ` · ` 로 이어 붙여 경로
  중간에서 꺾이고 구분점이 다음 줄 앞에 옴」으로 **축자에 가깝게** 적는다(③). 같은 명제의 정본은
  `index.css` 쪽 `.ev` 블록이었고 그 블록은 원장 10행에서 함께 판정했다.
- legend 배치 1행(「the legend … sits only under an axis that actually shows one」) — **바로 아래
  `section.items.some((item) => item.evidence.length === 0) && …` 가드가 그 문장 자체**이고(①),
  「as in the mockup」은 목업(②)과 PR #132 수정절 「legend 는 목업처럼 `근거 없음` 항목이 있는 축에만」(③)이
  복원한다. 세 경로가 겹치는 이 패스에서 가장 명백한 제거다.

### 유지 1행 — 리터럴을 묶는 함정

`/// A concrete path here is one the model can cite even when the tree does not contain it.`

프롬프트 리터럴은 「예시는 종류다」라는 **규칙만** 말하고 *왜* 그래야 하는지는 어디에도 없다.
「프롬프트에 박힌 구체 경로는 트리에 없어도 모델이 그대로 인용할 수 있다」는 LLM 경계의 실패 모드이고,
`AXIS_GUIDE` 리터럴을 고치는 사람이 바로 그 자리에서 읽어야 하는 제약이다. PR #134 「설계 메모」가
같은 말을 적지만(③), 이 명제를 **강제하는 테스트가 없다** — `the_prompt_describes_every_axis_in_screen_order`
는 축 키의 1:1 과 설명 줄의 탑재만 단정하고 「구체 경로가 없음」은 보지 않는다. 정책 본문의 「애매하면
남긴다」와 「실패 모드의 함정」에 걸리므로 3행을 1행으로 줄여 남겼다.

### 값

142(기재 · 트리거 바이트 일치) → 148(유입 후) → **143 /
`515d540af0130cb52fee733de80ed48f2dfd6aaa9831cbf3c28bcd7e541ddf2b`** (base `074c325` 측정).

**자매 착지 재실측(2026-09-24 · #137)** — 머지 직전에 #137(`fd6cdad`)이 착지해 `cross_cutting.rs` 에
순 +30행을 열었다(148 → 178). 판정을 다시 하지 않고 머지 시점 트리에서 **줄 수·지문만 재고정**한다:
**173 / `f184887421a70d8275f2369a454708f8f471576e8188132919cc4b9bcb9a2d4e`**. 이 패스의 순 제거 −5행은 불변이고(178 → 173),
원장 10행은 90 → 89 · 전역은 2700 → 2694 라 **행 합 −6 == 전역 −6, 잔차 0** 이 그대로 성립한다.
#137 이 들인 30행은 **판정하지 않고 다음 감지에 넘긴다**.

**자매 착지 재실측(2026-09-24 · #138)** — 위 재고정 직후 #138(`95d3395`)이 연이어 착지해 같은 파일에
순 +26행을 더 열었다(178 → 204). 같은 절차로 한 번 더 재고정한다: **199 /
`db92b2402cc84340e14148375e64e4080cdb16b1ad27b631ab7997c0cc2d669c`**. 순 제거 −5행은 불변이고(204 → 199), 원장 10행 90 → 89 ·
전역 2726 → 2720 으로 **행 합 −6 == 전역 −6, 잔차 0** 이 그대로 성립한다.
#138 이 들인 26행도 **판정하지 않고 다음 감지에 넘긴다**.

### 판정하지 않은 것

- #121 이 연 `backend/tests/progress.rs` 2행 — 9행의 「자매 착지 재실측」에 **이관으로 등재된** 항목이라
  이번 범위가 아니다.
- 머지 직전에 연달아 **착지한** PR **#137**(`fd6cdad`, +278/-10 → 주석 30행)과 **#138**(`95d3395`,
  +223/-29 → 주석 26행)이 `cross_cutting.rs` 에 들인 합 56행. 둘 다 훅은 모듈 머리·상수 doc·구분선·
  경로 샘플링 서술이라 **이 패스의 줄(20~22)과 겹치지 않았고**, 머지 시점 트리에서도 이 패스가 걷어낸
  명제를 되살리지 않는다(순 제거 −5행 불변 · 잔차 0). 줄 수·지문은 위 「자매 착지 재실측」 두 절에서
  재고정했고, 그 56행의 **판정은 다음 감지의 몫**이다.
