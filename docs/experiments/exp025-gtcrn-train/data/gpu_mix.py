"""exp025 · 批量混音的 torch 版（在 GPU 上算；CPU 只在 DataLoader worker 里做 mix.plan，读原始片段 + 抽随机数）。

和 mix.render（CPU 参考版，正本）同一组步骤、同一顺序；对拍测试 tests/test_gpu_mix.py 要求同一批 plan 两边逐样本差 ≤ 1e-4。
做法：
- 卷积与均衡全在频域一次做完：Y = X · RIR(ω) · B(ω)/A(ω)。FFT 长度 N ≥ n + RIR 长 + EQ_TAIL；二阶滤波极点半径 ≤ 约 0.83
  （|r| ≤ 3/8），尾巴到 EQ_TAIL = 512 个样本时 < 1e-40，所以循环卷积的回绕误差可忽略——等价于逐样本的 lfilter
- 电平（gated_level_db）与有声区间（intervals）照 exp013 dsp.py 的定义逐行改写成批量版；它们是 dsp.py 的第二处表示，
  tests/test_gpu_mix.py 和 dsp.py 逐个对拍
- 默认 float64（H100 双精度够快；电平门限在帧边界上不因精度翻转），可传 dtype=torch.float32
旋钮参数只在 recipe.json；早期混响长度只在 exp013 rooms.py（plan 里已切好，这里不碰）。
"""
import sys
from pathlib import Path

import numpy as np
import torch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import mix  # noqa: E402
from dsp import FRAME  # noqa: E402  mix 已把 exp013 放进 sys.path

EQ_TAIL = 512
EPS = mix.EPS
REL_DB = -35.0  # dsp.gated_level_db / dsp.intervals 的默认门限；测试核对和 dsp.py 的默认参数相同
GUARD = 2  # dsp.intervals 的默认外扩帧数；同上


def collate_plans(plans):
    """一批 Plan → numpy 数组字典（可以在 DataLoader worker 里做，跨进程传给主进程）。RIR 补零到本批最长。"""
    L = max(len(p.rir_full) for p in plans)

    def pad(x):
        out = np.zeros(L)
        out[: len(x)] = x
        return out

    return {
        "speech": np.stack([p.speech for p in plans]), "noises": np.stack([p.noises for p in plans]),
        "k": np.array([p.k for p in plans]), "rir_full": np.stack([pad(p.rir_full) for p in plans]),
        "rir_early": np.stack([pad(p.rir_early) for p in plans]), "eq_speech": np.stack([p.eq_speech for p in plans]),
        "eq_noise": np.stack([p.eq_noise for p in plans]), "rel_gain": np.stack([p.rel_gain for p in plans]),
        "snr_db": np.array([p.snr_db for p in plans]), "level_dbfs": np.array([p.level_dbfs for p in plans]),
        "silent": np.array([p.silent for p in plans]), "clip_u": np.array([p.clip_u for p in plans]),
        "meta": [p.meta for p in plans],
    }


def _nfft(n):
    return 1 << int(np.ceil(np.log2(n)))


