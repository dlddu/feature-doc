#!/usr/bin/env python3
# 데이터 저장 형식 변경 판정기 — PR 에 사람 리뷰가 필요한지 가르는 1차 필터.
#
# PR 의 변경(merge-base..head)을 읽어 **DB 스키마나 그 밖에 영속되는 데이터의 형식**을
# 건드렸을 가능성이 있는지 판정한다. 가능성이 없으면 워크플로
# `review — data format` 이 head 커밋에 commit status `review/data-format` = success 를
# 붙이고, 가능성이 있으면 **아무 status 도 붙이지 않는다**. 즉 이 status 는
# "저장 형식은 바뀌지 않았다 — 그 관점의 사람 리뷰는 생략해도 된다"는 **적극적 확인**이고,
# 없다고 해서 실패는 아니다.
#
# 판정은 의도적으로 **보수적(fail-closed)** 이다. 놓치면(형식이 바뀌었는데 success) 사람
# 리뷰 없이 운영 DB 가 깨질 수 있고, 과잉 판정(안 바뀌었는데 status 없음)은 리뷰 한 번이
# 늘 뿐이다. 그래서 규칙은 "확실히 무해한 것"을 고르는 대신 "조금이라도 저장 계층에 닿는
# 것"을 넓게 잡는다. 판정이 불가능하면(git·TOML 파싱 실패 등) 종료 코드 2 로 끝나고
# status 는 붙지 않는다.
#
# 의존성 0 (python3 ≥ 3.11 stdlib — tomllib). 형제 체커 `check-*.py` 와 같은 방침이다.
#
# ── 규칙 ──────────────────────────────────────────────────────────────────
#  D1 마이그레이션    `backend/migrations/**` 의 **모든** 변경. 주석·공백 한 글자도 sqlx
#                     체크섬을 바꿔 부팅을 깨뜨린다(`backend/migrations/README.md`).
#  D2 저장 계층 핵심  `db.rs`(연결·pragma) · `crypto.rs`(봉투 암호화 형식) · `models.rs`
#                     (행 타입 · 컬럼의 현재 의미) · `pipeline.rs`(DB 에 저장되는 stage 키·
#                     status 값) · `backend/tests/migrations.rs`(체크섬 고정값)의 모든 변경.
#  D3 영속화 코드     `backend/src/**/*.rs` 의 **주석이 아닌** 변경 줄이 다음 중 하나에 닿음:
#                     SQL 키워드(대문자) · sqlx API(`.bind(`·`query_as`·`FromRow`…) ·
#                     직렬화(`serde_json::*`·`#[serde`·`derive(Serialize/Deserialize)`) ·
#                     저장되는 JSON 의 모양(`json!(`·`"key":`) · 저장값 해시/암호(`Sha256`·
#                     `content_hash`·`crypto::`…).
#  D4 의존성          저장 형식에 영향을 주는 크레이트(sqlx · libsqlite3-sys · serde ·
#                     serde_json · aes-gcm · sha2 · uuid …)의 `Cargo.toml` 선언(dev 제외)이나
#                     `Cargo.lock` 고정 버전이 바뀜. 줄 diff 가 아니라 merge-base 와 head 의
#                     두 파일을 통째로 파싱해 비교한다 — lock 의 hunk 문맥에는 패키지 이름이
#                     안 보일 때가 많아서다.
#  D5 배포 저장소     `deploy/**/pvc*.yaml` 의 모든 변경, 그 밖의 `deploy/**/*.yaml` 에서 DB
#                     경로·볼륨·replica·배포 전략 줄의 변경(단일 writer 전제가 여기에 있다).
#                     이미지 태그 고정(`pin` 잡)은 여기에 걸리지 않는다.
#  D6 판정기 자신     이 파일과 워크플로 `data-format-review.yml` 의 변경. 워크플로는
#                     pull_request_target 이라 PR 이 아니라 **base 쪽 판정기**로 돌므로 PR 이
#                     규칙을 고쳐 자기를 통과시킬 수는 없다 — 이 규칙은 그 변경 자체를 사람
#                     눈에 올린다.
#
# ── 이 판정기가 보지 않는 것(의도적) ─────────────────────────────────────
#  * **의미**는 보지 않는다. 키워드가 닿았는지까지가 기계의 몫이고, 실제로 형식이 바뀌었는지는
#    리뷰의 몫이다. 그래서 오탐(리뷰가 한 번 더 붙음)은 허용하고 미탐을 줄이는 쪽으로 넓게 잡는다.
#  * `backend/tests/**`(migrations.rs 제외) · `frontend/**` · `e2e/**` · `docs/**` · `tools/**` 는
#    운영 데이터를 쓰지 않으므로 보지 않는다. 프런트엔드는 localStorage 등 브라우저 저장소를
#    쓰지 않는다 — 쓰기 시작하면 여기에 규칙을 더해야 한다.
#  * 줄 끝 주석(`foo(); // UPDATE`)은 코드 줄로 본다(과잉 판정 쪽). 줄 전체가 주석인 것만 뺀다.
#
# 사용:
#   python3 tools/check-data-format-change.py --base <sha> --head <sha> [--verbose]
#   CI 에서는 $GITHUB_OUTPUT 에 needs_review=true|false, $GITHUB_STEP_SUMMARY 에 근거를 쓴다.
# 종료 코드: 0 = 판정 완료(결과는 출력으로), 2 = 판정 불가.

