# 판정 상세 — 무인 자유 풀 4파일 (저장 계층 · 마이그레이션 테스트) · 2026-09-22

reconciler task `rct_20260922-0008`(모델 `tbm_feature-doc-comment-redundancy`) · 21차 패스.
19차 패스(#128)가 가짜 벽을 걷어 내며 **「벽은 걷혔지만 판정 자체가 없다 … 그 축과 함께 볼지
따로 볼지는 다음 계획에서 정한다」**로 명시 인계한 **4파일 33행**이 이 패스의 전부다.

> **이 문서의 측정 기재값은 PR head 의 base 에 묶인다.** 아래 「검증」·「이 패스가 병합되면」의
> 부모 SHA·테스트 수·게이트 결과는 그때의 base 에서 잰 값이다. 자매 착지로 base 를 올리면
> **원장 행뿐 아니라 이 파일의 수치까지 같이 다시 잰다.** 현재형으로 쓴 「현재 main」·「부모」·
> 「측정점」의 SHA 는 전부 그 base 여야 하고, 의도된 이력(세 base 열거 · 사람 PR 귀속)은 예외다.

| 파일 | 전 | 후 | 순 제거 | 요지 |
|---|---|---|---|---|
| `backend/src/db.rs` | 17 | **13** | 4 | `connect` 요약 2행이 바로 아래 빌더 호출의 번역 · `replicas: 1`/`Recreate` 문장은 매니페스트와 마이그레이션 README 가 그대로 말한다 |
| `backend/tests/migrations.rs` | 14 | **2** | 12 | 모듈 머리 3 → 1 · `APPLIED`/`PRE_CLEANUP` 출처 블록 7행이 `backend/migrations/README.md` 의 **축자 사본** · `every_migration_file_is_pinned` doc 2행 |
| `backend/src/models.rs` | 1 | **1** | 0 | 모듈 머리 `//!` 1행 — 유지 조항 |
| `deploy/k8s/pvc.yaml` | 1 | **1** | 0 | **판단이 갈려 남긴 것 1건**(아래) |
| 합 | **33** | **17** | **16** | |

## `backend/tests/migrations.rs` 는 D1 이 아니다 — 무인 경로를 이 PR 자신으로 쟀다

19차 패스가 probe 로 확인한 것을 이 패스는 **실제 diff 로** 확인했다. 현재 main(`95d3395`) 대비
이 브랜치로 `python3 tools/check-data-format-change.py --base origin/main --head HEAD --verbose` 를
돌리면 **`✅ 해당 없음 (review/manual-approval = success)`** 이다(검사한 파일 `backend/src/db.rs` ·
`backend/tests/migrations.rs`). D1 은 `backend/migrations/**` 경로이고 이 테스트 파일은 그 밖이라,
`.sql` 바이트를 한 글자도 건드리지 않는 이 패스는 sqlx 체크섬·수동 repair 게이트와 무관하다.

`APPLIED` 배열과 `PRE_CLEANUP` 배열의 **값은 한 글자도 바꾸지 않았다** — 지운 것은 그 위의 주석
블록뿐이다.

## `backend/src/db.rs` — 제거 4행

### ① 이름·본문을 영어로 옮긴 `///` 요약 2행 + 딸린 빈 `///` 1행

```
-/// Opens (creating if absent) the SQLite database at `database_url`, enables
-/// foreign-key enforcement, and applies all pending migrations.
-///
```

바로 아래 본문이 `.create_if_missing(true)` · `.foreign_keys(true)` ·
`sqlx::migrate!("./migrations").run(&pool)` 로 **세 절을 순서대로 그대로 말한다**. 12차 패스가 세우고
15차 패스가 `util.rs` 에서 그대로 적용한 판정 규칙 — 「정책이 유지하라는 `pub` 항목의 **요약 1줄**은
*요약*이지 *이름의 번역*이 아니다」 — 의 n번째 적용이다. 파일의 지도 역할은 모듈 머리
`//! SQLite connection pool + migrations.` 가 이미 한다(유지).

### ② 매니페스트 재진술 1행 (재랩으로 2행 → 1행)

```
-/// on a version bump. The Deployment runs `replicas: 1` with strategy
-/// `Recreate`, so there is only ever one writer.
+/// on a version bump.
```

- ① `deploy/k8s/deployment.yaml:8` 이 `replicas: 1`, 같은 파일 `:9-11` 이 `strategy.type: Recreate` 다.
- ② `backend/migrations/README.md:113` 이 「`featuredoc`은 `replicas: 1` + `strategy: Recreate` + RWO
  볼륨이라 동시에 두 파드가 붙지 않는 것이 전제입니다」로 **같은 명제를 같은 낱말로** 적는다.

WAL 문단의 논거(EFS 볼륨 · `-shm` 공유 · sqlx 0.8 기본값 변동)는 어느 복원 경로에도 없어 **전부
유지**했다 — 정의가 이름 붙인 「저장소 제약의 함정」이다. `synchronous=FULL` 문단도 같은 이유로 전건
유지다.

## `backend/tests/migrations.rs` — 제거 12행

### ③ 모듈 머리 3행 → 1행

```
-//! Verifies that `db::connect` applies the migrations and creates the schema —
-//! and that the migrations that have already been applied somewhere are never
-//! edited again.
+//! Migration application, schema shape, and applied-migration immutability.
```

뒤 두 줄은 `applied_migrations_are_never_edited` 라는 **테스트 이름 그 자체**(①)이고,
`backend/migrations/README.md:5` 의 「**한 번 배포된 `.sql` 파일은 한 글자도 고치지 않습니다**」(②)가
같은 명제의 정본이다. 「테스트 이름을 열거한 파일 머리」는 12·13·14차 패스가 같은 유형으로 이미
세 번 걷었다. 유지 조항대로 **한 줄짜리 지도**만 남겼다.

### ④ `APPLIED` 출처 블록 4행 — 마이그레이션 README 의 축자 사본

```
-// Pins the file bytes accepted by main at PR #98 — a comment-only pass whose
-// production `_sqlx_migrations` repair was done by hand before the merge.
-// This SHA-256 guard is not evidence of deployed DB repair; sqlx stores SHA-384.
-// See migrations/README.md for provenance and the release preflight.
```

| 지운 문장 | 복원 경로 |
|---|---|
| PR #98 의 커밋 바이트를 고정한다 · repair 를 사람이 머지 전에 했다 | ③④ — PR #98 본문과 그 커밋 메시지가 그 패스의 경위다 |
| 「SHA-256 고정값은 배포 DB repair 의 증거가 아니다 · sqlx 는 SHA-384 를 저장한다」 | ② — `backend/migrations/README.md:26-27` 이 **같은 두 문장**을 적는다 |
| 「출처와 릴리스 preflight 는 migrations/README.md 를 보라」 | ② — 링크만을 위한 문장은 남기지 않는다(정책 본문 「유지 대상 · doc 주석」) |

### ⑤ `every_migration_file_is_pinned` 의 `///` 2행

```
-/// The pin above is only as good as its coverage: a new migration that nobody adds
-/// to `APPLIED` is unprotected from the moment it ships.
```

① 함수 이름이 문장 전체이고, ② `backend/migrations/README.md:20-21` 이
「`every_migration_file_is_pinned`는 목록에서 빠진 SQL 파일을 검출합니다」로 같은 말을 한다.
`#[test]` 함수는 `pub` 항목이 아니므로 요약 1줄 유지 조항의 대상도 아니다.

### ⑥ `PRE_CLEANUP` 출처 블록 3행

```
-// SHA-384 of the six files changed by PR #44, from its parent
-// 636ee771dd404ad7cf383d1bfadcc5036ab7fac6. These are source fixtures, not
-// observations of a production database. Migration 0002 did not change.
```

`backend/migrations/README.md:29-32` 이 **PR #44 · 커밋 `4aa5590b…` · 부모
`636ee771dd404ad7cf383d1bfadcc5036ab7fac6` · 「0001·0003~0007의 주석을 바꿨습니다」(= 여섯 파일,
0002 제외)** 를 전부 적고, 같은 문서 `:36-37` 이 「임시 DB의 과거 해시는 이전 파일 바이트에서 계산한
값이며 실제 운영 DB를 수집한 값이 아닙니다」로 **「source fixtures, not observations」를 축자로**
적는다. 세 문장 전부 ② 다.

### ⑦ 리스 컬럼 주석 2행 → 1행

```
-    // The worker's lease columns are what make a claim reclaimable (AC4.5); an
-    // `ALTER TABLE` that silently went missing would only surface at runtime.
+    // An `ALTER TABLE` that silently went missing would only surface at runtime.
```

앞 절은 ① `lease_expires_at` · `claimed_by` 라는 **컬럼 이름 자체**와 ②
`docs/prd/04-platform.md` AC4.5(워커 워크로드 분리 · 큐 비동기 실행)로 복원된다. 남긴 뒷절은
**이 단정이 왜 여기 있는지**(마이그레이션에서 `ALTER TABLE` 이 조용히 빠지면 런타임에서야 드러난다)
로, 정의가 이름 붙인 「실패 모드의 함정」이다.

## 판단이 갈려 남긴 것 — 1건

**`deploy/k8s/pvc.yaml:8`**
`# SQLite is single-writer; one RWO volume backs the single app replica.`

뒷절(`one RWO volume backs the single app replica`)은 **같은 파일의 `accessModes: ReadWriteOnce` 와
`deploy/k8s/deployment.yaml:8` 의 `replicas: 1`** 이 그대로 말한다(①). 그러나 앞절
「SQLite is single-writer」 — **왜 RWO 여야 하는가** — 는 복원 경로 넷 어디에도 없다.
저장소 문서를 전수 grep 하면 `single-writer` 0히트이고, `backend/migrations/README.md:113` 은
「RWO 볼륨이라 동시에 두 파드가 붙지 않는다」는 **사실**만 적을 뿐 **SQLite 때문**이라는 인과를
적지 않는다. `backend/src/db.rs` 의 WAL 문단이 같은 인과를 담지만 **그것은 주석이지 복원 경로가
아니다**(15차 패스가 `main.rs`/`worker.rs` 의 PID 1 함정에서 세운 판정 — 「다른 파일의 주석에 적혀
있다」는 복원 경로가 아니므로 복제 정리는 규칙이 아니라 재량).

한 줄을 반으로 쪼개면 순 제거는 **0행**인데 유일한 인과가 깎일 위험은 남는다. 정책 본문
「**애매하면 남긴다** — 비용이 비대칭이다」에 따라 **그대로 유지**하고 이 절에 남긴다.
이 판정을 뒤집으려면 `db.rs` WAL 문단과 함께 증분 재판정으로 한 번에 해야 한다.

## 검증

- **주석 아닌 바이트가 부모와 동일 2/2** — 줄머리 `//`·`///`·`//!`·블록 주석을 걷어낸 잔여의 md5 가
  `backend/src/db.rs` · `backend/tests/migrations.rs` 둘 다 부모(`95d3395`)와 같다.
  `APPLIED`·`PRE_CLEANUP` 의 값과 SQL 파일은 무접촉이다.
- `cargo test --manifest-path backend/Cargo.toml --release` — **237 passed / 0 failed**(24 스위트).
- 문서 게이트 3종 rc=0(`check-journey-mockup.py` · `check-mockup-render.py` · `check-scenario-e2e.py`).
  새 패스 파일이 늘었으므로 허브(`docs/index.html`)에 `doc-row` 1건을 더하고 `Documents` 선언을
  48 → 49 로 올렸다(R9).
- **판정기 `✅ 해당 없음`** — 위 「무인 경로」 절.

## 이 패스가 병합되면

- 전역 주석 지문: 부모(`95d3395`) `lines=2726 files=135` → **`lines=2710 files=135`** /
  `e8db65a0d567d1535adef31c723d788035dea25ae035bf5951ac86e4e38b2b30`
  (**순 제거 16행**, 파일 수 불변 — 주석이 0행이 된 파일이 없다).
  이 패스는 **네 base 위에서 측정됐고 순 제거 16 은 넷 다 같다**: `b4a6b30`(#129 착지 전)
  `lines=2649` → `2633`, `27d9b81`(#129 착지 직후) `lines=2646` → `2630`,
  `074c325`(#131~#136 착지 후) `lines=2654` → `2638`, `95d3395`(현재 base — #137·#138 착지 후)
  `lines=2726` → `2710`. **절대값은 자매 착지로 움직이고 순 제거는 안 움직인다** —
  그래서 완료 기준이 절대 지문이 아니다(아래).
- **완료 기준은 절대 지문이 아니라 「부모 대비 순 제거 16행」과 「4파일이 원장 행을 갖는다」이다.**
  실제로 #129 가 먼저 착지했고 그 뒤 사람 PR 5건(#131~#136)이 더 들어와 절대값이 두 번 움직였지만,
  **순 제거 16 과 행 23 의 지문 `b15045231102a7e2…` 는 그대로다.**

## 범위 밖 (후속)

- **`e2e/tests/sc01-08-succeeded-stage-rerun.spec.ts` 13행** — #121(`b4a6b30`, AC1.5 확장)이 들여온
  **새 미판정 파일**이다. 이 패스의 측정점(`95d3395`)에 이미 들어와 있지만 task 의 gap 이 지목한
  4파일 밖이라 건드리지 않았다. 다음 감지가 여는 몫이다.
- **판정 완료 범위 안의 미판정 증분 47행** — #121 이 이미 판정된 파일들에 더한 **39행**(20차 패스가
  줄 수·지문만 자매 착지 재실측으로 따라 적어 행 열 **안**에 있다)과, 이 패스가 준비된 뒤 #132·#134 가
  행 9·10 에 더한 **8행**(행 열 **밖** — 그래서 이 트리에서 그 두 행이 재현되지 않는다)이다.
  증분 재판정은 새 행이 아니라 **원래 행의 결과 칸 갱신**으로 닫아야 하므로 이 패스에 섞지 않았다.
  뒤의 8행은 `rct_20260922-0009` 가 받았다.
- **마이그레이션 축 5파일 123행**(0009~0013) — 사람 게이트(전용 PR · 수동 repair · 사람 승인).
- **`tools/check-data-format-change.py` 36행** — D6, 주석 패스가 스스로 풀 수 없는 유일한 자리.

## 열린 자매 PR 과의 겹침 (착지 순서)

- **#126**(슬라이스 6e, 현재 `DIRTY`)이 `backend/tests/migrations.rs` 의 `APPLIED` 를 `; 13]` → `; 14]`
  로 고치고 0014 행을 더한다. 그 헝크의 문맥 3줄에 이 패스가 **지운 ④ 블록**이 들어 있어 재해소가
  필요하다. 해소 규칙은 **양쪽 다 취한다** — 주석 4행은 이쪽(삭제)을, `APPLIED` 의 `14` 와 0014 행은
  #126 쪽을 살린다. add/add 가 아니다.
- **#129**(자매 주석 패스 · 20차)는 이 트리에 **이미 착지했다.** 그 패스가 원장 1·4·5·7·9행을
  자매 착지 재실측으로 따라 적었으므로 #121 의 증분 39행은 행 열 **안**으로 들어왔다.
- **#131~#136**(사람 PR 5건)이 그 뒤에 착지했다. 그중 **#132·#134 가 행 9·10 에 8행**을 열어
  이 트리의 행 재현은 **21/23** 이다(행 9 `142→148` · 행 10 `88→90`). 산술은 그 8행을 행 열 밖으로
  세어 다시 닫힌다 — **행 열의 합 2,458 + 행 열 밖 증분 8 + 잔여 172 = 전역 2,638**.
