"""exp025 · 场景差距表两种：用来核对临时选定的加训场景（计划 014）。两表都出前 TOP_K 与重合度，结论由大总管交人定。

- **弱项表**（`weak`）：GTCRN 自己最弱的场景（下面第 1–5 条）
- **提升空间表**（`uplift`）：已有候选模型在不回混口径下比 GTCRN 好多少（第 6 条）

只读 `vdn-eval scorecard` 写出的 `scorecard.json`（每个 场景 × 档 × 设备 × 系统 一张卡，每项的均值、线、判定都是
Rust 算好的），不重算任何评分项，所以这里不是 verifier 的第二处表示。

算法（阈值的正本是本文件的常量）：
1. 只看一个系统（`--system`），只看「判定项」：卡里 `report_only` 为假、有均值的项，外加两个听感评分模型
   （`listen.models`）。逐段判的物理项（坏值、延迟）也算，它们的均值照样能排
2. **不过数**：每个场景，所有 档 × 设备 切片里判「不过」（`fail`）的项数之和
3. **弱点分**：每一项、每个 档 × 设备 切片，把有值的场景按均值从好到差排名，换算成 0–1（最差 = 1，最好 = 0，并列取平均）；
   场景的弱点分 = 它所有（项, 切片）得分的平均。和单位无关，不需要把 dB、比例、分数换算到一起
4. **排序**：先按不过数从多到少，再按弱点分从高到低。有值的（项, 切片）少于 `MIN_CELLS` 的场景不参与排名，单列
5. **核对临时选的加训场景**：取前 `TOP_K` 个场景，与 `PROVISIONAL` 求交。交集 ≥ `MIN_OVERLAP` → 「一致」，否则「不一致」。
   - **第 TOP_K 名边界并列**（排序键完全相同）时，按场景名截断会改变结论，所以重合数算成一个范围：范围整个 ≥ 线才「一致」，
     整个 < 线才「不一致」，跨线就「不确定（第 4 名边界并列）」
   - **数据不全不下结论**：场景数 < `MIN_SCENES`、有场景缺切片（≠ `SLICES`）、有场景因格子太少被排除、成绩单有没跑成的段，
     任一成立 → 「数据不足」，只展示排名。这只是判据，结论由大总管在出数后下

6. **提升空间表**：读 `vdn-eval compare --a <GTCRN> --b <候选> --by-scene` 的 `compare.json`（成对 bootstrap 的
   「涨 / 退 / 不确定」由 Rust 判好），外加基线的 `scorecard.json`（只用来认哪些是判定项）。每个场景：
   **涨数** = 判定项里判「涨」的个数，**退数** 同理；**提升分** = 每项按「候选比 GTCRN 好多少」（差值均值换成越大越好的方向）
   把场景排名换成 0–1（提升最大 = 1），再对项取平均。先按涨数、再按提升分排序；前 TOP_K 与临时选择的重合用同一条线
   `--exclude` 去掉两边量法不同的项（例如比较的两份来自不同代码版本、其中某项改过量法）。
   **数据不全不下结论**（提升空间表自己的条件）：逐场景结果 < `MIN_SCENES`；有场景的有差值判定项 < `MIN_ITEMS`；
   有片段只在一边；某场景的配对段数 ≠ 基线成绩单里该场景的段数（两边共同缺段时 only_a / only_b 是空的，靠这条抓；
   基线卡里没有段数也算核不了）；基线成绩单自己有基线系统没跑成的段。任一成立 → 「数据不足」。compare 不带切片信息，所以这张表不查切片

用法：
  python gap.py weak <scorecard.json> [--system gtcrn] [--out <目录>]                       → gap-weak.md / .json
  python gap.py uplift <compare.json> --card <基线 scorecard.json> [--exclude V-TIMBRE] [--out <目录>] → gap-uplift.md / .json
打印 Markdown；`--out` 时另写文件（只新建）
"""
import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path

TOP_K = 4  # 取最弱的几个场景（计划 014：先加训四个场景）
MIN_OVERLAP = 3  # 前 TOP_K 与临时选择至少重合几个，算「一致」
MIN_CELLS = 10  # 一个场景至少要有几个有值的（项, 切片）才参与排名
MIN_SCENES = 15  # 数据完整才下判断：开会场景数（与 src/eval/suite/scenes.json 的场景数比对，见 test_gap.py）
SLICES = 4  # 每个场景要有的 档 × 设备 切片数（常见 / 压力 × 两种设备）
MIN_ITEMS = 5  # 提升空间表：每个场景至少要有几个有差值的判定项
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


