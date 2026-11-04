"""exp013 · 通用信号小工具：电平、活动区间、最小相位 FIR、混响时间、样本哈希。

两种电平函数**故意分开写**（016 S2③：实测不能用设增益的同一个函数自证）：
- `gated_level_db`：生成时设增益用。20 ms 帧，取能量高于全段最大帧 −35 dB 的帧求 RMS
- `p56_level_db`：自检实测用。ITU-T P.56 方法 B 的活动语音电平（包络 + 门限 + 15.9 dB 余量）
"""
import hashlib

import numpy as np
from scipy import signal

SR = 16000
FRAME = 320  # 20 ms
EPS = 1e-12


def rms(x):
    x = np.asarray(x, np.float64)
    return float(np.sqrt(np.mean(x * x) + EPS)) if x.size else EPS


def db(v):
    return 20.0 * np.log10(max(float(v), EPS))


def frame_energy(x, frame=FRAME):
    n = len(x) // frame
    if n == 0:
        return np.zeros(0)
    f = np.asarray(x[: n * frame], np.float64).reshape(n, frame)
    return np.mean(f * f, axis=1)


def gated_level_db(x, rel_db=-35.0):
    """生成用：活动帧（> 最大帧 rel_db）的 RMS，dBFS。全静音返回 −inf 量级。"""
    e = frame_energy(x)
    if e.size == 0 or e.max() <= EPS:
        return -240.0
    act = e > e.max() * 10 ** (rel_db / 10)
    return 10 * np.log10(np.mean(e[act]) + EPS)


def p56_level_db(x, sr=SR):
    """实测用：ITU-T P.56 方法 B 活动语音电平（dBFS，满幅正弦 = 0 dBFS 口径换成 RMS 口径：满幅方波 = 0）。"""
    x = np.asarray(x, np.float64)
    if x.size == 0 or np.max(np.abs(x)) < 1e-9:
        return -240.0
    g = np.exp(-1.0 / (sr * 0.03))
    # 两级一阶平滑的整流包络
    p = signal.lfilter([1 - g], [1, -g], np.abs(x))
    q = signal.lfilter([1 - g], [1, -g], p)
    hang = int(0.2 * sr)
    energy = np.mean(x * x)
    thr_db = np.arange(-100.0, 1.0, 2.0) + 20 * np.log10(np.max(np.abs(x)))
    act_db, diff = [], []
    for t in thr_db:
        c = 10 ** (t / 20)
        above = q >= c
        if not above.any():
            act_db.append(-240.0)
            diff.append(-np.inf)
            continue
        # 超过门限后保持 hang 个样本
        idx = np.flatnonzero(above)
        mask = np.zeros(len(x), bool)
        starts = idx[np.r_[True, np.diff(idx) > hang]]
        ends = idx[np.r_[np.diff(idx) > hang, True]]
        for s, e in zip(starts, ends):
            mask[s : min(len(x), e + hang)] = True
        frac = mask.mean()
        a = 10 * np.log10(energy / max(frac, EPS) + EPS)
        act_db.append(a)
        diff.append(a - t)
    act_db, diff = np.array(act_db), np.array(diff)
    m = 15.9
    ok = np.flatnonzero(diff >= m)
    if ok.size == 0:
        return float(act_db[0])
    j = ok[-1]
    if j + 1 < len(diff) and np.isfinite(diff[j + 1]):
        # 在相邻两个门限间按差值线性插值
        w = (diff[j] - m) / max(diff[j] - diff[j + 1], EPS)
        return float(act_db[j] + w * (act_db[j + 1] - act_db[j]))
    return float(act_db[j])


