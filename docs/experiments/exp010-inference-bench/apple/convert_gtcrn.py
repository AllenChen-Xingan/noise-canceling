"""exp010 · 把 GTCRN 流式模型（DNS3 权重，与 gtcrn_simple.onnx 同源）转成 Core ML。

产出（写到 <输出目录>，不进库）：
  gtcrn_fp32.mlpackage   mlprogram，compute_precision=FLOAT32，三个缓存是显式输入输出（与 ONNX 同接口）
  gtcrn_fp16.mlpackage   同上，FLOAT16（神经引擎只跑 fp16，测它要这一版）
  gtcrn_state_fp16.mlpackage  有状态版（ct.StateType，缓存留在模型里），只在 coremltools 支持时产出
  gtcrn_unrolled_fp32.mlpackage  GRU 手动展开版（每个时间步写成普通矩阵乘和逐元素算子）。
      原因：coremltools 把 torch 的 GRU 翻成逐步 while_loop（019 notes/coreml.md），展开后看 Core ML / BNNS 能否整图融合
  convert.json           coremltools / torch 版本、各文件 sha256

用法：python convert_gtcrn.py <resources/gtcrn/stream 目录> <输出目录> [fp32|fp16|unrolled|state]
每个变体单独一个进程跑（fp16 那一步 coremltools 9.0 + torch 2.15 会在原生代码里崩，崩了不连累别的），记录写 convert_<变体>.json。
名字与 ONNX 一致：输入 mix conv_cache tra_cache inter_cache；输出 enh conv_cache_out tra_cache_out inter_cache_out。
"""
import hashlib, json, os, sys, types

import numpy as np
import torch

src, out = sys.argv[1], sys.argv[2]
ONLY = sys.argv[3] if len(sys.argv) > 3 else None
want = lambda v: ONLY is None or ONLY == v
# coremltools 的 Python 侧 predict 在本机偶发段错误（2026-11-02 实测）；逐值一致由 apple-rs 对 ORT 转储判，这里默认不跑
SELFCHECK = os.environ.get("CT_SELFCHECK") == "1"
os.makedirs(out, exist_ok=True)
sys.path.insert(0, src)
sys.modules.setdefault("einops", types.SimpleNamespace(rearrange=None))  # 源码 import 了但没用
import coremltools as ct  # noqa: E402
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ct_compat  # noqa: E402,F401  coremltools 9.0 × numpy 2.5 补丁
from gtcrn import GTCRN  # noqa: E402
from gtcrn_stream import StreamGTCRN  # noqa: E402
from modules.convert import convert_to_stream  # noqa: E402

import gtcrn_stream as _gs  # noqa: E402


# ── 把缓存的原地切片写回（conv_cache[:, :, :2, :] = …）换成拼接出新张量 ──
# 原因（2026-11-02 实测）：coremltools 9.0 转原地写回的图，第 0 帧就和 ORT 差 0.77；torch 追踪图本身对 ORT 差 7e-7。
# 算法不变，只是不再原地改；改完由 apple-rs 对 ORT 转储判逐值一致。
def _enc_forward(self, x, conv_cache, tra_cache):
    en_outs = []
    for i in range(2):
        x = self.en_convs[i](x)
        en_outs.append(x)
    x, c0, t0 = self.en_convs[2](x, conv_cache[:, :, :2, :], tra_cache[0]); en_outs.append(x)
    x, c1, t1 = self.en_convs[3](x, conv_cache[:, :, 2:6, :], tra_cache[1]); en_outs.append(x)
    x, c2, t2 = self.en_convs[4](x, conv_cache[:, :, 6:16, :], tra_cache[2]); en_outs.append(x)
    return x, en_outs, torch.cat([c0, c1, c2], dim=2), torch.stack([t0, t1, t2])


