#!/usr/bin/env python3
"""표면 하나를 게이트 밖에 두지 말 것 — 그 표는 손으로 유지하는 값이 되고 실측과
갈라져도 통과한다. 그래서 세 표면을 한 게이트가 함께 잰다.

강제하는 것(어기면 rc=1):
  I1  표면의 판정 대상 줄을 가진 파일은 그 표면의 표에서 정확히 한 행에 속한다 — 중복 소속 없음
  I2  그 파일 집합의 합집합이 그 표면의 실측 파일 집합을 덮는다 — 미등재 없음
  I3  행의 줄 수·지문이 그 표면의 실측과 같다
  I4  판정 축 표기가 유효하다 (표면 무관: `—` 또는 ①②③④의 순서 있는 부분집합)
  I5  표면 절 셋이 모두 있고, 표와 「원장 읽는 법」·표면 절 제목 밖에 산문이 없다

측정 범위와 표면 추출은 모델 `asIs.versionScript`와 **같은 규칙**이어야 한다. 게이트가 자기
규칙을 따로 들고 있으면 둘이 갈라져, 지문에는 있는데 원장 불변식은 모르는 주석이 생긴다.
"""

import ast
import hashlib
import io
import os
import re
import subprocess
import sys
import tokenize

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

SURFACES = ("L", "D", "E")
DE_TAG = {0: "C", 1: "HASH", 2: "DASH"}
DE_DIRECTIVE = re.compile(DIRECTIVE.pattern.replace(":#!|", "", 1))
DE_SUF = re.compile(r"\.(example|sample|template|tmpl|tpl|dist|in|j2)$")
HASH_E_EXT = {"sh", "bash", "zsh", "fish", "rb", "pl", "r", "yaml", "yml", "toml",
              "tf", "tfvars", "hcl", "mk", "nix", "awk"}
MAKE = re.compile(r"^(Makefile|GNUmakefile)$|\.mk$")
OPEN_OK = set(" \t=([{,:")


def repo_exclude():
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


class DeScanner:
    """지문 스크립트의 DE 블록을 **논리 무수정으로** 이식한 것이다. 옮긴 것은 바깥
    껍데기뿐이고 규칙은 한 글자도 바꾸지 않는다 — 등가는 편집 전 트리에서 지문 스크립트의
    D·E 덤프와 이 클래스의 출력을 정렬해 바이트 대조하는 것으로 증명한다.
    """

    def __init__(self):
        self.out = []

    def emit(self, kind, path, text):
        t = re.sub(r"\s+", " ", text).strip()
        if t and not (kind == "E" and DE_DIRECTIVE.search(t)):
            self.out.append(f"{kind}:{path}:{t}")

    def rescue(self, path, comment, tail):
        """L 은 그 줄을 지시자와 함께 통째로 버리므로, 여기서 건지지 않으면 사유가
        어느 표면에도 잡히지 않는다.
        """
        if DE_DIRECTIVE.search(comment):
            m = re.search(tail, comment[1:])
            if m:
                self.emit("E", path, comment[1 + m.start():])

    def py(self, path, src):
        try:
            tree = ast.parse(src)
        except (SyntaxError, ValueError):
            self.out.append(f"X:{path}:ast")
            tree = None
        if tree is not None:
            for n in ast.walk(tree):
                if isinstance(n, (ast.Module, ast.ClassDef, ast.FunctionDef,
                                  ast.AsyncFunctionDef)):
                    for line in (ast.get_docstring(n, clean=True) or "").splitlines():
                        self.emit("D", path, line)
        lines = src.splitlines()
        try:
            for tok in tokenize.generate_tokens(io.StringIO(src).readline):
                if tok.type == tokenize.COMMENT:
                    r, c = tok.start
                    if lines[r - 1][:c].strip():
                        self.emit("E", path, tok.string)
                    else:
                        self.rescue(path, tok.string, r"\s#\s")
        except (tokenize.TokenError, IndentationError, SyntaxError):
            self.out.append(f"X:{path}:tokenize")

    def marker_lang(self, path, src, marker, quotes, make=False):
        for line in src.splitlines():
            s = line.lstrip()
            if s.startswith(marker):
                self.rescue(path, s, r"\s" + re.escape(marker) + r"\s")
                continue
            q = None
            i = 0
            while i < len(line):
                ch = line[i]
                if q:
                    if ch == "\\" and q == '"':
                        i += 2
                        continue
                    if ch == q:
                        q = None
                elif ch in quotes and (i == 0 or line[i - 1] in OPEN_OK):
                    q = ch
                elif line.startswith(marker, i) and (marker != "#" or line[i - 1] in " \t"):
                    c = line[i:]
                    if not (make and re.match(r"^##( |$)", c)):
                        self.emit("E", path, c)
                    break
                i += 1

    def slash_lang(self, path, src, line_comments):
        """' 와 " 는 줄 끝에서 닫는다 — 블록 주석과 달리 줄을 넘겨 이어지지 않는다."""
        q = None
        block = False
        for line in src.splitlines():
            s = line.lstrip()
            if q in ('"', "'"):
                q = None
            lead = not block and q is None and s.startswith(("//", "/*", "*", "{/*"))
            if lead and s.startswith("//"):
                self.rescue(path, s[1:], r"\s(--|//)\s")
            i = 0
            code = False
            while i < len(line):
                ch = line[i]
                nx = line[i + 1] if i + 1 < len(line) else ""
                if block:
                    if ch == "*" and nx == "/":
                        block = False
                        i += 2
                        continue
                    i += 1
                    continue
                if q:
                    if ch == "\\":
                        i += 2
                        continue
                    if ch == q:
                        q = None
                    i += 1
                    continue
                if ch == "\\":
                    i += 2
                    code = True
                    continue
                if line_comments and ch == "/" and nx == "/":
                    if code and not lead:
                        self.emit("E", path, line[i:])
                    break
                if ch == "/" and nx == "*":
                    end = line.find("*/", i + 2)
                    if code and not lead:
                        self.emit("E", path, line[i:] if end < 0 else line[i:end + 2])
                    if end < 0:
                        block = True
                        break
                    i = end + 2
                    continue
                if ch in "\"'`":
                    q = ch
                if not ch.isspace() and ch != "{":
                    code = True
                i += 1

    def scan(self, tag, path):
        try:
            src = open(os.path.join(ROOT, path), encoding="utf-8").read()
        except (UnicodeDecodeError, OSError):
            self.out.append(f"X:{path}:read")
            return
        name = DE_SUF.sub("", os.path.basename(path))
        ext = name.rsplit(".", 1)[1].lower() if "." in name else ""
        if tag == "C":
            n = len(self.out)
            self.slash_lang(path, src, ext != "css")
            if name in ("go.mod", "go.work"):
                self.out[n:] = [o for o in self.out[n:]
                                if not o.endswith(":// indirect")]
        elif tag == "DASH":
            self.marker_lang(path, src, "--", "'\"")
        elif tag in ("HASH", "SHEBANG"):
            if ext in ("py", "pyi") or (tag == "SHEBANG"
                                        and "python" in src.split("\n", 1)[0]):
                self.py(path, src)
            elif (tag == "SHEBANG" or ext in HASH_E_EXT or MAKE.search(name)
                  or name.startswith("requirements")):
                self.marker_lang(path, src, "#", "'\"",
                                 make=bool(MAKE.search(name)))


