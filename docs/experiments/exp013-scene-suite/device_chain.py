"""exp013 · 设备链模拟（022 simulation.md 第二节 9 步）。参数只从 device_chain.json 读（正本）。

用法（compose.py 调）：
    ch = Chain(rng, "mba-builtin", tier)          # 每段抽一次链参数，ch.params 进清单
    h, off, att = ch.source_rir(rir, rng)         # 第 1–2 步：有方向的源（直达部分按方位衰减）
    y = ch.diffuse(x)                             # 第 2 步：扩散声（底噪、babble）只做扩散场衰减
    y = ch.response(y)                            # 第 3 步：频响 FIR（每条轨都过，线性）
    fl = ch.floor(n, owner_level_db, rng)         # 第 4 步：处理后底噪（单独一轨）
    g = ch.level_gain_db(owner_level_db)          # 第 6–7 步：整段增益
    mix, clipped = ch.clip(mix)                   # 第 8 步：削波（只压力档、概率抽）
`external-mic` 只做第 1、6、9 步：source_rir 不衰减、diffuse / response 原样返回、floor 为 0、不削波。
"""
import json
from pathlib import Path

import numpy as np
from scipy import signal

from dsp import SR, minphase_fir

HERE = Path(__file__).resolve().parent
P = json.loads((HERE / "device_chain.json").read_text())
PAD = 256  # 零相位频域滤波的前置余量（样本），所有源一致，卷积后丢掉


def u(rng, r):
    return float(rng.uniform(r[0], r[1]))


def band_gain(freqs, low, mid, high, edges=None):
    """三频段 dB 增益，频段边界处用 1/3 倍频程余弦过渡。"""
    e1, e2 = edges or P["step2_direction"]["bands_hz"]
    lf = np.log2(np.maximum(freqs, 1.0))

    def step(edge):  # 0 → 1，跨过 edge 的 1/3 倍频程
        x = np.clip((lf - np.log2(edge)) / (1 / 3) + 0.5, 0, 1)
        return 0.5 - 0.5 * np.cos(np.pi * x)

    s1, s2 = step(e1), step(e2)
    return low * (1 - s1) + mid * (s1 - s2) + high * s2


def azimuth_atten(rng, az_deg, strength):
    """方位衰减表（022 第 2 步）：0–30° 不衰减；30→90→180° 线性插值；左右对称。三频段各抽各的。"""
    d = P["step2_direction"]
    a = abs(((az_deg + 180) % 360) - 180)
    t90, t180 = d["azimuth_table"]["90"], d["azimuth_table"]["180"]
    out = {}
    for b in ("low", "mid", "high"):
        v90, v180 = u(rng, t90[b]), u(rng, t180[b])
        flat = d["front_flat_deg"]
        if a <= flat:
            v = 0.0
        elif a <= 90:
            v = v90 * (a - flat) / (90 - flat)
        else:
            v = v90 + (v180 - v90) * (a - 90) / 90
        out[b] = v * strength
    return out


def zero_phase(h, gain_db_fn):
    """对短序列做零相位频域滤波；输出前面多 PAD 个样本。"""
    n = len(h) + 2 * PAD
    nfft = 1 << int(np.ceil(np.log2(n)))
    x = np.zeros(nfft)
    x[PAD : PAD + len(h)] = h
    f = np.fft.rfftfreq(nfft, 1 / SR)
    y = np.fft.irfft(np.fft.rfft(x) * 10 ** (gain_db_fn(f) / 20), nfft)
    return y[: len(h) + 2 * PAD]


def peaking_db(f, f0, gain_db, q):
    """RBJ 峰值滤波器的幅度响应（dB）。"""
    b, a = _peaking_ba(f0, gain_db, q)
    _, hz = signal.freqz(b, a, worN=np.asarray(f), fs=SR)
    return 20 * np.log10(np.maximum(np.abs(hz), 1e-9))


def _peaking_ba(f0, gain_db, q):
    A = 10 ** (gain_db / 40)
    w = 2 * np.pi * f0 / SR
    al = np.sin(w) / (2 * q)
    b = np.array([1 + al * A, -2 * np.cos(w), 1 - al * A])
    a = np.array([1 + al / A, -2 * np.cos(w), 1 - al / A])
    return b / a[0], a / a[0]


def response_curve(params, freqs):
    """第 3 步的目标幅度曲线（dB，相对 1 kHz）。"""
    s3 = P["step3_response"]
    f = np.asarray(freqs, float)
    if params["flat"]:
        _, hz = signal.freqz(*signal.butter(2, s3["flat_highpass_hz"], "highpass", fs=SR), worN=np.maximum(f, 1.0), fs=SR)
        g = 20 * np.log10(np.maximum(np.abs(hz), 1e-9))
    else:
        pts = []
        for lo, hi, v in s3["curve_db"]:
            pts += [(lo, v)] if lo == hi else [(lo, v), (hi, v)]
        pf, pv = np.array([p[0] for p in pts], float), np.array([p[1] for p in pts], float)
        lf = np.log2(np.maximum(f, 1.0))
        g = np.interp(lf, np.log2(pf), pv)
        below = f < pf[0]  # 80 Hz 以下按 80–100 Hz 的斜率外推，最低 −40 dB
        slope = (pv[1] - pv[0]) / (np.log2(pf[1]) - np.log2(pf[0]))
        g[below] = np.maximum(pv[0] + slope * (lf[below] - np.log2(pf[0])), -40.0)
    g = g + params["tilt_db_per_oct"] * np.log2(np.maximum(f, 1.0) / 1000.0)
    g = g + peaking_db(np.maximum(f, 1.0), params["peak_hz"], params["peak_db"], params["peak_q"])
    return g


