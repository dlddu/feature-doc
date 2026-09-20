#!/usr/bin/env python3
# 목업 ↔ 구현 렌더링 정합성 게이트 (reconciler `tbm_feature-doc-mockup-render`).
#
# 시각의 단일 소스(SSOT)는 `docs/mockups/` 다. 이 스크립트는 "구현 화면이 대응 목업과
# 어긋나는가"를 기계로 판정한다. 판정 결과가 붉으면 두 길 중 하나로만 닫는다 —
# **구현을 목업에 맞추거나**, `docs/doc-tracker.md` 의 원장에 **사유·해소 시점과 함께
# 등재하거나**. 원장이 유일한 면제 통로이며, 스크립트는 원장 자체의 무결성도 검사한다.
#
# 의존성 0 (python3 stdlib). 형제 게이트 `check-journey-mockup.py` 와 같은 방침이다.
#
# ── 규칙 ──────────────────────────────────────────────────────────────────
#  M0 파싱 무결성   문서의 세 표(활성 대조 · 대조 보류 · 편차 원장)가 파싱되고,
#                   활성 대조 대상이 1건 이상이다.
#  M1 매핑 완비     상단 주석에 목업 매핑을 가진 `frontend/src/*.tsx` 를 화면으로 보고,
#                   참조 목업 파일이 실재하며 `#STP-*` 앵커가 그 파일의 `data-step` 으로
#                   실재한다(dangling 0). 대조 범위 표의 화면 집합이 발견된 화면 집합과
#                   정확히 같다(누락·유령 0).
#  M2 토큰 1:1      `docs/design-system.md` §5.2 `:root` ↔ `frontend/src/index.css` `:root`
#                   의 이름·값 차집합이 0이다.
#  M3 카피 대조     (A) 목업→구현: 활성 단계의 카피가 구현 화면들에 존재한다.
#                   (B) 구현→목업: 활성 쌍의 구현 카피가 그 쌍의 목업 단계에 존재한다.
#                   미존재는 전부 원장에 등재돼야 한다.
#  M4 원장 무결성   행이 5칸이고, 목업 측 백틱 토큰이 **현재 목업에 실재**하며(공전 행 0
#                   — 삭제된 목업을 근거로 남은 행을 잡는다), 구현 측 백틱 토큰이 그
#                   대상 파일에 실재하고, 캡션 집계가 실제 행 수와 같다.
#  M5 래칫          미해소 편차 상한과 대조 보류 상한. 늘면 실패 — 줄이면 상한을 낮추라고
#                   실패한다(형제 게이트의 원장 래칫과 같은 방침).
#  M6 표기 규약     `data-sample` 규약의 오용(잎이 아닌 자리·상호작용 요소)과
#                   `data-variant` 규약의 오용(열거 없는 단독 변이·전진 행동)을 잡는다.
#  M7 앱바 슬롯     활성 (화면, 단계) 쌍마다 목업 `<header class="appbar">` 의 슬롯 수와
#                   구현 앱바의 슬롯 수를 대조한다. 슬롯은 `icon-btn`·`appbar-title`·
#                   `appbar-sub`·`btn-link` 중 하나를 클래스로 가진 요소다. 원장 행의
#                   「목업이 표현하는 것」 칸에 `(앱바 구조)` 마커가 있는 화면은 뺀다 —
#                   `(단계 전체)` 와 같은 방식의 명시적 면제다.
#
# ── 이 게이트가 보지 않는 것(의도적) ─────────────────────────────────────
#  * 규칙 5(구조·수치) 중 **자동화된 것은 앱바 슬롯 수(M7) 하나뿐**이다. 나머지 CSS
#    클래스·px 대조는 여전히 사람 몫이다. M7 을 넣은 이유는 그 한 조각이 카피 게이트의
#    사각지대에 정확히 들어앉기 때문이다 — 슬롯이 통째로 빠져도 카피는 한 글자도 줄지
#    않아 M3 가 영원히 초록이다(2026-09-18, 우측 자리표시자 부재 3건이 그렇게 숨어 있었다).
#  * 실행 스크린샷 픽셀 비교는 모델 정의상 범위 밖이다.
#  * 구현측 카피 추출(M3B)은 **JSX 텍스트 노드 · 한글 포함 문자열 리터럴 ·
#    JSX children 위치의 문자열 리터럴**만 본다. 모듈 상수 테이블에 영문으로만 적힌
#    라벨(예: `STATUS_BADGE` 의 `Queued`)은 잡지 못한다 — 그런 라벨을 가진 화면은
#    「대조 보류」에 있어야 하고, 보류 상한이 그 사실을 붙잡아 둔다.
#    반대 방향의 새는 곳도 하나 막혀 있다 — 인라인 스타일의 **CSS 길이 값**
#    (`letterSpacing: '0.1em'`)은 `=` 앞이 아니라 `:` 앞이라 속성 값 필터에 걸리지
#    않고 카피로 새어 들었다. 길이 리터럴은 제품 카피일 수 없으므로 TECHNICAL 이
#    거른다(2026-09-02, Analysis Progress 승격이 드러냈다 — 원장에 넣었으면 거짓 부채였다).
#    두 방향의 **접근성 이름**은 2026-09-18 에 대칭을 맞췄다. 구현측(M3B)은 한글이 든
#    문자열 리터럴을 속성 값이어도 카피로 세므로 `aria-label="나가기"` 를 집는데,
#    목업측은 태그를 통째로 지워 같은 `aria-label` 을 못 봤다 — 그래서 목업을 그대로
#    옮긴 구현이 「목업에 없는 카피」로 몰렸다(가짜 양성). 목업측도 `placeholder` 처럼
#    `aria-label` 을 뽑아 대조 집합에 넣는다. 이것은 규칙 5 의 사각지대를 닫는 것이
#    **아니다** — M3A 의 건초더미는 화면 전체를 이어 붙인 부분 문자열 검색이라
#    (`앱 닫고 나가기` 가 `나가기` 를 덮는다) 여전히 관대한 쪽으로 튄다.
#  추출 결과는 `--verbose` 로 전부 출력된다. 무엇이 비교됐는지 눈으로 확인할 것.