def _dec_forward(self, x, en_outs, conv_cache, tra_cache):
    x, c6, t0 = self.de_convs[0](x + en_outs[4], conv_cache[:, :, 6:16, :], tra_cache[0])
    x, c2, t1 = self.de_convs[1](x + en_outs[3], conv_cache[:, :, 2:6, :], tra_cache[1])
    x, c0, t2 = self.de_convs[2](x + en_outs[2], conv_cache[:, :, :2, :], tra_cache[2])
    for i in range(3, 5):
        x = self.de_convs[i](x + en_outs[4 - i])
    return x, torch.cat([c0, c2, c6], dim=2), torch.stack([t0, t1, t2])


def _gtcrn_forward(self, spec, conv_cache, tra_cache, inter_cache):
    spec_ref = spec
    spec_real = spec[..., 0].permute(0, 2, 1)
    spec_imag = spec[..., 1].permute(0, 2, 1)
    spec_mag = torch.sqrt(spec_real ** 2 + spec_imag ** 2 + 1e-12)
    feat = torch.stack([spec_mag, spec_real, spec_imag], dim=1)
    feat = self.sfe(self.erb.bm(feat))
    feat, en_outs, ce, te = self.encoder(feat, conv_cache[0], tra_cache[0])
    feat, i0 = self.dpgrnn1(feat, inter_cache[0])
    feat, i1 = self.dpgrnn2(feat, inter_cache[1])
    m_feat, cd, td = self.decoder(feat, en_outs, conv_cache[1], tra_cache[1])
    m = self.erb.bs(m_feat)
    spec_enh = self.mask(m, spec_ref.permute(0, 3, 2, 1)).permute(0, 3, 2, 1)
    return spec_enh, torch.stack([ce, cd]), torch.stack([te, td]), torch.stack([i0, i1])


if os.environ.get("GTCRN_INPLACE") != "1":  # 设 1 可复现原版写法（用来对照）
    _gs.StreamEncoder.forward = _enc_forward
    _gs.StreamDecoder.forward = _dec_forward
    _gs.StreamGTCRN.forward = _gtcrn_forward

class SplitConvT(torch.nn.Module):
    """分组 ConvTranspose2d → 每组一个不分组的 ConvTranspose2d，再在通道上拼起来。数学上等价。
    原因（2026-11-02 实测，逐块对拍）：coremltools 9.0 转 groups=2 的 ConvTranspose2d（解码器第 4 层）差 3.46，其余各块都 ≤ 1.2e-5。"""

    def __init__(self, ct_):
        super().__init__()
        g = ct_.groups
        ig, og = ct_.in_channels // g, ct_.out_channels // g
        self.ig = ig
        self.parts = torch.nn.ModuleList()
        for k in range(g):
            m = torch.nn.ConvTranspose2d(ig, og, ct_.kernel_size, ct_.stride, ct_.padding, ct_.output_padding,
                                         groups=1, bias=ct_.bias is not None, dilation=ct_.dilation)
            m.weight.data.copy_(ct_.weight.data[k * ig:(k + 1) * ig])
            if ct_.bias is not None:
                m.bias.data.copy_(ct_.bias.data[k * og:(k + 1) * og])
            self.parts.append(m)

    def forward(self, x):
        return torch.cat([m(x[:, k * self.ig:(k + 1) * self.ig]) for k, m in enumerate(self.parts)], dim=1)


def split_grouped_convt(m):
    for name, child in list(m.named_children()):
        if isinstance(child, torch.nn.ConvTranspose2d) and child.groups > 1:
            setattr(m, name, SplitConvT(child))
        else:
            split_grouped_convt(child)
    return m


base = GTCRN().eval()
base.load_state_dict(torch.load(f"{src}/onnx_models/model_trained_on_dns3.tar", map_location="cpu")["model"])
stream = StreamGTCRN().eval()
convert_to_stream(stream, base)
split_grouped_convt(stream)


