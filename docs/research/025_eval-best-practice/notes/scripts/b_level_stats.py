"""B 组（电平与背景类）exp014 重算。只用 Python 标准库。

用途：从 exp014 逐段分数重算 B 组笔记（group-B.md）里的数：
  1. 第 6.1 节统计表：V-NR / V-INTF / V-SPLVL / V-BGSTAB 各系统均值与标准差，
     GTCRN − 原声直通、GTCRN − 参考降噪器的配对差（均值、标准差、场景×档内合并标准差），
     以及分辨 1 dB 所需段数 n = ((1.96 + 0.84) · sd / 1 dB)²（α = 0.05 双侧，把握度 0.8）
  2. 第 6.2 节「最差段 vs 场景均值」过线计数表（GTCRN，组合 = 场景 × 档 × 设备）
  3. 第五节 V-BGSTAB 原声直通偏差：段数、均值、标准差、最大值（及哪段）、超过 1 dB 的段数、停顿帧数中位
  4. 第一节旁证：GTCRN V-NR 纯噪声帧数中位 / 最小、简化 SNRI 均值

输入：run id 20261104-000444-e4511abc 的 clips.jsonl（每行一段 × 一个系统；480 段 × 3 个系统）。
  判「不知道」（verdict == "unknown"）或没有值的段不计；配对差只取两个系统都有值的段。

用法（在仓库根目录）：
  python3 -I b_level_stats.py \
    ../noise_canceling-card1/docs/experiments/exp014-scorecard-gtcrn/runs/20261104-000444-e4511abc/clips.jsonl
"""

import collections
import json
import math
import statistics as st
import sys

V = ["V-NR", "V-INTF", "V-SPLVL", "V-BGSTAB"]
Z = 1.96 + 0.84


def load(path):
    by = collections.defaultdict(dict)  # clip -> system -> {v: score}
    meta = {}
    for line in open(path, encoding="utf-8"):
        r = json.loads(line)
        by[r["clip"]][r["system"]] = {s["v"]: s for s in r["scores"]}
        meta[r["clip"]] = (r["scene"], r["tier"], r["device"])
    return by, meta


def val(m, v):
    s = m.get(v)
    if not s or s.get("verdict") == "unknown" or s.get("value") is None:
        return None
    return s["value"]


def table_61(by, meta):
    print("## 6.1 统计表")
    for v in V:
        print("==", v)
        for sy in ["passthrough", "ref", "gtcrn"]:
            xs = [val(by[c][sy], v) for c in by if sy in by[c]]
            xs = [x for x in xs if x is not None]
            if len(xs) > 2:
                print(f"  {sy}: n={len(xs)} mean={st.mean(xs):.2f} sd={st.stdev(xs):.2f}")
        for a, b in [("gtcrn", "passthrough"), ("gtcrn", "ref")]:
            ds, grp = [], collections.defaultdict(list)
            for c in by:
                if a in by[c] and b in by[c]:
                    x, y = val(by[c][a], v), val(by[c][b], v)
                    if x is not None and y is not None:
                        ds.append(x - y)
                        grp[meta[c][:2]].append(x - y)  # 场景 × 档（两种设备合并）
            if len(ds) < 3:
                continue
            sd = st.stdev(ds)
            ws = [st.stdev(g) for g in grp.values() if len(g) > 2]
            pw = math.sqrt(st.mean([s * s for s in ws]))
            n_all = math.ceil((Z * sd) ** 2)
            n_in = math.ceil((Z * pw) ** 2)
            mde12, mde20 = Z * pw / math.sqrt(12), Z * pw / math.sqrt(20)
            print(f"  {a}-{b}: n={len(ds)} mean={st.mean(ds):.2f} sd={sd:.2f} "
                  f"sd_within={pw:.2f} n_1dB_all={n_all} n_1dB_within={n_in} "
                  f"mde12={mde12:.2f} mde20={mde20:.2f}")


def table_62(by, meta):
    print("## 6.2 最差段 vs 场景均值（gtcrn）")
    g = collections.defaultdict(lambda: collections.defaultdict(list))
    for c in by:
        if "gtcrn" not in by[c]:
            continue
        for v in V:
            x = val(by[c]["gtcrn"], v)
            if x is not None:
                g[v][meta[c]].append(x)
    for v, thr, low in [("V-NR", -6, True), ("V-NR", -5, True), ("V-SPLVL", -3, False),
                        ("V-SPLVL", -4, False), ("V-INTF", -10, True), ("V-BGSTAB", 10, True)]:
        d = g[v]
        ok = (lambda x: x <= thr) if low else (lambda x: x >= thr)
        worst = sum(ok(max(xs) if low else min(xs)) for xs in d.values())
        mean = sum(ok(st.mean(xs)) for xs in d.values())
        lens = [len(x) for x in d.values()]
        print(f"  {v} 线 {thr}: 可判组合 {len(d)} 最差段过 {worst} 均值过 {mean} "
              f"每组合段数 {min(lens)}–{max(lens)}")


def bgstab_bias(by):
    print("## 5 V-BGSTAB 原声直通偏差")
    xs = []
    for c in by:
        m = by[c].get("passthrough")
        if m and val(m, "V-BGSTAB") is not None:
            xs.append((val(m, "V-BGSTAB"), m["V-BGSTAB"]["extra"].get("frames"), c))
    vs = [x[0] for x in xs]
    xs.sort()
    print(f"  n={len(vs)} mean={st.mean(vs):.2f} sd={st.stdev(vs):.2f} max={xs[-1][0]:.2f} ({xs[-1][2]}) "
          f">1dB={sum(v > 1 for v in vs)} frames_median={st.median([x[1] for x in xs])}")
    print("  最大 5 段:", [(round(v, 2), c) for v, _, c in xs[-5:]])


def nr_frames(by):
    print("## 1 GTCRN V-NR 帧数与简化 SNRI")
    fr, sn = [], []
    for c in by:
        m = by[c].get("gtcrn")
        if m and val(m, "V-NR") is not None:
            e = m["V-NR"]["extra"]
            fr.append(e.get("frames"))
            if e.get("snri_db") is not None:
                sn.append(e["snri_db"])
    print(f"  frames median={st.median(fr)} min={min(fr)} snri_mean={st.mean(sn):.2f}")


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    by, meta = load(sys.argv[1])
    table_61(by, meta)
    table_62(by, meta)
    bgstab_bias(by)
    nr_frames(by)


if __name__ == "__main__":
    main()