import html
import re
import sys
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TRACKER = sorted((ROOT / "docs" / "doc-tracker").glob("[0-9][0-9][0-9][0-9]-[0-9][0-9].md"))[-1]
DESIGN_SYSTEM = ROOT / "docs" / "design-system.md"
INDEX_CSS = ROOT / "frontend" / "src" / "index.css"
SRC_DIR = ROOT / "frontend" / "src"
MOCKUP_DIR = ROOT / "docs" / "mockups"

H_ACTIVE = "### 활성 대조 대상"
H_PENDING = "### 대조 보류"
H_LEDGER = "## 알려진 목업↔구현 편차"

failures: list[str] = []
verbose = "--verbose" in sys.argv


def fail(rule: str, message: str) -> None:
    failures.append(f"{rule}: {message}")


def norm(text: str) -> str:
    return re.sub(r"\s+", " ", unicodedata.normalize("NFC", text)).strip()


def note(message: str) -> None:
    if verbose:
        print(f"    · {message}")


# ── 마크다운 표 파싱 ────────────────────────────────────────────────────────
def table_after(doc: str, heading: str) -> list[list[str]]:
    """heading 다음에 처음 나오는 파이프 표의 본문 행(구분선 아래)을 돌려준다.

    표 헤더를 키워드로 거르지 않는다 — 구분선(`|---|`) 기준이 유일하게 안전하다.
    """
    start = doc.find(heading)
    if start < 0:
        return []
    rows: list[list[str]] = []
    seen_divider = False
    for line in doc[start + len(heading):].splitlines():
        stripped = line.strip()
        if stripped.startswith("#") and rows:
            break
        if not stripped.startswith("|"):
            if seen_divider and rows:
                break
            continue
        cells = [c.strip() for c in stripped.strip("|").split("|")]
        if all(re.fullmatch(r":?-{2,}:?", c) for c in cells):
            seen_divider = True
            continue
        if seen_divider:
            rows.append(cells)
    return rows


def is_reference(token: str) -> bool:
    """식별자·파일 경로는 카피가 아니라 참조다 — 문자열 대조 대상에서 뺀다."""
    return token.startswith("STP-") or token.endswith((".tsx", ".html", ".md")) or "/" in token


def ticked(cell: str) -> list[str]:
    """셀 안의 백틱 토큰 — 기계 대조는 이것만 쓰고, 나머지 산문은 사람용이다."""
    return [norm(html.unescape(t)) for t in re.findall(r"`([^`]+)`", cell)]


# ── 목업 파싱 ───────────────────────────────────────────────────────────────
# ── 예시값 표기 규약 (`data-sample`) ────────────────────────────────────────
# 목업이 그리는 문자열 중에는 **그 자리에 실제 데이터가 렌더된다**는 뜻일 뿐인 것이 있다
# (분석 결과 항목명, 근거 파일 경로 …). 구현은 같은 자리에 서버가 준 값을 그리므로 리터럴
# 카피 대조가 성립하지 않는다. 그런 값을 담은 **잎 요소**에 `data-sample` 을 붙이면 카피
# 대조에서 빠진다. 규약 전문은 docs/mockups/README.md 「예시값 표기 규약」.
#
# 잎에만 붙이는 이유: 값을 감싼 행 전체에 붙이면 같은 행의 제품 카피(예: `근거 없음` 태그)
# 까지 함께 사라진다. 숨김은 최소 범위여야 한다.
SAMPLE_OPEN = re.compile(r"<(\w+)(?=[^>]*\bdata-sample\b)([^>]*)>")

