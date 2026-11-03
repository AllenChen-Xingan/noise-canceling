#!/usr/bin/env python3
"""研究 020：把本机峰值（hw-peak）与逐层计时（gtcrn-layers）合成「离下界多少倍」表。

用法：bound_table.py <results 目录> [--qos-peak rt|bg] [--layers-qos rt|bg] [--pace 0|16]
                      [--paced-peak]（D79 能效核口径：常数取 <目录>/hwpeak_paced.jsonl，即按 16 ms 节奏醒来时量的）
                      [--peak-dir <目录>]（峰值文件在另一个结果目录时）
输入：<目录>/hwpeak.jsonl、<目录>/layers.jsonl（exp012 tools/run_peak_layers.sh 产出）
输出：Markdown 表（stdout）。

三条下界（每段、整帧都算；都只用本机实测常数，取多轮中位的中位）：
- 算力界 = 乘加次数 / FMA 吞吐（f32 个 / ns） + 非线性次数 × 向量化非线性的理想代价（见 NL_LANE_OPS）
- 搬运界 = 权重字节 / L1 读带宽（权重全部 95 KB < L1 128 KB，稳态在 L1）+ 特征读写字节 / L1 读带宽
- 依赖链界 = 关键路径乘加步数 × FMA 延迟 + 关键路径非线性次数 × sig_fast 延迟
段的下界取三者最大；整帧严格下界 = max(Σ算力, Σ搬运, Σ依赖链)（各段可重叠，所以不是各段下界之和）。
"""
import json
import statistics as st
import sys
from pathlib import Path

# 一次 exp/sigmoid/tanh 向量化后每个元素约需的乘加级运算数（Cephes 式 6 阶多项式 + 指数拼接 + 1 次除法按 4 计）。
# 这是算力界里唯一的估计值，表里单列「其中非线性」便于核对。
NL_LANE_OPS = 14


def med(vals):
    return st.median(vals)


def load(p):
    return [json.loads(l) for l in Path(p).read_text().splitlines() if l.strip()]


def arg(flag, d):
    a = sys.argv
    return a[a.index(flag) + 1] if flag in a else d


