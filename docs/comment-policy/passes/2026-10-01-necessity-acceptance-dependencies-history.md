# 2026-10-01 — 필요성 판정 세 번째 슬라이스 (인수 시나리오 · 의존성 · 이력·삭제·설정 · API 셸 · e2e 잔여)

- **reconciler task**: `tbm_feature-doc-comment-necessity/rct_20261001-0002`
- **기준**: [README.md](../README.md) 「필요성 시험」 — *이 주석을 지우면, 이 코드를 고치러 온 사람이 틀린
  판단을 하거나 그 판단에 필요한 사실을 확인하는 데 상당한 비용을 치르는가?* 사유를 한 문장으로 댈 수
  없으면 지운다.
- **범위 선택**: 판정 칸이 `—` 인 행 중, 열린 PR #210(Web Push 슬라이스 — 22파일)과 파일이 겹치지 않고,
  마이그레이션 `.sql` 행(전용 PR·사람 repair 몫)과 `.github/workflows/` 행(수동 승인 판정기
  `data-format-review.yml` 을 품는다)이 아닌 행은 497줄이었다. 예산 400줄에 맞춰
  `backend/src/usage.rs`(41) · `backend/src/doc_conflict.rs`(32) · `backend/src/access_request.rs`(24) 세 행을
  다음 슬라이스로 남기고 나머지 18행(L 16 · E 2)을 판정했다. 주석이 0줄인 세 행(L `frontend/src/NoAccess.tsx` ·
  E `backend/src/session.rs` · E `tools/check-journey-prototype.js`)은 판정할 주석이 없다는 판정으로 닫는다.

## 판정 — 18행 400줄

| 덩어리 | 표면 | 판정 전 | 판정 뒤 | 제거 |
|---|---|---|---|---|
| 인수 시나리오 축 6파일(`backend/src/acceptance.rs` …) | L | 142 | 133 | 9 |
| 종단 의존성 축 7파일(`backend/src/dependencies.rs` …) | L | 109 | 100 | 9 |
| 이력·복원 축 4파일(`backend/src/doc_history.rs` …) | L | 29 | 26 | 3 |
| 삭제·보존 축 4파일(`backend/src/feature_delete.rs` …) | L | 15 | 15 | 0 |
| 출력 언어 설정 축 3파일(`backend/src/settings.rs` …) | L | 17 | 17 | 0 |
| API 셸 · 배포 베이스 5파일(`deploy/k8s/kustomization.yaml` …) | L | 19 | 19 | 0 |
| 봉투 암호화(`backend/src/crypto.rs`) | L | 7 | 7 | 0 |
| e2e 단독 행 여섯(`e2e/support/clock.ts` · `sc01-09` · `sc04-02` · `sc04-06` · `sc04-09` · `sc04-10`) | L | 58 | 58 | 0 |
| 화면 단독 행 둘(`frontend/src/FeatureEvidence.tsx` · `frontend/src/viewport.ts`) | L | 4 | 4 | 0 |
| 0줄 행 셋(L `NoAccess.tsx` · E `session.rs` · E `check-journey-prototype.js`) | L · E | 0 | 0 | 0 |
| 합 | | **400** | **379** | **21** |

「제거」는 줄 수 순감이다(`acceptance.rs` 모듈 머리의 빈 `//!` 1줄 포함). 비주석 diff 는 0줄이다.

### 제거 목록

