#!/usr/bin/env python3
import argparse
import json
import os
import re
import sqlite3
import subprocess
import sys

DOC_DIR = "docs/data-model"
README = DOC_DIR + "/README.md"
ERD = DOC_DIR + "/erd.md"
BLOCK_TAG = "data-model-scope"
SINGLE_KEYS = ("migrations", "checker")
ERE_KEYS = ("exclude", "site", "sql", "schema-exclude")
REQUIRED_KEYS = ("migrations", "checker", "scope", "site", "sql", "schema-exclude")
MIGRATION_RE = re.compile(r"^(\d+)_(.+?)(\.up|\.down)?\.sql$")
COLUMN_HEADER = ["컬럼", "타입", "NULL", "키"]
INDEX_HEADER = ["인덱스", "컬럼", "UNIQUE", "조건"]
PENDING = {
    "2": "쿼리 패턴 카탈로그(query-patterns.md)와 지점 추출(C4)은 판정 슬라이스 (2)에서 들어온다",
    "3": "지원 인덱스 판정(C5 쿼리 플랜)은 판정 슬라이스 (3)에서 들어온다",
}


class Undecidable(Exception):
    pass


def norm_space(text):
    return re.sub(r"\s+", " ", text).strip()


def norm_type(text):
    return norm_space(text or "").upper()


def read(root, rel):
    path = os.path.join(root, rel)
    try:
        with open(path, encoding="utf-8") as fh:
            return fh.read()
    except OSError as exc:
        raise Undecidable(f"{rel} 를 읽을 수 없다: {exc.strerror}")


def parse_block(text):
    lines = text.splitlines()
    body = None
    for i, line in enumerate(lines):
        if line == "```" + BLOCK_TAG or line.startswith("```" + BLOCK_TAG + " "):
            body = []
            for inner in lines[i + 1:]:
                if inner.startswith("```"):
                    break
                body.append(inner)
            break
    if body is None:
        raise Undecidable(f"{README} 에 ```{BLOCK_TAG} 블록이 없다")
    values = {}
    for line in body:
        if not line.strip():
            continue
        key, sep, value = line.partition(":")
        if not sep:
            raise Undecidable(f"블록 줄을 읽을 수 없다: {line!r}")
        values.setdefault(key.strip(), []).append(value.strip())
    missing = [k for k in REQUIRED_KEYS if not any(values.get(k, []))]
    if missing:
        raise Undecidable(f"블록에 키가 없거나 비었다: {', '.join(missing)}")
    block = {}
    for key, vals in values.items():
        if key in SINGLE_KEYS:
            block[key] = vals[0]
        elif key == "scope":
            block[key] = " ".join(vals).split()
        elif key in ERE_KEYS:
            joined = "|".join(vals)
            try:
                block[key] = re.compile(joined)
            except re.error as exc:
                raise Undecidable(f"블록 `{key}` ERE 가 깨졌다: {exc}")
    block.setdefault("exclude", None)
    return block


def migration_files(root, rel):
    path = os.path.join(root, rel)
    if not os.path.isdir(path):
        raise Undecidable(f"마이그레이션 디렉터리 {rel} 가 없다")
    found = []
    for name in os.listdir(path):
        m = MIGRATION_RE.match(name)
        if not m or m.group(3) == ".down":
            continue
        found.append((int(m.group(1)), name))
    versions = [v for v, _ in found]
    if len(set(versions)) != len(versions):
        raise Undecidable("마이그레이션 버전이 겹친다")
    return [os.path.join(rel, name) for _, name in sorted(found)]


def apply_migrations(root, files):
    db = sqlite3.connect(":memory:")
    for rel in files:
        try:
            db.executescript(read(root, rel))
        except sqlite3.Error as exc:
            raise Undecidable(f"마이그레이션 적용 실패 {rel}: {exc}")
    return db


def index_expr_list(sql):
    if not sql:
        return None, None
    m = re.search(r"\bON\s+\S+\s*\(", sql, re.I)
    if not m:
        return None, None
    depth, start = 1, m.end()
    for pos in range(start, len(sql)):
        ch = sql[pos]
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                break
    inner, rest = sql[start:pos], sql[pos + 1:]
    parts, depth, cur = [], 0, ""
    for ch in inner:
        if ch == "," and depth == 0:
            parts.append(norm_space(cur))
            cur = ""
            continue
        depth += ch == "("
        depth -= ch == ")"
        cur += ch
    parts.append(norm_space(cur))
    where = re.search(r"\bWHERE\b(.*)$", rest, re.I | re.S)
    return parts, norm_space(where.group(1)) if where else None


