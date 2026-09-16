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

### CI 검사와 기존 DB 확인은 별개입니다

`backend/tests/migrations.rs`의 `applied_migrations_are_never_edited`는 파일 전체의
SHA-256을 고정하고, `every_migration_file_is_pinned`는 목록에서 빠진 SQL 파일을
검출합니다. 새 마이그레이션을 추가할 때만 파일명과 해시를 함께 등록합니다.
기존 파일과 고정값을 동시에 바꾸면 이 검사를 우회하므로 그런 diff는 별도 검토가
필요합니다. `cargo test --manifest-path backend/Cargo.toml --test migrations --release`로
검사합니다.

이 고정값은 **소스 파일 불변성 검사**입니다. sqlx가 DB에 저장하는 값은 SHA-384이며,
소스의 SHA-256 고정값을 갱신했다고 기존 DB가 호환되거나 repair가 끝난 것은 아닙니다.

2026-09-15 PR #44(`4aa5590b6af1c0476a7c52d11065c86daf545ab5`)는 SQL문을 바꾸지
않고 0001·0003~0007의 주석을 바꿨습니다. 이 브랜치의 고정값은 그 커밋의 파일에
맞췄습니다. **SQL 파일을 다시 수정하거나 DB 체크섬을 자동으로 바꾸지 않습니다.**
변경 전 기준은 부모 `636ee771dd404ad7cf383d1bfadcc5036ab7fac6`입니다.

회귀 검사는 새 임시 DB에서 현재 SHA-384 이력이 재기동을 허용하는지 확인하고,
여섯 개의 변경 전 해시를 각각 주입해 `VersionMismatch(버전)`으로 거부되는지,
실패가 DB의 체크섬 이력을 변경하지 않는지 확인합니다. 임시 DB의 과거 해시는
이전 파일 바이트에서 계산한 값이며 실제 운영 DB를 수집한 값이 아닙니다.

**PR #43의 main 머지 전 선결 조건**: 해당 기능 task의 사람 승인과 별도로,
실행자는 승인된 읽기 전용 경로로 실제 `_sqlx_migrations`의 버전·성공 여부·SHA-384를
수집해 정확한 릴리스 소스/이미지와 비교해야 합니다. 증거가 없거나 값이 다르면
릴리스를 멈추고 별도로 승인된 복구 계획으로 넘깁니다. 이 테스트의 성공이나 준비 PR의
브랜치 통합은 운영 DB가 복구됐다는 증거가 아닙니다.

## 주석에 무엇을 쓰나

`.sql` 파일 안의 주석은 **"이 변경을 왜 했는가"** — 그 시점의 기록만 남깁니다.
과거 사실이라 나중에 바뀔 일이 없기 때문입니다.

**쓰지 않습니다** — 나중에 리네임·삭제될 이름:

- 파일 경로 (`docs/mockups/foo.html`, `src/bar.rs`)
- 목업·와이어프레임 파일명
- 언제든 옮겨질 수 있는 모듈·함수 경로

**문서에 사는 식별자는 아예 쓰지 않습니다.** AC 번호, 여정·단계 ID(`JRN-*`/`STP-*`),
목업 이름 — 전부 제품 문서 쪽에서 언제든 추가·수정·삭제되는 것들입니다. 마이그레이션
파일만 그 변화를 따라갈 수 없습니다.

이 규칙이 옳다는 증거가 바로 나왔습니다. 이 문서의 초판은 금지 예시로 화면 ID `S04` 를
들었는데, 며칠 뒤 화면 ID 체계 자체가 폐지되면서 **그 예시가 가리키는 것이 사라졌습니다**.
금지 목록에 적은 이름조차 삭제를 못 피한 셈입니다.

번호가 안 밀린다는 것도 안전을 뜻하지 않습니다. `AC4.1` 은 원래 "사용자 GitHub
Token 등록·교체·폐기"였는데, 인증을 PAT 에서 GitHub App 으로 바꾼 `ef2660c` 에서
**같은 번호 자리에** "GitHub App 설치를 통한 저장소 접근 연결·변경·해제"를
덮어썼습니다. 죽은 파일 경로는 링크 체커가 잡아내지만, 이런 의미 drift 는 참조가
멀쩡해 보이는 채로 어긋나서 아무도 모릅니다.

**대신 요구사항을 자체 완결적으로 서술합니다.** 번호를 가리키는 대신 그 번호가
요구하는 바를 그 자리에 씁니다. 그러면 문서가 어떻게 개편되든 주석은 계속 참입니다.

```sql
-- 나쁨:  AC4.1 을 만족시키기 위한 테이블.
-- 나쁨:  short-lived tokens are minted on demand and never stored (AC4.1).
-- 좋음:  short-lived installation access tokens are minted on demand and never
--        stored.
```

정말 참조가 필요하면 **불변인 것만** 씁니다 — PR 번호, 커밋 SHA 퍼머링크. 이건
내용이 갈아끼워지지 않습니다.

## 시점 서술을 쓰지 않습니다

"지금은 / 아직 / 나중 슬라이스에" 같은 **현재 상태에 대한 주장**은 파일을 고칠 수
없는 곳에 두면 시간이 지나 그냥 거짓말로 남습니다.

`0004` 의 "Only stage 1 (`fetch`) executes today; stages 2-5 stay 'pending'" 이
실제로 그렇게 됐습니다 — 워커는 그 뒤로 stage 2·3 까지 실행하게 됐는데 주석은
못 고쳐서 한동안 틀린 채로 있었습니다.

같은 내용도 구조로 쓰면 늙지 않습니다: "stage 2-5 는 아직 안 돈다"가 아니라
"executor 가 없는 stage 는 'pending' 으로 남는다".

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
     "UPDATE _sqlx_migrations SET checksum = X'<새_체크섬_hex>' WHERE version = <버전>;"  # 바뀐 파일마다 한 번씩
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
