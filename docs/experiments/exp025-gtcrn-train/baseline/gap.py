"""exp025 · 场景差距表：哪几个开会场景最弱，用来核对临时选定的加训场景（计划 014）。

只读 `vdn-eval scorecard` 写出的 `scorecard.json`（每个 场景 × 档 × 设备 × 系统 一张卡，每项的均值、线、判定都是
Rust 算好的），不重算任何评分项，所以这里不是 verifier 的第二处表示。

算法（阈值的正本是本文件的常量）：
1. 只看一个系统（`--system`），只看「判定项」：卡里 `report_only` 为假、有均值的项，外加两个听感评分模型
   （`listen.models`）。逐段判的物理项（坏值、延迟）也算，它们的均值照样能排
2. **不过数**：每个场景，所有 档 × 设备 切片里判「不过」（`fail`）的项数之和
3. **弱点分**：每一项、每个 档 × 设备 切片，把有值的场景按均值从好到差排名，换算成 0–1（最差 = 1，最好 = 0，并列取平均）；
   场景的弱点分 = 它所有（项, 切片）得分的平均。和单位无关，不需要把 dB、比例、分数换算到一起
4. **排序**：先按不过数从多到少，再按弱点分从高到低。有值的（项, 切片）少于 `MIN_CELLS` 的场景不参与排名，单列
5. **核对临时选的加训场景**：取前 `TOP_K` 个场景，与 `PROVISIONAL` 求交。交集 ≥ `MIN_OVERLAP` → 「临时选择与数据一致」；
   否则 → 「临时选择与数据不一致，建议按前 TOP_K 重选」。这只是判据，结论由大总管在出数后下

用法：python gap.py <scorecard.json> [--system gtcrn] [--out <目录>]  → 打印 Markdown，`--out` 时另写 gap.md + gap.json（只新建）
"""
import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path

TOP_K = 4  # 取最弱的几个场景（计划 014：先加训四个场景）
MIN_OVERLAP = 3  # 前 TOP_K 与临时选择至少重合几个，算「一致」
MIN_CELLS = 10  # 一个场景至少要有几个有值的（项, 切片）才参与排名
# 临时选定的四个加训场景（exp016 旧表，回混口径）：街道 · 车站 · 车内 · 咖啡馆
PROVISIONAL = ("street-outdoor", "transit-hub", "car-passenger", "cafe")


def _cells(card):
    """一张卡里参与排名的项：(项名, 均值, 越小越好?, 是否不过)。"""
    out = []
    items = [i for i in card["items"] if not i.get("report_only")]
    items += card.get("listen", {}).get("models", [])
    for i in items:
        m = i.get("mean")
        if m is None:
            continue
        out.append((i["v"], float(m), i.get("better") == "lower", i.get("status") == "fail"))
    return out


def _ranks(vals):
    """{场景: 值}（已按「越大越差」统一方向）→ {场景: 0–1 名次分}，最差 = 1，并列取平均。"""
    keys = sorted(vals, key=lambda k: vals[k])
    n = len(keys)
    if n == 1:
        return {keys[0]: 0.5}
    out, i = {}, 0
    while i < n:
        j = i
        while j + 1 < n and vals[keys[j + 1]] == vals[keys[i]]:
            j += 1
        r = (i + j) / 2 / (n - 1)
        for k in keys[i : j + 1]:
            out[k] = r
        i = j + 1
    return out


def gap_table(card_json, system):
    """返回 dict：rows（排好序的场景）· excluded（格子太少的场景）· top · overlap · verdict。"""
    cards = [c for c in card_json["scenes"] if c["system"] == system]
    if not cards:
        raise ValueError(f"scorecard.json 里没有系统 {system}")
    fails = defaultdict(int)
    by = defaultdict(dict)  # (项, 档, 设备) → {场景: 统一方向后的值}
    for c in cards:
        for v, m, lower, failed in _cells(c):
            by[(v, c["tier"], c["device"])][c["scene"]] = m if lower else -m
            fails[c["scene"]] += int(failed)
    score, n = defaultdict(float), defaultdict(int)
    for vals in by.values():
        for s, r in _ranks(vals).items():
            score[s] += r
            n[s] += 1
    scenes = sorted({c["scene"] for c in cards})
    rows = [{"scene": s, "fails": fails[s], "weakness": score[s] / n[s], "cells": n[s]} for s in scenes if n[s] >= MIN_CELLS]
    excluded = [{"scene": s, "cells": n[s]} for s in scenes if n[s] < MIN_CELLS]
    rows.sort(key=lambda r: (-r["fails"], -r["weakness"], r["scene"]))
    for i, r in enumerate(rows, 1):
        r["rank"] = i
    top = [r["scene"] for r in rows[:TOP_K]]
    overlap = sorted(set(top) & set(PROVISIONAL))
    verdict = "一致" if len(overlap) >= MIN_OVERLAP else "不一致"
    return {"system": system, "rows": rows, "excluded": excluded, "top": top, "provisional": list(PROVISIONAL),
            "overlap": overlap, "verdict": verdict,
            "params": {"TOP_K": TOP_K, "MIN_OVERLAP": MIN_OVERLAP, "MIN_CELLS": MIN_CELLS}}


def to_markdown(g, source):
    s = [f"# 场景差距表（{g['system']}）", "", f"来源：`{source}`；算法与阈值见 `gap.py` 文件头。", "",
         "| 名次 | 场景 | 不过的（项, 切片）数 | 弱点分（0 最好 · 1 最差） | 有值格数 | 临时选定 |", "|---|---|---|---|---|---|"]
    for r in g["rows"]:
        s.append(f"| {r['rank']} | {r['scene']} | {r['fails']} | {r['weakness']:.3f} | {r['cells']} | "
                 f"{'是' if r['scene'] in g['provisional'] else ''} |")
    if g["excluded"]:
        s += ["", "格子太少、不参与排名：" + " · ".join(f"{e['scene']}（{e['cells']} 格）" for e in g["excluded"])]
    s += ["", f"前 {g['params']['TOP_K']}：{' · '.join(g['top'])}",
          f"临时选定：{' · '.join(g['provisional'])}",
          f"重合 {len(g['overlap'])} 个（{' · '.join(g['overlap']) or '无'}），线 {g['params']['MIN_OVERLAP']} → **{g['verdict']}**",
          "", "结论（加训哪几个场景）由大总管在看过本表后下。"]
    return "\n".join(s) + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("scorecard")
    ap.add_argument("--system", default="gtcrn")
    ap.add_argument("--out")
    a = ap.parse_args(argv)
    g = gap_table(json.loads(Path(a.scorecard).read_text()), a.system)
    md = to_markdown(g, a.scorecard)
    if a.out:
        out = Path(a.out)
        out.mkdir(parents=True, exist_ok=True)
        for name, text in (("gap.md", md), ("gap.json", json.dumps(g, ensure_ascii=False, indent=1) + "\n")):
            with open(out / name, "x") as f:  # 只新建，不覆盖
                f.write(text)
    sys.stdout.write(md)


if __name__ == "__main__":
    main()
