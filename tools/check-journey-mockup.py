#!/usr/bin/env python3
"""사용자 여정 ↔ 목업 1:1 정합성 체커.

규약은 docs/mockups/README.md 「여정 페이지 규약」. 이 스크립트는 그 규약이
실제 파일 상태와 일치하는지를 기계적으로 확인한다 — 의존성 0, 파이썬 stdlib 만.

SSOT 방향:
  여정 문서(docs/user-journey/JRN-*.md)  →  목업 페이지(docs/mockups/JRN-*.html)
                                        →  매핑 인덱스(docs/mockups/README.md)
                                        →  허브(docs/index.html) · 추적(docs/doc-tracker.md)
문서가 원천이고 나머지는 반영물이다. 어긋나면 반영물을 고친다.

규칙
  R0 마커 무결성 — README 의 기계 판독 구간(jmap/steps/ledger)이 열리고 닫힌다
  R1 여정 → 목업 페이지 — README 가 선언한 여정별 상태가 실제 페이지 수와 일치
  R2 목업 페이지 → 여정 — 모든 docs/mockups/*.html 이 data-journey 를 정확히 1개
                          선언하거나, 이관 대기 원장에 등재되어 있다
  R3 단계 집합 일치 — 여정 페이지의 data-step 집합 == 여정 문서의 STP 헤딩 집합
  R4 분기 대응 — 여정 문서 §4 분기표의 모든 "이어지는 단계"가 페이지에서 이동 가능
                 (다른 여정의 단계로 이어지는 갈래는 handoff 선언으로 해소한다)
  R5 앵커 무결성 — 페이지 안의 이동 대상이 전부 실재 섹션 id · 여정 밖 분기 규약
  R6 화면 대조 — 단계 섹션이 선언한 data-screens 집합 == 여정 문서의 그 단계
                 터치포인트 줄에서 파싱한 S01~S10 집합
  R7 원장 래칫 — 원장 밖 위반 · 이미 해소됐는데 남은 공전 행 · 상한 초과 ·
                 「쓰는 여정」 칸이 여정 문서에서 파생한 실측과 일치
  R8 링크 무결성 — docs/ 안의 상대 링크가 전부 실재 파일
  R9 집계 일치 — README·허브·doc-tracker 가 선언한 숫자가 실측과 같다
  R10 보조 레이어 — 문서 메타가 기본 닫힌 <details> 안에만 있다 (규칙 5b)

규칙 5 의 (c)(d)(e) 는 정적 대조로 확인할 수 없다 — 파일을 읽어 속성만 세면
배선이 끊긴 버튼과 살아 있는 버튼이 구분되지 않는다. 그쪽은 DOM 하네스
tools/check-journey-prototype.js 가 같은 CI 게이트에서 집행한다.

화면 본문을 원본과 바이트 동일하게 고정하는 검사(구 R6 의 data-sha256)는
쓰지 않는다. 지문으로 화면을 못 박으면 실제 입력 요소와 상태 변형을 넣을 길이
구조적으로 막힌다 — 여정 페이지의 화면은 복제해 온 스냅샷이 아니라 그 자체가 원본이다.
"""
from __future__ import annotations

import html as html_mod
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
UJ = DOCS / "user-journey"
MK = DOCS / "mockups"

failures = []
notes = []


def fail(rule, msg):
    failures.append("%s: %s" % (rule, msg))


def read(p):
    return p.read_text(encoding="utf-8")


def marked(text, name, rule="R0"):
    """<!-- name:begin --> ... <!-- name:end --> 사이를 돌려준다.

    산문이 파서의 입력이 되지 않도록 경계를 마커로만 잡는다.
    """
    m = re.search(r"<!-- %s:begin -->\n(.*?)<!-- %s:end -->" % (re.escape(name), re.escape(name)), text, re.S)
    if not m:
        fail(rule, "docs/mockups/README.md 에 `%s` 마커 구간이 없거나 닫히지 않았다" % name)
        return None
    return m.group(1)


def norm_text(x):
    """마크다운 셀과 HTML 조각을 비교 가능한 평문으로 맞춘다."""
    x = re.sub(r"<[^>]+>", "", x)
    x = html_mod.unescape(x)
    x = x.replace("`", "").replace("**", "").replace("*", "")
    return re.sub(r"\s+", " ", x).strip()


def table_rows(block):
    rows = []
    for line in block.splitlines():
        line = line.strip()
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if all(set(c) <= set("-: ") for c in cells):
            continue
        rows.append(cells)
    return rows[1:] if rows else []  # 헤더 제외


