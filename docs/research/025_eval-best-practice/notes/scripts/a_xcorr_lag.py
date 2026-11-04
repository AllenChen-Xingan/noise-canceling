#!/usr/bin/env python3
"""A 组 · V-LAT 互相关取峰重算（调研 025，group-A.md §3.1、§3.6 附注）。

用途：用纯标准库复现 `src/eval/src/analysis.rs:230` 的 `xcorr_lag`
（未归一化点积、前 32000 样本、滞后 0..2048、取正峰最大），在「原声直通」情形
（输入与自身相关）下找出哪几段会取错峰，并给出归一化相关系数作对照。

输入：exp014（run id 20261104-000444-e4511abc）所用的 480 段场景，取 exp013 build
`20261103230756-17f5` 下每段目录里的 `mix.wav`（float32 或 16 位 PCM 均可）。
  - 详查：给一个或几个 mix.wav 路径
  - 全扫：--scan <build 目录>，逐个子目录读 mix.wav（纯 Python 约 1.5 s/段，480 段约 12 分钟）

用法示例（从仓库根运行，路径为仓库相对路径）：
  python3 -I a_xcorr_lag.py \
    ../noise_canceling/_tmp/exp013/20261103230756-17f5/phone-booth-c07-ext/mix.wav \
    ../noise_canceling/_tmp/exp013/20261103230756-17f5/phone-booth-c07-mba/mix.wav
  python3 -I a_xcorr_lag.py --scan ../noise_canceling/_tmp/exp013/20261103230756-17f5

输出对应 group-A.md：
  - §3.1：c07-ext argmax 106、r(0) 1.9742、r(106) 1.9765、归一化 0.824、两窗能量 1.974 / 2.915、
          每 0.1 s 能量；c07-mba argmax 106、r(0) 2.0337、r(106) 2.0456、归一化 0.827
  - §3.6 附注：全扫时只列 argmax ≠ 0 的段（应只有 c07 两段）
"""

import math
import os
import struct
import sys
import time

WIN = 32000
MAX_LAG = 2048


def read_wav(path):
    """读单声道（多声道取第 1 路）WAV，支持格式 1（PCM16）与 3（float32）。"""
    with open(path, "rb") as f:
        d = f.read()
    i, fmt, ch, sr, sw, body = 12, None, None, None, None, None
    while i + 8 <= len(d):
        cid = d[i : i + 4]
        sz = struct.unpack("<I", d[i + 4 : i + 8])[0]
        chunk = d[i + 8 : i + 8 + sz]
        if cid == b"fmt ":
            fmt, ch, sr = struct.unpack("<HHI", chunk[:8])
            sw = struct.unpack("<H", chunk[14:16])[0] // 8
        elif cid == b"data":
            body = chunk
        i += 8 + sz + (sz & 1)
    if body is None or sw is None:
        raise ValueError(f"{path}: 没有 fmt 或 data 块")
    n = len(body) // sw
    if sw == 2:
        x = [v / 32768 for v in struct.unpack(f"<{n}h", body[: n * 2])]
    elif sw == 4 and fmt == 3:
        x = list(struct.unpack(f"<{n}f", body[: n * 4]))
    else:
        raise ValueError(f"{path}: 不支持 fmt={fmt} sw={sw}")
    return x[::ch], sr


def dot(a, b):
    return sum(u * v for u, v in zip(a, b))


def xcorr_lag(x, y, max_lag=MAX_LAG, win=WIN):
    """与 analysis.rs:230 同口径；返回 (lag, 各滞后点积)。"""
    n = min(win, len(x))
    a = x[:n]
    r = []
    for lag in range(max_lag + 1):
        if lag + n > len(y):
            break
        r.append(dot(a, y[lag : lag + n]))
    best = max(range(len(r)), key=lambda k: (r[k], -k))  # 并列取最小滞后，同 Rust 的严格 >
    return (best if r[best] > 0 else None), r


def detail(path):
    x, sr = read_wav(path)
    lag, r = xcorr_lag(x, x)
    n = min(WIN, len(x))
    e0 = dot(x[:n], x[:n])
    el = dot(x[lag : lag + n], x[lag : lag + n])
    norm = r[lag] / math.sqrt(e0 * el) if e0 > 0 and el > 0 else float("nan")
    seg = os.path.basename(os.path.dirname(path))
    print(
        f"{seg} sr={sr} len={len(x)} argmax={lag} ({lag * 1000 / sr:.3f} ms) "
        f"r0={r[0]:.4f} r_max={r[lag]:.4f} norm_at_max={norm:.3f} "
        f"E[0:n]={e0:.3f} E[lag:lag+n]={el:.3f}"
    )
    step = sr // 10
    es = [dot(x[i * step : (i + 1) * step], x[i * step : (i + 1) * step]) for i in range(22)]
    print("  每 0.1 s 能量:", " ".join(f"{v:.2g}" for v in es))


def scan(root):
    bad = 0
    total = 0
    for d in sorted(os.listdir(root)):
        p = os.path.join(root, d, "mix.wav")
        if not os.path.exists(p):
            continue
        x, _ = read_wav(p)
        lag, _ = xcorr_lag(x, x)
        total += 1
        if lag != 0:
            bad += 1
            print(f"{d} argmax={lag}", flush=True)
    print(f"共 {total} 段，argmax ≠ 0 的 {bad} 段")


def main(argv):
    t0 = time.time()
    if len(argv) >= 2 and argv[0] == "--scan":
        scan(argv[1])
    elif argv:
        for p in argv:
            detail(p)
    else:
        print(__doc__)
        return 2
    print(f"用时 {time.time() - t0:.1f} s", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
