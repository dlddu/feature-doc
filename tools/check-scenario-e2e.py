#!/usr/bin/env python3
# 테스트 시나리오 ↔ e2e 1:1 정합성 게이트 (reconciler `tbm_feature-doc-scenario-e2e`).
#
# 시나리오의 단일 소스(SSOT)는 `docs/test/` 의 `### 시나리오 N: …` 헤딩이다. 이 스크립트는
# "모든 시나리오가 자기를 주검증하는 spec 파일을 정확히 1개 갖는가"를 기계로 판정한다.
# 붉으면 닫는 길은 셋뿐이다 — **전용 spec 을 만들거나**, `docs/doc-tracker.md` 의
# 「e2e 매핑」 절 **예외 목록**(규칙 4: 자동화가 곤란)이나 **구현 대기 표**(규칙 6: 기능
# 미구현)에 사유와 함께 **등재하거나**. 등재가 유일한 면제 통로이며, 스크립트는 등재 자체의
# 무결성도 검사한다.
#
# 의존성 0 (python3 stdlib). 형제 게이트 `check-journey-mockup.py` ·
# `check-mockup-render.py` 와 같은 방침이다.
#
# ── 규칙 ──────────────────────────────────────────────────────────────────
#  S0 파싱 무결성   「e2e 매핑」 절의 네 표(매핑 · 예외 · 구현 대기 · 미매핑 잔여)와
#                   집계 문단이 파싱되고, 표 캡션의 건수가 실제 행 수와 같다.
#  S1 선언 형식     `e2e/tests/` 최상위 `*.spec.ts` 는 각각
#                   `// 검증 시나리오: <문서 파일명>#시나리오 <N>` 을 **정확히 1개** 선언한다.
#                   0개면 고아(규칙 3), 2개 이상이면 묶음(규칙 2) — 둘 다 실패.
#                   파일명 `sc<문서번호>-<시나리오번호>-<slug>.spec.ts` 가 선언과 교차 일치한다.
#  S2 선언 실재성   선언된 시나리오가 `docs/test/<문서>` 에 `### 시나리오 <N>:` 로 실재하고,
#                   두 파일이 같은 시나리오를 선언하지 않는다(규칙 1 의 "2개 이상이면 중복").
#  S3 등재 무결성   매핑 표의 (시나리오, 파일) 쌍이 실제 파일 선언과 정확히 같다(누락·유령 0).
#                   예외·구현 대기·미매핑 표의 시나리오가 전부 실재하고, 네 버킷이 서로
#                   겹치지 않는다. 사라진 시나리오를 근거로 남은 공전 행을 여기서 잡는다.
#  S4 집계·불변식   시나리오 총수 = 매핑 + 예외 + 구현 대기 + 미매핑. 집계 문단에 적힌
#                   숫자가 실측과 같다. 모델의 불변식 (시나리오 − 예외 − 구현 대기) 는
#                   목표 매칭 파일 수이고, 지금 그것과 실제 파일 수의 차가 미매핑이다.
#  S5 래칫          미매핑 집합이 등재 표와 **집합으로** 일치하고, 그 수가 캡션의 상한 이하다.
#                   늘면 실패 — 줄면 상한을 낮추라고 실패한다(형제 게이트의 원장 래칫과 같다).
#
# ── 이 게이트가 보지 않는 것(의도적) ─────────────────────────────────────
#  * **시나리오 문구와 단정 내용의 의미적 일치**는 보지 않는다. 선언이 붙어 있는지까지가
#    기계의 몫이고, "이 spec 이 그 시나리오를 제대로 검증하는가"는 리뷰의 몫이다.
#    모델 정의도 그것을 task 단계의 판단 사항으로 남긴다.
#  * **AC ↔ 시나리오 층**은 범위 밖이다(모델 정의: 그 층은 제품 문서 체계의 몫).
#  * `e2e/smoke.sh` · `playwright.config.ts` · `package.json` · `e2e/support/` 는
#    매칭 단위가 아니므로 세지 않는다.
#  대조된 내역은 `--verbose` 로 전부 출력된다. 무엇이 비교됐는지 눈으로 확인할 것.

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TRACKER = ROOT / "docs" / "doc-tracker.md"
TEST_DOC_DIR = ROOT / "docs" / "test"
SPEC_DIR = ROOT / "e2e" / "tests"

SECTION = "## e2e 매핑"
H_MAPPING = "**매핑"
H_EXEMPT = "**예외 목록"
H_PENDING = "**구현 대기"
H_UNMAPPED = "**미매핑 잔여"
H_TOTALS = "**집계**"

