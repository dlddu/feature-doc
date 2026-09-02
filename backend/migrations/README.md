# 마이그레이션 규칙

## 원칙: 적용된 마이그레이션은 불변이다

**한 번 배포된 `.sql` 파일은 한 글자도 고치지 않습니다. 주석도, 공백도, 오타도.**

`db.rs`의 `sqlx::migrate!("./migrations")`는 각 파일의 SQL 바이트 전체를 SHA-384로
해싱해 `_sqlx_migrations.checksum`(BLOB)에 저장하고, 부팅할 때마다 파일과 대조합니다
(`sqlx-core`의 `Migrator::run`). 다르면 `VersionMismatch`로 부팅이 실패합니다.
해시는 SQL문만이 아니라 **파일 전체 바이트**를 대상으로 하므로 주석 한 글자를
바꿔도 값이 달라집니다.

Flyway의 `repair`에 해당하는 명령이 sqlx에는 **없습니다**. `set_ignore_missing()`은
파일이 *사라졌을* 때를 위한 것이지 *바뀌었을* 때가 아닙니다. 즉 되돌릴 방법은
DB를 직접 손대는 것뿐입니다.

### CI는 이 사고를 잡아주지 못합니다

`cargo test`도 `scripts/e2e.sh`의 kind e2e도 매번 **새 임시 DB**에서 돕니다.
적용 이력이 없으니 체크섬 대조가 일어나지 않고, 전부 통과합니다.
실패는 기존 볼륨을 쓰는 `feature-doc/featuredoc`에 롤아웃될 때 처음 드러납니다.
지금 이 경로를 막는 자동 검사는 없으므로, 리뷰에서 사람이 봐야 합니다
(PR diff에 기존 `.sql` 파일 *수정*이 있으면 멈추고 이 문서를 확인하세요).

## 주석에 무엇을 쓰나

`.sql` 파일 안의 주석은 **"이 변경을 왜 했는가"** — 그 시점의 기록만 남깁니다.
과거 사실이라 나중에 바뀔 일이 없기 때문입니다.

**쓰지 않습니다** — 나중에 리네임·삭제될 이름:

- 파일 경로 (`docs/mockups/foo.html`, `src/bar.rs`)
- 목업·와이어프레임 파일명
- 언제든 옮겨질 수 있는 모듈·함수 경로

**대신 안정 식별자를 씁니다**:

- AC 번호 (`AC4.5`), 가치 번호 (`V3`)
- 여정·단계 ID (`JRN-discover-features`, `STP-leave-and-return`)
- 화면 ID (`S04`)
- PR 번호, 커밋 SHA 퍼머링크

**스키마의 "지금 의미"는 이 파일에 두지 않습니다.** 의미는 계속 변하는데 파일은
못 고치기 때문입니다. SQLite에는 `COMMENT ON COLUMN`이 없으므로, 컬럼의 현재
의미는 그 컬럼을 다루는 코드 옆 rustdoc(`models.rs`, `worker_api.rs` 등)에 둡니다.

## 이미 적용된 파일의 내용이 틀렸을 때

파일을 고치지 말고 **아래 「정정 로그」에 적습니다.** 죽은 참조를 지우는 것도
체크섬 변경이라 똑같이 부팅을 깨뜨립니다.

### 정정 로그

| 파일 | 정정 내용 |
| --- | --- |
| _(없음)_ | |

## 그래도 파일을 고쳐야 한다면 (repair)

프로덕션 DB를 직접 UPDATE하는 최후 수단입니다. **단독으로 하지 말고 아래 순서를
지킵니다.** `featuredoc`은 `replicas: 1` + `strategy: Recreate` + RWO 볼륨이라
동시에 두 파드가 붙지 않는 것이 전제입니다.

1. 새 체크섬을 계산합니다.

   ```bash
   python3 -c "import hashlib,pathlib; \
     p=pathlib.Path('backend/migrations/0004_analysis_stages.sql'); \
     print(hashlib.sha384(p.read_bytes()).hexdigest())"
   ```

2. **DB를 먼저 고치고, 머지는 나중에 합니다.** main에 머지하면 CI의 `pin` 잡이
   새 이미지 SHA를 매니페스트에 자동 커밋하고 Flux가 그대로 롤아웃하므로,
   중간에 사람이 끼어들 게이트가 없습니다. 머지가 먼저면 새 파드는 무조건
   `VersionMismatch`로 죽습니다.

   반대 순서(DB 먼저)의 노출 구간은 "구 이미지 파드가 롤아웃 전에 재기동되는
   경우"뿐입니다. 짧고, 5의 `.bak`으로 되돌릴 수 있습니다.

3. Flux 리컨실을 멈추고 API를 내립니다. **멈추지 않으면 Flux가
   `replicas: 1`을 되돌려 놓습니다** — 매니페스트가 SSOT이기 때문입니다.

   ```bash
   flux suspend kustomization <featuredoc-kustomization>
   kubectl -n feature-doc scale deploy/featuredoc --replicas=0
   kubectl -n feature-doc rollout status deploy/featuredoc --timeout=120s
   ```

   RWO 볼륨은 정비용 파드가 잡아야 하므로 파드가 완전히 내려간 것을 확인하고
   넘어갑니다.

4. 볼륨을 마운트한 임시 파드에서 UPDATE합니다. 런타임 이미지에는 `sqlite3`가
   없으므로 별도 이미지를 씁니다.

   ```bash
   kubectl -n feature-doc run sqlite-repair --rm -it --restart=Never \
     --image=keinos/sqlite3:latest \
     --overrides='{"spec":{"containers":[{"name":"sqlite-repair","image":"keinos/sqlite3:latest","stdin":true,"tty":true,"command":["sh"],"volumeMounts":[{"name":"data","mountPath":"/data"}]}],"volumes":[{"name":"data","persistentVolumeClaim":{"claimName":"featuredoc-data"}}]}}'
   ```

   파드 안에서 **먼저 현재 상태를 확인**하고(그 버전이 실제로 적용돼 있는지,
   기존 값이 무엇인지), 그 다음 해당 행만 갱신합니다.

   ```sh
   cp /data/featuredoc.db /data/featuredoc.db.bak-$(date +%Y%m%d%H%M)
   sqlite3 /data/featuredoc.db \
     "SELECT version, description, hex(checksum) FROM _sqlx_migrations ORDER BY version;"
   sqlite3 /data/featuredoc.db \
     "UPDATE _sqlx_migrations SET checksum = X'<새_체크섬_hex>' WHERE version = <버전>;"
   ```

   적용된 적 없는 버전이라면 행이 없습니다. 그때는 UPDATE가 0행을 바꾸고
   끝나며, 애초에 repair가 필요 없는 경우입니다.

5. Flux를 재개하고 변경을 main에 머지합니다. `test` → `push` → `pin`이 돌아
   새 이미지 SHA가 매니페스트에 커밋되고, Flux가 그것을 롤아웃합니다.

   ```bash
   flux resume kustomization <featuredoc-kustomization>
   ```

6. 부팅 로그와 `/hello`(마이그레이션이 적용된 뒤에만 응답합니다)로 확인합니다.

   ```bash
   kubectl -n feature-doc rollout status deploy/featuredoc --timeout=180s
   kubectl -n feature-doc logs deploy/featuredoc --tail=50
   ```

> **되돌리기**: `VersionMismatch`가 뜨면 4에서 떠둔 `.bak`을 제자리에 복사하고
> 매니페스트의 이미지 태그를 직전 SHA로 되돌립니다. 이미지는 GHCR에 이미
> 있으므로 재빌드가 필요 없습니다.
