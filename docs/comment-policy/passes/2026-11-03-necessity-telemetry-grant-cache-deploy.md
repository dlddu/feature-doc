# 2026-11-03 — 필요성 판정 (OpenTelemetry 2파일 · heartbeat 접근 캐시 · 워커·더블 배선 증분 5줄)

- **reconciler task**: `tbm_feature-doc-comment-necessity/rct_20261101-0001`
- **기준**: [README.md](../README.md) 「필요성 시험」 — 지우면 이 코드를 고치러 온 사람이 틀린 판단을 하거나 확인에 상당한 비용을 치르는가. 사유를 한 문장으로 댈 수 없으면 지운다.
- **범위 선택**: 판정 칸이 `—` 인 L 행 3개 전부(telemetry 46줄 · grant_cache 14줄 · 워커·더블 배선 축 증분 5줄 — 합 65줄 판정, 예산 400 이하). 남은 대상이 없어 예산보다 작게 끝난다. D · E 표는 미판정 0. 열린 PR 0건이라 겹치는 파일이 없다. `.sql` 파일은 없어 「적용된 마이그레이션의 주석」 절의 대상이 아니다.

## 판정

| 덩어리 | 표면 | 판정 전 | 판정 뒤 | 제거 |
|---|---|---|---|---|
| `backend/src/telemetry.rs` · `backend/tests/telemetry.rs` | L | 49 (#238~#241 로 늘어난 뒤 실측) | 46 | 3 |
| `backend/src/grant_cache.rs` | L | 14 | 14 | 0 |
| 워커·더블 배선 축 7파일 (증분 5줄만) | L | 172 | 172 | 0 |

### 제거 목록

- `backend/src/telemetry.rs` 모듈 머리 `//!` 3줄(빈 `//!` 1줄 포함) 「The worker's job is the unit worth following … trace of its own.」 — 같은 명제(작업 하나를 루트로 삼아 단계·API 호출이 한 트레이스를 공유한다)를 `run_job` 의 doc 주석이 이미 적고 있어, 지워도 이 코드를 고치러 온 사람이 틀릴 판단이 없다.

### 유지 목록 (묶음마다 필요 사유 한 문장)

- 모듈 머리 `//!` 나머지(요약 · opt-in 동작) — doc 주석 수준(정책 「유지 대상」).
- 「Why an SDK when OBI already traces…」 단락 — 지우면 OBI 가 있으니 SDK 를 걷자는 판단을 막을 것이 없는데, Traefik 이 헤더를 알파벳 순으로 써 `traceparent` 가 1024바이트 창 밖으로 밀린다는 사실은 다른 문서·매니페스트에 없다.
- `PROBE_PATHS` 위 2줄 — 지우면 유휴 claim 폴링에 span 을 달아 트래픽 대부분을 노이즈로 만든다.
- `init` doc 의 `shutdown` 호출 · 환경 변수 우선순위 — 지우면 종료 시 마지막 배치가 유실된다.
- 서버 span 이름 규칙(매칭된 라우트) — 지우면 경로의 ID 로 span 이름이 매번 달라진다.
- `send`/`send_external` 의 무트레이스·`traceparent` 미전파 — 지우면 클러스터 밖(GitHub)으로 trace context 가 새거나 유휴 폴링이 트레이스를 만든다.
- LLM span doc 의 「never the prompt, the response or the key」 — 자격증명·프롬프트 비노출 불변식이 그 자리에서 지켜져야 하는 이유.
- 나머지 `run_job`/`run_stage`/경로 정규화 doc 요약 — doc 주석 수준.
- `backend/tests/telemetry.rs` 머리 · 긴 헤더 뒤 traceparent 테스트 앞 2줄(미들웨어가 존재하는 이유) · stand-in API doc — 테스트 의도를 이름만으로 알 수 없다.
- `backend/src/grant_cache.rs` 머리 단락 — 지우면 TTL 만큼 늦게 알려지는 대가, 성공만 기억한다는 점, 설치 row 는 매번 읽는다는 점, 프로세스별 메모리(API 단일 replica 전제)를 몰라 캐시를 공유 저장소로 옮기거나 실패도 기억하게 바꾼다. 나머지 doc 2개는 요약 수준.
- 증분 5줄 — `deployment.yaml` 3줄: 공개 설정이 Secret 밖에 있는 이유와 worker 가 읽지 않는다는 사실(지우면 worker 에도 envFrom 을 맞추려 한다); `secret.yaml.example` 2줄: client id 의 발급자 겸용(지우면 키를 잘못 옮긴다)과 두 키가 ConfigMap 에 사는 위치.
