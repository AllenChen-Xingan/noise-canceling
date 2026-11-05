"""数每帧乘加次数（MAC）和参数量：在真模型上挂钩子数，不手抄层表。

口径：
- 卷积：输出元素数 × (输入通道 / 组数) × 核高 × 核宽；转置卷积用输入元素数 × (输出通道 / 组数) × 核面积
- 线性层：输出元素数 × 输入维度（含 ERB 的两个固定矩阵）
- GRU：每步每方向 3 × (输入 × 隐层 + 隐层 × 隐层)，再乘步数 × 批 × 方向数
- 候选模块手写的矩阵乘：模块自报 `extra_macs`
- 不计：激活、归一化、逐元素乘加、softmax（量级小，另由测速覆盖）
"""
import torch
import torch.nn as nn


def _conv(mod, inp, out):
    k = mod.kernel_size[0] * mod.kernel_size[1]
    if isinstance(mod, nn.ConvTranspose2d):
        return inp[0].numel() * (mod.out_channels // mod.groups) * k
    return out.numel() * (mod.in_channels // mod.groups) * k


def _linear(mod, inp, out):
    return out.numel() * mod.in_features


def _gru(mod, inp, out):
    x = inp[0]
    b, t = (x.shape[0], x.shape[1]) if mod.batch_first else (x.shape[1], x.shape[0])
    dirs = 2 if mod.bidirectional else 1
    per = 3 * (mod.input_size * mod.hidden_size + mod.hidden_size ** 2)
    return b * t * dirs * per * mod.num_layers


RULES = [((nn.Conv2d, nn.ConvTranspose2d), _conv), (nn.Linear, _linear), (nn.GRU, _gru)]


def count_frame(model, cond=None):
    """推一帧，返回 {'total': 总乘加, 'extra': 候选部分乘加, 'by_type': {...}}。"""
    tally = {"conv": 0, "linear": 0, "gru": 0}
    extra_ids = {id(m) for m in model.extra.modules()}
    extra_tally = [0]
    hooks = []
    for mod in model.modules():
        for types, fn in RULES:
            if isinstance(mod, types):
                key = {_conv: "conv", _linear: "linear", _gru: "gru"}[fn]

                def hook(m, i, o, fn=fn, key=key, mine=id(mod) in extra_ids):
                    o0 = o[0] if isinstance(o, tuple) else o
                    n = fn(m, i, o0)
                    tally[key] += n
                    if mine:
                        extra_tally[0] += n
                hooks.append(mod.register_forward_hook(hook))
    try:
        with torch.no_grad():
            st = model.init_state(1)
            model(torch.randn(1, 257, 1, 2), st, cond)
    finally:
        for h in hooks:
            h.remove()
    hand = int(getattr(model.extra, "extra_macs", 0))
    return {
        "total": sum(tally.values()) + hand,
        "extra": extra_tally[0] + hand,
        "by_type": dict(tally, hand=hand),
    }


def count_params(module, trainable_only=False):
    return sum(p.numel() for p in module.parameters()
               if p.requires_grad or not trainable_only)


def state_bytes(state):
    """流式状态（缓存）占多少字节（float32）。"""
    if state is None:
        return 0
    if isinstance(state, torch.Tensor):
        return state.numel() * state.element_size()
    if isinstance(state, dict):
        return sum(state_bytes(v) for v in state.values())
    return sum(state_bytes(v) for v in state)
