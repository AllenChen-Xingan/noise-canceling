"""exp025 · 现场混音，分两段：plan（CPU：抽全部随机数、从原始音频里截片段）→ render（纯算术）。

- plan(rng, speech, noises, ir, knobs, n)：只做「读 / 截 / 抽随机数」，结果是一个 Plan（numpy 数组 + 参数），
  可以在 DataLoader 的 worker 里做
- render(plan)：本文件里的 CPU 参考版（numpy / scipy，float64）
- gpu_mix.render_batch(plans)：torch 批量版，在 GPU 上算同样的步骤；对拍测试要求同一 plan 两边逐样本差 ≤ 1e-4

render 的步骤（每步对应 recipe.json 一个旋钮）：
1. reverb：人声卷整条 RIR → 混响人声；目标 = 卷 rooms.early_rir（直达 + 前 EARLY_REVERB_MS，正本在 exp013 rooms.py）
2. eq：人声一个随机二阶滤波（混响人声与目标同一个），每段噪声各一个
3. n_noises：k 段噪声各自归一到单位功率，第 2 段起乘 U(rel_lo_db, 0) dB，相加
4. snr：按 exp013 dsp.gated_level_db（人声）对噪声整段功率设增益
5. level：整体增益让混响人声电平 = N(mean, std) dBFS（夹在上下限内）；noisy 与 clean 同一增益
6. near_silent：人声置零（clean 全零、owner_ivs 为空）
   high_snr（plan 里最后抽）：clean_input 的段噪声增益为 0（第 3 步的 rel_gain 全零）；high_snr 的段第 4 步用高 SNR
7. 峰值 > 0.99 时 noisy · clean · 人声一起缩
8. clip：只削 noisy，阈值 = u × 当时的 max|noisy|
meta.owner_ivs 用 exp013 dsp.intervals（评测真值同一正本）在第 7 步后、第 8 步前的混响人声上算。
"""
import sys
from dataclasses import dataclass, field
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
MAX_NOISES = 3  # n_noises 的 p 里最大的段数；批量版按这个开槽位


def rand_biquad(rng, r_max):
    """Braun 2020 p.5：H(z) = (1 + r1 z⁻¹ + r2 z⁻²) / (1 + r3 z⁻¹ + r4 z⁻²)，r ~ U(−r_max, r_max)。|r| ≤ 3/8 时极点在单位圆内。"""
    r = rng.uniform(-r_max, r_max, 4)
    return np.array([1.0, r[0], r[1], 1.0, r[2], r[3]])


IDENTITY = np.array([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])  # 不加均衡时的「滤波器」


def power_db(x):
    return 10 * np.log10(np.mean(np.asarray(x, float) ** 2) + EPS)


def to_rir(ir, name="rir"):
    """实测 / 模拟 IR → exp013 的 Rir（直达窗与归一照 rirs.Measured，正本），再切早期部分。返回 (全长, 早期, Measured)。"""
    m = Measured("openslr28", "", name, 0, np.asarray(ir, float), 1.0)
    r, _ = m.at(1.0, 0.0)  # 实测距离 = 请求距离 → 直达增益 0 dB，不改 IR
    return r.full, early_rir(r).full, m


def fit(x, n, rng, pad=False):
    """取 n 个样本：够长就随机起点截；不够时噪声循环拼，人声（pad=True）随机位置补零（留出停顿，不重复同一句）。"""
    x = np.asarray(x, float)
    if len(x) >= n:
        s = int(rng.integers(0, len(x) - n + 1))
        return x[s : s + n]
    if pad:
        out = np.zeros(n)
        s = int(rng.integers(0, n - len(x) + 1))
        out[s : s + len(x)] = x
        return out
    return np.tile(x, int(np.ceil(n / max(len(x), 1))))[:n]


def draw_snr(rng, k, wind):
    if not k["on"]:
        return 5.0
    if k["dist"] == "normal":
        return float(rng.normal(k["mean_db"], k["std_db"]))
    lo = k["wind_lo_db"] if wind else k["lo_db"]
    return float(rng.uniform(lo, k["hi_db"]))


def draw_high_snr(rng, k, snr, silent):
    """high_snr 旋钮：返回 (snr, 是否高信噪比, 是否干净输入)。clean_frac 的段不加噪声，frac 的段 SNR ~ U(lo_db, hi_db)。
    抽中干净但本段 near_silent（人声为 0）时按 near_silent 算，不记干净。"""
    if not (k and k["on"]):
        return snr, False, False
    u = rng.random()
    if u < k["clean_frac"]:
        return snr, False, not silent
    if u < k["clean_frac"] + k["frac"]:
        return float(rng.uniform(k["lo_db"], k["hi_db"])), True, False
    return snr, False, False


def draw_k(rng, k):
    if not k["on"]:
        return 1
    ks = sorted(k["p"], key=int)
    p = np.array([k["p"][x] for x in ks], float)
    return int(ks[int(rng.choice(len(ks), p=p / p.sum()))])