def _verdict(rows, key):
    """rows 已按 key 从弱到强（从多到少）排好。返回 ((重合数下限, 上限), 判断)，第 TOP_K 名边界并列时重合数是范围。"""
    if len(rows) < TOP_K:
        return (0, 0), "不一致"
    kth = key(rows[TOP_K - 1])
    certain = [r for r in rows if key(r) > kth]
    edge = [r for r in rows if key(r) == kth]
    need = TOP_K - len(certain)
    a = sum(r["scene"] in PROVISIONAL for r in certain)
    b = sum(r["scene"] in PROVISIONAL for r in edge)
    c = len(edge) - b
    lo, hi = a + max(0, need - c), a + min(need, b)
    if lo >= MIN_OVERLAP:
        return (lo, hi), "一致"
    if hi < MIN_OVERLAP:
        return (lo, hi), "不一致"
    return (lo, hi), f"不确定（第 {TOP_K} 名边界并列）"


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
    slices = defaultdict(set)
    for c in cards:
        slices[c["scene"]].add((c["tier"], c["device"]))
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
    rng, verdict = _verdict(rows, lambda r: (r["fails"], r["weakness"]))
    incomplete = []
    if card_json.get("missing"):
        incomplete.append(f"成绩单有 {len(card_json['missing'])} 段没跑成")
    if len(rows) < MIN_SCENES:
        incomplete.append(f"参与排名的场景只有 {len(rows)} 个 < {MIN_SCENES}")
    short = sorted(s for s in scenes if len(slices[s]) != SLICES)
    if short:
        incomplete.append(f"缺切片的场景：{' · '.join(short)}")
    if incomplete:
        verdict = "数据不足"
    return {"system": system, "rows": rows, "excluded": excluded, "top": top, "provisional": list(PROVISIONAL),
            "overlap": overlap, "overlap_range": list(rng), "verdict": verdict, "incomplete": incomplete,
            "params": {"TOP_K": TOP_K, "MIN_OVERLAP": MIN_OVERLAP, "MIN_CELLS": MIN_CELLS, "MIN_SCENES": MIN_SCENES,
                       "SLICES": SLICES}}


def uplift_table(cmp_json, card_json, exclude=()):
    """提升空间表。`cmp_json` = compare.json（必须带 --by-scene），`card_json` = 基线 scorecard.json。"""
    judged = set()
    expect = defaultdict(int)  # 场景 → 基线成绩单里的段数（核两边共同缺段用）
    no_count = False
    # 只数比较里基线那个系统的卡（一份成绩单可能扫了好几个系统）；比较没写系统名时，成绩单必须只有一个系统
    base = cmp_json.get("meta", {}).get("a_system")
    names = {c["system"] for c in card_json["scenes"]}
    if base is None and len(names) > 1:
        raise ValueError(f"比较没写基线系统名，基线成绩单里又有多个系统 {sorted(names)}：分不清该数哪个")
    for c in (c for c in card_json["scenes"] if base is None or c["system"] == base):
        if "clips" in c:
            expect[c["scene"]] += int(c["clips"])
        else:
            no_count = True
        judged |= {i["v"] for i in c["items"] if not i.get("report_only")}
        judged |= {i["v"] for i in c.get("listen", {}).get("models", [])}
    judged -= set(exclude)
    scopes = [sc for sc in cmp_json["scopes"] if sc["scope"] != "all"]
    if not scopes:
        raise ValueError("compare.json 里没有逐场景的结果：比较时要加 --by-scene")
    ups, downs = defaultdict(int), defaultdict(int)
    by = defaultdict(dict)  # 项 → {场景: 提升（越大越好）}
    pairs = {}
    for sc in scopes:
        s = sc["scope"]
        pairs[s] = int(sc.get("pairs", 0))
        ups[s] += 0
        for i in sc["items"]:
            if i["v"] not in judged:
                continue
            ups[s] += int(i["call"] == "up")
            downs[s] += int(i["call"] == "down")
            if i.get("diff"):
                d = float(i["diff"]["mean"])
                by[i["v"]][s] = -d if i.get("better") == "lower" else d
    score, n = defaultdict(float), defaultdict(int)
    for vals in by.values():
        for s, r in _ranks(vals).items():
            score[s] += r
            n[s] += 1
    rows = [{"scene": s, "ups": ups[s], "downs": downs[s], "gain": score[s] / n[s] if n[s] else 0.0, "items": n[s]}
            for s in sorted(ups)]
    rows.sort(key=lambda r: (-r["ups"], -r["gain"], r["scene"]))
    for i, r in enumerate(rows, 1):
        r["rank"] = i
    top = [r["scene"] for r in rows[:TOP_K]]
    overlap = sorted(set(top) & set(PROVISIONAL))
    rng, verdict = _verdict(rows, lambda r: (r["ups"], r["gain"]))
    incomplete = [f"逐场景结果只有 {len(rows)} 个 < {MIN_SCENES}"] if len(rows) < MIN_SCENES else []
    thin = sorted(r["scene"] for r in rows if r["items"] < MIN_ITEMS)
    if thin:
        incomplete.append(f"有差值的判定项少于 {MIN_ITEMS} 个的场景：{' · '.join(thin)}")
    base_missing = [m for m in card_json.get("missing", []) if base is None or m.get("system") == base]
    if base_missing:
        incomplete.append(f"基线成绩单有 {len(base_missing)} 段没跑成，它的段数不能当预期数")
    if no_count:
        incomplete.append("基线成绩单的卡里没有段数，核不了覆盖")
    else:
        short = sorted(s for s in pairs if pairs[s] != expect.get(s, 0))
        if short:
            incomplete.append("配对段数与基线成绩单不符的场景：" + " · ".join(f"{s}（{pairs[s]}/{expect.get(s, 0)}）" for s in short))
    for sc in cmp_json["scopes"]:
        if sc["scope"] == "all" and (sc.get("only_a") or sc.get("only_b")):
            incomplete.append(f"有片段只在一边（只在基线 {len(sc.get('only_a', []))} 段，只在候选 {len(sc.get('only_b', []))} 段）")
    if incomplete:
        verdict = "数据不足"
    meta = cmp_json.get("meta", {})
    return {"baseline": meta.get("a_system"), "candidate": meta.get("b_system"), "rows": rows, "top": top,
            "provisional": list(PROVISIONAL), "overlap": overlap, "overlap_range": list(rng),
            "verdict": verdict, "incomplete": incomplete,
            "excluded_items": sorted(exclude), "params": {"TOP_K": TOP_K, "MIN_OVERLAP": MIN_OVERLAP}}