DECL_RE = re.compile(r"^// 검증 시나리오: ([0-9A-Za-z._-]+\.md)#시나리오 (\d+)\s*$")
HEADING_RE = re.compile(r"^### 시나리오 (\d+):")
FILENAME_RE = re.compile(r"^sc(\d{2})-(\d{2})-[a-z0-9-]+\.spec\.ts$")
SCENARIO_TOKEN_RE = re.compile(r"`([0-9A-Za-z._-]+\.md)#시나리오 (\d+)`")
SPEC_TOKEN_RE = re.compile(r"`(sc\d{2}-\d{2}-[a-z0-9-]+\.spec\.ts)`")

failures: list[str] = []
verbose = "--verbose" in sys.argv


def fail(rule: str, message: str) -> None:
    failures.append(f"[{rule}] {message}")


def note(message: str) -> None:
    if verbose:
        print(f"  · {message}")


def key(doc: str, num: int) -> str:
    return f"{doc}#시나리오 {num}"


# ── 실측: 시나리오 집합 ────────────────────────────────────────────────────

def read_scenarios() -> dict[str, list[int]]:
    """docs/test/*.md → {문서 파일명: [시나리오 번호…]} (선언 순서 그대로)."""
    found: dict[str, list[int]] = {}
    for path in sorted(TEST_DOC_DIR.glob("*.md")):
        nums = [
            int(m.group(1))
            for line in path.read_text(encoding="utf-8").splitlines()
            if (m := HEADING_RE.match(line))
        ]
        found[path.name] = nums
    return found


# ── 실측: spec 선언 ────────────────────────────────────────────────────────

def read_declarations(scenarios: dict[str, list[int]]) -> dict[str, str]:
    """e2e/tests/*.spec.ts → {시나리오 키: 파일명}. S1·S2 를 함께 판정한다."""
    declared: dict[str, str] = {}
    owner: dict[str, str] = {}
    for path in sorted(SPEC_DIR.glob("*.spec.ts")):
        name = path.name
        lines = path.read_text(encoding="utf-8").splitlines()
        decls = [m for line in lines if (m := DECL_RE.match(line))]

        if not decls:
            fail(
                "S1",
                f"{name}: `// 검증 시나리오:` 선언이 없다 — 어디에도 매핑되지 않은 "
                f"고아 파일이다(규칙 3). 시나리오를 선언하거나 파일을 정리할 것",
            )
            continue
        if len(decls) > 1:
            joined = " · ".join(key(m.group(1), int(m.group(2))) for m in decls)
            fail(
                "S1",
                f"{name}: 선언이 {len(decls)}개다({joined}) — 여러 시나리오를 묶은 파일은 "
                f"분리 대상이다(규칙 2). 파일당 정확히 1개만 선언할 것",
            )
            continue

        doc, num = decls[0].group(1), int(decls[0].group(2))
        k = key(doc, num)

        fname = FILENAME_RE.match(name)
        if not fname:
            fail(
                "S1",
                f"{name}: 파일명이 규약 `sc<문서번호>-<시나리오번호>-<slug>.spec.ts` 가 아니다",
            )
        else:
            doc_no, sc_no = fname.group(1), int(fname.group(2))
            if not doc.startswith(f"{doc_no}-"):
                fail("S1", f"{name}: 파일명의 문서번호 `{doc_no}` 가 선언한 문서 `{doc}` 와 어긋난다")
            if sc_no != num:
                fail("S1", f"{name}: 파일명의 시나리오번호 `{sc_no:02d}` 가 선언한 `{num}` 과 어긋난다")

        if doc not in scenarios:
            fail("S2", f"{name}: 선언한 문서 `{doc}` 가 `docs/test/` 에 없다")
        elif num not in scenarios[doc]:
            fail(
                "S2",
                f"{name}: 선언한 `{k}` 가 문서에 없다 — 시나리오가 개명·재배치·삭제됐다면 "
                f"선언과 등재 표를 함께 갱신할 것",
            )

        if k in owner:
            fail("S2", f"`{k}` 를 두 파일이 선언한다: `{owner[k]}` · `{name}` (규칙 1 중복)")
        else:
            owner[k] = name
            declared[k] = name
        note(f"선언 {name} → {k}")
    return declared


# ── 등재: doc-tracker 「e2e 매핑」 절 ─────────────────────────────────────

def slice_section(text: str) -> list[str] | None:
    lines = text.splitlines()
    try:
        start = next(i for i, line in enumerate(lines) if line.strip() == SECTION)
    except StopIteration:
        return None
    end = len(lines)
    for i in range(start + 1, len(lines)):
        if lines[i].startswith("## "):
            end = i
            break
    return lines[start:end]