# ── 1. 여정 문서 파싱 (SSOT) ──────────────────────────────────────
def parse_journeys():
    out = {}
    for p in sorted(UJ.glob("JRN-*.md")):
        doc = read(p)
        marks = list(re.finditer(r"^### `(STP-[a-z0-9-]+)` (.+?)$", doc, re.M))
        if not marks:
            fail("R3", "%s 에 `### `STP-…`` 단계 헤딩이 없다" % p.relative_to(ROOT))
            continue
        cut = doc.index("\n## 4.") if "\n## 4." in doc else len(doc)
        steps = []
        for i, m in enumerate(marks):
            end = marks[i + 1].start() if i + 1 < len(marks) else cut
            block = doc[m.end():end]
            t = re.search(r"^- \*\*터치포인트\*\*: (.+)$", block, re.M)
            screens = sorted(set(re.findall(r"\bS(?:0[1-9]|10)\b", t.group(1)))) if t else []
            steps.append({"id": m.group(1), "title": m.group(2).strip(), "screens": screens})
        branch_to = []
        if "\n## 4." in doc and "\n## 5." in doc:
            blk = doc[doc.index("\n## 4."):doc.index("\n## 5.")]
            for cells in table_rows(blk):
                if len(cells) == 3:
                    g = re.search(r"`(STP-[a-z0-9-]+)`", cells[2])
                    if g:
                        branch_to.append((cells[0], g.group(1)))
        title = re.search(r"^# (.+)$", doc, re.M).group(1).strip()
        out[p.stem] = {
            "path": p,
            "title": title.split(":", 1)[1].strip() if ":" in title else title,
            "steps": steps,
            "step_ids": [s["id"] for s in steps],
            "branch_to": branch_to,
        }
    if not out:
        fail("R3", "docs/user-journey/JRN-*.md 를 하나도 찾지 못했다")
    return out


# ── 2. 목업 페이지 파싱 ───────────────────────────────────────────
def parse_pages():
    out = {}
    for p in sorted(MK.glob("*.html")):
        s = read(p)
        # <body> 태그를 떼어 낸 다음 그 안에서 센다. 한 정규식으로 findall 하면
        # 같은 태그의 두 번째 data-journey 를 놓쳐(탐욕적 [^>]*) "정확히 1개"
        # 규칙이 공허해진다 — 네거티브 컨트롤이 잡아낸 결함이다.
        # 주석은 마크업이 아니므로 먼저 걷어낸다(문서 주석이 <body> 를 언급한다).
        markup = re.sub(r"<!--.*?-->", "", s, flags=re.S)
        bodytag = re.search(r"<body\b[^>]*>", markup)
        journeys = re.findall(r'data-journey="([^"]+)"', bodytag.group(0)) if bodytag else []
        if bodytag is None:
            fail("R2", "`%s` 에 <body> 태그가 없다" % p.name)
        sections = re.findall(r'<section [^>]*\bid="([^"]+)"[^>]*>', s)
        # 여정 밖 분기(handoff) 선언과 그 섹션 본문. 갈래가 다른 여정의 단계로
        # 이어질 때, 그 끝 블록이 대상을 선언한다 — 선언은 한 곳에만 둔다.
        handoffs = {}
        section_html = {}
        for sm in re.finditer(r'<section\b([^>]*)>(.*?)</section>', s, re.S):
            attrs, inner = sm.group(1), sm.group(2)
            sid = re.search(r'\bid="([^"]+)"', attrs)
            if not sid:
                continue
            section_html[sid.group(1)] = inner
            hj = re.search(r'\bdata-goto-journey="([^"]*)"', attrs)
            if hj:
                handoffs[sid.group(1)] = hj.group(1)
        steps = re.findall(r'\bdata-step="([^"]+)"', s)
        gotos = set(re.findall(r'\bdata-goto(?:-next|-prev)?="([^"]+)"', s))
        # 단계 섹션이 선언한 화면. 원본을 복제하지 않으므로 지문 대신 이 선언을
        # 여정 문서의 터치포인트와 대조한다(R6).
        step_screens = {}
        for m in re.finditer(r'<section [^>]*\bdata-step="([^"]+)"[^>]*\bdata-screens="([^"]*)"', s):
            step_screens[m.group(1)] = sorted(set(re.findall(r"\bS(?:0[1-9]|10)\b", m.group(2))))
        out[p.name] = {"path": p, "text": s, "journeys": journeys, "sections": sections,
                       "steps": steps, "gotos": gotos, "step_screens": step_screens,
                       "handoffs": handoffs, "section_html": section_html}
    return out


