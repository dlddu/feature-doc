#!/usr/bin/env python3
"""주석 판정 원장(docs/comment-policy/ledger.md)의 불변식을 CI에서 강제한다.

원장은 행 단위 사실의 표다. 손으로 적은 합계는 갱신되지 않아 낡고, 모든 PR이 그 한 줄을
고치면 파일 집합이 서로소인 PR끼리도 충돌한다. 그래서 집계는 저장하지 않고 이 게이트가
표와 실측에서 계산해 출력한다.

강제하는 것(어기면 rc=1):
  I1  판정 대상 주석 줄을 가진 파일은 정확히 한 행에 속한다 — 중복 소속 없음
  I2  그 파일 집합의 합집합이 실측 파일 집합을 덮는다 — 미등재 없음
  I3  행의 줄 수·지문이 실측과 같다
  I4  판정 축 표기가 유효하다 (`—` 또는 ①②③④의 순서 있는 부분집합)
  I5  표와 「원장 읽는 법」 밖에 산문이 없다

출력만 하는 것(실패가 아니다): 판정 완료 줄·행 수, 축별 미판정 행 수, 주석이 0행이 된 등재
파일. 미판정(`—`)은 판정하지 않았다는 사실을 행에 남긴 정상 상태이고, 그 행은 다음 판정
슬라이스가 가져간다.

측정 범위는 모델 `asIs.versionScript`와 **같은 규칙**이다 — 레포 전체에서 제외 세 부류를 뺀
것, 같은 언어군 표, 같은 `comment-scope-exclude` 블록. 게이트가 경로 목록을 따로 들고 있으면
둘이 갈라져, 지문에는 있는데 원장 불변식은 모르는 파일이 생긴다.
"""

import hashlib
import os
import re
import subprocess
import sys

ROOT = subprocess.run(["git", "rev-parse", "--show-toplevel"],
                      capture_output=True, text=True, check=True).stdout.strip()
LEDGER = "docs/comment-policy/ledger.md"
README = "docs/comment-policy/README.md"

FIXED_EXCLUDE = re.compile(
    r"^docs/|\.md$|(^|/)(vendor|node_modules|dist|build|target|\.venv|venv|__pycache__"
    r"|\.next|coverage)/|(^|/)(package-lock\.json|yarn\.lock|pnpm-lock\.yaml|go\.sum"
    r"|uv\.lock|poetry\.lock|Cargo\.lock|Pipfile\.lock)$")

DIRECTIVE = re.compile(
    r":#!|//go:|nolint|eslint-|@ts-|prettier-ignore|/// <reference|istanbul ignore"
    r"|c8 ignore|# ?noqa|# ?type:|# ?pragma|# ?pylint:|# ?fmt:|shellcheck |# ?syntax="
    r"|yaml-language-server:|검증 시나리오:|mock-exception:")

SUF = r"(\.(example|sample|template|tmpl|tpl|dist|in|j2))?$"
GROUPS = [
    (r"\.(go|rs|java|kt|kts|scala|groovy|gradle|swift|c|h|cc|cpp|hpp|cs|m|js|jsx|mjs"
     r"|cjs|ts|tsx|mts|cts|css|scss|less|proto|jsonc)" + SUF +
     r"|(^|/)(go\.mod|go\.work|tsconfig[^/]*\.json|jsconfig[^/]*\.json"
     r"|\.devcontainer/[^/]*\.json)$",
     r"^[ \t]*(//|/\*|\*([ \t]|$)|\{/\*)"),
    (r"\.(py|pyi|rb|sh|bash|zsh|fish|pl|r|ya?ml|toml|tf|tfvars|hcl|cfg|conf|ini|mk"
     r"|dockerfile|nix|awk|sed)" + SUF +
     r"|(^|/)(Makefile|GNUmakefile|Dockerfile[^/]*|Containerfile|Caddyfile|\.gitignore"
     r"|\.dockerignore|\.gitattributes|\.helmignore|\.editorconfig|\.env[^/]*|CODEOWNERS"
     r"|requirements[^/]*\.txt)$",
     r"^[ \t]*#"),
    (r"\.(sql|lua|hs|elm|ada|adb)" + SUF, r"^[ \t]*--"),
    (r"\.(html?|vue|svelte|astro)" + SUF, r"^[ \t]*(<!--|//|/\*|\*([ \t]|$)|\{/\*)"),
    (r"\.(xml|svg|xhtml|plist|xsd|xsl)" + SUF, r"^[ \t]*<!--"),
    (r"\.(json|jsonl|ndjson|csv|tsv|txt|avsc|snap|golden|pem|crt|key|pub|patch|diff"
     r"|log|lock|sum|mod|map|http)" + SUF +
     r"|(^|/)(LICENSE[^/]*|NOTICE|AUTHORS|\.nvmrc|\.node-version|\.python-version"
     r"|\.tool-versions|\.gitkeep|py\.typed)$", None),
]
GROUPS = [(re.compile(f), re.compile(p) if p else None) for f, p in GROUPS]