def uplift_markdown(u, source):
    s = [f"# 提升空间表（{u['candidate']} 比 {u['baseline']}）", "", f"来源：`{source}`；算法与阈值见 `gap.py` 文件头。", ""]
    if u["excluded_items"]:
        s += [f"不比的项（两边量法不同）：{' · '.join(u['excluded_items'])}", ""]
    s += ["| 名次 | 场景 | 判定项里「涨」 | 「退」 | 提升分（0 最小 · 1 最大） | 有值项数 | 临时选定 |", "|---|---|---|---|---|---|---|"]
    for r in u["rows"]:
        s.append(f"| {r['rank']} | {r['scene']} | {r['ups']} | {r['downs']} | {r['gain']:.3f} | {r['items']} | "
                 f"{'是' if r['scene'] in u['provisional'] else ''} |")
    s += ["", f"前 {u['params']['TOP_K']}：{' · '.join(u['top'])}", f"临时选定：{' · '.join(u['provisional'])}",
          f"重合 {len(u['overlap'])} 个（{' · '.join(u['overlap']) or '无'}；计入边界并列为 {u['overlap_range'][0]}–{u['overlap_range'][1]} 个），"
          f"线 {u['params']['MIN_OVERLAP']} → **{u['verdict']}**" + ("（" + "；".join(u["incomplete"]) + "）" if u["incomplete"] else ""),
          "", "结论（加训哪几个场景）由大总管交人定。"]
    return "\n".join(s) + "\n"


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
          f"重合 {len(g['overlap'])} 个（{' · '.join(g['overlap']) or '无'}；计入边界并列为 {g['overlap_range'][0]}–{g['overlap_range'][1]} 个），"
          f"线 {g['params']['MIN_OVERLAP']} → **{g['verdict']}**" + ("（" + "；".join(g["incomplete"]) + "）" if g["incomplete"] else ""),
          "", "结论（加训哪几个场景）由大总管在看过本表后下。"]
    return "\n".join(s) + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="mode", required=True)
    w = sub.add_parser("weak")
    w.add_argument("scorecard")
    w.add_argument("--system", default="gtcrn")
    w.add_argument("--out")
    u = sub.add_parser("uplift")
    u.add_argument("compare")
    u.add_argument("--card", required=True)
    u.add_argument("--exclude", default="")
    u.add_argument("--out")
    a = ap.parse_args(argv)
    if a.mode == "weak":
        g = gap_table(json.loads(Path(a.scorecard).read_text()), a.system)
        md, stem = to_markdown(g, a.scorecard), "gap-weak"
    else:
        ex = [x for x in a.exclude.split(",") if x]
        g = uplift_table(json.loads(Path(a.compare).read_text()), json.loads(Path(a.card).read_text()), ex)
        md, stem = uplift_markdown(g, a.compare), "gap-uplift"
    if a.out:
        out = Path(a.out)
        out.mkdir(parents=True, exist_ok=True)
        for name, text in ((f"{stem}.md", md), (f"{stem}.json", json.dumps(g, ensure_ascii=False, indent=1) + "\n")):
            with open(out / name, "x") as f:  # 只新建，不覆盖
                f.write(text)
    sys.stdout.write(md)


if __name__ == "__main__":
    main()
