"""exp010 · 认人声纹 WeSpeaker ECAPA-TDNN512 进 BE-EXHAUST：ONNX → PyTorch（onnx2torch）→ Core ML（BNNS 用同一个 mlmodelc）。

只有 ONNX（`~/vdn-data/models/wespeaker/voxceleb_ECAPA512.onnx`），没有 PyTorch 权重，所以经 onnx2torch 还原成 torch 图再转。
窗长固定 1.5 s → 148 帧 fbank（25 ms 窗、10 ms 移，(24000-400)/160+1），同判断回路（`src/speaker/src/judge.rs:55`）。
产出（写 <输出目录>，不进库）：
  ecapa.in.f32 / ecapa.ort.f32  N 个窗的输入（148×80）与 ORT 输出（192），小端 f32，按窗拼接（ORT = 参照，Python 调；
                                 rustbe 实测 Python 调与 Rust 调 ORT 输出逐位相同）
  ecapa_fp32.mlpackage           Core ML mlprogram fp32
  ecapa_prep.json                版本 · 哈希 · torch 还原图对 ORT 的差 · ORT（Python 调，只作对照）每次耗时
用法：python ecapa_prep.py <onnx> <输出目录> [--n 200]
"""
import hashlib, json, os, sys, time

import numpy as np
import onnx
import onnxruntime as ort
import torch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import coremltools as ct  # noqa: E402
import ct_compat  # noqa: E402,F401
from onnx2torch import convert  # noqa: E402

src, out = sys.argv[1], sys.argv[2]
N = int(sys.argv[sys.argv.index("--n") + 1]) if "--n" in sys.argv else 200
T, F, D = 148, 80, 192
os.makedirs(out, exist_ok=True)
rng = np.random.default_rng(0)
# fbank 经 CMN 后大致零均值；随机输入只为对拍和计时，不评认人准确率
x = (rng.standard_normal((N, 1, T, F)) * 3.0).astype(np.float32)

so = ort.SessionOptions()
so.intra_op_num_threads = 1
so.inter_op_num_threads = 1
so.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
so.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
s = ort.InferenceSession(src, so, providers=["CPUExecutionProvider"])
iname, oname = s.get_inputs()[0].name, s.get_outputs()[0].name
for _ in range(20):
    s.run(None, {iname: x[0]})
ts, ys = [], []
for i in range(N):
    t = time.perf_counter()
    y = s.run(None, {iname: x[i]})[0]
    ts.append((time.perf_counter() - t) * 1e3)
    ys.append(y.reshape(-1))
ys = np.stack(ys).astype("<f4")
assert ys.shape[1] == D, ys.shape
x.astype("<f4").tofile(os.path.join(out, "ecapa.in.f32"))
ys.tofile(os.path.join(out, "ecapa.ort.f32"))
ts = np.sort(ts)
info = {"onnx": src, "onnx_sha256": hashlib.sha256(open(src, "rb").read()).hexdigest(), "T": T, "N": N,
        "onnxruntime": ort.__version__, "torch": torch.__version__, "coremltools": ct.__version__,
        "ort_python_ms": {"p50": round(float(ts[N // 2]), 3), "max": round(float(ts[-1]), 3)}}

def constants_to_initializers(m):
    """onnx2torch 不收「Clip 的 min/max 来自 Constant 节点」（报 Dynamic value of min/max）。把这些 Constant 节点的值
    搬成 initializer（同名），图的计算不变。"""
    from onnx import numpy_helper
    g = m.graph
    have = {i.name for i in g.initializer}
    consts = {}
    for n in g.node:
        if n.op_type == "Constant":
            for at in n.attribute:
                if at.name == "value":
                    consts[n.output[0]] = numpy_helper.to_array(at.t)
    used = {name for n in g.node if n.op_type == "Clip" for name in list(n.input)[1:] if name}
    moved = 0
    for name in used:
        if name in consts and name not in have:
            g.initializer.append(numpy_helper.from_array(consts[name], name))
            moved += 1
    # Clip 省略了 min（输入名为空串）的：补一个 -3.4e38 的 min，计算不变；max 同理
    import numpy as _np
    for n in g.node:
        if n.op_type == "Clip":
            ins = list(n.input)
            for j, fill in ((1, -3.4e38), (2, 3.4e38)):
                if len(ins) > j and ins[j] == "":
                    nm = f"{n.name or n.output[0]}_clip{j}"
                    g.initializer.append(numpy_helper.from_array(_np.array(fill, dtype=_np.float32), nm))
                    ins[j] = nm
                    moved += 1
            del n.input[:]
            n.input.extend(ins)
    keep = [n for n in g.node if not (n.op_type == "Constant" and n.output[0] in used and n.output[0] in consts)]
    del g.node[:]
    g.node.extend(keep)
    return m, moved


om, moved = constants_to_initializers(onnx.load(src))
info["clip_constants_moved"] = moved
tm = convert(om).eval()
with torch.no_grad():
    tr = torch.jit.trace(tm, (torch.from_numpy(x[0]),), check_trace=False)
    d = max(float(np.abs(tr(torch.from_numpy(x[i])).numpy().reshape(-1) - ys[i]).max()) for i in range(5))
info["torch_from_onnx_max_abs_vs_ort"] = d
mlm = ct.convert(tr, inputs=[ct.TensorType(name="feats", shape=(1, T, F), dtype=np.float32)],
                 outputs=[ct.TensorType(name="embs", dtype=np.float32)], convert_to="mlprogram",
                 compute_precision=ct.precision.FLOAT32, minimum_deployment_target=ct.target.macOS15)
p = os.path.join(out, "ecapa_fp32.mlpackage")
mlm.save(p)
h = hashlib.sha256()
for root, _, fs in sorted(os.walk(p)):
    for f in sorted(fs):
        h.update(open(os.path.join(root, f), "rb").read())
info["mlpackage_sha256"] = h.hexdigest()
json.dump(info, open(os.path.join(out, "ecapa_prep.json"), "w"), indent=1)
print(json.dumps(info))