def measure():
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
            continue
        cand.append(f)

    hits, rest, de = {}, [], DeScanner()
    for f in cand:
        pat = None
        for i, (fre, cre) in enumerate(GROUPS):
            if fre.search(f):
                pat = cre
                if DE_TAG.get(i):
                    de.scan(DE_TAG[i], f)
                break
        else:
            with open(os.path.join(ROOT, f), "rb") as fh:
                if fh.read(2) == b"#!":
                    pat = GROUPS[1][1]
                    de.scan("SHEBANG", f)
                else:
                    rest.append(f)
                    continue
        if pat is None:
            continue
        got = []
        for line in open(os.path.join(ROOT, f), encoding="utf-8",
                         errors="replace").read().split("\n"):
            if not pat.match(line):
                continue
            raw = f"{f}:{line}"
            if DIRECTIVE.search(raw):
                continue
            got.append(re.sub(r"[ \t]+", " ", raw).strip())
        if got:
            hits[f] = got

    surfaces = {s: {} for s in SURFACES}
    surfaces["L"] = hits
    unparsed = []
    for line in de.out:
        kind, path, _ = line.split(":", 2)
        if kind == "X":
            unparsed.append(line)
        else:
            surfaces[kind].setdefault(path, []).append(line)
    return surfaces, rest, unparsed


AXES_OK = re.compile(r"^(—|①?②?③?④?)$")
SURFACE_HEAD = re.compile(r"^## ([LDE]) 표면 — ")


def parse_ledger():
    """bare 이름을 `.sql` 일 때만 `backend/migrations/` 로 resolve 한다 — 확장자 없는
    루트 파일(`Dockerfile`)에 그 규칙을 넓히면 있지도 않은 경로를 찾는다.
    """
    text = open(os.path.join(ROOT, LEDGER), encoding="utf-8").read()
    rows = {s: [] for s in SURFACES}
    prose, in_table, seen_how, surface = [], False, False, None
    for no, line in enumerate(text.split("\n"), 1):
        if line.startswith("## 원장 읽는 법"):
            seen_how = True
        head = SURFACE_HEAD.match(line)
        if head:
            surface = head.group(1)
            continue
        if line.startswith("|"):
            in_table = True
            if line.startswith("| 판정일") or set(line) <= set("|- "):
                continue
            cells = [c.strip() for c in line.strip().strip("|").split(" | ")]
            if len(cells) != 6:
                sys.exit(f"FAIL(I5): {LEDGER}:{no} 열이 6개가 아니다 ({len(cells)})")
            if surface is None:
                sys.exit(f"FAIL(I5): {LEDGER}:{no} 표면 절 밖의 표 — "
                         f"행은 `## <표면> 표면 — …` 절 아래에만 둔다")
            scope = cells[1]
            names = re.findall(r"`([^`]+)`", scope.split("(")[0])
            rows[surface].append(dict(no=no, date=cells[0], scope=scope, axes=cells[4],
                                      files=[n if "/" in n or not n.endswith(".sql")
                                             else "backend/migrations/" + n
                                             for n in names],
                                      lines=cells[2], fp=cells[3].strip("`")))
            continue
        if in_table and line.strip():
            prose.append((no, line))
    if not seen_how:
        sys.exit(f"FAIL(I5): {LEDGER} 에 「원장 읽는 법」 절이 없다")
    missing = [s for s in SURFACES if not rows[s]]
    if missing:
        sys.exit(f"FAIL(I5): {LEDGER} 에 표면 절이 없다: {', '.join(missing)} — "
                 f"표면마다 표를 하나씩 둔다")
    return rows, prose