def main():
    d = Path(sys.argv[1])
    qp = arg("--qos-peak", "rt")
    ql = arg("--layers-qos", qp)
    pace = float(arg("--pace", "16"))
    pd = Path(arg("--peak-dir", str(d)))
    variant = arg("--variant", "native")
    lays = [
        r
        for r in load(d / "layers.jsonl")
        if r["qos"] == ql and float(r["pace_ms"]) == pace and r.get("backend", "native") == variant
    ]
    if "--paced-peak" in sys.argv:
        pk = [r for r in load(pd / "hwpeak_paced.jsonl") if r["qos"] == qp]
        cont = [r for r in load(pd / "hwpeak.jsonl") if r["qos"] == "rt"] if (pd / "hwpeak.jsonl").exists() else []
        if not pk or not lays:
            sys.exit(f"没有 qos={qp} 的节奏峰值或 qos={ql} pace={pace} variant={variant} 的逐层数据")
        fma_tput = med([r["fma_tput_lanes_per_ns"]["median"] for r in pk])
        fma_lat = med([r["fma_lat_first_ns"]["median"] for r in pk])
        l1 = med([r["read16k_gbps"]["median"] for r in pk])
        if cont:  # 非线性延迟没在节奏下量：按 FMA 延迟之比从连续跑换算（推断，表头写明）
            nl_lat = med([r["sig_fast_lat_ns"]["median"] for r in cont]) * fma_lat / med([r["fma_lat_ns"]["median"] for r in cont])
            nl_note = "按 FMA 延迟之比从连续跑换算（推断）"
        else:
            nl_lat, nl_note = 20 * fma_lat, "无连续跑数据，按 20 个 FMA 延迟估计"
        consts = (
            f"本机实测常数（按 {pace} ms 节奏醒来时量，qos={qp}，{len(pk)} 轮 × 每轮 {pk[0]['bursts']} 次醒来，取中位的中位）："
            f"FMA 吞吐 {fma_tput:.1f} 个/ns · FMA 延迟 {fma_lat:.3f} ns · L1 读（16 KB）{l1:.0f} GB/s · sig_fast 延迟 {nl_lat:.2f} ns（{nl_note}）；"
            f"非线性按每个 {NL_LANE_OPS} 次运算计（估计）"
        )
    else:
        peaks = [r for r in load(pd / "hwpeak.jsonl") if r["qos"] == qp]
        if not peaks or not lays:
            sys.exit(f"没有 qos={qp} 的峰值或 qos={ql} pace={pace} variant={variant} 的逐层数据")
        fma_tput = med([r["fma_tput_lanes_per_ns"]["median"] for r in peaks])
        fma_lat = med([r["fma_lat_ns"]["median"] for r in peaks])
        nl_lat = med([r["sig_fast_lat_ns"]["median"] for r in peaks])
        rkey = "read16_gbps" if "read16_gbps" in peaks[0] else "read_gbps"
        l1 = med([max(r["read_gbps"]["64KB"]["median"], r.get(rkey, r["read_gbps"])["64KB"]["median"]) for r in peaks])
        consts = (
            f"本机实测常数（qos={qp}，{len(peaks)} 轮 × 每轮 {peaks[0]['reps']} 次，取中位的中位）："
            f"FMA 吞吐 {fma_tput:.1f} 个/ns · FMA 延迟 {fma_lat:.3f} ns · sig_fast 延迟 {nl_lat:.2f} ns · L1 读 {l1:.0f} GB/s；"
            f"非线性按每个 {NL_LANE_OPS} 次运算计（估计）"
        )

    n = len(lays[0]["segs"])
    rows = []
    tot = {"c": 0.0, "m": 0.0, "l": 0.0, "t": 0.0}
    for i in range(n):
        s0 = lays[0]["segs"][i]
        t = med([r["segs"][i]["mean_ns"] for r in lays])
        c = (s0["macs"] + s0["nonlin"] * NL_LANE_OPS + s0["other"]) / fma_tput
        m = (s0["w_bytes"] + s0["io_bytes"]) / l1
        lat = s0["chain_fma"] * fma_lat + s0["chain_nl"] * nl_lat
        b = max(c, m, lat)
        which = {c: "算力", m: "搬运", lat: "依赖链"}[b]
        rows.append((f"{s0['blk']}.{s0['op']}", t, s0["macs"], c, m, lat, b, which, t / b if b else float("inf")))
        tot["c"] += c
        tot["m"] += m
        tot["l"] += lat
        tot["t"] += t
    frame_t = med([r["frame_mean_ns"] for r in lays])
    strict = max(tot["c"], tot["m"], tot["l"])
    print(f"<!-- 生成：bound_table.py {d.name} qos-peak={qp} layers-qos={ql} pace={pace} -->")
    print(consts + "\n")
    print(f"逐层计时：{variant} · qos={ql} · 节奏 {pace} ms · {len(lays)} 轮 × {lays[0]['frames']} 帧，每段取各轮平均的中位\n")
    print("| 段 | 实测 ns | 乘加 | 算力界 ns | 搬运界 ns | 依赖链界 ns | 下界 ns | 卡在 | 实测 / 下界 |")
    print("|---|---:|---:|---:|---:|---:|---:|---|---:|")
    for name, t, macs, c, m, lat, b, w, ratio in rows:
        print(f"| {name} | {t:.0f} | {macs} | {c:.0f} | {m:.0f} | {lat:.0f} | {b:.0f} | {w} | {ratio:.1f} |")
    print(
        f"| **整帧** | **{frame_t:.0f}** | {lays[0]['total']['macs']} | {tot['c']:.0f} | {tot['m']:.0f} | {tot['l']:.0f} "
        f"| **{strict:.0f}** | {['算力', '搬运', '依赖链'][[tot['c'], tot['m'], tot['l']].index(strict)]} | **{frame_t / strict:.1f}** |"
    )
    print(f"\n各段实测之和 {tot['t']:.0f} ns（含打点开销；整帧实测 {frame_t:.0f} ns）")


if __name__ == "__main__":
    main()
