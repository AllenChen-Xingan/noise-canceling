#!/usr/bin/env python3
"""exp021 读数：逐段比对 · 混回取舍曲线。只用标准库。

  analyze.py check A/clips.jsonl <系统A> B/clips.jsonl <系统B>
      逐段逐项比对两路分数。除「预期会变」那几项外，差必须为 0，否则退出码 1。
  analyze.py pair A/clips.jsonl <系统A> B/clips.jsonl <系统B> --items V-NR,V-ESTOI,...
      逐段配对差（B − A）：每项均值 · 95% 区间（正态近似）· 段数 · 不为 0 的段数；另按场景给均值差。
  analyze.py oracle <clips.jsonl> <默认系统> [--onset-max 3.0] [--onset-up 9] [--bg-up 0]
      推演上限：每个场景从全部档里挑可懂度最高的一档，约束 = 字头均值 ≤ onset-max 且比默认升不超过 onset-up dB、
      背景稳比默认升不超过 bg-up；挑不到就留默认。报每场景所选档与可懂度提升，以及按段数加权的总提升。
  analyze.py curve <clips.jsonl>... --items V-NR,V-ONSET,... [--out 表.md]
      每个系统（= 一组混回参数）每项的平均值，按场景分组，写成表。
"""

import json
import math
import sys
from collections import defaultdict

# 读引擎门控 / 送原声状态或原始未对齐输出的项：预混 + 关保命这条路上必然不同，比对时只报不判。
# 来源：src/eval/src/verifiers/sim.rs（V-SIM-*，读 gate）· physical.rs（V-SWITCH 读 bypass，V-LAT 读 raw，
# V-FRAME 读每帧耗时：内置模型有、外部回放给空，所以「内置 gtcrn 对外部预混」时它必然不同）
EXPECTED_DIFF = {"V-SIM-GATE", "V-SIM-FA", "V-SIM-QUIET", "V-SIM-SWLAT", "V-SWITCH", "V-LAT", "V-FRAME"}
TOL = 0.0


