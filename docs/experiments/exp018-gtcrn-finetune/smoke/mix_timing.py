"""exp018 冒烟：训练数据「读入 + 在线混音」每段要多久，和单步训练耗时比，看训练卡在哪。

一段 = 读一段干净人声（随机起点裁 N 秒）+ 读一段噪声（随机起点）+ 必要时重采样到 16 kHz
       + 按概率卷一条房间冲激响应（RIR）+ 按随机信噪比混合 + 防削波。
只量速度：用哪些文件只为贴近真实读盘，不代表训练集选型（选型见调研 027）。

用法（单次 ≤ 约 1 分钟，不占重活名额）：
    python mix_timing.py --speech-glob '<…>/*.flac' --noise-glob '<…>/*.wav' --rir-glob '<…>/*.wav' \
        --clips 64 --workers 0 --out <主目录>/_tmp/027/<run-id>/mix-w0.json
"""
from __future__ import annotations

import argparse
import glob
import json
import os
import random
import statistics
import time
from multiprocessing import Pool

import numpy as np
import soundfile as sf
from scipy.signal import fftconvolve, resample_poly

FS = 16000


def read_crop(path: str, seconds: float, rng: random.Random) -> tuple[np.ndarray, int]:
    """按文件原采样率随机起点读 seconds 秒，只读需要的那一截（不整文件读）。"""
    info = sf.info(path)
    want = int(seconds * info.samplerate)
    start = rng.randint(0, max(0, info.frames - want))
    x, sr = sf.read(path, start=start, frames=want, dtype="float32", always_2d=True)
    return x[:, 0], sr


def to_16k(x: np.ndarray, sr: int) -> np.ndarray:
    if sr == FS:
        return x
    g = np.gcd(sr, FS)
    return resample_poly(x, FS // g, sr // g).astype(np.float32)


def fit(x: np.ndarray, n: int) -> np.ndarray:
    """裁或循环补到 n 点（短噪声循环，短人声补零）。"""
    if len(x) >= n:
        return x[:n]
    reps = int(np.ceil(n / max(1, len(x))))
    return np.tile(x, reps)[:n]


def mix_at_snr(clean: np.ndarray, noise: np.ndarray, snr_db: float) -> np.ndarray:
    pc = float(np.mean(clean ** 2)) + 1e-10
    pn = float(np.mean(noise ** 2)) + 1e-10
    noise = noise * np.sqrt(pc / (pn * 10 ** (snr_db / 10)))
    y = clean + noise
    peak = float(np.max(np.abs(y))) + 1e-9
    return y / peak * 0.9 if peak > 0.99 else y


def make_clip(args: tuple) -> dict:
    """生成一段，返回各阶段耗时（秒）。"""
    seed, speech, noise, rirs, seconds, p_rir = args
    rng = random.Random(seed)
    n = int(seconds * FS)
    t = {}
    t0 = time.perf_counter()
    s, ssr = read_crop(rng.choice(speech), seconds, rng)
    z, zsr = read_crop(rng.choice(noise), seconds, rng)
    h = None
    if rirs and rng.random() < p_rir:
        h, hsr = sf.read(rng.choice(rirs), dtype="float32", always_2d=True)
        h = h[:, 0]
    t1 = time.perf_counter()
    s = fit(to_16k(s, ssr), n)
    z = fit(to_16k(z, zsr), n)
    if h is not None:
        h = to_16k(h, hsr)
    t2 = time.perf_counter()
    if h is not None:
        s = fftconvolve(s, h / (np.max(np.abs(h)) + 1e-9))[:n].astype(np.float32)
    t3 = time.perf_counter()
    y = mix_at_snr(s, z, rng.uniform(-5.0, 15.0))
    t4 = time.perf_counter()
    t.update(read=t1 - t0, resample=t2 - t1, rir=t3 - t2, mix=t4 - t3, total=t4 - t0,
             n=len(y), finite=bool(np.isfinite(y).all()), used_rir=h is not None)
    return t


def run(speech: list[str], noise: list[str], rirs: list[str], clips: int, seconds: float,
        workers: int, p_rir: float, seed: int = 0) -> dict:
    jobs = [(seed + i, speech, noise, rirs, seconds, p_rir) for i in range(clips)]
    t0 = time.perf_counter()
    if workers <= 0:
        per = [make_clip(j) for j in jobs]
    else:
        with Pool(workers) as pool:
            per = pool.map(make_clip, jobs, chunksize=max(1, clips // (workers * 4)))
    wall = time.perf_counter() - t0

    def med(k: str) -> float:
        return statistics.median(p[k] for p in per)

    return {
        "clips": clips,
        "seconds_per_clip": seconds,
        "workers": workers,
        "p_rir": p_rir,
        "wall_s": wall,
        "clips_per_s": clips / wall,
        "audio_hours_per_wall_hour": clips * seconds / wall,
        "per_clip_median_ms": {k: 1000 * med(k) for k in ("read", "resample", "rir", "mix", "total")},
        "rir_used": sum(p["used_rir"] for p in per),
        "all_finite": all(p["finite"] for p in per),
        "all_len_ok": all(p["n"] == int(seconds * FS) for p in per),
        "pool_sizes": {"speech": len(speech), "noise": len(noise), "rir": len(rirs)},
        "cpu_count": os.cpu_count(),
    }


def expand(patterns: list[str]) -> list[str]:
    out: list[str] = []
    for p in patterns:
        out += sorted(glob.glob(os.path.expanduser(p), recursive=True))
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--speech-glob", action="append", required=True)
    ap.add_argument("--noise-glob", action="append", required=True)
    ap.add_argument("--rir-glob", action="append", default=[])
    ap.add_argument("--clips", type=int, default=64)
    ap.add_argument("--seconds", type=float, default=4.0)
    ap.add_argument("--workers", type=int, default=0, help="0 = 单进程")
    ap.add_argument("--p-rir", type=float, default=0.5)
    ap.add_argument("--out", default=None)
    a = ap.parse_args(argv)
    speech, noise, rirs = expand(a.speech_glob), expand(a.noise_glob), expand(a.rir_glob)
    if not speech or not noise:
        raise SystemExit("人声或噪声文件为空，检查 glob")
    res = run(speech, noise, rirs, a.clips, a.seconds, a.workers, a.p_rir)
    text = json.dumps(res, ensure_ascii=False, indent=2)
    if a.out:
        os.makedirs(os.path.dirname(a.out) or ".", exist_ok=True)
        if os.path.exists(a.out):
            raise SystemExit(f"拒绝覆盖已有文件：{a.out}")
        with open(a.out, "w", encoding="utf-8") as f:
            f.write(text + "\n")
    print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