# 예시값으로 위장해 대조를 빠져나갈 수 없게 한다 — 상호작용하는 것은 제품 카피다.
INTERACTIVE_ATTR = re.compile(r"\b(data-goto|data-goto-journey|data-cta|id|href|onclick)\b")
INTERACTIVE_TAG = {"a", "button"}


def drop_samples(body: str) -> str:
    """`data-sample` 이 붙은 요소를 내용째 걷어 낸다(같은 태그명 중첩을 세어 짝을 찾는다)."""
    while True:
        m = SAMPLE_OPEN.search(body)
        if not m:
            return body
        tag, depth, i = m.group(1), 1, m.end()
        step = re.compile(r"</?%s\b[^>]*>" % re.escape(tag))
        while depth and i < len(body):
            n = step.search(body, i)
            if not n:
                i = len(body)
                break
            depth += -1 if n.group(0).startswith("</") else 1
            i = n.end()
        body = body[: m.start()] + body[i:]


def sample_misuse() -> list[str]:
    """규약 위반을 사람 리뷰가 아니라 게이트가 잡는다."""
    bad = []
    for path in sorted(MOCKUP_DIR.glob("*.html")):
        src = re.sub(r"<!--.*?-->", "", path.read_text(encoding="utf-8"), flags=re.S)
        for m in SAMPLE_OPEN.finditer(src):
            tag, attrs = m.group(1).lower(), m.group(2)
            hit = INTERACTIVE_ATTR.search(attrs)
            if tag in INTERACTIVE_TAG:
                bad.append(f"docs/mockups/{path.name}: <{tag}> 는 상호작용 요소다 — "
                           f"제품 카피이므로 `data-sample` 을 붙일 수 없다")
            elif hit:
                bad.append(f"docs/mockups/{path.name}: `data-sample` 요소가 상호작용 속성 "
                           f"`{hit.group(1)}` 을 갖는다 — 예시값이 아니라 제품 카피다")
    return bad


# ── 제공자 변이 표기 규약 (`data-variant`) ──────────────────────────────────
# 목업이 그리는 자리 중에는 **사용자의 선택에 따라 문면이 갈리는** 것이 있다(LLM 제공자를
# 고르면 키 형식 안내와 입력 예시가 함께 갈린다). 정적 HTML 은 그런 자리에 첫 변이 한 벌만
# 그릴 수 있고, 구현은 선택을 따라 보간해 그린다 — 그래서 리터럴 대조가 **한 방향으로만**
# 성립한다. `data-variant="<축>"` 으로 묶인 요소는 그 비대칭을 그대로 표기한다:
#
#   * M3A(목업→구현) 에서 **빠진다** — 구현은 한 번에 한 변이만 그리므로 모든 변이가
#     구현에 실재하기를 요구하면 영원히 붉다.
#   * M3B(구현→목업) 의 건초더미에는 **남는다** — 목업이 열거한 변이 **밖**의 문면을
#     구현이 그리면 그것은 여전히 편차다. 열거가 곧 허용 집합이다.
#
# 그래서 이 표기는 면제가 아니라 **열거 의무**다. `data-sample` 과 다른 점이 여기다 —
# 예시값은 양방향에서 빠지지만(구현도 리터럴을 갖지 않는다), 변이는 구현이 그중 하나를
# 리터럴로 갖는다. 규약 전문은 docs/mockups/README.md 「제공자 변이 표기 규약」.
VARIANT_OPEN = re.compile(r"<(\w+)(?=[^>]*\bdata-variant=)([^>]*)>")
VARIANT_KEY = re.compile(r'\bdata-variant="([^"]*)"')
VOID_TAGS = {"input", "img", "br", "hr", "meta", "source", "area", "col", "embed"}
# 전진 행동은 변이로 감출 수 없다 — 무엇을 눌러 다음 단계로 가는지는 선택과 무관한 제품 카피다.
VARIANT_FORBIDDEN = re.compile(r"\b(data-goto|data-goto-journey|data-cta)\b")


def split_variants(body: str) -> tuple[str, str]:
    """`data-variant` 요소를 M3A 판정 대상에서 떼어 내고 따로 모아 돌려준다."""
    kept: list[str] = []
    taken: list[str] = []
    pos = 0
    while True:
        m = VARIANT_OPEN.search(body, pos)
        if not m:
            kept.append(body[pos:])
            return "".join(kept), "".join(taken)
        kept.append(body[pos : m.start()])
        tag = m.group(1).lower()
        if tag in VOID_TAGS:
            end = m.end()
        else:
            depth, i = 1, m.end()
            step = re.compile(r"</?%s\b[^>]*>" % re.escape(tag))
            while depth and i < len(body):
                n = step.search(body, i)
                if not n:
                    i = len(body)
                    break
                depth += -1 if n.group(0).startswith("</") else 1
                i = n.end()
            end = i
        taken.append(body[m.start() : end])
        pos = end


