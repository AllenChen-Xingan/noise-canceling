"""GTCRN 结构改动的候选，全部挂在中间瓶颈处（第二个双路 RNN 之后、解码器之前）。

瓶颈特征形状 (B, C=16, T=1, F=33)：16 个通道 × 33 个子带，每 16 ms 一帧。
每个候选都是「残差 + 输出零初始化」：刚接上时输出与原模型逐值相同，可以从现有权重热启动。
每个候选只看当前帧和过去帧，不看未来帧（不加算法延迟）。

接口：`init_state(batch)` 返回该模块的流式状态；`forward(x, state, cond)` 返回 (y, state)。
手写的矩阵乘（注意力打分、加权求和）由模块自己报乘加次数：`self.extra_macs`，
线性层、GRU 的乘加由 `macs.py` 的钩子数。
"""
import torch
import torch.nn as nn

C, F = 16, 33


class Identity(nn.Module):
    """不加任何东西（对照）。"""
    extra_macs = 0

    def init_state(self, batch=1):
        return None

    def forward(self, x, state, cond=None):
        return x, state


class TempAttn(nn.Module):
    """时间方向注意力：每个子带各自看过去 K 帧（含当前帧），多头点积注意力。

    状态 = 过去 K-1 帧的 key、value 缓存 (B, F, K-1, C)。没有位置编码（推断：靠缓存顺序 +
    GRU 已有的时序信息；如要加，相对位置偏置每头 K 个数，乘加可忽略）。
    """

    def __init__(self, k_frames=32, heads=2):
        super().__init__()
        self.k, self.h, self.dh = k_frames, heads, C // heads
        self.q = nn.Linear(C, C)
        self.kv = nn.Linear(C, 2 * C)
        self.out = nn.Linear(C, C)
        nn.init.zeros_(self.out.weight)
        nn.init.zeros_(self.out.bias)
        self.extra_macs = 0

    def init_state(self, batch=1):
        z = torch.zeros(batch, F, self.k - 1, C)
        return (z, z.clone())

    def forward(self, x, state, cond=None):
        b = x.shape[0]
        t = x.permute(0, 3, 2, 1).reshape(b, F, C)  # (B,F,C) 当前帧
        q = self.q(t)
        k_new, v_new = self.kv(t).chunk(2, dim=-1)
        k_all = torch.cat([state[0], k_new[:, :, None]], dim=2)  # (B,F,K,C)
        v_all = torch.cat([state[1], v_new[:, :, None]], dim=2)
        qh = q.view(b, F, self.h, 1, self.dh)
        kh = k_all.view(b, F, self.k, self.h, self.dh).transpose(2, 3)  # (B,F,H,K,dh)
        vh = v_all.view(b, F, self.k, self.h, self.dh).transpose(2, 3)
        att = torch.softmax((qh * kh).sum(-1) / self.dh ** 0.5, dim=-1)  # (B,F,H,K)
        o = (att[..., None] * vh).sum(-2).reshape(b, F, C)
        self.extra_macs = 2 * b * F * self.k * C  # 打分 + 加权求和
        y = t + self.out(o)
        y = y.reshape(b, F, 1, C).permute(0, 3, 2, 1)
        return y, (k_all[:, :, 1:], v_all[:, :, 1:])


class FreqAttn(nn.Module):
    """频率方向注意力：同一帧里 33 个子带互相看。无状态，不看未来帧。"""

    def __init__(self, heads=2):
        super().__init__()
        self.h, self.dh = heads, C // heads
        self.qkv = nn.Linear(C, 3 * C)
        self.out = nn.Linear(C, C)
        nn.init.zeros_(self.out.weight)
        nn.init.zeros_(self.out.bias)
        self.extra_macs = 0

    def init_state(self, batch=1):
        return None

    def forward(self, x, state, cond=None):
        b = x.shape[0]
        t = x.permute(0, 2, 3, 1).reshape(b, F, C)
        q, k, v = self.qkv(t).view(b, F, 3, self.h, self.dh).permute(2, 0, 3, 1, 4)
        att = torch.softmax(q @ k.transpose(-1, -2) / self.dh ** 0.5, dim=-1)  # (B,H,F,F)
        o = (att @ v).transpose(1, 2).reshape(b, F, C)
        self.extra_macs = 2 * b * F * F * C
        y = (t + self.out(o)).reshape(b, 1, F, C).permute(0, 3, 1, 2)
        return y, state


class FiLM(nn.Module):
    """条件输入：外部给一个场景 / 噪声类型向量（维度 d），算出每通道的缩放和偏移。

    零初始化：缩放 = 1、偏移 = 0，等于原模型。向量从哪来（辅助小模型）归 aux 线。
    """

    def __init__(self, cond_dim=8):
        super().__init__()
        self.d = cond_dim
        self.fc = nn.Linear(cond_dim, 2 * C)
        nn.init.zeros_(self.fc.weight)
        nn.init.zeros_(self.fc.bias)
        self.extra_macs = 0

    def init_state(self, batch=1):
        return None

    def forward(self, x, state, cond=None):
        if cond is None:
            cond = torch.zeros(x.shape[0], self.d)
        g, b = self.fc(cond).chunk(2, dim=-1)
        y = x * (1 + g[:, :, None, None]) + b[:, :, None, None]
        return y, state


class ExtraDPGRNN(nn.Module):
    """多加一层双路 RNN（与原来两层同形），可选更宽（经 1×1 投影进出）。

    零初始化做法：两个层归一化的缩放和偏移置 0 → 两条残差支路输出 0 → 整层等于恒等；
    更宽版本另把出投影置 0。注意：层归一化缩放为 0 时梯度仍能流到它自己（不会卡死）。
    """

    def __init__(self, DPGRNN, width=16):
        super().__init__()
        self.w = width
        self.proj_in = nn.Linear(C, width) if width != C else None
        self.proj_out = nn.Linear(width, C) if width != C else None
        self.block = DPGRNN(width, F, width)
        for ln in (self.block.intra_ln, self.block.inter_ln):
            nn.init.zeros_(ln.weight)
            nn.init.zeros_(ln.bias)
        if self.proj_out is not None:
            nn.init.zeros_(self.proj_out.weight)
            nn.init.zeros_(self.proj_out.bias)
        self.extra_macs = 0

    def init_state(self, batch=1):
        return torch.zeros(1, batch * F, self.w)

    def forward(self, x, state, cond=None):
        if self.proj_in is None:
            return self.block(x, state)
        h = self.proj_in(x.permute(0, 2, 3, 1)).permute(0, 3, 1, 2)
        h, state = self.block(h, state)
        y = x + self.proj_out(h.permute(0, 2, 3, 1)).permute(0, 3, 1, 2)
        return y, state


def build(name, DPGRNN):
    """按名字造候选。名字是比较卡与读数文件的唯一键。"""
    table = {
        "base": lambda: Identity(),
        "tattn-k16": lambda: TempAttn(16),
        "tattn-k32": lambda: TempAttn(32),
        "tattn-k64": lambda: TempAttn(64),
        "fattn": lambda: FreqAttn(),
        "film-d8": lambda: FiLM(8),
        "dpgrnn3": lambda: ExtraDPGRNN(DPGRNN, 16),
        "dpgrnn3-w32": lambda: ExtraDPGRNN(DPGRNN, 32),
    }
    return table[name]()


NAMES = ["base", "tattn-k16", "tattn-k32", "tattn-k64", "fattn",
         "film-d8", "dpgrnn3", "dpgrnn3-w32"]
