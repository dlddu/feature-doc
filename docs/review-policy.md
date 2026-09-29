# PR 수동 승인 정책

> 수동 승인 케이스 · 위험 표면 · 의도적 제외 · 미탐 원장의 SSOT. 2026-09-29 최초 등재 (reconciler `rct_20260929-0001`, 모델 `tbm_feature-doc-pr-manual-approval`).
> 판정기(`tools/check-data-format-change.py`)는 이 문서의 케이스 ID 를 판정 근거 라벨로 쓰고 케이스 본문을 되풀이하지 않는다.

이 레포의 PR 수동 승인은 **하나의 프로세스**다. 판정기 하나가 PR 이 아래 케이스 어디에도 닿지 않았다고 확인될 때만 head 커밋에 `review/manual-approval` = success 를 붙이고, 기본 브랜치 ruleset 이 그 status 를 required 로 요구한다. 닿았을 가능성이 있으면 판정기는 아무 status 도 붙이지 않는다 — status 의 **부재**가 곧 「사람이 본다」이고, 판정기가 죽거나 판정이 불가능해도(git 오류 등) 같은 결과가 된다. 사람의 승인은 PR 을 검토한 사람이 **같은 context 로 직접 success 를 붙이는 것**(수동 status) 하나뿐이다.

## 프로세스

| 항목 | 값 |
|---|---|
| status context | `review/manual-approval` |
| 판정기 | `tools/check-data-format-change.py` (`--base <sha> --head <sha>`; 케이스에 닿으면 `needs_review=true`) |
| 워크플로 | `.github/workflows/data-format-review.yml` — `pull_request_target`, **base 커밋을 체크아웃**해 base 쪽 판정기로 돌고, PR 커밋은 fetch 해 `git diff` 의 입력(데이터)으로만 쓴다. PR 코드는 실행하지 않는다 |
| 게이트 | 기본 브랜치 ruleset `default`(id 23666120)의 required status check — `ci-gate`(integration 15368) · `review/manual-approval`(integration 미고정). `review/` 계열 required context 는 이것 하나다. 확인: `gh api "repos/dlddu/feature-doc/rules/branches/main" --jq '.[] \| select(.type=="required_status_checks") \| .parameters.required_status_checks[] \| "\(.context) integration=\(.integration_id // "any")"'` → `review/manual-approval integration=any` 가 있어야 한다 |
| 사람 승인 | 수동 status. 승인할 수 있는 사람: 저장소 소유자(`dlddu`). PR 을 검토한 뒤 head 커밋에 붙인다: `gh api -X POST "repos/dlddu/feature-doc/statuses/<head sha>" -f state=success -f context=review/manual-approval -f description="수동 승인 — <승인자>" -f target_url=<PR URL>`. 사람의 지시로 도는 승인 전용 도구: reconciler `reconciler-manual-approval` 스킬(사람 세션 전용) → homelab-k3s-mcp `github_commit_status_create` — 붙일 수 있는 context 는 서버 env `GITHUB_COMMIT_STATUS_CONTEXT_PREFIXES` = `review/` 로 제한되고(`dlddu/homelab-k3s-mcp` `k8s/deployment.yaml`), 이 레포에서 그 네임스페이스의 required check 는 `review/manual-approval` 하나다. ruleset bypass · 관리자 머지 · required check 해제는 승인 경로가 아니다. 승인 뒤 새 푸시로 head 가 바뀌면 승인은 무효가 되고 판정기가 다시 판정한다 |
| 마지막 재검토 | 2026-09-29 |
| 다음 재검토 | 2026-12-28 |

재검토 때는 날짜만 옮기지 않는다: 아래 `surface` 블록이 여전히 위험을 다 담는지(ERE 가 모르는 새 저장 기술·새 디렉터리가 없는지), ruleset 의 bypass 목록이 비어 있는지(admin 권한 필요)를 사람이 보고 결과를 이 절에 한 줄로 남긴다.

## 수동 승인 케이스

| ID | 대상 | 이유 |
|---|---|---|
| D1 | `backend/migrations/**` 의 **모든** 변경(추가·수정·삭제·개명 — 판정기는 `--no-renames` 로 개명을 옛 경로 삭제 + 새 경로 추가로 본다) | sqlx 가 적용된 마이그레이션의 체크섬을 부팅 때 대조하므로, 주석·공백 한 글자도 운영 DB 에서 부팅을 깨뜨린다(`backend/migrations/README.md`). 새 마이그레이션은 운영 데이터의 스키마를 바꾼다 |
| D6 | 판정기·워크플로·정책 자신 — `tools/check-data-format-change.py` · `.github/workflows/data-format-review.yml` · `docs/review-policy.md` | 워크플로가 `pull_request_target` 이라 PR 은 **base 쪽 판정기**로 판정된다 — PR 이 규칙을 고쳐 자기를 통과시킬 수는 없지만, 그 변경 자체(케이스를 줄이거나 자기 경로를 빼는 편집)는 옛 판정기가 보지 못한 채 다음 PR 부터 효력을 가진다. 그래서 이 세 파일의 변경은 사람 눈에 올린다 |

## 위험 표면

케이스의 위험이 코드에 나타나는 모양. reconciler as-is 지문이 이 블록에 걸리는 (파일, 토큰) 집합을 잰다 — 새 파일이 표면이 되거나 토큰이 바뀌면 지문이 움직여 이 정책이 다시 판정된다. 걸리는 파일은 전부 위 케이스에 덮이거나 아래 「의도적 제외」에 있어야 한다.

