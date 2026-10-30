"""exp001：在本机 CPU 上量流式 ONNX 每帧推理耗时（单线程）。

只量模型一次 run 的耗时（不含 STFT），输入用固定随机数，状态每帧回灌。
用法：uv run --with onnxruntime --with numpy python bench.py <model.onnx> [...]
"""
import sys
import time

import numpy as np
import onnxruntime as ort

N_WARM = 200
N_FRAMES = 6000  # 16 kHz hop 10 ms → 60 秒音频


def concrete(shape):
    return [d if isinstance(d, int) and d > 0 else 1 for d in shape]


def bench(path):
    so = ort.SessionOptions()
    so.intra_op_num_threads = 1
    so.inter_op_num_threads = 1
    so.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
    so.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
    sess = ort.InferenceSession(path, sess_options=so, providers=["CPUExecutionProvider"])
    ins, outs = sess.get_inputs(), sess.get_outputs()
    rng = np.random.default_rng(0)
    feed = {i.name: (rng.standard_normal(concrete(i.shape)) * 0.01).astype(np.float32) for i in ins}
    # 状态回灌：第 k 个输出（k>=1）喂回第 k 个输入，形状相同才回灌
    loop = [(o.name, i.name) for o, i in zip(outs[1:], ins[1:]) if concrete(o.shape) == concrete(i.shape)]
    names = [o.name for o in outs]
    t = np.empty(N_FRAMES)
    for k in range(N_WARM + N_FRAMES):
        t0 = time.perf_counter()
        res = sess.run(names, feed)
        dt = time.perf_counter() - t0
        for (on, inn) in loop:
            feed[inn] = res[names.index(on)]
        if k >= N_WARM:
            t[k - N_WARM] = dt * 1e3
    print(f"{path.split('/')[-1]}\tinputs={[ (i.name, i.shape) for i in ins]}\tstate_loop={len(loop)}")
    print(f"  ms/frame mean={t.mean():.3f} p50={np.percentile(t,50):.3f} p99={np.percentile(t,99):.3f} "
          f"p99.9={np.percentile(t,99.9):.3f} max={t.max():.3f}")


if __name__ == "__main__":
    print("onnxruntime", ort.__version__)
    for p in sys.argv[1:]:
        bench(p)
