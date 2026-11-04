"""exp013 · 房间冲激响应（本线自写，不调用 exp006 `sim.make_rir`，016 S10⑤）。

做法：长方体房间，麦在桌面笔记本处；镜像源法算直达声与 1–2 阶反射（反射系数由 Sabine 公式按 T60 反推），
再接两频带指数衰减的高斯尾巴；尾巴能量按临界距离定（混响能量 / 直达能量 = (距离 / 临界距离)²）。
直达部分（直达峰前后 022 第 2 步的窗长）与其余部分分开给，设备链第 2 步只衰减直达部分。
T60 ≈ 0（户外）：只有直达声 + 一次地面反射。
"""
from dataclasses import dataclass

import numpy as np
from scipy import signal

from dsp import SR

C = 343.0
MIC_H = 0.8  # 桌面笔记本麦的高度（m）【推断】


@dataclass
class Rir:
    direct: np.ndarray  # 直达峰前后窗内的部分
    rest: np.ndarray  # 其余（早期反射 + 尾巴）
    t60: float  # 名义 T60
    dist_m: float  # 实际源—麦距离
    azimuth_deg: float  # 实际方位（声源被夹回房间后重算）
    dims: tuple
    delay: int  # 直达声到达的样本号

    @property
    def full(self):
        return self.direct + self.rest


def _alpha(dims, t60):
    L, W, H = dims
    V, S = L * W * H, 2 * (L * W + L * H + W * H)
    return float(np.clip(0.161 * V / (S * max(t60, 1e-3)), 0.02, 0.98))


def _place(dims, dist, az_deg, src_h):
    L, W, H = dims
    mic = np.array([L / 2, W * 0.4, MIC_H])
    a = np.deg2rad(az_deg)
    horiz = np.sqrt(max(dist**2 - (src_h - MIC_H) ** 2, 0.01))
    src = mic + np.array([horiz * np.sin(a), horiz * np.cos(a), src_h - MIC_H])
    src = np.clip(src, [0.1, 0.1, 0.1], [L - 0.1, W - 0.1, H - 0.1])
    # 被夹回房间后方位会变：按实际位置重算（0° = 正前方 +y，顺时针为正），左右对称故只取水平角
    d = src - mic
    az = float(np.rad2deg(np.arctan2(d[0], d[1])))
    return mic, src, az


def _images(dims, src, order):
    """1–order 阶镜像源（每面墙按 Allen–Berkley 的下标法），返回 [(位置, 反射次数)]。"""
    L, W, H = dims
    out = []
    rng_ = range(-order, order + 1)
    for nx in rng_:
        for ny in rng_:
            for nz in rng_:
                for px in (0, 1):
                    for py in (0, 1):
                        for pz in (0, 1):
                            k = abs(2 * nx - px) + abs(2 * ny - py) + abs(2 * nz - pz)
                            if k == 0 or k > order:
                                continue
                            pos = np.array([
                                (1 - 2 * px) * src[0] + 2 * nx * L,
                                (1 - 2 * py) * src[1] + 2 * ny * W,
                                (1 - 2 * pz) * src[2] + 2 * nz * H,
                            ])
                            out.append((pos, k))
    return out


def make_rir(rng, dims, t60, dist, az_deg, src_h=1.2, window_ms=None, order=2):
    """合成一条 RIR。返回 Rir（直达部分与其余分开）。"""
    dims = tuple(float(d) for d in dims)
    mic, src, az_real = _place(dims, dist, az_deg, src_h)
    d0 = float(np.linalg.norm(src - mic))
    n0 = int(round(d0 / C * SR))
    if t60 < 0.03:  # 户外：直达 + 地面反射
        n = n0 + int(0.05 * SR)
        h = np.zeros(n)
        h[n0] = 1.0 / d0
        img = src * np.array([1, 1, -1])
        d1 = float(np.linalg.norm(img - mic))
        h[min(n - 1, int(round(d1 / C * SR)))] += 0.8 / d1
    else:
        n = n0 + int(max(0.05, 1.2 * t60) * SR)
        h = np.zeros(n)
        h[n0] = 1.0 / d0
        beta = np.sqrt(1 - _alpha(dims, t60))
        early_e = 0.0
        for pos, k in _images(dims, src, order):
            d = float(np.linalg.norm(pos - mic))
            i = int(round(d / C * SR))
            if i < n:
                a = beta**k / d
                h[i] += a
                early_e += a * a
        # 尾巴：从 2 阶反射之后开始（不早于直达后 5 ms）
        L, W, H = dims
        V = L * W * H
        rc = 0.057 * np.sqrt(V / t60)
        rev_e = (1.0 / d0) ** 2 * (d0 / rc) ** 2
        tail_e = max(rev_e - early_e, 0.1 * rev_e)
        t_start = n0 + int(0.005 * SR)
        t = np.arange(n - t_start) / SR
        lo = signal.sosfilt(signal.butter(2, 3000, "lowpass", fs=SR, output="sos"), rng.standard_normal(len(t)))
        hi = signal.sosfilt(signal.butter(2, 3000, "highpass", fs=SR, output="sos"), rng.standard_normal(len(t)))
        tail = lo * np.exp(-6.91 * t / t60) + hi * np.exp(-6.91 * t / (0.8 * t60))
        ramp = min(len(t), int(0.003 * SR))
        tail[:ramp] *= np.linspace(0, 1, ramp)
        tail *= np.sqrt(tail_e / max(np.sum(tail * tail), 1e-20))
        h[t_start:] += tail
    if window_ms is None:  # 直达窗长的正本在 device_chain.json（022 第 2 步）
        from device_chain import P as _CP

        window_ms = _CP["step2_direction"]["direct_window_ms"]
    w = int(round(window_ms * 1e-3 * SR))
    direct = np.zeros_like(h)
    lo_i, hi_i = max(0, n0 - w), min(len(h), n0 + w + 1)
    direct[lo_i:hi_i] = h[lo_i:hi_i]
    rest = h - direct
    scale = d0  # 归一到直达幅度 1
    return Rir(direct * scale, rest * scale, float(t60), d0, az_real, dims, n0)


def drr_db(r: Rir):
    ed, er = float(np.sum(r.direct**2)), float(np.sum(r.rest**2))
    return 10 * np.log10(ed / max(er, 1e-20)) if er > 0 else 99.0