def observe_schema(db, exclude):
    tables = {}
    names = [r[0] for r in db.execute(
        "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")]
    for table in names:
        if exclude.search(table):
            continue
        cols = db.execute(f"PRAGMA table_info('{table}')").fetchall()
        columns = [(c[1], norm_type(c[2]), "NO" if c[3] else "YES") for c in cols]
        pk = tuple(c[1] for c in sorted((c for c in cols if c[5]), key=lambda c: c[5]))
        fks = {}
        for row in db.execute(f"PRAGMA foreign_key_list('{table}')"):
            fks.setdefault(row[0], []).append(row)
        fk_set = set()
        for rows in fks.values():
            rows.sort(key=lambda r: r[1])
            ref = rows[0][2]
            to_cols = [r[4] for r in rows]
            if any(c is None for c in to_cols):
                ref_info = db.execute(f"PRAGMA table_info('{ref}')").fetchall()
                to_cols = [c[1] for c in sorted((c for c in ref_info if c[5]), key=lambda c: c[5])]
            fk_set.add((tuple(r[3] for r in rows), ref, tuple(to_cols)))
        indexes = {}
        for _, iname, unique, origin, partial in db.execute(f"PRAGMA index_list('{table}')"):
            if origin == "pk":
                continue
            info = db.execute(f"PRAGMA index_xinfo('{iname}')").fetchall()
            keys = [r for r in info if r[5]]
            sql = db.execute("SELECT sql FROM sqlite_master WHERE type = 'index' AND name = ?",
                             (iname,)).fetchone()
            exprs, where = index_expr_list(sql[0] if sql else None)
            cols_seq = tuple(
                r[2] if r[1] >= 0 else (exprs[i] if exprs and i < len(exprs) else "<식>")
                for i, r in enumerate(keys))
            if origin == "c":
                label = iname
            else:
                label = f"UNIQUE({','.join(table + '.' + c for c in cols_seq)})"
            key = (cols_seq, "YES" if unique else "NO", where if partial else "-")
            indexes[key] = label
        tables[table] = {"columns": columns, "pk": pk, "fks": fk_set, "indexes": indexes}
    return tables


def split_row(line):
    cells = line.strip()
    if not (cells.startswith("|") and cells.endswith("|")):
        return None
    return [c.strip() for c in cells[1:-1].split("|")]


def unquote(cell):
    return cell[1:-1] if len(cell) >= 2 and cell[0] == cell[-1] == "`" else cell


def parse_tables_after(lines, start, header):
    rows = []
    for idx in range(start, len(lines)):
        cells = split_row(lines[idx])
        if cells == header:
            for line in lines[idx + 2:]:
                cells = split_row(line)
                if cells is None:
                    break
                rows.append(cells)
            return rows
    return None


REL_RE = re.compile(
    r"^\s*([A-Za-z0-9_]+)\s+(\|\||\|o)--(o\{|o\|)\s+([A-Za-z0-9_]+)\s*:\s*\"?([^\"]*)\"?\s*$")
ENTITY_RE = re.compile(r"^\s*([A-Za-z0-9_]+)\s*$")
MEANING_RE = re.compile(r"^의미:\s*\[`([^`]+)`\]\(([^)]+)\)\s*$")


