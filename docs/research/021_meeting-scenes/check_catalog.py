"""021 开会场景目录的检查脚本。

查 catalog.md · coverage.md · decision.md · sources.md 四份文件之间是否对得上：

- K1 主表：catalog.md 里以 | `键` | 开头的行只能出现在一张表里，且连续（mtgdata 的 check_cards.py 靠这条读键）
- K2 键：唯一；小写英文加连字符；必须有兜底键 `other`
- K3 字段：每行列数等于表头列数，每格非空
- K4 数字有出处：除前 5 列外，凡带单位的数字（dB、%、s、m、km/h、条 ……）所在的格，
  必须带 [Sxx]、【推断】或【仓内】之一
- K5 出处编号：sources.md 的编号唯一；四份文件（含 sources.md 自己）里引用的每个 [Sxx] 都在 sources.md 里有定义
- K6 覆盖矩阵：coverage.md 矩阵的行键 = 主表的键去掉 `other`，每个地点恰好一行；每行「空格数」等于该行 ○ 的个数；
  矩阵行不许以反引号键开头
- K7 矩阵汇总数只有一个正本（矩阵本身）：coverage.md「空格总数 N 个」与 decision.md「有 N 个格子」= 全表 ○ 之和；
  coverage.md「一个 ● 都没有的因素有 N 个」与 decision.md「真正覆盖的因素有 N 个」= 没有任何 ● 的因素列数

用法：python3 docs/research/021_meeting-scenes/check_catalog.py [目录]
全过退出 0，有问题逐条打印并退出 1。
"""
import re
import sys
from pathlib import Path

