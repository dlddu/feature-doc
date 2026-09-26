#!/usr/bin/env python3
# 목업 ↔ 구현 렌더링 정합성 게이트 (reconciler `tbm_feature-doc-mockup-render`).
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
#  M8 접힘 affordance  구현이 목업에 없는 접힘 기구(`<details class="…disclosure…">`)를 쓸 때,
#                   그것이 펼쳐진 채로 서는 폭(= `viewport.ts` 의 `WIDE_QUERY`)에서는 제목 줄에
#                   affordance(`cursor: pointer` · `summary::after` 표식)가 남아 있지 않다 —
#                   그 폭의 목업은 같은 줄을 정적인 `div.section-title` 로 그리기 때문이다.
#  M7 앱바 슬롯     활성 (화면, 단계) 쌍마다 목업 `<header class="appbar">` 의 슬롯 수와
#                   구현 앱바의 슬롯 수를 대조한다. 슬롯은 `icon-btn`·`appbar-title`·
#                   `appbar-sub`·`btn-link` 중 하나를 클래스로 가진 요소다. 원장 행의
#                   「목업이 표현하는 것」 칸에 `(앱바 구조)` 마커가 있는 화면은 뺀다 —
#                   `(단계 전체)` 와 같은 방식의 명시적 면제다.
#  M9 인라인 선언   목업 `id` ↔ 구현 `data-testid` 로 짝지어지고 **클래스 집합이 같은**
#                   요소 쌍마다, 인라인 style 의 선언 집합이 같다. 표기 차이는 환산하고
#                   (`margin-top:16px` ↔ `marginTop: 16`), 프로토타입 전용 장치
#                   (`display` · `.on`)와 값이 식인 속성은 대조에서 뺀다 — 식은 목업의
#                   예시값과 비교할 수 없으므로 `data-sample` 과 같은 이유로 빠진다.
#
# ── 이 게이트가 보지 않는 것(의도적) ─────────────────────────────────────
#  * 규칙 5(구조·수치) 중 **자동화된 것은 앱바 슬롯 수(M7) · 접힘 affordance 폭(M8) · 짝지어진 요소의
#    인라인 선언(M9) 셋**이다. 규칙 CSS(클래스 선언 자체)의 px·색 대조는 여전히 사람 몫이다.
#    M7 을 넣은 이유는 그 한 조각이 카피 게이트의
#    사각지대에 정확히 들어앉기 때문이다 — 슬롯이 통째로 빠져도 카피는 한 글자도 줄지
#    않아 M3 가 영원히 초록이다(2026-09-18, 우측 자리표시자 부재 3건이 그렇게 숨어 있었다).
#    M8 도 같은 자리다 — 표식 `▴` 가 목업에 없는 폭에 그려져도 카피는 한 글자도 줄지 않는다(2026-09-25).
#    M9 도 같은 자리다 — 목업이 준 인라인 색·간격을 구현이 빠뜨려도 카피는 한 글자도 줄지 않아
#    `#home-empty` 의 색 이탈이 7연속 task 동안 새어 나갔다(2026-09-26). 넣기 전 실측: 목업에 없는
#    인라인 색을 구현에 주입해도 게이트가 **rc=0 · 11개 규칙 카운터 바이트 불변**으로 통과했다.
#  * **클래스 축의 이탈은 M9 가 보지 않는다.** M9 는 클래스 집합이 같은 쌍만 대조 단위로 보므로,
#    클래스가 어긋난 쌍은 위반이 아니라 **제외**로 샌다. 2026-09-26 실측으로 공유 키 21건 중 4건이
#    그렇게 빠졌고 그중 셋은 실재하는 이탈이다 — `no-access`(목업 `notice warn` ↔ 구현 `notice err`) ·
#    `request-error`(`notice warn` ↔ `notice err`) · `rejected-note`(`notice info` ↔ `notice`).
#    나머지 하나 `sift-cost` 는 키가 서로 다른 요소에 붙은 경우라 이탈이 아니다. 이 축을 규칙으로
#    바꾸려면 「같은 자리인가」를 클래스 말고 다른 것으로 정해야 한다 — 아직 그 단위가 없다.
#  * **상태 블록이 서는 조건**(구현이 대응 목업 단계에 없는 조건으로 `notice`·`badge` 를 렌더하는 것)은
#    세지 않는다. 문면이 JSX 식으로 오면 M3B 의 카피 집합에 애초에 들어오지 않아, 같은 자리를 한국어
#    리터럴로 바꾸면 M3B 가 미등재 1건으로 붉히는 표면이 식일 때는 영원히 초록이다(2026-09-25, AC4.1
#    접근 해제 통지가 그렇게 숨었다 — 원장 등재로 닫았다). M7·M8 처럼 규칙으로 바꾸려면 대조 단위가
#    있어야 하는데 아직 없다 — 세려면 먼저 「어느 조건의 렌더인가」를 선언하는 표기가 목업 쪽에
#    있어야 한다.
#  * 실행 스크린샷 픽셀 비교는 모델 정의상 범위 밖이다.
#  * 구현측 카피 추출(M3B)은 모듈 상수 테이블에 영문으로만 적힌 라벨(예: `STATUS_BADGE`
#    의 `Queued`)을 잡지 못한다 — 그런 라벨을 가진 화면은 「대조 보류」에 있어야 하고,
#    보류 상한이 그 사실을 붙잡아 둔다.
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


