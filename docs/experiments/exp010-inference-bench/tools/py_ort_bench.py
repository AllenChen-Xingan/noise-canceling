"""Python onnxruntime 跑 GTCRN，与 Rust bench 同口径同输入，看 Rust 与 Python 的每帧耗时差多少。

输入：Rust bench `gtcrn-bench ort <onnx> <wav> --dump-io <前缀>` 写出的 `<前缀>.in.f32`（每帧 re[257] 再 im[257]）
与 `<前缀>.ort.f32`（Rust ORT 从零状态顺序跑的输出，同布局）。两边吃逐位相同的输入。
帧序：第 0 帧冷启动、接着 warmup 帧、再计时帧（与 bench 默认 1 + 200 + 6000 一致，由 json 的 frames_total 定）。
会话设置照 exp001 `bench.py::session`：intra = inter = 1、ORT_SEQUENTIAL、CPUExecutionProvider；
图优化用默认（Python 默认即 ORT_ENABLE_ALL，同 Rust bench）。
计时：只计 `sess.run` 一次调用（同 exp001 的 t_model）；张量拼装、缓存回灌不计。
另报 Python 输出对 Rust ORT 输出的逐值一致（max_abs · snr_db · first_bad > 1e-3），同 bench parity 算法。
输出一行 JSON，字段同 Rust bench；Python 侧不数分配（无对应字段）。

用法：python py_ort_bench.py <onnx> <dump 前缀> [--warmup 200]
"""
import json
import resource
import sys
import time

import ctypes

import numpy as np
import onnxruntime as ort

# 计时线程调度等级同 Rust bench 默认（--qos ui = QOS_CLASS_USER_INTERACTIVE = 0x21）
_r = ctypes.CDLL("/usr/lib/libSystem.B.dylib").pthread_set_qos_class_self_np(0x21, 0)
assert _r == 0, f"设 QoS 失败：{_r}"

onnx, prefix = sys.argv[1], sys.argv[2]
warm = int(sys.argv[sys.argv.index("--warmup") + 1]) if "--warmup" in sys.argv else 200
meta = json.load(open(prefix + ".json"))
n, L = meta["frames_total"], meta["frame_len"]
assert meta["model"] == "gtcrn" and L == 514
xin = np.fromfile(prefix + ".in.f32", dtype="<f4").reshape(n, L)
ref = np.fromfile(prefix + ".ort.f32", dtype="<f4").reshape(n, L)
frames = n - 1 - warm


def cpu():
    u = resource.getrusage(resource.RUSAGE_SELF)
    return u.ru_utime + u.ru_stime


t0 = time.perf_counter()
so = ort.SessionOptions()
so.intra_op_num_threads = 1
so.inter_op_num_threads = 1
so.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
sess = ort.InferenceSession(onnx, sess_options=so, providers=["CPUExecutionProvider"])
t_open = (time.perf_counter() - t0) * 1e3
ins = [i.name for i in sess.get_inputs()]
outs = [o.name for o in sess.get_outputs()]
caches = {i.name: np.zeros(i.shape, dtype=np.float32) for i in sess.get_inputs()[1:]}

times = np.empty(frames)
max_abs, sig, err, worst, first_bad, nonfinite = 0.0, 0.0, 0.0, 0, None, 0
t_first = 0.0
c0 = w0 = None
for k in range(n):
    f = xin[k]
    mix = np.stack([f[:257], f[257:]], -1)[None, :, None].astype(np.float32)  # [1,257,1,2]
    feed = {ins[0]: mix, **caches}
    if k == 1 + warm:
        c0, w0 = cpu(), time.perf_counter()
    a = time.perf_counter()
    res = sess.run(outs, feed)
    b = time.perf_counter()
    if k == 0:
        t_first = (b - a) * 1e3
    elif k > warm:
        times[k - 1 - warm] = (b - a) * 1e3
    for i_name, o_name in zip(ins[1:], outs[1:]):
        caches[i_name] = res[outs.index(o_name)]
    y = res[0].reshape(257, 2)
    y = np.concatenate([y[:, 0], y[:, 1]]).astype(np.float64)
    if not np.all(np.isfinite(y)):
        nonfinite += 1
    d = np.abs(y - ref[k].astype(np.float64))
    sig += float(np.sum(ref[k].astype(np.float64) ** 2))
    err += float(np.sum(d * d))
    if d.max() > max_abs:
        max_abs, worst = float(d.max()), k
    if first_bad is None and (not np.all(np.isfinite(y)) or not d.max() <= 1e-3):  # NaN 也算坏
        first_bad = (k, int(d.argmax()))
wall = time.perf_counter() - w0
cpu_s = cpu() - c0
times.sort()
pct = lambda p: times[min(int(round((len(times) - 1) * p)), len(times) - 1)]  # 同 bench 的 pct()
print(json.dumps({
    "model": "gtcrn", "backend": f"python-ort-{ort.__version__}", "frames": frames, "pace_ms": 0, "qos": "ui",
    "p10_ms": round(pct(0.1), 4), "p50_ms": round(pct(0.5), 4), "p90_ms": round(pct(0.9), 4), "p99_ms": round(pct(0.99), 4), "p999_ms": round(pct(0.999), 4),
    "max_ms": round(times[-1], 4), "mean_ms": round(times.mean(), 4), "open_ms": round(t_open, 2),
    "first_frame_ms": round(t_first, 3), "cpu_over_wall": round(cpu_s / wall, 3), "nonfinite_frames": nonfinite,
    "vs_rust_ort": {"max_abs": max_abs, "worst_frame": worst,
                    "snr_db": round(10 * np.log10(sig / max(err, 1e-300)), 2), "first_bad": str(first_bad)},
}, ensure_ascii=False))
