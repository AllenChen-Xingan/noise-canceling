"""把乘加计数和测速读数合成一张表，写到 run 目录的 summary.md。

用法：python report.py runs/<burst-run> [runs/<paced-run>]

用户端耗时：**情景推算，不是上下界，也没经 Rust 实测校准**（推断）。产品用的是手写 Rust 版
（020 的 v4i），不是 torch；019 standard 第 9 节规定 Python 测的数不进结论，所以 torch 读数只列出来看相对快慢，
不参与推算。情景：
- 一般候选：020 的 16 ms 节奏实测区间 × (1 + 候选新增乘加 / v4i 基座乘加 366,460)，即假设耗时与乘加成正比
- 多加一层双路 RNN：另取「020 逐层实测里一层双路 RNN 的时间」加到连续跑整帧上，再按同一比例换到节奏下；
  两种情景取较大者（GRU 卡在串行链上，乘加比例会低估；torch 里也看到 +31% 对乘加 +16.7%）
CPU 占比只作参考（019 standard §6.1：D75 后挡线是对性能的影响 BE-IMPACT 与耗电 BE-POWER，这两项未测）。
"""
import json
import statistics
import sys
from pathlib import Path

import macs
import model
import variants

# 020 调研的实测数（出处：docs/research/020_fastpath/decision.md「下界对照表」与「整帧按 16 ms 节奏」表）
V4I_BURST_US = 36.2             # 性能核连续跑整帧
V4I_BASE_MACS = 366_460         # 020 cost.rs 每帧乘加（ERB 按稀疏滤波计）
V4I_PACED_MS = (0.073, 0.092)   # 16 ms 节奏每帧平均（电池，3 轮）
# 两层双路 RNN 合计：沿时间 GRU+全连接 7.43 + 沿频率 GRU 7.41 + 沿频率全连接 1.16 + 两个 LN 0.48+0.99
V4I_DPGRNN_PAIR_US = 7.43 + 7.41 + 1.16 + 0.48 + 0.99
HOP_MS = 16.0
CPU_TARGET_PCT, CPU_LIMIT_PCT = 0.50, 1.00  # 019 standard.md：GTCRN 推理占一个核的目标 / 上限


def load(run):
    rows = [json.loads(l) for l in open(Path(run) / "bench.jsonl")]
    by = {}
    for r in rows:
        by.setdefault(r["name"], []).append(r)
    return {n: {"p50": statistics.median(x["p50_ms"] for x in v),
                "mean": statistics.median(x["mean_ms"] for x in v),
                "p99": max(x["p99_ms"] for x in v)} for n, v in by.items()}


def counts():
    out = {}
    for n in variants.NAMES:
        m = model.make(n, weights=False)
        c = macs.count_frame(m)
        out[n] = {"macs": c["total"], "extra_macs": c["extra"],
                  "extra_params": macs.count_params(m.extra),
                  "extra_state_kb": macs.state_bytes(m.extra.init_state()) / 1024}
    return out


def rust_estimate(n, c, burst=None):
    """返回乘到 020 节奏区间上的倍数（情景，见文件头）。burst 只为保留签名，不参与。"""
    r = 1 + c[n]["extra_macs"] / V4I_BASE_MACS
    if n.startswith("dpgrnn3"):
        add = V4I_DPGRNN_PAIR_US / 2 * (c[n]["extra_macs"] / c["dpgrnn3"]["extra_macs"])
        r = max(r, 1 + add / V4I_BURST_US)
    return r, r


def fmt_row(n, c, burst, paced):
    lo, hi = rust_estimate(n, c, burst)
    ms_lo, ms_hi = V4I_PACED_MS[0] * lo, V4I_PACED_MS[1] * hi
    pct = lambda ms: ms / HOP_MS * 100
    b = burst.get(n, {}) if burst else {}
    p = paced.get(n, {}) if paced else {}
    cell = lambda d, k: f"{d[k]:.3f}" if k in d else "—"
    return (f"| {n} | {c[n]['extra_params']} | {c[n]['extra_macs']} | "
            f"{c[n]['extra_macs'] / V4I_BASE_MACS * 100:+.1f}% | {c[n]['extra_state_kb']:.1f} | "
            f"{cell(b, 'p50')} | {cell(p, 'mean')} | {cell(p, 'p99')} | "
            f"{ms_lo:.3f}–{ms_hi:.3f} | {pct(ms_lo):.2f}–{pct(ms_hi):.2f}% |")


def main(argv):
    burst = load(argv[0]) if argv else {}
    paced = load(argv[1]) if len(argv) > 1 else {}
    c = counts()
    lines = [
        "# exp019 汇总（report.py 生成，勿手改）", "",
        f"burst run：`{Path(argv[0]).name if argv else '—'}` · paced run：`{Path(argv[1]).name if len(argv) > 1 else '—'}`", "",
        "列说明：加参数 / 加乘加 = 候选模块本身；乘加涨幅 = 新增乘加 / 020 口径基座 366,460；缓存 = 候选新增的流式状态。",
        "torch 列是实测（单线程、一次一帧），只看相对快慢、不进结论；v4i 列是情景推算（推断，见 report.py 头注释），不是上下界。",
        f"占一个核 = 每帧耗时 ÷ 16 ms，只作参考（019 §6.1 的旧分摊：目标 {CPU_TARGET_PCT}%、上限 {CPU_LIMIT_PCT}%；D75 后不挡线）。", "",
        "| 候选 | 加参数 | 加乘加/帧 | 乘加涨幅 | 加缓存 KB | torch burst p50 ms | torch 16ms 节奏 平均 ms | 节奏 p99 ms | v4i 情景推算 ms/帧（节奏） | 占一个核（参考） |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    lines += [fmt_row(n, c, burst, paced) for n in variants.NAMES]
    text = "\n".join(lines) + "\n"
    if argv:
        (Path(argv[0]) / "summary.md").write_text(text)
    print(text)


if __name__ == "__main__":
    main(sys.argv[1:])
