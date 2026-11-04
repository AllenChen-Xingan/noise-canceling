"""把 Distill-MOS v7 的「不分段」前向（固定 122,880 样本输入，含逐段峰值归一）导出成 ONNX。

分段与求均值不在图里，由 Rust（`src/qa/nrmos.rs` distill_segments）照上游 sqa.py forward 做。
torch 的 ONNX 导出器不支持复数 STFT，所以导出时把 sqa.py 的 `_complex_compressed` 换成同一算法的实数版
（分帧 × 周期 Hann 窗 × DFT 矩阵，center=False，输出形状同 view_as_real）；数值是否一致由下面的核对保证：
参考输出取**未替换**的原模型，ONNX 输出与它比。
导出后在本脚本里当场核对：同一组随机输入，torch 与 onnxruntime 的输出差 ≤ 1e-3，否则不写文件。
跑法（exp009 的 Python 环境，带 distillmos 0.9.1 与 torch）：
    <主目录>/_tmp/exp009-venv/bin/python -I src/eval/tools/export_distillmos.py <输出目录>
输出：<输出目录>/distill_mos_v7-seg122880.onnx，并打印 sha256（填进 nrmos.rs 的 DISTILL_SHA256）。
"""
import hashlib, pathlib, sys
import numpy as np
import torch
import distillmos
import onnxruntime as ort

SEQ = 122880
out_dir = pathlib.Path(sys.argv[1]).expanduser()
out_dir.mkdir(parents=True, exist_ok=True)
out = out_dir / "distill_mos_v7-seg122880.onnx"
if out.exists():
    sys.exit(f"已存在，不覆盖：{out}")

m = distillmos.ConvTransformerSQAModel(segmenting_in_forward=False).eval()
rng = np.random.default_rng(20261103)
probe = [(rng.standard_normal((1, SEQ)) * 0.1 * (i + 1)).astype(np.float32) for i in range(4)]
with torch.no_grad():
    ref = [float(m(torch.from_numpy(a))[0]) for a in probe]  # 原模型（复数 STFT）

orig = distillmos.sqa._complex_compressed
import torch.nn.functional as F

def real_complex_compressed(x, hop_length, win_length):
    n_fft = win_length
    pad = int((win_length - hop_length) / 2)
    x = F.pad(x, (pad, pad), mode="reflect")
    # conv1d 等价于「分帧 × 窗 × DFT」（导出器不支持 unfold）
    k = torch.arange(n_fft // 2 + 1, dtype=torch.float32)[:, None]
    n = torch.arange(n_fft, dtype=torch.float32)[None, :]
    ang = 2 * np.pi * k * n / n_fft
    w = torch.hann_window(win_length)[None, :]
    kc = (torch.cos(ang) * w)[:, None, :]
    ks = (-torch.sin(ang) * w)[:, None, :]
    xi = x[:, None, :]
    re = F.conv1d(xi, kc, stride=hop_length)  # (B, F, T)
    im = F.conv1d(xi, ks, stride=hop_length)
    x = torch.stack([re, im], dim=-1)  # (B, F, T, 2)
    mag = torch.sum(x**2, dim=-1, keepdim=True) ** 0.5
    return torch.clip(mag, 1e-12) ** 0.3 * x / torch.clip(mag, 1e-12)

# 先在 torch 里核对实数版与原版一致
distillmos.sqa._complex_compressed = real_complex_compressed
with torch.no_grad():
    patched = [float(m(torch.from_numpy(a))[0]) for a in probe]
print("原版 vs 实数版（torch）最大差", max(abs(a - b) for a, b in zip(ref, patched)))
x = torch.zeros(1, SEQ)
tmp = out.with_suffix(".tmp.onnx")
torch.onnx.export(m, (x,), str(tmp), input_names=["wav"], output_names=["mos"],
                  opset_version=17, dynamo=False)

s = ort.InferenceSession(str(tmp), providers=["CPUExecutionProvider"])
worst = 0.0
for i, a in enumerate(probe):
    t = ref[i]
    o = float(s.run(None, {"wav": a})[0].reshape(-1)[0])
    worst = max(worst, abs(t - o))
    print(f"随机输入 {i}: torch {t:.5f} · onnx {o:.5f}")
if worst > 1e-3:
    tmp.unlink()
    sys.exit(f"torch 与 onnx 差 {worst:.2e} > 1e-3，不写文件")
tmp.rename(out)
h = hashlib.sha256(out.read_bytes()).hexdigest()
print(f"已写 {out}（{out.stat().st_size} B）")
print(f"sha256 {h}")
print(f"原模型↔onnx 最大差 {worst:.2e}")
print(f"来源：distillmos {distillmos.__version__ if hasattr(distillmos, '__version__') else '0.9.1'} · torch {torch.__version__} · opset 17")
