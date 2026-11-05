"""exp014 新规则重跑（v2，人定 D85）对照：旧 run（v1，20261104-000444-e4511abc）vs 新 run。

用法：python3 -I final_compare.py <仓库根> <新 run 目录（含 scorecard.json · clips.jsonl）>  > compare.md
只读两份 run 的 scorecard.json 与 clips.jsonl，不重跑。
"""

import json
import statistics as st
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO, NEW = Path(sys.argv[1]), Path(sys.argv[2])
OLD = REPO / "docs/experiments/exp014-scorecard-gtcrn/runs/20261104-000444-e4511abc"
SYSTEMS = ["passthrough", "gtcrn", "ref"]


def scenes(d):
    return json.loads((d / "scorecard.json").read_text())


def rows(d):
    return [json.loads(l) for l in (d / "clips.jsonl").read_text().splitlines() if l.strip()]


def counts(sc):
    c = defaultdict(Counter)
    for s in sc["scenes"]:
        for i in s["items"]:
            c[(s["system"], i["v"])][i["status"]] += 1
        c[(s["system"], "听感")][s["listen"]["status"] or "unknown"] += 1
        c[(s["system"], "（场景）")][s["status"]] += 1
    return c


def fmt(cnt):
    return " / ".join(str(cnt.get(k, 0)) for k in ("pass", "fail", "unknown"))


def main():
    old_sc, new_sc = scenes(OLD), scenes(NEW)
    co, cn = counts(old_sc), counts(new_sc)
    out = sys.stdout
    out.write("# exp014 新规则重跑对照（v2，人定 D85）\n\n")
    out.write(f"旧 = `{OLD.name}`（旧规则 v1，原表）；新 = `{NEW.name}`（新规则 v2，重跑音频）。每格是 60 个组合的 过 / 不过 / 待测。"
              "新增项旧表没有，记 0 / 0 / 0。\n\n")
    out.write(f"新 run 没跑成的段：{len(new_sc['missing'])}；段数：{new_sc['clips_scored']}；元数据：`{json.dumps(new_sc['meta'], ensure_ascii=False)}`\n\n")
    items = []
    for s in new_sc["scenes"]:
        for i in s["items"]:
            if i["v"] not in items:
                items.append(i["v"])
    rule = {i["v"]: i["rule"] for s in new_sc["scenes"] for i in s["items"]}
    out.write("| 项 | 新判法 | " + " | ".join(f"{s} 旧 | {s} 新" for s in SYSTEMS) + " |\n")
    out.write("|---|---|" + "---|---|" * len(SYSTEMS) + "\n")
    for v in items + ["听感", "（场景）"]:
        cells = []
        for s in SYSTEMS:
            cells += [fmt(co.get((s, v), Counter())), fmt(cn.get((s, v), Counter()))]
        out.write(f"| {v} | {rule.get(v, '')[:14]} | " + " | ".join(cells) + " |\n")

    # B3：背景稳不稳改前改后（逐段）。新 run 的 uncorr_db 应等于旧 run 的主值（同一音频、同一算法 → 也验证重跑确定）
    ro, rn = rows(OLD), rows(NEW)
    key = lambda r: (r["clip"], r["system"])
    om = {key(r): {s["v"]: s for s in r["scores"]} for r in ro}
    out.write("\n## B3 背景稳不稳：改前（旧主值）vs 改后（扣输入侧偏差）\n\n")
    out.write("| 系统 | 段数 | 旧主值均值 | 新 run 的 uncorr_db 均值 | 与旧主值逐段最大差 | 改后主值均值 | 改后最大 |\n|---|---|---|---|---|---|---|\n")
    for sysn in SYSTEMS:
        a, b, c, diffs = [], [], [], []
        for r in rn:
            if r["system"] != sysn:
                continue
            ns = {s["v"]: s for s in r["scores"]}.get("V-BGSTAB")
            os_ = om.get(key(r), {}).get("V-BGSTAB")
            if not ns or not os_ or ns["value"] is None or os_["value"] is None:
                continue
            u = ns["extra"].get("uncorr_db")
            a.append(os_["value"]); b.append(u); c.append(ns["value"])
            diffs.append(abs(u - os_["value"]))
        if a:
            out.write(f"| {sysn} | {len(a)} | {st.mean(a):.3f} | {st.mean(b):.3f} | {max(diffs):.2e} | {st.mean(c):.3f} | {max(c):.3f} |\n")

    # 新项基线（逐段均值，判不了的段不计）
    out.write("\n## 新项与干净人声那一路的基线（逐段均值 · 有值段数）\n\n")
    vs = ["V-TSOS", "V-SWALLOW", "V-LEAK", "V-PSE-OVL", "V-PUMPV", "V-TIMBRE", "V-WEAK", "V-LAT"]
    out.write("| 系统 | " + " | ".join(vs) + " |\n|---|" + "---|" * len(vs) + "\n")
    for sysn in SYSTEMS:
        cells = []
        for v in vs:
            x = [s["value"] for r in rn if r["system"] == sysn for s in r["scores"] if s["v"] == v and s["value"] is not None]
            cells.append(f"{st.mean(x):.3f}（{len(x)}）" if x else "—")
        out.write(f"| {sysn} | " + " | ".join(cells) + " |\n")
    fr = Counter(r["system"] for r in rn for s in r["scores"] if s["v"] == "V-PSE-OVL" and s["extra"].get("fail_rate") == 1.0)
    out.write(f"\n同时说话只留用户：失败率 = 1 的段数 {dict(fr)}\n")


if __name__ == "__main__":
    main()
