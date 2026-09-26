#!/usr/bin/env python3
"""주석 판정 원장(docs/comment-policy/ledger.md)의 불변식을 CI에서 강제한다.

원장은 행 단위 사실의 표다. 손으로 적은 합계는 갱신되지 않아 낡고, 모든 PR이 그 한 줄을
고치면 파일 집합이 서로소인 PR끼리도 충돌한다. 그래서 집계는 저장하지 않고 이 게이트가
표와 실측에서 계산해 출력한다.

지문의 표면은 셋이고 원장도 표면마다 표를 하나씩 둔다 — L 줄머리 주석, D Python docstring
본문, E 코드 뒤에 오는 줄 끝·줄 중간 주석. 표면 하나가 게이트 밖에 있으면 그 표는 손으로
유지하는 값이 되고 실측과 갈라져도 통과한다. 그래서 세 표면을 한 게이트가 함께 잰다.

강제하는 것(어기면 rc=1):
  I1  표면의 판정 대상 줄을 가진 파일은 그 표면의 표에서 정확히 한 행에 속한다 — 중복 소속 없음
  I2  그 파일 집합의 합집합이 그 표면의 실측 파일 집합을 덮는다 — 미등재 없음
  I3  행의 줄 수·지문이 그 표면의 실측과 같다
  I4  판정 축 표기가 유효하다 (표면 무관: `—` 또는 ①②③④의 순서 있는 부분집합)
  I5  표면 절 셋이 모두 있고, 표와 「원장 읽는 법」·표면 절 제목 밖에 산문이 없다

출력만 하는 것(실패가 아니다): 표면별 판정 완료 줄·행 수, 축별 미판정 행 수, 그 표면에서
주석이 0행이 된 등재 파일, 파싱 실패 수. 미판정(`—`)은 판정하지 않았다는 사실을 행에 남긴
정상 상태이고, 그 행은 다음 판정 슬라이스가 가져간다. 파싱 실패는 모델 쪽 건강 문제라
여기서 실패시키지 않고 수를 드러내기만 한다.

측정 범위와 표면 추출은 모델 `asIs.versionScript`와 **같은 규칙**이다 — 레포 전체에서 제외 세
부류를 뺀 것, 같은 언어군 표, 같은 `comment-scope-exclude` 블록, 그리고 같은 D·E 스캐너.
게이트가 자기 규칙을 따로 들고 있으면 둘이 갈라져, 지문에는 있는데 원장 불변식은 모르는
주석이 생긴다.
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


class DeScanner:
    """D·E 표면 추출 — 모델 `asIs.versionScript` 의 DE 블록을 논리 무수정으로 이식한 것이다.

    옮긴 것은 바깥 껍데기뿐이다: 전역 `out` 리스트가 인스턴스 속성이 되고, 파일을 레포
    루트 기준으로 연다. 규칙(무엇을 D·E로 보는가, 어떤 파일에 어떤 스캐너가 붙는가)은 한
    글자도 바꾸지 않는다 — 규칙을 손으로 옮겨 적으면 게이트와 지문이 갈라지고, 그러면
    지문에는 잡히는데 원장은 모르는 주석이 다시 생긴다. 등가는 편집 전 트리에서 지문
    스크립트의 D·E 덤프와 이 클래스의 출력을 정렬해 바이트 대조하는 것으로 증명한다.

    출력 줄은 지문과 같은 꼴(`D:경로:본문` · `E:경로:본문`)이고, 파싱에 실패한 파일은
    `X:경로:사유` 로 남는다 — 조용히 표면에서 빠지지 않게 하기 위해서다.
    """

    def __init__(self):
        self.out = []

    def emit(self, kind, path, text):
        t = re.sub(r"\s+", " ", text).strip()
        if t and not (kind == "E" and DE_DIRECTIVE.search(t)):
            self.out.append(f"{kind}:{path}:{t}")

    def rescue(self, path, comment, tail):
        """기계가 읽는 주석 뒤에 사람이 덧붙인 사유를 E 로 건져 낸다.

        L 은 그 줄을 지시자와 함께 통째로 버리므로, 여기서 건지지 않으면 사유가 어느
        표면에도 잡히지 않는다. 사유는 기계가 읽지 않는 사람의 문장이라 판정 대상이다.
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
        """`#`·`--` 계열 줄 스캐너 — 줄머리 주석은 L 몫이라 건너뛰고 코드 뒤만 E 로 낸다."""
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
        """C 언어군 줄 스캐너 — 문자열(" ' `)과 블록 주석 상태를 줄을 넘어 따라간다.

        ' 와 " 는 줄 끝에서 닫는다. 줄머리에서 시작한 주석은 L 몫이라 내지 않는다.
        """
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
        """한 파일에 그 언어군의 스캐너를 붙인다 — 붙는 스캐너가 없으면 그 파일은 표면 밖이다.

        멤버십은 언어군이 아니라 **스캐너가 붙는가**로 정해진다. 줄 중간 `#` 이 주석이 아닌
        파일(Dockerfile·gitignore류·환경 파일·ini/cfg)은 `#` 언어군이면서도 E 스캐너가 붙지
        않는다. 표면 밖 파일을 표에 등재하면 영원히 0줄인 행이 되어, 묻지 않은 것이 판정된
        것처럼 보인다.
        """
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
    """실측 = ({표면: {파일: [정규화된 줄, ...]}}, 미분류 경로, 파싱 실패 줄)."""
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
    """표면 절마다 표를 하나씩 파싱한다 — 반환은 {표면: [행, ...]} 과 표 밖 산문이다.

    범위 칸의 백틱 경로만 파일이고 첫 여는 괄호 뒤는 덩어리 이름이다. 경로가 없는 bare
    이름은 `.sql` 일 때만 `backend/migrations/` 로 resolve 한다 — 마이그레이션 행이 같은
    디렉터리의 파일을 이름만으로 잇기 때문이다. 확장자 없는 루트 파일(`Dockerfile`)에 그
    규칙을 적용하면 있지도 않은 경로를 찾는다.
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
    """한 표면의 I1~I4 를 걸고 그 표면의 집계를 돌려준다 — 불변식은 표면 안에서 닫힌다.

    한 파일이 표면마다 다른 행에 드는 것은 정상이다(같은 파일의 줄머리 주석과 docstring 은
    서로 다른 덩어리에서 판정될 수 있다). 행을 요구하는 것은 그 표면에 줄이 **있는** 파일
    뿐이고, 줄이 0인 파일은 등재하지 않는다.
    """
    owner = {}                                                           # I1
    for r in rows:
        for f in r["files"]:
            if f in owner:
                fail.append(f"I1[{surface}] `{f}` 가 두 행에 있다 — "
                            f"{LEDGER}:{owner[f]} 와 :{r['no']}")
            owner[f] = r["no"]
    unregistered = sorted(set(hits) - set(owner))                        # I2
    if unregistered:
        fail.append(f"I2[{surface}] 미등재 {len(unregistered)}파일 "
                    f"{sum(len(hits[f]) for f in unregistered)}줄 — 판정하지 않아도 행은 "
                    f"만든다(판정 축 `—`): " + ", ".join(unregistered[:6]))

    judged_lines = judged_rows = 0
    per_axis = {a: 0 for a in "①②③④"}
    for r in rows:
        got = sorted(x for f in r["files"] for x in hits.get(f, []))
        want_fp = hashlib.sha256(("\n".join(got) + "\n").encode()).hexdigest()
        if r["lines"] != str(len(got)):                                   # I3
            fail.append(f"I3[{surface}] {LEDGER}:{r['no']} 줄 수 기재 {r['lines']} ≠ "
                        f"실측 {len(got)} ({r['files'][0]})")
        if r["fp"] != want_fp:
            fail.append(f"I3[{surface}] {LEDGER}:{r['no']} 지문 기재 {r['fp'][:12]}… ≠ "
                        f"실측 {want_fp[:12]}… ({r['files'][0]})")
        if not AXES_OK.match(r["axes"]):                                  # I4
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

    if prose:                                                            # I5
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
