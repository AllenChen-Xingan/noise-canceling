"""训练损失（027 第 3.2 节 · 034 修正）。

`L = HybridLoss + w_lvl·L_lvl + w_on·L_on [+ w_sp·L_sp + w_kd·L_kd]`

- HybridLoss：官方 `resources/gtcrn/loss.py` 原样导入，不改不抄。
- L_lvl 人声被压低：可微地复刻 V-SPLVL（正本 `src/eval/src/verifiers/perceptual.rs` `v_splvl`）的读数 d（dB，≤ 0），
  罚 relu(−d − 余量)，余量默认 2 dB。
- L_on 字头：可微地复刻 V-ONSET（正本同文件 `v_onset`）的读数「字头损失 − 整段有声损失」，罚 relu(超出)。
- L_sp（拉回原权重）· L_kd（原模型当老师，只在回放样本上）：开关，默认关（034 papers 第二节建议 7）。
- 电平归一（Braun）：开关，默认关。

**交叉校验**（CLAUDE.md 五.5）：`splvl_db` / `onset_excess_db` 就是参与反传的那份代码，
测试 `tests/test_losses_parity.py` 用 `parity_rs`（只读调用 vdn-eval）对拍；`paint` / `truth_from_clean` 同样对拍。
**样本不足**（027 3.2 第 3 条）：与 verifier 读「待测」的条件相同时，该段该项跳过、不进分母；整批都无效时该项为 0。
"""
from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import torch

from official import official_loss_module

SR = 16_000
N_FFT, HOP = 512, 256
MASK_FRAME = 160  # 10 ms，analysis.rs MASK_FRAME
ONSET_F = 320  # 20 ms，perceptual.rs v_onset F
ONSET_W = 800  # 50 ms，perceptual.rs v_onset W
SPLVL_MIN_FRAMES = 50  # perceptual.rs v_splvl：n < 50 读「待测」
ONSET_MIN = 3  # perceptual.rs v_onset：字头 < 3 读「待测」


# ---------- STFT（与官方 HybridLoss 内部的 istft 同参数） ----------

def window(device=None) -> torch.Tensor:
    return torch.hann_window(N_FFT, device=device).pow(0.5)


def stft(wav: torch.Tensor) -> torch.Tensor:
    """(B, N) → (B, 257, T, 2)。"""
    s = torch.stft(wav, N_FFT, HOP, N_FFT, window=window(wav.device), return_complex=True)
    return torch.view_as_real(s)


def istft(spec: torch.Tensor, length: int) -> torch.Tensor:
    z = torch.complex(spec[..., 0], spec[..., 1])
    return torch.istft(z, N_FFT, HOP, N_FFT, window=window(spec.device), length=length)


# ---------- 真值 → 掩码（复刻 analysis.rs paint / truth_from_clean） ----------

