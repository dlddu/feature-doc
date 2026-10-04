#!/usr/bin/env python3
# 수동 승인 판정기 — 케이스 D1·D6·D7 의 대상·이유와 이 판정기가 보지 않는 것은
# docs/review-policy.md 가 SSOT 다.

import argparse
import os
import subprocess
import sys
from collections import defaultdict

SELF_PATHS = {
    "tools/check-data-format-change.py",
    ".github/workflows/data-format-review.yml",
    "docs/review-policy.md",
}

CRITERIA_DOC = "docs/data-model/README.md"
CRITERIA_HEADING = "## 풀스캔 허용 기준"


def git(*args):
    r = subprocess.run(["git", *args], capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} 실패: {r.stderr.strip()}")
    return r.stdout


def criteria_rows(rev):
    spec = f"{rev}:{CRITERIA_DOC}"
    if subprocess.run(["git", "cat-file", "-e", spec], capture_output=True).returncode != 0:
        return None
    section = None
    for line in git("show", spec).splitlines():
        if line.strip() == CRITERIA_HEADING:
            section = []
        elif section is not None and line.startswith("## "):
            break
        elif section is not None and line.lstrip().startswith("|"):
            section.append(line.strip())
    return section


def classify(files, base, head):
    hits = defaultdict(list)
    if CRITERIA_DOC in files:
        mb = git("merge-base", base, head).strip()
        if criteria_rows(mb) != criteria_rows(head):
            hits["D7 풀스캔 허용 기준"].append(CRITERIA_DOC)
    for f in files:
        if f in SELF_PATHS:
            hits["D6 판정기 자신"].append(f)
        if f.startswith("backend/migrations/"):
            hits["D1 마이그레이션"].append(f)
    return hits


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", required=True, help="PR base 커밋 (merge-base 계산에 쓴다)")
    ap.add_argument("--head", required=True, help="PR head 커밋")
    ap.add_argument("--verbose", action="store_true", help="통과해도 검사한 파일 목록을 출력")
    args = ap.parse_args()

    rng = f"{args.base}...{args.head}"
    try:
        # `--no-renames`: 이름만 바뀐 마이그레이션도 옛 경로(삭제)와 새 경로(추가)로 둘 다 보인다.
        files = [f for f in git("diff", "--no-renames", "--name-only", rng).splitlines() if f]
    except RuntimeError as e:
        print(f"::error::판정 불가 — {e}", file=sys.stderr)
        return 2

    try:
        hits = classify(files, args.base, args.head)
    except RuntimeError as e:
        print(f"::error::판정 불가 — {e}", file=sys.stderr)
        return 2
    needs_review = bool(hits)

    out = ["## 수동 승인 케이스 판정: "
           + ("⚠️ 사람 리뷰 필요 (status 미부여)" if needs_review
              else "✅ 해당 없음 (`review/manual-approval` = success)"),
           "", f"변경 파일 {len(files)}개 · 범위 `{rng}`", ""]
    for rule in sorted(hits):
        out.append(f"### {rule} — {len(hits[rule])}건")
        out += [f"- `{h}`" for h in hits[rule]]
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
