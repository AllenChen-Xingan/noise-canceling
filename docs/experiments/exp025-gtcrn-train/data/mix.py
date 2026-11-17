"""exp025 · 现场混音：一段人声 + 1–3 段噪声 + 一条 RIR → (noisy, clean, meta)。旋钮全从 recipe.json 读。

顺序（每步对应 recipe.json 一个旋钮）：
1. reverb：人声卷整条 RIR → 混响人声；目标 = 卷 rooms.early_rir（直达 + 前 EARLY_REVERB_MS，正本在 exp013 rooms.py）
2. eq：人声一个随机二阶滤波（混响人声与目标同一个），每段噪声各一个
3. n_noises：叠 k 段噪声，第 2 段起相对第 1 段 U(rel_lo_db, 0) dB
4. snr：按 exp013 dsp.gated_level_db（人声）对噪声整段功率设增益
5. level：整体增益让混响人声电平 = N(mean, std) dBFS（夹在上下限内）；noisy 与 clean 同一增益；之后峰值 > 0.99 时一起缩
6. clip：只削 noisy
7. near_silent：人声置零（clean 全零、owner_ivs 为空）
meta.owner_ivs 用 exp013 dsp.intervals（评测真值同一正本）在第 5 步后、第 6 步前的混响人声上算。
"""
import sys
from pathlib import Path

import numpy as np
from scipy.signal import fftconvolve, lfilter

HERE = Path(__file__).resolve().parent
EXP013 = HERE.parents[1] / "exp013-scene-suite"
sys.path.insert(0, str(EXP013))
from dsp import gated_level_db, intervals  # noqa: E402
from rooms import EARLY_REVERB_MS, early_rir  # noqa: E402
from rirs import Measured  # noqa: E402

EPS = 1e-12
PEAK = 0.99


def rand_biquad(rng, r_max):
    """Braun 2020 p.5：H(z) = (1 + r1 z⁻¹ + r2 z⁻²) / (1 + r3 z⁻¹ + r4 z⁻²)，r ~ U(−r_max, r_max)。|r| ≤ 3/8 时极点在单位圆内。"""
    r = rng.uniform(-r_max, r_max, 4)
    return np.array([1.0, r[0], r[1]]), np.array([1.0, r[2], r[3]])


def power_db(x):
    return 10 * np.log10(np.mean(np.asarray(x, float) ** 2) + EPS)


def to_rir(ir, name="rir"):
    """实测 / 模拟 IR → exp013 的 Rir（直达窗与归一照 rirs.Measured，正本），再切早期部分。返回 (全长, 早期, Measured)。"""
    m = Measured("openslr28", "", name, 0, np.asarray(ir, float), 1.0)
    r, _ = m.at(1.0, 0.0)  # 实测距离 = 请求距离 → 直达增益 0 dB，不改 IR
    return r.full, early_rir(r).full, m


def fit(x, n, rng):
    """取 n 个样本：够长就随机起点截，不够就循环拼。"""
    x = np.asarray(x, float)
    if len(x) >= n:
        s = int(rng.integers(0, len(x) - n + 1))
        return x[s : s + n]
    return np.tile(x, int(np.ceil(n / max(len(x), 1))))[:n]


def draw_snr(rng, k, wind):
    if not k["on"]:
        return 5.0
    if k["dist"] == "normal":
        return float(rng.normal(k["mean_db"], k["std_db"]))
    lo = k["wind_lo_db"] if wind else k["lo_db"]
    return float(rng.uniform(lo, k["hi_db"]))


def draw_k(rng, k):
    if not k["on"]:
        return 1
    ks = sorted(k["p"], key=int)
    p = np.array([k["p"][x] for x in ks], float)
    return int(ks[int(rng.choice(len(ks), p=p / p.sum()))])


def make_example(rng, speech, noises, ir, knobs, n):
    """speech：干声人声（≥ 1 个样本，16 kHz）；noises：[(波形, 是否风噪, 来源名)]，至少 k 段（不够就循环用）；
    ir：RIR 波形或 None。返回 noisy, clean（float32，长 n）和 meta。"""
    K = knobs
    meta = {"near_silent": False}
    s = fit(speech, n, rng)
    if K["reverb"]["on"] and ir is not None and rng.random() < K["reverb"]["prob"]:
        full, early, m = to_rir(ir)
        rev, tgt = fftconvolve(s, full)[:n], fftconvolve(s, early)[:n]
        meta["reverb"] = {"t60_s": round(float(m.t60), 3), "early_ms": EARLY_REVERB_MS}
    else:
        rev, tgt = s.copy(), s.copy()
        meta["reverb"] = None
    eq = K["eq"]
    if eq["on"] and rng.random() < eq["prob"]:
        b, a = rand_biquad(rng, eq["r_max"])
        rev, tgt = lfilter(b, a, rev), lfilter(b, a, tgt)
    k = draw_k(rng, K["n_noises"])
    parts, wind, srcs = [], False, []
    for i in range(k):
        x, is_wind, src = noises[i % len(noises)]
        y = fit(x, n, rng)
        if eq["on"] and rng.random() < eq["prob"]:
            b, a = rand_biquad(rng, eq["r_max"])
            y = lfilter(b, a, y)
        y = y / np.sqrt(np.mean(y**2) + EPS)
        if i:
            y = y * 10 ** (rng.uniform(K["n_noises"]["rel_lo_db"], 0.0) / 20)
        parts.append(y)
        wind |= bool(is_wind)
        srcs.append(src)
    noise = np.sum(parts, axis=0)
    snr = draw_snr(rng, K["snr"], wind)
    g_n = 10 ** ((gated_level_db(rev) - snr - power_db(noise)) / 20)
    noise = noise * g_n
    lv = K["level"]
    want = float(np.clip(rng.normal(lv["mean_dbfs"], lv["std_db"]), lv["lo_dbfs"], lv["hi_dbfs"])) if lv["on"] else -28.0
    g = 10 ** ((want - gated_level_db(rev)) / 20)
    rev, tgt, noise = rev * g, tgt * g, noise * g
    silent = K["near_silent"]["on"] and rng.random() < K["near_silent"]["frac"]
    if silent:
        rev, tgt = np.zeros(n), np.zeros(n)
        meta["near_silent"] = True
    noisy = rev + noise
    peak = max(np.max(np.abs(noisy)), np.max(np.abs(tgt)), EPS)
    if peak > PEAK:
        noisy, tgt, rev = noisy * PEAK / peak, tgt * PEAK / peak, rev * PEAK / peak
    owner_ivs = [] if silent else intervals(rev)
    cl = K["clip"]
    meta["clip_thr"] = None
    if cl["on"] and rng.random() < cl["prob"]:
        thr = rng.uniform(cl["thr_lo"], cl["thr_hi"]) * np.max(np.abs(noisy))
        noisy = np.clip(noisy, -thr, thr)
        meta["clip_thr"] = round(float(thr), 5)
    meta.update({"snr_db": round(snr, 2), "level_dbfs": round(want, 2), "n_noises": k, "noises": srcs, "wind": wind,
                 "owner_ivs": owner_ivs, "interf_ivs": []})
    return noisy.astype(np.float32), tgt.astype(np.float32), meta