def paint(ivs, nf: int, guard: int = 0) -> np.ndarray:
    m = np.zeros(nf, dtype=bool)
    for a, b in ivs:
        fa = max(a // MASK_FRAME - guard, 0)
        fb = min(-(-b // MASK_FRAME) + guard, nf)
        if fb > fa:
            m[fa:fb] = True
    return m


def truth_from_clean(clean: np.ndarray) -> list[tuple[int, int]]:
    f = ONSET_F
    n = len(clean) // f
    p = (clean[: n * f].astype(np.float64) ** 2).reshape(n, f).sum(1) / f
    if n == 0 or p.max() <= 0:
        return []
    thr = p.max() * 10 ** (-35 / 10)
    out, start = [], None
    for i, v in enumerate(p):
        if v > thr and start is None:
            start = i
        elif v <= thr and start is not None:
            out.append((start * f, i * f))
            start = None
    if start is not None:
        out.append((start * f, n * f))
    return out


def owner_only_mask(owner_ivs, interf_ivs, n: int) -> np.ndarray:
    """V-SPLVL 用的帧：主说话人在、旁人不在（masks(truth) 的 owner && !interf，guard = 0）。

    区间来自 traindata，用的是 exp013 `dsp.intervals`（两端各外扩 2 个 20 ms 帧）——评测场景集的真值也是它算的，
    所以训练与评测的口径一致（自审时核过，不是遗漏）。"""
    nf = n // MASK_FRAME
    return paint(owner_ivs, nf) & ~paint(interf_ivs, nf)


# ---------- 可微读数 ----------

def db10(x: torch.Tensor) -> torch.Tensor:
    return 10 * torch.log10(torch.clamp(x, min=1e-20))


def st(clamped: torch.Tensor, raw: torch.Tensor) -> torch.Tensor:
    """前向取 clamped（与 Rust 读数逐值一致），反传按 raw 走（直通）。
    否则人声被压过 40 dB、碰到下限 / 封顶后梯度恰好为 0，压得越狠越不纠正（自审 P2）。"""
    return raw + (clamped - raw).detach()


def _proj(o: torch.Tensor, c: torch.Tensor):
    """⟨o,c⟩²/‖c‖² 与 ‖c‖²，沿最后一维（perceptual.rs proj_energy；‖c‖² = 0 时投影记 0）。"""
    oc = (o * c).sum(-1)
    cc = (c * c).sum(-1)
    p = torch.where(cc > 0, oc * oc / torch.where(cc > 0, cc, torch.ones_like(cc)), torch.zeros_like(cc))
    return p, cc


def splvl_db(out: torch.Tensor, clean: torch.Tensor, frames: torch.Tensor):
    """V-SPLVL 读数 d（dB，≤ 0）。out/clean (B, N)，frames bool (B, N//160)。返回 (d (B,), valid (B,))。"""
    nf = frames.shape[1]
    o = out[:, : nf * MASK_FRAME].reshape(out.shape[0], nf, MASK_FRAME)
    c = clean[:, : nf * MASK_FRAME].reshape(clean.shape[0], nf, MASK_FRAME)
    po, cc = _proj(o, c)
    w = frames.to(po.dtype)
    ps, pc = (po * w).sum(1), (cc * w).sum(1)
    valid = (frames.sum(1) >= SPLVL_MIN_FRAMES) & (pc > 1e-14)
    ps = st(torch.maximum(ps, pc * 1e-4), ps)
    d = torch.clamp(db10(ps) - db10(pc), max=0.0)  # 上限 0 不直通：「没被压低」本来就不该有梯度
    return torch.where(valid, d, torch.zeros_like(d)), valid


def onset_plan(clean_np: np.ndarray, n_out: int, skip_leading: bool = False):
    """一段的字头起点与有声 20 ms 帧下标（只依赖干净轨，不参与求导）。

    skip_leading：扫参开关，默认关（与 Rust 正本一致）。开时跳过 a = 0 的区间：随机裁切从说话中间开始时它不是真字头；
    但片头只静 10 ms 就开口也会得到 a = 0、那是真字头（codex 第 2 轮 #3 的反例），所以不默认开。"""
    ivs = truth_from_clean(clean_np)
    onsets, last_end = [], None
    for a, b in ivs:
        if (last_end is None or a >= last_end + 5 * ONSET_F) and a + ONSET_W <= n_out \
                and not (skip_leading and a == 0):
            onsets.append(a)
        last_end = b
    voiced = []
    for a, b in ivs:
        b = min(b, n_out)
        t = a
        while t + ONSET_F <= b:
            voiced.append(t // ONSET_F)  # a 是 320 的整数倍，所以帧对齐
            t += ONSET_F
    return onsets, voiced


def _onset_loss(eo: torch.Tensor, ec: torch.Tensor) -> torch.Tensor:
    raw = db10(ec) - db10(st(torch.maximum(eo, ec * 1e-4), eo))
    return st(torch.clamp(raw, max=40.0), raw)


def onset_excess_db(out: torch.Tensor, clean: torch.Tensor, skip_leading: bool = False):
    """V-ONSET 读数（字头平均损失 − 整段有声损失，dB）。返回 (excess (B,), valid (B,), 字头数 list)。"""
    b, n = out.shape
    cn = clean.detach().cpu().numpy()
    nfr = n // ONSET_F
    of = out[:, : nfr * ONSET_F].reshape(b, nfr, ONSET_F)
    cf = clean[:, : nfr * ONSET_F].reshape(b, nfr, ONSET_F)
    pof, ccf = _proj(of, cf)
    exc, valid, counts = [], [], []
    for i in range(b):
        onsets, voiced = onset_plan(cn[i], n, skip_leading)
        counts.append(len(onsets))
        if len(onsets) < ONSET_MIN:
            exc.append(out.new_zeros(()))
            valid.append(False)
            continue
        idx = torch.as_tensor(onsets, device=out.device)
        win = idx[:, None] + torch.arange(ONSET_W, device=out.device)
        po, pc = _proj(out[i][win], clean[i][win])
        per = _onset_loss(po, pc).mean()
        vi = torch.as_tensor(voiced, device=out.device, dtype=torch.long)
        avg = _onset_loss(pof[i, vi].sum(), ccf[i, vi].sum())
        exc.append(per - avg)
        valid.append(True)
    return torch.stack(exc), torch.as_tensor(valid, device=out.device), counts


def si_snr_db(est: torch.Tensor, ref: torch.Tensor) -> torch.Tensor:
    """尺度不变信噪比（dB），每段一个，只作读数（冒烟报告），不进损失。与官方 HybridLoss 里 SI-SNR 项同一个投影：
    s_t = ⟨est, ref⟩ ref / ‖ref‖²，结果 = 10·log10(‖s_t‖² / ‖est − s_t‖²)。"""
    est, ref = est.double(), ref.double()
    s_t = (est * ref).sum(-1, keepdim=True) * ref / ((ref ** 2).sum(-1, keepdim=True) + 1e-12)
    return 10 * torch.log10((s_t ** 2).sum(-1) / ((est - s_t) ** 2).sum(-1).clamp(min=1e-20))


def masked_mean(x: torch.Tensor, valid: torch.Tensor) -> torch.Tensor:
    """只在有效段上平均；整批无效时为 0（仍挂在计算图上）。"""
    k = valid.sum()
    return (x * valid.to(x.dtype)).sum() / torch.clamp(k, min=1).to(x.dtype)


# ---------- 组合 ----------

@dataclass
class LossCfg:
    w_lvl: float = 0.0
    w_on: float = 0.0
    lvl_margin_db: float = 2.0
    sp: bool = False
    w_sp: float = 1e-4
    kd: bool = False
    w_kd: float = 0.1
    level_norm: bool = False
    level_floor: float = 1e-3  # 归一除数的下限（034 修正 3-③：近静音段防除以 0）
    on_skip_leading: bool = False  # 见 onset_plan；默认与 V-ONSET 正本同口径


def active_rms(clean: torch.Tensor, frames: torch.Tensor, floor: float) -> torch.Tensor:
    """每段「主说话人单独说话」帧上的均方根（Braun 按活跃语音电平归一；整段 std 会被停顿占比拉低，自审 P1）。
    没有说话帧的段（近静音）用同批有说话段的中位数，整批都没有时用 1；最后夹到 ≥ floor（034 修正 3-③）。"""
    nf = frames.shape[1]
    c = clean[:, : nf * MASK_FRAME].reshape(clean.shape[0], nf, MASK_FRAME).detach()
    w = frames.to(c.dtype)
    n = w.sum(1)
    ms = ((c ** 2).mean(-1) * w).sum(1) / torch.clamp(n, min=1)
    rms = ms.sqrt()
    has = n > 0
    fill = rms[has].median() if has.any() else torch.ones((), dtype=c.dtype, device=c.device)
    return torch.clamp(torch.where(has, rms, fill), min=floor)


def compressed(spec: torch.Tensor) -> torch.Tensor:
    """官方 HybridLoss 的压缩复谱 s/|s|^0.7。"""
    mag = torch.sqrt(spec[..., 0] ** 2 + spec[..., 1] ** 2 + 1e-12)
    return spec / mag[..., None] ** 0.7


def l2sp(model: torch.nn.Module, ref: dict) -> torch.Tensor:
    """Σ‖θ − θ₀‖²，只算可训练参数；θ₀ 是起点（含 B 的零初始化层）。"""
    tot = None
    for name, p in model.named_parameters():
        if p.requires_grad:
            d = (p - ref[name].to(p.device)).pow(2).sum()
            tot = d if tot is None else tot + d
    return tot


class TrainLoss(torch.nn.Module):
    def __init__(self, cfg: LossCfg):
        super().__init__()
        self.cfg = cfg
        self.hybrid = official_loss_module().HybridLoss()

    def forward(self, pred_spec, clean, frames, *, model=None, sp_ref=None,
                teacher_spec=None, replay=None):
        """pred_spec (B,257,T,2)；clean (B,N) 波形；frames (B,N//160) 主说话人单独说话的帧。

        返回 (总损失, 各项字典)。各项字典里的读数是 float，直接写日志。
        """
        cfg, n = self.cfg, clean.shape[1]
        true_spec = stft(clean)
        p_h, t_h = pred_spec, true_spec
        if cfg.level_norm:
            g = 1.0 / active_rms(clean, frames, cfg.level_floor)
            p_h, t_h = pred_spec * g[:, None, None, None], true_spec * g[:, None, None, None]
        total = self.hybrid(p_h, t_h)
        logs = {"hybrid": total.item()}
        need_wav = cfg.w_lvl > 0 or cfg.w_on > 0
        if need_wav:
            out = istft(pred_spec, n)
        if cfg.w_lvl > 0:
            d, v = splvl_db(out, clean, frames)
            l_lvl = masked_mean(torch.relu(-d - cfg.lvl_margin_db), v)
            total = total + cfg.w_lvl * l_lvl
            logs |= {"lvl": l_lvl.item(), "lvl_valid": int(v.sum())}
        if cfg.w_on > 0:
            e, v, _ = onset_excess_db(out, clean, skip_leading=cfg.on_skip_leading)
            l_on = masked_mean(torch.relu(e), v)
            total = total + cfg.w_on * l_on
            logs |= {"on": l_on.item(), "on_valid": int(v.sum())}
        if cfg.sp:
            l_sp = l2sp(model, sp_ref)
            total = total + cfg.w_sp * l_sp
            logs["sp"] = l_sp.item()
        if cfg.kd and teacher_spec is not None and replay is not None and replay.any():
            r = replay.to(pred_spec.device)
            l_kd = (compressed(pred_spec[r]) - compressed(teacher_spec[r])).pow(2).mean()
            total = total + cfg.w_kd * l_kd
            logs["kd"] = l_kd.item()
        logs["total"] = total.item()
        return total, logs


def calibrate_weight(hybrid_mean: float, item_mean: float, share: float = 0.1) -> float:
    """027 3.2 权重标定：初始时该项约占 HybridLoss **大小**的 share；若该项初始为 0（没有超出），
    按「超出 1 dB 时等于 HybridLoss 大小的 share」标定，不除以 0。item_mean 只用有效段算。

    HybridLoss 含负对数（SI-SNR）项，均值可以 ≤ 0：按绝对值算尺度；接近 0 时尺度取 1。
    结果必须是有限正数，否则报错——非正权重会让保护项静默失效（codex 第 2 轮 #4）。"""
    if not (np.isfinite(hybrid_mean) and np.isfinite(item_mean)):
        raise ValueError(f"标定输入不是有限数：hybrid_mean={hybrid_mean}, item_mean={item_mean}")
    scale = abs(hybrid_mean) if abs(hybrid_mean) > 1e-6 else 1.0
    w = share * scale / (item_mean if item_mean > 1e-6 else 1.0)
    if not (np.isfinite(w) and w > 0):
        raise ValueError(f"标定出的权重不是有限正数：{w}（hybrid_mean={hybrid_mean}, item_mean={item_mean}）")
    return w