def split_blocks(section: list[str]) -> dict[str, list[str]]:
    """굵은 소제목(**…**)을 경계로 절을 나눈다. 키는 소제목 줄 전체."""
    blocks: dict[str, list[str]] = {}
    current: str | None = None
    for line in section:
        if line.startswith("**") and line.rstrip().endswith("**"):
            current = line.strip()
            blocks[current] = []
        elif current is not None:
            blocks[current].append(line)
    return blocks


def find_block(blocks: dict[str, list[str]], prefix: str) -> tuple[str, list[str]] | None:
    for head, body in blocks.items():
        if head.startswith(prefix):
            return head, body
    return None


def table_rows(body: list[str]) -> list[str]:
    """마크다운 표의 데이터 행만(헤더·구분선 제외)."""
    rows = []
    for line in body:
        s = line.strip()
        if not s.startswith("|"):
            continue
        if set(s) <= set("|-: "):
            continue
        rows.append(s)
    return rows[1:] if rows else rows


def caption_numbers(head: str) -> dict[str, int]:
    out: dict[str, int] = {}
    if m := re.search(r"\((\d+)건", head):
        out["count"] = int(m.group(1))
    if m := re.search(r"상한\s*(\d+)", head):
        out["cap"] = int(m.group(1))
    return out


def collect(head: str, body: list[str], rule: str, label: str) -> tuple[list[str], dict[str, int]]:
    """표 행에서 시나리오 토큰을 뽑고, 캡션 건수와 행 수의 일치를 본다."""
    rows = table_rows(body)
    keys: list[str] = []
    for row in rows:
        tokens = SCENARIO_TOKEN_RE.findall(row)
        if not tokens:
            fail(rule, f"{label}: 시나리오 토큰이 없는 행이 있다 — `{row[:70]}`")
            continue
        doc, num = tokens[0]
        keys.append(key(doc, int(num)))
    nums = caption_numbers(head)
    if "count" in nums and nums["count"] != len(keys):
        fail(rule, f"{label}: 캡션은 {nums['count']}건인데 표에는 {len(keys)}행이다")
    dupes = {k for k in keys if keys.count(k) > 1}
    if dupes:
        fail(rule, f"{label}: 같은 시나리오가 여러 행에 있다 — {' · '.join(sorted(dupes))}")
    return keys, nums