JOURNEYS = parse_journeys()
PAGES = parse_pages()

# 어떤 단계가 어느 여정의 것인가 — 여정 문서에서 파생한다. §4 의 「이어지는 단계」가
# 이 인덱스에서 다른 여정으로 나오면 그 갈래는 여정 밖 분기(handoff)다.
STEP_OWNER = {}
for _jid, _jr in JOURNEYS.items():
    for _sid in _jr["step_ids"]:
        STEP_OWNER.setdefault(_sid, _jid)
README = read(MK / "README.md")
HUB = read(DOCS / "index.html")
TRACKER = read(DOCS / "doc-tracker.md")

# ── R0 · README 기계 판독 구간 ────────────────────────────────────
jmap_block = marked(README, "jmap")
ledger_block = marked(README, "ledger")

# ── 예외 등재 (규칙 8) — SSOT 는 doc-tracker 「수용된 위험」 ──────
accepted = TRACKER[TRACKER.index("## 수용된 위험"):TRACKER.index("## 변경 이력")] if "## 수용된 위험" in TRACKER else ""
exempt_journeys = set()
for cells in table_rows(accepted):
    if len(cells) >= 4:
        g = re.match(r"`(JRN-[a-z0-9-]+)` 전체", cells[1])
        if g:
            exempt_journeys.add(g.group(1))

# ── R1 · 여정 → 목업 페이지 ──────────────────────────────────────
declared = {}
declared_page = {}
if jmap_block:
    for cells in table_rows(jmap_block):
        if len(cells) != 5:
            fail("R1", "Journeys 표의 행 형식이 5칸이 아니다: %s" % cells)
            continue
        g = re.match(r"`(JRN-[a-z0-9-]+)`", cells[0])
        if not g:
            fail("R1", "Journeys 표 첫 칸에 여정 식별자가 없다: %s" % cells[0])
            continue
        jid = g.group(1)
        declared[jid] = cells[4]
        pg = re.search(r"\((?:\./)?(JRN-[a-z0-9-]+\.html)\)", cells[2])
        if pg:
            declared_page[jid] = pg.group(1)

    if set(declared) != set(JOURNEYS):
        fail("R1", "Journeys 표의 여정 집합이 여정 문서와 다르다: 표에만 %s / 문서에만 %s"
             % (sorted(set(declared) - set(JOURNEYS)), sorted(set(JOURNEYS) - set(declared))))

owners = {}
for name, pg in PAGES.items():
    for j in pg["journeys"]:
        owners.setdefault(j, []).append(name)

for jid in sorted(JOURNEYS):
    got = owners.get(jid, [])
    state = declared.get(jid, "")
    if "이관 완료" in state:
        if len(got) != 1:
            fail("R1", "`%s` 는 README 가 이관 완료라고 선언했는데 자신을 선언한 페이지가 %d개다 (%s)" % (jid, len(got), got))
        elif declared_page.get(jid) != got[0]:
            fail("R1", "`%s` 의 README 목업 페이지 칸(%s)과 실제 선언 파일(%s)이 다르다"
                 % (jid, declared_page.get(jid), got[0]))
    elif "예외" in state:
        if jid not in exempt_journeys:
            fail("R1", "`%s` 를 README 가 예외로 선언했으나 doc-tracker 「수용된 위험」에 여정 단위 등재가 없다" % jid)
        if got:
            fail("R7", "`%s` 는 예외 등재인데 목업 페이지가 생겼다(%s) — README 를 갱신할 것(공전)" % (jid, got))
    elif "이관 대기" in state:
        if got:
            fail("R7", "`%s` 는 README 가 이관 대기라고 선언했는데 이미 페이지가 있다(%s) — 공전 행" % (jid, got))
        if jid in exempt_journeys:
            fail("R1", "`%s` 가 예외 등재이면서 이관 대기로도 선언돼 있다" % jid)
    else:
        fail("R1", "`%s` 의 상태 칸을 해석할 수 없다: %r" % (jid, state))

for jid in sorted(owners):
    if jid not in JOURNEYS:
        fail("R1", "목업 페이지가 존재하지 않는 여정 `%s` 를 선언한다 (%s)" % (jid, owners[jid]))
    elif len(owners[jid]) > 1:
        fail("R1", "`%s` 를 선언한 목업 페이지가 %d개다 (%s) — 여정당 1개여야 한다" % (jid, len(owners[jid]), owners[jid]))