def intervals(x, rel_db=-35.0, guard=2, frame=FRAME):
    """016 §4 区间规则：20 ms 帧能量 > 全段最大帧 rel_db 为活动；两端各外扩 guard 帧。返回 [[起, 止), …] 样本号。"""
    e = frame_energy(x, frame)
    if e.size == 0 or e.max() <= EPS:
        return []
    act = e > e.max() * 10 ** (rel_db / 10)
    if guard:
        k = np.ones(2 * guard + 1)
        act = np.convolve(act.astype(float), k, mode="same") > 0
    out, i, n = [], 0, len(act)
    while i < n:
        if act[i]:
            j = i
            while j < n and act[j]:
                j += 1
            out.append([i * frame, min(len(x), j * frame)])
            i = j
        else:
            i += 1
    return out


def iv_union(*ivs):
    pts = sorted([list(v) for iv in ivs for v in iv])
    out = []
    for s, e in pts:
        if out and s <= out[-1][1]:
            out[-1][1] = max(out[-1][1], e)
        else:
            out.append([s, e])
    return out


def iv_len(iv):
    return int(sum(e - s for s, e in iv))


def iv_overlap(a, b):
    tot, i, j = 0, 0, 0
    while i < len(a) and j < len(b):
        s, e = max(a[i][0], b[j][0]), min(a[i][1], b[j][1])
        tot += max(0, e - s)
        if a[i][1] < b[j][1]:
            i += 1
        else:
            j += 1
    return tot


def minphase_fir(freqs_hz, gains_db, taps=512, sr=SR, nfft=8192):
    """按幅度曲线（频率点上的 dB，点间按对数频率线性插值）设计最小相位 FIR（倒谱法）。"""
    f = np.linspace(0, sr / 2, nfft // 2 + 1)
    lf = np.log10(np.maximum(f, 1.0))
    g = np.interp(lf, np.log10(np.maximum(freqs_hz, 1.0)), gains_db)
    mag = 10 ** (g / 20)
    logm = np.log(np.maximum(mag, 1e-6))
    cep = np.fft.irfft(logm, nfft)
    w = np.zeros(nfft)
    w[0] = 1
    w[1 : nfft // 2] = 2
    w[nfft // 2] = 1
    h = np.fft.irfft(np.exp(np.fft.rfft(cep * w)), nfft)[:taps]
    fade = taps // 8
    h[-fade:] *= np.hanning(2 * fade)[fade:]
    return h


def schroeder_t60(h, sr=SR):
    """Schroeder 反向积分；用 −5 到 −25 dB 段（T20）外推到 60 dB。太短或无衰减返回 0。"""
    h = np.asarray(h, np.float64)
    if h.size < 8:
        return 0.0
    e = np.cumsum((h * h)[::-1])[::-1]
    if e[0] <= EPS:
        return 0.0
    edc = 10 * np.log10(e / e[0] + EPS)
    i5 = np.flatnonzero(edc <= -5)
    i25 = np.flatnonzero(edc <= -25)
    if i5.size == 0 or i25.size == 0 or i25[0] - i5[0] < 4:
        return 0.0
    t = np.arange(i5[0], i25[0]) / sr
    slope = np.polyfit(t, edc[i5[0] : i25[0]], 1)[0]
    t60 = float(-60.0 / slope) if slope < 0 else 0.0
    return t60 if t60 < 30.0 else 0.0  # 只有零星脉冲（如户外地面反射）时斜率近 0，按无混响算


def sample_hash(x):
    """样本内容哈希（float32 小端字节），不是文件哈希（016 S1）。"""
    return hashlib.sha256(np.asarray(x, "<f4").tobytes()).hexdigest()


def fit(x, n, rng=None):
    """截 / 循环到 n 个样本；给了 rng 就随机起点。"""
    x = np.asarray(x, np.float64)
    if len(x) == 0:
        return np.zeros(n)
    if len(x) < n:
        x = np.tile(x, int(np.ceil(n / len(x))) + 1)
    s = int(rng.integers(0, len(x) - n + 1)) if rng is not None else 0
    return x[s : s + n]


def resample(x, sr_from, sr_to=SR):
    if sr_from == sr_to:
        return np.asarray(x, np.float64)
    from math import gcd

    k = gcd(int(sr_from), int(sr_to))
    return signal.resample_poly(np.asarray(x, np.float64), sr_to // k, sr_from // k)