| 자리 | 유형 |
|---|---|
| `backend/src/acceptance.rs` 모듈 머리 「The wire key is `acceptance_dependencies` …」 2행 + 빈 `//!` | 중복 — 같은 명제의 정본은 키를 정의하는 `backend/src/pipeline.rs` 모듈 머리(직전 패스에서 유지)다. 키를 개명하려는 사람이 읽는 자리는 그쪽이다 |
| 같은 파일 `schema` doc(비 `pub`) · `situation` doc(비 `pub`) | 코드 재진술 — 함수 이름과 두 줄짜리 본문(`norm` 쌍) |
| `backend/tests/acceptance.rs` `run_to_candidates` · `run_stage_five` doc | 코드 재진술 — 함수 이름(직전 패스의 `run_through_stage_three`·`run_stage_four` 와 같은 판정) |
| 같은 파일 `the_document_is_readable_by_its_owner_and_nobody_else` doc | 코드 재진술 — 테스트 이름 |
| `e2e/tests/sc02-04-…` `DEVELOPER_VOCABULARY` JSDoc(비 export) | 코드 재진술 — 상수 이름 |
| `backend/src/dependencies.rs` `schema` doc(비 `pub`) | 코드 재진술 — `acceptance.rs` 의 같은 자리와 같은 판정 |
| `backend/tests/dependencies.rs` `SIFTED` doc · `run_to_confirmed` · `run_trace` doc | 코드 재진술 — 이름과 바로 아래 값. `SIFTED` doc 은 결과(무엇이 틀리게 되는가)를 말하지 않는다 |
| 같은 파일 · `e2e/tests/sc02-07-…` 「남의 분석은 존재하지 않는다.」 각 1행 | 코드 재진술 — 바로 아래 `404` 단정. 404 를 고르는 이유(존재 비노출)는 응답을 만드는 쪽과 다른 테스트 doc 이 말한다 |
| `frontend/src/FeatureDependencies.tsx` `tick` JSDoc | 코드 재진술 — 폴링 효과가 `setTick` 을 부르고 읽기 효과가 `tick` 에 의존한다는 것은 두 효과가 말한다 |
| `e2e/tests/sc02-07-…` 「화면에 그리는 것이 아니라 파일로 나간다」 · 「형식이 스스로를 설명한다」 | 중복·코드 재진술 — 파일 머리 판정 기준 문단과 바로 아래 단정 |
| `backend/src/doc_history.rs` `scenarios_with` doc(비 `pub`) | 코드 재진술 — 함수 이름 |
| `backend/tests/doc_history.rs` `approved_edit` doc | 코드 재진술 — 함수 이름 |
| `e2e/tests/sc03-08-…` `editWithHelp` JSDoc(비 export) | 코드 재진술 — 함수 이름과 본문의 두 요청 |

### 유지 목록 (묶음마다 필요 사유 한 문장)

**인수 시나리오 축** (L 133)
- `acceptance.rs` 모듈 머리 요약 · 두 번 부르는 이유 · 「모순 규칙은 코드」 — 한 번 호출로 합치면 「보강」을 견줄 *이전*이
  없어지고, 규칙을 프롬프트로 옮기면 모델이 굽히고 스텁과 실모델이 갈린다.
- `MAX_FEATURES`·`MAX_SCENARIOS` doc — 상한의 근거(실행 비용 · 비개발자가 끝까지 읽는 길이)라, 화면 길이를 이유로
  상한을 바꾸는 판단을 막는다.
- `Subject`·`Scenario`·`Feature`·`derive`·`features`·`detail`·`is_test_path` 요약과 `SOURCE_LOGIC` doc — `pub` 요약
  (정책). `is_test_path` 본문은 판정 자리를 한 곳으로 묶어 둔 이유이고, `SOURCE_LOGIC` 본문은 두 목록으로 나누는
  「정리」가 문서의 서술 순서를 깨는 이유다.
- `scenarios_for` doc — 지시문을 믿고 답 검증을 지우면 트리에 없는 근거 경로가 문서에 들어간다.
- `merge` doc · 같은 상황·같은 결말 인라인 — 빈 갈래(`Some(_) => {}`)를 버그로 보고 고치거나 다툼 있는 로직
  시나리오를 버리면 비개발자 문서에서 그 상황이 사라진다.
- `stub_logic`·`stub_tests` doc · 두 번째 기준 인라인 — 상수 스텁은 배선이 끊겨도 통과하고, 지어낸 근거 경로는
  이 단계가 막는 바로 그 실패다(충실도 경계).
- 빈 테스트 목록 인라인 — 「보강할 것 없음」과 「보강 실패」를 합치거나 없는 입력으로 모델을 부르면 사용자 돈을 쓴다.
- `backend/tests/acceptance.rs` 모듈 머리 — 단위/통합 경계와, 픽스처를 손으로 쓰면 워커가 실제로 내는 모양과
  갈린다는 공유 규약(직전 패스의 테스트 머리와 같은 사유).
