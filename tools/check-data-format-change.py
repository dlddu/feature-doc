#!/usr/bin/env python3
# 데이터 저장 형식 변경 판정기 — PR 에 사람 리뷰가 필요한지 가르는 1차 필터.
#
# PR 의 변경(merge-base..head)에서 **변경된 파일 경로**만 보고, 사람 리뷰가 꼭 필요한 두
# 경우에 닿았는지 판정한다. 닿지 않았으면 워크플로 `review — data format` 이 head 커밋에
# commit status `review/manual-approval` = success 를 붙이고, 닿았으면 **아무 status 도 붙이지
# 않는다**. 즉 이 status 는 "그 관점의 사람 리뷰는 생략해도 된다"는 **적극적 확인**이고,
# 없다고 해서 실패는 아니다. 판정이 불가능하면(git 실패 등) 종료 코드 2 로 끝나고 status 는
# 붙지 않는다.
#
# 의존성 0 (python3 stdlib). 형제 체커 `check-*.py` 와 같은 방침이다.
#
# ── 규칙 ──────────────────────────────────────────────────────────────────
#  D1 마이그레이션    `backend/migrations/**` 의 **모든** 변경. 주석·공백 한 글자도 sqlx
#                     체크섬을 바꿔 부팅을 깨뜨린다(`backend/migrations/README.md`).
#  D6 판정기 자신     이 파일과 워크플로 `data-format-review.yml` 의 변경. 워크플로는
#                     pull_request_target 이라 PR 이 아니라 **base 쪽 판정기**로 돌므로 PR 이
#                     규칙을 고쳐 자기를 통과시킬 수는 없다 — 이 규칙은 그 변경 자체를 사람
#                     눈에 올린다.
#
# ── 이 판정기가 보지 않는 것(의도적, 2026-09-22 축소) ───────────────────────
#  예전의 D2(저장 계층 핵심 파일) · D3(영속화 코드 키워드) · D4(저장 관련 크레이트 버전) ·
#  D5(배포 저장소·replica) 는 뺐다. 키워드·경로 기반이라 오탐이 대부분이었고(읽기 전용
#  SELECT 한 줄, 와이어 필드의 `#[serde(default)]` 하나에도 울렸다), 그 몫은 일반 코드 리뷰가
#  진다. 그러므로 **마이그레이션 없이 운영 데이터를 깨뜨리는 변경**은 이 신호가 잡지 않는다 —
#  리뷰어가 따로 본다:
#    * `backend/src/crypto.rs` 의 봉투 암호화 형식 변경(저장된 LLM 키 복호화 불가)
#    * `backend/src/pipeline.rs` 의 stage key·status 값 변경(기존 행이 고아가 됨)
#    * 분석 문서 등 DB 에 JSON 으로 저장되는 구조체의 모양 변경
#    * sqlx·libsqlite3-sys 등 저장 크레이트의 메이저 업그레이드
#    * `deploy/` 의 PVC·DB 경로·replica·배포 전략(SQLite 단일 writer 전제)
#
# 사용:
#   python3 tools/check-data-format-change.py --base <sha> --head <sha> [--verbose]
#   CI 에서는 $GITHUB_OUTPUT 에 needs_review=true|false, $GITHUB_STEP_SUMMARY 에 근거를 쓴다.
# 종료 코드: 0 = 판정 완료(결과는 출력으로), 2 = 판정 불가.

import argparse
import os
import subprocess
import sys
from collections import defaultdict

SELF_PATHS = {
    "tools/check-data-format-change.py",
    ".github/workflows/data-format-review.yml",
}


def git(*args):
    r = subprocess.run(["git", *args], capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} 실패: {r.stderr.strip()}")
    return r.stdout


def classify(files):
    """규칙별 근거 목록. 비어 있으면 사람 리뷰 불필요."""
    hits = defaultdict(list)
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

    hits = classify(files)
    needs_review = bool(hits)

    out = ["## 데이터 저장 형식 변경 판정: "
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