class Chain:
    def __init__(self, rng, device, tier):
        if device not in P["devices"]:
            raise ValueError(f"未知设备键 {device}")
        self.device = device
        self.steps = set(P["devices"][device]["steps"])
        s2, s3, s4, s6, s7, s8 = (P[k] for k in ("step2_direction", "step3_response", "step4_floor", "step6_level", "step7_vpio", "step8_clip"))
        p = {"device": device, "mic_mode": P["devices"][device]["mic_mode"]}
        lvl = float(np.clip(rng.normal(s6["mean_dbfs"], s6["sigma_db"]), *s6["clip_range"]))
        p["owner_level_dbfs"] = lvl
        if 2 in self.steps:
            p["strength"] = u(rng, s2["strength"]["range"])
            p["reverb_stress"] = bool(rng.random() < s2["reverb_stress_prob"])
        if 3 in self.steps:
            p["flat"] = bool(rng.random() < s3["flat_prob"])
            p["tilt_db_per_oct"] = u(rng, s3["tilt_db_per_oct"])
            p["peak_hz"] = float(np.exp(u(rng, np.log(s3["peak"]["freq_hz"]))))
            p["peak_hz"] = min(p["peak_hz"], 0.45 * SR)  # 16 kHz 下峰值频率不过 7.2 kHz
            p["peak_db"] = u(rng, s3["peak"]["gain_db"])
            p["peak_q"] = u(rng, s3["peak"]["q"])
        if 4 in self.steps:
            p["floor_ratio_db"] = u(rng, s4["ratio_db"]["range"])
            p["floor_lowmid_excess_db"] = u(rng, s4["lowmid_excess_db"])
        p["vpio_db"] = u(rng, s7["gain_db"]) if (7 in self.steps and s7["enabled"] and rng.random() < s7["prob"]) else 0.0
        p["clip_drive_db"] = u(rng, s8["drive_db"]) if (8 in self.steps and tier in s8["tiers"] and rng.random() < s8["prob"]) else None
        self.params = p
        self._fir = minphase_fir(*self._curve_pts(), taps=s3["fir_taps"]) if 3 in self.steps else None

    def _curve_pts(self):
        f = np.geomspace(20, SR / 2, 256)
        return f, response_curve(self.params, f)

    # ---- 第 1–2 步 ----
    def source_rir(self, rir, rng):
        """有方向的源：直达部分按方位衰减，其余按扩散场近似（混响压力分支则不衰减）。返回 (h, PAD)。"""
        if 2 not in self.steps:
            return np.concatenate([np.zeros(PAD), rir.full, np.zeros(PAD)]), PAD, {}
        att = azimuth_atten(rng, rir.azimuth_deg, self.params["strength"])
        hd = zero_phase(rir.direct, lambda f: band_gain(f, att["low"], att["mid"], att["high"]))
        hr = rir.rest if self.params["reverb_stress"] else None
        if hr is None:
            dz = P["step2_direction"]["diffuse_db"]
            hr = zero_phase(rir.rest, lambda f: band_gain(f, dz["low"], dz["mid"], dz["high"]))
        else:
            hr = np.concatenate([np.zeros(PAD), hr, np.zeros(PAD)])
        return hd + hr, PAD, att

    def diffuse(self, x):
        if 2 not in self.steps:
            return np.asarray(x, float)
        dz = P["step2_direction"]["diffuse_db"]
        n = len(x)
        nfft = 1 << int(np.ceil(np.log2(n + 1)))
        f = np.fft.rfftfreq(nfft, 1 / SR)
        y = np.fft.irfft(np.fft.rfft(x, nfft) * 10 ** (band_gain(f, dz["low"], dz["mid"], dz["high"]) / 20), nfft)
        return y[:n]

    # ---- 第 3 步 ----
    def response(self, x):
        if self._fir is None:
            return np.asarray(x, float)
        return signal.fftconvolve(x, self._fir)[: len(x)]

    # ---- 第 4 步 ----
    def floor(self, n, owner_level_db, rng):
        if 4 not in self.steps:
            return np.zeros(n)
        s4 = P["step4_floor"]
        nfft = 1 << int(np.ceil(np.log2(n)))
        f = np.fft.rfftfreq(nfft, 1 / SR)
        lo, hi = s4["lowmid_band_hz"]
        ex = self.params["floor_lowmid_excess_db"]
        # 200–1000 Hz 比 2 kHz 以上高 ex dB；中间对数频率线性过渡
        lf = np.log2(np.maximum(f, 1.0))
        g = np.interp(lf, np.log2([lo, hi, s4["high_band_hz"]]), [ex, ex, 0.0])
        w = np.fft.irfft(np.fft.rfft(rng.standard_normal(nfft)) * 10 ** (g / 20), nfft)[:n]
        w /= np.sqrt(np.mean(w * w))
        return w * 10 ** ((owner_level_db - self.params["floor_ratio_db"]) / 20)

    # ---- 第 6–8 步 ----
    def level_gain_db(self, owner_level_db):
        return self.params["owner_level_dbfs"] - owner_level_db + self.params["vpio_db"]

    def clip(self, mix):
        d = self.params["clip_drive_db"]
        if d is None:
            return mix, False
        g = 10 ** (d / 20) / max(np.max(np.abs(mix)), 1e-9)
        return np.clip(mix * g, -1.0, 1.0) / g, True
