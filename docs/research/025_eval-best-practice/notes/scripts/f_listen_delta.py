"""F 组重算：听感分 Δ（输出分 − 带噪原声分）的分布、集合层判定与所需段数。

用途：复现 025 F 组笔记（group-F.md）里的 exp014 数字，不跑任何模型，只读已打好的分。
输入：exp014 run 20261104-000444-e4511abc 的 clips.jsonl（全部 1440 行 = 3 个系统 × 480 段），
      只用 V-NRMOS-SCOREQ / V-NRMOS-DISTILL 两项的 value 与 extra.mos_mix。
用法（仓库根下）：
  python3 -I f_listen_delta.py \
    ../noise_canceling-card1/docs/experiments/exp014-scorecard-gtcrn/runs/20261104-000444-e4511abc/clips.jsonl
输出对应笔记：
  [1] 第 2 节第 4 问「exp014 实测」：各系统 Δ 均值 / 段间标准差 / 负段数 / 最差；
      逐段判定的 both<0、方向相反段数
  [2] 第 2 节第 4 问：SCOREQ Δ 与 Distill Δ 逐段相关 0.45；按带噪原声 SCOREQ 分档的负段数
  [3] 第 2 节第 5 问：组合内 Δ 段间标准差中位数；所需段数（不劣、比两个系统）
  [4] 第 0 节第 3 条、第 4 节、第 6 节第 1 题：组合均值的 bootstrap 判定，两种口径分开输出
      （δ = 0 / 0.05 / 0.1 / 0.15；每个 组合×模型 2000 次重抽，种子 20261030，两种口径共用同一批重抽）
      (a) 旧口径（对照）：单侧 95% 下界 ≥ −δ = 该模型说过，否则算「不过」；两模型都过 = 过，都不过 = 不过，其余待测
      (b) decision X2 三态：双侧 95% 区间（下界 2.5%、上界 97.5%）；下界 ≥ −δ = 该模型说过，
          上界 < −δ = 该模型说不过，其余 = 该模型说待测；两模型都过 = 过，都不过 = 不过，其余 = 待测
"""
import collections
import json
import math
import random
import statistics as st
import sys

V = ("V-NRMOS-SCOREQ", "V-NRMOS-DISTILL")


def load(path):
    rows = [json.loads(l) for l in open(path, encoding="utf-8")]
    val, mix, combo = {}, {}, {}
    for r in rows:
        combo[(r["system"], r["clip"])] = (r["scene"], r["tier"], r["device"])
        for s in r["scores"]:
            if s["v"] in V and s.get("value") is not None:
                val[(r["system"], r["clip"], s["v"])] = s["value"]
                mix[(r["clip"], s["v"])] = s["extra"].get("mos_mix")
    return rows, val, mix, combo


def pearson(a, b):
    ma, mb = st.mean(a), st.mean(b)
    num = sum((x - ma) * (y - mb) for x, y in zip(a, b))
    return num / math.sqrt(sum((x - ma) ** 2 for x in a) * sum((y - mb) ** 2 for y in b))


B = 2000


def boot_means(x, rng, b=B):
    """组合均值的 bootstrap 分布（已排序）。重抽顺序与旧版 lower_bound 相同，(a) 口径的数不变。"""
    return sorted(st.mean(rng.choices(x, k=len(x))) for _ in range(b))


def model_say_a(m, delta):
    """(a) 单侧 95% 下界：过 / 不过（不过 = 下界 < −δ）。"""
    return "过" if m[int(0.05 * B)] >= -delta else "不过"


def model_say_b(m, delta):
    """(b) 双侧 95% 区间三态：过 / 不过 / 待测。"""
    lo, hi = m[int(0.025 * B)], m[int(0.975 * B) - 1]
    if lo >= -delta:
        return "过"
    if hi < -delta:
        return "不过"
    return "待测"


def combine(a, b):
    if a == b and a in ("过", "不过"):
        return a
    return "待测"