def variant_misuse() -> list[str]:
    """열거 의무를 게이트가 지킨다 — 변이가 하나뿐이면 그것은 열거가 아니라 은닉이다."""
    bad = []
    for path in sorted(MOCKUP_DIR.glob("*.html")):
        src = re.sub(r"<!--.*?-->", "", path.read_text(encoding="utf-8"), flags=re.S)
        for part in re.split(r'(?=<section id=")', src):
            head = re.match(r'<section id="([^"]+)"([^>]*)>', part)
            if not head or "data-step=" not in head.group(2):
                continue
            seen: dict[str, int] = {}
            for m in VARIANT_OPEN.finditer(part):
                attrs = m.group(2)
                key = (VARIANT_KEY.search(attrs) or [None, ""])[1]
                seen[key] = seen.get(key, 0) + 1
                hit = VARIANT_FORBIDDEN.search(attrs)
                if hit:
                    bad.append(f"docs/mockups/{path.name}#{head.group(1)}: `data-variant` "
                               f"요소가 전진 속성 `{hit.group(1)}` 을 갖는다 — 선택과 무관한 제품 카피다")
            for key, count in sorted(seen.items()):
                if count < 2:
                    bad.append(f"docs/mockups/{path.name}#{head.group(1)}: "
                               f"`data-variant=\"{key}\"` 가 {count}개뿐이다 — 변이는 "
                               f"**열거**해야 한다(2개 이상). 한 벌만 숨기는 것은 면제다")
    return bad


def mockup_steps(path: Path) -> dict[str, dict[str, list[str]]]:
    """`data-step` 섹션별 가시 텍스트(+ placeholder).

    단계마다 두 벌을 돌려준다 — `judged` 는 M3A 가 「구현에 실재해야 한다」고 요구하는
    카피이고, `pool` 은 M3B 가 「구현이 그려도 되는 것」으로 인정하는 건초더미다. 변이
    (`data-variant`)는 뒤쪽에만 들어간다.
    """
    src = path.read_text(encoding="utf-8")
    steps: dict[str, list[str]] = {}
    for part in re.split(r'(?=<section id=")', src):
        head = re.match(r'<section id="([^"]+)"([^>]*)>', part)
        if not head or "data-step=" not in head.group(2):
            continue
        body = re.sub(r"<script.*?</script>", "", part, flags=re.S)
        body = re.sub(r"<style.*?</style>", "", body, flags=re.S)
        body = re.sub(r"<!--.*?-->", "", body, flags=re.S)
        body = drop_samples(body)
        judged_body, variant_body = split_variants(body)

        def harvest(part: str) -> list[str]:
            placeholders = re.findall(r'placeholder="([^"]*)"', part)
            aria = re.findall(r'aria-label="([^"]*)"', part)
            text = re.sub(r"<[^>]+>", "\x00", part).split("\x00")
            return [c for c in (norm(html.unescape(t))
                                for t in text + placeholders + aria) if c]

        judged = harvest(judged_body)
        steps[head.group(1)] = {"judged": judged, "pool": judged + harvest(variant_body)}
    return steps


def all_mockup_text() -> str:
    """공전 행 검사용 — 모든 목업 파일의 원문(주석 제외)을 이어 붙인 건초더미."""
    parts = []
    for path in sorted(MOCKUP_DIR.glob("*.html")):
        src = re.sub(r"<!--.*?-->", "", path.read_text(encoding="utf-8"), flags=re.S)
        parts.append(norm(html.unescape(src)))
    for path in sorted(MOCKUP_DIR.glob("*.md")):
        parts.append(norm(path.read_text(encoding="utf-8")))
    return " \x01 ".join(parts)


UNIT_ONLY = re.compile(r"(?i)(min|mins|sec|secs|hr|hrs|mb|kb|gb|b)")


def is_copy(chunk: str) -> bool:
    """샘플 수치·기호를 걸러 내고 '제품 카피'만 남긴다.

    `847 · 2.3 MB` · `~6 min` · `~$0.80` 같은 예시 값은 카피가 아니다 — 구현은 실제
    값을 그리므로 문자열로 대조할 대상이 아니며, 대조하면 영구히 붉은 행이 된다.
    """
    if len(chunk) < 2:
        return False
    letters = re.sub(r"[^A-Za-z가-힣]", "", chunk)
    if len(letters) < 2:
        return False
    return not UNIT_ONLY.fullmatch(letters)


# ── 구현 파싱 ───────────────────────────────────────────────────────────────
def strip_comments(src: str) -> str:
    src = re.sub(r"/\*.*?\*/", "", src, flags=re.S)
    return re.sub(r"(?m)^\s*//.*$", "", src)


