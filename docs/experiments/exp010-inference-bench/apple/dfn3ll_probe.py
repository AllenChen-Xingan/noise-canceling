"""exp010 · DFN3 `_ll` 每帧算力下界探测（不是正式基准，D68：Python 调 ORT，只作估算）。

做法：官方 DeepFilterNet3_ll_onnx 三张子图（enc · erb_dec · df_dec）时间维是符号 S、状态不显式。
这里把 S 取 1，喂随机输入，单线程 ORT 逐次调用，三图耗时相加 = 「每 10 ms 一帧的网络算力」的下界
（GRU 只走 1 步、不带隐状态进出；真正逐帧要重新导出、把状态做成输入输出，见 019 decision）。
不算 STFT / ERB 特征 / deep filtering（这些在 libDF 里由 Rust 做）。
用法：python dfn3ll_probe.py <DeepFilterNet3_ll_onnx.tar.gz> <解包目录> [--calls 3000]
"""
import json, os, sys, tarfile, time
import numpy as np
import onnxruntime as ort

tgz, out = sys.argv[1], sys.argv[2]
calls = int(sys.argv[sys.argv.index("--calls") + 1]) if "--calls" in sys.argv else 3000
os.makedirs(out, exist_ok=True)
with tarfile.open(tgz) as t:
    t.extractall(out, filter="data")
paths = {}
for root, _, fs in os.walk(out):
    for f in fs:
        if f.endswith(".onnx"):
            paths[f[:-5]] = os.path.join(root, f)
so = ort.SessionOptions()
so.intra_op_num_threads = 1
so.inter_op_num_threads = 1
so.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
so.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
rng = np.random.default_rng(0)
res = {"onnxruntime": ort.__version__, "calls": calls, "graphs": {}}
for name in ("enc", "erb_dec", "df_dec"):
    s = ort.InferenceSession(paths[name], so, providers=["CPUExecutionProvider"])
    feeds = {}
    shapes = {}
    # 先按输入形状造 S=1 的输入；df_dec / erb_dec 的输入来自 enc 的输出，这里统一用随机数，只为计时
    for i in s.get_inputs():
        sh = [1 if (not isinstance(d, int) or d <= 0) else d for d in i.shape]
        shapes[i.name] = sh
        feeds[i.name] = (rng.standard_normal(sh) * 0.1).astype(np.float32)
    for _ in range(200):
        s.run(None, feeds)
    ts = []
    for _ in range(calls):
        t = time.perf_counter()
        s.run(None, feeds)
        ts.append((time.perf_counter() - t) * 1e3)
    ts = np.sort(ts)
    res["graphs"][name] = {"inputs": shapes, "p50_ms": round(float(ts[len(ts) // 2]), 4),
                           "p999_ms": round(float(ts[int(len(ts) * 0.999)]), 4), "max_ms": round(float(ts[-1]), 4)}
tot = sum(g["p50_ms"] for g in res["graphs"].values())
res["sum_p50_ms"] = round(tot, 4)
res["core_share_at_10ms_hop"] = round(tot / 10.0, 4)  # 每 10 ms 一帧，占一个核的比例
print(json.dumps(res, ensure_ascii=False))
