"""exp013 · 自写合成声音（只评测；不调用 exp006 `sim.py` 的任何取材 / 合成函数，016 S10⑤）。

两类接口：
- 连续声（底噪 / 有规律）：`fn(rng, n) -> (x, params)`，长 n 个样本
- 突发声：`fn(rng) -> (x, params)`，长度由声音自己定
另有语音外放滤波 `speaker(x, rng, style)`（电视 / 导航 / 广播 / 隔墙）与广播前的提示音 `pa_chime`。
电平都不管（compose 统一设）；params 进清单，便于复现与按参数拆分。
"""
import numpy as np
from scipy import signal

from dsp import SR


def _u(rng, a, b):
    return float(rng.uniform(a, b))


def _sos(kind, f, order=2):
    return signal.butter(order, f, btype=kind, fs=SR, output="sos")


def _filt(x, kind, f, order=2):
    return signal.sosfilt(_sos(kind, f, order), x)


def _colored(rng, n, slope_db_oct):
    """功率谱斜率 slope dB/倍频程的噪声（0 = 白，−3 = 粉，−6 = 布朗）。"""
    nfft = 1 << int(np.ceil(np.log2(n + 1)))
    f = np.fft.rfftfreq(nfft, 1 / SR)
    # 20 Hz 以下按 20 Hz 的增益封顶、10 Hz 以下置零：否则 −3 / −6 dB 斜率会把亚音频漂移放大成几十 dB 的峰值因数
    g = (np.maximum(f, 20.0) / 1000.0) ** (slope_db_oct / (20 * np.log10(2)))
    g[f < 10.0] = 0
    x = np.fft.irfft(np.fft.rfft(rng.standard_normal(nfft)) * g, nfft)[:n]
    return x / (np.std(x) + 1e-12)


def _slow_env(rng, n, rate_hz, depth_db):
    """平滑随机包络（dB 深度），用于缓慢起伏。"""
    k = max(2, int(n / SR * rate_hz) + 2)
    pts = rng.uniform(-1, 1, k)
    e = np.interp(np.linspace(0, k - 1, n), np.arange(k), pts)
    return 10 ** (depth_db * e / 20)


def _tones(n, f0s, amps, phases=None, jitter=None, rng=None):
    t = np.arange(n) / SR
    x = np.zeros(n)
    for i, (f, a) in enumerate(zip(f0s, amps)):
        ph = 0 if phases is None else phases[i]
        if jitter is not None:
            fm = f * (1 + jitter * _slow_env(rng, n, 0.5, 1.0) - jitter)
            x += a * np.sin(2 * np.pi * np.cumsum(fm) / SR + ph)
        else:
            x += a * np.sin(2 * np.pi * f * t + ph)
    return x