def main(path):
    rows, val, mix, combo = load(path)
    systems = sorted({r["system"] for r in rows})
    clips = sorted({r["clip"] for r in rows})
    print("[1] 各系统 Δ 分布")
    for sy in systems:
        for v in V:
            x = [val[(sy, c, v)] for c in clips if (sy, c, v) in val]
            print(f"  {sy:12s} {v:16s} n {len(x)} 均值 {st.mean(x):+.3f} 标准差 {st.pstdev(x):.3f} "
                  f"负段 {sum(1 for t in x if t < 0)} 最差 {min(x):+.3f}")
        both = opp = 0
        for c in clips:
            a, b = val.get((sy, c, V[0])), val.get((sy, c, V[1]))
            if a is None or b is None:
                continue
            both += a < 0 and b < 0
            opp += (a < 0) != (b < 0)
        print(f"  {sy:12s} 逐段：两个都 <0 {both} · 方向相反 {opp}")
    print("[2] GTCRN：两模型相关与地板效应")
    g = {v: [val[("gtcrn", c, v)] for c in clips] for v in V}
    print(f"  SCOREQ Δ vs Distill Δ 逐段相关 {pearson(g[V[0]], g[V[1]]):.3f}")
    for lo, hi in ((0, 1.5), (1.5, 2.5), (2.5, 5.1)):
        s = [val[("gtcrn", c, V[0])] for c in clips if lo <= mix[(c, V[0])] < hi]
        print(f"  原声 SCOREQ [{lo},{hi}) 段 {len(s)} 负段 {sum(1 for t in s if t < 0)} 均值 {st.mean(s):+.3f}")
    print("[3] 组合内离散与所需段数")
    by = collections.defaultdict(lambda: collections.defaultdict(list))
    for (sy, c, v), x in val.items():
        by[(sy,) + combo[(sy, c)]][v].append(x)
    sizes = collections.Counter(len(d[V[0]]) for k, d in by.items() if k[0] == "gtcrn")
    print(f"  GTCRN 每组合段数分布 {dict(sizes)}")
    for v in V:
        sd_in = st.median(st.stdev(d[v]) for k, d in by.items() if k[0] == "gtcrn")
        diff = [val[("gtcrn", c, v)] - val[("ref", c, v)] for c in clips]
        sd_d = st.stdev(diff)
        print(f"  {v}: 组合内标准差中位 {sd_in:.3f}；不劣 n(δ=0.1) {((1.645 + 0.84) * sd_in / 0.1) ** 2:.1f}"
              f" n(δ=0.05) {((1.645 + 0.84) * sd_in / 0.05) ** 2:.1f}；"
              f"gtcrn−ref 配对差标准差 {sd_d:.3f} → 分出 0.1 要 {((1.96 + 0.84) * sd_d / 0.1) ** 2:.1f}"
              f" 段，0.05 要 {((1.96 + 0.84) * sd_d / 0.05) ** 2:.1f} 段")
    print("[4] 组合均值 bootstrap 判定（两种口径，同一批重抽）")
    rng = random.Random(20261030)
    boots = {k: tuple(boot_means(d[v], rng) for v in V) for k, d in sorted(by.items())}
    for name, say in (("(a) 单侧 95% 下界（旧口径，对照）", model_say_a),
                      ("(b) 双侧 95% 区间三态（decision X2）", model_say_b)):
        print(f"  {name}")
        for delta in (0.0, 0.05, 0.1, 0.15):
            c = collections.Counter()
            hi_fail = 0
            for k, ms in boots.items():
                says = [say(m, delta) for m in ms]
                hi_fail += sum(1 for t in says if t == "不过")
                c[(k[0], combine(*says))] += 1
            parts = []
            for sy in sorted({k[0] for k in boots}):
                parts.append(f"{sy} 过 {c[(sy, '过')]} / 不过 {c[(sy, '不过')]} / 待测 {c[(sy, '待测')]}")
            print(f"    δ={delta}: " + " · ".join(parts) + f"  ｜单个模型说「不过」的 组合×模型 数 {hi_fail}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