STRING_LITERAL = re.compile(r"""(?<!\\)(['"`])((?:(?!\1)[^\\\n]|\\.)*)\1""")
TECHNICAL = re.compile(
    r"""^(?:
          [a-z][a-zA-Z0-9]*                      # idle, checking, anthropic …
        | [a-z][a-z0-9-]*(?:\s+[a-z][a-z0-9-]*)* # btn btn-primary block …
        | [./#][^\s]*                            # ./api, #/analyses/…
        | var\(--[a-z-]+\)
        | -?\d+(?:\.\d+)?(?:px|em|rem|ch|vh|vw|fr|deg|%|s|ms)  # 0.1em, 1.5rem …
      )$""",
    re.X,
)


def literal_spans(src: str) -> list[tuple[int, str]]:
    return [(m.start(), m.group(2)) for m in STRING_LITERAL.finditer(src)]


CODE_ISH = re.compile(r"[;=]")


def jsx_text_nodes(src: str) -> list[str]:
    """태그 사이의 순수 텍스트.

    ⚠️ TypeScript 제네릭(`useState<string | null>(null); const …`)도 `>`…`<` 사이에
    걸린다. JSX 텍스트에는 `;` 나 `=` 가 나타나지 않으므로 그것으로 코드를 가른다.
    """
    return [norm(t) for t in re.findall(r">([^<>{}]+)<", src) if not CODE_ISH.search(t)]


def is_attribute_value(src: str, start: int) -> bool:
    """리터럴 바로 앞이 `=` 또는 `={` 이면 속성 값이다(카피가 아니다)."""
    i = start - 1
    while i >= 0 and src[i] in " \t\n":
        i -= 1
    if i >= 0 and src[i] == "{":
        i -= 1
        while i >= 0 and src[i] in " \t\n":
            i -= 1
    return i >= 0 and src[i] == "="


HANGUL = re.compile(r"[가-힣]")


def impl_copy(path: Path) -> list[str]:
    """구현 화면이 실제로 그리는 카피(보수적 추출 — 헤더의 한계 설명 참조)."""
    src = strip_comments(path.read_text(encoding="utf-8"))
    found: list[str] = [t for t in jsx_text_nodes(src) if t]
    for start, raw in literal_spans(src):
        text = norm(raw)
        if not text:
            continue
        if HANGUL.search(text):
            found.append(text)                       # 한글 = 제품 카피 신호
            continue
        if is_attribute_value(src, start):
            continue
        if TECHNICAL.match(text):
            continue
        found.append(text)
    return [t for t in dict.fromkeys(found) if is_copy(t)]


def impl_haystack(paths: list[Path]) -> str:
    """목업→구현 방향의 건초더미.

    태그 이름(`<main>`)이 카피로 오인되지 않도록 원문이 아니라 **문자열 리터럴 +
    JSX 텍스트 노드**만 모은다. 속성 값(testid 등)도 포함하는 관대한 집합이라,
    '없는데 있다고 판정'하는 쪽이 아니라 '있는데 없다고 판정'하는 쪽으로만 튄다.
    """
    parts: list[str] = []
    for path in paths:
        src = strip_comments(path.read_text(encoding="utf-8"))
        parts.extend(jsx_text_nodes(src))
        parts.extend(norm(raw) for _, raw in literal_spans(src))
    return " \x01 ".join(p for p in parts if p)


# ── M1 매핑 발견 ────────────────────────────────────────────────────────────
MAPPING_REF = re.compile(r"docs/mockups/([A-Za-z0-9._-]+\.html)((?:#STP-[a-z0-9-]+)?)")


def discover_screens() -> dict[str, list[tuple[str, str]]]:
    """상단 주석에 목업 매핑이 있는 tsx → [(목업파일, 앵커)]."""
    screens: dict[str, list[tuple[str, str]]] = {}
    for path in sorted(SRC_DIR.glob("*.tsx")):
        header = path.read_text(encoding="utf-8")[:2000]
        refs = [(m.group(1), m.group(2).lstrip("#")) for m in MAPPING_REF.finditer(header)]
        if refs:
            screens[f"frontend/src/{path.name}"] = refs
    return screens


# ── M7 앱바 슬롯 ────────────────────────────────────────────────────────────
# 앱바는 목업 전 페이지에서 **슬롯의 나열**이다 — 좌·중앙·우가 각각 하나씩이거나(3슬롯),
# `JRN-connect-repo` 의 홈 계열처럼 제목 + 링크 둘이거나(2슬롯). 카피 대조는 이 축을 볼 수
# 없다: 자리표시자(`icon-btn ghost`)는 글자가 없고, 제목을 감싼 래퍼가 바뀌어도 텍스트는
# 그대로다. 그래서 슬롯 **수**만 따로 센다. 어느 컨트롤이 어느 슬롯에 있어야 하는가는
# 여전히 카피 대조와 원장의 몫이다.
APPBAR_MARKER = "(앱바 구조)"
SLOT_CLASSES = {"icon-btn", "appbar-title", "appbar-sub", "btn-link"}
CLASS_ATTR = re.compile(r'class(?:Name)?="([^"]*)"')