def parse_erd(text):
    lines = text.splitlines()
    mermaid = None
    for i, line in enumerate(lines):
        if line.strip() == "```mermaid":
            body = []
            for inner in lines[i + 1:]:
                if inner.strip().startswith("```"):
                    break
                body.append(inner)
            mermaid = body
            break
    if mermaid is None or not mermaid or mermaid[0].strip() != "erDiagram":
        raise Undecidable(f"{ERD} 에 ```mermaid erDiagram 블록이 없다")
    entities, relations = set(), set()
    for line in mermaid[1:]:
        if not line.strip():
            continue
        m = REL_RE.match(line)
        if m:
            parent, pcard, ccard, child, label = m.groups()
            entities.update((parent, child))
            relations.add((parent, pcard, ccard, child, label.strip()))
            continue
        m = ENTITY_RE.match(line)
        if m:
            entities.add(m.group(1))
            continue
        raise Undecidable(f"mermaid 줄을 읽을 수 없다: {line.strip()!r}")
    sections = {}
    heads = [(i, re.match(r"^### `([^`]+)`\s*$", l)) for i, l in enumerate(lines)]
    heads = [(i, m.group(1)) for i, m in heads if m]
    for n, (i, table) in enumerate(heads):
        end = heads[n + 1][0] if n + 1 < len(heads) else len(lines)
        chunk = lines[i + 1:end]
        if table in sections:
            raise Undecidable(f"{ERD} 에 `{table}` 절이 두 번 있다")
        meaning = [MEANING_RE.match(l.strip()) for l in chunk if l.strip().startswith("의미:")]
        cols = parse_tables_after(chunk, 0, COLUMN_HEADER)
        idx = parse_tables_after(chunk, 0, INDEX_HEADER)
        if cols is None or idx is None:
            raise Undecidable(f"{ERD} `{table}` 절에 컬럼 표나 인덱스 표가 없다")
        sections[table] = {"meaning": meaning, "columns": cols, "indexes": idx}
    return entities, relations, sections


def doc_schema(table, sec, problems):
    columns, pk, fks = [], [], set()
    for row in sec["columns"]:
        if len(row) < 4:
            problems.append(f"`{table}` 컬럼 표 행의 칸이 모자라다: {row}")
            continue
        name, typ, null, key = unquote(row[0]), norm_type(unquote(row[1])), row[2], row[3]
        columns.append((name, typ, null))
        for part in [p.strip() for p in key.split(",") if p.strip()]:
            if part == "PK":
                pk.append(name)
                continue
            m = re.match(r"^FK → `?([A-Za-z0-9_]+)\.([A-Za-z0-9_]+)`?$", part)
            if m:
                fks.add(((name,), m.group(1), (m.group(2),)))
            else:
                problems.append(f"`{table}`.`{name}` 키 칸을 읽을 수 없다: {part!r}")
    indexes = {}
    for row in sec["indexes"]:
        if len(row) < 4:
            problems.append(f"`{table}` 인덱스 표 행의 칸이 모자라다: {row}")
            continue
        label = unquote(row[0])
        cols_seq = tuple(unquote(c.strip()) for c in row[1].split(","))
        cond = unquote(row[3])
        indexes[(cols_seq, row[2], norm_space(cond) if cond != "-" else "-")] = label
    return {"columns": columns, "pk": tuple(pk), "fks": fks, "indexes": indexes}


def expected_relations(schema):
    rels = set()
    for table, t in schema.items():
        nullable = {c[0]: c[2] == "YES" for c in t["columns"]}
        uniques = {k[0] for k in t["indexes"] if k[1] == "YES" and k[2] == "-"}
        if t["pk"]:
            uniques.add(t["pk"])
        for from_cols, ref, _ in t["fks"]:
            pcard = "|o" if any(nullable.get(c, True) for c in from_cols) else "||"
            ccard = "o|" if from_cols in uniques else "o{"
            rels.add((ref, pcard, ccard, table, ",".join(from_cols)))
    return rels


def fmt_col(c, t):
    keys = []
    if c[0] in t["pk"]:
        keys.append("PK")
    for from_cols, ref, to_cols in sorted(t["fks"]):
        if from_cols == (c[0],):
            keys.append(f"FK → {ref}.{to_cols[0]}")
    return f"| `{c[0]}` | {c[1]} | {c[2]} | {', '.join(keys)} |"


def fmt_index(key, label):
    return f"| `{label}` | {', '.join(key[0])} | {key[1]} | {key[2] if key[2] == '-' else '`' + key[2] + '`'} |"


def fmt_rel(r):
    return f"    {r[0]} {r[1]}--{r[2]} {r[3]} : \"{r[4]}\""


def render_section(table, t):
    out = [f"### `{table}`", "", "의미: <코드 doc 주석 링크>", "",
           "| " + " | ".join(COLUMN_HEADER) + " |", "|---|---|---|---|"]
    out += [fmt_col(c, t) for c in t["columns"]]
    out += ["", "| " + " | ".join(INDEX_HEADER) + " |", "|---|---|---|---|"]
    out += [fmt_index(k, l) for k, l in sorted(t["indexes"].items(), key=lambda kv: kv[1])]
    return "\n".join(out)