```surface
scope: backend/migrations backend/src backend/Cargo.toml deploy/k8s
ere: CREATE[[:space:]]+(TABLE|INDEX|UNIQUE[[:space:]]+INDEX|VIEW|TRIGGER)|ALTER[[:space:]]+TABLE|DROP[[:space:]]+(TABLE|INDEX|VIEW|TRIGGER)
ere: sqlx::migrate!\("[^"]*"\)
ere: json_(extract|each|tree)
ere: pub struct Envelope
ere: STAGES: \[Stage; [0-9]+\]
ere: ^(sqlx|rusqlite|libsqlite3-sys)[[:space:]]*=[^#]*"[0-9][0-9.]*"
ere: kind: PersistentVolumeClaim|claimName: [a-z0-9-]+|type: Recreate|^[[:space:]]+replicas: [0-9]+
```

## 앵커

케이스가 이름으로 지목하는 경로. 하나라도 사라지면 그 케이스는 죽은 케이스다 — 판정기와 이 문서를 새 위치로 옮기거나, 위험 자체가 사라졌으면 케이스를 지운다.

```anchors
backend/migrations/*.sql
tools/check-data-format-change.py
.github/workflows/data-format-review.yml
docs/review-policy.md
```

## 의도적 제외

판정기가 보지 않지만 운영 데이터를 깨뜨릴 수 있는 변경. 2026-09-22 축소(#125)로 옛 D2~D5 를 뺐다 — 키워드·경로 기반이라 오탐이 대부분이었고(읽기 전용 `SELECT` 한 줄, 와이어 필드의 `#[serde(default)]` 하나에도 울렸다), 그 몫은 일반 코드 리뷰가 진다. **그러므로 `review/manual-approval` 초록은 「마이그레이션 없이 운영 데이터를 깨뜨리는 변경이 없다」의 보증이 아니다.** 재개 조건이 참이 되면 그 행을 지우고 케이스를 더한다. 제외를 연장하려면 새 재개 조건을 쓴다.

| 대상 | 사유 | 재개 조건 | 소관 |
|---|---|---|---|
| `backend/src/crypto.rs` 봉투 암호화 형식(`Envelope` 필드·`seal`/`open`) | 형식이 바뀌면 저장된 LLM 키·토큰을 복호화할 수 없지만, 파일 단위로 케이스를 걸면 형식과 무관한 수정마다 울린다 | `Envelope` 가 형식 버전 필드를 갖거나 두 번째 봉투 구조체가 생긴다 — `git grep -nE '^[[:space:]]*pub (version\|format)[a-z_]*:' backend/src/crypto.rs` 1건 이상, 또는 `git grep -c 'pub struct Envelope' -- backend/src` 합계 2 이상 | 코드 리뷰 |
| `backend/src/pipeline.rs` stage key · status 값 | 값을 바꾸면 기존 행이 고아가 되지만 상수 이름·주석 수정과 구별되지 않는다 | 단계 수가 5 에서 바뀐다 — `git grep -c 'STAGES: \[Stage; 5\]' backend/src/pipeline.rs` 가 0 | 코드 리뷰 |
| DB 에 JSON 으로 저장되는 구조체의 모양(분석 문서 등) | JSON 은 DB 가 해석하지 않는 불투명 값이라 모양 변경은 역직렬화 쪽에서만 드러나고, 해당 구조체가 여러 파일에 흩어져 있다 | SQL 이 JSON 내부를 질의하기 시작한다 — `git grep -nE 'json_(extract\|each\|tree)' -- backend` 1건 이상 | 코드 리뷰 |
| `backend/Cargo.toml` 저장 크레이트 메이저 업그레이드(sqlx 등) | 드물고, 업그레이드 PR 은 CI 의 마이그레이션 테스트(`backend/tests/migrations.rs`)가 실 SQLite 로 돈다 | `sqlx` 메이저가 0.8 에서 바뀌거나 SQLite 직접 의존이 생긴다 — `grep -cE '^sqlx = \{ version = "0\.8"' backend/Cargo.toml` 가 0, 또는 `grep -cE '^(rusqlite\|libsqlite3-sys)' backend/Cargo.toml` 1 이상 | 코드 리뷰 |
| `deploy/k8s/` 의 PVC · DB 경로 · replica · 배포 전략 | SQLite 단일 writer 전제(ReadWriteOnce PVC 하나 · API replica 1 · `Recreate`)를 깨면 DB 가 잠기거나 갈라지지만, 매니페스트 수정 대부분은 무관하다 | API 가 단일 writer 가 아니게 된다 — `deploy/k8s/deployment.yaml` 에서 `replicas: 1` 또는 `type: Recreate` 가 사라지거나, `git grep -l 'kind: PersistentVolumeClaim' deploy/k8s` 가 2파일 이상 | 코드 리뷰 |
| `backend/src/db.rs` 의 `sqlx::migrate!` 호출 | 마이그레이션을 바이너리에 싣는 한 줄이라 D1 의 전제이지 위험 자체가 아니다 | 마이그레이션 디렉터리가 바뀐다 — `git grep -c 'sqlx::migrate!("./migrations")' backend/src/db.rs` 가 0 | 코드 리뷰 |

## 미탐 원장

`review/manual-approval` 이 자동으로 붙어 머지된 PR 에서, 사람 승인이 필요했던 변경을 뒤늦게 발견하면 한 행을 더한다. 보강 케이스 ID 는 케이스 표와 판정기에 실재해야 한다. 비어 있는 것이 정상이다.

| PR | 놓친 것 | 보강 케이스 ID | 기록일 |
|---|---|---|---|
