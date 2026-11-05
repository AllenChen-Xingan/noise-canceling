#!/usr/bin/env python3
"""E 组重算：吃字（V-EAT-ASR）按场景合并 + bootstrap 区间，可懂度（V-ESTOI）分布，所需字数粗估。

用途：复现调研 025 E 组笔记（group-E.md）里的数，只汇总已有逐段分数，不跑模型。
输入：exp014 run `20261104-000444-e4511abc` 的 `clips.jsonl`（1440 行 = 480 段 × 3 个系统：
      passthrough / ref / gtcrn）；全部 480 段都用。
用法（在仓库根运行）：
    python3 -I e_eat_estoi_recompute.py \
        ../noise_canceling-card1/docs/experiments/exp014-scorecard-gtcrn/runs/20261104-000444-e4511abc/clips.jsonl
输出对应笔记：
    [A] 每段多错字数分布、参考字数中位与合计      → 3.1 表「每段多错字数的分布」，一节 V-EAT-ASR 背景
    [B] 全集合并 Δ(删+替)、按段 / 按同一句成组 bootstrap 95% 区间、多错段 / 少错段、符号检验 z、
        Σ|Δ| ÷ 总字数（不一致字比例下限）       → 3.1 表；3.4「至少占 2.6%」
    [C] ΔESTOI 判定计数、均值、标准差、最小值、每「场景 × 档」段数范围与标准差中位
                                                → 第一节第 3 问（最低 −0.017 / −0.011）、第 5 问（0.014 / 0.011 / 约 0.01）
    [D] 所需字数 / 段数粗估（纯算术，推断）       → 3.4 两张表

方法：多错字数 = round(V-EAT-ASR 值 × ref_len ÷ 100)；合并 Δ = Σ多错 ÷ Σ参考字 × 100。
bootstrap：每个系统新建 random.Random(20261030)，先按「场景 × 档 × 句号 cNN」成组（同一句的两种设备版本
一起抽，240 组）抽 4000 次，再用同一个随机数流按段（480 段）抽 4000 次；取第 100 与第 3900 个
（排序后）作 2.5% / 97.5% 分位。
"""
import collections
import json
import math
import random
import statistics as st
import sys

SYSTEMS = ("passthrough", "ref", "gtcrn")
REPS = 4000
SEED = 20261030


def score(row, vid):
    for s in row["scores"]:
        if s["v"] == vid:
            return s
    return None


def boot(pairs, rnd):
    out = []
    for _ in range(REPS):
        s = [pairs[rnd.randrange(len(pairs))] for _ in pairs]
        out.append(sum(a for a, _ in s) / sum(b for _, b in s) * 100)
    out.sort()
    return out[int(REPS * 0.025)], out[int(REPS * 0.975)]


def eat_block(rows, name):
    d, lens, keys, verdicts = [], [], [], collections.Counter()
    for r in rows:
        s = score(r, "V-EAT-ASR")
        verdicts[s["verdict"]] += 1
        if s["verdict"] == "unknown" or "ref_len" not in s["extra"]:
            continue
        ln = s["extra"]["ref_len"]
        d.append(round(s["value"] * ln / 100))
        lens.append(ln)
        keys.append((r["scene"], r["tier"], r["clip"].rsplit("-", 1)[0]))
    dist = sorted(collections.Counter(d).items())
    print(f"[A] {name}: 判定 {dict(verdicts)}；参考字数中位 {st.median(lens):.0f}，合计 {sum(lens):.0f}")
    print(f"    每段多错字数分布（字数, 段数）：{dist}")
    pos, neg = sum(x > 0 for x in d), sum(x < 0 for x in d)
    n = pos + neg
    z = (pos - n / 2) / math.sqrt(n / 4) if n else float("nan")
    pooled = sum(d) / sum(lens) * 100
    absr = sum(abs(x) for x in d) / sum(lens) * 100
    if n == 0:
        print(f"[B] {name}: 合并 Δ = {pooled:+.2f} 个百分点，所有段 Δ = 0（不做 bootstrap）")
        return
    groups = collections.defaultdict(lambda: [0, 0])
    for k, x, ln in zip(keys, d, lens):
        groups[k][0] += x
        groups[k][1] += ln
    rnd = random.Random(SEED)
    g_lo, g_hi = boot([tuple(v) for v in groups.values()], rnd)
    s_lo, s_hi = boot(list(zip(d, lens)), rnd)
    print(
        f"[B] {name}: 合并 Δ(删+替) = {pooled:+.2f} 个百分点；Σ|Δ|÷总字数 = {absr:.2f}%；"
        f"多错段 {pos} / 少错段 {neg}，符号检验 z = {z:.2f}"
    )
    print(f"    按段 bootstrap 95% [{s_lo:+.2f}, {s_hi:+.2f}]；按同一句成组（{len(groups)} 组）[{g_lo:+.2f}, {g_hi:+.2f}]")


def estoi_block(rows, name):
    sc = [(r, score(r, "V-ESTOI")) for r in rows]
    verdicts = collections.Counter(s["verdict"] for _, s in sc)
    vals = [s["value"] for _, s in sc if s["verdict"] != "unknown"]
    by = collections.defaultdict(list)
    for r, s in sc:
        if s["verdict"] != "unknown":
            by[(r["scene"], r["tier"])].append(s["value"])
    sizes = [len(v) for v in by.values()]
    sds = [st.stdev(v) for v in by.values() if len(v) > 2]
    print(
        f"[C] {name}: 判定 {dict(verdicts)}；ΔESTOI 均值 {st.mean(vals):.4f}，标准差 {st.pstdev(vals):.4f}，"
        f"最小 {min(vals):.3f}；每「场景 × 档」{min(sizes)}–{max(sizes)} 段，标准差中位 {st.median(sds):.4f}"
    )


def sample_size():
    z = 1.96 + 0.84  # 双侧 5%、80% 把握
    deff = 1.5  # 同一段里的字相关，设计效应（推断）
    print("[D] 所需字数（q = 两边对错不一致字比例，d = 要分出的合并差；× 设计效应 1.5；每段 27 字）")
    for q in (0.02, 0.05, 0.10):
        for d in (0.01, 0.02, 0.03):
            n = z * z * q / d / d * deff
            print(f"    q={q:.2f} d={d * 100:.0f}pp：约 {n:.0f} 字，约 {n / 27:.0f} 段")
    print("[D] 给定字数时的最小可分差（MDE）与 95% 区间半宽")
    for q in (0.02, 0.05, 0.10):
        for n in (450, 900, 6500, 13000):
            se = math.sqrt(q / n * deff)
            print(f"    q={q:.2f} n={n}：MDE {z * se * 100:.2f}pp，半宽 {1.96 * se * 100:.2f}pp")


def main():
    if len(sys.argv) != 2:
        sys.exit("用法：e_eat_estoi_recompute.py <clips.jsonl>")
    with open(sys.argv[1], encoding="utf-8") as f:
        rows = [json.loads(line) for line in f if line.strip()]
    for name in SYSTEMS:
        sub = [r for r in rows if r["system"] == name]
        eat_block(sub, name)
        estoi_block(sub, name)
    sample_size()


if __name__ == "__main__":
    main()
