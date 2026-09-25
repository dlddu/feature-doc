# 주석 판정 원장

정책 본문은 [README.md](README.md), 각 행의 근거 상세는 [passes/](passes/)에 있다. 이 파일은
판정 결과의 표면이다 — 규칙을 여기에 다시 쓰지 않고, 근거 상세를 여기에 펴지 않는다.

## 원장 읽는 법

- 한 행 = 판정한 범위(파일 단위). **지문**은 그 범위의 주석 줄을 정규화·정렬해 해싱한 값으로,
  주석이 추가·수정·삭제될 때만 변한다(본문 「지문과 사각지대」).
- **행 지문을 재현하는 법** — 행의 파일 목록에 전역 지문과 **같은** 주석 패턴·DIRECTIVE 제외·공백
  정규화·정렬을 적용한 뒤 `printf '%s\n' "$HITS" | sha256sum`, 즉 **개행을 포함해** 해싱한다.
  전역 지문(모델 `asIs.versionScript`)은 `printf '%s'` 로 **개행 없이** 해싱하므로 두 규약이 다르다 —
  같은 줄 집합이어도 값이 갈린다. 재현이 안 맞으면 먼저 이 개행부터 의심한다.
- **범위 칸의 괄호 문구는 파일 목록의 일부다.** 「… 는 D2 사람 게이트 풀로 보류」처럼 특정 파일을
  **빼는** 단서가 붙은 행이 있으므로, 백틱 경로를 기계적으로 전부 긁으면 안 된다(13행이 그 예 —
  `backend/src/crypto.rs` 를 넣으면 42행이 55행이 되어 값이 어긋난다). 줄 수 칸이 그 자리의 검산이다.
- 지문이 어긋났는데 **줄 수는 맞다면** 제자리 수정(주석 문면만 바뀜)이다. 이 경우는 행 열의 합에도
  「전역 − 잔여」에도 잡히지 않으니 **산술이 전부 초록이어도 어긋날 수 있다** — 재고정은 지문을 직접
  재현해서만 확인된다.
- 같은 범위의 증분 재판정은 새 행을 만들지 않고 원래 행의 결과 칸을 갱신하며, 패스 상세는
  원래 패스 파일에 절을 더한다.
- 결과 칸의 요약 뒤에는 해당 패스 파일을 링크한다.