def count_slots(block: str) -> int:
    return sum(1 for m in CLASS_ATTR.finditer(block)
               if m.group(1).split()[:1] and m.group(1).split()[0] in SLOT_CLASSES)


def mockup_appbar(path: Path, step: str) -> str | None:
    src = path.read_text(encoding="utf-8")
    section = re.search(r'<section id="%s"(.*?)(?=<section id="|\Z)' % re.escape(step),
                        src, flags=re.S)
    if not section:
        return None
    header = re.search(r'<header class="appbar">(.*?)</header>', section.group(1), flags=re.S)
    return header.group(1) if header else None


def impl_appbar(path: Path) -> str | None:
    src = strip_comments(path.read_text(encoding="utf-8"))
    header = re.search(r'<header className="appbar">(.*?)</header>', src, flags=re.S)
    return header.group(1) if header else None


def root_tokens(text: str) -> dict[str, str]:
    block = re.search(r":root\s*\{(.*?)\}", text, flags=re.S)
    if not block:
        return {}
    return {
        m.group(1): norm(m.group(2))
        for m in re.finditer(r"(--[a-z0-9-]+)\s*:\s*([^;]+);", block.group(1))
    }


def main() -> int:
    doc = TRACKER.read_text(encoding="utf-8")

    # ── M0 ──────────────────────────────────────────────────────────────
    active_rows = table_after(doc, H_ACTIVE)
    pending_rows = table_after(doc, H_PENDING)
    ledger_rows = table_after(doc, H_LEDGER)
    if not active_rows:
        fail("M0", f"「{H_ACTIVE}」 표를 찾지 못했거나 비어 있다")
    if not ledger_rows:
        fail("M0", f"「{H_LEDGER}」 표를 찾지 못했거나 비어 있다")
    for name, rows, width in (("활성 대조", active_rows, 3), ("대조 보류", pending_rows, 5),
                              ("편차 원장", ledger_rows, 6)):
        for row in rows:
            if len(row) != width:
                fail("M0", f"{name} 표의 행 칸 수가 {width}가 아니다: {row}")
    if failures:
        report()
        return 1
    print(f"M0 파싱 무결성 — 활성 {len(active_rows)} · 보류 {len(pending_rows)} · 편차 {len(ledger_rows)}")

    active: dict[str, list[str]] = {}
    for screen_cell, mockup_cell, steps_cell in active_rows:
        screen = (ticked(screen_cell) or [""])[0]
        active[screen] = [s for s in ticked(steps_cell) if s.startswith("STP-")]
        note(f"활성 {screen} ↔ {(ticked(mockup_cell) or ['?'])[0]} {active[screen]}")
    pending = {(ticked(r[0]) or [""])[0] for r in pending_rows}

    # ── M1 ──────────────────────────────────────────────────────────────
    screens = discover_screens()
    steps_by_file: dict[str, dict[str, dict[str, list[str]]]] = {}
    for screen, refs in screens.items():
        for filename, anchor in refs:
            path = MOCKUP_DIR / filename
            if not path.exists():
                fail("M1", f"{screen} 의 매핑이 없는 목업을 가리킨다: docs/mockups/{filename}")
                continue
            steps_by_file.setdefault(filename, mockup_steps(path))
            if anchor and anchor not in steps_by_file[filename]:
                fail("M1", f"{screen} 의 매핑 앵커가 dangling 이다: {filename}#{anchor}")
    declared = set(active) | pending
    if declared != set(screens):
        for missing in sorted(set(screens) - declared):
            fail("M1", f"화면 {missing} 가 대조 범위 표에 없다 — 활성이나 보류로 등재할 것")
        for ghost in sorted(declared - set(screens)):
            fail("M1", f"대조 범위 표의 {ghost} 는 매핑을 가진 화면이 아니다(유령 행)")
    print(f"M1 매핑 완비 — 화면 {len(screens)}개, dangling 앵커 0, 범위 표 일치")

    # ── M2 ──────────────────────────────────────────────────────────────
    ds_section = DESIGN_SYSTEM.read_text(encoding="utf-8")
    ds_start = ds_section.find("### 5.2")
    spec = root_tokens(ds_section[ds_start:]) if ds_start >= 0 else {}
    impl = root_tokens(INDEX_CSS.read_text(encoding="utf-8"))
    if not spec:
        fail("M2", "design-system.md §5.2 의 `:root` 를 파싱하지 못했다")
    for name in sorted(set(spec) - set(impl)):
        fail("M2", f"토큰 {name} 이 index.css 에 없다")
    for name in sorted(set(impl) - set(spec)):
        fail("M2", f"토큰 {name} 이 index.css 에만 있다(디자인 시스템 밖의 새 값)")
    for name in sorted(set(spec) & set(impl)):
        if spec[name] != impl[name]:
            fail("M2", f"토큰 {name} 값 불일치 — 목업 `{spec[name]}` vs 구현 `{impl[name]}`")
    print(f"M2 토큰 1:1 — {len(spec)}개 이름·값 일치")

    # ── 원장 인덱스 ─────────────────────────────────────────────────────
    exempt_steps: set[str] = set()
    exempt_strings: set[str] = set()
    appbar_exempt: set[str] = set()
    for target_cell, mockup_cell, impl_cell, _kind, _why, _when in ledger_rows:
        if APPBAR_MARKER in mockup_cell:
            appbar_exempt.update(t for t in ticked(target_cell) if t.endswith(".tsx"))
        tokens = ticked(mockup_cell)
        if "(단계 전체)" in mockup_cell:
            exempt_steps.update(t for t in tokens if t.startswith("STP-"))
        exempt_strings.update(t for t in tokens if not t.startswith("STP-"))
        exempt_strings.update(ticked(impl_cell))
        note(f"원장 {(ticked(target_cell) or ['?'])[0]}: 목업{tokens} 구현{ticked(impl_cell)}")

    # ── M3A 목업 → 구현 ─────────────────────────────────────────────────
    active_steps = {s for steps in active.values() for s in steps}
    haystack = impl_haystack([ROOT / s for s in screens])
    undocumented: list[str] = []
    judged = 0
    for filename, steps in steps_by_file.items():
        for step, chunks in steps.items():
            if step not in active_steps or step in exempt_steps:
                continue
            for chunk in chunks["judged"]:
                if not is_copy(chunk):
                    continue
                judged += 1
                if chunk in haystack or chunk in exempt_strings:
                    note(f"M3A ok [{step}] {chunk}")
                    continue
                undocumented.append(f"[{step}] {chunk}")
    for item in undocumented:
        fail("M3A", f"목업 카피가 구현에 없고 원장에도 없다 — {item}")
    print(f"M3A 목업→구현 — 활성 단계 {len(active_steps)}개 / 카피 {judged}건 대조, "
          f"미등재 {len(undocumented)}건")

    # ── M3B 구현 → 목업 ─────────────────────────────────────────────────
    extra: list[str] = []
    checked = 0
    for screen, steps in active.items():
        pool = " \x01 ".join(
            chunk
            for filename in steps_by_file
            for step in steps
            for chunk in steps_by_file[filename].get(step, {}).get("pool", [])
        )
        for chunk in impl_copy(ROOT / screen):
            checked += 1
            if chunk in pool or chunk in exempt_strings:
                note(f"M3B ok [{screen}] {chunk}")
                continue
            extra.append(f"[{screen}] {chunk}")
        note(f"M3B {screen}: 구현 카피 {checked}건")
    for item in extra:
        fail("M3B", f"구현 카피가 목업에 없고 원장에도 없다 — {item}")
    print(f"M3B 구현→목업 — 활성 쌍 {len(active)}개 / 카피 {checked}건 대조, 미등재 {len(extra)}건")

    # ── M4 원장 무결성 ──────────────────────────────────────────────────
    mockup_text = all_mockup_text()
    phantom = 0
    for target_cell, mockup_cell, impl_cell, kind_cell, why_cell, when_cell in ledger_rows:
        targets = ticked(target_cell)
        label = targets[0] if targets else target_cell
        if not kind_cell.strip() or not why_cell.strip() or not when_cell.strip():
            fail("M4", f"원장 행 [{label}] 에 유형·사유·해소 시점 중 빈 칸이 있다")
        for token in ticked(mockup_cell):
            if token.endswith((".tsx", ".html", ".md")):
                continue
            if token.startswith("STP-"):
                if not any(token in steps for steps in steps_by_file.values()):
                    fail("M4", f"원장 행 [{label}] 의 단계 `{token}` 가 목업에 없다(공전 행)")
                continue
            if token not in mockup_text:
                fail("M4", f"원장 행 [{label}] 의 목업 문자열 `{token}` 이 현재 목업에 없다(공전 행)")
                phantom += 1
        # 대상 파일은 「대상」 칸뿐 아니라 「현재 구현」 칸에서도 찾는다 — 단계 단위 행은
        # 대상이 목업 파일이고 어느 화면이 그 단계를 그리는지는 구현 칸에 적히기 때문이다.
        files = [t for t in targets + ticked(impl_cell)
                 if t.endswith(".tsx") and (ROOT / t).exists()]
        if not files:
            files = [t for t in targets + ticked(impl_cell)
                     if t.endswith(".tsx") and (SRC_DIR / t).exists()]
            files = [f"frontend/src/{t}" for t in files]
        for token in ticked(impl_cell):
            if is_reference(token):
                continue
            # 소스는 줄바꿈·들여쓰기로 카피를 쪼개 놓으므로 정규화 후 대조한다.
            if files and not any(token in norm((ROOT / t).read_text(encoding="utf-8"))
                                 for t in files):
                fail("M4", f"원장 행 [{label}] 의 구현 문자열 `{token}` 이 그 파일에 없다(공전 행)")
                phantom += 1
    caption = re.search(r"알려진 편차:\s*\*\*(\d+)건\*\*", doc)
    if not caption:
        fail("M4", "원장 캡션 「알려진 편차: **N건**」 을 찾지 못했다")
    elif int(caption.group(1)) != len(ledger_rows):
        fail("M4", f"원장 캡션 집계 {caption.group(1)}건 ≠ 실제 행 수 {len(ledger_rows)}건")
    print(f"M4 원장 무결성 — 행 {len(ledger_rows)}건, 공전 행 {phantom}건")

    # ── M5 래칫 ─────────────────────────────────────────────────────────
    for label, pattern, actual in (
        ("미해소 편차", r"미해소 편차 상한:\s*\*\*(\d+)\*\*", len(ledger_rows)),
        ("대조 보류", r"대조 보류 상한:\s*\*\*(\d+)\*\*", len(pending_rows)),
    ):
        found = re.search(pattern, doc)
        if not found:
            fail("M5", f"「{label} 상한: **N**」 문구를 찾지 못했다")
            continue
        cap = int(found.group(1))
        if actual > cap:
            fail("M5", f"{label} {actual}건 > 상한 {cap} — 늘릴 수 없다")
        elif actual < cap:
            fail("M5", f"{label} 가 {actual}건으로 줄었다 — 상한을 {actual}로 낮출 것(래칫)")
        else:
            print(f"M5 래칫 — {label} {actual}/{cap}")

    # ── M6 예시값 표기 규약 ─────────────────────────────────────────────
    marked = sum(len(SAMPLE_OPEN.findall(re.sub(r"<!--.*?-->", "", p.read_text(encoding="utf-8"),
                                                flags=re.S)))
                 for p in sorted(MOCKUP_DIR.glob("*.html")))
    varied = sum(len(VARIANT_OPEN.findall(re.sub(r"<!--.*?-->", "", p.read_text(encoding="utf-8"),
                                                 flags=re.S)))
                 for p in sorted(MOCKUP_DIR.glob("*.html")))
    misuse = sample_misuse() + variant_misuse()
    for message in misuse:
        fail("M6", message)
    print(f"M6 예시값·변이 표기 — `data-sample` {marked}건 · `data-variant` {varied}건, "
          f"오용 {len(misuse)}건")

    # ── M7 앱바 슬롯 ────────────────────────────────────────────────────
    compared = 0
    mismatched = 0
    exempted = 0
    for screen, steps in active.items():
        if screen in appbar_exempt:
            exempted += 1
            note(f"M7 면제 [{screen}] — 원장 {APPBAR_MARKER} 행")
            continue
        # 앱바가 **양쪽 다 없는** 화면이 있다 — 온보딩 계열(`STP-sign-in`)은 목업이
        # 앱바 대신 브랜드 `toprow` 를 쓴다. 목업 쪽을 먼저 풀어야 이것을 「구현이
        # 앱바를 빠뜨렸다」와 구분할 수 있다: 목업에 앱바가 없으면 구현에도 없어야
        # 맞고(합의), 목업에 있는데 구현에 없으면 그때가 누락이다.
        blocks = {
            step: next(
                (b for filename, _ in screens.get(screen, [])
                 if (b := mockup_appbar(MOCKUP_DIR / filename, step)) is not None),
                None,
            )
            for step in steps
        }
        block = impl_appbar(ROOT / screen)
        if all(b is None for b in blocks.values()):
            if block is not None:
                fail("M7", f"{screen} 의 목업에는 앱바가 없는데 구현은 앱바를 그린다")
            else:
                note(f"M7 앱바 없음 일치 [{screen}] — 목업이 `toprow` 계열")
            continue
        if block is None:
            fail("M7", f'{screen} 에서 `<header className="appbar">` 를 찾지 못했다')
            continue
        impl_slots = count_slots(block)
        for step in steps:
            found = blocks[step]
            if found is None:
                fail("M7", f"{screen} 의 활성 단계 {step} 에서 목업 앱바를 찾지 못했다")
                continue
            compared += 1
            mockup_slots = count_slots(found)
            if mockup_slots != impl_slots:
                mismatched += 1
                fail("M7", f"앱바 슬롯 수가 다르다 [{screen} ↔ {step}] — "
                           f"목업 {mockup_slots} vs 구현 {impl_slots}")
            else:
                note(f"M7 ok [{screen} ↔ {step}] 슬롯 {mockup_slots}")
    print(f"M7 앱바 슬롯 — 대조 {compared}쌍 · 면제 {exempted}화면 · 불일치 {mismatched}건")

    report()
    return 1 if failures else 0


def report() -> None:
    if failures:
        print()
        print(f"✗ 위반 {len(failures)}건")
        for item in failures:
            print(f"  - {item}")
    else:
        print()
        print("✓ 목업 ↔ 구현 렌더링 정합성: 전 규칙 통과")


if __name__ == "__main__":
    sys.exit(main())