def table_after(doc: str, heading: str) -> list[list[str]]:
    """표 헤더를 키워드로 거르지 않는다 — 구분선(`|---|`) 기준이 유일하게 안전하다."""
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
    return token.startswith("STP-") or token.endswith((".tsx", ".html", ".md")) or "/" in token


def ticked(cell: str) -> list[str]:
    """셀 안의 백틱 토큰 — 기계 대조는 이것만 쓰고, 나머지 산문은 사람용이다."""
    return [norm(html.unescape(t)) for t in re.findall(r"`([^`]+)`", cell)]


SAMPLE_OPEN = re.compile(r"<(\w+)(?=[^>]*\bdata-sample\b)([^>]*)>")

INTERACTIVE_ATTR = re.compile(r"\b(data-goto|data-goto-journey|data-cta|id|href|onclick)\b")
INTERACTIVE_TAG = {"a", "button"}


def drop_samples(body: str) -> str:
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


VARIANT_OPEN = re.compile(r"<(\w+)(?=[^>]*\bdata-variant=)([^>]*)>")
VARIANT_KEY = re.compile(r'\bdata-variant="([^"]*)"')
VOID_TAGS = {"input", "img", "br", "hr", "meta", "source", "area", "col", "embed"}
VARIANT_FORBIDDEN = re.compile(r"\b(data-goto|data-goto-journey|data-cta)\b")


def split_variants(body: str) -> tuple[str, str]:
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
    """`data-step` 섹션별 가시 텍스트(+ placeholder·aria-label)."""
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
    parts = []
    for path in sorted(MOCKUP_DIR.glob("*.html")):
        src = re.sub(r"<!--.*?-->", "", path.read_text(encoding="utf-8"), flags=re.S)
        parts.append(norm(html.unescape(src)))
    for path in sorted(MOCKUP_DIR.glob("*.md")):
        parts.append(norm(path.read_text(encoding="utf-8")))
    return " \x01 ".join(parts)


UNIT_ONLY = re.compile(r"(?i)(min|mins|sec|secs|hr|hrs|mb|kb|gb|b)")


def is_copy(chunk: str) -> bool:
    if len(chunk) < 2:
        return False
    letters = re.sub(r"[^A-Za-z가-힣]", "", chunk)
    if len(letters) < 2:
        return False
    return not UNIT_ONLY.fullmatch(letters)


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
    """⚠️ TypeScript 제네릭(`useState<string | null>(null); const …`)도 `>`…`<` 사이에
    걸린다. JSX 텍스트에는 `;` 나 `=` 가 나타나지 않으므로 그것으로 코드를 가른다.
    """
    return [norm(t) for t in re.findall(r">([^<>{}]+)<", src) if not CODE_ISH.search(t)]


