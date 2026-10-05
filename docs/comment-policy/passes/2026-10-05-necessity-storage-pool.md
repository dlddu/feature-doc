# 2026-10-05 — 필요성 판정 (저장소 풀 · 마이그레이션 테스트 · 행 타입 · PVC)

- **reconciler task**: `tbm_feature-doc-comment-necessity/rct_20261005-0001`
- **기준**: [README.md](../README.md) 「필요성 시험」 — *이 주석을 지우면, 이 코드를 고치러 온 사람이 틀린
  판단을 하거나 그 판단에 필요한 사실을 확인하는 데 상당한 비용을 치르는가?* 사유를 한 문장으로 댈 수
  없으면 지운다.
- **범위 선택**: 판정 칸이 `—` 인 행은 L 표의 이 한 행뿐이었다(D · E 표는 미판정 0). 예산 400줄보다 작게
  끝나는 사유는 남은 대상이 없기 때문이다. 직전 슬라이스들이 이 행을 뺀 사유(열린 PR #223 과
  `backend/tests/migrations.rs` 파일 겹침)는 #223 머지로 사라졌다. `.sql` 파일은 이 행에 없어 「적용된
  마이그레이션의 주석」 절(전용 PR · 사람 repair)의 대상이 아니다.
- 옛 기준(복원 경로 넷)의 판정 기록은 [2026-09-22-free-pool-storage-axis.md](2026-09-22-free-pool-storage-axis.md)에
  있다. 그 파일은 고치지 않았다 — 당시 사실의 기록이다.

## 판정 — 1행 17줄

| 덩어리 | 표면 | 판정 전 | 판정 뒤 | 제거 |
|---|---|---|---|---|
| 저장소 풀 4파일(`backend/src/db.rs` 13 · `backend/tests/migrations.rs` 2 · `backend/src/models.rs` 1 · `deploy/k8s/pvc.yaml` 1) | L | 17 | 17 | 0 |

비주석 diff 는 0줄이다. 바뀐 소스는 `backend/src/db.rs` 의 `///` 3줄뿐이고(아래 「틀린 주석 — 고침」), 그 doc
주석에는 코드 블록이 없어 doctest 가 생기지 않는다. `backend/tests/migrations.rs` 의 `APPLIED` 오라클 값과
`backend/migrations/*.sql` 은 건드리지 않았다.

### 제거 목록

없다. 이 범위의 재진술·경위 주석은 옛 기준 패스가 이미 걷었고(순 제거 16), 남은 17줄은 아래 유지 목록의
사유가 줄마다 선다.

### 틀린 주석 — 고침

| 자리 | 무엇이 틀렸나 | 고친 문장 |
|---|---|---|
| `backend/src/db.rs` `connect` doc 의 `journal_mode` 문단 끝 3줄 | 「sqlx 는 0.8 부터 `journal_mode` 를 건드리지 않고 0.7 까지는 WAL 이 기본이었다」 — 기본값이 바뀐 릴리스는 0.6.1 이다. 상류 `sqlx-core/src/sqlite/options/mod.rs` 는 v0.6.0 까지 `journal_mode = WAL` 을 기본 pragma 로 넣고, v0.6.1 부터 「Don't set `journal_mode` unless the user requested it」으로 비워 둔다(v0.6.2 · v0.6.3 · v0.7.4 · 이 레포가 잠근 v0.8.6 모두 같다) | 「sqlx has left `journal_mode` untouched by default since 0.6.1 (it defaulted to WAL through 0.6.0), so relying on that default would silently reintroduce WAL if a version bump moved it again.」 |

같은 doc 의 `synchronous` 문단(「sqlx 0.8 does not emit the pragma unless asked」)은 v0.8.6 소스(`synchronous`
pragma 를 `None` 으로 둔다)와 맞아 고치지 않았다. 문단의 결론(「기본값은 릴리스 사이에 움직인 적이 있으니 명시한다」)은
고친 뒤에도 그대로 성립한다 — 움직인 시점만 틀렸다.

### 유지 목록 (묶음마다 필요 사유 한 문장)

**저장소 풀** (L 17)
- `backend/src/db.rs` · `backend/tests/migrations.rs` · `backend/src/models.rs` 모듈 머리 `//!` 각 1줄 — doc 주석
  수준(정책 「유지 대상」)이라 사유를 따로 대지 않는다.
- `backend/src/db.rs` `connect` doc 의 `journal_mode` 문단 6줄 — 지우면 `.journal_mode(Delete)` 를 기본값과 같은
  군더더기로 보고 걷거나 동시성을 이유로 WAL 로 바꾸는데, DB 파일이 네트워크 파일시스템(EFS) 위에 있어 WAL 의
  `-shm` 공유가 보장되지 않는다는 사실은 이 레포의 다른 어느 문서·매니페스트에도 적혀 있지 않다.
- 같은 doc 의 `synchronous` 문단 5줄과 두 문단을 가르는 빈 `///` 1줄 — 지우면 `.synchronous(Full)` 을 SQLite
  기본값의 중복으로 보고 걷어 내구성을 다시 드라이버 기본값에 묶거나, 롤백 저널에서 `NORMAL` 이 커밋된
  트랜잭션의 전원 손실 생존을 보장하지 않는다는 것을 모른 채 속도를 이유로 낮춘다. 빈 줄은 rustdoc 의 문단
  경계다.
- `backend/tests/migrations.rs` 「An `ALTER TABLE` that silently went missing would only surface at runtime.」 —
  지우면 테이블 존재 단정 바로 뒤의 컬럼 단정이 왜 `analyses` 의 다섯 컬럼만 보는지 알 수 없어 중복으로 보고
  걷는다: 그 다섯은 `0004_analysis_stages.sql` 이 `ALTER TABLE` 로 덧붙인 컬럼이라 테이블 존재 단정으로는 빠진
  것이 잡히지 않는다.
- `deploy/k8s/pvc.yaml` 「SQLite is single-writer; one RWO volume backs the single app replica.」 — 지우면
  `accessModes` 만 고치러 온 사람이 무중단 롤링을 위해 `ReadWriteMany` 로 넓히는 판단을 이 파일 안에서 막을 것이
  없다(같은 제약을 말하는 `deploy/k8s/deployment.yaml` 의 `Recreate` 주석은 그 파일을 고칠 때만 읽힌다 — 그
  주석도 2026-10-04 판정에서 같은 사유로 유지됐다).