# ── R7 · 이관 대기 원장 (화면 단위 잔여) ─────────────────────────
ledger_files = []
ledger_rows = []
if ledger_block:
    for cells in table_rows(ledger_block):
        if len(cells) != 4:
            fail("R7", "원장 행 형식이 4칸이 아니다: %s" % cells)
            continue
        g = re.search(r"\((?:\./)?([a-z0-9._-]+\.html)\)", cells[0])
        if not g:
            fail("R7", "원장 행에서 파일명을 읽을 수 없다: %s" % cells[0])
            continue
        ledger_files.append(g.group(1))
        ledger_rows.append((g.group(1), cells[1], cells[2]))

cap = re.search(r"\*\*상한: (\d+)\*\*", README)
cap_n = int(cap.group(1)) if cap else None
if cap_n is None:
    fail("R7", "README 에 화면 단위 잔여 상한(`**상한: N**`) 선언이 없다")

wait_cap = re.search(r"\*\*이관 대기 상한: (\d+)\*\*", README)
wait_cap_n = int(wait_cap.group(1)) if wait_cap else None
if wait_cap_n is None:
    fail("R7", "README 에 이관 대기 상한(`**이관 대기 상한: N**`) 선언이 없다")

screen_files = sorted(n for n in PAGES if re.fullmatch(r"s\d\d-.+\.html", n))
journey_files = sorted(n for n in PAGES if n.startswith("JRN-"))

for f in ledger_files:
    if f not in PAGES:
        fail("R7", "원장에 있는 `%s` 가 실제로 없다 — 이관·삭제됐다면 원장 행을 지울 것(공전)" % f)
for f in screen_files:
    if f not in ledger_files:
        fail("R7", "화면 단위 파일 `%s` 가 이관 대기 원장에 없다 — 등재하거나 이관할 것" % f)
if cap_n is not None and len(screen_files) > cap_n:
    fail("R7", "화면 단위 잔여 %d개가 상한 %d을 넘는다" % (len(screen_files), cap_n))

# ── R7 · 원장 「쓰는 여정」 칸은 파생이다 ────────────────────────
# 손으로 적은 목록은 이관이 진행되면 조용히 낡는다. 그 칸의 원천은 여정 문서의
# 터치포인트이며, 남은 소비자 = {그 화면을 쓰는 여정} − {이미 이관된 여정}
# − {규칙 8 예외 여정}. 예외 여정은 페이지를 갖지 않기로 등재된 것이라 원본
# 삭제를 영원히 막지 않는다. 소비자가 0이 되면 그 파일은 흡수·삭제할 차례다.
def screen_users(screen):
    out = set()
    for j, jr in JOURNEYS.items():
        if any(screen in st["screens"] for st in jr["steps"]):
            out.add(j)
    return out


for fname, screen_cell, users_cell in ledger_rows:
    sg = re.search(r"\bS(?:0[1-9]|10)\b", screen_cell)
    if not sg:
        fail("R7", "원장 행 `%s` 의 화면 칸에서 화면 ID 를 읽을 수 없다: %r" % (fname, screen_cell))
        continue
    screen = sg.group(0)
    want_users = screen_users(screen) - set(owners) - exempt_journeys
    got_users = set(re.findall(r"JRN-[a-z0-9-]+", users_cell))
    if not want_users:
        fail("R7", "`%s`(%s) 를 쓰는 여정이 전부 이관됐다 — 흡수·삭제하고 원장 행을 지울 것"
             % (fname, screen))
    elif got_users != want_users:
        fail("R7", "원장 `%s`(%s) 의 「쓰는 여정」 %s 이 여정 문서에서 파생한 실측 %s 과 다르다"
             % (fname, screen, sorted(got_users), sorted(want_users)))

waiting = [j for j, st in declared.items() if "이관 대기" in st]
if wait_cap_n is not None and len(waiting) > wait_cap_n:
    fail("R7", "이관 대기 여정 %d개가 상한 %d을 넘는다" % (len(waiting), wait_cap_n))

# ── R2 · 목업 페이지 → 여정 ──────────────────────────────────────
for name, pg in sorted(PAGES.items()):
    n = len(pg["journeys"])
    if n == 1:
        continue
    if n == 0:
        if name not in ledger_files:
            fail("R2", "`%s` 이 data-journey 를 선언하지 않고 원장에도 없다 (고아 목업)" % name)
    else:
        fail("R2", "`%s` 이 data-journey 를 %d개 선언한다 — 정확히 1개여야 한다 (%s)" % (name, n, pg["journeys"]))