def is_attribute_value(src: str, start: int) -> bool:
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
    parts: list[str] = []
    for path in paths:
        src = strip_comments(path.read_text(encoding="utf-8"))
        parts.extend(jsx_text_nodes(src))
        parts.extend(norm(raw) for _, raw in literal_spans(src))
    return " \x01 ".join(p for p in parts if p)


MAPPING_REF = re.compile(r"docs/mockups/([A-Za-z0-9._-]+\.html)((?:#STP-[a-z0-9-]+)?)")


def discover_screens() -> dict[str, list[tuple[str, str]]]:
    screens: dict[str, list[tuple[str, str]]] = {}
    for path in sorted(SRC_DIR.glob("*.tsx")):
        header = path.read_text(encoding="utf-8")[:2000]
        refs = [(m.group(1), m.group(2).lstrip("#")) for m in MAPPING_REF.finditer(header)]
        if refs:
            screens[f"frontend/src/{path.name}"] = refs
    return screens


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


DETAILS_DISCLOSURE = re.compile(r'<details[^>]*className=\{?["\'`][^"\'`]*\bdisclosure\b')


def wide_breakpoint() -> int | None:
    path = SRC_DIR / "viewport.ts"
    if not path.exists():
        return None
    found = re.search(r"WIDE_QUERY\s*=\s*['\"`]\(min-width:\s*(\d+)px\)['\"`]",
                      path.read_text(encoding="utf-8"))
    return int(found.group(1)) if found else None


def css_declarations(text: str, width: int, prefix: str) -> dict[tuple[str, str], str]:
    """같은 명세도의 규칙만 다루므로 뒤에 오는 선언이 이긴다 — 그래서 `@media` 블록이
    그 블록이 잡으려는 규칙보다 **앞**에 서 있으면 조용히 무력해진다. 이 함수는 그
    순서를 그대로 따라가므로 선언 위치가 틀리면 같이 틀린 답을 낸다(= 그것을 잡는다)."""
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    out: dict[tuple[str, str], str] = {}

    def walk(block: str, active: bool) -> None:
        i = 0
        while True:
            open_at = block.find("{", i)
            if open_at < 0:
                return
            head = block[i:open_at].strip()
            depth, j = 1, open_at + 1
            while j < len(block) and depth:
                depth += (block[j] == "{") - (block[j] == "}")
                j += 1
            body = block[open_at + 1:j - 1]
            if head.startswith("@media"):
                low = re.search(r"min-width:\s*(\d+)px", head)
                high = re.search(r"max-width:\s*(\d+)px", head)
                applies = active and "prefers-" not in head
                if low and width < int(low.group(1)):
                    applies = False
                if high and width > int(high.group(1)):
                    applies = False
                walk(body, applies)
            elif not head.startswith("@") and active:
                for selector in (part.strip() for part in head.split(",")):
                    if not selector.startswith(prefix):
                        continue
                    for decl in body.split(";"):
                        if ":" not in decl:
                            continue
                        prop, value = decl.split(":", 1)
                        out[(selector, prop.strip())] = value.strip()
            i = j

    walk(text, True)
    return out


def root_tokens(text: str) -> dict[str, str]:
    block = re.search(r":root\s*\{(.*?)\}", text, flags=re.S)
    if not block:
        return {}
    return {
        m.group(1): norm(m.group(2))
        for m in re.finditer(r"(--[a-z0-9-]+)\s*:\s*([^;]+);", block.group(1))
    }