class Explicit(torch.nn.Module):
    """缓存显式进出。先 clone，避免原地改写输入（trace 时会变成对图输入的写，转换器不收）。"""

    def __init__(self, m):
        super().__init__()
        self.m = m

    def forward(self, mix, conv, tra, inter):
        return self.m(mix, conv.clone(), tra.clone(), inter.clone())


class UnrolledGRU(torch.nn.Module):
    """单层 nn.GRU（batch_first）的逐步展开版，权重与原模块共用，trace 时循环被展开成直线图。门顺序 r,z,n 同 torch。"""

    def __init__(self, g):
        super().__init__()
        assert g.num_layers == 1 and g.batch_first
        self.g, self.bi, self.H = g, g.bidirectional, g.hidden_size

    def _dir(self, x, h, sfx, reverse):
        g, H = self.g, self.H
        wih, whh = getattr(g, "weight_ih_l0" + sfx), getattr(g, "weight_hh_l0" + sfx)
        bih, bhh = getattr(g, "bias_ih_l0" + sfx), getattr(g, "bias_hh_l0" + sfx)
        gi_all = torch.nn.functional.linear(x, wih, bih)  # (B,L,3H)，输入那一半整段一次算
        steps = range(x.shape[1] - 1, -1, -1) if reverse else range(x.shape[1])
        outs = [None] * x.shape[1]
        for t in steps:
            gi = gi_all[:, t]
            gh = torch.nn.functional.linear(h, whh, bhh)
            r = torch.sigmoid(gi[:, :H] + gh[:, :H])
            z = torch.sigmoid(gi[:, H:2 * H] + gh[:, H:2 * H])
            n = torch.tanh(gi[:, 2 * H:] + r * gh[:, 2 * H:])
            h = (1 - z) * n + z * h
            outs[t] = h
        return torch.stack(outs, dim=1), h

    def forward(self, x, h0=None):
        if h0 is None:
            h0 = x.new_zeros(2 if self.bi else 1, x.shape[0], self.H)
        yf, hf = self._dir(x, h0[0], "", False)
        if not self.bi:
            return yf, hf[None]
        yb, hb = self._dir(x, h0[1], "_reverse", True)
        return torch.cat([yf, yb], dim=-1), torch.stack([hf, hb])


def unroll_grus(m):
    for name, child in list(m.named_children()):
        if isinstance(child, torch.nn.GRU):
            setattr(m, name, UnrolledGRU(child))
        else:
            unroll_grus(child)
    return m


SHAPES = {"mix": (1, 257, 1, 2), "conv_cache": (2, 1, 16, 16, 33), "tra_cache": (2, 3, 1, 1, 16), "inter_cache": (2, 1, 33, 16)}
OUTS = ["enh", "conv_cache_out", "tra_cache_out", "inter_cache_out"]
ex = tuple(torch.zeros(s) for s in SHAPES.values())
ex = (torch.randn(SHAPES["mix"]),) + ex[1:]

with torch.no_grad():
    traced = torch.jit.trace(Explicit(stream).eval(), ex, check_trace=False)
    ref = traced(*ex)

info = {"torch": torch.__version__, "coremltools": ct.__version__, "files": {},
        "gtcrn_inplace": os.environ.get("GTCRN_INPLACE") == "1", "split_grouped_convt": True}


def save(mlm, name):
    p = os.path.join(out, name)
    mlm.save(p)
    h = hashlib.sha256()
    for root, _, fs in sorted(os.walk(p)):
        for f in sorted(fs):
            with open(os.path.join(root, f), "rb") as fh:
                h.update(fh.read())
    info["files"][name] = h.hexdigest()