# ── R3~R6 · 여정 페이지 내부 ─────────────────────────────────────
for name in journey_files:
    pg = PAGES[name]
    if len(pg["journeys"]) != 1:
        continue  # R2 가 이미 보고
    jid = pg["journeys"][0]
    jr = JOURNEYS.get(jid)
    if jr is None:
        continue  # R1 이 이미 보고

    # R3 단계 집합 양방향 일치
    page_steps = pg["steps"]
    if len(page_steps) != len(set(page_steps)):
        fail("R3", "`%s` 에 중복된 data-step 이 있다: %s" % (name, page_steps))
    only_page = sorted(set(page_steps) - set(jr["step_ids"]))
    only_doc = sorted(set(jr["step_ids"]) - set(page_steps))
    if only_page:
        fail("R3", "`%s` 에만 있는 단계 (문서 미반영): %s" % (name, only_page))
    if only_doc:
        fail("R3", "여정 문서에만 있는 단계 (미시각화): %s" % only_doc)

    # 단계 섹션의 id 와 data-step 이 같은 값이어야 딥링크가 성립한다 (규칙 5e)
    for m in re.finditer(r'<section [^>]*\bid="([^"]+)"[^>]*\bdata-step="([^"]+)"', pg["text"]):
        if m.group(1) != m.group(2):
            fail("R3", "`%s` 의 섹션 id(%s)와 data-step(%s)이 다르다" % (name, m.group(1), m.group(2)))

    # R4 분기 대응 — 멤버십만 보면 공허하다(버튼의 대상을 바꿔도 자기 자신과는
    # 늘 일치한다). 여정 문서 §4 의 각 행이 "그 상황 → 그 단계" 로 짝지어져
    # 페이지에 있는지를 순서까지 대조한다.
    page_branches = []
    for m in re.finditer(r'<li><span class="sit">(.*?)</span>.*?<button [^>]*data-goto="([^"]+)"', pg["text"], re.S):
        goto = m.group(2)
        # 여정 밖 분기는 이 여정의 끝(END-*)으로 가고, 그 끝 블록이 대상 여정·단계를
        # 선언한다. 실효 대상은 그 선언에서 읽는다 — 페이지가 다른 여정의 단계 id 를
        # 직접 data-goto 로 쓰면 갈 곳이 없어 R5 가 잡는다.
        decl = pg["handoffs"].get(goto)
        if decl and "#" in decl:
            goto = decl.split("#", 1)[1]
        page_branches.append((norm_text(m.group(1)), goto))
    doc_branches = [(norm_text(sit), to) for sit, to in jr["branch_to"]]
    if len(page_branches) != len(doc_branches):
        fail("R4", "`%s` 의 분기 항목 %d개가 여정 문서 §4 의 %d행과 다르다"
             % (name, len(page_branches), len(doc_branches)))
    for i, (want, got) in enumerate(zip(doc_branches, page_branches)):
        if want != got:
            fail("R4", "`%s` 의 분기 %d번이 여정 문서 §4 와 다르다: 문서 %s / 페이지 %s" % (name, i + 1, want, got))

    # R5 앵커 무결성
    ids = set(pg["sections"])
    dangling = sorted(t for t in pg["gotos"] if t not in ids)
    if dangling:
        fail("R5", "`%s` 의 이동 대상이 실재하지 않는다: %s" % (name, dangling))
    for href in sorted(set(re.findall(r'href="#([^"]+)"', pg["text"]))):
        if href not in ids:
            fail("R5", "`%s` 의 내부 앵커 #%s 가 실재하지 않는다" % (name, href))
    if not re.search(r'id="END-[a-z0-9-]+"', pg["text"]):
        fail("R5", "`%s` 에 갈래의 끝(END-*) 블록이 없다 (규칙 5f)" % name)

    # R5 여정 밖 분기(handoff) — §4 의 「이어지는 단계」가 다른 여정의 것이면 그
    # 갈래는 이 여정의 끝이며, 끝 블록이 대상을 선언하고 실제로 그리로 링크한다.
    # 선언 대상의 실재 여부는 페이지가 아니라 여정 문서에서 확인한다(자기참조 금지).
    for sec_id in sorted(pg["handoffs"]):
        decl = pg["handoffs"][sec_id]
        g = re.fullmatch(r"(JRN-[a-z0-9-]+)#(STP-[a-z0-9-]+)", decl)
        if not g:
            fail("R5", "`%s` 의 `%s` 가 선언한 data-goto-journey %r 이 "
                       "`JRN-<여정>#STP-<단계>` 형식이 아니다" % (name, sec_id, decl))
            continue
        tj, tstep = g.groups()
        if not sec_id.startswith("END-"):
            fail("R5", "`%s` 의 여정 밖 분기 선언이 `%s` 에 있다 — 갈래의 끝(END-*)이어야 한다"
                 % (name, sec_id))
        if tj == jid:
            fail("R5", "`%s` 의 `%s` 가 자기 여정을 여정 밖 분기로 선언한다" % (name, sec_id))
            continue
        if tj not in JOURNEYS:
            fail("R5", "`%s` 의 `%s` 가 존재하지 않는 여정 `%s` 로 넘긴다" % (name, sec_id, tj))
            continue
        if tstep not in JOURNEYS[tj]["step_ids"]:
            fail("R5", "`%s` 의 `%s` 가 `%s` 에 없는 단계 `%s` 로 넘긴다 "
                       "(그 여정 문서의 단계: %s)" % (name, sec_id, tj, tstep, JOURNEYS[tj]["step_ids"]))
            continue
        # 넘기는 자리는 실제로 눌러 갈 수 있어야 한다. 대상 여정에 페이지가 있으면
        # 그 단계 앵커로, 아직 없으면 여정 문서를 reader 로 연다(.md 직결은 R8 금지).
        target_pages = owners.get(tj, [])
        want = ("./%s#%s" % (target_pages[0], tstep) if target_pages
                else "../reader.html?doc=user-journey/%s.md" % tj)
        if ('href="%s"' % want) not in pg["section_html"].get(sec_id, ""):
            fail("R5", "`%s` 의 `%s` 에 대상으로 나가는 링크 `%s` 가 없다 "
                       "(대상 여정에 페이지가 %s)" % (name, sec_id, want,
                                                    "있다" if target_pages else "아직 없다"))

    # R6 화면 대조 — 단계 섹션이 선언한 data-screens 가 여정 문서의 터치포인트와 같은가.
    # 기대값의 원천은 페이지가 아니라 여정 문서다(자기참조면 무엇을 바꿔도 통과한다).
    for st in jr["steps"]:
        if st["id"] not in pg["step_screens"]:
            fail("R6", "`%s` 의 `%s` 섹션에 data-screens 선언이 없다 "
                       "(터치포인트가 없는 단계도 data-screens=\"\" 로 명시할 것)" % (name, st["id"]))
            continue
        got = pg["step_screens"][st["id"]]
        want = sorted(st["screens"])
        if got != want:
            fail("R6", "`%s` 의 `%s` 선언 화면 %s 이 여정 문서 터치포인트 %s 와 다르다"
                 % (name, st["id"], got, want))

    # 금지된 기법이 되살아나지 않았는지 — 화면을 원본과 바이트 동일하게 못 박는 방식은
    # 실제 입력 요소·상태 변형을 넣을 길을 구조적으로 막는다(모델 규칙 5).
    if re.search(r"\bdata-sha256=", pg["text"]) or "<!--embed:" in pg["text"]:
        fail("R6", "`%s` 에 바이트 동일 임베드 흔적(data-sha256 / <!--embed:)이 있다 — "
                   "여정 페이지의 화면은 복제본이 아니라 그 자체가 원본이어야 한다" % name)

    # R10 보조 레이어 — 문서 메타(단계 번호·식별자·터치포인트·연결 AC)는 제품 화면과
    # 같은 평면에 상시 노출되지 않아야 한다(규칙 5b). 기본으로 접힌 <details> 안에만 둔다.
    layers = re.findall(r'<details\b[^>]*\bdata-meta="doc"[^>]*>', pg["text"])
    if len(layers) != 1:
        fail("R10", "`%s` 에 문서 메타 보조 레이어 <details data-meta=\"doc\"> 가 %d개다 — 1개여야 한다"
             % (name, len(layers)))
    else:
        if re.search(r"\bopen\b", layers[0]):
            fail("R10", "`%s` 의 보조 레이어가 open 으로 시작한다 — 연 직후 보이는 것은 제품이어야 한다" % name)
        # <head>(title·meta)는 제품 화면 평면이 아니다 — <body> 안만 본다.
        bm = re.search(r"<body\b[^>]*>(.*)</body>", pg["text"], re.S)
        rest = bm.group(1) if bm else pg["text"]
        rest = re.sub(r"<details\b[^>]*\bdata-meta=\"doc\"[^>]*>.*?</details>", "", rest, flags=re.S)
        rest = re.sub(r"<(script|style)\b.*?</\1>", "", rest, flags=re.S | re.I)
        rest = re.sub(r"<!--.*?-->", "", rest, flags=re.S)
        visible_text = re.sub(r"<[^>]+>", " ", rest)   # 속성값(id/data-step)은 텍스트가 아니다
        leaked = sorted({t for t in re.findall(r"STP-[a-z0-9-]+|JRN-[a-z0-9-]+|AC\d+\.\d+|터치포인트|연결 AC",
                                               visible_text)})
        if leaked:
            fail("R10", "`%s` 의 제품 화면 평면에 문서 메타가 노출돼 있다: %s (규칙 5b — 보조 레이어로 옮길 것)"
                 % (name, leaked))