- `SIFTED` · `tree` doc — 근거 경로가 스텁 트리에 있어야 단계의 근거 검사가 공허하지 않고, 기대값과 단계 검증이
  한 출처를 쓴다.
- 첫 읽기 실체화 인라인 · 「The gate is the queue's」 · 거부 재큐잉 금지 · 두 번째 확정 재개방 doc — lazy seed·
  큐 게이트·커버리지 술어를 모르면 헬퍼를 「단순화」하거나 테스트를 다른 경로의 검증으로 바꾼다.
- 404 인라인 둘(아직 안 돎 · 남의 id) — 200 빈 문서·403 으로 바꾸면 상태 구분과 존재 비노출이 깨진다.
- `e2e/support/acceptance.ts` 머리 — 셋업만 공유하고 단정은 spec 이 소유한다는 규약과, `testDir` 밖이라 AC↔spec
  매칭 단위로 세지 않는다는 사실(옮기면 `check-scenario-e2e.py` 가 센다).
- 같은 파일 export JSDoc 셋(`acceptanceOf` · `signInWithCredentials` · `runToAcceptance`) — export 요약 1줄(정책)과,
  셋업을 화면으로 걷지 않는 이유 · 임대는 호출자가 쥔다는 규약.
- 같은 파일 실체화 · 확정이 5단계를 연다는 인라인 — 확정을 단정으로 바꾸거나 읽기를 지우면 걷기가 멈춘다.
- `sc02-01`·`sc02-04` 머리 판정 기준 문단 · Isolation 블록 · 새로고침 · 「기능 둘을 확정」 — 공유 규약 블록과,
  기대값 상수화·한 기능 확정으로 바꾸면 성질이 관측되지 않는다는 판정 기준.
- `FeatureAcceptance.tsx` 머리 1~2행 · 서버 출처 · `evidenceOf` · 함께 읽기 · `Appbar` — 목업 매핑 선언
  (`check-mockup-render.py`), 클라이언트 상태 도입, M3B 템플릿 리터럴 함정, 읽기 분리, 빈 `icon-btn` 제거가
  틀리는 이유(직전 패스의 같은 자리와 같은 사유).

**종단 의존성 축** (L 100)
- `dependencies.rs` 모듈 머리 — 파이프라인 단계를 더하지 않고 요청 행이 게이트이자 상태라는 설계라, `STAGES` 에
  단계를 더하는 「정리」를 막는다.
- `MAX_ITEMS` · `CATEGORIES` · `request_status` doc — 상한 근거, 분류 7종의 단일 정의(마이그레이션 CHECK·화면 칩
  동기화), 중간 상태가 없는 이유(claim 배타성 — 리스가 끊긴 요청은 `queued` 로 남는다).
- `Item` doc — `evidence: None` 을 흠결로 보고 채우는 판단을 막는다.
- `category_rank` doc — 순서를 바꾸면 화면 칩 순서와 저장 `seq` 가 함께 바뀐다.
- `items`·`document`·`detail`·`category_for`·`derive` 요약 — `pub` 요약. `items` 본문은 해석 지점이 하나여야 화면과
  역방향 질의가 갈리지 않는다는 불변식, `category_for` 본문은 분류는 모델이 정하고 이 함수는 결정성 자리라는 경계다.
- `item_name` doc — 파일 이름으로 바꾸면 `mod.rs`·`index.ts` 같은 이름이 서로 다른 의존성을 한 이름으로 합쳐 역방향
  질의가 틀린 기능을 돌려준다.
- `stub_dependencies` doc — 상수 스텁은 배선이 끊겨도 통과한다(충실도 경계).
- `backend/tests/dependencies.rs` 머리 — 테스트 머리 공유 규약. 「Run stage 5 …」 인라인 — 의존성 테스트가 5단계를
  돌리는 이유(커버리지 술어가 쉬어야 한다)를 모르면 무관한 셋업으로 보고 지운다.
- `urlencoding` doc — `char` 단위로 바꾸면 비ASCII 이름이 UTF-8 바이트가 아니라 코드 포인트로 인코딩된다.
- 일곱 칸 · 공유 항목 이름 · 사유 없는 실패 인라인 — 빈 분류를 걸러내거나, 기대 이름을 상수로 박거나, 실패 사유
  강제를 느슨하게 하는 판단을 막는다.