for prec, tag in [(ct.precision.FLOAT32, "fp32"), (ct.precision.FLOAT16, "fp16")]:
    if not want(tag):
        continue
    mlm = ct.convert(
        traced,
        inputs=[ct.TensorType(name=k, shape=s, dtype=np.float32) for k, s in SHAPES.items()],
        outputs=[ct.TensorType(name=n, dtype=np.float32) for n in OUTS],
        convert_to="mlprogram",
        compute_precision=prec,
        minimum_deployment_target=ct.target.macOS15,
    )
    # 转换端自检：CPU 上和 torch 比一次（不是逐值一致验收，验收走 bench 对 ORT 输出）
    if SELFCHECK:
        got = mlm.predict({k: v.numpy() for k, v in zip(SHAPES, ex)})
        info[f"selfcheck_{tag}_max_abs_vs_torch"] = float(np.abs(got["enh"] - ref[0].numpy()).max())
    save(mlm, f"gtcrn_{tag}.mlpackage")

# GRU 展开版（fp32 显式缓存）：先在 torch 里和原版比一次，确认展开不改输出
import copy  # noqa: E402

if want("unrolled"):
  unrolled = unroll_grus(copy.deepcopy(stream)).eval()
  with torch.no_grad():
      tu = torch.jit.trace(Explicit(unrolled).eval(), ex, check_trace=False)
      info["unrolled_torch_max_abs_vs_orig"] = float((tu(*ex)[0] - ref[0]).abs().max())
  mlu = ct.convert(
      tu,
      inputs=[ct.TensorType(name=k, shape=s, dtype=np.float32) for k, s in SHAPES.items()],
      outputs=[ct.TensorType(name=n, dtype=np.float32) for n in OUTS],
      convert_to="mlprogram",
      compute_precision=ct.precision.FLOAT32,
      minimum_deployment_target=ct.target.macOS15,
  )
  if SELFCHECK:
      got = mlu.predict({k: v.numpy() for k, v in zip(SHAPES, ex)})
      info["selfcheck_unrolled_fp32_max_abs_vs_torch"] = float(np.abs(got["enh"] - ref[0].numpy()).max())
  save(mlu, "gtcrn_unrolled_fp32.mlpackage")

# 有状态版：缓存作为 register_buffer 留在模型里，每帧只进 mix 出 enh
try:
    if not want("state"):
        raise StopIteration
    class Stateful(torch.nn.Module):
        def __init__(self, m):
            super().__init__()
            self.m = m
            self.register_buffer("conv_state", torch.zeros(SHAPES["conv_cache"]))
            self.register_buffer("tra_state", torch.zeros(SHAPES["tra_cache"]))
            self.register_buffer("inter_state", torch.zeros(SHAPES["inter_cache"]))

        def forward(self, mix):
            enh, c, t, i = self.m(mix, self.conv_state.clone(), self.tra_state.clone(), self.inter_state.clone())
            self.conv_state.copy_(c)
            self.tra_state.copy_(t)
            self.inter_state.copy_(i)
            return enh

    with torch.no_grad():
        ts = torch.jit.trace(Stateful(stream).eval(), (ex[0],), check_trace=False)
    mls = ct.convert(
        ts,
        inputs=[ct.TensorType(name="mix", shape=SHAPES["mix"], dtype=np.float32)],  # 进出口 fp32，内部和状态 fp16
        outputs=[ct.TensorType(name="enh", dtype=np.float32)],
        states=[
            ct.StateType(wrapped_type=ct.TensorType(shape=SHAPES[k + "_cache"], dtype=np.float16), name=k + "_state")
            for k in ("conv", "tra", "inter")
        ],
        convert_to="mlprogram",
        compute_precision=ct.precision.FLOAT16,
        minimum_deployment_target=ct.target.macOS15,
    )
    save(mls, "gtcrn_state_fp16.mlpackage")
except StopIteration:
    pass
except Exception as e:  # 有状态只是加分项，失败照记
    info["stateful_error"] = repr(e)[:500]

with open(os.path.join(out, f"convert_{ONLY or 'all'}.json"), "w") as f:
    json.dump(info, f, indent=1, ensure_ascii=False)
print(json.dumps(info, ensure_ascii=False))