# ── R8 · docs/ 상대 링크 무결성 ──────────────────────────────────
def strip_code(md):
    md = re.sub(r"```.*?```", "", md, flags=re.S)
    return re.sub(r"`[^`]*`", "", md)


link_count = 0
for p in sorted(DOCS.rglob("*")):
    if not p.is_file() or p.suffix not in (".md", ".html"):
        continue
    text = read(p)
    if p.suffix == ".md":
        targets = re.findall(r"\]\(([^)\s]+)\)", strip_code(text))
    else:
        # 마크업만 본다 — <script>/<style> 안의 문자열(예: 마크다운 렌더러의
        # '<a href="$2">' 치환 템플릿)은 링크가 아니다.
        markup = re.sub(r"<(script|style)\b.*?</\1>", "", text, flags=re.S | re.I)
        targets = re.findall(r'(?:href|src)="([^"]+)"', markup)
    for t in targets:
        if re.match(r"^[a-z]+:", t) or t.startswith("//") or t.startswith("#"):
            continue
        link_count += 1
        path, _, frag = t.partition("#")
        query = ""
        if "?" in path:
            path, _, query = path.partition("?")
        if path:
            target = (p.parent / path).resolve()
            if not target.exists():
                fail("R8", "%s → `%s` 가 실재하지 않는다" % (p.relative_to(ROOT), t))
                continue
        q = re.match(r"doc=([^&]+)$", query)
        if q and not (DOCS / q.group(1)).exists():
            fail("R8", "%s → reader 문서 `%s` 가 실재하지 않는다" % (p.relative_to(ROOT), q.group(1)))
            continue
        # docs/ 에 .nojekyll 이 있어 Pages 는 .md 를 렌더링하지 않는다 — HTML 에서
        # .md 를 직접 걸면 클릭 시 파일이 내려받아진다. 레포 규약대로 reader 를 경유해야
        # 한다(2026-08-27 문서 포털). 여정 페이지가 여정 문서의 링크를 옮겨 실을 때
        # 실제로 밟은 함정이다.
        if p.suffix == ".html" and path.endswith(".md") and not query:
            fail("R8", "%s → `%s` 를 직접 건다 — HTML 에서 .md 는 reader.html?doc= 를 경유할 것"
                 % (p.relative_to(ROOT), t))
        # 다른 HTML 파일의 앵커로 들어가는 링크는 그 앵커가 실재하는지까지 본다.
        # R5 는 페이지 *안*의 앵커만 보므로, 허브가 여정 페이지의 단계로 거는 링크
        # (index.html → mockups/JRN-*.html#STP-*)는 아무도 검사하지 않는 사각이었다.
        # reader 경유(?doc=)는 프래그먼트가 마크다운 헤딩이라 대상이 다르므로 제외한다.
        if frag and not query and path and target.suffix == ".html":
            if 'id="%s"' % frag not in read(target):
                fail("R8", "%s → `%s` 의 앵커 #%s 가 대상 파일에 없다"
                     % (p.relative_to(ROOT), path, frag))