def load(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def by_clip(rows, system):
    out = {}
    for r in rows:
        if r["system"] == system:
            out[r["clip"]] = {s["v"]: s for s in r["scores"]}
    return out


def same(a, b):
    if a is None or b is None:
        return a is None and b is None
    return abs(a - b) <= TOL


def check(rows_a, sys_a, rows_b, sys_b):
    """返回 (每项统计, 是否通过)。统计：段数 · 值不同的段数 · 最大差 · 判定不同的段数。"""
    a, b = by_clip(rows_a, sys_a), by_clip(rows_b, sys_b)
    stats = defaultdict(lambda: {"n": 0, "diff": 0, "max": 0.0, "verdict": 0})
    ok = True
    if set(a) != set(b) or not a:
        return {"_clips": {"a": len(a), "b": len(b)}}, False
    for clip in sorted(a):
        va, vb = a[clip], b[clip]
        for v in sorted(set(va) | set(vb)):
            st = stats[v]
            st["n"] += 1
            sa, sb = va.get(v), vb.get(v)
            if sa is None or sb is None:
                st["diff"] += 1
                if v not in EXPECTED_DIFF:
                    ok = False
                continue
            x, y = sa.get("value"), sb.get("value")
            if not same(x, y):
                st["diff"] += 1
                if x is not None and y is not None:
                    st["max"] = max(st["max"], abs(x - y))
                else:
                    st["max"] = math.inf
            if sa.get("verdict") != sb.get("verdict"):
                st["verdict"] += 1
            if v not in EXPECTED_DIFF and (st["diff"] or st["verdict"]):
                ok = False
    return dict(stats), ok


def pair(rows_a, sys_a, rows_b, sys_b, items):
    """{项: {"n", "nz", "mean", "lo", "hi", "scenes": {场景: 均值差}}}，差 = B − A。"""
    a, b = by_clip(rows_a, sys_a), by_clip(rows_b, sys_b)
    scene = {r["clip"]: r["scene"] for r in rows_a}
    out = {}
    for v in items:
        d, by_sc = [], defaultdict(list)
        for clip in sorted(set(a) & set(b)):
            x = a[clip].get(v, {}).get("value")
            y = b[clip].get(v, {}).get("value")
            if x is None or y is None:
                continue
            d.append(y - x)
            by_sc[scene[clip]].append(y - x)
        if not d:
            continue
        m = sum(d) / len(d)
        sd = math.sqrt(sum((t - m) ** 2 for t in d) / (len(d) - 1)) if len(d) > 1 else 0.0
        h = 1.96 * sd / math.sqrt(len(d))
        out[v] = {"n": len(d), "nz": sum(1 for t in d if t != 0), "mean": m, "lo": m - h, "hi": m + h,
                  "scenes": {k: sum(x) / len(x) for k, x in sorted(by_sc.items())}}
    return out


def oracle(cur, default, onset_max=3.0, onset_up=9.0, bg_up=0.0):
    """cur = curve() 的结果；返回 {场景: (所选系统, 可懂度提升)}。"""
    out = {}
    scenes = sorted(k for k in cur.get(default, {}) if k != "_all")
    for sc in scenes:
        d = cur[default][sc]
        best = (default, 0.0)
        for sys_, by in cur.items():
            m = by.get(sc)
            if not m or m.get("V-ESTOI") is None or d.get("V-ESTOI") is None:
                continue
            on, on0 = m.get("V-ONSET"), d.get("V-ONSET")
            if on is not None and (on > onset_max or on - on0 > onset_up):
                continue
            bg, bg0 = m.get("V-BGSTAB"), d.get("V-BGSTAB")
            if bg is not None and bg0 is not None and bg - bg0 > bg_up:
                continue
            gain = m["V-ESTOI"] - d["V-ESTOI"]
            if gain > best[1]:
                best = (sys_, gain)
        out[sc] = best
    return out


def mean(xs):
    xs = [x for x in xs if x is not None]
    return sum(xs) / len(xs) if xs else None


def curve(rows, items):
    """{系统: {场景: {项: 平均值}}}，另加场景 "_all"。"""
    acc = defaultdict(lambda: defaultdict(lambda: defaultdict(list)))
    for r in rows:
        for s in r["scores"]:
            if s["v"] in items:
                for scene in (r["scene"], "_all"):
                    acc[r["system"]][scene][s["v"]].append(s.get("value"))
    return {
        sys_: {sc: {v: mean(xs) for v, xs in d.items()} for sc, d in scenes.items()}
        for sys_, scenes in acc.items()
    }


def fmt(x):
    return "—" if x is None else f"{x:.3f}"


def main(argv):
    if len(argv) >= 5 and argv[0] == "check":
        stats, ok = check(load(argv[1]), argv[2], load(argv[3]), argv[4])
        print("| 项 | 段数 | 值不同 | 最大差 | 判定不同 | 预期会变 |")
        print("|---|---|---|---|---|---|")
        for v, st in sorted(stats.items()):
            if v.startswith("_"):
                print(f"段数对不上：{st}")
                continue
            print(f"| {v} | {st['n']} | {st['diff']} | {st['max']:.3g} | {st['verdict']} | "
                  f"{'是' if v in EXPECTED_DIFF else ''} |")
        print("核对：" + ("通过" if ok else "不通过"))
        return 0 if ok else 1
    if len(argv) >= 5 and argv[0] == "pair":
        items = next((argv[i + 1] for i, x in enumerate(argv) if x == "--items"), "").split(",")
        res = pair(load(argv[1]), argv[2], load(argv[3]), argv[4], items)
        print(f"差 = {argv[4]} − {argv[2]}")
        print("| 项 | 段数 | 差不为 0 | 均值差 | 95% 区间 |")
        print("|---|---|---|---|---|")
        for v, r in res.items():
            print(f"| {v} | {r['n']} | {r['nz']} | {r['mean']:+.4f} | [{r['lo']:+.4f}, {r['hi']:+.4f}] |")
        scenes = sorted({k for r in res.values() for k in r["scenes"]})
        print("\n按场景的均值差\n\n| 场景 | " + " | ".join(res) + " |\n|---" * 1 + "|---" * len(res) + "|")
        for sc in scenes:
            print(f"| {sc} | " + " | ".join(f"{res[v]['scenes'].get(sc, float('nan')):+.4f}" for v in res) + " |")
        return 0
    if len(argv) >= 3 and argv[0] == "oracle":
        def opt(k, dflt):
            return float(argv[argv.index(k) + 1]) if k in argv else dflt
        rows = load(argv[1])
        cur = curve(rows, {"V-ESTOI", "V-ONSET", "V-BGSTAB", "V-NR"})
        n_sc = defaultdict(int)
        for r in rows:
            if r["system"] == argv[2]:
                n_sc[r["scene"]] += 1
        res = oracle(cur, argv[2], opt("--onset-max", 3.0), opt("--onset-up", 9.0), opt("--bg-up", 0.0))
        print("| 场景 | 段数 | 所选档 | 可懂度提升 |\n|---|---|---|---|")
        for sc, (sys_, g) in res.items():
            print(f"| {sc} | {n_sc[sc]} | {sys_} | {g:+.4f} |")
        tot = sum(n_sc[sc] * g for sc, (_, g) in res.items()) / max(1, sum(n_sc.values()))
        print(f"\n按段数加权的可懂度总提升：{tot:+.4f}")
        return 0
    if len(argv) >= 2 and argv[0] == "curve":
        files = [a for a in argv[1:] if a.endswith(".jsonl")]
        items = next((argv[i + 1] for i, a in enumerate(argv) if a == "--items"), "").split(",")
        rows = [r for f in files for r in load(f)]
        c = curve(rows, set(items))
        lines = ["| 系统 | 场景 | " + " | ".join(items) + " |", "|---" * (len(items) + 2) + "|"]
        for sys_ in sorted(c):
            for sc in sorted(c[sys_]):
                lines.append(f"| {sys_} | {sc} | " + " | ".join(fmt(c[sys_][sc].get(v)) for v in items) + " |")
        text = "\n".join(lines) + "\n"
        out = next((argv[i + 1] for i, a in enumerate(argv) if a == "--out"), None)
        if out:
            with open(out, "x", encoding="utf-8") as f:
                f.write(text)
        print(text)
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