def check_meaning(root, table, sec, out):
    if len(sec["meaning"]) != 1 or sec["meaning"][0] is None:
        out.append({"kind": "meaning", "table": table,
                    "detail": "`의미: [`항목`](경로)` 줄이 정확히 하나여야 한다"})
        return
    item, link = sec["meaning"][0].groups()
    target = os.path.normpath(os.path.join(DOC_DIR, link.split("#")[0]))
    path = os.path.join(root, target)
    if not os.path.isfile(path):
        out.append({"kind": "meaning", "table": table, "detail": f"링크 대상 {target} 가 없다"})
        return
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    stem = os.path.splitext(os.path.basename(target))[0]
    if item == stem:
        first = next((l for l in text.splitlines() if l.strip()), "")
        if not first.startswith("//!"):
            out.append({"kind": "meaning", "table": table,
                        "detail": f"{target} 첫 줄이 모듈 doc 주석(`//!`)이 아니다"})
        return
    pat = re.compile(r"((?:^[ \t]*///.*\n)(?:^[ \t]*(?:///|#\[).*\n)*)^[ \t]*(?:pub(?:\([^)]*\))? )?"
                     r"(?:struct|enum|type|trait|fn|const|mod) " + re.escape(item) + r"\b", re.M)
    if not pat.search(text):
        out.append({"kind": "meaning", "table": table,
                    "detail": f"{target} 에 doc 주석(`///`)이 달린 항목 `{item}` 이 없다"})


def check_erd(root, schema):
    entities, relations, sections = parse_erd(read(root, ERD))
    out = []
    for table in sorted(set(schema) - set(sections)):
        out.append({"kind": "missing-table", "table": table,
                    "expected": render_section(table, schema[table])})
    for table in sorted(set(sections) - set(schema)):
        out.append({"kind": "extra-table", "table": table})
    for table in sorted(set(schema) & set(sections)):
        actual, problems = schema[table], []
        doc = doc_schema(table, sections[table], problems)
        out += [{"kind": "unreadable", "table": table, "detail": p} for p in problems]
        check_meaning(root, table, sections[table], out)
        want_cols = {c[0]: c for c in actual["columns"]}
        have_cols = {c[0]: c for c in doc["columns"]}
        t = actual
        for name in sorted(set(want_cols) - set(have_cols)):
            out.append({"kind": "missing-column", "table": table, "expected": fmt_col(want_cols[name], t)})
        for name in sorted(set(have_cols) - set(want_cols)):
            out.append({"kind": "extra-column", "table": table, "column": name})
        for name in sorted(set(want_cols) & set(have_cols)):
            if want_cols[name] != have_cols[name]:
                out.append({"kind": "column-differs", "table": table, "column": name,
                            "expected": fmt_col(want_cols[name], t)})
        if doc["pk"] != actual["pk"]:
            out.append({"kind": "pk-differs", "table": table,
                        "expected": list(actual["pk"]), "documented": list(doc["pk"])})
        for fk in sorted(actual["fks"] - doc["fks"]):
            out.append({"kind": "missing-fk", "table": table,
                        "expected": f"{','.join(fk[0])} → {fk[1]}.{','.join(fk[2])}"})
        for fk in sorted(doc["fks"] - actual["fks"]):
            out.append({"kind": "extra-fk", "table": table,
                        "documented": f"{','.join(fk[0])} → {fk[1]}.{','.join(fk[2])}"})
        for key in sorted(set(actual["indexes"]) - set(doc["indexes"])):
            out.append({"kind": "missing-index", "table": table,
                        "expected": fmt_index(key, actual["indexes"][key])})
        for key in sorted(set(doc["indexes"]) - set(actual["indexes"])):
            out.append({"kind": "extra-index", "table": table,
                        "documented": fmt_index(key, doc["indexes"][key])})
        for key in sorted(set(doc["indexes"]) & set(actual["indexes"])):
            if doc["indexes"][key] != actual["indexes"][key]:
                out.append({"kind": "index-label-differs", "table": table,
                            "expected": fmt_index(key, actual["indexes"][key])})
    for table in sorted(set(schema) - entities):
        out.append({"kind": "missing-entity", "expected": f"    {table}"})
    for table in sorted(entities - set(schema)):
        out.append({"kind": "extra-entity", "entity": table})
    want = expected_relations(schema)
    for rel in sorted(want - relations):
        out.append({"kind": "missing-relation", "expected": fmt_rel(rel)})
    for rel in sorted(relations - want):
        out.append({"kind": "extra-relation", "documented": fmt_rel(rel)})
    return out