notes.append("docs/ 상대 링크 %d건 확인" % link_count)

# ── R9 · 집계 일치 ───────────────────────────────────────────────
n_j = len(JOURNEYS)
n_ex = len(exempt_journeys & set(JOURNEYS))
n_target = n_j - n_ex
n_done = len(journey_files)
n_wait = n_target - n_done

agg = re.search(r"집계: 여정 \*\*(\d+)\*\*개 · 규칙 8 예외 \*\*(\d+)\*\*개 · 판정 대상 \*\*(\d+)\*\*개 · "
                r"이관 완료 \*\*(\d+)\*\*개 · 이관 대기 \*\*(\d+)\*\*개", README)
if not agg:
    fail("R9", "README 의 집계 선언 문장을 찾을 수 없다")
else:
    want = (n_j, n_ex, n_target, n_done, n_wait)
    got = tuple(int(x) for x in agg.groups())
    if got != want:
        fail("R9", "README 집계 선언 %s 이 실측 %s 과 다르다 (여정 · 예외 · 판정 대상 · 이관 완료 · 이관 대기)" % (got, want))

# 이관 완료 여정의 단계 표가 문서와 같은가
for jid in sorted(declared):
    if "이관 완료" not in declared.get(jid, ""):
        continue
    blk = marked(README, "steps:%s" % jid, "R9")
    if blk is None:
        continue
    rows = table_rows(blk)
    doc_steps = JOURNEYS[jid]["steps"]
    if len(rows) != len(doc_steps):
        fail("R9", "README 의 `%s` 단계 표가 %d행인데 문서는 %d단계다" % (jid, len(rows), len(doc_steps)))
        continue
    for row, st in zip(rows, doc_steps):
        g = re.match(r"`(STP-[a-z0-9-]+)`", row[1])
        if not g or g.group(1) != st["id"]:
            fail("R9", "README 의 `%s` 단계 표 순서/식별자가 문서와 다르다: %r vs %s" % (jid, row[1], st["id"]))
            continue
        want_sc = " · ".join(st["screens"]) if st["screens"] else None
        if want_sc is None:
            if "시각화 없음" not in row[2]:
                fail("R9", "`%s`/%s 는 터치포인트 화면이 없는데 README 가 %r 라고 적었다" % (jid, st["id"], row[2]))
        elif row[2] != want_sc:
            fail("R9", "`%s`/%s 의 README 임베드 화면 %r 이 문서 터치포인트 %r 와 다르다" % (jid, st["id"], row[2], want_sc))