# ── M9 인라인 선언 대조 ──────────────────────────────────────────────────
# 목업 `id` ↔ 구현 `data-testid` 는 같은 자리를 가리키는 훅이다(프로토타입 훅 대 구현
# 훅의 관행 차이). 그 공유 키 중 **클래스 집합이 같은** 쌍만 같은 대조 단위로 보고,
# 인라인 style 의 선언 집합을 대조한다. 클래스가 다른 쌍은 애초에 같은 요소가 아니거나
# (`sift-cost` — 목업은 안쪽 `span.metric`, 구현은 바깥 `div.card row between`) 클래스
# 축의 이탈이라(`no-access` — 목업 `notice warn` ↔ 구현 `notice err`) 이 규칙이 아니라
# **클래스 축**이 답해야 한다. 그 축은 아직 규칙이 없고, 위 「보지 않는 것」에 적는다.
PROTO_CLASSES = {"on"}      # 목업 전용 가시성 토글(`.stp.on`·`.notice.on` = `display:block`).
                            # 구현은 조건부 렌더라 `.on` 규칙 자체가 없다 — 무력한 클래스다.
PROTO_PROPS = {"display"}   # 목업이 숨긴 변이를 정적 HTML 에 열거하는 장치(`display:none`).


def _kebab(prop: str) -> str:
    return re.sub(r"([A-Z])", lambda m: "-" + m.group(1).lower(), prop)


def open_tag(src: str, at: int) -> str:
    """`at` 을 품은 여는 태그를 통째로 돌려준다. JSX 는 중괄호 식 안에 `>` 가 들어갈 수
    있으므로(화살표 함수) 깊이를 세어 닫는 `>` 를 찾는다 — 정규식 하나로는 못 자른다."""
    start = src.rfind("<", 0, at)
    depth = 0
    for i in range(start, len(src)):
        ch = src[i]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
        elif ch == ">" and depth == 0:
            return src[start:i + 1]
    return src[start:at]


def tag_classes(tag: str) -> list[str]:
    found = CLASS_ATTR.search(tag)
    return [c for c in (found.group(1).split() if found else []) if c not in PROTO_CLASSES]


def mockup_inline(tag: str) -> dict[str, str]:
    found = re.search(r'style="([^"]*)"', tag)
    decls: dict[str, str] = {}
    for decl in (found.group(1).split(";") if found else []):
        if ":" not in decl:
            continue
        prop, value = decl.split(":", 1)
        prop = prop.strip().lower()
        if prop not in PROTO_PROPS:
            decls[prop] = norm(value)
    return decls


def impl_inline(tag: str) -> tuple[dict[str, str], set[str]]:
    """구현의 `style={{...}}` 를 목업 표기로 환산한다. 값이 식이면 목업의 정적 값과
    비교할 수 없으므로 그 속성을 대조에서 빼고(`dynamic`) 건수를 돌려준다 — 예시값을
    대조에서 빼는 `data-sample` 규약과 같은 이유다."""
    found = re.search(r"style=\{\{(.*?)\}\}", tag, flags=re.S)
    decls: dict[str, str] = {}
    dynamic: set[str] = set()
    for decl in re.split(r",(?![^(]*\))", found.group(1) if found else ""):
        if ":" not in decl:
            continue
        prop, value = decl.split(":", 1)
        prop = _kebab(prop.strip().strip("'\""))
        value = value.strip()
        if prop in PROTO_PROPS:
            continue
        if re.fullmatch(r"\d+", value):
            decls[prop] = f"{value}px"
        elif re.fullmatch(r"'[^']*'|\"[^\"]*\"", value):
            decls[prop] = norm(value[1:-1])
        else:
            dynamic.add(prop)
    return decls, dynamic


def keyed_elements(paths: list[Path], attr: str) -> dict[str, tuple[str, int, str]]:
    """`attr="<키>"` 를 가진 요소를 키 → (파일명, 줄, 여는 태그) 로 모은다."""
    found: dict[str, tuple[str, int, str]] = {}
    for path in paths:
        src = path.read_text(encoding="utf-8")
        for m in re.finditer(rf'{attr}="([A-Za-z0-9_-]+)"', src):
            key = m.group(1)
            if key not in found:
                found[key] = (path.name, src[:m.start()].count("\n") + 1,
                              open_tag(src, m.start()))
    return found