def main() -> int:
    scenarios = read_scenarios()
    if not scenarios:
        fail("S0", "`docs/test/` 에서 테스트 문서를 하나도 찾지 못했다")
        return report()
    all_keys = {key(doc, n) for doc, nums in scenarios.items() for n in nums}
    total = len(all_keys)
    note(f"시나리오 실측 {total}건 — " + " · ".join(f"{d}:{len(n)}" for d, n in scenarios.items()))

    declared = read_declarations(scenarios)

    section = slice_section(TRACKER.read_text(encoding="utf-8"))
    if section is None:
        fail("S0", f"`docs/doc-tracker.md` 에 `{SECTION}` 절이 없다")
        return report()
    blocks = split_blocks(section)

    buckets: dict[str, list[str]] = {}
    caps: dict[str, dict[str, int]] = {}
    mapping_pairs: list[tuple[str, str]] = []
    for name, prefix, rule in (
        ("매핑", H_MAPPING, "S3"),
        ("예외", H_EXEMPT, "S3"),
        ("구현 대기", H_PENDING, "S3"),
        ("미매핑", H_UNMAPPED, "S5"),
    ):
        found = find_block(blocks, prefix)
        if found is None:
            fail("S0", f"「e2e 매핑」 절에 `{prefix}…**` 소제목이 없다")
            buckets[name] = []
            caps[name] = {}
            continue
        head, body = found
        keys, nums = collect(head, body, rule, name)
        buckets[name] = keys
        caps[name] = nums
        if name == "매핑":
            for row in table_rows(body):
                sc = SCENARIO_TOKEN_RE.findall(row)
                sp = SPEC_TOKEN_RE.findall(row)
                if sc and sp:
                    mapping_pairs.append((key(sc[0][0], int(sc[0][1])), sp[0]))
                elif sc:
                    fail("S3", f"매핑: `{key(sc[0][0], int(sc[0][1]))}` 행에 spec 파일 토큰이 없다")

    if failures and not buckets.get("매핑"):
        return report()

    # S3 — 등재 시나리오의 실재성과 버킷 간 배타성
    for name, keys in buckets.items():
        for k in keys:
            if k not in all_keys:
                fail(
                    "S3",
                    f"{name}: `{k}` 가 `docs/test/` 에 없다 — 사라진 시나리오를 근거로 남은 "
                    f"공전 행이다. 행을 지우거나 시나리오를 되살릴 것",
                )
    names = list(buckets)
    for i, a in enumerate(names):
        for b in names[i + 1 :]:
            overlap = set(buckets[a]) & set(buckets[b])
            if overlap:
                fail("S3", f"{a} 와 {b} 에 같은 시나리오가 있다 — {' · '.join(sorted(overlap))}")

    # S3 — 매핑 표 ↔ 실제 파일 선언
    table_map = dict(mapping_pairs)
    for k, fname in sorted(table_map.items()):
        if k not in declared:
            fail("S3", f"매핑: `{k} → {fname}` 이 등재돼 있는데 그렇게 선언한 파일이 없다(유령 행)")
        elif declared[k] != fname:
            fail("S3", f"매핑: `{k}` 의 등재 파일 `{fname}` 과 실제 선언 파일 `{declared[k]}` 이 다르다")
    for k, fname in sorted(declared.items()):
        if k not in table_map:
            fail("S3", f"매핑: `{fname}` 이 `{k}` 를 선언하는데 등재 표에 없다(누락 행)")
    note(f"등재 대조 — 매핑 {len(table_map)} · 선언 {len(declared)}")

    # S4 — 집계·불변식
    counted = {name: len(keys) for name, keys in buckets.items()}
    covered = sum(counted.values())
    if covered != total:
        missing = sorted(all_keys - set().union(*(set(v) for v in buckets.values())) if buckets else all_keys)
        fail(
            "S4",
            f"시나리오 {total}건 중 등재된 것은 {covered}건이다 "
            f"(매핑 {counted['매핑']} + 예외 {counted['예외']} + 구현 대기 {counted['구현 대기']} "
            f"+ 미매핑 {counted['미매핑']}). 어디에도 없는 것: "
            + (" · ".join(missing) if missing else "(중복 등재)"),
        )
    totals = find_block(blocks, H_TOTALS)
    if totals is None:
        fail("S0", "「e2e 매핑」 절에 `**집계**` 문단이 없다")
    else:
        body = "\n".join(totals[1])
        expected = {
            "시나리오": total,
            "매핑": counted["매핑"],
            "예외": counted["예외"],
            "구현 대기": counted["구현 대기"],
            "미매핑": counted["미매핑"],
        }
        for label, want in expected.items():
            m = re.search(rf"{re.escape(label)}[^0-9\n]*\*\*(\d+)\*\*", body)
            if m is None:
                fail("S4", f"집계 문단에 `{label}` 수치가 없다(굵게 적을 것: `{label}: **{want}**`)")
            elif int(m.group(1)) != want:
                fail("S4", f"집계 문단의 `{label}` 은 {m.group(1)} 인데 실측은 {want} 다")
    target = total - counted["예외"] - counted["구현 대기"]
    note(f"불변식 목표 (시나리오 {total} − 예외 {counted['예외']} − 구현 대기 {counted['구현 대기']}) = {target} · 실제 매칭 파일 {len(declared)}")

    # S5 — 래칫
    computed = sorted(all_keys - set(buckets["매핑"]) - set(buckets["예외"]) - set(buckets["구현 대기"]))
    listed = sorted(set(buckets["미매핑"]))
    if computed != listed:
        only_real = sorted(set(computed) - set(listed))
        only_doc = sorted(set(listed) - set(computed))
        detail = []
        if only_real:
            detail.append("표에 없는 실측 잔여: " + " · ".join(only_real))
        if only_doc:
            detail.append("실측에 없는 표 행: " + " · ".join(only_doc))
        fail("S5", "미매핑 잔여 표가 실측 집합과 다르다 — " + " / ".join(detail))
    cap = caps.get("미매핑", {}).get("cap")
    if cap is None:
        fail("S5", "미매핑 잔여 소제목에 `상한 N` 이 없다")
    elif len(computed) > cap:
        fail("S5", f"미매핑 잔여가 {len(computed)}건으로 상한 {cap} 을 넘었다 — 늘릴 수 없다")
    elif len(computed) < cap:
        fail("S5", f"미매핑 잔여가 {len(computed)}건으로 상한 {cap} 보다 적다 — 상한을 {len(computed)} 로 낮출 것")
    else:
        note(f"래칫 미매핑 {len(computed)}/{cap}")

    return report()


def report() -> int:
    if failures:
        print(f"✗ 시나리오 ↔ e2e 정합성 실패 {len(failures)}건")
        for f in failures:
            print(f"  {f}")
        return 1
    print("✓ 시나리오 ↔ e2e 정합성 통과")
    return 0


if __name__ == "__main__":
    sys.exit(main())
