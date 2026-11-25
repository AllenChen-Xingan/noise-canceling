"""「掩码族能达到的损失」（oracle）：给过拟合判据当下限对照（大总管 2026-11-25）。

GTCRN 的输出 = 噪声谱 × 复数掩码（官方 `Mask`）。掩码的约束（读官方 `gtcrn.py`）：
- 解码器末层是 tanh，所以 ERB 域掩码的实部、虚部各在 (−1, 1) 内
- 低频 65 个频点直接用；高频 192 个频点由 64 个 ERB 子带经固定线性展开（`ERB.bs` 里的 `ierb_fc`）得到

oracle 的做法：理想复数掩码 clean / noisy → 实虚部截到 ±MASK_LIM → 高频用 `ierb_fc` 权重的伪逆投影到 64 个子带、再截断、再展开。
它是掩码族里的**一个可行点**，不是证明过的最优点：真正的最好损失 ≤ oracle 损失。
所以「下限 < oracle 损失」只说明这个可行点到不了下限；按大总管定的口径记作「下限不可达」。
"""
from __future__ import annotations

import math

import torch

import losses
import train
from official import official_module

MASK_LIM = 0.999  # tanh 取不到 ±1
N_LOW = 65        # 官方 ERB(65, 64)：前 65 个频点不分子带


def _clip(z: torch.Tensor) -> torch.Tensor:
    return torch.complex(z.real.clamp(-MASK_LIM, MASK_LIM), z.imag.clamp(-MASK_LIM, MASK_LIM))


def _ierb_weight(device) -> torch.Tensor:
    return official_module().ERB(N_LOW, 64).ierb_fc.weight.detach().to(device)  # (192, 64)


def _c(spec: torch.Tensor) -> torch.Tensor:
    return torch.complex(spec[..., 0], spec[..., 1])  # (B, F, T)


def family_mask(X: torch.Tensor, S: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
    """返回 (低频掩码 (B,65,T), ERB 子带掩码 (B,64,T))，都已截到 ±MASK_LIM。X、S 是 (B,F,T,2) 的谱。"""
    Xc, Sc = _c(X), _c(S)
    ok = Xc.abs() > 1e-12
    M = _clip(torch.where(ok, Sc / torch.where(ok, Xc, torch.ones_like(Xc)), torch.zeros_like(Sc)))
    P = torch.linalg.pinv(_ierb_weight(X.device))  # (64, 192)
    hi = M[:, N_LOW:, :]
    m_erb = torch.complex(torch.einsum("ef,bft->bet", P, hi.real), torch.einsum("ef,bft->bet", P, hi.imag))
    return M[:, :N_LOW, :], _clip(m_erb)


def oracle_spec(X: torch.Tensor, S: torch.Tensor) -> torch.Tensor:
    """oracle 掩码作用在噪声谱上的输出，(B,F,T,2)。"""
    m_low, m_erb = family_mask(X, S)
    W = _ierb_weight(X.device)
    hi = torch.complex(torch.einsum("fe,bet->bft", W, m_erb.real), torch.einsum("fe,bet->bft", W, m_erb.imag))
    Y = _c(X) * torch.cat([m_low, hi], dim=1)
    return torch.stack([Y.real, Y.imag], dim=-1)


@torch.no_grad()
def oracle_loss(crit, batch, device) -> dict:
    """oracle 输出在这一批上的损失（与过拟合同一个损失）和平均 SI-SNR（dB）。"""
    noisy, clean = batch["noisy"].to(device), batch["clean"].to(device)
    fr = train.frames_mask(batch, clean.shape[1]).to(device)
    pred = oracle_spec(losses.stft(noisy), losses.stft(clean))
    loss = crit(pred, clean, fr)[0].item()
    snr = float(losses.si_snr_db(losses.istft(pred, clean.shape[1]), clean).mean())
    return {"loss": loss, "si_snr_db": snr}


def floor_reachable(floor: float, oracle_loss: float) -> bool:
    """下限 ≥ oracle 损失才算可达；任一个不是有限数就判不可达。"""
    return bool(math.isfinite(floor) and math.isfinite(oracle_loss) and floor >= oracle_loss)


def frac_of_span(first: float, last: float, target: float) -> float | None:
    """(first − last) / (first − target)；距离 ≤ 0 或有非有限数时返回 None。"""
    if not all(math.isfinite(x) for x in (first, last, target)) or first - target <= 0:
        return None
    return (first - last) / (first - target)
