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
CATALOG = DOC_DIR + "/query-patterns.md"
BLOCK_TAG = "data-model-scope"
SINGLE_KEYS = ("migrations", "checker")
ERE_KEYS = ("exclude", "site", "sql", "schema-exclude")
REQUIRED_KEYS = ("migrations", "checker", "scope", "site", "sql", "schema-exclude")
MIGRATION_RE = re.compile(r"^(\d+)_(.+?)(\.up|\.down)?\.sql$")
COLUMN_HEADER = ["컬럼", "타입", "NULL", "키"]
INDEX_HEADER = ["인덱스", "컬럼", "UNIQUE", "조건"]
PATTERN_HEADER = ["ID", "형태", "지원 인덱스 또는 허용 사유", "호출 지점"]
MANUAL_HEADER = ["호출 지점", "패턴 ID", "대표 SQL", "추출 불가 사유"]
PATTERN_ID_RE = re.compile(r"^Q-[0-9]+$")
WRITE_OPS = ("insert", "upsert")
PENDING = {
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
    return [c.strip().replace("\\|", "|") for c in re.split(r"(?<!\\)\|", cells[1:-1])]


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


IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
CHAR_RE = re.compile(r"'(?:\\u\{[0-9a-fA-F]+\}|\\x[0-9a-fA-F]{2}|\\.|[^\\'\n])'")
ESCAPES = {"n": "\n", "t": "\t", "r": "\r", "0": "\0", "\\": "\\", "'": "'", '"': '"'}


def unescape(body):
    out, i = [], 0
    while i < len(body):
        ch = body[i]
        if ch != "\\":
            out.append(ch)
            i += 1
            continue
        nxt = body[i + 1]
        if nxt == "\n":
            i += 2
            while i < len(body) and body[i] in " \t\r\n":
                i += 1
        elif nxt == "x":
            out.append(chr(int(body[i + 2:i + 4], 16)))
            i += 4
        elif nxt == "u":
            end = body.index("}", i)
            out.append(chr(int(body[i + 3:end], 16)))
            i = end + 1
        else:
            out.append(ESCAPES.get(nxt, nxt))
            i += 2
    return "".join(out)


def lex(text):
    toks, skip, i, n = [], [], 0, len(text)
    while i < n:
        ch = text[i]
        if ch in " \t\r\n":
            i += 1
        elif text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            skip.append((i, j))
            i = j
        elif text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            skip.append((i, j))
            i = j
        else:
            m = re.compile(r'b?r(#*)"').match(text, i)
            if m:
                close = '"' + m.group(1)
                j = text.index(close, m.end())
                toks.append(("str", text[m.end():j], i, j + len(close)))
                skip.append((i, j + len(close)))
                i = j + len(close)
                continue
            if ch == '"' or text.startswith('b"', i):
                j = i + (2 if ch == "b" else 1)
                while text[j] != '"':
                    j += 2 if text[j] == "\\" else 1
                toks.append(("str", unescape(text[i + (2 if ch == "b" else 1):j]), i, j + 1))
                skip.append((i, j + 1))
                i = j + 1
                continue
            if ch == "'":
                m = CHAR_RE.match(text, i)
                if m:
                    toks.append(("char", m.group(0), i, m.end()))
                    i = m.end()
                    continue
                m = IDENT_RE.match(text, i + 1)
                toks.append(("life", m.group(0), i, m.end()))
                i = m.end()
                continue
            m = IDENT_RE.match(text, i)
            if m:
                toks.append(("id", m.group(0), i, m.end()))
                i = m.end()
                continue
            m = re.compile(r"[0-9][0-9A-Za-z_.]*").match(text, i)
            if m:
                toks.append(("num", m.group(0), i, m.end()))
                i = m.end()
                continue
            toks.append(("p", ch, i, i + 1))
            i += 1
    return toks, skip


class Unextractable(Exception):
    pass


def matching(toks, i):
    pairs = {"(": ")", "[": "]", "{": "}"}
    opener, closer, depth = toks[i][1], pairs[toks[i][1]], 0
    for j in range(i, len(toks)):
        if toks[j][0] == "p" and toks[j][1] == opener:
            depth += 1
        elif toks[j][0] == "p" and toks[j][1] == closer:
            depth -= 1
            if depth == 0:
                return j
    raise Unextractable("Rust 괄호가 닫히지 않는다")


def is_p(tok, ch):
    return tok[0] == "p" and tok[1] == ch


def frames(toks):
    out, stack, pending, nest = [], [], None, 0
    for i, tok in enumerate(toks):
        if tok[0] == "p" and tok[1] in "([":
            nest += 1
        elif tok[0] == "p" and tok[1] in ")]":
            nest -= 1
        if tok[0] == "id" and tok[1] in ("fn", "mod") and i + 1 < len(toks) and toks[i + 1][0] == "id":
            test = tok[1] == "mod" and is_cfg_test(toks, i)
            pending = (tok[1], toks[i + 1][1], test)
        elif tok[0] == "id" and tok[1] == "impl" and (i == 0 or not is_p(toks[i - 1], "<")):
            pending = ("impl", impl_type(toks, i), False)
        elif is_p(tok, ";") and nest == 0 and pending and pending[0] != "impl":
            pending = None
        elif is_p(tok, "{"):
            stack.append((pending or ("block", None, False), tok[2]))
            pending = None
        elif is_p(tok, "}"):
            frame, start = stack.pop()
            if frame[0] != "block":
                out.append((start, tok[3], frame))
    return out


def is_cfg_test(toks, i):
    j = i - 1
    while j >= 0 and toks[j][0] == "id" and toks[j][1] in ("pub",):
        j -= 1
    if j < 0 or not is_p(toks[j], "]"):
        return False
    k = j
    while k >= 0 and not is_p(toks[k], "#"):
        k -= 1
    return [t[1] for t in toks[k:j + 1]] == ["#", "[", "cfg", "(", "test", ")", "]"]


def impl_type(toks, i):
    j, depth, names, after_for = i + 1, 0, [], None
    while j < len(toks) and not (depth == 0 and is_p(toks[j], "{")):
        tok = toks[j]
        if is_p(tok, "<"):
            depth += 1
        elif is_p(tok, ">"):
            depth -= 1
        elif depth == 0 and tok[0] == "id":
            if tok[1] == "for":
                after_for = len(names)
            elif tok[1] != "where":
                names.append(tok[1])
            else:
                break
        j += 1
    if after_for is not None and after_for < len(names):
        return names[after_for]
    return names[-1] if names else "?"


def site_name(rel, offset, spans):
    fn, impl, test = None, None, False
    for start, end, frame in spans:
        if start <= offset < end:
            kind, name, is_test = frame
            if kind == "mod" and is_test:
                test = True
            elif kind == "impl":
                impl = (start, name)
            elif kind == "fn":
                if fn is None or start > fn[0]:
                    fn = (start, name)
    if fn is None:
        return None, test
    parts = [rel]
    if impl and impl[0] < fn[0]:
        parts.append(impl[1])
    parts.append(fn[1])
    return "::".join(parts), test


def split_args(toks, open_i):
    close = matching(toks, open_i)
    args, cur, j = [], [], open_i + 1
    while j < close:
        tok = toks[j]
        if tok[0] == "p" and tok[1] in "([{":
            end = matching(toks, j)
            cur.extend(toks[j:end + 1])
            j = end + 1
            continue
        if is_p(tok, ","):
            args.append(cur)
            cur = []
        else:
            cur.append(tok)
        j += 1
    if cur:
        args.append(cur)
    return args, close


def find_const(toks, name):
    for i in range(len(toks) - 6):
        if toks[i][1] == "const" and toks[i + 1][1] == name and is_p(toks[i + 2], ":"):
            j = i + 3
            while j < len(toks) and not is_p(toks[j], "="):
                j += 1
            if j + 2 < len(toks) and toks[j + 1][0] == "str" and is_p(toks[j + 2], ";"):
                return toks[j + 1][1]
    return None


def find_format_fn(toks, name):
    for i in range(len(toks) - 4):
        if toks[i][1] == "fn" and toks[i + 1][1] == name and is_p(toks[i + 2], "(") and is_p(toks[i + 3], ")"):
            j = i + 4
            while j < len(toks) and not is_p(toks[j], "{"):
                j += 1
            body_end = matching(toks, j)
            body = toks[j + 1:body_end]
            if len(body) >= 4 and [t[1] for t in body[:3]] == ["format", "!", "("] and matching(body, 2) == len(body) - 1:
                return body
    return None


PLACEHOLDER_RE = re.compile(r"\{\{|\}\}|\{([A-Za-z_][A-Za-z0-9_]*|[0-9]+)?(?::[^}]*)?\}")


def resolve_format(toks, fmt, depth=0):
    args, _ = split_args(fmt, 2)
    if not args or len(args[0]) != 1 or args[0][0][0] != "str":
        raise Unextractable("format! 의 첫 인자가 문자열 리터럴이 아니다")
    template, positional, named = args[0][0][1], [], {}
    for arg in args[1:]:
        if len(arg) >= 2 and arg[0][0] == "id" and is_p(arg[1], "=") and not (len(arg) > 2 and is_p(arg[2], "=")):
            named[arg[0][1]] = arg[2:]
        else:
            positional.append(arg)
    out, last, seq = [], 0, 0
    for m in PLACEHOLDER_RE.finditer(template):
        out.append(template[last:m.start()])
        last = m.end()
        if m.group(0) in ("{{", "}}"):
            out.append(m.group(0)[0])
            continue
        key = m.group(1)
        if key is None:
            expr, label = positional[seq] if seq < len(positional) else None, f"{{{seq}}}"
            seq += 1
        elif key.isdigit():
            expr, label = positional[int(key)] if int(key) < len(positional) else None, m.group(0)
        else:
            expr, label = named.get(key, [("id", key, 0, 0)]), m.group(0)
        text = "".join(out)
        if text.count("'") % 2 == 1:
            out.append("x")
            continue
        if expr and len(expr) == 1 and expr[0][0] == "id":
            value = find_const(toks, expr[0][1])
            if value is not None:
                out.append(value)
                continue
        if expr and len(expr) == 3 and expr[0][0] == "id" and is_p(expr[1], "(") and is_p(expr[2], ")") and depth < 3:
            body = find_format_fn(toks, expr[0][1])
            if body is not None:
                out.append(resolve_format(toks, body, depth + 1))
                continue
        shown = " ".join(t[1] for t in expr) if expr else label
        raise Unextractable(f"format! 보간 {label} = `{shown}` 를 정적으로 풀 수 없다")
    out.append(template[last:])
    return "".join(out)


def call_sql(toks, i):
    j = i + 4
    if is_p(toks[j], ":") and is_p(toks[j + 1], ":") and is_p(toks[j + 2], "<"):
        depth, j = 0, j + 2
        while True:
            depth += is_p(toks[j], "<") - is_p(toks[j], ">")
            j += 1
            if depth == 0:
                break
    if is_p(toks[j], "!"):
        j += 1
    if not is_p(toks[j], "("):
        raise Unextractable("호출 괄호를 찾지 못했다")
    args, _ = split_args(toks, j)
    if not args:
        raise Unextractable("인자가 없다")
    arg = args[0]
    while arg and is_p(arg[0], "&"):
        arg = arg[1:]
    if len(arg) == 1 and arg[0][0] == "str":
        return arg[0][1]
    if len(arg) == 1 and arg[0][0] == "id":
        value = find_const(toks, arg[0][1])
        if value is None:
            raise Unextractable(f"`{arg[0][1]}` 가 같은 파일의 const SQL 이 아니다")
        return value
    if len(arg) >= 3 and [t[1] for t in arg[:3]] == ["format", "!", "("]:
        return resolve_format(toks, arg)
    raise Unextractable("인자가 SQL 리터럴·const·format! 이 아니다: " + " ".join(t[1] for t in arg))


def sites(rel, text, site_re):
    toks, skip = lex(text)
    spans = frames(toks)
    by_start = {t[2]: k for k, t in enumerate(toks)}
    line_starts = [0] + [m.end() for m in re.finditer("\n", text)]
    out = []
    for lineno, start in enumerate(line_starts, 1):
        end = text.find("\n", start)
        line = text[start:] if end < 0 else text[start:end]
        for m in site_re.finditer(line):
            off = start + m.start()
            where = f"{rel}:{lineno}"
            if any(a <= off < b for a, b in skip):
                out.append({"line": where, "ignored": "주석·문자열 안의 텍스트다"})
                continue
            name, test = site_name(rel, off, spans)
            if test:
                out.append({"line": where, "ignored": "#[cfg(test)] 모듈"})
                continue
            if name is None:
                out.append({"line": where, "site": None, "error": "둘러싼 함수가 없다"})
                continue
            k = by_start.get(off)
            if k is None:
                out.append({"line": where, "site": name, "error": "토큰 경계가 아니다"})
                continue
            try:
                out.append({"line": where, "site": name, "sql": call_sql(toks, k)})
            except Unextractable as exc:
                out.append({"line": where, "site": name, "error": str(exc)})
    return out


SQL_TOKEN_RE = re.compile(
    r"\s+|'(?:[^']|'')*'|\"[^\"]*\"|`[^`]*`|[A-Za-z_][A-Za-z0-9_]*|[0-9]+(?:\.[0-9]+)?"
    r"|<=|>=|<>|!=|==|\|\||[-+*/%=<>(),.?;]")
KEYWORDS = {
    "select", "from", "where", "join", "inner", "left", "right", "full", "outer", "cross", "on",
    "using", "group", "by", "having", "order", "limit", "offset", "union", "all", "intersect",
    "except", "insert", "into", "values", "update", "set", "delete", "replace", "or", "and",
    "not", "in", "is", "null", "like", "between", "exists", "as", "asc", "desc", "distinct",
    "conflict", "do", "nothing", "returning", "ignore", "abort", "fail", "rollback", "case",
    "when", "then", "else", "end", "collate", "nocase", "with", "default", "true", "false",
}
CLAUSE_END = {"where", "group", "having", "order", "limit", "offset", "returning", "on",
              "join", "inner", "left", "right", "full", "cross", "union", "intersect", "except",
              "set", "values"}


class ShapeError(Exception):
    pass


def tokenize(sql):
    pos, out = 0, []
    while pos < len(sql):
        m = SQL_TOKEN_RE.match(sql, pos)
        if not m:
            raise ShapeError(f"SQL 토큰을 읽을 수 없다: {sql[pos:pos + 20]!r}")
        pos = m.end()
        text = m.group(0)
        if text.isspace():
            continue
        if text[0] in "\"`":
            out.append(("id", text[1:-1].lower()))
        elif text[0] == "'":
            out.append(("lit", text))
        elif re.match(r"[A-Za-z_]", text):
            low = text.lower()
            out.append(("kw" if low in KEYWORDS else "id", low))
        elif text[0].isdigit():
            out.append(("lit", text))
        elif text == "?":
            out.append(("param", "?"))
        else:
            out.append(("op", text))
    return out


def tree(tokens):
    stack = [[]]
    for tok in tokens:
        if tok == ("op", "("):
            stack.append([])
        elif tok == ("op", ")"):
            if len(stack) == 1:
                raise ShapeError("괄호가 맞지 않는다")
            group = stack.pop()
            stack[-1].append(("group", group))
        elif tok == ("op", ";"):
            continue
        else:
            stack[-1].append(tok)
    if len(stack) != 1:
        raise ShapeError("괄호가 닫히지 않는다")
    return stack[0]


def is_kw(node, *words):
    return node[0] == "kw" and node[1] in words


def is_query(group):
    return bool(group) and is_kw(group[0], "select", "with")


def split_top(nodes, pred):
    parts, cur = [], []
    for node in nodes:
        if pred(node):
            parts.append(cur)
            cur = []
        else:
            cur.append(node)
    parts.append(cur)
    return parts


class Scope:
    def __init__(self, parent, schema):
        self.parent, self.schema, self.aliases, self.tables = parent, schema, {}, []

    def add(self, table, alias):
        if table not in self.schema:
            raise ShapeError(f"스키마에 없는 테이블 `{table}`")
        self.tables.append(table)
        self.aliases[alias or table] = table
        self.aliases.setdefault(table, table)

    def resolve(self, qual, col):
        scope = self
        while scope:
            if qual:
                if qual in scope.aliases:
                    return scope.aliases[qual], col
            else:
                owners = sorted({t for t in scope.tables
                                 if col == "rowid" or col in self.schema[t]})
                if len(owners) == 1:
                    return owners[0], col
                if len(owners) > 1:
                    raise ShapeError(f"컬럼 `{col}` 의 테이블이 모호하다")
            scope = scope.parent
        raise ShapeError(f"컬럼 `{qual + '.' if qual else ''}{col}` 를 해석할 수 없다")


class Shape:
    def __init__(self, schema):
        self.schema, self.op, self.tables = schema, None, set()
        self.eq, self.range, self.order, self.joins = [], [], [], []

    def render(self):
        multi = len(self.tables) > 1

        def name(ref):
            if isinstance(ref, tuple):
                return f"{ref[0]}.{ref[1]}" if multi else ref[1]
            return ref

        def cell(kind, items):
            seen = []
            for item in items:
                if item not in seen:
                    seen.append(item)
            return f"{kind}({', '.join(seen)})" if seen else "-"

        eq = sorted({name(r) for r in self.eq} | {
            "=".join(sorted((name(a), name(b)))) for a, b in self.joins})
        rng = sorted({name(r) for r in self.range})
        order = [f"{name(r)} {d}" for r, d in self.order]
        return " | ".join([f"{self.op} {','.join(sorted(self.tables))}",
                           cell("eq", eq), cell("range", rng), cell("order", order)])


def column_ref(nodes, scope):
    if len(nodes) == 1 and nodes[0][0] == "id":
        return scope.resolve(None, nodes[0][1])
    if len(nodes) == 3 and nodes[0][0] == "id" and nodes[1] == ("op", ".") and nodes[2][0] in ("id", "kw"):
        return scope.resolve(nodes[0][1], nodes[2][1])
    return None


def expr_text(nodes, scope, shape):
    out, i = [], 0
    while i < len(nodes):
        node = nodes[i]
        if node[0] == "group":
            if is_query(node[1]):
                raise ShapeError("식 안의 서브쿼리는 형태 텍스트로 쓸 수 없다")
            out.append("(" + expr_text(node[1], scope, shape) + ")")
            i += 1
            continue
        if node[0] == "id" and i + 2 < len(nodes) and nodes[i + 1] == ("op", ".") \
                and nodes[i + 2][0] in ("id", "kw"):
            ref = scope.resolve(node[1], nodes[i + 2][1])
            out.append(f"{ref[0]}.{ref[1]}" if len(shape.tables) > 1 else ref[1])
            i += 3
            continue
        if node[0] == "id" and not (i + 1 < len(nodes) and nodes[i + 1][0] == "group"):
            ref = scope.resolve(None, node[1])
            out.append(f"{ref[0]}.{ref[1]}" if len(shape.tables) > 1 else ref[1])
            i += 1
            continue
        out.append(node[1])
        i += 1
    text = " ".join(out)
    text = re.sub(r" ?([(),.]) ?", lambda m: m.group(1) + (" " if m.group(1) == "," else ""), text)
    text = text.replace("( ", "(")
    return re.sub(r"\s+", " ", text).strip()


def operand(nodes, scope, shape):
    if len(nodes) == 1 and nodes[0][0] == "group" and is_query(nodes[0][1]):
        return None
    ref = column_ref(nodes, scope)
    if ref:
        return ref
    if all(n[0] in ("param", "lit") or is_kw(n, "null", "true", "false") for n in nodes):
        return None
    if any(n[0] in ("id",) for n in nodes) or any(n[0] == "group" for n in nodes):
        return ("expr", expr_text(nodes, scope, shape))
    return None


def conjuncts(nodes):
    parts, cur, between = [], [], False
    for node in nodes:
        if is_kw(node, "between"):
            between = True
        if is_kw(node, "and"):
            if between:
                between = False
            else:
                parts.append(cur)
                cur = []
                continue
        cur.append(node)
    parts.append(cur)
    return [p for p in parts if p]


def collect_subqueries(nodes, scope, shape):
    for node in nodes:
        if node[0] == "group":
            if is_query(node[1]):
                query(node[1], scope, shape)
            else:
                collect_subqueries(node[1], scope, shape)


def condition(nodes, scope, shape):
    collect_subqueries(nodes, scope, shape)
    if any(is_kw(n, "or") for n in nodes):
        return
    if len(nodes) == 1 and nodes[0][0] == "group" and not is_query(nodes[0][1]):
        for part in conjuncts(nodes[0][1]):
            condition(part, scope, shape)
        return
    if any(is_kw(n, "not") for n in nodes) or any(n in (("op", "!="), ("op", "<>")) for n in nodes):
        return
    if is_kw(nodes[0], "exists"):
        return
    for k, node in enumerate(nodes):
        if node[0] == "op" and node[1] in ("=", "==", "<", "<=", ">", ">=") or is_kw(node, "is", "in", "like", "between"):
            left, right, op = nodes[:k], nodes[k + 1:], node[1]
            break
    else:
        return
    while right and is_kw(right[-1], "nocase", "collate"):
        right = right[:-1]
    if len(right) >= 2 and is_kw(right[-2], "collate"):
        right = right[:-2]
    if op == "between":
        right = conjuncts(right)[0] if right else right
    if len(left) >= 2 and is_kw(left[-2], "collate"):
        left = left[:-2]
    lhs, rhs = operand(left, scope, shape), (operand(right, scope, shape) if op not in ("in",) else None)
    if op in ("in", "is") and right and right[0][0] == "group" and not is_query(right[0][1]):
        rhs = None
    if lhs is None and rhs is not None and op in ("=", "==", "<", "<=", ">", ">="):
        lhs, rhs = rhs, None
        op = {"<": ">", ">": "<", "<=": ">=", ">=": "<="}.get(op, op)
    if lhs is None:
        return
    target = lhs if not (isinstance(lhs, tuple) and lhs[0] == "expr") else lhs[1]
    if op in ("=", "==") and rhs is not None and not (isinstance(rhs, tuple) and rhs[0] == "expr"):
        shape.joins.append((target, rhs))
        return
    if op in ("=", "==", "is", "in"):
        shape.eq.append(target)
    else:
        shape.range.append(target)


def from_items(nodes, scope):
    items = split_top(nodes, lambda n: n == ("op", ",") or is_kw(n, "join"))
    ons = []
    for item in items:
        while item and is_kw(item[-1], "inner", "left", "right", "full", "outer", "cross"):
            item = item[:-1]
        if not item:
            continue
        on = None
        for k, node in enumerate(item):
            if is_kw(node, "on"):
                item, on = item[:k], item[k + 1:]
                break
        if item[0][0] == "group":
            raise ShapeError("FROM 절의 파생 테이블(서브쿼리)은 바깥 쿼리와 한 형태로 합칠 수 없다")
        if item[0][0] != "id":
            raise ShapeError(f"FROM 항목을 읽을 수 없다: {item}")
        alias = item[-1][1] if len(item) >= 2 and item[-1][0] == "id" else None
        scope.add(item[0][1], alias)
        if on:
            ons.append(on)
    return ons


def clauses(nodes, words):
    out, cur, key = {}, [], None
    i = 0
    while i < len(nodes):
        node = nodes[i]
        word = None
        if node[0] == "kw":
            if node[1] in ("group", "order") and i + 1 < len(nodes) and is_kw(nodes[i + 1], "by"):
                word = node[1] + " by"
            elif node[1] in words:
                word = node[1]
        if word:
            out[key] = cur
            key, cur = word, []
            i += 2 if word.endswith(" by") else 1
            continue
        cur.append(node)
        i += 1
    out[key] = cur
    return out


def order_items(nodes, scope, shape):
    for item in split_top(nodes, lambda n: n == ("op", ",")):
        direction = "asc"
        if item and is_kw(item[-1], "asc", "desc"):
            direction, item = item[-1][1], item[:-1]
        if len(item) >= 2 and is_kw(item[-2], "collate"):
            item = item[:-2]
        try:
            ref = column_ref(item, scope) or expr_text(item, scope, shape)
        except ShapeError:
            ref = " ".join(n[1] for n in item if n[0] != "group")
        shape.order.append((ref, direction))


def select_core(nodes, scope, shape):
    parts = clauses(nodes, {"from", "where", "having", "limit", "offset"})
    ons = from_items(parts.get("from", []), scope)
    shape.tables.update(scope.tables)
    collect_subqueries(parts.get(None, []), scope, shape)
    for on in ons:
        for part in conjuncts(on):
            condition(part, scope, shape)
    for part in conjuncts(parts.get("where", [])):
        condition(part, scope, shape)
    for key in ("group by", "having"):
        collect_subqueries(parts.get(key, []), scope, shape)
    return parts


def query(nodes, parent, shape):
    if is_kw(nodes[0], "with"):
        raise ShapeError("CTE(WITH)는 아직 합치지 않는다")
    arms = split_top(nodes, lambda n: is_kw(n, "union", "intersect", "except"))
    last = None
    for arm in arms:
        if arm and is_kw(arm[0], "all"):
            arm = arm[1:]
        scope = Scope(parent, shape.schema)
        last = (select_core(arm, scope, shape), scope)
    parts, scope = last
    if "order by" in parts:
        order_items(parts["order by"], scope, shape)


def statement_shape(sql, schema):
    nodes = tree(tokenize(sql))
    shape = Shape(schema)
    head = nodes[0] if nodes else ("", "")
    if is_kw(head, "select", "with"):
        shape.op = "select"
        query(nodes, None, shape)
    elif is_kw(head, "insert", "replace"):
        scope = Scope(None, schema)
        k = next(i for i, n in enumerate(nodes) if is_kw(n, "into"))
        upsert = is_kw(head, "replace") or (len(nodes) > 2 and is_kw(nodes[1], "or") and is_kw(nodes[2], "replace"))
        scope.add(nodes[k + 1][1], None)
        shape.tables.update(scope.tables)
        conflict = [i for i, n in enumerate(nodes) if is_kw(n, "conflict") and is_kw(nodes[i - 1], "on")]
        if conflict:
            upsert = True
            target = nodes[conflict[0] + 1]
            if target[0] == "group":
                for item in split_top(target[1], lambda n: n == ("op", ",")):
                    shape.eq.append(column_ref(item, scope))
        rest = nodes[k + 2:conflict[0] - 1] if conflict else nodes[k + 2:]
        collect_subqueries(rest, scope, shape)
        shape.op = "upsert" if upsert else "insert"
    elif is_kw(head, "update"):
        shape.op = "update"
        scope = Scope(None, schema)
        k = 1
        if is_kw(nodes[1], "or"):
            k = 3
        scope.add(nodes[k][1], nodes[k + 1][1] if nodes[k + 1][0] == "id" else None)
        shape.tables.update(scope.tables)
        parts = clauses(nodes[k + 1:], {"set", "where", "returning", "from"})
        collect_subqueries(parts.get("set", []), scope, shape)
        for part in conjuncts(parts.get("where", [])):
            condition(part, scope, shape)
    elif is_kw(head, "delete"):
        shape.op = "delete"
        scope = Scope(None, schema)
        scope.add(nodes[2][1], nodes[3][1] if len(nodes) > 3 and nodes[3][0] == "id" else None)
        shape.tables.update(scope.tables)
        parts = clauses(nodes[3:], {"where", "returning"})
        for part in conjuncts(parts.get("where", [])):
            condition(part, scope, shape)
    else:
        raise ShapeError("SELECT·INSERT·UPDATE·DELETE·REPLACE 로 시작하지 않는다")
    return shape.op, shape.render()


def extract_sites(root, block, schema):
    try:
        listed = subprocess.run(["git", "ls-files", "--", *block["scope"]], cwd=root,
                                capture_output=True, text=True, check=True).stdout.split("\n")
    except (OSError, subprocess.CalledProcessError) as exc:
        raise Undecidable(f"git ls-files 실패: {exc}")
    columns = {t: {c[0] for c in v["columns"]} for t, v in schema.items()}
    out = []
    for rel in sorted(f for f in listed if f):
        if block["exclude"] and block["exclude"].search(rel):
            continue
        try:
            with open(os.path.join(root, rel), encoding="utf-8") as fh:
                text = fh.read()
        except (OSError, UnicodeDecodeError):
            continue
        if not block["site"].search(text):
            continue
        try:
            found = sites(rel, text, block["site"])
        except (Unextractable, ValueError, IndexError) as exc:
            raise Undecidable(f"{rel} 를 Rust 로 읽을 수 없다: {exc}")
        for cand in found:
            if "sql" in cand:
                try:
                    cand["op"], cand["shape"] = statement_shape(cand["sql"], columns)
                except (ShapeError, StopIteration, IndexError) as exc:
                    cand["reason"] = str(exc) or "SQL 을 형태로 읽을 수 없다"
            elif "error" in cand:
                cand["reason"] = cand.pop("error")
            if cand.get("site") is None and "ignored" not in cand:
                cand["site"] = cand["line"]
            out.append(cand)
    return out


def parse_catalog(text):
    lines = text.splitlines()
    patterns = parse_tables_after(lines, 0, PATTERN_HEADER)
    manual = parse_tables_after(lines, 0, MANUAL_HEADER)
    if patterns is None or manual is None:
        raise Undecidable(f"{CATALOG} 에 패턴 표나 수동 형태 표가 없다")
    return patterns, manual


def md_cell(text):
    return text.replace("|", "\\|")


def support_hint(op):
    return "—" if op in WRITE_OPS else "판정 전"


def check_catalog(root, candidates, db):
    patterns, manual = parse_catalog(read(root, CATALOG))
    out, rows, by_shape = [], {}, {}
    for cells in patterns:
        if len(cells) != 4:
            out.append({"kind": "unreadable", "detail": f"패턴 표 행의 칸 수가 4가 아니다: {cells}"})
            continue
        pid, shape = unquote(cells[0]), unquote(cells[1])
        listed = [unquote(c.strip()) for c in cells[3].split(",") if c.strip()]
        if not PATTERN_ID_RE.match(pid) or pid in rows:
            out.append({"kind": "unreadable", "detail": f"패턴 ID 가 `Q-<n>` 이 아니거나 겹친다: {pid}"})
            continue
        rows[pid] = (shape, listed)
        by_shape.setdefault(shape, []).append(pid)
    for shape, pids in sorted(by_shape.items()):
        if len(pids) > 1:
            out.append({"kind": "ambiguous-shape", "detail": f"`{shape}` 를 {', '.join(pids)} 가 함께 가진다"})
    unextractable = {c["site"]: c for c in candidates if "reason" in c}
    code_pairs = {(c["site"], c["shape"]) for c in candidates if "shape" in c}
    manual_pairs, manual_sites = set(), set()
    for cells in manual:
        if len(cells) != 4:
            out.append({"kind": "unreadable", "detail": f"수동 형태 표 행의 칸 수가 4가 아니다: {cells}"})
            continue
        site, pid, sql, reason = unquote(cells[0]), unquote(cells[1]), unquote(cells[2]), cells[3]
        manual_sites.add(site)
        if pid not in rows:
            out.append({"kind": "manual-unknown-pattern", "site": site, "detail": f"패턴 {pid} 가 패턴 표에 없다"})
        else:
            manual_pairs.add((site, rows[pid][0]))
        if reason in ("", "-"):
            out.append({"kind": "unreadable", "site": site, "detail": "추출 불가 사유가 비었다"})
        try:
            params = sum(1 for t in tokenize(sql) if t[0] == "param")
            db.execute("EXPLAIN " + sql, [None] * params)
        except (sqlite3.Error, ShapeError) as exc:
            out.append({"kind": "manual-sql-invalid", "site": site, "detail": f"대표 SQL 이 빈 DB 에서 준비되지 않는다: {exc}"})
    for site in sorted(set(unextractable) - manual_sites):
        cand = unextractable[site]
        sql = norm_space(cand.get("sql", "<대표 SQL>"))
        out.append({"kind": "missing-manual", "site": site, "detail": cand["reason"],
                    "expected": f"| `{site}` | Q-? | `{md_cell(sql)}` | {md_cell(cand['reason'])} |"})
    for site in sorted(manual_sites - set(unextractable)):
        out.append({"kind": "extra-manual", "site": site,
                    "detail": "체커가 이 지점을 추출 불가로 보고하지 않는다 — 수동 형태 행을 지우고 추출된 형태로 귀속한다"})
    live = code_pairs | {p for p in manual_pairs if p[0] in unextractable}
    documented = {(site, shape) for shape, listed in rows.values() for site in listed}
    missing = {}
    for site, shape in sorted(live - documented):
        missing.setdefault(shape, []).append(site)
    ops = {c["shape"]: c["op"] for c in candidates if "shape" in c}
    for shape, listed in sorted(missing.items()):
        sites_text = ", ".join(f"`{x}`" for x in listed)
        if shape in by_shape:
            out.append({"kind": "unregistered-site", "pattern": by_shape[shape][0],
                        "detail": f"호출 지점에 {sites_text} 를 더한다"})
        else:
            out.append({"kind": "unregistered-pattern", "detail": f"`{shape}`",
                        "expected": f"| Q-? | `{md_cell(shape)}` | {support_hint(ops.get(shape))} | {sites_text} |"})
    for pid, (shape, listed) in sorted(rows.items(), key=lambda kv: int(kv[0][2:])):
        dead = [x for x in listed if (x, shape) not in live]
        for site in dead:
            out.append({"kind": "dead-site", "pattern": pid, "site": site,
                        "detail": f"`{site}` 에 `{shape}` 형태의 쿼리가 없다"})
        if listed and len(dead) == len(listed) or not listed:
            out.append({"kind": "dead-pattern", "pattern": pid, "detail": f"`{shape}` 의 살아 있는 지점이 없다"})
    return out, len(rows), len(manual_sites)


def run(root):
    block = parse_block(read(root, README))
    files = migration_files(root, block["migrations"])
    db = apply_migrations(root, files)
    schema = observe_schema(db, block["schema-exclude"])
    candidates = extract_sites(root, block, schema)
    report = {
        "engine": f"sqlite {sqlite3.sqlite_version} (python sqlite3)",
        "migrations": len(files),
        "counts": {
            "tables": len(schema),
            "columns": sum(len(t["columns"]) for t in schema.values()),
            "fks": sum(len(t["fks"]) for t in schema.values()),
            "indexes": sum(len(t["indexes"]) for t in schema.values()),
            "site_candidates": len(candidates),
            "sites_extracted": sum(1 for c in candidates if "shape" in c),
            "sites_unextractable": sum(1 for c in candidates if "reason" in c),
            "sites_excluded": sum(1 for c in candidates if "ignored" in c),
        },
        "sites": candidates,
        "invariants": {},
    }
    erd = check_erd(root, schema)
    report["invariants"]["1"] = {"status": "violations" if erd else "ok", "violations": erd}
    try:
        catalog, patterns, manual = check_catalog(root, candidates, db)
        report["counts"]["patterns"], report["counts"]["manual_shapes"] = patterns, manual
        report["invariants"]["2"] = {"status": "violations" if catalog else "ok", "violations": catalog}
    except Undecidable as exc:
        report["invariants"]["2"] = {"status": "undecidable", "reason": str(exc)}
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
               f" · site 후보 {c['site_candidates']}줄(추출 {c['sites_extracted']} · 추출 불가 {c['sites_unextractable']}"
               f" · 제외 {c['sites_excluded']}) · 패턴 {c.get('patterns', '?')} · 수동 형태 {c.get('manual_shapes', '?')}")
    for key in sorted(report["invariants"]):
        inv = report["invariants"][key]
        if inv["status"] == "ok":
            out.append(f"✓ 불변식 {key}")
        elif inv["status"] == "undecidable":
            out.append(f"? 불변식 {key} 판정 불가 — {inv['reason']}")
        else:
            out.append(f"✗ 불변식 {key} 위반 {len(inv['violations'])}건")
            for v in inv["violations"]:
                where = v.get("table") or v.get("pattern") or v.get("site") or ""
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