def repo_exclude():
    """대상 레포 고유 제외 — README 의 ```comment-scope-exclude 블록(한 줄에 ERE 하나).

    모델이 아니라 대상 레포에 두는 이유: 범위가 낡았을 때의 수정이 대상 레포 PR 로 끝나야
    이 루프가 스스로 수렴시킬 수 있다. 블록이 깨졌으면 빈 범위로 뭉개지 않고 실패한다.
    """
    path = os.path.join(ROOT, README)
    if not os.path.exists(path):
        return []
    out, inside = [], False
    for line in open(path, encoding="utf-8"):
        if re.match(r"^```comment-scope-exclude[ \t]*$", line):
            inside = True
            continue
        if inside:
            if line.startswith("```"):
                break
            if line.strip():
                out.append(line.strip())
    for ere in out:
        try:
            re.compile(ere)
        except re.error as exc:
            sys.exit(f"FAIL: invalid ERE in comment-scope-exclude block: {ere!r} ({exc})")
    return out


def is_text(path):
    try:
        with open(path, "rb") as fh:
            return b"\0" not in fh.read(8192)
    except OSError:
        return False


def measure():
    """실측 = {파일: [정규화된 주석 줄, ...]} + 미분류 경로 목록."""
    extra = [re.compile(e) for e in repo_exclude()]
    files = subprocess.run(["git", "ls-files"], cwd=ROOT, capture_output=True,
                           text=True, check=True).stdout.split("\n")
    cand = []
    for f in sorted(x for x in files if x):
        if FIXED_EXCLUDE.search(f) or any(e.search(f) for e in extra):
            continue
        full = os.path.join(ROOT, f)
        if not os.path.isfile(full) or not is_text(full):
            continue
        head = "".join(open(full, encoding="utf-8", errors="replace").readlines()[:5])
        if "DO NOT EDIT" in head or "@generated" in head:
            continue          # ② 편집 불가 — 지워도 재생성된다
        cand.append(f)

    hits, rest = {}, []
    for f in cand:
        pat = None
        for fre, cre in GROUPS:
            if fre.search(f):
                pat = cre
                break
        else:
            with open(os.path.join(ROOT, f), "rb") as fh:
                if fh.read(2) == b"#!":
                    pat = GROUPS[1][1]
                else:
                    rest.append(f)
                    continue
        if pat is None:
            continue          # NONE 언어군 — 제외 ③
        got = []
        for line in open(os.path.join(ROOT, f), encoding="utf-8",
                         errors="replace").read().split("\n"):
            if not pat.match(line):
                continue
            raw = f"{f}:{line}"
            if DIRECTIVE.search(raw):
                continue      # 기계가 읽는 주석 — 지문·판정 모두에서 제외
            got.append(re.sub(r"[ \t]+", " ", raw).strip())
        if got:
            hits[f] = got
    return hits, rest


AXES_OK = re.compile(r"^(—|①?②?③?④?)$")