- `FeatureDependencies.tsx` 머리 1~2행 · `CATEGORY_LABELS` · `POLL_MS` · 폴링 효과 · `Graph` · `LOADING` · `Appbar` —
  목업 매핑 선언, 서버 키 동기화(키 없는 라벨은 빈 여덟째 분류를 그린다), 폴링 주기·조건의 근거, 고정 그림은
  모든 기능에 같은 그림을 그린다는 충실도, 편차 카피를 한 자리에 모은 규약, 두 버튼이 같은 곳으로 가는 것이
  의도라는 사실.
- `e2e/support/dependencies.ts` 머리 · `CATEGORIES` 사본 · export JSDoc 둘 — 셋업 공유 규약, 서버 정의의 사본이
  넓어지지 않고 실패해야 한다는 동기화, export 요약 1줄(정책).
- `sc02-05`·`sc02-06`·`sc02-07` 머리 판정 기준 · Isolation 블록 — 공유 규약 블록과, 기대값을 상수화하거나 화면 JSON
  표시를 export 로 보는 판단을 막는 기준. `sc02-06` 「이 질의에 화면은 없다」 — 목업에 없는 화면을 지어내지 않는다.
- `sc02-05` 분류 수 · 새로고침, `sc02-06` 두 기능 · 다른 차원 · 7종 밖, `sc02-07` 추적 전 기능 제외 인라인 — 상수
  기대값·한 기능 질의·자유 텍스트 분류·빈 항목 포함으로 바꾸면 단정이 공허해지거나 규칙이 사라진다.

**이력·복원 축** (L 26)
- `doc_history.rs` 모듈 머리 · `AUTO` · `standing_edits` · `current_restore` 요약 — doc 주석 수준. `AUTO` 본문은 행이
  아니라 자리라서 예약어라는 사실이다.
- `RESTORE_SOURCE` doc — 복원 출처를 `user_llm` 으로 바꾸면 시점을 고른 주체가 틀리게 기록된다.
- 항목 주소 라우트 인라인 — 미리보기를 화면 상태로 옮기면 새로고침이 미리보기를 잃는다.
- `steps` doc — 세대에 들지 못한 편집을 버리는 「정리」가 이력에서 편집을 조용히 지운다.
- `walk` doc — 반환 맵이 「그 사건 직후에 서 있던 편집」이라는 의미론이 튜플 타입에서 드러나지 않는다.
- `backend/tests/doc_history.rs` 머리 — doc 주석 수준.
- `sc03-08` Isolation · `REVISION` · 절 배너 여덟 — 공유 규약, 더블의 셋째 리비전이 무엇을 바꾸는지(EXT-03),
  시나리오 단계와 spec 블록의 대응(배너는 직전 패스의 `── 탭 N ──` 과 같은 공유 규약).
- `FeatureHistory.tsx` 머리 · 서버 계산 인라인 — 세 단계를 한 화면에 접은 편차와, 미리보기를 클라이언트에서
  짐작하면 「미리 본 것」과 「실제로 되는 것」이 갈린다는 불변식.

**삭제·보존 축** (L 15)
- `feature_delete.rs` 모듈 머리 · `RETENTION_DAYS` · `PreviousDeletion` · `overlay` 요약 — doc 주석 수준.
  `RETENTION_DAYS` 본문은 기한이 행에 적힌다(값을 바꿔도 기존 삭제의 기한은 안 바뀐다), `overlay` 본문은 추가 겹침
  뒤·편집 겹침 앞이라는 순서 제약이다.
- `restore` doc — 409 를 404 로 바꾸면 「지금 못 함」과 「없음」이 섞인다.
- 앞선 삭제 doc — 후보 거부 이월과 같은 동률 규칙(`rowid`)을 모르면 초 단위 동률에서 다른 삭제를 고른다.
- `current_document` doc — 편집 겹침을 얹지 않는 이유(이름·키 불변)를 모르면 겹침을 더해 비용만 늘린다.
- `backend/tests/feature_delete.rs` 머리 · `sc03-05`·`sc03-06` 임대 포인터 — doc 주석 수준 · 공유 규약.

