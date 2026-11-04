"""把 Distill-MOS v7 的「不分段」前向（固定 122,880 样本输入，含逐段峰值归一）导出成 ONNX。

分段与求均值不在图里，由 Rust（`src/qa/nrmos.rs` distill_segments）照上游 sqa.py forward 做。
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
x = torch.zeros(1, SEQ)
tmp = out.with_suffix(".tmp.onnx")
torch.onnx.export(m, (x,), str(tmp), input_names=["wav"], output_names=["mos"],
                  opset_version=17, dynamo=False)

rng = np.random.default_rng(20261103)
s = ort.InferenceSession(str(tmp), providers=["CPUExecutionProvider"])
worst = 0.0
for i in range(4):
    a = (rng.standard_normal((1, SEQ)) * 0.1 * (i + 1)).astype(np.float32)
    with torch.no_grad():
        t = float(m(torch.from_numpy(a))[0])
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
print(f"torch↔onnx 最大差 {worst:.2e}")
