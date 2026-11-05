"""exp018：绕开 MPS 上「沿第一空间轴带空洞的逐通道卷积」算错的问题（只在训练时用）。

实测（`runs/20261105-150606-layers/`，torch 2.15.0，Apple M5）：
`nn.Conv2d(groups=通道数, dilation=(d>1, 1))` 在 MPS 上和 CPU 差约 100%；
不带空洞、普通分组、或空洞在第二轴上，都对（差 < 1e-6）。GTCRN 里中招的是
编码器 en_convs.3 / en_convs.4 的 depth_conv（dilation 2 · 5）。转置卷积和 GRU 都对。

做法：不换模块、不换参数，只把这几个卷积实例的 forward 换成数学等价的写法——
沿时间轴的 k 个抽头各做一次「无空洞、核高 1」的逐通道卷积，再相加。
权重还是原来那几个 Parameter，state_dict 的键与形状不变，官方检查点照常载入，
训练出的权重照常导出给引擎（引擎不用这个文件）。
"""
from __future__ import annotations

import types

import torch
import torch.nn.functional as F
from torch import nn


def _needs_fix(m: nn.Module) -> bool:
    return (isinstance(m, nn.Conv2d) and m.groups == m.in_channels and m.groups > 1
            and m.dilation[0] > 1)


def _tapwise_forward(self: nn.Conv2d, x: torch.Tensor) -> torch.Tensor:
    """等价于 F.conv2d(x, w, b, stride=1, padding, dilation=(d, d1), groups=C)。"""
    if self.stride != (1, 1) or self.padding_mode != "zeros" or isinstance(self.padding, str):
        raise NotImplementedError("只支持 stride=1、零填充、数字形式的 padding")
    ph, pw = self.padding
    d, d1 = self.dilation
    kh = self.kernel_size[0]
    x = F.pad(x, [0, 0, ph, ph])  # 时间轴的填充先做掉；频率轴交给每个抽头的卷积
    t_out = x.shape[2] - d * (kh - 1)
    out = None
    for k in range(kh):
        xs = x[:, :, k * d: k * d + t_out, :]
        wk = self.weight[:, :, k: k + 1, :]
        y = F.conv2d(xs, wk, None, stride=1, padding=(0, pw), dilation=(1, d1), groups=self.groups)
        out = y if out is None else out + y
    if self.bias is not None:
        out = out + self.bias.view(1, -1, 1, 1)
    return out


def patch(model: nn.Module) -> list[str]:
    """就地给中招的卷积换 forward，返回换了哪些（名字）。可重复调用。"""
    done = []
    for name, m in model.named_modules():
        if _needs_fix(m):
            m.forward = types.MethodType(_tapwise_forward, m)
            done.append(name)
    return done