def _biquad_freq(c, N):
    """c: [..., 6] = b0 b1 b2 a0 a1 a2 → 频响 B/A 在 rfft 的 N//2+1 个点上。"""
    w = torch.arange(N // 2 + 1, device=c.device, dtype=c.dtype) * (2 * np.pi / N)
    z1 = torch.polar(torch.ones_like(w), -w)  # e^{-jω}
    z2 = z1 * z1
    b = c[..., 0:1] + c[..., 1:2] * z1 + c[..., 2:3] * z2
    a = c[..., 3:4] + c[..., 4:5] * z1 + c[..., 5:6] * z2
    return b / a


def frame_energy(x):
    """dsp.frame_energy 的批量版：x [B, n] → [B, n // FRAME]。"""
    m = x.shape[-1] // FRAME
    f = x[..., : m * FRAME].reshape(*x.shape[:-1], m, FRAME)
    return (f * f).mean(-1)


def gated_level_db(x):
    """dsp.gated_level_db 的批量版：活动帧（> 最大帧 −35 dB）能量均值的 dB；全静音返回 −240。"""
    e = frame_energy(x)
    emax = e.max(-1, keepdim=True).values
    act = e > emax * 10 ** (REL_DB / 10)
    lvl = 10 * torch.log10((e * act).sum(-1) / act.sum(-1).clamp(min=1) + EPS)
    return torch.where(emax.squeeze(-1) <= EPS, torch.full_like(lvl, -240.0), lvl)


def activity(x):
    """dsp.intervals 前半段的批量版：活动帧（含两端外扩 GUARD 帧）。返回 bool [B, 帧数]。"""
    e = frame_energy(x)
    emax = e.max(-1, keepdim=True).values
    act = (e > emax * 10 ** (REL_DB / 10)) & (emax > EPS)
    if GUARD:
        k = torch.ones(1, 1, 2 * GUARD + 1, device=x.device, dtype=x.dtype)
        act = torch.nn.functional.conv1d(act.to(x.dtype).unsqueeze(1), k, padding=GUARD).squeeze(1) > 0
    return act


def runs(act_row, n):
    """dsp.intervals 后半段：活动帧 → [[起, 止), …] 样本号（CPU 上做，数据很小）。"""
    a = np.asarray(act_row, bool)
    if not a.any():
        return []
    d = np.diff(np.concatenate([[0], a.astype(np.int8), [0]]))
    starts, ends = np.flatnonzero(d == 1), np.flatnonzero(d == -1)
    return [[int(s * FRAME), int(min(n, e * FRAME))] for s, e in zip(starts, ends)]


def render_batch(b, device="cuda", dtype=torch.float64):
    """b = collate_plans 的输出（numpy）。返回 dict：noisy · clean [B, n]（float32 张量，在 device 上）·
    owner_ivs · interf_ivs（list）· near_silent · replay 由调用方从 meta 取 · clip_thr [B] · meta。"""
    T = lambda a: torch.as_tensor(np.asarray(a), device=device, dtype=dtype)  # noqa: E731
    s, ns = T(b["speech"]), T(b["noises"])
    B, M, n = ns.shape
    L = b["rir_full"].shape[1]
    N = _nfft(n + L + EQ_TAIL)
    S = torch.fft.rfft(s, N)
    Heq = _biquad_freq(T(b["eq_speech"]), N)
    rev = torch.fft.irfft(S * torch.fft.rfft(T(b["rir_full"]), N) * Heq, N)[..., :n]
    tgt = torch.fft.irfft(S * torch.fft.rfft(T(b["rir_early"]), N) * Heq, N)[..., :n]
    Nn = _nfft(n + EQ_TAIL)
    y = torch.fft.irfft(torch.fft.rfft(ns, Nn) * _biquad_freq(T(b["eq_noise"]), Nn), Nn)[..., :n]
    used = (torch.arange(M, device=device)[None, :] < torch.as_tensor(b["k"], device=device)[:, None]).to(dtype)
    y = y / torch.sqrt((y * y).mean(-1, keepdim=True) + EPS)
    noise = (y * (T(b["rel_gain"]) * used).unsqueeze(-1)).sum(1)
    pn = 10 * torch.log10((noise * noise).mean(-1) + EPS)
    lv = gated_level_db(rev)
    noise = noise * (10 ** ((lv - T(b["snr_db"]) - pn) / 20)).unsqueeze(-1)
    g = (10 ** ((T(b["level_dbfs"]) - lv) / 20)).unsqueeze(-1)
    rev, tgt, noise = rev * g, tgt * g, noise * g
    keep = (~torch.as_tensor(b["silent"], device=device)).to(dtype).unsqueeze(-1)
    rev, tgt = rev * keep, tgt * keep
    noisy = rev + noise
    peak = torch.maximum(noisy.abs().max(-1).values, tgt.abs().max(-1).values).clamp(min=EPS)
    sc = torch.where(peak > mix.PEAK, mix.PEAK / peak, torch.ones_like(peak)).unsqueeze(-1)
    noisy, tgt, rev = noisy * sc, tgt * sc, rev * sc
    act = activity(rev).cpu().numpy()
    u = T(b["clip_u"])
    thr = u * noisy.abs().max(-1).values
    clipped = torch.maximum(torch.minimum(noisy, thr.unsqueeze(-1)), -thr.unsqueeze(-1))
    noisy = torch.where((u > 0).unsqueeze(-1), clipped, noisy)
    silent = np.asarray(b["silent"], bool)
    thr_np = thr.cpu().numpy()
    return {
        "noisy": noisy.to(torch.float32), "clean": tgt.to(torch.float32),
        "owner_ivs": [[] if silent[i] else runs(act[i], n) for i in range(B)], "interf_ivs": [[] for _ in range(B)],
        "clip_thr": [round(float(thr_np[i]), 5) if b["clip_u"][i] > 0 else None for i in range(B)],
        "meta": b["meta"],
    }