def check_surface(surface, rows, hits, fail):
    owner = {}
    for r in rows:
        for f in r["files"]:
            if f in owner:
                fail.append(f"I1[{surface}] `{f}` 가 두 행에 있다 — "
                            f"{LEDGER}:{owner[f]} 와 :{r['no']}")
            owner[f] = r["no"]
    unregistered = sorted(set(hits) - set(owner))
    if unregistered:
        fail.append(f"I2[{surface}] 미등재 {len(unregistered)}파일 "
                    f"{sum(len(hits[f]) for f in unregistered)}줄 — 판정하지 않아도 행은 "
                    f"만든다(판정 축 `—`): " + ", ".join(unregistered[:6]))

    judged_lines = judged_rows = 0
    per_axis = {a: 0 for a in "①②③④"}
    for r in rows:
        got = sorted(x for f in r["files"] for x in hits.get(f, []))
        want_fp = hashlib.sha256(("\n".join(got) + "\n").encode()).hexdigest()
        if r["lines"] != str(len(got)):
            fail.append(f"I3[{surface}] {LEDGER}:{r['no']} 줄 수 기재 {r['lines']} ≠ "
                        f"실측 {len(got)} ({r['files'][0]})")
        if r["fp"] != want_fp:
            fail.append(f"I3[{surface}] {LEDGER}:{r['no']} 지문 기재 {r['fp'][:12]}… ≠ "
                        f"실측 {want_fp[:12]}… ({r['files'][0]})")
        if not AXES_OK.match(r["axes"]):
            fail.append(f"I4[{surface}] {LEDGER}:{r['no']} 판정 축 표기 {r['axes']!r} 가 "
                        f"유효하지 않다 — `—` 또는 ①②③④ 의 순서 있는 부분집합")
        if r["axes"] == "①②③④":
            judged_rows += 1
            judged_lines += len(got)
        for a in "①②③④":
            if a not in r["axes"]:
                per_axis[a] += 1
    empty = sorted(f for f in owner if f not in hits)
    return owner, unregistered, empty, judged_lines, judged_rows, per_axis


def main():
    surfaces, unclassified, unparsed = measure()
    rows, prose = parse_ledger()
    fail = []

    if prose:
        fail.append(f"I5 표 뒤 산문 {len(prose)}줄 — 경위·집계는 passes/ 의 자리다: "
                    + "; ".join(f"{LEDGER}:{n} {l[:50]!r}" for n, l in prose[:3]))

    total = sum(len(v) for v in surfaces["L"].values())
    print(f"판정 대상 실측 : lines={total} files={len(surfaces['L'])} "
          f"unclassified={len(unclassified)}")
    print(f"표면 D·E 실측  : "
          f"docstring={sum(len(v) for v in surfaces['D'].values())}/{len(surfaces['D'])} "
          f"eol={sum(len(v) for v in surfaces['E'].values())}/{len(surfaces['E'])} "
          f"unparsed={len(unparsed)}")

    for s in SURFACES:
        hits = surfaces[s]
        n = sum(len(v) for v in hits.values())
        owner, unregistered, empty, jl, jr, per_axis = check_surface(
            s, rows[s], hits, fail)
        print(f"[{s}] 원장 행 {len(rows[s])} · 등재 파일 {len(owner)} · 미등재 "
              f"{len(unregistered)} · 이 표면에서 0행이 된 등재 파일 {len(empty)}")
        print(f"[{s}] 판정 완료(①②③④) {jl}줄 / {jr}행 ({jl * 100 // max(n, 1)}% of "
              f"{n}줄) · 축별 미판정 행 "
              + " · ".join(f"{a} {per_axis[a]}" for a in "①②③④"))
        if empty:
            print(f"[{s}] 0행 등재    : " + ", ".join(empty))
    if unclassified:
        print("미분류(언어군 밖): " + ", ".join(unclassified))
    if unparsed:
        print("파싱 실패(모델 건강 문제): " + ", ".join(unparsed))

    if fail:
        print("\n".join(["", "FAIL — 원장 불변식 위반:"] + [f"  - {m}" for m in fail]))
        return 1
    print("\n통과 — 원장 불변식 I1~I5 이상 없음")
    return 0


if __name__ == "__main__":
    sys.exit(main())