| 판정 범위 | 현재 주석 줄 수 | 지문 | 결과 요약 |
|---|---|---|---|
| `backend/src/analysis.rs` · `backend/src/llm.rs` · `backend/src/worker_api.rs` · `backend/src/llmkey.rs` (backend 집중 4파일) | 613 | `666abdf8f6db8eaa0b9e8ef84adb9000ebbc0d25278617552ad6e4051ac803b6` | 순 제거 58행(구분선 16 · 빈 주석 행 3 · 문서·선언 재진술·작업 흔적 39 — 총 62행 제거 중 불변식 보존 2행 재작성) · 유지 507행 · 판단 갈림 3건 — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) · **증분 재판정 ①**(2026-09-18): #55가 더한 `llm.rs` 21행은 stub↔real 충실도 경계라 **전건 유지**, 제거 후보 3행(테스트 case 라벨)은 `llm.rs`가 #49와 경합이라 보류 — **해소됨**(2026-09-25 · 31차 패스 · `rct_20260925-0008`): #49 가 `2026-09-18T15:40:05Z` 에 머지돼 경합이 소멸했으므로 그 3행을 **집행 제거**했다(판정은 18차 패스의 「중복 유형 ④」를 그대로 쓴다 — 재판정 아님) · 616/`df4a5627…` → **613/`666abdf8…`** — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) 「보류분 집행」 · **증분 재판정 ②**(2026-09-18): #43이 `analysis.rs`·`worker_api.rs`에 더한 40행을 판정해 **순 제거 10행**(AC 조항 재진술 · 호출자·구조체 본문 재진술 · rustdoc 링크만의 교차 참조) · 유지 30행(리스 계약 · 게이트는 큐의 성질 · `acceptance_pending`의 술어 함정 · `work_remains`의 경합) · **증분 재판정 ③**(2026-09-19): #71이 `worker_api.rs`에 더한 37행을 판정해 **순 제거 18행**(라우트 선택 근거의 세 벌째 · AC 꼬리표 · 함수 이름 재진술 · SQL이 이미 말하는 「승인된 후보만」 · rustdoc 링크만의 교차 참조) · 유지 19행(요청 행이 곧 게이트 · 실패는 재시도되지 않음 · 같은 트랜잭션에서 행 교체) — [passes/2026-09-19-dependencies-axis.md](passes/2026-09-19-dependencies-axis.md) · **증분 재판정 ④**(2026-09-19): #71이 `analysis.rs`에 더한 36행과 #75가 더한 24행, 합 **60행**을 판정해 **순 제거 29행**(절 제목 · AC 조항 재진술 6 · 여정·목업 인용 · 선언 재진술 · 라우트 테이블과 핸들러 doc 의 두 벌 중 한 벌) · 유지 31행(`(created_at, rowid)` 정렬 이유 · 「묻지 않은 것 ≠ 없던 것」과 0008 의 두 테이블 분리 · 재큐잉 없는 요청은 아무도 돌리지 않는다는 한 트랜잭션 계약 · 실패 후 재요청이 행을 `queued` 로 되돌린다) — [passes/2026-09-19-reanalysis-diff-axis.md](passes/2026-09-19-reanalysis-diff-axis.md) · **증분 재판정 ⑤**(2026-09-21): #93이 `llm.rs`에 더한 `assert_strict_schema` doc 5행을 **명제 단위로** 판정해 **순 제거 2행**(함수 본문의 두 `assert` 재진술 · 단정 메시지가 이미 말하는 「optional 대신 nullable」과 스키마 리터럴 `["string","null"]` 재진술 — 5행 → 3행 재작성) · 유지 3행(`pub(crate)` 요약 1줄 · 「`required` 누락은 모델 실행 전 400」의 상류 거부 조건 · stub 은 스키마를 어디에도 보내지 않는다는 충실도 경계). **이 자리가 「required-but-nullable」 명제의 정본이다** — 강제하는 코드 `assert_strict_schema` 옆(4·6·8행의 같은 명제 사본 4행은 그래서 걷었다) — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) · **미판정 증분 없음** · **증분 재판정 ⑥**(2026-09-21): #92가 `analysis.rs` `document()` 에 더한 4행(승인 편집의 겹쳐 읽기와 `content_hash` 불변의 이유 — 0009 머리·doc-tracker·PR #92 의 **네 벌째**)을 **전건 제거** · 줄 수·지문은 #92 이전 값으로 바이트 동일 복귀 — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) 「증분 재판정 ⑥」 · **증분 재판정 ⑦**(2026-09-21): #107 이 `analysis.rs` 에 더한 2행(`document()` 의 추가 겹침 호출 위 1행 — `feature_add::overlay` doc 의 순서 계약이 정본 · `approved_candidate_name` doc 에 덧붙인 1행 — 함수 본문의 두 질의와 `confirmed_name` 요약)을 **전건 제거** · 줄 수·지문은 #107 이전 값으로 바이트 동일 복귀 — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) 「증분 재판정 ⑦」 · **증분 재판정 ⑧**(2026-09-21): #112 가 `analysis.rs` 에 더한 5행(`document()` 의 삭제 겹침 호출 위 1행 — 순서 계약을 `feature_delete::overlay` doc 으로 옮겨 정본화 · `CandidateView.previously_deleted` 의 AC3.3 PRD 축자 인용 4행 — `previous_deletion` pub doc·테스트 fn 이름·doc-tracker 6c 행의 사본)을 **전건 제거** · 줄 수·지문은 #112 이전 값 590/`9757b6b5…` 로 바이트 동일 복귀 — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) 「증분 재판정 ⑧」 · **증분 재판정 ⑨**(2026-09-22): 사람 PR #108(AC4.9 출력 언어)이 `analysis.rs` 5 · `llm.rs` 18 · `worker_api.rs` 3 = **26행**을 들여와 **제거 15 · 유지 11** — 한 명제의 일곱 벌 복제를 「정본을 어디에 둘 것인가」로 판정(스냅숏 계약의 정본 = `analysis.rs::create` 의 복사 지점 3행 **유지**, 「`None` = 설정 이전에 시작된 분석」의 정본 = `settings.rs::analysis_language`) · rustdoc 링크 전용 교차 참조 2행(`anthropic_body` → `openai_body`) · 단정·선언 재진술 · 판단 갈림 2건 유지(조인이 깨진다 4행 · 「언어가 user turn 에 새어 들면 안 된다」 1행) — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) 「증분 재판정 ⑨」 · 맥락 [passes/2026-09-22-output-language-axis.md](passes/2026-09-22-output-language-axis.md) · **증분 재판정 ⑩**(2026-09-22): #114(슬라이스 6d)가 `worker_api.rs` `submit_document` 에 더한 2행을 판정해 **순 제거 1행** — 앞머리 「인수 문서가 서는 순간이 직전 분석의 편집을 이어받을 자리다(AC3.5)」는 doc-tracker 6d 행이 「재분석 문서가 저장되는 자리(`worker_api::submit_document`)에서 … 재생한다」로 축자에 가깝게 적고(②) AC 꼬리표는 ③ 이라 걷었고, **유지 1행**은 「저장과 같은 요청 안에서 이어받아야 워커가 5단계를 `succeeded` 로 보고하기 전에 충돌이 서 있다」 — 호출 **순서**가 만드는 동시성 계약이라 어느 복원 경로에도 없다(정책 본문 「동시성 계약」) · 2행 → 1행 재작성 — [passes/2026-09-22-conflict-axis.md](passes/2026-09-22-conflict-axis.md) 「증분」 · **자매 착지 재실측**(2026-09-22): 이 패스의 머지 직전에 #123(`77158c2` — `llm.rs` 의 `stub_answer` env 레이스 해소)이 먼저 착지해 이 행에 **순 +4행**(더한 6 · 지운 2)을 열었다. 판정이 아니라 **머지 시점 트리에서의 줄 수·지문 재고정**이고(「행 지문을 재현하는 법」 — 개행 포함 해시), 값은 602/`87c1c5cd…` → **606/`6c3ba120…`** 이다. 그 4행은 아래 **증분 재판정 ⑪** 이 닫았다. · **증분 재판정 ⑪**(2026-09-22 · `rct_20260922-0006`): 그 4행(추가 6 · 제거 2)을 명제 단위로 판정해 **순 제거 3행** — 테스트 본문의 `//` 3행(「The needle is deliberately unlike any other prompt in this binary … `an_ask()` is shared with the sibling tests」)은 열 줄 위 `///` doc 의 마지막 절과 **같은 명제의 두 벌째**이고, 남는 사실(`an_ask()` 가 공유 픽스처)은 같은 `mod tests` 의 네 테스트가 그것을 부르는 코드가 복원하며(①) 경위는 PR #123 본문 4항이 축자에 가깝게 다시 적는다(③) — 증분 재판정 ⑤·⑨ 의 「복제된 명제는 강제하는 코드 옆 한 벌만」에서 **정본을 `///` doc** 으로 골랐다 (두 자리가 같은 함수 안 열 줄 거리라 「코드 옆」은 둘 다 참이고, `///` 만이 *왜* 고유해야 하는지를 말한다) · **유지 3행**은 그 `///` 의 재작성분(2행 → 3행) — 프로세스 전역 env 를 `stub_answer` 가 **모든 Stub 모드 ask 에서** 읽는다는 **동시성 계약**이자 **실패 모드의 함정**(정책 본문의 유지 대상 두 항목)이고, #123 이 거짓이던 두 절을 정정한 자리다 · 유지분(순 +1)이 남아 #123 이전 값 602/`87c1c5cd…` 로는 돌아가지 않는다 · 606/`6c3ba120…` → **603/`1e39fbe5dc8715e585a1517d395aba3ca1f5be09b65c81faee52ea05ae514f7f`** — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) 「증분 재판정 ⑪」  · **자매 착지 재실측**(2026-09-22 · #121): 20차 패스의 머지 직전에 사람 PR **#121**(`b4a6b30`, AC1.5 — 끝난 단계만 다시 실행)이 착지해 이 행에 **순 +26행**(`analysis.rs` +5 · `worker_api.rs` +21)을 열었다. 판정이 아니라 **머지 시점 트리에서의 줄 수·지문 재고정**이고(「행 지문을 재현하는 법」 — 개행 포함 해시), 값은 603/`1e39fbe5…`(증분 재판정 ⑪ 직후) → **629/`39b8ea030947a0fcf65b26057198a392c7dd8ec308857ce92c4c3c3f82ca7ba8`** 다. 그 26행은 **판정하지 않고 다음 감지에 넘긴다** — **해소됨**(2026-09-25 · `rct_20260925-0002` · 아래 증분 재판정 ⑫). · **증분 재판정 ⑫**(2026-09-25 · `rct_20260925-0002`): 그 26행(#121 이 **새로 쓴 39행** · 제거 13행)을 **명제 단위로** 판정해 **순 제거 13행 · 유지 13행** — `analysis.rs` 6(AC1.5 조항 축자 인용과 `test/01 시나리오` 꼬리표를 걷어 요약 2행 → 1행 · 「뒤 단계는 무효화되지 않는다」 4행과 딸린 `///` 은 **세 벌째**라 전건 제거: 같은 doc 세 줄 위의 「Sibling stage rows are not touched…」 · 본문 SQL 의 `AND key = ?` · PRD AC1.5 **검증 방법**의 「뒤 단계의 결과와 사용자가 내린 승인·결정은 그대로 유지된다」) · `worker_api.rs` 7(`Gates` 네 필드 doc 은 필드 이름과 `offered_stages` 본문의 `if` 가 축자로 말하는 위에 `(AC1.3)` 꼬리표와 **rustdoc 링크만의 교차 참조**가 얹혀 제거 4 · `cross_cutting_document` 필드 doc 3행 → 1행 은 `offered_stages` doc 과 같은 명제의 두 벌째 · 비공개 `stored_landscape` 요약 1행은 이름과 `Option` 반환 타입이 축자) · **유지 13행** — `offered_stages` 의 규칙 doc 15행이 파일 전체가 가리키는 **정본**이고(같은 파일 `executable_stages` doc 의 「The rule is spelled out in [`offered_stages`]」), 「끝난 단계가 `pending` 으로 돌아오는 것은 사람이 다시 실행을 요청했을 때뿐」은 `retry_stage` 와 이 함수 **사이의** 계약이라 어느 한쪽 코드로도 복원되지 않으며, 「stage-2 문서가 없으면 3단계가 계획할 대상 없이 도착해 조용히 아무것도 안 한다」와 「`pending`, not "not succeeded"」는 정책 본문의 **실패 모드의 함정**·**동시성 계약**이다(「애매하면 남긴다」) · 모듈 머리 `//!` 라우트 목록 2행도 형제 항목 여섯 개가 모두 유지로 닫힌 목록의 한 칸이라 유지 · 603 판정 완료 + 유지 13 = **616 전건 판정 완료**(이 행의 미판정 증분 0) · 629/`39b8ea03…` → **616/`df4a5627c1acc0ae760914c1362c336a461e0a3670e050917ed36c59352a7696`** — [passes/2026-09-17-backend-concentrated.md](passes/2026-09-17-backend-concentrated.md) 「증분 재판정 ⑫」 |
| `tools/check-journey-prototype.js` · `backend/src/config.rs` (열린 통합 PR 무접촉 2파일) | 141 | `f9af32efe86eec6fad0875a7f259f36f12386aa7de41d6fb588240cb893c3a05` | 순 제거 35행(파일 머리 되풀이 인라인 마커 8 · 등록부 존재 이유 되풀이 · 같은 문구 5회 반복 5 · 절 제목 5 · 선언 재진술 — diff 기준 56행 삭제 · 4행 재작성, 차이는 블록 주석 본문이 지문에 안 보이기 때문) · 유지 130행 · 판단 갈려 남긴 것 11건 — [passes/2026-09-18-uncontested-harness-config.md](passes/2026-09-18-uncontested-harness-config.md) · **증분 재판정 ①**(2026-09-18): #60이 `config.rs`에 `Mode`·`Doubles` doc으로 더한 23행을 판정해 **순 제거 13행**(variant·시그니처 재진술 · 경계 식별자 필드 doc 6 · 정책 문서 인용 2) · 유지 10행(경계별 선택 불변식 · 안전 기본값) · **증분 재판정 ②**(2026-09-21): #92가 `Doubles.llm` 필드에 더한 doc 2행을 **전건 제거**(바로 위 `Doubles` doc 의 「the analysis worker holds its own set」 재진술 · 필드 이름) · 줄 수·지문은 #92 이전 값으로 복귀 — [passes/2026-09-18-uncontested-harness-config.md](passes/2026-09-18-uncontested-harness-config.md) 「증분 재판정 ②」 · **증분 재판정 ③**(2026-09-21): #107 이 `Doubles.repo_scan` 필드에 더한 doc 3행을 **전건 제거**(② 와 같은 형 — `Doubles` doc 의 경계별 선택 불변식 · doc-tracker 6b 행 · `docs/e2e-mocking-policy.md` EXT-03 배선 표) · 줄 수·지문은 #107 이전 값으로 복귀 — [passes/2026-09-18-uncontested-harness-config.md](passes/2026-09-18-uncontested-harness-config.md) 「증분 재판정 ③」 · **증분 재판정 ④**(2026-09-22): #115 가 `check-journey-prototype.js` 에 더한 4행 블록 주석을 판정해 **전건 유지 · 순 제거 0**(사고 기록 · 검사가 존재하는 이유 · 뮤테이션 내성 근거 — 복원 경로 ③(PR #115 본문)은 **실재하나**, 같은 잣대가 이 파일의 유지 선례 13행을 함께 뒤집으므로 17행 묶음 재판정의 몫이다) · 지문에 들어오는 것은 `/*` 여는 1행뿐이라 140 → **141** — [passes/2026-09-18-uncontested-harness-config.md](passes/2026-09-18-uncontested-harness-config.md) 「증분 재판정 ④」 |
| `e2e/tests/sc01-01-full-pipeline-run.spec.ts` · `e2e/tests/sc01-06-partial-retry.spec.ts` · `e2e/support/cluster.ts` · `e2e/smoke.sh` · `e2e/playwright.config.ts` (e2e 하네스 비경합 5파일) | 145 | `97a280f44314fe280874964f08e529e912f06c385b3c6b51937dbb92eedb0c2a` | 순 제거 49행(시나리오 문서 인용 10 · 절 제목 18 · 제목+AC 2 · 작업 흔적 2 · `finally` 재진술 2 · smoke 머리·인라인 6 · 나머지 선언 재진술 — diff 기준 52행 삭제 · 3행 재작성) · 유지 142행(`playwright.config.ts`는 전건 유지) · 기계 판독 `// 검증 시나리오:` 선언 2개 보존 — [passes/2026-09-18-e2e-uncontested.md](passes/2026-09-18-e2e-uncontested.md) · **증분 재판정 ①**(2026-09-20): #79가 `sc01-01`에 더한 순증 3행을 판정해 **순 제거 3행**(슬라이스 번호를 단 작업 흔적 2행 제거 · 목업 인용을 뺀 단정 의미 1행만 유지) · 범위는 145행 → 142행으로 돌아왔으나 **지문은 `fb27e0db…`가 아니라 `0e5c3d31…`**이다(살아남은 줄의 문면이 #79 이전과 다르다) — [passes/2026-09-20-pipeline-cross-cutting-axis.md](passes/2026-09-20-pipeline-cross-cutting-axis.md) · **증분 재판정 ②**(2026-09-20): #83 이 `sc01-01` 에 연 순증 2행을 판정해 **순 제거 2행**(슬라이스 번호를 단 작업 흔적 · 화면 전이 서술 · 교차 참조 `(선례: sc01-05)`) · 유지 2행(「셋업을 API 로 끝내도 로드는 자격증명 화면에서 시작한다」는 상태 머신 함정) · 범위는 144행 → 142행이지만 **지문은 `0e5c3d31…` 가 아니라 `93be69ea…`** 다 — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) · **증분 재판정 ③**(2026-09-21): #99(`899800e`)가 `sc01-01`(+5) · `sc01-06`(+5) 에 더한 10행을 판정해 **순 제거 10행**(원장 항목 번호 ⑼⒃㉑⑿ 를 현재형으로 인용하는 작업 흔적 5 — 인용 대상 4행은 같은 커밋이 doc-tracker 에서 닫아 지웠다 · PR #99 계획 1·2·3 과 doc-tracker 변경 이력의 축자 · 시나리오 6 기대 결과 재진술 · 바로 아래 단정이 그 문장 자체인 것) · 유지 0행 · 줄 수와 지문이 **둘 다 #99 이전 값으로 되돌아왔다**(142 / `93be69ea…` — 부모 `3567755` 실측과 바이트 동일) — [passes/2026-09-18-e2e-uncontested.md](passes/2026-09-18-e2e-uncontested.md) 「증분 재판정 ③」 · **증분 재판정 ④**(2026-09-25, 30차 패스): #152(`ea04fe0`)가 `playwright.config.ts` 에 연 3행을 판정해 **전건 유지 · 순 제거 0** — `actionTimeout` 이 없으면 오조준 클릭 하나가 테스트 상한까지 매달리고 `test timeout` 만 남긴다는 것은 **라이브러리의 문서화되지 않은 경계**이고(본문 1차 판정이 같은 파일의 `workers: 1` 5행을 같은 이유로 「전건 유지」로 닫은 선례 그대로), `30s > 관측 최장 21.2s` · `600s/20` 은 **실측 근거**다 — PR #152 본문 ⑷ 절에 같은 말이 있으나(③) PR 본문은 편집 지점에서 읽히지 않고, 이 줄들은 「이 값이 무엇과 같아야 하는가」를 말한다 · **열린 #148 에 대해 불변**: `sc01-01`·`sc01-06` 을 고치지만 주석 줄은 0행이라 `main`·`main+#148`·`main+#153` 세 트리에서 145/`97a280f4…` 가 바이트 동일하다 — [passes/2026-09-18-e2e-uncontested.md](passes/2026-09-18-e2e-uncontested.md) 「증분 재판정 ④」 |
| `backend/src/acceptance.rs` · `backend/tests/acceptance.rs` · `e2e/support/acceptance.ts` · `e2e/tests/sc02-01-acceptance-from-logic.spec.ts` · `e2e/tests/sc02-04-user-facing-acceptance-doc.spec.ts` · `frontend/src/FeatureAcceptance.tsx` (인수 축 비경합 6파일) | 142 | `2ff8f6ff8251699d1571004d3ebb518ebcc373bed06bf175d5f8a40e17dba5d3` | 순 제거 127행(PRD-2 AC2.1~AC2.3 조항 재진술 · doc-tracker 등재 편차 재진술 10 · 절 제목 9 · 목업 카피·페인포인트 인용 · 단언 재진술 · 이름 재진술 — diff 기준 180행 삭제 · 52행 재작성) · 유지 141행 · 판단이 갈려 남긴 것 4건 · 기계 판독 `// 검증 시나리오:` 2건과 `mock-exception:` 2건 보존 — [passes/2026-09-18-acceptance-axis.md](passes/2026-09-18-acceptance-axis.md) · **증분 재판정 ①**(2026-09-21): #93이 `acceptance.rs`에 더한 1행(`// Required-but-nullable; see feature_candidates::schema.`)은 두 줄 위 `required` 배열과 바로 아래 `["string","null"]` 이 축자로 말하는 것(①) + 교차 참조뿐이라 **제거 1행** — 줄 수·지문이 #93 이전 값(141 / `e38dcf20…`)으로 되돌아왔다 — [passes/2026-09-18-acceptance-axis.md](passes/2026-09-18-acceptance-axis.md) 「증분 재판정 ①」 · **증분 재판정 ②**(2026-09-21): #112 가 `FeatureAcceptance.tsx` 에 더한 5행을 판정해 **순 제거 4행**(AC3.3 꼬리표 「가려질 뿐 보관소에 남는다」 · `reload` JSDoc — 파일 머리의 화면 불변식이 정본 · 「보관소만 있는 상태」 — PR #112 · 내부 fn `Archive` JSDoc — JSX 카피와 `disabled={… !restorable}`) · **유지 1행**(「문서와 함께 읽어 두 목록이 같은 시점의 서버 상태를 그린다」 — `Promise.all` 을 고른 이유, 판단이 갈려 남긴 것 5건째) · 줄 수·지문은 141/`e38dcf20…` → 146/`655305bc…`(트리거) → **142/`2ff8f6ff…`** — [passes/2026-09-18-acceptance-axis.md](passes/2026-09-18-acceptance-axis.md) 「증분 재판정 ②」 · **미판정 증분 없음** · **증분 재판정 ③**(2026-09-22): #114 가 `acceptance.rs` 에 더한 4행을 **전건 제거** — 스텁 리비전 3 의 재현 의도 2행은 doc-tracker 6d 행(「스텁 트리에 리비전 3 … 을 두어 첫 문장을 다르게 읽는 코드 변경을 결정적으로 재현했다」)의 사본이고, `stub_logic` 의 2행은 `02#시나리오 8` 을 인용한 제품 문서 재진술이다(같은 doc-tracker 행이 「리비전 2 의 문장 불변은 단위 테스트가 지킨다 — 02#8 의 단정 보호」로 다시 적는다) · 리비전 어휘의 **정본은 `repo_scan::Revision` 의 variant doc** 으로 두었다(같은 패스에서 유지) · 줄 수·지문이 #114 이전 값 **142 / `2ff8f6ff…` 로 바이트 동일 복귀** — [passes/2026-09-22-conflict-axis.md](passes/2026-09-22-conflict-axis.md) 「증분」  · **자매 착지 재실측**(2026-09-22 · #121): 20차 패스의 머지 직전에 사람 PR **#121**(`b4a6b30`, AC1.5 — 끝난 단계만 다시 실행)이 착지해 이 행에 **순 +7행**(`backend/tests/acceptance.rs` +7)을 열었다. 판정이 아니라 **머지 시점 트리에서의 줄 수·지문 재고정**이고(「행 지문을 재현하는 법」 — 개행 포함 해시), 값은 142/`2ff8f6ff…` → **149/`a459c9173ce41a9f347923a75257319978c4f02338cac2d5a9cc2d7695393fd9`** 다. 그 7행은 **판정하지 않고 다음 감지에 넘긴다** — **해소됨**(2026-09-25 · `rct_20260925-0002` · 아래 증분 재판정 ④). · **증분 재판정 ④**(2026-09-25 · `rct_20260925-0002`): 그 7행을 **전건 제거** — 테스트 doc 3행은 fn 이름 `rerunning_a_succeeded_stage_offers_that_stage_alone` 이 첫 절을 축자로 담고(①) 「Later stages keep their output and the reviewer keeps their decisions」가 PRD AC1.5 **검증 방법**의 번역이며(②) `AC1.5 / test/01 시나리오 8` 은 꼬리표다(②③ — 비공개 테스트 fn 이라 `pub` doc 요약 유지 규칙 대상이 아니고 기계 판독 `// 검증 시나리오:` 규약과도 형태가 다르다) · `fetch` 2행은 `offered_stages` doc 의 「`fetch` is always offered…」와 두 벌째이고 아래 `cases` 리터럴 다섯 개가 전부 `"fetch"` 로 시작하는 것이 축자(①) · `Stage 3 plans over the stored landscape` 1행은 바로 아래 `if key == "discovery_strategy"` 분기와 `crossCuttingDocument` 단언이(①) · 검토자 결정 1행은 바로 아래 `assert_eq!(approved, 1)` 이(①) 말한다 · 줄 수·지문이 #121 이전 값 **142 / `2ff8f6ff…` 로 바이트 동일 복귀**다(증분 재판정 ③ 이 #114 에 대해 닫은 것과 같은 형태) — [passes/2026-09-18-acceptance-axis.md](passes/2026-09-18-acceptance-axis.md) 「증분 재판정 ④」 · **증분 재판정 ⑤**(2026-09-25, 28차 패스): #126 이 `FeatureAcceptance.tsx` 에 연 1행(이력 진입점의 사유 — 여정 `JRN-restore-history` 「이력 진입점이 검수 화면 깊숙이 있으면 이 여정 자체가 시작되지 않는다」의 축자(②)이고 PR #126 본문도 같은 문장을 적는다(③))을 **제거** · 줄 수·지문은 #126 이전 값 142/`2ff8f6ff…` 로 **바이트 동일 복귀** — [passes/2026-09-25-history-restore-axis.md](passes/2026-09-25-history-restore-axis.md) 「증분 재판정 ⑤」 |
| `backend/src/bin/worker.rs` · `deploy/e2e/kustomization.yaml` · `deploy/k8s/deployment.yaml` · `deploy/k8s/secret.yaml.example` · `deploy/k8s/worker-deployment.yaml` · `e2e/tests/sc04-07-api-availability-without-workers.spec.ts` · `e2e/tests/sc04-08-worker-horizontal-scale.spec.ts` (워커 · 더블 배선 축 비경합 7파일) | 190 | `80748ab3041408f76823e2cac502b043fef3ebe055628efc883742b36ae593a5` | 순 제거 122행(5단계 열거·선언 재진술 · 시나리오 본문 인용 12 · 더블 배선 설명 14 · AC 조항 인용 9 · 3중 복제된 경위 서술 19 · 작업 흔적 8 · 절 제목 3 · 세 벌 중복 중 두 벌 — diff 기준 185행 삭제 · 63행 재작성) · 유지 185행 · 판단이 갈려 남긴 것 3건 · 기계 판독 `// 검증 시나리오:` 2건 보존 — [passes/2026-09-18-worker-double-axis.md](passes/2026-09-18-worker-double-axis.md) · **증분 재판정 ①**(2026-09-19): #71이 `bin/worker.rs`에 더한 12행을 판정해 **순 제거 7행**(같은 문장의 네 번째 벌 · AC 꼬리표 · 라우트 선택 근거) · 유지 5행(한 feature 의 실패가 잡을 죽이지 않는다는 격리 계약) — [passes/2026-09-19-dependencies-axis.md](passes/2026-09-19-dependencies-axis.md) · **증분 재판정 ②**(2026-09-21): #92가 `deploy/e2e/kustomization.yaml` 의 API env 에 더한 2행(「편집 제안은 워커가 아니라 이 프로세스가 부른다」 — doc-tracker·PR #92 재진술)을 **전건 제거** · 줄 수·지문은 #92 이전 값으로 복귀 — [passes/2026-09-18-worker-double-axis.md](passes/2026-09-18-worker-double-axis.md) 「증분 재판정 ②」 · **증분 재판정 ③**(2026-09-21): #107 이 같은 파일의 API env 에 더한 2행(`FEATUREDOC_DOUBLE_REPO_SCAN` 위 — doc-tracker 6b 행 · PR #107 · e2e-mocking-policy 배선 표 재진술)을 **전건 제거** · 줄 수·지문은 #107 이전 값으로 복귀 — [passes/2026-09-18-worker-double-axis.md](passes/2026-09-18-worker-double-axis.md) 「증분 재판정 ③」 · **증분 재판정 ④**(2026-09-22): #108 이 `bin/worker.rs` 에 더한 7행(`Claim.llm_language` 필드 doc 2 — 스냅숏·일곱 벌 두 명제의 사본 · `language_for` doc 5 — 본문이 `job.llm_language` 를 읽고, 바로 위 `provider_for` doc 이 같은 이유를 이미 말한다)을 **전건 제거** · 줄 수·지문은 #108 이전 값 190/`632b0475…` 로 바이트 동일 복귀 — [passes/2026-09-18-worker-double-axis.md](passes/2026-09-18-worker-double-axis.md) 「증분 재판정 ④」 · 맥락 [passes/2026-09-22-output-language-axis.md](passes/2026-09-22-output-language-axis.md)  · **자매 착지 재실측**(2026-09-22 · #121): 20차 패스의 머지 직전에 사람 PR **#121**(`b4a6b30`, AC1.5 — 끝난 단계만 다시 실행)이 착지해 이 행에 **순 +3행**(`backend/src/bin/worker.rs` +3)을 열었다. 판정이 아니라 **머지 시점 트리에서의 줄 수·지문 재고정**이고(「행 지문을 재현하는 법」 — 개행 포함 해시), 값은 190/`632b0475…` → **193/`d086c236228b02d1086ce4ca03eb43d693c1b97c7e9540140f9155f30367ed45`** 다. 그 3행은 **판정하지 않고 다음 감지에 넘긴다** — **해소됨**(2026-09-24 · `rct_20260924-0001` · 아래 증분 재판정 ⑤). · **증분 재판정 ⑤**(2026-09-24 · `rct_20260924-0001`): 위 등재분 **#121 3행**과 그 뒤 착지한 **#137**(`fd6cdad`)의 **2행**, 합 **5행**(전부 `backend/src/bin/worker.rs`)을 판정해 **순 제거 3행** — 「Seeded from the claim when stage 3 re-runs without stage 2 (AC1.5)」 2행은 **같은 PR 이 `Claim` 에 더한 필드 doc 의 두 벌째**이고(정본은 와이어 계약이 사는 필드 쪽) `(AC1.5)` 는 AC 꼬리표다 · 임대 갱신 2행은 뒤 절이 **626행 `// The model call is the long one in this job; renew before it as `fetch` does.` 의 두 벌째**라 **왜 한 번 더 갱신하는가**만 남겨 1행으로 재작성했다 · **유지 2행**(필드 doc 1행 — 자매 필드 셋이 같은 모양으로 살아 있는 「언제 채워지는가」 규약 · 임대 재작성 1행 — 정책 본문이 유지 대상으로 명시 열거한 동시성 계약이라 ③ 과 갈려도 「애매하면 남긴다」). 195(#137 착지 후) → **192 / `b36ee596e5bf5353de6ca3cb8ea3dac138ca58ec2370c5d0ca493a4968e2dcc3`**. **이 행에 미판정 증분 없음** — [passes/2026-09-18-worker-double-axis.md](passes/2026-09-18-worker-double-axis.md) 「증분 재판정 ⑤」 |
| `backend/src/dependencies.rs` · `backend/tests/dependencies.rs` · `frontend/src/FeatureDependencies.tsx` · `e2e/support/dependencies.ts` · `e2e/tests/sc02-05-dependency-extraction.spec.ts` · `e2e/tests/sc02-06-dependency-reverse-query.spec.ts` · `e2e/tests/sc02-07-dependency-export.spec.ts` (의존성 축 비경합 7파일) | 112 | `e2ff313a778df1e7388fc7bbb07bd72c323ead751d854c5db743c3c2eee133e9` | 순 제거 137행(PRD-2 AC2.4·AC2.5 조항 재진술 17 · 목업·여정·doc-tracker 인용 21 · 시나리오 문서 인용 22 · 테스트 이름을 다시 쓴 doc 11 · 단정 재진술 18 · 선언 재진술 19 · 구분선 3 · 세 벌 중복 중 두 벌 — diff 기준 189행 삭제 · 50행 재작성) · 유지 112행 · 판단이 갈려 남긴 것 6건 · 기계 판독 `// 검증 시나리오:` 3건과 `mock-exception:` 1건 보존 — [passes/2026-09-19-dependencies-axis.md](passes/2026-09-19-dependencies-axis.md) · **증분 재판정 ①**(2026-09-21): #93이 `dependencies.rs`에 더한 1행(4행과 같은 문장의 두 번째 벌)은 같은 근거로 **제거 1행** — 줄 수·지문이 #93 이전 값(112 / `e2ff313a…`)으로 되돌아왔다 — [passes/2026-09-19-dependencies-axis.md](passes/2026-09-19-dependencies-axis.md) 「증분 재판정 ①」 · **미판정 증분 없음** |
| `backend/src/diff.rs` · `backend/tests/diff.rs` · `frontend/src/AnalysisDiff.tsx` · `e2e/tests/sc02-08-reanalysis-diff.spec.ts` · `backend/src/repo_scan.rs` · `frontend/src/AnalysisProgress.tsx` · `backend/src/lib.rs` (재분석 diff 축 비경합 7파일) | 149 | `bed7da485434122680d635110535ba2624a3c7c2f939599ab911615035efc7c3` | 순 제거 168행(화면 두 개의 머리 주석 50 — 목업·여정·doc-tracker 인용과 「이 슬라이스가 그리지 않는 것」 목록이 원장에 이미 있다고 주석 스스로 적는다 · 시나리오 문서 인용 12 · 선언·시그니처 재진술 41(rustdoc 링크만의 교차 참조와 이미 낡은 `(future)` 포함) · 테스트 이름을 다시 쓴 doc 27 · 네 벌까지 복제된 명제 중 잉여 18 · 구분선 8 · AC 꼬리표) · 유지 132행 · 판단이 갈려 남긴 것 5건 · 기계 판독 `// 검증 시나리오:` 1건 · `mock-exception:` 1건 · **화면 머리의 목업 매핑 2건**(M1 이 읽는다) 보존 — [passes/2026-09-19-reanalysis-diff-axis.md](passes/2026-09-19-reanalysis-diff-axis.md) · **증분 재판정 ①**(2026-09-21): #99(`899800e`)가 `AnalysisProgress.tsx` 에 더한 4행을 판정해 **순 제거 3행**(`subOf` 머리의 「서버 사유를 그리지 않는다」 3행 — doc-tracker 변경 이력 ⑿ · PR #99 계획 3 · 바로 아래 `return '실패했어요'` 와 `stage-failed` 안내의 삼중) · 유지 1행(「한 번에 실패하는 단계는 하나 — 파이프라인이 거기서 멈춘다」 — `stages.find` 의 전제인 백엔드 불변식인데 `docs/` · `pipeline.rs` 어디에도 문장으로 없다, 판단이 갈려 남김) · 지문 밖 JSX 블록 연속행 3행(「Stage 3 is deliberately not one of them …」 — doc-tracker ⒃ · PR 계획 2 축자)도 함께 제거 · 132행 → 133행(판정 시점 `4aa7a8eb…`) — [passes/2026-09-19-reanalysis-diff-axis.md](passes/2026-09-19-reanalysis-diff-axis.md) 「증분 재판정 ①」 · **증분 재판정 ②**(2026-09-21): #101(`3d147d6`, 슬라이스 ⑪)이 증분 재판정 ①의 머지 직전에 `STAGE_TITLES` 머리에 더한 2행을 명제 단위로 판정해 **순 제거 2행**(「wire `key` 로 고른다 · 서버 `title` 은 fallback」 — 바로 아래 `STAGE_TITLES[stage.key] ?? stage.title` 그 자체 · 「시드 `title` 은 enqueue 때 영속돼 카피를 바꿔도 옛 행은 옛 문구」 — doc-tracker 슬라이스 ⑪ 문단 두 벌 + PR #101 본문 축자 · 「화면이 카피를 소유한다」 — 같은 문단·PR 제목·커밋 제목) · 유지 0행(유일한 「저장소 제약의 함정」 명제가 docs 에 문장째로 있어 ①의 유지 1행과 같은 기준·반대 결론) · 비주석 코드 무접촉(스트립 잔여 md5 부모와 동일) · 줄 수·지문이 **증분 재판정 ① 시점 값으로 되돌아왔다**(135 / `47fdf929…` → 133 / `4aa7a8eb…`) — [passes/2026-09-19-reanalysis-diff-axis.md](passes/2026-09-19-reanalysis-diff-axis.md) 「증분 재판정 ②」 · **미판정 증분 없음** · **증분 재판정 ③**(2026-09-22): #114 가 `repo_scan.rs` 에 +14 −5(제자리 재작성 — 같은 명제 5행이 한국어에서 영어로 다시 쓰였다) · `AnalysisDiff.tsx` 에 +6 을 열어 **15행 증분**을 판정해 **순 제거 3행** — `repo_scan.rs` 의 variant doc 2행(「The tree every first analysis sees.」 · 「Paths this revision adds on top of the previous one.」)은 variant·필드 이름이 그대로 말하는 ① 선언 재진술이고, `AnalysisDiff.tsx` 의 `onResolve` prop JSDoc 1행은 이름과 화살표 카피의 재진술 · **유지 12행** — 「리비전은 더하기만 한다(지우거나 이름을 바꾸면 그 위에 세워진 feature 키가 함께 움직여 「같은 feature 의 표현이 갱신됐다」를 관측할 수 없다)」 4행은 **스텁 충실도의 불변식**이고 리비전 2·3 의 variant doc 5행은 이 축의 **정본**(원장 4행의 사본 2건을 이 판정이 걷었다) · 「한 곳을 읽어야 넘어간다」 표시의 수명 4행과 「배너가 세는 것은 충돌 행이 아니라 **기능**이다」 1행은 화면 불변식이라 유지 — [passes/2026-09-22-conflict-axis.md](passes/2026-09-22-conflict-axis.md) 「증분」  · **자매 착지 재실측**(2026-09-22 · #121): 20차 패스의 머지 직전에 사람 PR **#121**(`b4a6b30`, AC1.5 — 끝난 단계만 다시 실행)이 착지해 이 행에 **순 +1행**(`frontend/src/AnalysisProgress.tsx` +1)을 열었다. 판정이 아니라 **머지 시점 트리에서의 줄 수·지문 재고정**이고(「행 지문을 재현하는 법」 — 개행 포함 해시), 값은 145/`2e366521…` → **146/`db03aa4ae962896e95d567dcc8eeacc77d43a73088fc3f6d65bc611832aebfc4`** 다. 그 1행은 **판정하지 않고 다음 감지에 넘긴다** — **해소됨**(2026-09-24 · `rct_20260924-0001` · 아래 증분 재판정 ④). · **증분 재판정 ④**(2026-09-24 · `rct_20260924-0001`): 위 등재분 **#121 1행**과 **#137**(`fd6cdad`)이 `backend/src/repo_scan.rs` 에 연 **14행**, 합 **15행**을 판정해 **순 제거 12행** — `stub_read` 의 충실도 경계 doc 3행은 **`docs/e2e-mocking-policy.md` 39~40행이 세 절을 모두 적는다**(②, 같은 PR 이 넣은 등재의 사본이고 그 문서가 정본) · `read_files` doc **본문** 5행은 #137 PR 본문 「개별 파일 실패(404·5xx)는 건너뛰고 경로만으로 진행, 토큰 없음은 기존 스캔과 같은 오류」(③)와 코드(①)가 복원하며 끝 문장은 rustdoc 링크만의 교차 참조다(**요약 1줄은 남긴다** — 같은 파일 `scan` 의 생존 규약) · `excerpt` doc 2행은 바로 아래 `is_char_boundary` 루프가 말하고(①) **뒤 절은 사실과도 어긋난다**(`from_utf8_lossy` 가 이미 돌아 비경계 슬라이스는 대체문자가 아니라 패닉이다) · `AnalysisProgress.tsx` 3행은 AC 조항 재진술(②)·#121 PR 본문(③)·바로 다음 줄 가드 `!ACTIVE.has(analysis.status)`(①)가 네 절을 모두 복원해 전건 제거 · `truncated` 필드 doc 은 **왜 알려야 하는가**만 남겨 2행 → 1행 재작성 · **유지 3행**. 160(#137 착지 후) → **148 / `d55bd63e1adc1672800636757f49c4ef9e406ea6e6f9b17b0117f743feac6481`**. **이 행에 미판정 증분 없음** — [passes/2026-09-19-reanalysis-diff-axis.md](passes/2026-09-19-reanalysis-diff-axis.md) 「증분 재판정 ④」 · **증분 재판정 ⑤**(2026-09-25, 28차 패스): #126 이 `diff.rs::scenario_of` doc 에 연 2행 중 1행(「저장된 문서에서 읽을 때(`scenarios_of`)와 같은 규칙」 — 그 호출 관계는 코드가 말한다(①))을 **제거** · 유지 1행(요약) — [passes/2026-09-25-history-restore-axis.md](passes/2026-09-25-history-restore-axis.md) 「증분 재판정 ⑤」 |
| `backend/src/discovery_strategy.rs` · `backend/src/feature_candidates.rs` · `backend/tests/strategy.rs` · `backend/tests/candidates.rs` · `frontend/src/DiscoveryStrategy.tsx` · `frontend/src/FeatureCandidates.tsx` · `e2e/tests/sc01-04-strategy-edit-and-approve.spec.ts` · `e2e/tests/sc01-07-candidate-rejection-carryover.spec.ts` (후보·전략 축 비경합 8파일) | 141 | `a431e905c7f2b2e08b033c28b6776705c4995a25a6e21bfb3034288900842d70` | 순 제거 222행(파일 머리 넷의 AC1.3·AC1.4 검증 방법 재진술 56 · 목업·여정·doc-tracker 인용 27 · 구분선 21 · AC 꼬리표 · 선언·시그니처 재진술 · 테스트 이름을 다시 쓴 doc · **여섯 벌까지 복제된 명제 중 잉여** — diff 기준 290행 삭제 · 65행 재작성) · 유지 140행 · 판단이 갈려 남긴 것 6건 · 기계 판독 `// 검증 시나리오:` 2건 · `mock-exception:` 3건 · **화면 머리의 목업 매핑 2건**(M1 이 읽는다) 보존 — [passes/2026-09-19-candidate-strategy-axis.md](passes/2026-09-19-candidate-strategy-axis.md) · **증분 재판정 ①**(2026-09-21): #93이 `feature_candidates.rs`에 더한 2행(「required-but-nullable — OpenAI `strict` 는 `required` 누락을 400 으로 거부」)은 같은 명제의 정본이 `llm.rs::assert_strict_schema` 옆에 있고(1행 재판정 ⑤) 같은 파일의 테스트 `schema_is_accepted_by_openai_strict_mode` 가 이름으로 그 자리를 가리키므로 **제거 2행**(12차 패스의 「복제된 명제는 강제하는 코드 옆 한 벌만」) — [passes/2026-09-19-candidate-strategy-axis.md](passes/2026-09-19-candidate-strategy-axis.md) 「증분 재판정 ①」 · **증분 재판정 ②**(2026-09-21): #99(`899800e`)가 `sc01-04`(+5) · `sc01-07`(+1) · `DiscoveryStrategy.tsx`(+5) 에 더한 11행을 판정해 **순 제거 10행**(원장 번호 ⑼⒃ 를 현재형으로 인용하는 작업 흔적 · 바로 아래 세 단정이 그 문장 자체인 것 · doc-tracker 변경 이력 ⑼ 를 축자로 옮긴 JSX 머리 · 빈 `catch` + `setInterval` 이 그 문장 자체 · `POLL_MS` 이름·값만의 교차 참조 · 「승인이 4단계를 재큐잉한다」 — diff 기준 11행 삭제 · 1행 재작성) · 유지 1행(폴링을 택한 이유 — `docs/` 어디에도 「왜 한 번 읽지 않고 폴링하는가」 가 없다, 재작성한 1행) · 지문 밖 JSX 블록 연속행 3행도 함께 제거 · 151행 → **141행** — [passes/2026-09-19-candidate-strategy-axis.md](passes/2026-09-19-candidate-strategy-axis.md) 「증분 재판정 ②」 · **증분 재판정 ③**(2026-09-21): #107 이 `FeatureCandidates.tsx` 의 `onFinish` prop 에 더한 JSDoc 1행을 **제거**(`disabled={list.undecided !== 0}` 과 버튼 카피 「결정 끝 — 빠진 기능 확인」이 그 문장 · doc-tracker 6b 행) · 줄 수·지문은 #107 이전 값 141/`a431e905…` 로 복귀 — [passes/2026-09-19-candidate-strategy-axis.md](passes/2026-09-19-candidate-strategy-axis.md) 「증분 재판정 ③」 · **증분 재판정 ④**(2026-09-21): #112 가 `FeatureCandidates.tsx` 의 내부 fn `quotedDeletion` 에 더한 JSDoc 1행을 **제거**(`prev.reason === null ? '(' + when + ')' : …` 과 `reason: string | null` 이 그 문장 · doc-tracker 편차 행 「사유(없으면 날짜만)」) · 줄 수·지문은 #112 이전 값 141/`a431e905…` 로 복귀 — [passes/2026-09-19-candidate-strategy-axis.md](passes/2026-09-19-candidate-strategy-axis.md) 「증분 재판정 ④」 · **미판정 증분 없음** |
| `backend/src/pipeline.rs` · `backend/src/cross_cutting.rs` · `backend/tests/documents.rs` · `backend/tests/progress.rs` · `frontend/src/CrossCuttingConcerns.tsx` · `e2e/tests/sc01-02-repo-out-of-scope.spec.ts` · `e2e/tests/sc01-03-cross-cutting-determinism.spec.ts` · `e2e/tests/sc01-05-resume-after-app-exit.spec.ts` · `e2e/tests/sc02-03-contradiction-separation.spec.ts` (파이프라인 · 횡단 관심사 축 비경합 9파일) | 150 | `23fdab1d6e7206c5f21efc8253d7cbb901b99d83c20fc37495f9108dc2385fd1` | 순 제거 198행(AC1.2 조항·시나리오 2·3·5 본문 재진술 · **13벌까지 복제된 워커 임대 문단** · 목업·여정 인용 · 작업 흔적(PR·슬라이스·task id) · 선언·시그니처 재진술 · 구분선 25 · 단정을 산문으로 옮긴 주석 6) · 유지 139행 · 판단이 갈려 남긴 것 5건 · **낡아서 거짓이 된 주석 2건 적발**(`cross_cutting.rs`의 「목업은 4축만 그린다 · 편차 등재됨」 — 목업은 5축을 그리고 doc-tracker는 「등재하지 않는다」고 적는다 / `sc01-05`의 「3 of 5」 — 같은 트리의 `pipeline.rs`는 「다섯 단계 모두 돈다」고 적는다) · 기계 판독 `// 검증 시나리오:` 4건 · `mock-exception:` 2건 · **화면 머리의 목업 매핑 1건**(M1 이 읽는다) 보존 — [passes/2026-09-20-pipeline-cross-cutting-axis.md](passes/2026-09-20-pipeline-cross-cutting-axis.md) · **증분 재판정 ①**(2026-09-20): #83 이 `sc01-02`(+1) · `sc01-05`(+2) 에 연 순증 3행을 판정해 **순 제거 2행**(목업 카피 인용 · 슬라이스 ⑦ 작업 흔적) · 유지 2행(같은 버튼을 연달아 두 번 누르는 코드가 복사 실수로 읽히지 않게 하는 한 줄씩) · 139행이 아니라 **140행**으로 내려온다 — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) · **증분 재판정 ②**(2026-09-20): #85(제품 fix)가 `sc01-02` 에 연 순증 1행을 판정해 **순 제거 1행**(목업 카피 인용 · 바로 아래 세 줄의 단정이 그 문장 자체) · 유지 0행 · 줄 수와 지문이 **둘 다 #85 이전 값으로 되돌아왔다**(140 / `03babd78…` — #85 의 부모 `e340bdc` 실측과 바이트 동일) — [passes/2026-09-20-tools-checker-axis.md](passes/2026-09-20-tools-checker-axis.md) · **미판정 증분 없음**  · **자매 착지 재실측**(2026-09-22 · #121): 20차 패스의 머지 직전에 사람 PR **#121**(`b4a6b30`, AC1.5 — 끝난 단계만 다시 실행)이 착지해 이 행에 **순 +2행**(`backend/tests/progress.rs` +2)을 열었다. 판정이 아니라 **머지 시점 트리에서의 줄 수·지문 재고정**이고(「행 지문을 재현하는 법」 — 개행 포함 해시), 값은 140/`03babd78…` → **142/`5a4aea6e4774ce331a8212f020eacbf14b63e41c79001cdd79138ea57d91d739`** 다. 그 2행은 **판정하지 않고 다음 감지에 넘긴다** — **해소됨**(2026-09-24 · `rct_20260924-0001` · 아래 증분 재판정 ④). · **증분 재판정 ③**(2026-09-24 · `rct_20260922-0009`): #134 가 `cross_cutting.rs` 에 더한 3행과 #132 가 `CrossCuttingConcerns.tsx` 에 더한 3행, 합 **6행**을 판정해 **순 제거 5행** — `AXIS_GUIDE` doc 머리의 「축마다 무엇을 묻는지」는 상수 이름과 리터럴이 그대로 말하는 선언 재진술이고, 뒤따르는 「`AXES` 라벨은 화면 카피」는 `AXES` 자신의 doc(「the screen renders the axes in this order」)과 PR #134 본문(「한글 라벨(`AXES`)은 화면 카피라 건드리지 않았다」)의 두 벌째다(①③) · 「예시는 증거의 **종류**이지 구체 경로가 아니다」는 **열 줄 아래 프롬프트 리터럴이 모델에게 그대로 말하는 문장**(`The examples describe kinds of evidence, not files in this repository`)의 두 벌째라 **강제하는 쪽 한 벌만** 남긴다(①, 증분 재판정 ⑤·⑨·⑪ 의 「복제된 명제는 강제하는 코드 옆 한 벌만」과 같은 잣대) · 근거 줄의 `' · '` join 실패 모드 2행은 PR #132 본문 증상절(「경로를 ` · ` 로 이어 붙여 경로 중간에서 꺾이고 구분점이 다음 줄 앞에 옴」)이 축자에 가깝게 적고(③), 같은 명제의 정본은 `index.css` 쪽 블록이었다 · legend 배치 1행은 **바로 아래 `section.items.some((item) => item.evidence.length === 0)` 가드가 그 문장 자체**이고(①) 목업(②)·PR #132 수정절(③)이 함께 복원한다 · **유지 1행**은 `AXIS_GUIDE` 리터럴을 묶는 함정 — 「프롬프트에 박힌 구체 경로는 트리에 없어도 모델이 인용할 수 있다」는 *왜* 예시가 종류에 머물러야 하는지이고, 프롬프트 리터럴은 **규칙만 말하고 이유를 말하지 않으며** 이를 강제하는 테스트가 없다(3행 → 1행 재작성) · 기재 142(트리거 `27d9b81` 에서 바이트 일치) → 유입 후 148 → 판정 후 **143 / `515d540af0130cb52fee733de80ed48f2dfd6aaa9831cbf3c28bcd7e541ddf2b`** — [passes/2026-09-20-pipeline-cross-cutting-axis.md](passes/2026-09-20-pipeline-cross-cutting-axis.md) 「증분 재판정 ③」 · **자매 착지 재실측**(2026-09-24 · #137): 21차 패스의 머지 직전에 사람 PR **#137**(`fd6cdad`, 횟단 관심사 단계에 진입점·매니페스트 문맥 제공)이 착지해 이 행에 `cross_cutting.rs` 주석 **순 +30행**(148 → 178)을 열었다. 판정이 아니라 **머지 시점 트리에서의 줄 수·지문 재고정**이고(「행 지문을 재현하는 법」 — 개행 포함 해시), 값은 143/`515d540a…`(base `074c325` 측정) → **173/`f184887421a70d8275f2369a454708f8f471576e8188132919cc4b9bcb9a2d4e`** 다. 이 패스의 순 제거 −5행은 그대로다 — 착지분 기준 178 → 173 · 원장 10행 90 → 89 · 전역 2700 → 2694 로 **행 합 −6 == 전역 −6, 잔차 0**. #137 이 들인 30행은 **판정하지 않고 다음 감지에 넘긴다** — **해소됨**(2026-09-24 · `rct_20260924-0001` · 아래 증분 재판정 ④). · **자매 착지 재실측**(2026-09-24 · #138): 위 #137 재고정 직후, 머지 전에 **#138**(`95d3395`, 경로 400개 상한을 디렉터리 고른 샘플로)이 연이어 착지해 같은 파일에 주석 순 **+26행**(178 → 204)을 더 열었다. 역시 판정이 아니라 **머지 시점 트리에서의 재고정**이고, 값은 173/`f1848874…` → **199/`db92b2402cc84340e14148375e64e4080cdb16b1ad27b631ab7997c0cc2d669c`** 다. 순 제거 −5행은 여전히 그대로다 — 착지분 기준 204 → 199 · 원장 10행 90 → 89 · 전역 2726 → 2720 으로 **행 합 −6 == 전역 −6, 잔차 0**. #138 이 들인 26행도 **판정하지 않고 다음 감지에 넘긴다** — **해소됨**(2026-09-24 · `rct_20260924-0001` · 아래 증분 재판정 ④). · **미판정으로 남는 것**: #121 이 연 `backend/tests/progress.rs` 2행(위 「자매 착지 재실측」(2026-09-22)에 등재된 이관분)과, 머지 직전에 연달아 착지한 **#137**(30행)·**#138**(26행)이 `cross_cutting.rs` 에 들인 합 56행(위 「자매 착지 재실측」(2026-09-24) 두 절에 등재 — 줄 수·지문만 재고정하고 판정은 하지 않았다) — 모두 다음 감지의 몫이다. **해소됨**(2026-09-24 · `rct_20260924-0001` · 아래 증분 재판정 ④). · **증분 재판정 ④**(2026-09-24 · `rct_20260924-0001`): 위 「미판정으로 남는 것」 **두 항목 58행**(#121 `backend/tests/progress.rs` 2행 · #137 30행 · #138 26행)을 판정해 **순 제거 45행** — 판정의 대부분이 **③④**다: 경로 샘플링 doc 본문 8행은 **#138 커밋 제목이 결론을 그대로 적고**(「알파벳 뒤쪽 서비스가 통째로 빠지지 않게」, ④) PR 본문 「## 왜」가 dear-baby 458파일 실측까지 적는다 · `keep_listed_evidence` doc 9행은 #137 「## 추가」 절이 **`acceptance::merge` 비교까지** 적는다 · `index.ts`·`cmd/*/main.go`·이름 다양화 9행은 #138 「## 무엇을」의 축자다 · 정렬 전순서 2행은 **결정성을 강제하는 테스트가 있어** 증분 재판정 ③ 이 1행을 남긴 근거(「강제하는 테스트가 없다」)가 반대로 떨어진다 · 배열 안 절 제목 4행은 이 패스가 걷은 **구분선 25행**과 같은 유형이고 상수 doc 2행은 바로 아래 리터럴 `8`·`4000` 의 곱이다 · `Dropped`·`MAX_PATHS` 의 링크만의 문장 2행 · 모듈 머리 `//!` 는 **LLM 경계 불변식만** 남겨 6행 → 4행 재작성 · `tests/progress.rs` 는 AC 조항 재진술 2행 제거 + 테스트 이름 재진술을 뺀 3행 → 2행 재작성 · **유지 13행**(자격증명 비노출 불변식 2행 · `SKIPPED_DIRS` 의 **기준** 2행 · `MAX_KEY_FILE_DEPTH` 의 왜 1행 · `ENTRY_DIR_HINTS` 1행 · `directory_of` 의 **양쪽 실패 모드** 3행 — PR 본문은 「상위 두 단계」라는 결과만 적는다 · 요약 4행). 199 → **154 / `bd5221f83e91398f103943b598b814c107036cb874cfc947c062855e06230759`**. **이 행에 미판정 증분 없음** — [passes/2026-09-20-pipeline-cross-cutting-axis.md](passes/2026-09-20-pipeline-cross-cutting-axis.md) 「증분 재판정 ④」 |
| `frontend/src/api.ts` · `frontend/src/App.tsx` · `frontend/src/RegisterLlmKey.tsx` · `frontend/src/GrantRepoAccess.tsx` · `frontend/src/HomeRepositories.tsx` · `frontend/src/SignIn.tsx` · `frontend/src/index.css` · `frontend/src/format.ts` (프런트 데이터·셸 축 — `frontend/src` 잔여 전량 8파일) | 93 | `407308c167187925a200ce94eed5243c8e9b8c33b5171bd1eaf32dff3b6a032c` | 순 제거 260행(절 제목 33 + `api.ts` AC 꼬리표 절 제목 5 · 선언·시그니처 재진술 · AC 조항·시나리오·목업 카피 인용 · 작업 흔적(슬라이스 ⑥⑦ · task id · 판정일) · **4벌까지 복제된 인계 계약과 404 명제 중 잉여** — diff 기준 367행 삭제 · 57행 재작성) · 유지 80행 · 판단이 갈려 남긴 것 12건 · **낡아서 거짓이 된 주석 1건 적발**(`SignIn.tsx` 머리가 #83 이 지운 `CredentialsSetup.tsx` 를 현재형으로 서술) · `format.ts` 는 전건 제거로 주석 0행이 되어 지문의 파일 집합에서 빠졌다(8파일 판정 → 지문 `files=7`) · **화면 머리의 목업 매핑 4건**(M1 이 읽는다) 보존 — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) · **증분 재판정 ①**(2026-09-20): #85(제품 fix)가 `HomeRepositories.tsx`(+5) · `index.css`(+1) 에 연 순증 6행을 판정해 **순 제거 6행**(`pick()` JSDoc 4 — 본문 한 줄과 여섯 줄 위 `edit()` 의 재진술 + 목업 `renderHome()` 인용 · `stopPropagation` 인라인 1 · CSS 선택자 재진술 1) · 유지 0행 · 줄 수와 지문이 **둘 다 #85 이전 값으로 되돌아왔다**(80 / `641e9457…` — 부모 `e340bdc` 실측과 바이트 동일) — [passes/2026-09-20-tools-checker-axis.md](passes/2026-09-20-tools-checker-axis.md) · **증분 재판정 ②**(2026-09-21): #91(`3567755`, 반응형 레이아웃)이 `index.css` 의 반응형 블록에 연 순증 5행(물리 9줄)을 판정해 **순 제거 4행**(절 제목 겸 §3.4 축자 재진술 블록 1 — 물리 6줄 · 선택자 재진술 1 · §4.7 축자 1 · §3.4 불릿 축자 1 — 전부 같은 커밋이 신설한 `docs/design-system.md` §3.4·§4.7·§5.4 로 복원) · **유지 1행**(`.screen > .tabbar { animation: none }` 위의 함정 — 진입 애니메이션 `rise` 의 `transform` 이 탭바의 `translateX(-50%)` 를 덮어쓴다; PR #91 본문에도 있으나 「실패 모드의 함정」이라 판단이 갈려 남긴 것 **13건째**) · 유지분이 남아 줄 수·지문은 #91 이전 값(80 / `641e9457…`)으로 **돌아가지 않는다**(81 / `af9fbb9d…`) · 비주석 코드 무접촉(빌드 CSS 산출물 sha256 부모와 동일) — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) 「증분 재판정 — 원장 10행에 #91 이 연 +5행」 · **미판정 증분 없음** · **증분 재판정 ③**(2026-09-21): #92가 `api.ts` 에 더한 5행 · `App.tsx` 에 더한 2행을 판정해 **순 제거 5행**(타입·필드 JSDoc 3 — `doc_edit.rs` 의 `Sentences` 요약·`lines()` doc·0009 가 정본 · `App.tsx` 라우트 필드 2 — 0-based 자리 계약과 「서버가 들고 있는 제안」의 사본) · 유지 2행(`proposeEdit`·`decideEdit` 의 export 함수 JSDoc 요약 1줄) · 줄 수·지문은 88/`4346a888…` → **83/`ec54e678…`** — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) 「증분 재판정 ③」 · **증분 재판정 ④**(2026-09-21): #107 이 `api.ts` 에 더한 7행을 판정해 **순 제거 5행**(타입·필드 JSDoc 5 — 0010 `source`·`key` 문단 · `feature_add.rs` 의 같은 문장(함께 제거) · `AddFeature.tsx` 의 조건 렌더가 정본) · 유지 2행(`draftAddition`·`decideAddition` 의 export 함수 JSDoc 요약 1줄 — ③ 과 같은 모양) · 줄 수·지문은 83/`ec54e678…` → 90/`d88323c6…`(트리거) → **85/`89959a07…`** — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) 「증분 재판정 ④」 · **증분 재판정 ⑤**(2026-09-21): #112 가 `api.ts` 에 더한 2행(`previouslyDeleted` 필드 JSDoc — `feature_delete::previous_deletion` pub doc 이 정본 · `FeatureDeletion` 타입 JSDoc — 필드 `restoreUntil`·`restorable` 과 0011 머리)을 **전건 제거** · 줄 수·지문은 #112 이전 값 85/`89959a07…` 로 바이트 동일 복귀(export 함수 JSDoc 요약은 #112 가 더하지 않았다) — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑤」 · **미판정 증분 없음** · **증분 재판정 ⑥**(2026-09-22): #108 이 `api.ts` 2 · `RegisterLlmKey.tsx` 4 = 6행을 들여와 **제거 3 · 유지 3** — 제거는 `LlmLanguage` 타입 JSDoc(선언이 목록을 그대로 적는다) · `Analysis.llmLanguage` JSDoc(두 명제의 사본) · `LANGUAGES` 위 1행(순서·라벨을 리터럴이 말한다), 유지는 「저장은 키 등록 폼과 독립」 2행과 「`null` until the stored value arrives」 1행 — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑥」 · 맥락 [passes/2026-09-22-output-language-axis.md](passes/2026-09-22-output-language-axis.md) · **증분 재판정 ⑦**(2026-09-22): #119 가 `RegisterLlmKey.tsx` 의 유지 1행을 `button`→`option` 으로 **고쳐 쓰기만** 해 **전건 유지 · 순 제거 0 · 88행 불변**, 지문만 `fd84aea8…` → `7e6915aa…` 로 이동 — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑦」 · **증분 재판정 ⑧**(2026-09-22): #114 가 `api.ts` 에 더한 3행(`DocConflict` 타입 JSDoc · `decideConflict` · `proposeMerge` 의 함수 JSDoc)을 **전건 제거** — 셋 다 **전송 경계의 재진술**이고 정본은 `doc_conflict.rs` 쪽이다(⑥ 의 「한 명제의 일곱 벌」과 같은 잣대) · AC3.5 꼬리표는 ③ · 줄 수·지문이 #114 이전 값 **88 / `7e6915aa…` 로 바이트 동일 복귀** — [passes/2026-09-22-conflict-axis.md](passes/2026-09-22-conflict-axis.md) 「증분」 · **증분 재판정 ⑨**(2026-09-24 · `rct_20260922-0009`): #132 가 `index.css` 에 더한 2행을 판정해 **순 제거 1행** — `.ev` 이관 블록은 네 겹으로 복원된다: 「목업 `.ev` 를 옮긴다」는 ②(목업)와 ③(PR #132 「근거 줄을 목업 `.ev`/`.ename`/`.esrc` 구조로 이관」) · 「목업은 경로를 하나만 그리지만 구현은 근거 경로를 전부 그린다」는 `frontend/src/CrossCuttingConcerns.tsx` 머리의 **유지된 정본**(「an item renders *every* path it cites where the mockup draws one」)의 두 벌째 · 「이름 칸은 최소 42%」는 바로 아래 `.ev .esrc { max-width: 58% }` 의 산술 여집합이라 리터럴 재진술(①) · 「`.ename`·`.esrc` 는 `.ev` 아래로 한정한다」는 같은 파일 위쪽 `.dep` 스코프 주석의 두 벌째이고 그 주석이 이유까지 적는다 · **유지 1행**은 `-webkit-text-size-adjust` 위의 iOS Safari 텍스트 자동 확대 — 상류 브라우저의 **문서화되지 않은 동작**으로, `docs/design-system.md` §5.3 은 선언 두 줄만 옮겨 적어 *이유* 를 복원하지 않고(② 는 반쪽), PR #132 본문은 한 스크린샷의 증상으로만 적는다 ⇒ 정책 본문 「애매하면 남긴다」의 비대칭 비용이 그대로 걸리는 자리라 **판단 갈림 1건**으로 남긴다 · 기재 88(트리거 `27d9b81` 에서 바이트 일치) → 유입 후 90 → 판정 후 **89 / `39baa2ff0e02877a92978a28d415bf7c128f75fc4dfd1fbf396948b781f8c5d6`** — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑨」 · **미판정으로 남는 것**: 열린 PR #126 이 `api.ts`·`App.tsx` 에 들일 주석 — 착지 뒤 다음 감지의 몫이다. · **증분 재판정 ⑩**(2026-09-25, 28차 패스): #126 이 `api.ts` 에 연 5행 중 3행(`carriedFrom` 의 「두 번 세지 않게 한다」 — `doc_history.rs` 같은 문장의 **두 벌째** · `standing` 과 「복원은 재생 구간을 자른다」 — doc-tracker 2026-09 슬라이스 6e 절의 축자(②))을 **제거** · 유지 2행(`HistoryEntry`·`PreviewView` 의 export 요약 1줄씩) — [passes/2026-09-25-history-restore-axis.md](passes/2026-09-25-history-restore-axis.md) 「증분 재판정 ⑩」 · **증분 재판정 ⑪**(2026-09-25, 30차 패스): #152 가 `index.css` 에 연 `.disclosure` 머리 주석(지문 1행 · 물리 3행)을 **제자리 재작성해 물리 2행 제거** — 「긴 결과의 요약 → 상세 단계 노출(AC4.4)」은 PRD AC4.4 축자(②)이고 「Compact 에서는 제목 줄만 남기고 접히고 첫 확장 브레이크포인트부터 펼쳐진다」는 `docs/doc-tracker/2026-09.md` 편차 표 세 행(`AnalysisProgress`·`CrossCuttingConcerns`·`FeatureAcceptance`)의 축자다(②) · **유지 1행**은 「여는 주체는 화면이 넘기는 `open` — CSS 로 펼치면 DOM 상태와 보이는 상태가 갈리므로 표식만 그린다」로, 정책이 유지 대상으로 이름 붙인 「무엇을 넣지 말라」이고 이 파일에 `display` 를 더하는 것이 **조용히 깨지는 유일한 경로**다 · 지문 줄 수는 91 → **92**(줄머리 필터에 continuation 2행이 보이지 않는다 — 본문 「지문과 사각지대」) — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑪」 · **증분 재판정 ⑫**(2026-09-25, 32차 패스): #156 이 `index.css` 의 `@media (min-width: 600px)` 블록 머리에 연 주석(지문 1행 · 물리 4행)을 **제자리 재작성해 물리 2행 제거** — 「목업은 같은 줄을 정적인 `div.section-title` 로 그린다」는 `tools/check-mockup-render.py` M8 의 `fail()` 문면 축자(①)이고 「AC4.4 의 검증 방법도 「데스크톱은 동일 화면을 확장 적용한다」다」는 `docs/prd/04-platform.md:36` 축자(②)인데, **같은 날 같은 문장이 `sc04-06` 에서 「제거 2행」 판정을 이미 받았다**([passes/2026-09-25-residual-pool-closeout.md](passes/2026-09-25-residual-pool-closeout.md) 행 `:110-111`) — ⑪ 이 이 블록의 AC4.4 축자를 지운 자리에 #156 이 다른 문장으로 되살린 것이다 · **유지 1행**은 「이 블록은 위 규칙들보다 **뒤**에 서 있어야 한다 — 앞으로 옮기면 같은 명세도라 조용히 무력해진다」로, ⑪ 의 유지 1행과 같은 「이 순서를 바꾸면 무엇이 조용히 깨지는가」형 가드다(`docs/doc-tracker/2026-09.md:716` ⓓ 가 더 길게 적지만 **그 문서는 편집 지점에서 읽히지 않는다** — 같은 날 `sc04-06 :53` 이 ② 축자임에도 같은 사유로 유지된 선례) · 지문 줄 수는 92 → **93**(#156 의 +1 이 들어온 채 제자리 수정이라 되돌아가지 않는다) · 비주석 CSS 선언 **부모와 바이트 동일**(md5 `e50755cb…`) — [passes/2026-09-20-frontend-shell-axis.md](passes/2026-09-20-frontend-shell-axis.md) 「증분 재판정 ⑫」 |
| `tools/check-mockup-render.py` · `tools/check-journey-mockup.py` · `tools/check-scenario-e2e.py` (`tools/` 체커 축 — 정적 게이트 3파일) | 117 | `47494365e997014b36adec7e5778397d3f451fcf241ee7e09bd87d727fe35eaa` | 순 제거 101행(형제 게이트 셋이 각자 한 벌씩 적은 SSOT·면제 통로 서술 14 · `data-sample`·`data-variant` 규약 전문 22 — 주석 스스로 `docs/mockups/README.md` 를 복원처로 지목하고 그 포인터의 절 제목마저 이미 낡아 있었다 · 인라인 규칙 마커 21(M0~M7 · R0~R11 · S0~S5 — 파일 머리 목록과 `fail()`·`print()` 문면의 세 번째 벌) · 절 제목 13 · 여정 밖 분기 설명의 다섯 벌 중 셋 7 · M7 블록 머리의 두 번째 벌 5 · 의존성 0·`--verbose` 사용법 5 · R6·R10 규칙 재진술 4 — diff 기준 152행 삭제 · 6행 재작성, 차이는 파이썬 docstring 51행이 지문에 안 보이기 때문) · 유지 112행 · 판단이 갈려 남긴 것 17건 · **파일 머리의 규칙 목록 M0~M7·R0~R11·S0~S5 는 원본이라 보존**(워크플로 셋이 「무엇을 검사하는지는 이 헤더에 있다」로 가리킨다 — 2026-09-18 판정이 `check-journey-prototype.js` 의 P1~P7 을 남긴 것과 같은 결정) · 기계 판독 `// 검증 시나리오:` 선언 1건(S1 항목) 보존 · **AST 동일 3/3** — [passes/2026-09-20-tools-checker-axis.md](passes/2026-09-20-tools-checker-axis.md) · **증분 재판정 ①**(2026-09-25, 32차 패스): #156 이 `check-mockup-render.py` 에 연 순 +10행(지문)을 판정해 **순 제거 5행**(지문) · 물리 8행 — 제거는 `main()` 의 **M8 블록 머리 4행**(파일 머리 M8 항목과 바로 아래 `fail()` 문면 「그 폭의 목업은 정적인 `div.section-title` 다」의 **세 번째 벌**(①) · 「권위 순서상 AC4.4 가 위다」류는 `docs/doc-tracker/2026-09.md:716` 이 더 길게(②)·PR #156 본문 「## 왜 목업 쪽으로 수렴시켰나」가 축자로(③) 갖는다 — 이 행이 2026-09-20 에 걷은 「M7 블록 머리의 두 번째 벌 5행」·「인라인 규칙 마커 21행」과 **같은 유형·같은 파일**이다) · 「공전 방지」 1행(바로 아래 조건식 `folds == 0 or disclosure_rules == 0` 과 `fail()` 문면 「규칙이 공전한다」가 그 문장 자체(①) · doc-tracker:716 ⓑ 「모집단 0 을 초록으로 통과시키지 않는다」(②)) · **지문 밖 docstring 2행** — `wide_breakpoint` 1행(이름과 본문 두 줄 `SRC_DIR / "viewport.ts"`·`WIDE_QUERY` 정규식이 그 문장이다 ①) · `css_declarations` 의 요약 1행(시그니처·타입 애너테이션 재진술 ① — 2026-09-20 판정 ⑦ 과 같은 잣대) · **유지 6행**(지문) + docstring 3행 — 파일 머리 **M8 항목 4행**은 이 행이 「파일 머리의 규칙 목록은 **원본이라 보존**」으로 명시 판정한 목록의 n번째 항목이고 · 「이 게이트가 보지 않는 것」 절의 **제자리 수정 1행**(「M7 하나뿐」 → 「M7 과 M8 둘뿐」)은 유지 행의 거짓이 된 전제를 같은 칸에서 갱신한 **준수**이며 · 같은 절의 **신규 1행**(「M8 도 같은 자리다 — 표식 `▴` 가 목업에 없는 폭에 그려져도 카피는 한 글자도 줄지 않는다」)은 바로 위 2026-09-18 M7 짝이 유지된 선례와 같은 모양의 「이 게이트를 M3 와 겹친다고 보고 지우지 말라」 가드다 · `css_declarations` docstring 의 **순서 가드 3행**은 파서를 순서 무관하게 「고치면」 M8 이 조용히 눈머는 자리이고 `index.css` 쪽 가드와 **편집 지점이 다르다** · **docstring 제거 정규화 AST 부모와 동일**(`83e329be75454743cae4043491fe7205`) · 게이트 3종 출력 **부모와 바이트 동일** · 줄 수·지문 112 → **117 / `47494365…`** — [passes/2026-09-20-tools-checker-axis.md](passes/2026-09-20-tools-checker-axis.md) 「증분 재판정 ①」 |
| `backend/src/github_app.rs` · `backend/src/github.rs` · `backend/src/github_api.rs` · `backend/src/github_tokens.rs` · `backend/src/auth.rs` · `backend/src/session.rs` · `backend/src/cookies.rs` · `backend/src/installations.rs` · `backend/src/users.rs` · `backend/tests/github.rs` · `backend/tests/auth.rs` · `e2e/tests/sc04-01-app-install-and-scope.spec.ts` · `e2e/tests/sc04-11-unauthenticated-block-and-signin.spec.ts` · `e2e/tests/sc04-12-logout-session-invalidation.spec.ts` (GitHub App · 인증 경계 축 14파일) | 78 | `97ae560b9ae8673b2aa747c947c71aee8c075392de5a0394a2cdbec299f7f273` | 순 제거 152행(선언·시그니처만 영어로 풀어 쓴 `///` 요약 19건 · AC 꼬리표 `(AC4.1)`·`(AC4.3)`·`(AC4.7)`·`(AC4.8)` 전건 · 시나리오 원문 축자 인용과 본문 절 제목 (AC4.8 검증 방법의 복사) · 작업 흔적(분리 이력 · 정정 이력 · `rct_20260916-0002`) · **네 벌까지 복제된 명제 중 잉여** — Setup URL `installation_id` 스푸핑 4벌 · OAuth 토큰 보관 사유 3벌 · 미리보기 state 태깅 3벌 · adoption best-effort 2벌) · 유지 71행 · 판단이 갈려 남긴 것 2건 · **낡아서 거짓이 된 주석 2건 적발**(`github.rs` 의 「distinct users → distinct ids」 — 식은 `rem_euclid(90_000)` 이라 충돌한다 / `tests/auth.rs` 머리가 테스트 넷을 열거하는데 파일에는 여섯 개다) · 기계 판독 `// 검증 시나리오:` 3건 · `mock-exception:` 6건 보존 · **주석 제거 후 부모와 바이트 동일 14/14** — [passes/2026-09-20-github-app-auth-axis.md](passes/2026-09-20-github-app-auth-axis.md) · **증분 재판정 ⑫(25차 패스)**: #141 이 연 +14행(85행)을 판정해 **순 제거 7행**(정책 규칙 재진술 2 — 주석 스스로 `docs/e2e-mocking-policy.md, 충실도 보증` 을 인용한다 · 테스트 이름이 그대로 말하는 `///` 3 · 절 제목 1 · 바로 아래 두 줄이 말하는 절 1) · 유지 78행 · **판단이 갈려 남긴 것 2건**(`stub_installation_id` 의 배치 사유 · `real 이라면 … Forbidden` 의 충실도 경계) — [같은 파일의 「증분 재판정 ⑫」 절](passes/2026-09-20-github-app-auth-axis.md) |
| `backend/tests/llmkey.rs` · `e2e/tests/sc04-13-unsupported-provider-rejection.spec.ts` · `e2e/tests/sc04-04-revoked-key-blocks-calls.spec.ts` · `e2e/tests/sc04-03-llm-key-registration.spec.ts` · `e2e/tests/sc04-05-credential-log-exposure.spec.ts` · `backend/tests/security.rs` · `backend/src/audit.rs` · `backend/tests/crypto.rs` (자격증명 · LLM 키 경계 축 8파일 — `backend/src/crypto.rs` 는 **D2 사람 게이트 풀**로 보류) | 42 | `1823c6f17a5f8e68f171dabd6ef88646211b863657172e006ed1b4db999bfa8a` | 순 제거 104행(단정·선언 재진술 — 주석 바로 아래 한두 줄이 그 문장 자체인 것 · 시그니처만 영어로 풀어 쓴 `///` 1건 · **테스트 이름을 다시 쓴 파일 머리 `//!` 3건** · 시나리오 원문 축자 인용 2건(시나리오 4·13) · **「자동화 밖 잔여」 4건 전건**(정본은 doc-tracker 「e2e 매핑」의 마지막 열이고 주석 스스로 그렇게 적는다) · AC 꼬리표 전건 · 작업 흔적(분리 이력 `rct_20260916-0002` 3벌 · 슬라이스 ⑦ 3건 · 목업 식별자 인용) · **4벌까지 복제된 명제 중 잉여** — 「App 연결이 선행돼야 한다」 4벌 중 3벌 제거하고 `sc04-03` 을 정본으로 · backdate 사유 2벌 중 뒤의 벌 압축 — diff 기준 125행 삭제 · 18행 재작성, 차이는 불변식·함정 주석을 지우지 않고 되풀이된 절반만 걷어 다시 썼기 때문) · 유지 42행 · 판단이 갈려 남긴 것 2건 · **낡아서 거짓이 된 주석 1건 적발**(`tests/crypto.rs` 머리가 테스트를 셋 열거하는데 파일에는 다섯이다 — `cargo test --test crypto` 5 passed 로 실측; 12차 패스의 `tests/auth.rs` 와 같은 유형의 두 번째 사례) · 기계 판독 `// 검증 시나리오:` 4건 보존(spec 당 정확히 1개) · `backend/tests/crypto.rs` 는 전건 제거로 주석 0행이 되어 지문의 파일 집합에서 빠졌다(8파일 판정 → 지문 `files=7`) · **주석 제거 후 부모와 바이트 동일 8/8** · **필수 status 판정기 `✅ 변경 없음`**(9파일 트리는 D2 로 `⚠️` 였다 — `crypto.rs` 13행은 판정만 마치고 잔여에 남긴다) · `cargo test` 16 passed · 문서 게이트 3종 rc=0 — [passes/2026-09-20-credential-llm-key-axis.md](passes/2026-09-20-credential-llm-key-axis.md) · **지문 정정**(2026-09-22): 기록돼 있던 `96636e79…` 는 **등재 커밋 `7415771`(#95) 에서부터 한 번도 재현되지 않았다**(그 시점에도 현재 값 `1823c6f1…`). 등재 시점의 계산 오류이며 — 16행의 「#98 은 리베이스 전 계산」과 같은 부류 — 줄 수 42 와 판정 결과는 처음부터 맞았다. **판정 결과 무수정, 핀만 정정** · **사유 정정**(2026-09-24 · `rct_20260924-0003`): 「계산 오류」가 아니다. `96636e79…` 는 이 파일 「행 지문을 재현하는 법」이 구분해 둔 **전역 규약**(`printf '%s'` — 개행 없음)으로 계산한 값이고, 등재 커밋 `7415771` 에서 **바이트 일치로 재현된다**(같은 줄 집합 42행 — 행 규약 `1823c6f1…` / 전역 규약 `96636e79…`). 줄 집합도 값도 틀리지 않았고 **규약만 섞였다.** 현재 핀(행 규약)은 그대로 둔다 — 정정되는 것은 **사유뿐**이다.** |
| `backend/tests/worker.rs` · `backend/tests/common/mod.rs` · `backend/tests/analyses.rs` · `scripts/e2e.sh` · `e2e/tests/sc02-02-acceptance-from-tests.spec.ts` (테스트 하네스 축 5파일) | 21 | `d4414d5fc3a243b00181ac2ad4c75e7408e31491730fc5a86723881f09d35c97` | 순 제거 68행(테스트 이름이 그대로 말하는 `///` 10 · 절 제목 7 · 인라인 단정 재진술 5 · 시그니처·이름 재진술 5 · **테스트 이름을 열거한 파일 머리 `//!` 2건 11행**(12·13차가 같은 유형의 낡은 주석을 둘 적발했다) · 시나리오 2 기대 결과 **축자 인용 2벌 4행**(`검증 시나리오:` 마커가 이미 가리킨다) · AC2.2 제목 축자 · `sc02-03` 교차 참조 · spec 11개 열거 · 슬라이스 번호 · **워커 임대 문단 4행 → 1행**(정본은 `e2e/support/cluster.ts`)) · 유지 20행 · 판단이 갈려 남긴 것 **0건** · 기계 판독 `// 검증 시나리오:` 1건 보존 · `backend/tests/analyses.rs` 는 전건 제거로 주석 0행이 되어 지문의 파일 집합에서 빠졌다(5파일 판정 → 지문 `files=4`) · **주석 제거 후 부모와 동일 5/5** · `cargo test --tests` 159 passed(3회 연속) · 문서 게이트 3종 rc=0 · **판정기 `✅ 변경 없음`** · **원장 정정: 후보 ① `tools/check-data-format-change.py` 는 D6 로, `deploy/k8s/pvc.yaml` 은 D5 로 경로 매칭돼 무인 머지 경로가 없다**(음성 대조 실측) — [passes/2026-09-20-test-harness-axis.md](passes/2026-09-20-test-harness-axis.md) · **지문 정정**(2026-09-22): 기록돼 있던 `8b8764df…` 는 **등재 커밋 `445ec57`(#96) 에서부터 한 번도 재현되지 않았다**(그 시점에도 현재 값 `e8a9deb5…`). 13행과 같은 등재 시점 계산 오류다. 줄 수 20 과 판정 결과는 처음부터 맞았다. **판정 결과 무수정, 핀만 정정.** · **사유 정정**(2026-09-24 · `rct_20260924-0003`): 13행과 「같은 부류」가 맞지만, 그 부류는 계산 오류가 아니라 **규약 혼동**이다. `8b8764df…` 는 이 파일 「행 지문을 재현하는 법」이 명시한 **전역 규약**(`printf '%s'`)으로 계산한 값이고, 등재 커밋 `445ec57` 에서 **바이트 일치로 재현된다**(같은 줄 집합 20행 — 행 규약 `e8a9deb5…` / 전역 규약 `8b8764df…`). 당시 패스 문서가 적은 값이 그것이다([passes/2026-09-20-test-harness-axis.md](passes/2026-09-20-test-harness-axis.md) 「집계」). **재현이 안 맞을 때 이 파일이 먼저 의심하라고 적어 둔 개행을, 그 정정 자신이 의심하지 않았다.** 현재 핀(행 규약)은 그대로 둔다 — 정정되는 것은 **사유뿐**이다. · **증분 재판정 ①**(2026-09-24 · `rct_20260924-0003`): #142(`82be2f4`, e2e 이미지 재사용 · 브라우저 캐시)가 `scripts/e2e.sh` 에 더한 1행(「CI 는 e2e 의존성과 Playwright 브라우저를 앞 단계에서 (캐시와 함께) 설치해 둔다.」)을 **전건 제거** — 복원 경로 셋이 동시에 서 있다: ① `.github/workflows/ci.yml` 138~161행이 브라우저 캐시 복원 스텝과 `SKIP_PLAYWRIGHT_INSTALL: '1'` 을 그대로 적고 그 파일 자신의 137행 주석이 같은 문장을 담는다 · ② `README.md` 179행이 **같은 커밋에서** 「Playwright chromium(main 이 저장한 브라우저 캐시를 복원, 적중 시 apt 의존성만 설치) → `SKIP_BUILD=1 SKIP_PLAYWRIGHT_INSTALL=1 scripts/e2e.sh`」로 갱신됐다 · ③ PR #142 본문 §4 가 축자로 서술한다. 플래그가 *무엇을* 하는지는 바로 아래 선언과 `if [ "${SKIP_PLAYWRIGHT_INSTALL}" != "1" ]` 블록이 이미 말한다(유형 ① 선언 재진술 + ② 문서 재진술). 같은 파일 60·61행의 `kubectl rollout` 조기 반환 주석은 「실패 모드의 함정」이라 유지가 맞고, 이번 줄은 그 성격이 없어 「애매하면 남긴다」의 보호 대상이 아니다 · **줄 수·지문은 칸 무변경** — 제거하면 부모 `6e6478a` 값으로 **바이트 동일 복귀**한다(21/`5f016e49…` → 20/`e8a9deb5…`, 전역 규약도 `8b8764df…` 로 동시 복귀). 재고정이 아니라 **원복**이라 핀 칸을 손대지 않았다 — [passes/2026-09-20-test-harness-axis.md](passes/2026-09-20-test-harness-axis.md) 「증분 재판정 ①」 · **증분 재판정 ②**(2026-09-25 · `rct_20260925-0009`): #155(`9c01489`, 슬라이스 7b — AC4.7 분석 작업의 격리)가 `backend/tests/worker.rs` 에 더한 **22행**(gross == net — 이 창에서 제거 0행)을 **명제 단위로** 판정해 **순 제거 21행 · 유지 1행** — ⑴ 비공개 헬퍼 doc 4행(`login_installed_with` 2 는 시그니처의 `installation_id: i64` 와 유일 호출부의 `11_001`·`11_002` 가 축자이고(①) #155 본문 §1 이 「위임만 바꿔 호출부 동작이 불변」을 적는다(③) · `register_key` 2 는 이름·본문의 `POST /api/llm-keys`·단정 메시지 `register {provider} key` 가 그 문장 자체이며(①) 자매 헬퍼 `set_language` 는 애초에 무주석이다) — **비공개 fn 이라 「`pub` 항목 요약 1줄 유지」 조항 대상이 아니다**(행 4 증분 재판정 ④ 선례) · ⑵ 새 테스트의 `///` **12행 전건 제거**(AC 꼬리표 + `04-platform.md#시나리오 10` 기대 결과 두 번째 문장 축자 2행(②)과 fn 이름 `one_worker_claiming_two_users_jobs_never_mixes_their_context`(①) · 「이 단정이 왜 claim 층에 있나」 6행은 **`docs/doc-tracker/2026-09.md` 매핑 행의 「자동화 밖 잔여」 칸과 변경 이력 행이 축자에 가깝게 소유**하고(②) #155 본문 §1 의 *「워커 프로세스를 두 번 돌리지 않는 이유」* 절이 다시 적으며(③), 「워커는 영속을 하나도 소유하지 않는다」는 **주석 스스로 `worker_api` 모듈 주석을 정본으로 지목**하고 실측으로 그 자리에 있다(「So the worker owns *no* persistence」) — 「왜 이렇게 만들었나」는 정책 본문이 ③ 의 자리로 둔 **경위**다 · 관측 가능하게 만드는 두 값 2행은 doc-tracker 두 자리가 `ghs_stub_<installation_id>_…` 를 축자로 적고(②) `a_token.starts_with("ghs_stub_11001_")` 와 그 FAIL 문면이 그 말 자체다(①) · 딸린 빈 `///` 2행) · ⑶ 인라인 5행 제거(언어 스냅숏 1 — 바로 아래 `set_language(&state, &bob, "en")` 가 bob 만이고(①) 스냅숏 계약의 **정본은 행 1 증분 재판정 ⑨ 가 지정한 `analysis.rs` 의 복사 지점**이다 · 「같은 worker id 로 두 번 · job id 로 되찾는다」 1 — `claim(&state, "w1")` 두 번과 `by_id.get(&alice_job)` 가 축자 · 절 제목 ⑴⑶⑷ 3 — 네 단정의 FAIL 문면 「alice 의 job 에 alice 의 키」·「alice 의 설치로 발급되지 않았다」·「bob 이 고른 언어」/「alice 는 고른 적이 없다」가 그 문장 자체) · **유지 1행**(⑵ 의 「값 비교만으로는 "둘 다 실렸다"를 못 잡는다」 — ②③ 히트는 있으나 이 두 `assert!` 를 빼면 위 등식 단정만 남아 테스트가 **조용히 약해지는** 편집 지점 가드다(정책 본문 「애매하면 남긴다」의 비용 비대칭) · 절 제목만 떼고 **제자리 재작성**) · **주석 제거 후 코드 md5 부모와 동일 1/1** · 비주석 diff **0줄** · `assert` 62 == 62 · `;` 168 == 168 · 20 → 42(#155 착지) → **21 / `d4414d5fc3a243b00181ac2ad4c75e7408e31491730fc5a86723881f09d35c97`** — [passes/2026-09-20-test-harness-axis.md](passes/2026-09-20-test-harness-axis.md) 「증분 재판정 ②」 |
| `deploy/k8s/kustomization.yaml` · `backend/src/util.rs` · `backend/src/main.rs` · `backend/src/error.rs` · `backend/src/state.rs` (API 셸 · 배포 베이스 축 5파일 — 잔여의 무인 자유 풀 전량) | 19 | `3b0735c725531481568605ca1a47b0573a706afce46b723edfcc66ab382db4e7` | 순 제거 19행(`kustomization.yaml` 머리 **15행 전건** — `README.md` §배포·§CI 가 문장 단위로 되풀이하는 것(핀 파이프라인 6 · 두 워크로드 3 · secret 외부 제공 4)과 빈 주석 행 2 · 이름·시그니처를 영어로 옮긴 `///` 3행(`util.rs`) · `error.rs` 모듈 머리의 **동작 서술 3행 → 불변식 2행 재작성**) · 유지 19행(PID 1 시그널 함정 5 · OAuth `state` 접두사의 CSRF 불변식 8 · 모듈 머리 셋 4 · `error.rs` 불변식 2) · 판단이 갈려 남긴 것 **1건**(`main.rs` 의 PID 1 함정은 `bin/worker.rs:118-122` 와 같은 명제의 두 번째 벌 — **4차 패스가 그 벌을 「유지」로 닫았고**, 복제 정리는 복원 경로 넷에 없는 재량이며 두 바이너리는 독립 표면이라 전건 유지. 뒤집으려면 `worker.rs` 를 포함한 증분 재판정으로 한 번에) · `deploy/k8s/kustomization.yaml` 은 전건 제거로 주석 0행이 되어 지문의 파일 집합에서 빠졌다(5파일 판정 → 지문 `files=4`) · **주석 제거 후 부모와 바이트 동일 3/3** · 비주석 diff 0줄 · `cargo test --tests` 159 passed · 문서 게이트 4종 rc=0(`check-journey-mockup` · `check-mockup-render` · `check-scenario-e2e` · `check-journey-prototype`) · 허브 `Documents` 40 → 41 · **판정기 `✅ 변경 없음`** · **이 패스 뒤 무인 자유 풀은 0** — 남는 242행은 전부 사람 게이트 3몫이다(단 #92·#93 이 머지되면 새 주석이 들어와 다시 열린다) — [passes/2026-09-20-api-shell-deploy-base-axis.md](passes/2026-09-20-api-shell-deploy-base-axis.md) |
| `backend/migrations/0001_init.sql` · `0002_github_tokens.sql` · `0003_analyses.sql` · `0004_analysis_stages.sql` · `0005_analysis_documents.sql` · `0006_discovery_strategies.sql` · `0007_feature_candidates.sql` · `0008_feature_dependencies.sql` (적용된 마이그레이션 축 8파일 — **사람 게이트**) | 123 | `1b35c33a821701c748cc127e43169b26f3b1b7f586c29f75e2fa7b1f0d3fe878` | 순 제거 16행(**절 제목 7행 + 딸린 빈 주석 행 5행** — `0001`·`0004`(2)·`0005`·`0006`·`0007`·`0008` 의 머리 라벨은 바로 아래 `CREATE TABLE`·`ALTER TABLE` 이 그대로 말한다 · **선언 재진술 3행** — `0003` 의 소유·범위 문장(`0001` 머리의 두 벌째) · `0004` 「Lease + lifecycle columns.」 · `0007` 의 `decision` 열거와 `reject_reason` NOT NULL(두 `CHECK` 이 글자 그대로 말한다) · **금지 참조 1행** — `0008` 머리의 AC 문면 인용) · 유지 123행 · **금지 참조 9곳을 문단 재작성으로 걷어냄**(파일·모듈 경로 `db.rs`·`pipeline::STAGES`·`worker_api::claim`·`feature_candidates::candidate_key`·`backend/src/dependencies.rs` · 문서 식별자 `AC2.4`·`AC2.5`·`AC4.6`·여정 `JRN-review-feature` · 시점 서술 「아직」) — `backend/migrations/README.md` §「주석에 무엇을 쓰나」가 이 범위에만 거는 금지 목록이고, 그 문서가 `S04` 폐지와 `AC4.1` 의미 drift 를 자기 이력으로 들고 있다 · `0002_github_tokens.sql` 은 **한 글자도 안 바꿨다**(전건 유지 — 상류 API 의 스푸핑 함정) · **SQL문 변경 0 — 8파일 전건 「주석·빈 줄 걷어낸 나머지 바이트 동일」** · 전역 지문 `lines=2310 files=108` → `lines=2295 files=108`(−15 = `.sql` −16 · `backend/tests/migrations.rs` 출처 주석 +1, 파일 수 불변 — 전건 제거된 파일 없음) · 판단이 갈려 남긴 것 **2건**(`0003` 「display only, never a hard cap」 의 금지는 코드에 없다 · `0006`·`0007` 의 단계 번호는 README 금지 목록의 「문서에 사는 식별자」가 아니라 유지) · **판정기 `⚠️ 사람 리뷰 필요`(D1 7건 · D2 1건) — 이 축의 기대값이다.** 무인 머지 경로가 없어 운영 DB `_sqlx_migrations` 의 SHA-384 7건을 **사람이 손으로 repair 한 뒤에만** 머지한다(repair 표·절차는 패스 상세) · `backend/tests/migrations.rs` 는 `APPLIED` 고정값 7개와 출처 주석만 갱신했고 **그 파일의 주석 13행은 판정하지 않았다**(D2 몫으로 잔여에 남는다) — [passes/2026-09-20-migrations-axis.md](passes/2026-09-20-migrations-axis.md) |
| `backend/src/doc_edit.rs` · `backend/tests/doc_edit.rs` · `e2e/tests/sc03-01-llm-assisted-edit.spec.ts` · `e2e/tests/sc03-02-rejected-suggestion-avoided.spec.ts` · `frontend/src/DecideDiff.tsx` · `frontend/src/RequestEdit.tsx` (문서 편집 축 — 슬라이스 6a #92 가 들여온 새 파일 6개) | 59 | `d0a019a9654ce08249f6d7b03a831aeadd0f4167dab23b090151d075e5b6bec3` | 순 제거 87행(`doc_edit.rs` 모듈 머리 15 — doc-tracker 슬라이스 6a 행·PR #92 「설계 판단 셋」·0009 머리의 **네 벌째** · 필드·fn doc 의 선언 재진술과 AC 꼬리표 22 · `overlay` doc 의 AC3.5 재진술 4 · 스텁·테스트 doc 10 · `tests/doc_edit.rs` 13 — 머리는 `dependencies.rs` 정본의 사본, 나머지는 fn 이름·단정 메시지 · 두 spec 19 — Isolation 4행 → 1행 ×2(13차 규약) · 단정 옆 주석 13 · `DecideDiff.tsx` 4 — 편차 원장 행 축자 · 「사유 없는 거부」 **여섯 벌** 중 정본 둘(0009 CHECK 문단 · `decide()` 오류 문자열)만 남김 — diff 기준 103행 삭제 · 3행 재작성) · 유지 56행(`lines()` 의 칸 단위/시나리오 단위 이유 · 「제안도 주소를 가진다」 · `stub_edit` 의 회피 규칙과 `ADD_HINT` 충실도 경계 · `overlay` 의 「마지막 승인이 이긴다」 · 「회피의 단위는 마지막 문장」 · 두 spec 머리의 판정 이유 문단 · 두 화면의 불변식 머리 · 기계 판독 M1 매핑 2 · `mock-exception:` 1) · 판단이 갈려 남긴 것 3건(`MAX_AVOIDED` 의 목적 · `sc03-01` 의 `── 탭 N ──` 3행 · `RequestEdit.tsx` 의 「키를 쓰는 호출」 사유) · **주석 제거 후 부모와 바이트 동일 6/6**(증분 5파일 포함 10/10) · 문서 게이트 4종 rc=0 · `npm run build` rc=0 · **판정기 `✅ 변경 없음`**(10/10) · 허브 `Documents` 41 → 42 · `0009_feature_doc_edits.sql` 30행은 사람 게이트라 **집지 않았다** (— **해소됨(2026-09-25, 27차 패스, `rct_20260925-0003`)**: 새 행 25 로 판정 — 유지) — [passes/2026-09-21-doc-edit-axis.md](passes/2026-09-21-doc-edit-axis.md) · **증분 재판정 ①**(2026-09-21): #107 이 `doc_edit.rs` `document_of` 의 인라인 주석에 덧붙인 1행(「사람이 직접 더한 feature(AC3.2)까지 포함해서」 — 바로 아래 `feature_add::overlay` 호출과 그 doc 이 정본)을 **제거** · 줄 수·지문은 #107 이전 값 56/`b9e6778d…` 으로 바이트 동일 복귀 — [passes/2026-09-21-doc-edit-axis.md](passes/2026-09-21-doc-edit-axis.md) 「증분 재판정 ①」 · **증분 재판정 ②**(2026-09-22): #108 이 `doc_edit.rs::propose` 에 더한 1행(「고친 문장은 문서의 나머지와 같은 언어 — 사용자의 지금 설정이 아니라 분석의 언어」 — 호출하는 `settings::analysis_language` 의 이름과 그 doc 이 정본)을 **제거** · 줄 수·지문은 #108 이전 값 56/`b9e6778d…` 으로 바이트 동일 복귀 — [passes/2026-09-21-doc-edit-axis.md](passes/2026-09-21-doc-edit-axis.md) 「증분 재판정 ②」 · 맥락 [passes/2026-09-22-output-language-axis.md](passes/2026-09-22-output-language-axis.md) · **증분 재판정 ③**(2026-09-25, 28차 패스): #126 이 `doc_edit.rs` 에 연 9행(순증 7) 중 5행(`SOURCE_AUTO` doc 의 AC3.4 어휘 재진술 · `propose`·`overlay` 의 0014 세대 서술 2 — PR #126 본문 「순서는 시각이 아니라 세대로 엮는다」의 축자(③) · `splice`·`base_document` doc 의 둘째 줄 2 — doc-tracker 슬라이스 6e 절(②))을 **제거** · 유지 4행 — [passes/2026-09-25-history-restore-axis.md](passes/2026-09-25-history-restore-axis.md) 「증분 재판정 ③」 · **증분 재판정 ④**(2026-09-25, 30차 패스): #152 가 `sc03-01` 「탭 3」 뒤에 연 3행을 판정해 **순 제거 2 · 유지 1(재작성)** — 「390px 에서는 AC4.4 에 따라 인수 시나리오 절이 접힌 채 서므로 목록을 보려면 요약을 한 번 펼쳐야 한다」는 AC4.4 재진술(②)이고 바로 아래 `scenarios-disclosure` → `summary` 클릭이 그 문장 자체(①) · 「세는 탭 셋(`request-edit`·`send-request`·`approve-diff`)은 위에서 이미 끝났다」는 위 세 `── 탭 N ──` 표식과 같은 testid 열거라 ① · **유지 1행**은 파일 머리의 유지된 정본(「결과 확인만 저장된 문서를 읽는다」)과 승인 **뒤에** 버튼을 누르는 이 블록을 화해시키는 「탭 계수 밖」 한 문장이다 — 그것이 없으면 다음 편집자가 이 클릭을 네 번째 탭으로 세거나 「탭 3」 위로 올려 단정의 의미를 조용히 깬다 — [passes/2026-09-21-doc-edit-axis.md](passes/2026-09-21-doc-edit-axis.md) 「증분 재판정 ④」 |
| `backend/migrations/0014_feature_doc_restores.sql` (D1 적용된 마이그레이션 — 슬라이스 6e #126 이 들여온 새 파일, 27차 패스가 `0009`~`0013` 을 닫은 뒤 **구조적으로 재충전된 한 칸**) | 33 | `9ebe4ef258cf1d20be70401346d566575a0ddeafb176eb1332ef4f7d368e0ddf` | 순 제거 **0행** · 유지 **18행** · 제거 후보 **9행**(이월) — 본문 「적용된 마이그레이션」 절의 기준(선언 재진술은 제거 후보 · 의도·제약 서술은 커밋 메시지와 겹쳐도 애매하면 유지)을 줄 단위로 적용했다. **유지**: `:6-7` 이 표의 한 행이 곧 재생을 자르는 지점이라는 불변식 · `:9-12` 보상 편집을 택하지 않은 이유(「무엇을 넣지 말라」형 가드 — #126 본문과 축자에 가깝지만 **PR 본문은 편집 지점에서 읽히지 않는다**) · `:14-17` 도출값을 따로 저장하지 않는 이유(0009`:11-17` 과 **두 벌** — 27차 패스의 `content_hash` 두 벌 유지 판정을 승계, 비용 비대칭) · `:25-32` `seq`·`after_restore` 가 시각이 아니라 순번·세대로 잇는 이유(「이 순서를 바꾸면 무엇이 조용히 깨지는가」 — 「복원 뒤에 한 편집이 조용히 사라진다」) · `:47` 인덱스 축(애매 → 남긴다). **제거 후보**: `:1` 표 이름 재진술 + AC 꼬리표 · `:3-5` 겹쳐 읽기 구조(① `doc_history::standing_edits`·`doc_edit::overlay` 와 네 마이그레이션 머리) · `:19-23` `target_kind`/`target_id` 어휘(① `:38`·`:43`·`:44` CHECK 셋과 `doc_history::kind` 상수 · ② 여정 `JRN-restore-history` 축자). **`.sql` 무접촉이라 D1 이 발화하지 않는다** — 판정기가 이 PR 의 실제 diff 에 `✅ 해당 없음` 을 내고 `review/manual-approval` 이 자동으로 붙는다(repair 창 없음). 제거 집행은 27차 패스가 등재한 `0009`~`0013` 의 제거 후보 3자리(기대 6~8행)와 **한 repair 창에 묶는다**(본문 지시) — 합산 기대 순 제거 15~19행 — [passes/2026-09-25-residual-pool-closeout.md](passes/2026-09-25-residual-pool-closeout.md) |
| `tools/check-data-format-change.py` (D6 판정기 자신 1파일 — #64 가 2026-09-20 들여온 뒤 **어느 판정도 받은 적 없던 마지막 잔여**) | 36 | `9b5b5b79d16129ff3bce26d5b823034e6c39e9e21a3a9d5405e530b788d19656` | 순 제거 **0행** · 유지 **19행** · 제거 후보 **12행**(이월) — 짝인 `.github/workflows/data-format-review.yml` 은 판정 범위 밖이지만 **복원 경로로는 유효**하고(이 스크립트를 고치는 손이 반드시 함께 여는 파일), 그 헤더가 「규칙(D1·D6)과 의도적으로 보지 않는 것은 이 헤더에 있다」로 **이 자리를 정본으로 지정**한다. **유지**: `:13-19` 규칙 D1·D6 의 *왜*(sqlx 체크섬 · `pull_request_target` 이라 PR 이 규칙을 고쳐 자기를 통과시킬 수 없다 — `SELF_PATHS` 에서 자신을 빼는 변경을 막는 유일한 문장) · `:21-31` 의도적으로 보지 않는 다섯 가지(#125 「트레이드오프」 절과 겹치지만 정본 지정을 받은 자리이고 「이 초록을 무엇의 보증으로 읽으면 안 되는가」를 말하는 가드) · `:77` `--no-renames` 의 효과(복원 경로 넷 어디에도 없다). **제거 후보**: `:2` 한 줄 요약(① 파일·워크플로 이름) · `:4-9` status 의 뜻과 종료 코드(① 워크플로 헤더 `:3-7` 이 같은 문단을 소유 — status 를 실제로 붙이는 쪽이 정본 · `:80` `return 2` / `:107` `return 0`) · `:11` 의존성 0(① import 5줄 전부 stdlib) · `:33-36` 사용법·CI 출력(① argparse `:68-71` · `:101-106`). ③ 모집단은 **저작 PR 만** 세었다(#64·#125·#127) — 판정 패스 PR 의 인용은 ③ 가 아니다(세면 정책이 지키라는 주석이 전부 제거 대상이 되는 자기 무효화 고리). **`SELF_PATHS` 무접촉이라 D6 이 발화하지 않는다** — 제거 집행은 필수 status 가 아예 붙지 않는 전용 PR 로 분리하고, `:21-31` 을 남기므로 워크플로의 정본 지정 포인터는 그대로 유효하다 — [passes/2026-09-25-residual-pool-closeout.md](passes/2026-09-25-residual-pool-closeout.md) |
| `backend/src/feature_add.rs` · `backend/tests/feature_add.rs` · `e2e/tests/sc03-03-manual-feature-add-with-evidence.spec.ts` · `e2e/tests/sc03-04-manual-feature-add-no-evidence.spec.ts` · `frontend/src/AddFeature.tsx` (빠진 feature 직접 추가 축 — 슬라이스 6b 새 파일 5개) | 45 | `b596e7392db0110e03721163ee6946ca87d58b58aba894f6a556f9ec481294a7` | 순 제거 106행(유입 151 — `feature_add.rs` 모듈 머리 21 → 1 · 필드·fn doc 과 `grounded`·스텁 규칙 서술 · 두 spec 의 머리 문단(doc-tracker 매핑 행 축자)과 단정 옆 주석 16 · Isolation 4 → 1 · 테스트 doc — diff 기준 106행 삭제 · 7행 재작성) · 유지 45행(`routes()` 「초안도 주소를 가진다」 정본 · `overlay` 의 순서 계약 · `status` CHECK 어휘 · 상한의 이유 · 「이 사용자의 키로 부른다」 · 「한 트랜잭션」 · `stub_matches`·`stub_draft` 의 충실도 경계 · pub 요약 · `AddFeature` 의 화면 불변식 머리와 M3B 함정 · 기계 판독 M1 매핑 1 · `sc03-03` 의 `.legend` text-transform 함정 · `mock-exception:` 1) · 판단이 갈려 남긴 것 3건(테스트 doc 「의존성만으로 feature 를 세우지 않는다」 · 픽스처 상수 doc 4행 · `AddFeature.tsx` 의 「키를 쓰는 호출」 사유 — 승계) · **주석 제거 후 부모와 바이트 동일 5/5**(증분 6파일 포함 11/11) · 문서 게이트 4종 rc=0 · `npm run build` rc=0 · **판정기 `✅ 변경 없음`**(11/11) · 허브 `Documents` 43 → 44 · `0010_feature_additions.sql` 30행은 사람 게이트라 **집지 않았다** (— **해소됨(2026-09-25, 27차 패스, `rct_20260925-0003`)**: 새 행 25 로 판정 — 유지) — [passes/2026-09-21-feature-add-axis.md](passes/2026-09-21-feature-add-axis.md) · **증분 재판정 ①**(2026-09-22): #108 이 `feature_add.rs::draft` 에 더한 1행(「초안은 이 분석의 목록에 들어가므로 목록의 나머지와 같은 언어로」 — 호출하는 `settings::analysis_language` 의 이름과 그 doc 이 정본)을 **제거** · 줄 수·지문은 #108 이전 값 45/`b596e739…` 로 바이트 동일 복귀 — [passes/2026-09-21-feature-add-axis.md](passes/2026-09-21-feature-add-axis.md) 「증분 재판정 ①」 · 맥락 [passes/2026-09-22-output-language-axis.md](passes/2026-09-22-output-language-axis.md) |
| `backend/src/feature_delete.rs` · `backend/tests/feature_delete.rs` · `e2e/tests/sc03-05-feature-delete-and-restore.spec.ts` · `e2e/tests/sc03-06-deleted-feature-rediscovery.spec.ts` (feature 삭제·보존 축 — 슬라이스 6c 새 파일 4개) | 15 | `3dfb50500ad6fc34aa799c7a466cd49461e086f01047888a982dcafd259cc4ad` | 순 제거 65행(유입 80 — `feature_delete.rs` 모듈 머리 17 → 1(보관소·`content_hash`·재발견 세 문단 = 0011 머리·doc-tracker 6c·PR #112 의 네 벌째) · 필드·private fn doc 4 · 테스트 머리 7 → 1 과 픽스처 fn doc·단정 옆 주석 7 · 두 spec 의 머리 문단(doc-tracker 매핑 행 축자) 10 과 단정 옆 주석 12 · Isolation 4 → 1 ×2 · 내부 fn JSDoc 1 — diff 기준 69행 삭제 · 4행 재작성) · 유지 15행(`//!` 요약 · `RETENTION_DAYS`·`PreviousDeletion` pub 요약 · `restore()` 의 409-not-404 이유 · `overlay` 를 순서 계약의 **정본**으로 재작성(±0, 호출부 사본은 1행 증분 ⑧이 걷음) · `previous_deletion` 의 `(unix 초, rowid)` 타이브레이커 · `current_document` 의 「편집 겹침은 얹지 않는다」 · 테스트 요약 1 · 두 spec 의 Isolation 1행씩) · 판단이 갈려 남긴 것 1건(`current_document` 의 세 번째 겹침을 빼는 이유) · **주석 제거 후 부모와 바이트 동일 4/4**(증분 4파일 포함 8/8) · 문서 게이트 4종 rc=0 · `npm run build` rc=0 · `cargo test --release` 200/0 · **판정기 `✅ 변경 없음`**(8/8) · 허브 `Documents` 44 → 45 · `0011_feature_deletions.sql` 25행은 사람 게이트라 **집지 않았다** (— **해소됨(2026-09-25, 27차 패스, `rct_20260925-0003`)**: 새 행 25 로 판정 — 유지) — [passes/2026-09-21-feature-delete-axis.md](passes/2026-09-21-feature-delete-axis.md) |
| `backend/src/settings.rs` · `backend/tests/settings.rs` · `e2e/tests/sc04-14-llm-output-language.spec.ts` (출력 언어 설정 축 — AC4.9 #108 이 들여온 새 파일 3개) | 17 | `9458795199cc7a8cc2c0f5ea12240c6657343d398d73fc9b9854d3edc64faba6` | 순 제거 14행 · 유지 17행 — `settings.rs` 5(모듈 머리의 스냅숏 계약 3 + 빈 `//!` 1 = `analysis.rs::create` 의 사본 · `SettingsView.llm_language` 의 rustdoc 링크 전용 1) · `tests/settings.rs` 4(테스트 fn doc 2건 — fn 이름과 바로 아래 단정이 그 문장 자체, 이로써 이 파일의 판정 대상 주석이 0행이 되어 지문 파일 수 130 → 129) · `sc04-14` 5(단정 재진술 한국어 한 줄 주석 — 복원 경로 ①(바로 아래 `toHaveAttribute`)와 ②(`docs/test/04-platform.md#시나리오 14` 기대 결과가 「누르는 즉시 저장」·「다시 열어도 유지」를 **축자**로 적는다) 둘 다) · **유지**: `analysis_language` doc 5행(이 축의 **정본** — API 가 직접 거는 호출까지 분석의 언어로 쓴다) · `llm_language` 의 폴백 사유 3행(모르는 값은 실패가 아니라 기본값 — 움직인 쪽은 코드다) · `sc04-14` 머리 블록 7행(**9차 패스가 `sc01-02` 에서 같은 모양을 유지로 닫은 선례 — 뒤집으려면 `sc01-02` 를 포함한 증분 재판정으로 한 번에**) · 마이그레이션 `0012_llm_language.sql` 14행은 사람 게이트라 **집지 않았다**(잔여 `0009`·`0010`·`0011` 과 합쳐 99행 한 배치) (— **해소됨(2026-09-25, 27차 패스, `rct_20260925-0003`)**: 새 행 25 로 판정 — 유지)**— 실제로는 `0013` 까지 더해 123행 한 배치로 닫혔다** — [passes/2026-09-22-output-language-axis.md](passes/2026-09-22-output-language-axis.md) |
| `backend/src/doc_conflict.rs` · `backend/tests/doc_conflict.rs` · `frontend/src/ResolveConflict.tsx` · `e2e/tests/sc03-07-auto-vs-user-edit-conflict.spec.ts` (충돌 해소 축 — 슬라이스 6d #114 가 들여온 새 파일 4개 — `backend/migrations/0013_feature_doc_conflicts.sql` 는 **D1 사람 게이트 풀**로 보류 (— **해소됨(2026-09-25, 27차 패스, `rct_20260925-0003`)**: 새 행 25 로 판정 — 유지)) | 32 | `ba3e2072149d6b26b4a63d17f134bce26d3efa25a66b2052b03f2d2e395722ab` | 순 제거 55행(모듈 머리의 기제 서술 21 + 3 — doc-tracker 6d 행이 재생·이월·충돌·결정 셋·미결정 재생·범위 밖을 거의 축자로 적는다(②) · 전송 경계와 선언 재진술 · 테스트 helper·의도 주석 8 · e2e 절 구분 5 · 화면 prop JSDoc) · **유지 31행** · **낡아서 거짓인 주석 1건 적발** — `status` 모듈 doc 이 「0012 의 CHECK 와 같은 값이어야 한다」고 적었으나 그 CHECK 는 **0013** 에 있고 0012 는 「어휘는 코드가 판정하며 이 파일에 CHECK 로 박지 않는다」고 스스로 적는다(제거 근거 강화) · 기계 판독 `// 검증 시나리오:` 1건 · `mock-exception:` 1건 · **화면 머리의 목업 매핑 1건**(M1 이 읽는다) 보존 — [passes/2026-09-22-conflict-axis.md](passes/2026-09-22-conflict-axis.md) · **증분 재판정 ①**(2026-09-25, 28차 패스): #126 이 연 5행 중 4행(`inherit` 의 「되살아나면 복원이 한 분석짜리 거짓말이 된다」 2 — PR #126 본문의 축자(③) · `after_restore` 의 NULL 서술 1 — 바인딩이 말한다(①) · `tests/doc_conflict.rs` 의 의도 주석 1 — fn 이름과 단정(①))을 **제거** · 유지 1행 — [passes/2026-09-25-history-restore-axis.md](passes/2026-09-25-history-restore-axis.md) 「증분 재판정 ①」 |

| `backend/src/crypto.rs` (봉투 암호화 1파일 — 13차 패스가 판정해 두고 **없어진 D2 규칙** 때문에 보류했던 몫) | 7 | `c646e3003f7eb8e96c33ede3d86b0365ea303cafe61529f488c68630b8df9e0f` | 순 제거 6행 · 유지 7행 — 13차 패스가 「보류분의 판정 결과」에 적어 둔 **제거 6 · 유지 7** 을 문면 그대로 적용했다(시그니처 재진술 `///` 2건(`seal`·`open`) · 모듈 머리의 **동작 서술** 3행(`seal()` 본문이 그대로 그 순서다) · `(AC4.3)` 꼬리표 · `Envelope` doc 의 rustdoc 링크 전용 교차 참조(`[`seal`]`·`[`open`]`) · `// Best-effort scrub …` 인라인은 모듈 머리 불변식에 **흡수**(2벌 → 1벌)) · **유지 7행**: 모듈 머리 불변식(persist 되는 것은 wrapped DEK 와 ciphertext 뿐 — 평문 비밀·평문 DEK 는 디스크에 닿지 않고 DEK 는 wrap 즉시 best-effort 로 지워진다; 「best-effort」는 Rust 가 소거를 보장하지 않아 코드에서 복원되지 않는다) · `Envelope` 의 「저장되는 전부이며 KEK 없이는 어느 것도 비밀을 드러내지 않는다」 · `open` 의 「변조(GCM 태그 불일치)·잘못된 KEK 는 **오류가 되지, 쓰레기 평문이 되지 않는다**」 · **주석 제거 후 부모와 코드 바이트 동일 1/1** · `cargo test --release` 통과 · 문서 게이트 3종 rc=0 · **판정기 `✅ 해당 없음`**(가짜 벽이 걷혔다는 세 번째 증거) · 허브 `Documents` 47 → 48 — [passes/2026-09-22-false-wall-teardown.md](passes/2026-09-22-false-wall-teardown.md) |
| `e2e/support/github-app.ts` · `e2e/tests/sc01-08-succeeded-stage-rerun.spec.ts` (e2e 잔여 미판정 2파일 — 22차 패스가 「판정 행 밖이라 **잔여**」로 명시 인계한 몫) | 3 | `07b7fb6e37e0014995b39a1a1dbec946d787583a0e32417d5fcbe68a72126ce6` | 순 제거 14행(`github-app.ts`: 제품 문서 재진술 2 — `docs/e2e-mocking-policy.md` 충실도 보증 절이 이 파일을 이름으로 들어 같은 내용을 적는다 · `request.get(url)` 재진술 1 / `sc01-08`: 시나리오 8 원문 축자 인용과 코드 재진술 4 · 세 번째 사본이 된 임대 규약 블록 3(계약은 `cluster.ts` 가 소유 — 원장 3행) · 절 제목 2 · 빈 `//` 2) · 유지 3행 — export 함수 JSDoc 요약 1줄(정책 유지 조항)과 `startedAt` 이 리셋으로 지워진다는 단정 설계 근거 2행 — [passes/2026-09-25-e2e-unadjudicated-remainder.md](passes/2026-09-25-e2e-unadjudicated-remainder.md) |
| `backend/src/db.rs` · `backend/tests/migrations.rs` · `backend/src/models.rs` · `deploy/k8s/pvc.yaml` (무인 자유 풀 4파일 — 19차 패스가 「판정 자체가 없다」로 명시 인계한 몫) | 17 | `b15045231102a7e23c54dd0f43311233c7ff438202b5ea308275881e760a9907` | 순 제거 16행(`connect` 요약 `///` 2 + 딸린 빈 `///` 1 — 바로 아래 빌더 세 호출의 번역 · `replicas: 1`/`Recreate` 문장 1(`deploy/k8s/deployment.yaml:8-11` + `backend/migrations/README.md:113`) · `backend/tests/migrations.rs` 모듈 머리 3 → 1 · **`APPLIED`·`PRE_CLEANUP` 출처 블록 7행이 `backend/migrations/README.md:26-32,36-37` 의 축자 사본** · `every_migration_file_is_pinned` doc 2 · 리스 컬럼 주석 2 → 1) · 유지 17행 · **판단이 갈려 남긴 것 1건**(`deploy/k8s/pvc.yaml` 의 「SQLite is single-writer」 — 인과가 복원 경로 넷 어디에도 없다) · `APPLIED`·`PRE_CLEANUP` 의 **값과 `.sql` 파일은 무접촉** · **주석 아닌 바이트가 부모와 동일 2/2** · `cargo test --release` **238 passed / 0 failed**(24 스위트) · 문서 게이트 3종 rc=0 · **판정기 `✅ 해당 없음`(probe 가 아니라 이 PR 의 실제 diff 로 실측)** — [passes/2026-09-22-free-pool-storage-axis.md](passes/2026-09-22-free-pool-storage-axis.md) |
| `backend/migrations/0009_feature_doc_edits.sql` · `0010_feature_additions.sql` · `0011_feature_deletions.sql` · `0012_llm_language.sql` · `0013_feature_doc_conflicts.sql` (D1 적용된 마이그레이션 풀 5파일 — 16차 패스가 `0001`~`0008` 을 닫은 뒤 **열 패스째 이월된 나머지 전부**) | 123 | `3568954f74e1293bfd823b6c23095e331f868fbc0b5f697d10da7db3aab6ff04` | 순 제거 **0행** · 유지 **123행** — 앞선 패스들이 이 문단들을 **정본으로 지정하며 코드 쪽 사본을 걷어 냈으므로** 지금 이 파일들이 해당 명제들의 **유일한 복원 원본**이다(실측: `content_hash` 명제의 코드 쪽 주석 사본 **0벌**). **정본 지정 9건**(승계 4 — `0009` 의 `content_hash`·`scenario_index`·`reason` CHECK 와 `0011` 머리 문단 / 신설 5 — `0012` 의 「어휘를 CHECK 로 박지 않는다」·두 자리 분리 근거, `0013:43` `status` 어휘·`0013:44` `source` 어휘, `0002:3` 「spoofable」). **판단 갈림 1건** — `0009` 11~17행 ↔ `0010` 17~20행의 `content_hash` 논거 두 벌은 정책의 비용 비대칭 조항으로 유지(제거 이득 3~4행 ↔ 비용은 운영 DB repair 창). **`.sql` 무접촉이라 D1 이 발화하지 않는다** — 판정기가 이 PR 의 실제 diff 에 **`✅ 해당 없음`** 을 내고 `review/manual-approval` 이 붙는다(사람 게이트·repair 창 없음). 전역 지문 **불변**(`lines=2635 files=136`) · 허브 `Documents` 50 → 51 · 제거 후보 3자리(기대 순 제거 6~8행)는 다음 repair 창 PR 의 몫으로 등재 — [passes/2026-09-25-migration-pool.md](passes/2026-09-25-migration-pool.md) |
| `backend/src/doc_history.rs` · `backend/tests/doc_history.rs` · `e2e/tests/sc03-08-history-and-point-in-time-restore.spec.ts` · `frontend/src/FeatureHistory.tsx` (변경 이력·임의 시점 복원 축 — 슬라이스 6e #126 이 들여온 새 파일 4개 — `backend/migrations/0014_feature_doc_restores.sql` 는 **D1 사람 게이트 풀**로 보류) | 29 | `03b56b572a1e3f1a0bf1ddb5981eb27deece7d00ecd47fef520ffbf8afc58fdb` | 순 제거 58행(`doc_history.rs` 모듈 머리 15 → 1 — 네 문단이 각각 PRD AC3.4 의 출처 어휘(②)·doc-tracker 2026-09 슬라이스 6e 절(②)·PR #126 본문의 절 제목 「복원은 문서를 다시 쓰지 않고 재생 구간을 자른다」와 「순서는 시각이 아니라 세대로 엮는다」(③)의 **네 벌째** · `EntryView`·`PreviewView` 필드 doc 10 — 필드 이름·`Option`·바로 아래 제어 흐름이 그 문장 자체 · `Step` 의 rustdoc 링크 전용 교차 참조 1(본문 규칙) · `steps`·`walk`·`standing_edits`·`baseline_at` doc 8 → 3(중복 서술과 PR 본문 축자 제거, 요약만) · `restore()` 의 「확인만 하고 유지」 2 — 여정 §4 분기표의 축자(②)이고 바로 아래 오류 문자열이 같은 말(①) · `tests/doc_history.rs` 머리 5 → 1 과 단정 옆 주석 4 — fn 이름·단정과 doc-tracker 매핑 행(②) · `sc03-08` 머리 문단 4 — doc-tracker 매핑 행의 여정 서술 축자(②) · 단정 옆 주석 3 · `FeatureHistory.tsx` 머리 8 → 1 — 「목업이 없다」 다섯 줄은 **자신이 출처를 지목한다**(`docs/doc-tracker/` 「수용된 위험」·「활성 대조 대상」 절, ②)이고 PR #126 에 전용 절이 있다(③) · `sourceLabel` doc 1 — 네 반환 문자열이 그 어휘 자체(①)) · 유지 29행(`AUTO` 의 「행이 아니라 자리라서 예약어」 · `RESTORE_SOURCE` 요약 · `routes()` 의 「항목도 주소를 가진다」(6a 「제안도」·6b 「초안도」 정본과 같은 규약) · `steps` 의 **어느 세대에도 들지 못한 편집을 마지막에 붙이는 이유**(「이력에서 조용히 빠지느니 순서가 거친 편이 낫다」 — 네 경로 어디에도 없다) · `walk`·`current_restore`·`scenarios_with` 요약 · `sc03-08` 의 **Isolation 2행**(sc03-07 과 바이트 동일한 공유 규약 — 9차·13차·17차가 같은 모양을 유지로 고정) · 절 구분 배너 8(6a 의 `── 탭 N ──` 선례를 승계) · 픽스처 doc 2(EXT-03 더블의 충실도 경계) · `FeatureHistory` 화면 머리 1 · 「고른 시점의 상태는 서버가 계산한다」 2) · 판단이 갈려 남긴 것 2건(절 구분 배너 8행 — 문면은 doc-tracker 매핑 행과 겹치지만 6a 가 유지로 닫은 선례를 뒤집지 않았다 · `RESTORE_SOURCE` 의 「모델이 끼어들지 않는다」) · **주석 제거 후 부모와 바이트 동일 4/4**(증분 6파일 포함 10/10 — 비주석 diff **0줄**) · 문서 게이트 3종 rc=0 · **판정기 `✅ 해당 없음`**(10/10 — `.sql`·`SELF_PATHS` 무접촉) · 허브 `Documents` 48 → 49 · `0014_feature_doc_restores.sql` 33행은 사람 게이트라 **집지 않았다** (— **해소됨(2026-09-25, 29차 패스, `rct_20260925-0006`)**: 새 행 27 로 판정 — 유지 18행 · 제거 후보 9행은 D1 repair 창으로 이월) · `tools/check-data-format-change.py` 36행은 **D6 판정기 자신**이라 무인 머지 경로가 없어 집지 않았다 (— **해소됨(2026-09-25, 29차 패스, `rct_20260925-0006`)**: 새 행 28 로 판정 — 유지 19행 · 제거 후보 12행은 D6 전용 PR 로 이월) — [passes/2026-09-25-history-restore-axis.md](passes/2026-09-25-history-restore-axis.md) |
| `e2e/tests/sc04-06-mobile-first-session.spec.ts` (AC4.4 모바일 우선 세션 spec 1파일 — 슬라이스 7a #152 가 들여온 신설 파일, 행 열 밖 잔여로 들어왔다) | 17 | `341304837fcd65e817fbfd943674dbda917f9d42f564863efdc2a1fb4ca86383` | 순 제거 **12행**(파일 머리 15 → 7 — 목업이 접힘을 그리지 않는다는 편차 등재 경위 5행은 #152 본문 「목업과 어긋나는 지점」 절(③)과 편차 원장 3행(②)이 소유 · 「쓰인 수는 전부 문서에서 왔다」 2행은 #152 본문 축자(③)이고 390px·5분은 `04-platform.md#시나리오 6`(②)이며 여기 적힌 600px 은 이 파일에 없는 수다 · 단계 소유권 2행은 본문 12~14행과 `(c)`·`(d)` 블록에 **두 벌**이라 편집 지점 쪽만 남겼다 · `SESSION_BUDGET_MS` doc 1행 — 이름과 `:108` 의 FAIL 문면 「네 단계가 5분 예산을 넘었다」가 그 문장 자체(①) · `(d)` 블록 3 → 2 — 소유권 꼬리만 · 데스크톱 블록 2행 — PRD AC4.4 검증 방법 「데스크톱은 동일 화면을 확장 적용한다」 축자(②)이고 바로 아래 두 줄이 그것을 집행한다(①)) · 유지 **17행**(관측 기준 2 — 이 spec 이 목업 카피가 아니라 폭 축으로 판정한다는 판별식이고 형제 spec 전부가 같은 자리에 같은 모양을 갖는다 · 「화면을 질러가지 않는다」 1 — `sc03-01` 의 유지형과 같은 가드 · **Isolation 1행** — `Leases the analysis worker …` 는 e2e spec 10개에 **바이트 동일**한 공유 규약이라 한 축에서 혼자 지우면 첫 이탈 · `fitsOneHandedWidth` doc 1 — 이름·FAIL 문면과 겹치지만 「한 손 조작 = 가로 스크롤 0」이 **문서에 없는 px 임계를 세우지 않기 위한 선택**이라 비용 비대칭으로 남겼다(**판단이 갈려 남긴 것**) · `:53` 예산 시작점 1 — 위로 옮기면 5분 예산이 사전 조건을 조용히 삼킨다 · `(a)`~`(d)` 단계 표식 4 — ② 축자이지만 네 블록과 시나리오 실행 단계의 유일한 지도라 남겼다(**판단이 갈려 남긴 것**) · `request-edit` 오조준 함정 2 · 승인 뒤 재마운트 1 · `dependencies-unasked` 2 — 셋 다 #152 재작업 절이 축자로 갖지만(③) **PR 본문은 편집 지점에서 읽히지 않는** 가드이고, 이 셋이 곧 그 PR 을 한 번 거부시킨 자리다) · **주석 제거 후 부모와 바이트 동일** · 비주석 diff 0줄 — [passes/2026-09-25-residual-pool-closeout.md](passes/2026-09-25-residual-pool-closeout.md) |
| `frontend/src/viewport.ts` (접힘 브레이크포인트 단일 소스 1파일 — 슬라이스 7a #152 가 들여온 신설 파일) | 3 | `3a1eabd627e69c4770647633443eb3beb86b068265ed4f52f331739eaede5a2b` | 순 제거 **4행**(`useWideViewport` 의 doc 5 → 1 — 「화면이 긴 결과를 요약으로 시작할지 정한다(AC4.4)」는 PRD AC4.4(②)와 #152 본문 「여는 주체는 화면이 넘기는 `open`」 절(③)이 소유하고, `open={wide}` 호출부가 그것을 보인다(①)) · 유지 **3행**(`WIDE_QUERY` doc 1 — **`index.css` 가 확장하는 첫 폭과 같아야 한다**는 두 소스 일치 불변식이라 남기되 `index.css` 를 이름으로 못박아 **값 옆으로 옮겨** 한 벌로 만들었다(제자리 재작성, 줄 수 불변) · `useWideViewport` 요약 1 — 본문 「TS export 함수의 JSDoc 요약 1줄은 유지」 · `:13` 경합 1 — 첫 렌더와 effect 사이에 질의가 이미 뒤집혔을 수 있다는 것이 **바로 아래 `setWide(query.matches)` 가 존재하는 유일한 이유**이고 네 경로 어디에도 없다) · **주석 제거 후 부모와 바이트 동일** — [passes/2026-09-25-residual-pool-closeout.md](passes/2026-09-25-residual-pool-closeout.md) |
| `e2e/tests/sc04-10-cross-user-data-isolation.spec.ts` (AC4.7 사용자 간 데이터 격리 spec 1파일 — 슬라이스 7b #155 가 들여온 신설 파일, 행 열 밖 잔여로 들어왔다) | 9 | `ff166536b6e33e0d86deffaee4c826d187d254f60879d635c23cdf8f32b4ec3d` | 순 제거 **12행** · 유지 **9행** — 파일 머리 13 → 5(stub 배포·`?as=<handle>` **일반 규약** 5행은 `docs/doc-tracker/2026-09.md` 89행이 소유하고(②) `passes/2026-09-20-github-app-auth-axis.md` 가 그 자리를 「각 파일에는 이 spec 이 어느 신원을 쓰는가만」으로 못박은 선례이며, 「사용자에서 파생해 두 stub 사용자가 충돌하지 않는다」는 **주석 스스로 이름으로 지목한 `github_app::stub_installation_id` 의 doc** 이 정본이다 · 시나리오 실행 단계 축자 3행은 `docs/test/04-platform.md#시나리오 10`(②)과 doc-tracker 매핑 행·변경 이력(②)·#155 본문 §2(③)이 소유 · **403 이 아니라 404** 2행은 주석 자신이 `analysis::owned_analysis` 를 지목하고 그 doc 이 「`404` (not `403`) … so the API never confirms that an id exists (AC4.7)」로 축자 소유하며 doc-tracker 매핑 행이 「그 이유는 `analysis::owned_analysis` 주석에 있다」로 정본을 못박는다 · 딸린 빈 `//` 2) · `signIn` JSDoc 1 — **비export 지역 함수**라 「TS export 함수의 JSDoc 요약 1줄 유지」 조항 대상이 아니고 이름·본문·`Promise<string>` 이 축자 · 절 제목 ⑴⑵ 2 — `expect` 메시지 「A 의 분석 목록」·「A 의 키 목록은 A 의 것뿐」·「A 가 B 의 ${url} 를 읽을 수 없다」가 그 문장 자체 · ⑷ 2 → 1) · **유지 9행**(자기 신원·두 컨텍스트 2 + 관측 선택 2 — 「목록에서 안 보이는 것과 id 를 알고도 못 읽는 것은 다른 관측」이라 이 spec 이 화면을 걷지 않는다는 **가드**이고 행 29 의 「화면을 질러가지 않는다」 유지형 선례와 같은 형태다 · 딸린 빈 `//` 1 · **절 구분 배너 2**(`── 사용자 B/A ──` — 6a 의 `── 탭 N ──` 와 행 29 의 배너 8행 유지 선례를 승계해 한 축에서 혼자 뒤집지 않았다) · 쓰기 경로 가드 1 (「거부가 조회에만 걸려 있으면 격리가 아니다」 — 쓰기 3경로 블록을 지우면 격리가 조용히 반쪽이 된다, 제자리 재작성) · 404 의 의미 고정 1(「"없는 자원"이 아니라 "남의 자원"」 — ② 히트지만 위 12개 404 단정의 의미가 이 한 줄에 걸려 있다, 2 → 1 재작성)) · 기계 판독 `// 검증 시나리오:` 1건 보존 · **주석 제거 후 코드 md5 부모와 동일** · 비주석 diff **0줄** · `expect` 12 == 12 · `;` 40 == 40 · 허브 `Documents` 53 → 54 — [passes/2026-09-25-ac47-isolation-spec.md](passes/2026-09-25-ac47-isolation-spec.md) |

**합계**: 판정 **149파일**(지문의 파일 집합 기준으로는 144 — `format.ts` · `backend/tests/crypto.rs` ·
`backend/tests/analyses.rs` · `deploy/k8s/kustomization.yaml` · `backend/tests/settings.rs` 가 전건 제거로 주석
0행이 되어 빠졌다. **파일 수는 지문의 `files` 가 아니라 행의 목록 길이로 센다** — 범위 칸의 백틱 경로를
첫 여는 괄호 앞까지만 취하고 bare `.sql` 을 `backend/migrations/` 로 resolve 해 센 값이다) ·
⚠️ **앞 줄의 수는 29차 패스가 실측으로 정정한 것이다.** 이 자리에 있던 **135**(지문 기준 128)는
26차 패스 이후 갱신되지 않아 **9 낡아 있었다** — 27차 패스가 행 25(5파일)를, 28차 패스가 행 26(4파일)을
더하면서 합계 서술을 함께 올리지 않았다. 위 세는 법으로 원장 이력을 훑으면 행 20~24 시점의
기재값 124·128·129·133·135 가 **전건 재현**되고 `06d26ab`(140) · `f4b7db3`(144)에서만 갈라진다 —
즉 기재가 멈춘 것이지 세는 법이 바뀐 것이 아니다. 낡은 값은 지우지 않고 여기 인용해 둔다 ·
순 제거 누적 **2,630행**
(직전 2,375 + 21차 패스(#139) 6 + 22차 패스 16 + 23차 패스(#143) 60 + 24차 패스(#145) 1 + 25차 패스 21
+ 26차 패스 20 + 27차 패스 **0**(전건 유지) + 28차 패스 **72** + 29차 패스 **16**(무인 자유 풀 2파일 —
게이트 뒤 2파일의 제거 후보 21행은 이월) — 23·24차는 합계 서술을 갱신하지 않고
넘겼으므로 22차 패스가 이어 적었고, 이 줄은 29차 패스가 이어 적었고, 30차 패스가 **2**(행 열 안 미판정 증분 7행의 증분 재판정)를, 31차 패스가 **3**(원장 1행의 #49 경합 보류분 집행)을, 32차 패스가 **5**(행 10·11 에 #156 이 연 11행의 증분 재판정)를, 33차 패스가 **33**(#155 이 다시 연 43행의 판정)을 이어 적는다) ·
판정 범위의 현재 합계 **2,721행**(행 열의 합 — 29차 패스가 **행 열 밖 잔여를 0** 으로, 30차 패스가
**행 열 안 미판정 증분을 0** 으로 만들었다. 다만 30차 패스의 CI 대기 창에 착지한 **#155**(슬라이스 7b,
`9c01489`)가 둘 다 다시 열었다 — 아래 산술의 ⑴ 21 · ⑵ 22 가 그것이고, **다음 감지 주기의 몫**이다
(— **해소됨**(2026-09-25, 33차 패스, `rct_20260925-0009`): ⑴ 은 **새 행 31**, ⑵ 는 **행 14 의 증분 재판정 ②**).
31차 패스의 `ci-gate` 대기 창에 **#156**(`84d312a`)이 11행을 더 열었다 — `frontend/src/index.css` +1(**행 10**) ·
`tools/check-mockup-render.py` +10(**행 11**), **둘 다 행 열 *안*** 이라 움직인 것은 ⑵ 뿐이다(22 → **33**, ⑴ 은 21 그대로).
31차 패스는 ⑴·⑵ 어느 쪽도 판정하지 않고 **행 열의 합에서만 3** 을 덜어냈다. **32차 패스가 그 11행을 증분 재판정으로 닫았다** — 행 10 을 92 → 93, 행 11 을 112 → 117 로 옮겨 행 열의 합이 2,705 → **2,711** 이 되고 ⑵ 가 33 → **22** 로 내려왔다. 남은 그 **⑴ 21 · ⑵ 22** 를 **33차 패스(이 패스)가 닫는다** — 행 열의 합에 **+10**(행 14 +1 · 새 행 31 의 9)을 얹어 **2,721** 로 만들고 ⑴·⑵ 를 **둘 다 0** 으로 되돌린다).

**산술은 두 축으로 갈라 화해한다** — 한 덩어리로 쓰면 「잔여」가 ⑴·⑵ 를 섞어 버린다.

```
행 열의 합 2,721  +  ⑴ 행 열 밖 잔여 0  +  ⑵ 행 열 안 미판정 증분 0  =  전역 live 2,721
```

이 세 값은 **32차 패스의 머지 프리이미지**(PR head + `origin/main@b09f505`)에서 잰 것이다. 재는 법은
**행 열의 합** = 위 표 줄 수 칸의 합(`|` split 파싱 — 백틱 정규식은 축 이름 안의 디렉터리까지 긁고
「… 로 보류」 단서 경로를 다른 행에 겹쳐 센다) · **⑴** = 표에 이름이 **없는** 파일의 live 주석 ·
**⑵** = 표에 이름이 **있는** 파일의 live 합 − 행 열의 합.
⑴·⑵·전역 live 는 **판정 없이도 자매 착지만으로 움직인다** — #155 가 ⑴·⑵ 를, #156 이 ⑵ 를 열었다.
**⑵ 의 남은 22 는 #155 가 `backend/tests/worker.rs` 에 연 몫이다** — #156 이 연 11 은 32차 패스가 닫았다.
움직였으면 이 세 줄을 위 세는 법으로 **다시 재어 맞춘다**(판정이 아니라 재측정이다).
**판정이 움직이는 값은 행 열의 합 하나뿐이다.**

⑵ 의 7행은 **슬라이스 7a(#152)가 이미 판정된 세 행 안에** 연 것이다(`e2e/playwright.config.ts`
5 → 8 = 행 3 · `frontend/src/index.css` 5 → 6 = 행 10 · `e2e/tests/sc03-01-llm-assisted-edit.spec.ts`
9 → 12 = 행 17 — **해소됨(2026-09-25, 30차 패스)**: 아래 「30차 패스(이 패스)가 이 트리를 만들었다」). **29차 패스는 ⑵ 를 판정하지 않았다** — 증분 재판정은 원래 행·원래 패스 파일에
절을 더하는 별도 슬라이스이고(본문 「판정 절차」 3), 열린 #148 이 `e2e` 12파일을 건드리므로 행 3 의
증분은 착지 전에는 확정되지도 않는다고 봤다. 세 행의 줄 수 칸은 **그 재판정(30차 패스)이 옮겼다.**
**그중 판정 완료는 2,433행**이고 **미판정 증분은 33행**이다 — #114 가 판정 완료 범위 안 네 행(원장 1·4·7·10행)에
연 **순 +24행**을 18차 패스가 증분 재판정으로 닫았고(순 제거 11행), 같은 슬라이스가 들여온 새 파일 4개는
**새 행 21**(31행)이 됐다. 18차 패스가 판정하지 않고 넘긴 4행(**#123** 이 원장 1행에 연 것)은 이 패스의
**증분 재판정 ⑪** 이 닫았다(순 제거 3행). 남은 39행은 **이 패스의 머지 직전에 착지한 #121**(`b4a6b30`,
AC1.5 확장)이 원장 **1·4·5·7·9행**에 연 **39행** 중 **행 1 +26 · 행 4 +7 의 33행**이다(각 행의
「자매 착지 재실측」 — 판정하지 않고 다음 감지에 넘긴다). 행 5·7·9 의 몫 6행은 **23차 패스(#143)가
#137·#138 의 증분과 함께 닫았다.** 같은 PR 이 들여온 **새 파일**
`e2e/tests/sc01-08-succeeded-stage-rerun.spec.ts` 13행은 판정 행 밖이라 **잔여**로 들어갔다
(— **해소됨(2026-09-25, 25차 패스)**: 새 행 24).
행 열의 합은 **2,466 = 판정 완료 2,433 + 미판정 증분 33** 이었다.

**30차 패스(이 패스)가 이 트리를 만들었다.** 29차 패스(#153)가 「⑵ 축 증분 재판정 — 위 세 행.
**#148 착지 뒤**에 하는 것이 맞다」로 인계한 몫 — #152(`ea04fe0`)가 **판정 완료 행 안에** 연
**미판정 증분 7행**(행 3 `playwright.config.ts` +3 · 행 10 `index.css` +1 · 행 17 `sc03-01` +3) — 을 받아
증분 재판정 ④·⑪·④ 로 닫았다 — **순 제거 2행 · 유지 5행**. 「같은 범위의 증분 재판정은 새 행을 만들지 않고
원래 행의 결과 칸을 갱신하고 상세는 원래 패스 파일에 절을 더한다」(본문 「판정 절차」 3항)를 따라
**새 행도 새 패스 파일도 만들지 않았다** — 그래서 허브 `docs/index.html` 의 문서 집합이 그대로이고
`check-journey-mockup.py` R8·R9 가 손대지 않은 채 통과한다.

🔴 **인계 사유의 「#148 을 기다려라」는 실측에서 거짓이었다.** #148 은 행 3 의 `sc01-01`·`sc01-06` 을 고치지만
**주석 줄은 0행**이어서(diff 의 ±줄 중 주석 시작 패턴에 걸리는 것이 없다) 행 3 의 지문은 `main` ·
`main+#148` · `main+#153` **세 트리에서 145 / `97a280f4…` 가 바이트 동일**하다. 증분은 이미 확정돼 있었고,
기다리면 그만큼 미판정 상태가 길어질 뿐이었다. 행 10·17 의 파일은 **어느 열린 PR 도 건드리지 않는다**
(#148 15파일 · #153 5파일 전건 교집합 0) ⇒ 이 패스의 소스 판정은 **착지 순서에 대해 자유롭다.**

**⑴ 축과는 겹치지 않는다.** 이 패스가 닫는 것은 **행 열 *안*** 의 증분이고, 열린 #153(29차 패스)이 닫는
것은 **행 열 *밖*** 의 잔여 105행(신규 2파일 36 + D1 33 + D6 36)이다. 소스 파일 교집합이 0 이므로 두 PR 은
`ledger.md` 의 이 합계 문단에서만 만난다 — **뒤에 서는 쪽이 합계를 live 에서 재측정해 다시 적는다**(절대값을
베끼지 않는다). **#153 이 먼저 착지했으므로 이 패스가 뒤에 서서 재측정했다** — 이 패스만의 효과는
리베이스 뒤 부모 `e240cac` 의 `lines=2710 files=143` → **`lines=2708 files=143`** 이고(계획 시점
부모 `ea04fe0` 기준 예고였던 `2726 → 2724` 는 base 이동으로 지났다 — 효과인 **순 제거 2행 · `files` 불변**은 같다),
비주석 코드는 두 파일 모두 부모와 **바이트 동일**(`index.css` 는 블록 주석 인식 stripper 로 md5 `4d98ba8c`,
선언 619 == 619 · 규칙 184 == 184 · `sc03-01` 은 줄머리 필터로도 동일)이다.

**이 패스만의 트리(부모 `e240cac`)에서 검산은 행 열의 합 2,708 + ⑴ 0 + ⑵ 0 = 전역 2,708** 이었다.
⑴ 105 는 전부 #153 의 몫이었고 그 PR 이 먼저 착지해 **0** 이 됐다. ⑵ 는 이 패스가 **7 → 0** 으로 닫았다.

⚠️ **머지되는 트리(부모 `9c01489`)의 검산은 다르다.** 이 패스의 `ci-gate` 를 기다리는 동안 **#155**
(AC4.7 분석 작업의 격리 — 슬라이스 7b)가 착지해 주석 **+43행 · `files` +1** 을 들여왔다.
소스 파일 교집합은 **0** 이라 텍스트 머지는 깨끗했고(이 패스의 ± 줄 집합 md5 `35b7c12f…` 불변),
움직인 것은 합계 문면뿐이다 — 실측으로 다시 적었다.

- **⑴ 21** — `e2e/tests/sc04-10-cross-user-data-isolation.spec.ts`(#155 가 들여온 신설 파일, **어느 행에도 없다**)
- **⑵ 22** — `backend/tests/worker.rs` 6 → 28(**행 14** 「테스트 하네스 축 5파일」 *안*)
- 합쳐 **행 열의 합 2,708 + 21 + 22 = 전역 live 2,751** 로 화해한다. 이 43행은 이 패스의 판정 범위
  **밖**이며(승인된 계획은 #152 가 연 7행만 닫는다), **다음 감지 주기가 후속 task 로 연다.**
  (— **해소됨**(2026-09-25, 33차 패스, `rct_20260925-0009`): ⑴ 은 **새 행 31**, ⑵ 는 **행 14 의 증분 재판정 ②** —
  아래 「33차 패스(이 패스)가 이 트리를 만들었다」.)

**33차 패스(이 패스)가 이 트리를 만들었다 — #155 의 43행.** 30차 패스가 위에 인계한 43행을 받아 **한 슬라이스로** 닫았다. 쪼개지 않은 이유는 **게이트 종류가 같기**
때문이다 — 두 파일 모두 `tools/check-data-format-change.py` 의 **D1**(`backend/migrations/**`)·**D6**
(`SELF_PATHS` = `tools/check-data-format-change.py` · `.github/workflows/data-format-review.yml` 두 항목,
실측) 밖이라 판정기가 `✅ 해당 없음` 을 내고 `review/manual-approval` 이 자동 success 가 된다(28차 패스가
D1·D6 를 가른 근거가 「게이트가 다를 때만 쪼갠다」이므로 여기서는 쪼갤 이유가 없다).

기재 형태는 원장 본문 「판정 절차」 3항이 이미 정한다. `backend/tests/worker.rs` 는 **행 14 안**이라 새 행을
만들지 않고 결과 칸에 **증분 재판정 ②** 를 이어 적고 상세는 원래 패스 파일
[passes/2026-09-20-test-harness-axis.md](passes/2026-09-20-test-harness-axis.md) 에 절로 붙였다.
`sc04-10` 은 **어느 행에도 없는 신설 파일**이라 **새 행 31** 과 새 패스 파일이 필요했고, 그래서 이 패스는
30차 패스와 달리 `docs/` 에 `.md` 를 하나 더한다 — `check-journey-mockup.py` **R9** 가 허브 요약
`Documents` 선언 수와 링크 집합을 `docs/` 의 실제 md 집합과 1:1 로 대조하므로 `docs/index.html` 에
`doc-row` 1건을 더하고 선언을 **53 → 54** 로 올렸다(계획 단계에서 게이트를 직접 굴려 잡았다).

🔴 **위 산술의 `⑵ 22` 는 이 패스가 집는 시점에 이미 낡아 있었다 — 값이 아니라 배분이.** #155 뒤에 **#156**
(`84d312a`, 목업↔구현 수렴)이 착지해 주석 **순 +11행**(추가 12 · 제거 1)을 들여왔고, 그 11행은 **행 열
*밖*이 아니다**: `frontend/src/index.css` +1 은 **행 10**, `tools/check-mockup-render.py` +11/−1 은 **행 11**
로 둘 다 이미 행이 있다. 따라서 `84d312a` 에서의 올바른 화해는 **⑴ 21 + ⑵ 33 = 2,762** 이며(⑴ 을 32 로
읽으면 이미 행이 있는 두 파일에 **새 행을 만들게 된다**). 그 11행은 **`rct_20260925-0010`(32차 패스, PR #159,
`4d2cc84`)이 이 패스의 준비 창에 먼저 착지해** 증분 재판정으로 닫았다(행 10 92 → 93 · 행 11 112 → 117) —
이 패스는 그 11행을 건드리지 않고, 두 패스가 만난 자리는 이 합계 문단 하나뿐이었다(소스 교집합 0).
**뒤에 선 이 패스가 live 에서 재측정해 다시 적었다** — 그래서 이 패스가 닫히면 ⑴·⑵ 가 **둘 다 0** 이다.
검산은 원장을 `|` 로 쪼개
행 열을 직접 더하고 행별 live 값을 지문에서 다시 센 것이다 — 이 패스를 집은 시점(`84d312a`)에는 어긋난 행이
**셋**(10: 92 vs 93 · 11: 112 vs 122 · 14: 20 vs 42, 합 **33**)이었고, 32차 패스가 10·11 을 닫은 뒤(`4d2cc84`)
**하나**(14: 20 vs 42, **22**)였으며, **이 패스가 닫으면 0** 이다.

⚠️ **이 축은 구조적으로 재충전된다.** #126 → #152 → #155 → #156 으로, 자매 모델
`tbm_feature-doc-docs-impl` 의 슬라이스가 착지할 때마다 **AC 꼬리표·시나리오 문장 인용·단정 옆 절 제목**이
같은 유형으로 다시 들어온다(이 패스가 제거한 33행 중 AC 꼬리표 1 · 시나리오 축자 5 · 절 제목 5). 행 단위로
닫는 것만으로는 다음 슬라이스가 같은 유형을 다시 들여오므로, **근본 대응은 그 유형을 잡는 레포 게이트**다
(현재 이 축의 게이트 하중은 **0** — `tools/` 6종 · `.github/workflows/` 7종 전수에 주석 비중복성 검사가
없다). 그것은 이 모델의 판정 표면 밖이므로 **예고로만** 적어 둔다: 다음 docs-impl 슬라이스가 같은 유형을
다시 들여오면 **게이트 신설을 별도 task 로 연다.** — 🔴 **이 예고는 적는 사이에 이미 발화했다**: 열린
**#158**(슬라이스 — AC4.1 「해제」 조항)이 **이 패스가 판정한 그 파일** `backend/tests/worker.rs` 에 **+117행**
(그중 주석 약 12행 — `AC4.1 「해제」` 꼬리표 2건 · 픽스처 doc · 단정 옆 절)을 얹는다. 순수 추가라 이 패스의
헝크와는 겹치지 않고(`--stat` 117 insertions / 0 deletions) `sc04-10` 도 건드리지 않으므로 **착지 순서는
자유**지만, 착지하면 **행 14 가 또 열린다**. 그러니 행 14 의 완료 기준은 절대값이 아니라 **부모 대비 −21행**
이다(같은 이유로 이 패스는 21/`d4414d5f…` 를 세 base `84d312a`·`b09f505`·`4d2cc84` 에서 바이트 동일로 재고정했다).

⚠️ **25차 패스가 닫은 것은 그 33행이 아니다 — 26차 패스가 실측으로 정정한다.** 25차 패스는 이 자리에
「그 33행을 전부 닫았다」로 적었지만, 그것이 닫은 것은 **행 12 의 증분 14행**(증분 재판정 ⑫)과
**행 밖 잔여였던 `e2e/support/github-app.ts` 4행 + `e2e/tests/sc01-08-succeeded-stage-rerun.spec.ts` 13행**
(**새 행 24**), 합 **31행**이다. 「미판정 증분 33행」은 **행 1 +26 · 행 4 +7** 로 다른 몫이었고, 25차 패스가
착지한 트리(`39ee044`)에서도 행 1 은 629, 행 4 는 149 로 **#121 의 주석이 그대로 남아 재현된다**(실측).
**두 수가 겹쳐 보인 것은 「23행 전부가 재현된다」를 판정의 증거로 읽었기 때문이다** — 「읽는 법」이 적은 대로
지문 재현은 *그 줄이 판정을 받았음*을 뜻하지 않는다. 그 33행은 **26차 패스(이 패스)** 가 증분 재판정
⑫(행 1)·④(행 4)로 닫았다 — 순 제거 20 · 유지 13.

이 트리의 행 열의 합은 **2,456 = 판정 완료 2,456 + 미판정 증분 0** 이고, 이 패스의 base(`39ee044`)에서
**24행 전부가 줄 수·지문 둘 다 재현된다**(실측 24/24 — 행 1·4 는 이 패스가 재고정한 새 값으로). 이 24/24 는
**그 base 에 묶인 값**이다 — 열린 자매 PR 이 착지하면 그쪽이 연 증분만큼 다시 어긋나고
(현재 열린 것: **#126** → 행 4·7·10·21 과 새 파일 5개 · **#148** → `e2e/playwright.config.ts` 등 행 3),
그것은 이 패스의 오류가 아니라 **다음 감지가 여는 몫**이다.

**26차 패스(이 패스)가 이 트리를 만들었다.** 22차 패스(#130)가 「미판정 증분은 33행 … 판정하지 않고 다음
감지에 넘긴다」로 인계하고 25차 패스(#147)가 **자기 몫이 아닌 채로 닫혔다고 적은** 몫 — **원장 행 1 의 26행과
행 4 의 7행** — 을 받아 증분 재판정 ⑫·④ 로 닫았다. **순 제거 20행 · 유지 13행.**
「같은 범위의 증분 재판정은 새 행을 만들지 않고 원래 행의 결과 칸을 갱신하고 상세는 원래 패스 파일에 절을
더한다」(본문 「판정 절차」 3항 · 「읽는 법」)를 따라 **새 행도 새 패스 파일도 만들지 않았다** — 그래서
허브 `docs/index.html` 의 문서 집합이 그대로이고 `check-journey-mockup.py` R8·R9 가 손대지 않은 채 통과한다.
`.sql` 은 한 글자도 건드리지 않았고(세 파일 모두 D1 경로 `backend/migrations/**` 밖이라 판정기가 이 PR 의
실제 diff 에 `✅ 해당 없음` 을 냈다), 세 파일의 **비주석 줄은 부모와 md5 동일**하다.
**이 패스만의 효과**는 `lines=2635 files=136` → `lines=2615 files=136` 다.
**이 트리의 검산은 행 열의 합 2,456 + 잔여 159 = 전역 2,615** 이고, 잔여 159 는 25차 패스가 적은 그대로
전부 범위 밖이다 — D1 사람 게이트(`backend/migrations/0009`~`0013` 123행)와 D6 판정기 자신
(`tools/check-data-format-change.py` 36행). 이 패스는 행 1·4 의 파일만 건드렸으므로
**행 합 −20 == 전역 −20, 잔차 0** 이다.

**25차 패스가 그 직전 트리를 만들었다.** 22차 패스가 「다음 감지가 여는 몫」으로 명시 인계한 **행 12 의
미판정 증분 14행**과, 같은 문단이 「판정 행 밖이라 잔여」로 남긴 **새 파일 2개 17행**을 함께 받아
증분 재판정 ⑫ 와 **새 행 24**(3행)로 닫았다 — **순 제거 21행**. `.sql` 은 한 글자도 건드리지 않았고
바뀐 파일 4개의 diff 는 **주석 줄뿐**이다(비주석 +/- 0줄). **이 패스만의 효과**는
`lines=2656 files=136` → `lines=2635 files=136` 다.
**28차 패스가 이 트리를 만들었다.** 27차 패스가 「다음 감지가 여는 몫」으로 명시 인계한 **140행** 중
**무인 풀 107행**을 받아 **새 행 26**(29행)과 증분 재판정 5건(행 4·7·10·17·21)으로 닫았다 —
**순 제거 72행**. 인계된 140 을 통째로 집지 않고 **벽 종류로 갈라** 가져온 것이 이 패스의 판단이다:
나머지 둘은 벽이 서로 다르고 **둘 다 이 슬라이스에서 넘을 수 없다** — D1
(`backend/migrations/0014_feature_doc_restores.sql` 33행)은 사람 repair 창이, D6
(`tools/check-data-format-change.py` 36행)은 `SELF_PATHS` 라 **원리적으로 무인 머지 경로**가 없다.
한 덩어리로 묶었으면 가벼운 107행이 그 둘의 일정에 묶였을 것이다.
유입량은 **gross 로 셌다** — `doc_edit.rs` 는 2행을 지우고 9행을 더해 순증이 7이므로, 순증만 보면
그 파일의 판정 표면을 7행으로 과소 진술하게 된다(실제 22행).
**`.sql` 과 `SELF_PATHS` 를 한 글자도 건드리지 않았으므로 D1·D6 이 발화하지 않고**, 판정기가 이 PR 의
실제 diff 에 **`✅ 해당 없음`** 을 낸다. 🔴 **공유 규약 하나를 의도적으로 남겼다** — `sc03-08` 머리의
`// Isolation:` 2행은 `sc03-07` 의 것과 **바이트 동일**하고 같은 형태가 e2e spec **14개**에 있다.
과거 패스가 줄인 것은 4행짜리 옛 형태이고 2행 형태는 9차·17차·22차가 유지로 고정했다 — 여기서만
지우면 공유 규약의 **첫 이탈**이 된다. **이 패스만의 효과**는 `lines=2755 files=141` →
**`lines=2683 files=141`** 이고, 비주석 diff 는 **0줄**이다(주석 제거 후 부모와 코드 바이트 동일 10/10).

**이 트리의 검산은 행 열의 합 2,614 + 잔여 69 = 전역 2,683** 이고, **행 열 밖 미판정 증분은 0** 이다.
그 잔여 69 는 전부 범위 밖이다 — D1 사람 게이트(`0014` 33행)와 D6 판정기 자신(36행). 27차 패스가
`0009`~`0013` 을 비운 **바로 그 창에서 `0014` 가 D1 풀을 다시 채웠다**: 슬라이스마다 마이그레이션이
1건씩 들어오므로 이 풀은 **구조적으로 재충전된다**.

**27차 패스가 그 직전 트리를 만들었다.** 16차 패스(#98)가 `0001`~`0008` 을 닫은 뒤 **열 패스째 「범위 밖」으로
이월돼 온 D1 마이그레이션 풀 5파일 123행**(`0009`~`0013`)을 받아 **새 행 25** 로 닫았다 — **순 제거 0행**.
판정 결과가 「전건 유지」인 것은 게이트를 피한 결과가 아니라 **앞선 세 패스가 이 문단들을 정본으로 지정하며
코드 쪽 사본을 이미 걷어 냈기 때문**이다(실측: `content_hash` 명제의 코드 쪽 주석 사본 **0벌**). 그래서
`feature-delete-axis` 패스가 「0011 을 판정할 때 그 문단들은 **유지 후보**다」로, `conflict-axis` 패스가
「**0013 을 판정할 때 그 자리를 정본으로 못박는다**」로, `github-app-auth-axis` 패스가 「**마이그레이션
패스가 이 표의 정본 지정을 이어받으면 된다**」로 넘긴 인계 **3건을 여기서 모두 수신**했다 —
이 패스의 산출은 제거가 아니라 **등재와 정본 지정 9건**이다.
**`.sql` 을 한 글자도 건드리지 않았으므로 D1 규칙이 발화하지 않고**, 판정기가 이 PR 의 실제 diff 에
**`✅ 해당 없음`** 을 내 `review/manual-approval` 이 붙는다 — 이 축이 열 패스째 이월된 이유였던
**운영 DB repair 창(suspend → scale 0 → SHA-384 `UPDATE` → resume)과 사람 게이트가 이 슬라이스에는 없다.**
**이 패스만의 효과**는 전역 지문에 **없다** — 주석 줄을 하나도 바꾸지 않았으므로
`lines=2755 files=141` · `580bec76b471…` 가 부모와 **동일**하고, 움직인 것은 to-be(`docs/comment-policy`
tree hash)뿐이다. **수치는 머지 직전 트리(base `9373c51`)에서 다시 재어 적는다** — 계획이 「열린 자매」로
적은 둘이 **모두 먼저 착지했다**: **#149**(26차 패스)가 `4457c95`, **#126**(슬라이스 6e)이 `9373c51`.
#149 가 행 1 에서 13행 · 행 4 에서 7행을 지워 행 열의 합이 2,599 → **2,579** 로 내려갔고, #126 은 주석
**+140행**을 들여왔다 — 판정 완료 범위 안(행 4 +1 · 7 +2 · 10 +5 · 17 +7 · 21 +5)의 **미판정 증분 20행**과,
**새 파일 5개 120행**(`backend/migrations/0014_feature_doc_restores.sql` 33 · `backend/src/doc_history.rs` 46 ·
`backend/tests/doc_history.rs` 10 · `e2e/tests/sc03-08-history-and-point-in-time-restore.spec.ts` 20 ·
`frontend/src/FeatureHistory.tsx` 11)이 잔여에 더해진 몫이다. **이 트리의 검산은 행 열의 합 2,579 +
행 열 밖 미판정 증분 20 + 잔여 156 = 전역 2,755** 다(새 행 25 의 **123행 · `3568954f74e1…`** 는 `.sql`
무접촉이라 **어느 착지 순서에서도 불변**이고 이 트리에서 바이트 재현된다). 그 20 + 120 행은 이 패스의
범위가 아니라 **다음 감지가 여는 몫**이다 — 착지 순서의 함수일 뿐 이 패스의 오류가 아니다.
잔여 156 중 **D6 판정기 자신**(`tools/check-data-format-change.py`) 36행은 `SELF_PATHS` 에 있어
건드리는 순간 status 가 붙지 않으므로 **무인 머지 경로가 없다**(원장 14행이 음성 대조로 등재해 둔 그대로).
제거 후보로 검토했으나 비용 비대칭으로 유지한 세 자리(기대 순 제거 **6~8행**)는 사람이 repair 창을 여는
전용 PR 의 몫으로 판정 상세에 등재했다.

**이 트리의 검산은 행 열의 합 2,476 + 잔여 159 = 전역 2,635** 이고, 그 잔여 159 는 전부 범위 밖이다 —
D1 사람 게이트(`backend/migrations/0009`~`0013` 123행)와 D6 판정기 자신(`tools/check-data-format-change.py` 36행).

**22차 패스가 그 직전 트리를 만들었다.** 19차 패스(#128)가 「벽은 걷혔지만 **판정 자체가 없다** …
그 축과 함께 볼지 따로 볼지는 **다음 계획에서 정한다**」로 명시 인계한 **무인 자유 풀 4파일 33행**을 받아
**새 행 23**(17행)으로 닫았다 — **순 제거 16행**. `.sql` 은 한 글자도 건드리지 않았고
(`backend/tests/migrations.rs` 는 D1 경로 `backend/migrations/**` 밖이다), 판정기가 **이 PR 의 실제 diff 에
`✅ 해당 없음`** 을 냈다. **이 패스만의 효과**는 `lines=2672 files=136` → `lines=2656 files=136` 다.
**이 트리의 검산은 행 열의 합 2,466 + 행 열 밖 미판정 증분 14 + 잔여 176 = 전역 2,656** 이다.

⚠️ **이 트리에서 행 12 는 재현되지 않는다 — 기록 오류가 아니라 #141 의 미판정 증분이다.**
(— **해소됨(2026-09-25, 25차 패스, `rct_20260925-0001`)**: 행 12 는 78행·`97ae560b…` 로 재고정됐고
`e2e/support/github-app.ts` 는 새 행 24 에 들어갔다. 아래 문단은 22차 패스 시점의 기록이다.)
자매 모델의 reconcile PR **#141**(`c405ad7`, `tbm_feature-doc-e2e-mock-policy` `rct_20260924-0001`)이
판정 완료 범위 안 그 행에 **+14행**을 열었다(`github_app.rs`·`backend/tests/github.rs`). 증분 재판정은
**새 행이 아니라 원래 행의 결과 칸 갱신**이므로(「읽는 법」) 22차 패스는 이것을 섞지 않고 **다음 감지가
여는 몫**으로 넘긴다. 같은 PR 이 들여온 새 파일 `e2e/support/github-app.ts` 4행은 판정 행 밖이라
**잔여**로 들어갔다.
(이 패스가 준비된 뒤 열린 다른 증분은 자매 주석 패스들이 이미 닫았다 — #132·#134 의 행 9·10 8행은
**21차 패스(#139)**, #121·#137·#138 이 행 5·7·9 에 남긴 78행은 **23차 패스(#143)**, #142 가 행 14 에 연
1행은 **24차 패스(#145)** 가 닫았다. #140 이 행 9 에서 지운 4행도 #143 의 재판정에 흡수됐다.)
**나머지 22행은 줄 수·지문 둘 다 전건 재현된다**(실측 22/23 — 같은 파서를 base·head 양쪽에 굴려
미재현 집합이 {12} 임을 확인).

**21차 패스(#139, `6e6478a`)가 그 직전 트리를 만들었다.** 자매 task `rct_20260922-0009` 가
#132·#134 의 미판정 증분 8행(행 9 +6 · 행 10 +2)을 **증분 재판정**으로 닫았다 — 새 행도 새 판정 파일도
만들지 않고 원래 행의 결과 칸을 갱신하고 상세는 원래 패스 파일(`passes/2026-09-20-pipeline-cross-cutting-axis.md` ·
`passes/2026-09-20-frontend-shell-axis.md`)에 절을 더했다(원장 「읽는 법」). 그 착지로 행 9 는 142 → **199**,
행 10 은 88 → **89** 가 됐고 전역은 `lines=2726` → `lines=2720`(순 제거 6)으로 움직였다. 이 문단은 그
패스가 합계 서술을 갱신하지 않고 넘긴 자리를 22차 패스가 이어 적은 것이다.

**20차 패스(#129, `27d9b81`)가 그보다 앞선 트리를 만들었다.** 사람 PR 이 아니라 **자매 모델의 reconcile PR #123**
(`77158c2` · `tbm_feature-doc-e2e-mock-policy` `rct_20260922-0001`)이 판정 완료 범위 안
`backend/src/llm.rs` 에 연 **순 +4행**(추가 6 · 제거 2)을 **증분 재판정 ⑪** 로 닫았다 — **순 제거 3행 ·
유지 3행**. 18차 패스(#124)가 「판정하지 않고 다음 감지에 넘긴다」로 넘긴 바로 그 몫이다. 새 행도 새 판정
파일도 만들지 않았으므로(원장 「읽는 법」: 같은 범위의 증분 재판정은 원래 행의 결과 칸을 갱신하고 상세는
원래 패스 파일에 절을 더한다) 판정 **129파일**은 불변이고 허브(`docs/index.html`)도 무접촉이다.
이 패스는 13차 패스가 **등재만 해 둔 유보**(`llm.rs:494-499` 의 주석이 거짓이고 그 거짓이 실제 flake 를
덮는다)도 함께 닫는다 — 그 「별개 작업」을 #123 이 이미 수행했으므로 원장 1곳 · 판정 상세 3곳의 서술을
**해소됨**으로 바꿔 적었다(발견 기록은 지우지 않는다 — #123 의 착수 근거다).
**이 패스만의 효과**는 `backend/src/llm.rs` 주석 **3행 제거**뿐이다 — 전역이 19차 패스의 `lines=2597` 에서
`lines=2646` 으로 움직인 것은 이 패스가 아니라 **#121(`b4a6b30`, AC1.5 확장)의 착지(+52 = 판정 행 안 39 +
새 파일 13)** 때문이고, 이 패스는 그 39행을 **판정하지 않고** 원장 1·4·5·7·9행의 줄 수·지문만 재고정했다
(각 행의 「자매 착지 재실측」). **그 트리의 검산은 22행 전건 재현과 행 열의 합 2,441 + 잔여 205 = 2,646** 이다.

**19차 패스(#128, `c1b39ad`)가 그보다 더 앞선 트리를 만들었다.** 18차 패스(#124)가 「분류는 다시 재지 않았다」로 넘긴 몫을
받아, **#125 가 지운 D2·D3·D4·D5 를 근거로 남아 있던 원장의 「경로 벽」 단언 다섯 자리를 실측대로 고치고**
(위 「경로 벽」 항목 · 「슬라이스 전 필수 절차」 · 「경로 벽 6파일 82행」 · 「`backend/src` 도 `deploy/k8s` 도」 ·
「다음 무인 패스의 후보 풀」), 그 벽 뒤에 갇혀 있던 **`crypto.rs` 13행**을 13차 패스의 기판정 결과대로 닫았다
(순 제거 6). **그 패스만의 효과**는 `lines=2603 files=134` → `lines=2597 files=134` 다. 원장 정정은 줄 수를
바꾸지 않는다 — 그 어긋남은 **분류**였고, 그래서 #124 가 합을 82 로 고친 뒤에는 **산술이 전부 초록인 채로
틀려 있었다**(「읽는 법」이 스스로 적은 사각지대의 실례).

**18차 패스(#124, `0fc6552`)가 `lines=2603 files=134` 트리를 만들었다.** 사람 PR **#114**(`aacd0b4`, 슬라이스 6d — AC3.5 코드 자동 분석과
사용자 편집의 충돌 처리)가 들여온 **139행 / 5파일**(순 +134) 중 **마이그레이션 `0013_feature_doc_conflicts.sql`
24행을 뺀 무인 115행**을 판정해 **순 제거 66행**을 기록했다(새 행 1개 = 충돌 해소 축 31행 · 원장 1·4·7·10행의
증분 재판정 11행). **이 패스만의 효과**는 `lines=2687 files=134` → `lines=2621 files=134`(순 제거 66)이고,
**머지 시점 트리의 실측은 `lines=2603 files=134`** 다 — 머지 직전에 자매 #122(`1bb68d0`) · #123(`77158c2`) ·
#125(`78183d2`) · #127(`df5afd5`)가 먼저 착지해 절대값이 함께 움직였다(이 패스의 순 제거 66 은 그대로다).
**전역 지문의 절대값은 여기 고정하지 않는다** — 잔여 파일의 제자리 수정만으로도 움직인다(#127 은 판정기 주석
2줄을 줄 수 그대로 고쳐 썼다). 그 트리의 검산은 **21행 전건 재현**과 **행 열의 합 2,398 + 잔여 205 = 2,603** 이었다.
**17차 패스(#117, `1289a7f`)는 `lines=2553 files=129` 를 만들었고**, 그 뒤 #114 가 `lines=2687 files=134` 로
올린 것을 18차 패스가 받았다.

**16차 행(#98)의 문면 수치는 그 패스가 #92 위로 리베이스하기 전 계산이다** — 「전역 `lines=2517`」·「`backend/tests/migrations.rs`
42」·「잔여 13파일/305」는 지금 트리에서 참이 아니다(실측: `migrations.rs` 14 — #92 는 그 파일에 주석을 더하지
않았다). 16차 행의 **범위 지문 123/`1b35c33a…` 자체는 현재 트리에서 재현된다.** 문서 편집 축(#106)이 #98 과
같은 창에 착지하며 머지 트리에서 재실측해 적어 둔 정정이고, 이 패스에서도 그대로다.

**미판정 잔여**: **8파일 / 176행**
(이 패스의 트리 기준 — 전역 `lines=2656 files=136` 에서 **행 열 밖 미판정 증분 14 를 뺀 2,642**).
전역 지문의 파일 목록에서 판정 133파일을 **집합으로 뺀** 값이며, 뺄셈과도 일치한다(176 == 2,642 − 2,466). #114 가 들여온 마이그레이션 `0013_feature_doc_conflicts.sql`
24행이 잔여에 새로 들어왔고, **#121 이 들여온 새 파일 `e2e/tests/sc01-08-succeeded-stage-rerun.spec.ts` 13행**도
같은 형태로 들어왔다 — 문서 편집 축이 「자매 슬라이스가 새 파일을 들여올 때마다 새 잔여가 생긴다」고 적어
둔 그대로다(#92 → 0009, #107 → 0010, #112 → 0011, #108 → 0012, **#114 → 0013**, **#121 → `sc01-08`**,
**#141 → `e2e/support/github-app.ts` 4행**).

**잔여는 세 몫이다**(합 7파일 · 172행) — **⑴ 사람 게이트(D1 마이그레이션) 5파일 123행**
(`0009` 30 · `0010` 30 · `0011` 25 · `0012` 14 · `0013` 24) · **⑵ 경로 벽(D6 판정기 자신) 1파일 36행**
(`tools/check-data-format-change.py`) · **⑶ 새 미판정 파일 1개 13행**
(`e2e/tests/sc01-08-succeeded-stage-rerun.spec.ts` — #121 이 들여왔다. 20차 패스가 19차와 같은 음성 대조로
확인했다: 부모 `b4a6b30` 위에 그 파일에 **주석 한 줄만** 덧붙인 probe 가 **`✅ 해당 없음`**).
**직전까지 ⑶ 에 함께 있던 무인 자유 풀 4파일 33행은 22차 패스가 새 행 23 으로 닫았다**
(`db.rs` 17 · `backend/tests/migrations.rs` 14 · `models.rs` 1 · `deploy/k8s/pvc.yaml` 1 → 합 17행).
**그 4파일은 2026-09-22 까지 ⑴·⑵ 로 잘못 분류돼 있었다** — 위 「경로 벽」 항목의 무효화 기록을 함께 읽을 것.
**무인 자유 풀은 지금 ⑶ 13행 + #121 이 연 미판정 증분 39행**이다(위 합계). 이 조건은 PR 번호에 묶이지
않는다 — **자매 모델의 슬라이스가 범위 안 파일을 다시 쓰거나 새 파일을 들여올 때마다** 증분·새 잔여가
생기고, #91·#99·#92·#107·#112 가 그 형태였다. #92·#107·#112 는 **새 파일**을 들여온 경우라 증분(판정 행 안)과
새 행(판정 행 밖)이 한 패스에 같이 열렸다.

- **`backend/migrations/0009_feature_doc_edits.sql` 1파일 / 30행** — 본문 「적용된 마이그레이션」 절의 전용 PR ·
  수동 repair · 사람 승인 게이트를 거치는 **별도 패스**다. 16차 패스(#98)가 0001~0008 을 닫았고, 0009 는 #92 로
  그 뒤에 main 에 들어가 롤아웃됐으므로 같은 게이트(suspend · scale 0 · `UPDATE` · resume)를 다시 요구한다.
  #92 는 머지 전(`9b36a98`)에 0009 를 마이그레이션 README 의 작성 규칙(문서 식별자·모듈 경로·시점 서술 금지)에
  맞춰 한 번 고쳤으므로 남은 판정은 중복성뿐이다. 이 패스가 0009 의 머리 문단(`content_hash` 불변의 이유 ·
  거부의 기록 · `scenario_index` 0-based · `source` 어휘 · `reason` CHECK)을 코드 쪽 사본들의 **정본**으로
  삼았으므로, 0009 를 판정할 때 그 문단들은 유지 후보다.
- **`backend/migrations/0010_feature_additions.sql` 1파일 / 30행** — 같은 게이트의 두 번째 몫. #107 로 main 에
  들어가 `pin deployment image` → `deploy` 브랜치로 롤아웃됐으므로 **적용된 마이그레이션**이다. 0009 와 함께
  **한 번의 repair** 로 모으는 것이 본문 1항(「한 패스에서 마이그레이션 판정을 끝까지 모아」)에 맞다 — 그
  패스를 여는 것은 사람의 결정이다. 빠진 feature 직접 추가 축이 0010 의 머리 문단(`feature_candidates` 에
  섞지 않는 이유 · `analysis_documents` 불변 · `key` 접두사 · `source` 어휘와 NULL)을 코드 쪽 사본들의
  **정본**으로 삼았으므로, 0010 을 판정할 때 그 문단들은 유지 후보다.
- **`backend/migrations/0011_feature_deletions.sql` 1파일 / 25행** — 같은 게이트의 세 번째 몫. #112 로 main 에
  들어가 `pin deployment image` → `deploy` 브랜치(`63fec7f`)로 롤아웃됐으므로 **적용된 마이그레이션**이다. 0009 ·
  0010 과 함께 **한 번의 repair** 로 모은다. feature 삭제·보존 축이 0011 의 머리 문단(보관소 원리 · 되돌린 행도
  남는다 · `content_hash` 불변 · `restore_until` 이유 · `reason` 선택 · 부분 유일 인덱스)을 코드 쪽 사본들의
  **정본**으로 삼았으므로, 0011 을 판정할 때 그 문단들은 유지 후보다.
- **`backend/migrations/0012_llm_language.sql` 1파일 / 14행** — 같은 게이트의 네 번째 몫. #108(`b1c4efe`,
  AC4.9 출력 언어)로 main 에 들어갔고 `deploy` 브랜치(`0a555cb`)가 `1289a7f` 를 고정해 롤아웃됐으므로
  **적용된 마이그레이션**이다. 0009 · 0010 · 0011 과 함께 **한 번의 repair** 로 모은다. 0012 의 머리 문단
  (`users.llm_language` 와 `analyses.llm_language` 를 두 자리에 두는 이유 = 「지금 원하는 언어」와 「이 분석이
  쓰인 언어」는 다른 사실이다 · 사용자 쪽 NOT NULL + 기본값 `'ko'` · 분석 쪽 NULL 의 뜻 · 어휘를 `CHECK` 로
  박지 않는 이유)은 **어느 패스도 아직 정본으로 지정하지 않았다** — 17차 패스는 `settings.rs` 축만 판정했다.
  0012 를 판정할 때 정본 지정부터 한다.
- **`backend/migrations/0013_feature_doc_conflicts.sql` 1파일 / 24행** — 같은 게이트의 **다섯 번째 몫**. #114
  (`aacd0b4`, 슬라이스 6d)로 main 에 들어갔으므로 **적용된 마이그레이션**이다. 0009 · 0010 · 0011 · 0012 와 함께
  **한 번의 repair** 로 모은다 — 본문 1항(「repair 비용이 크므로 한 패스에서 마이그레이션 판정을 끝까지 모아」)
  때문에 18차 패스가 이것만 따로 집지 않았다. 0013 의 머리 문단(열린 충돌은 자리당 하나인 부분 유일 인덱스 ·
  `status`/`source` 의 CHECK 어휘 · `merge_edit_id` 가 `merged` 에서만 서는 이유)은 **어느 패스도 아직 정본으로
  지정하지 않았다** — 18차 패스는 `doc_conflict.rs` 축만 판정했고, 그 판정에서 `status` 모듈 doc 이 어휘의 정본을
  **0013 의 CHECK** 로 지목했다(그 주석 자신은 0012 로 잘못 적어 제거됐다). 0013 을 판정할 때 정본 지정부터 한다.
- **경로 벽은 `tools/check-data-format-change.py` 1파일 / 36행뿐이다(D6 판정기 자신).** 이 파일은
  `SELF_PATHS` 에 **경로로** 걸리고 「주석 아닌 줄」 예외가 없어 **주석 한 줄만 고쳐도**
  `needs_review=true` 가 된다 — 주석 패스가 스스로 풀 수 없는 유일한 자리다.
  (`backend/src/pipeline.rs` 는 범위 안 주석이 0행이라 잔여에 없다.)
- **나머지 4파일 33행에는 벽이 없다 — 무인 자유 풀이다**: `backend/src/db.rs` 17 ·
  `backend/tests/migrations.rs` 14(16차 패스가 `APPLIED` 출처 주석을 +1) · `backend/src/models.rs` 1 ·
  `deploy/k8s/pvc.yaml` 1. ⚠️ `backend/tests/migrations.rs` 는 **D1 이 아니다** — D1 은
  `backend/migrations/**` 경로이고 이 파일은 그 밖이다(이름만 닮았다).
  **— 해소됨(2026-09-22 · 22차 패스 · `rct_20260922-0008` · 새 행 23).** 그 4파일 33행은 판정돼 17행이
  됐고, 「D1 이 아니다」는 판정기가 **이 패스의 실제 diff 에 `✅ 해당 없음`** 을 내면서 세 번째로 확인됐다.
  발견 기록은 지우지 않는다 — 다음 감지가 같은 자리를 다시 열지 않도록 결과만 덧붙인다.
  - **무효화된 기록 — 이 두 항목은 2026-09-22 까지 「D2 4파일 45행 + D5 `pvc.yaml` 1행도 벽」이라고 적었다.**
    그 단언은 **적힌 시점에 옳았다**(14차 패스의 음성 대조 실측이 근거다). 틀린 것은 그 패스가 아니라
    **전제가 사라진 것**이다: **#125(`78183d2`, 2026-09-22)가 판정기에서 D2(`CORE_FILES`) ·
    D3(`D3_PATTERNS`) · D4(`STORAGE_CRATES`) · D5(`is_pvc()`) 를 전부 삭제**해 규칙이 **D1 · D6 둘**만
    남았다. 18차 패스(#124)는 줄 수만 58 → 36 으로 재실측하고 **분류는 다음 계획의 몫**으로 넘겼고,
    19차 패스(#128)가 그 몫을 받아 같은 방법으로 다시 쟀다.
  - **음성 대조 재실측**(부모 트리 `0fc6552` = #124 착지 직후 main · probe = 그 위에 **주석 한 줄씩만** 덧붙인 트리):

    | probe | `check-data-format-change.py` 판정 | 근거 |
    |---|---|---|
    | `db.rs` · `crypto.rs` · `models.rs` · `backend/tests/migrations.rs` · `deploy/k8s/pvc.yaml` 5파일 | **`✅ 해당 없음`** | 히트 0 |
    | `tools/check-data-format-change.py` 1파일 | **`⚠️ 사람 리뷰 필요`** | D6 1건 |

    **19차 패스(#128) 자신이 세 번째 증거다** — 가짜 벽 뒤에 있던 `crypto.rs` 를 실제로 편집한 그 PR 이 `✅` 를 받았다.
  - **필수 status 의 이름이 바뀌었다 — 지금은 `review/manual-approval` 이다**(`/rules/branches/main` 실측,
    ruleset `updated_at 2026-09-22T12:43:47Z`). #127(`df5afd5`)이 워크플로가 붙이는 이름을
    `review/data-format` → `review/manual-approval` 로 개명했고, ruleset 의 필수 context 가 **12분 뒤에**
    따라왔다. 그 사이(12:31 ~ 12:43) 워크플로가 붙이는 이름과 ruleset 이 요구하는 이름이 달라
    **열린 PR 이 전부 `BLOCKED`** 였다(#121 · #124 · #126 의 `mergeStateStatus` 실측). 지금은 풀렸다.
    **이 자리의 이름 인용은 개명 때마다 낡는다** — 다음 감지·계획은 필수 status 이름을 원장에서 읽지 말고
    `/rules/branches/main` 에서 읽을 것. 19차 패스가 같은 자리에서 두 번(판정기 규칙 · status 이름)
    낡은 인용을 고쳤다.


**다음 무인 패스의 후보 풀은 새 파일 1개 13행 + 미판정 증분 47행(#121 의 39 + #132·#134 의 8)이다.** #114 의 무인 115행은 18차 패스(#124)가,
가짜 벽 뒤 `crypto.rs` 13행은 19차 패스(#128)가, 그 벽 뒤에 남아 있던 **4파일 33행은 22차 패스(이 패스)가**
닫았다. 지금 열려 있는 것의 대부분은 **사람 PR #121**(`b4a6b30`, AC1.5 확장)이 연 몫이다 — 새 파일
`e2e/tests/sc01-08-succeeded-stage-rerun.spec.ts` 13행(새 행이 될 몫)과, 판정 완료 범위 안 다섯 행
(원장 1·4·5·7·9행)에 연 증분 39행(**원래 행의 결과 칸 갱신**으로 닫을 몫 — 20차 패스가 줄 수·지문만
자매 착지 재실측으로 따라 적었다). #123 이 연 4행은 20차 패스의 **증분 재판정 ⑪** 이 닫았다. 여기에 더해
자매 슬라이스가 판정 완료 범위 안에 새 주석을 더하거나 새 파일을 들여올 때 또 열린다 —
실제로 **#132·#134 가 행 9·10 에 8행**을 더 열었고 그 몫은 `rct_20260922-0009` 가 받는다. **「자유 풀」의 수치는 이
트리에서만 참인 조건부 진술이다** — 다음 감지는 절대 수치를 믿지 말고 **전역 지문의 파일 목록에서 위 133파일을
집합으로 빼서** 잔여를, 행 열의 합과 「전역 − 잔여」의 차로 미판정 증분을 다시 계산해야 한다.
증분 밖에서 진짜로 남은 일은 **마이그레이션 축의 다음 사람 게이트 패스**(0009 + 0010 + 0011 + 0012 + **0013**,
합 **123행** — 16차 패스와 똑같은 suspend · scale 0 · `UPDATE` · resume 를 한 번으로 요구한다)와, 13차 패스가
후속 후보로 적어 둔 **이미 판정된 spec 12개의 Isolation 블록**(원장 3·4·5·6·7·8행에 걸치는 증분 재판정 — 한
패스로 모아서)이다. 후자는 #92 이전부터 있던 것이라 감지가 열지 않는다 — 사람이 열거나, 그 행들에 다음 증분이
들어올 때 함께 집는다. 경로 벽은 **`tools/check-data-format-change.py` 1파일 36행**뿐이고 이것만
**주석 패스가 스스로 풀 수 없다**(판정기 자신을 고치는 것도 D6 다).

**세 번째 후속 후보 — 원장 2행의 17행 묶음 재판정.** 이 패스의 증분 재판정 ④ 가 **복원 경로 ③ 의 실재를
기록했다**(PR #115 본문이 그 4행을 거의 축자로 다시 적는다). 그런데 같은 잣대를 적용하면 2026-09-18 패스가
「판단이 갈려 남긴」으로 고정한 13행(⚠️ 등록부 3행 · P1 해시 블록 3행 · `sk-`/`sk-ant-` 접두사 근거 7행)도
함께 뒤집힌다 — 그중에는 사고가 `doc-tracker/2026-08.md` 에 남은 채 유지된 것도 있다. 원장 「읽는 법」이
같은 범위의 재판정을 **원래 행의 묶음 증분 재판정**으로 정해 두었으므로, 뒤집는다면 **13 + 4 = 17행을 한
번에** 본다. 이번 4행짜리 증분이 단독으로 선례를 뒤집는 자리가 아니다. 좌표 둘은 그때 함께 본다:
`tools/check-journey-prototype.js:220` 의 `(AC4.9)` 꼬리표와 `:222` 의 「형태와 … 함께 본다」 절(⑤형 재진술).

**#114 는 착지했고(`aacd0b4`) 이 패스가 그 유입을 받았다.** 예고대로 `docs/comment-policy/` 는 바이트 하나
건드리지 않았고(부모 `e86af06` 와 tree hash 동일), `backend/src`·`backend/tests`·`frontend/src`·`e2e` 의 판정
완료 범위 안에 증분을 열고 `0013_feature_doc_conflicts.sql` 이라는 새 잔여를 들여왔다. **예고가 실제와 갈린 곳
둘을 여기 적어 둔다** — ⑴ 새 잔여는 `0013…sql` **하나가 아니라 5파일 110행**이었다(`doc_conflict.rs` 54 ·
`0013…sql` 24 · `sc03-07…spec.ts` 13 · `ResolveConflict.tsx` 10 · `backend/tests/doc_conflict.rs` 9) — 그중
무인 86행은 **새 행 21**이 받았고 `0013…sql` 24행만 사람 게이트 풀로 남겼다. ⑵ 증분은 원장 **1·4·7·10행**에
걸친 **순 +24행**이었고, 네 행 모두 줄 수·지문이 **둘 다** 어긋난 채였다 — 이 패스의 증분 재판정이 닫았다.
**#120(`5166415`, 17차 패스의 지문 재고정)의 10행 재고정은 태어날 때 이미 낡아 있었다**: #120 은 11:20:52 에,
#114 는 11:16:56 에 머지됐는데 #120 이 적은 10행 지문은 **#114 이전** 트리의 값이었다(부모에서 재현되고 현재
트리에서 안 됐다). 13·14행 재고정은 #114 무접촉이라 지금도 유효하다. **교훈은 아래 「행 지문을 재현하는 법」이
이미 말하는 것의 실증이다 — 재고정은 브랜치 base 가 아니라 머지 시점 트리에서 계산하고, 완료 기준은 절대
지문이 아니라 「부모 대비 순 제거 N행」으로 건다.**

**슬라이스 전 필수 절차 — 판정기를 돌린다.** 원장이 다음 축을 파일·행수까지 지목해도, 집기 전에
후보 트리에서 `python3 tools/check-data-format-change.py --base <main tip> --head <probe> --verbose`
를 돌려 **`✅ 해당 없음`** 을 확인한다. **규칙 목록을 이 원장에 옮겨 적지 않는다** — 판정기 출력만이
충분한 근거이고, 문면에 박아 둔 규칙 이름은 판정기가 바뀌면 **조용히 거짓이 된다.** 실제로 #125 가
D2·D3·D4·D5 를 지운 뒤 이 문단은 없는 규칙을 확인하라고 지시하고 있었다(2026-09-22, 19차 패스가 정정).
14차 패스가 원장의 후보 ①에서 이 절차로 벽을 미리 찾아냈고, 19차 패스가 같은 절차로 그 벽이 걷힌 것을
확인했다 — 절차는 그대로 유지한다.

**경합 0 · 열린 PR 3건** (2026-09-22T11:5xZ `/pulls?state=open` 전수 재실측 — #121 `feat/rerun-succeeded-stage` ·
#122 `reconcile/rct_20260922-0004-scenario-e2e` · #123 `reconcile/rct_20260922-0001-opus5-1m`, 셋 다 base `main`).
각 PR 의 `/pulls/<n>/files` 를 이 패스의 **9파일**과 대조하고 `git merge-tree` 로 3-way 를 실제로 돌려 **겹침 0**
을 확인했다. ⚠️ #121 은 `docs/doc-tracker/2026-09.md` 에서 **main 과** 충돌한다(이 패스와 무관한 선행 충돌 —
`merge-tree main refs/pull/121/head` 로 재현된다). #121 은 `worker_api.rs`(원장 1행) · `analysis.rs`(1행) ·
`backend/tests/progress.rs`(9행) · `AnalysisProgress.tsx`(7행) · `tools/check-journey-prototype.js`(2행)를,
#123 은 `llm.rs`(1행)를, #122 는 `tools/check-scenario-e2e.py`(11행)를 건드리므로 **먼저 머지되면 그 행들의
절대 지문은 움직인다** — 그래서 이 패스의 완료 기준도 절대 지문이 아니라 **「부모 `5166415` 대비 순 제거 66행」**
이고, 행 21 의 범위 지문은 이 새 파일 4개만의 값이라 자매 머지에 무관하다. **머지 직전에 행 지문을 머지 시점
트리에서 다시 계산한다**(#120 이 넘어진 자리 — 위 「#114 는 착지했고」 절).
**머지 직전 실측 결과**: #122(`1bb68d0`) · #123(`77158c2`)에 더해 계획 때 없던 #125(`78183d2`)까지 셋이 먼저
착지했고(그 뒤 #127(`df5afd5`)까지 넷), #121 은 그대로 열려 있다. 머지 시점 트리에서 21행 전건을 다시 재 보니 **움직인 것은 원장 1행 하나**
(#123 의 `llm.rs` — 602 → 606)다. #122 가 `check-scenario-e2e.py` 에 더한 1줄은 주석이 아니어서 11행은
불변이고(112/`8b41c689…` 재현), #125 가 줄인 22행은 잔여(`check-data-format-change.py`) 몫이라 행에 닿지 않는다.

그래서 **`backend/src` 도 `deploy/k8s` 도 닫히지 않았다.** `backend/src` 에는 `db.rs`·`models.rs` 가,
`deploy/k8s` 에는 `pvc.yaml` 이 남아 있다(19차 패스가 `crypto.rs` 를 닫았다). **다만 이 셋은 더 이상
규칙에 걸리지 않는다** — 위 「경로 벽」 항목의 재실측 참조. **디렉터리 단위 종료를 선언하지 않는다** —
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
>   **— 해소됨(2026-09-22 · PR #123 `77158c2` · `tbm_feature-doc-e2e-mock-policy` `rct_20260922-0001`).**
>   그 「별개 작업」이 수행됐다: 트리거 needle 을 공유 픽스처 `an_ask()` 가 담을 수 없는 고유 문자열로
>   바꿔 **레이스 자체를 없앴고**(`llm::` 모듈 5회 반복 전건 통과 · 백엔드 전 스위트 222 passed / 0 failed),
>   거짓이던 두 절을 「the variable is process-global and `stub_answer` reads it for every Stub-mode
>   ask in this binary — so the needle must be one no other test's prompt can carry」로 **정정**했다
>   (「The env writes stay inside this one test」는 실측상 참이라 남았다). 그 편집이 1행에 들여온 4행은
>   **19차 패스의 증분 재판정 ⑪** 이 닫았다(제거 3 · 유지 3). **이 항목은 더 이상 열린 결함이 아니다** —
>   재감지가 같은 자리를 다시 열지 않도록 발견 기록은 지우지 않고 결과만 덧붙인다.
> - 지문 계산 규약은 앞 블록과 같다: 원장의 **범위 지문**은 후행 개행을 **포함**하고
>   (`echo "$HITS" | sha256sum`), 모델의 **전역 지문**은 versionScript 그대로 후행 개행을
>   **제외**한다(`printf '%s'`). 같은 입력에도 두 값은 다르다.
