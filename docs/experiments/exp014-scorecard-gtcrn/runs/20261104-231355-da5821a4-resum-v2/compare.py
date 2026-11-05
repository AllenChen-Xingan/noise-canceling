"""exp014 旧规则（v1）与新规则（v2，人定 D85）对照 + X3 分组粒度重估。

用法（仓库根目录）：
    python3 -I docs/experiments/exp014-scorecard-gtcrn/runs/20261104-231355-da5821a4-resum-v2/compare.py > <本目录>/compare.md
输入：旧 run 的 scorecard.json（v1，exp014 原表）与本目录的 scorecard.json（v2，`vdn-eval scorecard --from-clips` 重新汇总）。
只读这两份文件，不重跑音频。
"""

import json
import statistics
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
OLD = HERE.parent / "20261104-000444-e4511abc" / "scorecard.json"
NEW = HERE / "scorecard.json"
SYSTEMS = ["passthrough", "gtcrn", "ref"]
ZH = {"pass": "过", "fail": "不过", "unknown": "待测"}
SHORT = {
    "逐段：物理硬事实": "逐段",
    "逐段：比原声多错": "≥5字",
    "均值 + 95% 区间": "区间",
    "只报不判（人定": "只报",
    "只报不判：旧口径": "只报·旧口径",
}


def short(rule):
    for k, v in SHORT.items():
        if rule.startswith(k):
            return v
    return "逐段"


def load(p):
    return json.loads(p.read_text())["scenes"]


def counts(scenes):
    """(系统, 项) → Counter(状态)；听感记作「听感」，场景记作「（场景）」。"""
    c = defaultdict(Counter)
    rule = {}
    for s in scenes:
        for i in s["items"]:
            c[(s["system"], i["v"])][i["status"]] += 1
            rule[i["v"]] = short(i.get("rule", ""))
        c[(s["system"], "听感")][s["listen"]["status"] or "unknown"] += 1
        c[(s["system"], "（场景）")][s["status"]] += 1
    return c, rule


def fmt(cnt):
    return " / ".join(str(cnt.get(k, 0)) for k in ("pass", "fail", "unknown"))


def main():
    old, new = load(OLD), load(NEW)
    co, _ = counts(old)
    cn, rule = counts(new)
    items = []
    for s in new:
        for i in s["items"]:
            if i["v"] not in items:
                items.append(i["v"])
    items += ["听感", "（场景）"]
    out = sys.stdout
    out.write("# exp014 新旧规则对照（只重新汇总，未重跑音频）\n\n")
    out.write("每格是 60 个组合（场景 × 档 × 设备）里 **过 / 不过 / 待测** 的个数。旧 = 原表（任一段不过即不过）；"
              "新 = 人定 D85 的规则（判法列见下）。生成：本目录 `compare.py`。\n\n")
    out.write("判法：逐段 = 物理硬事实，任一段不过即否决 · ≥5字 = 单段多错 ≥ 5 字否决 · 区间 = 均值 95% 区间三态 + Holm · "
              "只报 = 只报不判 · 只报·旧口径 = 该在干净人声单独过一遍上算，旧 run 没有，待重跑\n\n")
    head = "| 项 | 判法 | " + " | ".join(f"{s} 旧 | {s} 新" for s in SYSTEMS) + " |\n"
    out.write(head + "|---|---|" + "---|---|" * len(SYSTEMS) + "\n")
    for v in items:
        cells = []
        for s in SYSTEMS:
            cells += [fmt(co.get((s, v), Counter())), fmt(cn.get((s, v), Counter()))]
        r = rule.get(v, "区间" if v == "听感" else "")
        out.write(f"| {v} | {r} | " + " | ".join(cells) + " |\n")
    # X3 重估：新规则下判定项落「待测」的比例、区间半宽与离线距离
    out.write("\n## 分组粒度重估（025 X3，只报，不改判定）\n\n")
    out.write("对每个按区间判的项：组合层落「待测」的个数，区间半宽中位数，均值离过线距离中位数。"
              "半宽 > 离线距离的组合多半落待测，合并到「场景 × 档」或场景组可能让它分得出（推断，未重算）。\n\n")
    out.write("| 项 | 系统 | 待测 / 有区间 | 半宽中位数 | 离线距离中位数 |\n|---|---|---|---|---|\n")
    per = defaultdict(list)
    for s in new:
        for i in s["items"]:
            ci, thr = i.get("ci"), i.get("threshold")
            if ci and thr is not None and short(i.get("rule", "")) == "区间":
                per[(i["v"], s["system"])].append(
                    (i["status"], (ci["hi"] - ci["lo"]) / 2, abs(ci["mean"] - thr)))
    for (v, s), xs in sorted(per.items()):
        unk = sum(1 for x in xs if x[0] == "unknown")
        if unk == 0:
            continue
        hw = statistics.median(x[1] for x in xs)
        dist = statistics.median(x[2] for x in xs)
        out.write(f"| {v} | {s} | {unk} / {len(xs)} | {hw:.3f} | {dist:.3f} |\n")


if __name__ == "__main__":
    main()
