"""exp010 · 把 FastEnhancer-M（DNS 权重，与 onnx-dns-v1.0.0/fastenhancer_m.onnx 同源）的 wav2wav 流式图转成 Core ML。

权重：官方 release ckpt-dns-v1.0.0/fastenhancer_m.zip（sha256 6f25dd9e…ffe0b），解出的 00500.pth + config.yaml。
图与官方 scripts/export_onnx.py 的 Model 一样：STFT → 网络 → iSTFT 都在图里，每次进 160 个样本、出 160 个样本，6 个缓存。
名字与 ONNX 一致：输入 wav_in cache_in_0..5；输出 wav_out cache_out_0..5。

用法：python convert_fem.py <resources/fastenhancer 目录> <ckpt 目录（含 00500.pth、config.yaml）> <输出目录>
产出：fem_fp32.mlpackage · fem_fp16.mlpackage · convert_fem.json（版本、sha256、和 torch 的自检差）
"""
import hashlib, importlib, json, os, sys

import numpy as np
import torch

src, ckdir, out = sys.argv[1], sys.argv[2], sys.argv[3]
os.makedirs(out, exist_ok=True)
sys.path.insert(0, src)
os.chdir(src)  # 上游代码按仓库根目录相对导入
import importlib.util, types  # noqa: E402

import coremltools as ct  # noqa: E402

# 只要 hparams 一个文件：utils/__init__ 会拉进训练用的画图、评测依赖；librosa 只在 mel 滤波器里用到，导出图不走那里
sys.modules.setdefault("librosa", types.ModuleType("librosa"))
sys.modules.setdefault("librosa.filters", types.SimpleNamespace(mel=None))
_spec = importlib.util.spec_from_file_location("fe_hparams", os.path.join(src, "utils", "hparams.py"))
_hp = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_hp)
get_hparams = _hp.get_hparams

hps = get_hparams(os.path.join(ckdir, "config.yaml"))
mod = importlib.import_module(f"models.{hps.model}.model")
net = mod.ONNXModel(**hps.model_kwargs)
sd = torch.load(os.path.join(ckdir, "00500.pth"), map_location="cpu")["model"]
sd = {k.removeprefix("module."): v for k, v in sd.items()}
net.load_state_dict(sd, strict=True)
net.remove_weight_reparameterizations()
net.eval()


class W2W(torch.nn.Module):
    """同 export_onnx.Model.forward。"""

    def __init__(self, m):
        super().__init__()
        self.model = m

    def forward(self, wav_in, cache_stft, cache_istft, *cache_model):
        spec_in, cache_stft = self.model.stft(wav_in, cache_stft)
        spec_out, *cache_model = self.model(spec_in, *cache_model)
        wav_out, cache_istft = self.model.stft.inverse(spec_out, cache_istft)
        return (wav_out, cache_stft, cache_istft, *cache_model)


w = W2W(net).eval()
caches = net.stft.initialize_cache(torch.randn(1)) + net.initialize_cache(torch.randn(1))
hop = hps.model_kwargs.hop_size
ex = (torch.randn(1, hop) * 0.1, *caches)
with torch.no_grad():
    traced = torch.jit.trace(w, ex, check_trace=False)
    ref = traced(*ex)

names = ["wav_in"] + [f"cache_in_{i}" for i in range(len(caches))]
outs = ["wav_out"] + [f"cache_out_{i}" for i in range(len(caches))]
info = {"torch": torch.__version__, "coremltools": ct.__version__, "hop": hop,
        "shapes": {n: list(t.shape) for n, t in zip(names, ex)}, "files": {}}

for prec, tag in [(ct.precision.FLOAT32, "fp32"), (ct.precision.FLOAT16, "fp16")]:
    mlm = ct.convert(
        traced,
        inputs=[ct.TensorType(name=n, shape=tuple(t.shape), dtype=np.float32) for n, t in zip(names, ex)],
        outputs=[ct.TensorType(name=n, dtype=np.float32) for n in outs],
        convert_to="mlprogram",
        compute_precision=prec,
        minimum_deployment_target=ct.target.macOS15,
    )
    got = mlm.predict({n: t.numpy() for n, t in zip(names, ex)})
    info[f"selfcheck_{tag}_max_abs_vs_torch"] = float(np.abs(got["wav_out"] - ref[0].numpy()).max())
    p = os.path.join(out, f"fem_{tag}.mlpackage")
    mlm.save(p)
    h = hashlib.sha256()
    for root, _, fs in sorted(os.walk(p)):
        for f in sorted(fs):
            with open(os.path.join(root, f), "rb") as fh:
                h.update(fh.read())
    info["files"][os.path.basename(p)] = h.hexdigest()

with open(os.path.join(out, "convert_fem.json"), "w") as f:
    json.dump(info, f, indent=1, ensure_ascii=False)
print(json.dumps(info, ensure_ascii=False))