@dataclass
class Plan:
    """一段样本的全部输入与随机参数（render 只用这些，不再抽随机数）。"""
    speech: np.ndarray  # [n] 截好的干声
    noises: np.ndarray  # [MAX_NOISES, n] 截好的噪声；没用到的槽位全零
    k: int  # 用了几段噪声
    rir_full: np.ndarray  # 整条 RIR（不加混响时为 [1.0]）
    rir_early: np.ndarray  # 早期 RIR（同上）
    eq_speech: np.ndarray  # [6] b0 b1 b2 a0 a1 a2
    eq_noise: np.ndarray  # [MAX_NOISES, 6]
    rel_gain: np.ndarray  # [MAX_NOISES] 线性增益，第 0 段为 1，未用槽位为 0
    snr_db: float
    level_dbfs: float
    silent: bool
    clip_u: float  # 削波阈值相对 max|noisy| 的比例；不削为 0
    meta: dict = field(default_factory=dict)


def plan(rng, speech, noises, ir, knobs, n):
    """speech：干声；noises：[(波形, 是否风噪, 来源名)]（不够 k 段就循环用）；ir：RIR 波形或 None。"""
    K = knobs
    meta = {"near_silent": False}
    s = fit(speech, n, rng, pad=True)
    if K["reverb"]["on"] and ir is not None and rng.random() < K["reverb"]["prob"]:
        full, early, m = to_rir(ir)
        meta["reverb"] = {"t60_s": round(float(m.t60), 3), "early_ms": EARLY_REVERB_MS}
    else:
        full = early = np.array([1.0])
        meta["reverb"] = None
    eq = K["eq"]
    eq_s = rand_biquad(rng, eq["r_max"]) if eq["on"] and rng.random() < eq["prob"] else IDENTITY
    k = draw_k(rng, K["n_noises"])
    if k > MAX_NOISES:
        raise ValueError(f"n_noises 最多 {MAX_NOISES} 段（批量版的槽位数），配方给了 {k}")
    ns, eq_n, rel = np.zeros((MAX_NOISES, n)), np.tile(IDENTITY, (MAX_NOISES, 1)), np.zeros(MAX_NOISES)
    wind, srcs = False, []
    for i in range(k):
        x, is_wind, src = noises[i % len(noises)]
        ns[i] = fit(x, n, rng)
        if eq["on"] and rng.random() < eq["prob"]:
            eq_n[i] = rand_biquad(rng, eq["r_max"])
        rel[i] = 1.0 if i == 0 else 10 ** (rng.uniform(K["n_noises"]["rel_lo_db"], 0.0) / 20)
        wind |= bool(is_wind)
        srcs.append(src)
    snr = draw_snr(rng, K["snr"], wind)
    lv = K["level"]
    want = float(np.clip(rng.normal(lv["mean_dbfs"], lv["std_db"]), lv["lo_dbfs"], lv["hi_dbfs"])) if lv["on"] else -28.0
    silent = bool(K["near_silent"]["on"] and rng.random() < K["near_silent"]["frac"])
    cl = K["clip"]
    u = float(rng.uniform(cl["thr_lo"], cl["thr_hi"])) if cl["on"] and rng.random() < cl["prob"] else 0.0
    snr, high, clean = draw_high_snr(rng, K.get("high_snr"), snr, silent)  # 最后才抽：关着时不多抽随机数
    if clean:
        rel = np.zeros(MAX_NOISES)  # 不加噪声：render 里噪声整段为 0
    meta.update({"near_silent": silent, "snr_db": None if clean else round(snr, 2), "level_dbfs": round(want, 2), "n_noises": k,
                 "noises": srcs, "wind": wind, "interf_ivs": [], "high_snr": high, "clean_input": clean})
    return Plan(s, ns, k, np.asarray(full, float), np.asarray(early, float), eq_s, eq_n, rel, snr, want, silent, u, meta)


def _filt(c, x):
    return lfilter(c[:3], c[3:], x)


def render(p, n=None):
    """CPU 参考版：返回 noisy, clean（float32）和 meta（含 owner_ivs · clip_thr）。"""
    n = len(p.speech) if n is None else n
    rev = fftconvolve(p.speech, p.rir_full)[:n]
    tgt = fftconvolve(p.speech, p.rir_early)[:n]
    rev, tgt = _filt(p.eq_speech, rev), _filt(p.eq_speech, tgt)
    noise = np.zeros(n)
    for i in range(p.k):
        y = _filt(p.eq_noise[i], p.noises[i])
        noise += p.rel_gain[i] * y / np.sqrt(np.mean(y**2) + EPS)
    noise = noise * 10 ** ((gated_level_db(rev) - p.snr_db - power_db(noise)) / 20)
    g = 10 ** ((p.level_dbfs - gated_level_db(rev)) / 20)
    rev, tgt, noise = rev * g, tgt * g, noise * g
    if p.silent:
        rev, tgt = np.zeros(n), np.zeros(n)
    noisy = rev + noise
    peak = max(np.max(np.abs(noisy)), np.max(np.abs(tgt)), EPS)
    if peak > PEAK:
        noisy, tgt, rev = noisy * PEAK / peak, tgt * PEAK / peak, rev * PEAK / peak
    meta = dict(p.meta, owner_ivs=[] if p.silent else intervals(rev), clip_thr=None)
    if p.clip_u > 0:
        thr = p.clip_u * np.max(np.abs(noisy))
        noisy = np.clip(noisy, -thr, thr)
        meta["clip_thr"] = round(float(thr), 5)
    return noisy.astype(np.float32), tgt.astype(np.float32), meta


def make_example(rng, speech, noises, ir, knobs, n):
    """CPU 一步到位（旧接口；对拍与本机冒烟用）。"""
    return render(plan(rng, speech, noises, ir, knobs, n), n)