# 허브 요약 수치
def hub_summary(label):
    m = re.search(r'<span class="summary-value">(\d+)</span>\s*<span class="meta-label">%s</span>' % re.escape(label), HUB)
    return int(m.group(1)) if m else None


n_docs = len(list(DOCS.rglob("*.md")))
for label, want in (("Journeys", n_j), ("Mockup pages", len(PAGES)), ("Documents", n_docs)):
    got = hub_summary(label)
    if got is None:
        fail("R9", "허브 요약에 `%s` 항목이 없다" % label)
    elif got != want:
        fail("R9", "허브 요약 `%s` 선언 %d 이 실측 %d 과 다르다" % (label, got, want))

hub_doc_links = set(re.findall(r"reader\.html\?doc=([^\"]+)", HUB))
all_md = set(str(p.relative_to(DOCS)) for p in DOCS.rglob("*.md"))
if hub_doc_links != all_md:
    fail("R9", "허브의 문서 링크가 docs/ 의 md 집합과 다르다: 허브에만 %s / 파일에만 %s"
         % (sorted(hub_doc_links - all_md), sorted(all_md - hub_doc_links)))

# 허브가 여정 페이지를 특정 단계에서 열도록 프래그먼트를 붙일 수 있다.
# 링크 대상 파일만 센다 — 프래그먼트 자체는 R8 이 아니라 페이지의 앵커 규칙(R5)이 본다.
hub_mockups = set(re.findall(r'(?:href|src)="mockups/([A-Za-z0-9._-]+\.html)(?:#[^"]*)?"', HUB))
if hub_mockups != set(PAGES):
    fail("R9", "허브의 목업 링크가 docs/mockups 의 파일 집합과 다르다: 허브에만 %s / 파일에만 %s"
         % (sorted(hub_mockups - set(PAGES)), sorted(set(PAGES) - hub_mockups)))

tm = re.search(r"- 목업: \*\*(\d+)개 페이지\*\* — 여정 페이지 \*\*(\d+)개\*\*.*?화면 단위 \*\*(\d+)개\*\*", TRACKER, re.S)
if not tm:
    fail("R9", "doc-tracker 「현재 상태 요약」의 목업 집계 문장을 찾을 수 없다")
else:
    got = tuple(int(x) for x in tm.groups())
    want = (len(PAGES), len(journey_files), len(screen_files))
    if got != want:
        fail("R9", "doc-tracker 목업 집계 %s 이 실측 %s 과 다르다 (총 · 여정 페이지 · 화면 단위)" % (got, want))

# ── 보고 ─────────────────────────────────────────────────────────
print("여정 %d (예외 %d · 이관 완료 %d · 이관 대기 %d) · 목업 페이지 %d (여정 %d + 화면 %d) · 문서 %d"
      % (n_j, n_ex, n_done, n_wait, len(PAGES), len(journey_files), len(screen_files), n_docs))
for n in notes:
    print("  ·", n)

if failures:
    print("\n실패 %d건:" % len(failures))
    for f in failures:
        print("  ✗", f)
    sys.exit(1)
print("\n통과 — 여정 ↔ 목업 정합성 규칙 R0~R10 이상 없음")