def _ramp(x, ms=10):
    k = min(len(x) // 2, int(ms * 1e-3 * SR))
    if k > 0:
        w = np.hanning(2 * k)
        x[:k] *= w[:k]
        x[-k:] *= w[k:]
    return x


# ---------------- 连续声 ----------------
def room_tone(rng, n):
    lp = _u(rng, 1500, 4000)
    x = _filt(_colored(rng, n, -4.5), "lowpass", lp)
    hum = rng.random() < 0.5
    mains = int(rng.choice([50, 60]))
    if hum:
        x = x / np.std(x) + 0.15 * _tones(n, [mains, 2 * mains, 3 * mains], [1, 0.5, 0.25], rng.uniform(0, 6, 3))
    return x, {"lowpass_hz": lp, "hum_hz": mains if hum else None}


def hvac(rng, n):
    x = _colored(rng, n, -5.0)
    x = _filt(x, "lowpass", _u(rng, 800, 2000)) + 0.6 * _filt(_colored(rng, n, 0), "bandpass", (120, 600))
    bp = _u(rng, 40, 120)
    x = x / np.std(x) + 0.1 * _tones(n, [bp, 2 * bp], [1, 0.4], rng.uniform(0, 6, 2))
    return x * _slow_env(rng, n, 0.2, 1.0), {"blade_hz": bp}


def fan(rng, n):
    rot = _u(rng, 12, 25)
    blades = int(rng.integers(3, 8))
    bpf = rot * blades
    x = _filt(_colored(rng, n, -2.0), "bandpass", (150, _u(rng, 2500, 5000)))
    t = np.arange(n) / SR
    x = x * (1 + 0.15 * np.sin(2 * np.pi * rot * t))
    x = x / np.std(x) + 0.3 * _tones(n, [bpf, 2 * bpf, 3 * bpf], [1, 0.5, 0.3], rng.uniform(0, 6, 3))
    return x, {"rot_hz": rot, "blades": blades}


def fridge_hum(rng, n):
    mains = int(rng.choice([50, 60]))
    x = _tones(n, [mains * k for k in (1, 2, 3, 4, 6)], [0.4, 1.0, 0.5, 0.3, 0.15], rng.uniform(0, 6, 5), jitter=0.002, rng=rng)
    x = x + 0.2 * _filt(_colored(rng, n, -3), "lowpass", 1500)
    return x, {"mains_hz": mains}


def hvac_cycle(rng, n):
    x, p = hvac(rng, n)
    t_sw = _u(rng, 2.0, 9.0)
    turn_on = bool(rng.random() < 0.5)
    ramp = int(_u(rng, 0.3, 1.0) * SR)
    g = np.zeros(n) if turn_on else np.ones(n)
    k = int(t_sw * SR)
    seg = np.linspace(0, 1, ramp) if turn_on else np.linspace(1, 0, ramp)
    g[k : k + ramp] = seg[: max(0, min(ramp, n - k))]
    g[k + ramp :] = 1.0 if turn_on else 0.0
    # 启动 / 停机时的一声继电器 / 风门碰撞
    click = np.zeros(n)
    if k < n - 200:
        click[k : k + 200] = rng.standard_normal(200) * np.exp(-np.arange(200) / 30) * 3
    return x * g + _filt(click, "highpass", 500), {**p, "switch_s": t_sw, "turn_on": turn_on}


def coffee_machine(rng, n):
    """磨豆（电机谐波 + 豆粒碎裂的颗粒噪声）与蒸汽（高频嘶声带抖动）各占一段，其余安静。"""
    x = np.zeros(n)
    segs = []
    t0 = _u(rng, 0.5, 3.0)
    for kind in rng.permutation(["grind", "steam"])[: int(rng.integers(1, 3))]:
        d = _u(rng, 2.0, 5.0)
        a, b = int(t0 * SR), min(n, int((t0 + d) * SR))
        if b - a < SR // 2:
            break
        m = b - a
        if kind == "grind":
            f0 = _u(rng, 140, 260)
            motor = _tones(m, [f0 * k for k in (1, 2, 3, 4)], [1, 0.6, 0.4, 0.2], rng.uniform(0, 6, 4), jitter=0.01, rng=rng)
            grain = _filt(rng.standard_normal(m), "bandpass", (1000, 6000))
            grain *= np.abs(_filt(rng.standard_normal(m), "lowpass", _u(rng, 20, 40))) * 3
            y = motor / np.std(motor) + grain / (np.std(grain) + 1e-12)
        else:
            y = _filt(rng.standard_normal(m), "bandpass", (2000, 7500))
            y *= 1 + 0.4 * _filt(rng.standard_normal(m), "lowpass", 15)
        x[a:b] += _ramp(y, 80)
        segs.append({"kind": str(kind), "start_s": t0, "dur_s": (b - a) / SR})
        t0 += d + _u(rng, 0.5, 2.0)
    return x, {"segments": segs}


def wind(rng, n):
    base = _filt(_colored(rng, n, -6.0), "lowpass", _u(rng, 150, 400))
    gust = np.maximum(_filt(rng.standard_normal(n), "lowpass", _u(rng, 0.3, 1.0)), 0)
    gust = gust / (gust.max() + 1e-12)
    x = base * (0.2 + 2.0 * gust**1.5)
    return x, {"kind": "gusty-lowfreq"}


def cabin(rng, n):
    kind = str(rng.choice(["air", "rail"]))
    if kind == "air":
        x = _colored(rng, n, -2.5)
        x = _filt(x, "lowpass", _u(rng, 3000, 6000)) + 0.8 * _filt(_colored(rng, n, -3), "bandpass", (80, 400))
        f = _u(rng, 90, 180)
        x = x / np.std(x) + 0.05 * _tones(n, [f, 2 * f], [1, 0.5], rng.uniform(0, 6, 2), jitter=0.003, rng=rng)
    else:
        x = _filt(_colored(rng, n, -5.5), "lowpass", _u(rng, 1500, 3000))
        joint = np.zeros(n)
        period = _u(rng, 0.6, 1.2)
        for k in np.arange(_u(rng, 0, period), n / SR, period):  # 钢轨接缝的「咔哒」
            i = int(k * SR)
            joint[i : i + 400] += np.exp(-np.arange(min(400, n - i)) / 80)
        x = x / np.std(x) + 0.8 * _filt(joint * rng.standard_normal(n), "lowpass", 600)
    return x * _slow_env(rng, n, 0.3, 1.5), {"kind": kind}


# ---------------- 突发声 ----------------
def dishes(rng):
    hits = int(rng.integers(1, 4))
    out = np.zeros(int(_u(rng, 0.6, 1.5) * SR))
    for _ in range(hits):
        m = int(_u(rng, 0.08, 0.35) * SR)
        f = rng.uniform(1800, 7500, int(rng.integers(4, 9)))
        dec = rng.uniform(0.02, 0.2, len(f))
        t = np.arange(m) / SR
        y = sum(np.sin(2 * np.pi * fi * t + rng.uniform(0, 6)) * np.exp(-t / di) for fi, di in zip(f, dec))
        y[:40] += rng.standard_normal(40) * 2
        i = int(rng.integers(0, max(1, len(out) - m)))
        out[i : i + m] += y[: len(out) - i]
    return out, {"hits": hits}


def phone_ring(rng):
    if rng.random() < 0.5:  # 双音振铃 2 s 响 + 1 s 停，再响一次
        on = _tones(int(2.0 * SR), [440, 480], [1, 1]) * (1 + np.sign(np.sin(2 * np.pi * 20 * np.arange(int(2.0 * SR)) / SR))) / 2
        x = np.concatenate([on, np.zeros(SR), on[: int(_u(rng, 0.5, 2.0) * SR)]])
        return _ramp(x, 5), {"kind": "dual-tone"}
    notes = rng.choice([523, 587, 659, 784, 880, 988], int(rng.integers(4, 8)))
    d = _u(rng, 0.12, 0.22)
    seq = [np.sin(2 * np.pi * f * np.arange(int(d * SR)) / SR) * np.exp(-np.arange(int(d * SR)) / (0.08 * SR)) for f in notes]
    x = np.concatenate(seq * 2)
    return _ramp(x, 5), {"kind": "melody", "notes": [int(v) for v in notes]}


def notify(rng):
    notes = rng.choice([880, 1047, 1175, 1319, 1568], int(rng.integers(2, 4)), replace=False)
    d = _u(rng, 0.08, 0.2)
    m = int(d * SR)
    t = np.arange(int(0.5 * SR)) / SR
    x = np.zeros(m * len(notes) + len(t))
    for k, f in enumerate(notes):
        y = (np.sin(2 * np.pi * f * t) + 0.3 * np.sin(4 * np.pi * f * t)) * np.exp(-t / 0.12)
        x[k * m : k * m + len(t)] += y
    return _ramp(x, 3), {"notes": [int(v) for v in notes]}


def turn_signal(rng):
    rate = _u(rng, 1.3, 1.8)
    dur = _u(rng, 3.0, 6.0)
    x = np.zeros(int(dur * SR))
    click = _filt(rng.standard_normal(300) * np.exp(-np.arange(300) / 40), "bandpass", (800, 4000))
    for t in np.arange(0, dur - 0.1, 1 / rate):
        for off, a in ((0.0, 1.0), (0.5 / rate, 0.7)):  # 「嘀—嗒」两声
            i = int((t + off) * SR)
            if i + 300 < len(x):
                x[i : i + 300] += a * click
    return x, {"rate_hz": rate}


def door_chime(rng):
    if rng.random() < 0.5:
        f = [659.0, 523.0]
        d = 0.6
    else:
        f = [988.0, 988.0, 988.0]
        d = 0.25
    t = np.arange(int(d * SR)) / SR
    seq = [(np.sin(2 * np.pi * fi * t) + 0.2 * np.sin(6 * np.pi * fi * t)) * np.exp(-t / (0.5 * d)) for fi in f]
    return _ramp(np.concatenate(seq), 3), {"tones_hz": f}


def luggage(rng):
    dur = _u(rng, 2.0, 5.0)
    n = int(dur * SR)
    rate = _u(rng, 6, 18)  # 轮子碾过地砖缝
    x = 0.4 * _filt(_colored(rng, n, -3), "bandpass", (100, 1500))
    for t in np.arange(0, dur, 1 / rate):
        i = int((t + rng.normal(0, 0.005)) * SR)
        if 0 <= i < n - 200:
            x[i : i + 200] += _filt(rng.standard_normal(200), "bandpass", (300, 3000)) * np.exp(-np.arange(200) / 40)
    env = np.minimum(1, np.minimum(np.arange(n), n - np.arange(n)) / (0.4 * SR))
    return x * env, {"bump_rate_hz": rate}


def page_turn(rng):
    d = _u(rng, 0.3, 0.8)
    n = int(d * SR)
    x = _filt(rng.standard_normal(n), "bandpass", (1000, 7500))
    x *= np.abs(_filt(rng.standard_normal(n), "lowpass", _u(rng, 30, 80))) * 4
    i = int(0.8 * n)
    x[i : i + 160] += _filt(rng.standard_normal(min(160, n - i)), "lowpass", 2000) * 3
    return _ramp(x, 15), {"dur_s": d}


# ---------------- 语音外放 ----------------
def speaker(x, rng, style):
    """把干语音变成外放声：电视 / 导航（小喇叭）、广播（窄带 + 压缩 + 轻失真）、隔墙（低通）。"""
    if style == "tv":
        y = _filt(x, "bandpass", (_u(rng, 120, 200), _u(rng, 6000, 7500)), 3)
        y = np.tanh(1.5 * y / (np.max(np.abs(y)) + 1e-12))
        return y, {"style": style}
    if style == "nav":
        y = _filt(x, "bandpass", (_u(rng, 250, 400), _u(rng, 5000, 7000)), 3)
        return np.tanh(2.0 * y / (np.max(np.abs(y)) + 1e-12)), {"style": style}
    if style == "announce":
        y = _filt(x, "bandpass", (_u(rng, 250, 400), _u(rng, 3500, 5000)), 4)
        return np.tanh(3.0 * y / (np.max(np.abs(y)) + 1e-12)), {"style": style}
    if style == "muffled":
        return _filt(x, "lowpass", _u(rng, 500, 1000), 4), {"style": style}
    return x, {"style": style}


def pa_chime(rng):
    f = sorted(rng.choice([523.0, 659.0, 784.0, 1047.0], 3, replace=False))
    t = np.arange(int(0.4 * SR)) / SR
    seq = [(np.sin(2 * np.pi * fi * t) + 0.3 * np.sin(4 * np.pi * fi * t)) * np.exp(-t / 0.25) for fi in f]
    return _ramp(np.concatenate(seq), 3)


CONTINUOUS = {"room_tone": room_tone, "hvac": hvac, "fan": fan, "fridge_hum": fridge_hum, "hvac_cycle": hvac_cycle,
              "coffee_machine": coffee_machine, "wind": wind, "cabin": cabin}
EVENTS = {"dishes": dishes, "phone_ring": phone_ring, "notify": notify, "turn_signal": turn_signal,
          "door_chime": door_chime, "luggage": luggage, "page_turn": page_turn}