def parse_ledger():
    """표를 파싱한다. 범위 칸의 백틱 경로만 파일이고 첫 여는 괄호 뒤는 덩어리 이름이다.

    경로가 없는 bare 이름은 `.sql` 일 때만 `backend/migrations/` 로 resolve 한다 — 마이그레이션
    행이 같은 디렉터리의 파일을 이름만으로 잇기 때문이다. 확장자 없는 루트 파일(`Dockerfile`)에
    그 규칙을 적용하면 있지도 않은 경로를 찾는다.
    """
    text = open(os.path.join(ROOT, LEDGER), encoding="utf-8").read()
    rows, prose, in_table, seen_how = [], [], False, False
    for no, line in enumerate(text.split("\n"), 1):
        if line.startswith("## 원장 읽는 법"):
            seen_how = True
        if line.startswith("|"):
            in_table = True
            if line.startswith("| 판정일") or set(line) <= set("|- "):
                continue
            cells = [c.strip() for c in line.strip().strip("|").split(" | ")]
            if len(cells) != 6:
                sys.exit(f"FAIL(I5): {LEDGER}:{no} 열이 6개가 아니다 ({len(cells)})")
            scope = cells[1]
            names = re.findall(r"`([^`]+)`", scope.split("(")[0])
            rows.append(dict(no=no, date=cells[0], scope=scope, axes=cells[4],
                             files=[n if "/" in n or not n.endswith(".sql")
                                    else "backend/migrations/" + n for n in names],
                             lines=cells[2], fp=cells[3].strip("`")))
            continue
        if in_table and line.strip():
            prose.append((no, line))
    if not seen_how:
        sys.exit(f"FAIL(I5): {LEDGER} 에 「원장 읽는 법」 절이 없다")
    return rows, prose


def main():
    hits, unclassified = measure()
    rows, prose = parse_ledger()
    fail = []

    if prose:                                                            # I5
        fail.append(f"I5 표 뒤 산문 {len(prose)}줄 — 경위·집계는 passes/ 의 자리다: "
                    + "; ".join(f"{LEDGER}:{n} {l[:50]!r}" for n, l in prose[:3]))

    owner = {}                                                           # I1
    for r in rows:
        for f in r["files"]:
            if f in owner:
                fail.append(f"I1 `{f}` 가 두 행에 있다 — {LEDGER}:{owner[f]} 와 :{r['no']}")
            owner[f] = r["no"]

    unregistered = sorted(set(hits) - set(owner))                        # I2
    if unregistered:
        fail.append(f"I2 미등재 {len(unregistered)}파일 "
                    f"{sum(len(hits[f]) for f in unregistered)}줄 — 판정하지 않아도 행은 "
                    f"만든다(판정 축 `—`): " + ", ".join(unregistered[:6]))

    judged_lines = judged_rows = 0
    per_axis = {a: 0 for a in "①②③④"}
    for r in rows:
        got = sorted(x for f in r["files"] for x in hits.get(f, []))
        want_fp = hashlib.sha256(("\n".join(got) + "\n").encode()).hexdigest()
        if r["lines"] != str(len(got)):                                   # I3
            fail.append(f"I3 {LEDGER}:{r['no']} 줄 수 기재 {r['lines']} ≠ 실측 {len(got)} "
                        f"({r['files'][0]})")
        if r["fp"] != want_fp:
            fail.append(f"I3 {LEDGER}:{r['no']} 지문 기재 {r['fp'][:12]}… ≠ 실측 "
                        f"{want_fp[:12]}… ({r['files'][0]})")
        if not AXES_OK.match(r["axes"]):                                  # I4
            fail.append(f"I4 {LEDGER}:{r['no']} 판정 축 표기 {r['axes']!r} 가 유효하지 않다 "
                        f"— `—` 또는 ①②③④ 의 순서 있는 부분집합")
        if r["axes"] == "①②③④":
            judged_rows += 1
            judged_lines += len(got)
        for a in "①②③④":
            if a not in r["axes"]:
                per_axis[a] += 1

    total = sum(len(v) for v in hits.values())
    empty = sorted(f for f in owner if f not in hits)
    print(f"판정 대상 실측 : lines={total} files={len(hits)} "
          f"unclassified={len(unclassified)}")
    print(f"원장           : 행 {len(rows)} · 등재 파일 {len(owner)} · 미등재 "
          f"{len(unregistered)} · 주석 0행이 된 등재 파일 {len(empty)}")
    print(f"판정 완료(①②③④): {judged_lines}줄 / {judged_rows}행 "
          f"({judged_lines * 100 // max(total, 1)}% of lines)")
    print("축별 미판정 행  : " + " · ".join(f"{a} {per_axis[a]}" for a in "①②③④"))
    if unclassified:
        print("미분류(언어군 밖): " + ", ".join(unclassified))
    if empty:
        print("주석 0행        : " + ", ".join(empty))

    if fail:
        print("\n".join(["", "FAIL — 원장 불변식 위반:"] + [f"  - {m}" for m in fail]))
        return 1
    print("\n통과 — 원장 불변식 I1~I5 이상 없음")
    return 0


if __name__ == "__main__":
    sys.exit(main())
