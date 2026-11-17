"""模型变体（整段版，训练用）。

- **A 原结构**：官方 `GTCRN` 类本身，不改一行。
- **B 原结构 + 时间注意力头 + 条件输入口子**（028 第一、二节 · D98）：挂在第二个双路 RNN 之后、解码器之前。
  - 时间注意力：每个子带各自回看 K 帧（含当前帧，默认 32 ≈ 0.5 s），多头点积注意力，残差接入，
    输出层零初始化。开头不足 K 帧时用全零的 key / value 补齐——与 exp019 流式版的初始缓存一致，
    所以整段版与逐帧版逐值相同（测试 `test_offline_matches_exp019_stream`）。
  - 条件输入：外部向量 cond（每段一个 (B, d)，或每帧一个 (B, T, d)）经**不带偏置**的线性层算出
    每通道缩放与偏移，零初始化。cond 全零时任何时候（含训练后）这一层都是恒等（D98）。

两种变体的 forward 都是 `forward(spec, cond=None)`，spec 形状 (B, 257, T, 2)，与官方一致。
"""
from __future__ import annotations

import torch
import torch.nn as nn

from official import official_module

C, F = 16, 33  # 瓶颈：16 通道 × 33 子带


class TempAttnSeq(nn.Module):
    """时间方向注意力，整段输入 x: (B, C, T, F) → 同形。参数名与 exp019 `TempAttn` 相同，权重可互换。"""

    def __init__(self, k_frames: int = 32, heads: int = 2):
        super().__init__()
        if C % heads:
            raise ValueError("heads 必须整除 16")
        self.k, self.h, self.dh = k_frames, heads, C // heads
        self.q = nn.Linear(C, C)
        self.kv = nn.Linear(C, 2 * C)
        self.out = nn.Linear(C, C)
        nn.init.zeros_(self.out.weight)
        nn.init.zeros_(self.out.bias)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        b, _, t, f = x.shape
        z = x.permute(0, 3, 2, 1)  # (B,F,T,C)
        q = self.q(z)
        k, v = self.kv(z).chunk(2, dim=-1)
        pad = self.k - 1
        k = nn.functional.pad(k, [0, 0, pad, 0])  # 开头补零 key / value（= 流式初始缓存）
        v = nn.functional.pad(v, [0, 0, pad, 0])
        kw = k.unfold(2, self.k, 1)  # (B,F,T,C,K)：第 i 帧的窗口 = 补零后的 [i, i+K)，末位是当前帧
        vw = v.unfold(2, self.k, 1)
        qh = q.view(b, f, t, self.h, self.dh)
        kh = kw.reshape(b, f, t, self.h, self.dh, self.k)
        vh = vw.reshape(b, f, t, self.h, self.dh, self.k)
        s = torch.einsum("bfthd,bfthdk->bfthk", qh, kh) / self.dh ** 0.5
        att = torch.softmax(s, dim=-1)
        o = torch.einsum("bfthk,bfthdk->bfthd", att, vh).reshape(b, f, t, C)
        y = z + self.out(o)
        return y.permute(0, 3, 2, 1)


class CondFiLM(nn.Module):
    """条件输入口子：y = x·(1+g) + β，[g, β] = W·cond，W 不带偏置、零初始化。参数名与 exp019 `FiLM` 相同。"""

    def __init__(self, cond_dim: int = 8):
        super().__init__()
        self.d = cond_dim
        self.fc = nn.Linear(cond_dim, 2 * C, bias=False)
        nn.init.zeros_(self.fc.weight)

    def forward(self, x: torch.Tensor, cond: torch.Tensor | None) -> torch.Tensor:
        if cond is None:
            return x  # 等价于全零向量（无偏置 → g = β = 0）
        gb = self.fc(cond)
        if gb.dim() == 2:  # (B, 2C)：整段一个向量
            g, bt = gb[:, :C, None, None], gb[:, C:, None, None]
        elif gb.dim() == 3:  # (B, T, 2C)：每帧一个向量
            g = gb[..., :C].permute(0, 2, 1)[..., None]
            bt = gb[..., C:].permute(0, 2, 1)[..., None]
        else:
            raise ValueError("cond 形状应为 (B, d) 或 (B, T, d)")
        return x * (1 + g) + bt


def _official_cls():
    return official_module().GTCRN


class GTCRNB(nn.Module):
    """变体 B。基座是官方 GTCRN 实例（`self.base`，state_dict 键前缀 `base.`）。

    forward 只重写一次官方的串接顺序并在瓶颈处插入两层；`attn=None, film=None` 时与官方逐值相同（测试核）。
    """

    def __init__(self, k_frames: int = 32, heads: int = 2, cond_dim: int = 8,
                 attn: bool = True, film: bool = True):
        super().__init__()
        self.base = _official_cls()()
        self.attn = TempAttnSeq(k_frames, heads) if attn else None
        self.film = CondFiLM(cond_dim) if film else None

    def forward(self, spec: torch.Tensor, cond: torch.Tensor | None = None) -> torch.Tensor:
        m = self.base
        re = spec[..., 0].permute(0, 2, 1)
        im = spec[..., 1].permute(0, 2, 1)
        mag = torch.sqrt(re ** 2 + im ** 2 + 1e-12)
        feat = torch.stack([mag, re, im], dim=1)
        feat = m.sfe(m.erb.bm(feat))
        feat, en_outs = m.encoder(feat)
        feat = m.dpgrnn2(m.dpgrnn1(feat))
        if self.attn is not None:
            feat = self.attn(feat)
        if self.film is not None:
            feat = self.film(feat, cond)
        mfeat = m.decoder(feat, en_outs)
        mask = m.erb.bs(mfeat)
        out = m.mask(mask, spec.permute(0, 3, 2, 1))
        return out.permute(0, 3, 2, 1)

    def load_base(self, state: dict) -> None:
        self.base.load_state_dict(state)


class GTCRNA(nn.Module):
    """变体 A：官方 GTCRN 原样包一层，只为与 B 同一个 forward 签名（cond 被忽略）。"""

    def __init__(self):
        super().__init__()
        self.base = _official_cls()()

    def forward(self, spec: torch.Tensor, cond: torch.Tensor | None = None) -> torch.Tensor:
        return self.base(spec)

    def load_base(self, state: dict) -> None:
        self.base.load_state_dict(state)


def build(variant: str, **kw) -> nn.Module:
    """按名字造模型。名字是配置文件与运行记录里的唯一键。"""
    if variant == "A":
        return GTCRNA()
    if variant == "B":
        return GTCRNB(**kw)
    raise ValueError(f"未知变体 {variant!r}（只有 A · B）")


def param_counts(model: nn.Module) -> dict:
    """可训练 / 固定参数数。固定的是 ERB 两个滤波器组矩阵（官方 `requires_grad=False`）。"""
    tr = sum(p.numel() for p in model.parameters() if p.requires_grad)
    fx = {n: p.numel() for n, p in model.named_parameters() if not p.requires_grad}
    return {"trainable": tr, "fixed": sum(fx.values()), "fixed_names": fx, "total": tr + sum(fx.values())}