def site_candidates(root, block):
    try:
        listed = subprocess.run(["git", "ls-files", "--", *block["scope"]], cwd=root,
                                capture_output=True, text=True, check=True).stdout.split("\n")
    except (OSError, subprocess.CalledProcessError) as exc:
        raise Undecidable(f"git ls-files 실패: {exc}")
    count = 0
    for rel in sorted(f for f in listed if f):
        if block["exclude"] and block["exclude"].search(rel):
            continue
        try:
            with open(os.path.join(root, rel), encoding="utf-8") as fh:
                count += sum(1 for line in fh if block["site"].search(line))
        except (OSError, UnicodeDecodeError):
            continue
    return count


def run(root):
    block = parse_block(read(root, README))
    files = migration_files(root, block["migrations"])
    schema = observe_schema(apply_migrations(root, files), block["schema-exclude"])
    report = {
        "engine": f"sqlite {sqlite3.sqlite_version} (python sqlite3)",
        "migrations": len(files),
        "counts": {
            "tables": len(schema),
            "columns": sum(len(t["columns"]) for t in schema.values()),
            "fks": sum(len(t["fks"]) for t in schema.values()),
            "indexes": sum(len(t["indexes"]) for t in schema.values()),
            "site_candidates": site_candidates(root, block),
        },
        "invariants": {},
    }
    erd = check_erd(root, schema)
    report["invariants"]["1"] = {"status": "violations" if erd else "ok", "violations": erd}
    for key, reason in PENDING.items():
        report["invariants"][key] = {"status": "undecidable", "reason": reason}
    statuses = [v["status"] for v in report["invariants"].values()]
    report["exit"] = 2 if "undecidable" in statuses else 1 if "violations" in statuses else 0
    return report


def human(report):
    out = []
    if "error" in report:
        return f"판정 불가: {report['error']}"
    c = report["counts"]
    out.append(f"엔진 {report['engine']} · 마이그레이션 {report['migrations']} · 테이블 {c['tables']}"
               f" · 컬럼 {c['columns']} · FK {c['fks']} · 인덱스(PK 제외) {c['indexes']}"
               f" · site 후보 {c['site_candidates']}줄")
    for key in sorted(report["invariants"]):
        inv = report["invariants"][key]
        if inv["status"] == "ok":
            out.append(f"✓ 불변식 {key}")
        elif inv["status"] == "undecidable":
            out.append(f"? 불변식 {key} 판정 불가 — {inv['reason']}")
        else:
            out.append(f"✗ 불변식 {key} 위반 {len(inv['violations'])}건")
            for v in inv["violations"]:
                where = v.get("table", "")
                extra = v.get("detail") or v.get("column") or v.get("documented") or v.get("entity") or ""
                out.append(f"  - {v['kind']} {where} {extra}".rstrip())
                if "expected" in v:
                    exp = v["expected"]
                    if isinstance(exp, list):
                        exp = ", ".join(exp)
                    out.extend("      " + l for l in str(exp).splitlines())
    out.append({0: "정합", 1: "위반", 2: "판정 불가"}[report["exit"]] + f" (exit {report['exit']})")
    return "\n".join(out)


def main():
    parser = argparse.ArgumentParser(
        description="docs/data-model 의 ERD·쿼리 패턴·인덱스 지원을 실재와 대조한다 "
                    "(0 정합 · 1 위반 · 2 판정 불가). 규칙은 docs/data-model/README.md.")
    parser.add_argument("--json", action="store_true", help="기계 판독형 리포트")
    parser.add_argument("--root", default=os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    args = parser.parse_args()
    try:
        report = run(args.root)
    except Undecidable as exc:
        report = {"error": str(exc), "exit": 2}
    print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) if args.json else human(report))
    return report["exit"]


if __name__ == "__main__":
    sys.exit(main())