import argparse
import fnmatch
import os
import re
import subprocess
import sys
import tomllib
from collections import defaultdict

SELF_PATHS = {
    "tools/check-data-format-change.py",
    ".github/workflows/data-format-review.yml",
}

CORE_FILES = {
    "backend/src/db.rs",
    "backend/src/crypto.rs",
    "backend/src/models.rs",
    "backend/src/pipeline.rs",
    "backend/tests/migrations.rs",
}

# 저장 형식(스키마·직렬화·암호·해시·id)에 영향을 주는 크레이트.
STORAGE_CRATES = {
    "sqlx", "sqlx-core", "sqlx-sqlite", "sqlx-macros", "sqlx-macros-core",
    "libsqlite3-sys", "serde", "serde_json", "serde_derive",
    "aes-gcm", "aes", "aead", "sha2", "uuid", "base64", "hex",
}

D3_PATTERNS = [
    ("SQL 키워드", re.compile(
        r"\b(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|REPLACE|UPSERT|RETURNING|"
        r"VALUES|WHERE|JOIN|CONFLICT|PRAGMA|INDEX|TABLE|COLUMN|CHECK|UNIQUE|DEFAULT)\b")),
    ("sqlx API", re.compile(
        r"sqlx::|\.bind\(|query_as|query_scalar|FromRow|#\[sqlx|\.fetch_(one|all|optional)\(")),
    ("직렬화", re.compile(
        r"serde_json::(to_string|to_vec|to_value|to_writer|from_str|from_slice|from_value)|"
        r"#\[serde|derive\([^)]*\b(Serialize|Deserialize)\b")),
    ("저장 JSON 모양", re.compile(r"json!\s*\(|\"[A-Za-z_][A-Za-z0-9_]*\"\s*:(?!:)")),
    ("저장값 해시·암호", re.compile(
        r"\b(Sha256|Sha384|Sha512|Digest|content_hash|Aes256Gcm|Envelope|wrapped_dek|"
        r"dek_nonce|fingerprint)\b|crypto::")),
]

D5_LINE = re.compile(
    r"DATABASE_URL|volume|mountPath|claimName|persistentVolumeClaim|storageClass|"
    r"accessModes|strategy|replicas|Recreate", re.IGNORECASE)


def git(*args):
    r = subprocess.run(["git", *args], capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} 실패: {r.stderr.strip()}")
    return r.stdout


def is_comment_only(line):
    s = line.strip()
    return s == "" or s.startswith("//") or s.startswith("/*") or s.startswith("*")


def is_pvc(path):
    return path.startswith("deploy/") and fnmatch.fnmatch(os.path.basename(path), "pvc*.y*ml")


def parse_diff(diff_text):
    """파일별 변경 줄 목록: path -> [(줄 번호, '+'|'-', 내용)]."""
    changed = defaultdict(list)
    path = None
    old_no = new_no = 0
    # `diff --git` ~ 첫 `@@` 사이만 헤더다. 삭제된 SQL 주석 `-- x` 는 본문에서 `--- x` 로
    # 보이므로 헤더 판별을 이 구간 밖에서 하면 파일 경계를 잘못 읽는다.
    in_header = False
    for raw in diff_text.splitlines():
        if raw.startswith("diff --git "):
            path, in_header = None, True
            continue
        if in_header and raw.startswith("--- "):
            p = raw[4:]
            if p != "/dev/null":
                path = p[2:] if p.startswith("a/") else p
            continue
        if in_header and raw.startswith("+++ "):
            p = raw[4:]
            if p != "/dev/null":
                path = p[2:] if p.startswith("b/") else p
            continue
        if raw.startswith("@@"):
            in_header = False
            m = re.match(r"@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@", raw)
            if m:
                old_no, new_no = int(m.group(1)), int(m.group(2))
            continue
        if in_header or path is None or not raw or raw[0] not in "+- ":
            continue
        sign, text = raw[0], raw[1:]
        if sign == "+":
            changed[path].append((new_no, sign, text))
            new_no += 1
        elif sign == "-":
            changed[path].append((old_no, sign, text))
            old_no += 1
        else:
            old_no += 1
            new_no += 1
    return changed


def read_toml(rev, path):
    """rev 의 path 를 TOML 로 읽는다. 파일이 없으면 {}. 파싱 실패는 판정 불가로 올린다."""
    r = subprocess.run(["git", "show", f"{rev}:{path}"], capture_output=True)
    if r.returncode != 0:
        return {}
    return tomllib.loads(r.stdout.decode("utf-8"))


def storage_deps(manifest):
    """Cargo.toml 에서 운영 바이너리에 들어가는 저장 관련 의존성 선언만 뽑는다.
    dev-dependencies 는 운영 데이터를 쓰지 않으므로 뺀다."""
    tables = [manifest.get("dependencies", {}), manifest.get("build-dependencies", {})]
    for tgt in manifest.get("target", {}).values():
        tables += [tgt.get("dependencies", {}), tgt.get("build-dependencies", {})]
    out = {}
    for t in tables:
        for name, spec in t.items():
            real = spec.get("package", name) if isinstance(spec, dict) else name
            if real in STORAGE_CRATES:
                out[name] = repr(spec)
    return out