def main() -> int:
    doc = TRACKER.read_text(encoding="utf-8")

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

    wide = wide_breakpoint()
    folds = sum(len(DETAILS_DISCLOSURE.findall(path.read_text(encoding="utf-8")))
                for path in sorted(SRC_DIR.glob("*.tsx")))
    css_text = INDEX_CSS.read_text(encoding="utf-8")
    disclosure_rules = sum(1 for (selector, _) in css_declarations(css_text, 0, ".disclosure"))
    if wide is None:
        fail("M8", "`viewport.ts` 의 `WIDE_QUERY` 에서 확장 브레이크포인트를 읽지 못했다")
    elif folds == 0 or disclosure_rules == 0:
        fail("M8", f"접힘 기구를 찾지 못했다(`<details …disclosure>` {folds}건 · "
                   f"`.disclosure` 규칙 {disclosure_rules}건) — 규칙이 공전한다")
    else:
        if not re.search(rf"@media \(min-width:\s*{wide}px\)", css_text):
            fail("M8", f"`index.css` 에 `@media (min-width: {wide}px)` 블록이 없다 — "
                       f"접힘과 레이아웃이 「compact」를 다르게 본다")
        wide_decls = css_declarations(css_text, wide, ".disclosure")
        live = []
        if wide_decls.get((".disclosure > summary", "cursor")) == "pointer":
            live.append("`cursor: pointer`")
        for selector in (".disclosure > summary::after", ".disclosure[open] > summary::after"):
            content = wide_decls.get((selector, "content"))
            if content is not None and content != "none":
                live.append(f"`{selector} {{ content: {content} }}`")
        for item in live:
            fail("M8", f"{wide}px 에서 접힘 affordance 가 남아 있다 — {item}. "
                       f"그 폭의 목업은 정적인 `div.section-title` 다")
        print(f"M8 접힘 affordance 폭 — 접힘 {folds}건 · `.disclosure` 선언 {disclosure_rules}개 · "
              f"{wide}px 잔존 affordance {len(live)}건")

    mockup_keyed = keyed_elements(sorted(MOCKUP_DIR.glob("*.html")), "id")
    impl_keyed = keyed_elements(sorted(SRC_DIR.glob("*.tsx")), "data-testid")
    shared = sorted(set(mockup_keyed) & set(impl_keyed))
    matched, unmatched, skipped = [], [], 0
    for key in shared:
        m_file, m_line, m_tag = mockup_keyed[key]
        i_file, i_line, i_tag = impl_keyed[key]
        if tag_classes(m_tag) != tag_classes(i_tag):
            unmatched.append(key)
            continue
        matched.append(key)
        want = mockup_inline(m_tag)
        got, dynamic = impl_inline(i_tag)
        skipped += len(dynamic)
        want = {p: v for p, v in want.items() if p not in dynamic}
        for prop in sorted(set(want) | set(got)):
            if want.get(prop) == got.get(prop):
                continue
            fail("M9", f"`#{key}` 인라인 선언이 목업과 다르다 — `{prop}`: "
                       f"목업 `{want.get(prop, '없음')}`({m_file}:{m_line}) ↔ "
                       f"구현 `{got.get(prop, '없음')}`({i_file}:{i_line})")
    if not matched:
        fail("M9", f"대조 단위가 하나도 없다(공유 키 {len(shared)}건 · 클래스 일치 0건) "
                   f"— 규칙이 공전한다")
    else:
        print(f"M9 인라인 선언 — 공유 키 {len(shared)}건 · 대조 {len(matched)}쌍 · "
              f"클래스 불일치로 제외 {len(unmatched)}건 · 식이라 제외한 속성 {skipped}건")


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