**출력 언어 설정 축** (L 17)
- `settings.rs` 모듈 머리 · `llm_language` · `analysis_language` — 모르는 값을 기본으로 읽는 이유(행은 사용자 것,
  어휘는 코드 것)와, API 가 직접 부르는 LLM 호출도 분석의 고정 언어를 쓴다는 규칙 · 설정 이전 분석의 `None`.
- `sc04-14` 머리 — 스텁이 고정 문서를 답해 언어가 관측되지 않으므로 기록된 설정을 단정한다는 충실도 경계와,
  프롬프트 문면 단정의 소관(`backend/src/llm.rs`).

**API 셸 · 배포 베이스** (L 19)
- `util.rs` 모듈 머리 · `oauth_state` — `pub` 요약과, 접두사가 비밀이 아니고 CSRF 는 전체 문자열 대조에 기댄다는
  보안 불변식(접두사를 신뢰 근거로 쓰는 변경을 막는다).
- `main.rs` 시그널 핸들러 인라인 — PID 1 이 핸들러 없는 시그널을 버려 `Recreate` 롤아웃 내내 다운타임이 된다는
  실패 모드의 함정.
- `error.rs` 머리 — 자격증명 비노출 불변식(`Internal` 상세는 호출자 입력·상류 본문으로 짜지 않는다).
- `state.rs` 머리 — 모듈 머리(정책).

**봉투 암호화** (L 7)
- `crypto.rs` 머리 · `Envelope` · `open` — 평문·DEK 비영속과 best-effort 소거, 저장 필드가 KEK 없이 아무것도 드러내지
  않는다는 불변식, 변조·틀린 KEK 가 쓰레기가 아니라 오류라는 계약.

**e2e 단독 행 여섯** (L 58)
- `e2e/support/clock.ts` `afterSecond` — export 요약과, `startedAt` 초 단위 저장 때문에 재실행을 못 보는 함정.
- `sc01-09`·`sc04-02`·`sc04-09` 임대 포인터 · `sc04-02` 판정 기준 · `ALL_REPOS` 출처 · 큐 비우기 — 공유 규약, 무엇을
  재는가, 더블의 설치 범위가 어디서 오는가, 워커 타이밍에 흔들리지 않게 하는 셋업 이유.
- `sc04-06` 머리 셋 · `fitsOneHandedWidth` · 세션 시작점 · (a)~(d) 단계 표지와 그 인라인 — 관측 기준(폭에 따른
  접힘), 네 단계를 질러가지 않는 이유, 진입점이 의존성 라우트에만 선다는 함정, 재마운트 뒤 다시 접힘, 추적 헬퍼의 소유.
- `sc04-09` `count` · 음성 대조 둘 · 홈 진입 두 줄 · 홈 마운트 단정 — ICU 차이, 「추정을 실측으로 부르지 않는다」·
  「총합 복사가 아니다」의 음성 대조, 홈은 주소가 없다는 공유 규약, 두 실패를 가르는 단정 순서.
- `sc04-10` 머리 · 사용자 배너 둘 · 쓰기 경로 전수 · 「남의 자원」 못박기 — 두 쿠키 항아리, 목록 비노출과 직접
  주소 거부가 다른 관측이라는 판정 기준, 조회만 막는 것은 격리가 아니라는 것, 404 가 없는 자원이 아니라는 대조.

**화면 단독 행 둘** (L 4)
- `FeatureEvidence.tsx` 머리 — 목업 매핑 선언.
- `viewport.ts` `WIDE_QUERY`·`useWideViewport` · 첫 렌더 뒤 뒤집힘 인라인 — `index.css` 브레이크포인트와의 동기화,
  export 요약, 효과 등록 전에 쿼리가 뒤집히는 경합.

**0줄 행 셋** (L 0 · E 0)
- L `frontend/src/NoAccess.tsx` · E `backend/src/session.rs` · E `tools/check-journey-prototype.js` — 현재 지문(빈 집합)에
  판정할 주석이 없다. 옛 기준 전건 제거 뒤 0행으로 남은 등재 행이고, 새 주석이 들어오면 지문이 움직여 `—` 로 돌아간다.