def storage_locked(lock):
    """Cargo.lock 에서 저장 관련 크레이트의 (버전, 소스, 체크섬) 집합."""
    out = defaultdict(set)
    for pkg in lock.get("package", []):
        if pkg.get("name") in STORAGE_CRATES:
            out[pkg["name"]].add((pkg.get("version"), pkg.get("source"), pkg.get("checksum")))
    return out


def dependency_hits(base, head):
    hits = []
    path = "backend/Cargo.toml"
    a, b = storage_deps(read_toml(base, path)), storage_deps(read_toml(head, path))
    for name in sorted(set(a) | set(b)):
        if a.get(name) != b.get(name):
            hits.append(f"{path} [{name}] {a.get(name, '(없음)')} → {b.get(name, '(없음)')}")
    path = "backend/Cargo.lock"
    a, b = storage_locked(read_toml(base, path)), storage_locked(read_toml(head, path))
    for name in sorted(set(a) | set(b)):
        if a.get(name) != b.get(name):
            va = ",".join(sorted(v for v, _, _ in a.get(name, ()) if v)) or "(없음)"
            vb = ",".join(sorted(v for v, _, _ in b.get(name, ()) if v)) or "(없음)"
            hits.append(f"{path} [{name}] {va} → {vb}")
    return hits


def classify(files, changed, dep_hits):
    """규칙별 근거 목록. 비어 있으면 저장 형식 변경 없음."""
    hits = defaultdict(list)

    for f in files:
        if f in SELF_PATHS:
            hits["D6 판정기 자신"].append(f)
        if f.startswith("backend/migrations/"):
            hits["D1 마이그레이션"].append(f)
        if f in CORE_FILES:
            hits["D2 저장 계층 핵심"].append(f)
        if is_pvc(f):
            hits["D5 배포 저장소"].append(f)

    for f, lines in changed.items():
        if f.startswith("backend/src/") and f.endswith(".rs") and f not in CORE_FILES:
            for no, sign, text in lines:
                if is_comment_only(text):
                    continue
                for label, pat in D3_PATTERNS:
                    if pat.search(text):
                        hits["D3 영속화 코드"].append(
                            f"{f}:{no} ({sign}, {label}) {text.strip()[:120]}")
                        break
        if f.startswith("deploy/") and f.endswith((".yaml", ".yml")) and not is_pvc(f):
            for no, sign, text in lines:
                if not text.strip().startswith("#") and D5_LINE.search(text):
                    hits["D5 배포 저장소"].append(f"{f}:{no} ({sign}) {text.strip()[:120]}")

    if dep_hits:
        hits["D4 의존성"].extend(dep_hits)
    return hits


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", required=True, help="PR base 커밋 (merge-base 계산에 쓴다)")
    ap.add_argument("--head", required=True, help="PR head 커밋")
    ap.add_argument("--verbose", action="store_true", help="통과해도 검사한 파일 목록을 출력")
    args = ap.parse_args()

    rng = f"{args.base}...{args.head}"
    try:
        merge_base = git("merge-base", args.base, args.head).strip()
        files = [f for f in git("diff", "--no-renames", "--name-only", rng).splitlines() if f]
        # 외부 diff 드라이버·textconv 를 끈다 — PR 이 넣은 .gitattributes 가 출력 모양을
        # 바꿔 판정을 흐리지 못하게.
        diff = git("diff", "--no-renames", "--no-color", "--no-ext-diff", "--no-textconv",
                   "-U3", rng)
        dep_hits = dependency_hits(merge_base, args.head)
    except (RuntimeError, tomllib.TOMLDecodeError, UnicodeDecodeError) as e:
        print(f"::error::판정 불가 — {e}", file=sys.stderr)
        return 2

    hits = classify(files, parse_diff(diff), dep_hits)
    needs_review = bool(hits)

    out = ["## 데이터 저장 형식 변경 판정: "
           + ("⚠️ 사람 리뷰 필요 (status 미부여)" if needs_review
              else "✅ 변경 없음 (`review/data-format` = success)"),
           "", f"변경 파일 {len(files)}개 · 범위 `{rng}`", ""]
    for rule in sorted(hits):
        out.append(f"### {rule} — {len(hits[rule])}건")
        for h in hits[rule][:30]:
            out.append(f"- `{h}`")
        if len(hits[rule]) > 30:
            out.append(f"- … 외 {len(hits[rule]) - 30}건")
        out.append("")
    if args.verbose or not needs_review:
        out.append("<details><summary>검사한 파일</summary>\n")
        out += [f"- `{f}`" for f in files] or ["- (없음)"]
        out.append("\n</details>")
    report = "\n".join(out)
    print(report)

    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as fh:
            fh.write(report + "\n")
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as fh:
            fh.write(f"needs_review={'true' if needs_review else 'false'}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