KEY_RE = re.compile(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$")
MAIN_ROW_RE = re.compile(r"^\|\s*`([^`]*)`\s*\|")
SRC_REF_RE = re.compile(r"\[(S\d{2,3})\]")
SRC_DEF_RE = re.compile(r"^\|\s*(S\d{2,3})\s*\|")
TAGS = ("【推断", "【仓内")
# 带单位的数字：这些才是「要有出处的数」；数据集名里的数字（MISP2021、CHiME-5）不算
QTY_RE = re.compile(
    r"\d+(?:\.\d+)?\s*(?:%|dB|s\b|s[ /，,；)]|m\b|m[ /，,；)²³]|km/h|cm|h\b|条|段|间|人|户|座|次|年)"
)
FREE_COLS = 5  # 键 · 中文名 · 一句描述 · 对应用户层 · 对应 H 场景：不要求出处
OTHER_KEY = "other"


def split_row(line):
    """把一行 Markdown 表格拆成格（去掉首尾的竖线）。"""
    cells = [c.strip() for c in line.strip().split("|")]
    return cells[1:-1]


def is_sep(line):
    return bool(re.match(r"^\|[\s:|-]+\|\s*$", line))


def parse_main_table(text):
    """返回 (表头, [(行号, 格列表)], 问题列表)。"""
    lines = text.splitlines()
    idx = [i for i, l in enumerate(lines) if MAIN_ROW_RE.match(l)]
    problems = []
    if not idx:
        return [], [], ["K1 catalog.md 里没有以 | `键` | 开头的主表行"]
    first, last = idx[0], idx[-1]
    for i in range(first, last + 1):
        if not MAIN_ROW_RE.match(lines[i]):
            problems.append(f"K1 第 {i + 1} 行：主表中间夹了别的行，反引号键行必须连续在一张表里")
            break
    if first < 2 or not is_sep(lines[first - 1]):
        problems.append(f"K1 第 {first + 1} 行：主表行上方不是表头分隔行")
        header = []
    else:
        header = split_row(lines[first - 2])
    rows = [(i + 1, split_row(lines[i])) for i in idx]
    return header, rows, problems


def check_catalog(text):
    header, rows, problems = parse_main_table(text)
    keys = []
    for ln, cells in rows:
        key = cells[0].strip("`") if cells else ""
        keys.append(key)
        if not KEY_RE.match(key):
            problems.append(f"K2 第 {ln} 行：键 `{key}` 不是小写英文加连字符")
        if header and len(cells) != len(header):
            problems.append(f"K3 第 {ln} 行 `{key}`：{len(cells)} 格，表头是 {len(header)} 格")
        for j, c in enumerate(cells):
            col = header[j] if j < len(header) else f"第 {j + 1} 列"
            if not c:
                problems.append(f"K3 第 {ln} 行 `{key}`：「{col}」是空的")
            elif j >= FREE_COLS and QTY_RE.search(c):
                if not (SRC_REF_RE.search(c) or any(t in c for t in TAGS)):
                    problems.append(f"K4 第 {ln} 行 `{key}`：「{col}」有数字但没有出处：{c[:40]}")
    seen = set()
    for k in keys:
        if k in seen:
            problems.append(f"K2 键 `{k}` 重复")
        seen.add(k)
    if rows and OTHER_KEY not in seen:
        problems.append("K2 缺兜底键 `other`")
    return keys, problems


def check_sources(src_text, cited_texts):
    problems = []
    defs = [m.group(1) for l in src_text.splitlines() if (m := SRC_DEF_RE.match(l))]
    dup = sorted({d for d in defs if defs.count(d) > 1})
    for d in dup:
        problems.append(f"K5 sources.md 编号 {d} 重复")
    defined = set(defs)
    for name, t in cited_texts.items():
        for ref in sorted(set(SRC_REF_RE.findall(t))):
            if ref not in defined:
                problems.append(f"K5 {name} 引用了 [{ref}]，sources.md 里没有")
    return problems


def matrix_stats(cov_text):
    """从矩阵算 (○ 总数, 没有 ● 的因素列数)；找不到矩阵返回 None。"""
    lines = cov_text.splitlines()
    hdr = next((i for i, l in enumerate(lines) if l.startswith("| 地点键 |") and "空格数" in l), None)
    if hdr is None:
        return None
    header = split_row(lines[hdr])
    factor_cols = [j for j, h in enumerate(header) if j > 0 and "空格数" not in h]
    rows = []
    for l in lines[hdr + 2:]:
        if not l.startswith("|"):
            break
        rows.append(split_row(l))
    empty = sum(r[j].startswith("○") for r in rows for j in factor_cols if j < len(r))
    no_full = sum(not any(j < len(r) and r[j].startswith("●") for r in rows) for j in factor_cols)
    return empty, no_full


CLAIMS = (  # (文件, 句式, 取哪个统计量)
    ("coverage.md", r"空格总数 (\d+) 个", 0),
    ("coverage.md", r"一个 ● 都没有的因素有 (\d+) 个", 1),
    ("decision.md", r"有 (\d+) 个格子关键", 0),
    ("decision.md", r"真正覆盖的因素有 (\d+) 个", 1),
)


def check_claims(texts):
    stats = matrix_stats(texts["coverage.md"])
    if stats is None:
        return []  # 缺矩阵已由 K6 报
    problems = []
    for name, pat, k in CLAIMS:
        found = re.findall(pat, texts[name])
        if not found:
            problems.append(f"K7 {name} 找不到汇总句「{pat}」")
        for n in found:
            if int(n) != stats[k]:
                problems.append(f"K7 {name} 写 {n}，矩阵算出来是 {stats[k]}")
    return problems


def check_coverage(cov_text, keys):
    problems = []
    lines = cov_text.splitlines()
    for i, l in enumerate(lines):
        if MAIN_ROW_RE.match(l):
            problems.append(f"K6 coverage.md 第 {i + 1} 行以反引号键开头，会被当成主表")
    hdr = next((i for i, l in enumerate(lines) if l.startswith("| 地点键 |") and "空格数" in l), None)
    if hdr is None:
        return problems + ["K6 coverage.md 找不到矩阵表头（| 地点键 | … | 空格数 ○ |）"]
    header = split_row(lines[hdr])
    cnt_col = next(j for j, h in enumerate(header) if "空格数" in h)
    got = []
    for i in range(hdr + 2, len(lines)):
        if not lines[i].startswith("|"):
            break
        cells = split_row(lines[i])
        key = cells[0]
        got.append(key)
        n_empty = sum(c.startswith("○") for j, c in enumerate(cells[1:], 1) if j != cnt_col)
        try:
            claimed = int(cells[cnt_col])
        except (IndexError, ValueError):
            problems.append(f"K6 coverage.md 第 {i + 1} 行 {key}：空格数不是整数")
            continue
        if claimed != n_empty:
            problems.append(f"K6 coverage.md 第 {i + 1} 行 {key}：写了 {claimed} 个空格，实际 ○ 有 {n_empty} 个")
    for k in sorted({k for k in got if got.count(k) > 1}):
        problems.append(f"K6 coverage.md 矩阵里地点 {k} 出现了 {got.count(k)} 行")
    want = [k for k in keys if k != OTHER_KEY]
    for k in sorted(set(want) - set(got)):
        problems.append(f"K6 coverage.md 矩阵缺地点 {k}")
    for k in sorted(set(got) - set(want)):
        problems.append(f"K6 coverage.md 矩阵多出地点 {k}（主表里没有）")
    return problems


def run(folder):
    folder = Path(folder)
    texts = {n: (folder / n).read_text(encoding="utf-8")
             for n in ("catalog.md", "coverage.md", "decision.md", "sources.md")}
    keys, problems = check_catalog(texts["catalog.md"])
    problems += check_sources(texts["sources.md"], texts)
    problems += check_coverage(texts["coverage.md"], keys)
    problems += check_claims(texts)
    return keys, problems


def main(argv):
    folder = argv[1] if len(argv) > 1 else Path(__file__).resolve().parent
    keys, problems = run(folder)
    for p in problems:
        print("FAIL " + p)
    print(f"{'ok' if not problems else 'FAIL'}  主表 {len(keys)} 个键，问题 {len(problems)} 处")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
