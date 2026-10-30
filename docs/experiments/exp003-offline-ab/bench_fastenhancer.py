"""exp003 附：FastEnhancer（DNS 版官方 wav2wav ONNX）B / S / M 三档单线程逐帧计时，口径同 exp001。

- 输入同 exp001：resources/gtcrn/test_wavs/mix.wav（16 kHz），循环拼够长度
- 每次喂一个 hop（B/S 256 样本 = 16 ms，M 160 样本 = 10 ms），缓存原样回灌（官方 scripts/test_onnx.py 的做法）
- STFT / iSTFT / OLA 都在官方 ONNX 图里，所以计的就是整步耗时
- 热身 200 步、计时 6000 步；统计 p50 / p99 / p99.9 / max，并核对输出全是有限值、输出与输入能量比
- 选档规则（大总管定）：「最坏一帧 ≤ 周期一半」里最大的一档
输出：docs/experiments/exp003-offline-ab/fastenhancer_bench.json
用法（worktree 根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp003-offline-ab/bench_fastenhancer.py
"""
import json
import platform
import time
from pathlib import Path

import numpy as np
import onnxruntime as ort
import soundfile as sf

from _paths import RES, ROOT

HERE = ROOT / "docs/experiments/exp003-offline-ab"
WAV = RES / "gtcrn/test_wavs/mix.wav"
MODEL_DIR = Path.home() / "vdn-data/models/fastenhancer-dns-v1.0.0"
N_WARM, N_TIMED = 200, 6000
SR = 16000


def session(path):
    so = ort.SessionOptions()
    so.intra_op_num_threads = 1
    so.inter_op_num_threads = 1
    so.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
    so.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
    return ort.InferenceSession(str(path), sess_options=so, providers=["CPUExecutionProvider"])


def run(size, x):
    sess = session(MODEL_DIR / f"fastenhancer_{size}.onnx")
    hop = sess.get_inputs()[0].shape[-1]
    feed = {i.name: np.zeros(i.shape, np.float32) for i in sess.get_inputs() if i.name.startswith("cache_in_")}
    need = (N_WARM + N_TIMED) * hop
    xx = np.tile(x, need // len(x) + 1)[:need].astype(np.float32)
    t = np.empty(N_TIMED)
    nonfinite, e_in, e_out = 0, 0.0, 0.0
    for k in range(N_WARM + N_TIMED):
        feed["wav_in"] = xx[None, k * hop: (k + 1) * hop]
        t0 = time.perf_counter()
        out = sess.run(None, feed)
        t1 = time.perf_counter()
        for j in range(len(out) - 1):
            feed[f"cache_in_{j}"] = out[j + 1]
        if k >= N_WARM:
            t[k - N_WARM] = (t1 - t0) * 1e3
            nonfinite += int(not np.all(np.isfinite(out[0])))
            e_in += float(np.sum(feed["wav_in"] ** 2))
            e_out += float(np.sum(out[0] ** 2))
    period = hop / SR * 1e3
    st = {k: round(float(v), 3) for k, v in dict(p50=np.percentile(t, 50), p99=np.percentile(t, 99),
                                                  p999=np.percentile(t, 99.9), max=t.max(), mean=t.mean()).items()}
    return {"size": size, "hop": hop, "period_ms": period, "step_ms": st,
            "max_over_period": round(st["max"] / period, 3), "p999_over_period": round(st["p999"] / period, 3),
            "max_le_half_period": st["max"] <= period / 2, "nonfinite_steps": nonfinite,
            "out_in_energy_db": round(10 * np.log10(e_out / e_in), 2)}


def main():
    x, sr = sf.read(WAV, dtype="float32")
    assert sr == SR
    out = {"onnxruntime": ort.__version__, "platform": platform.platform(), "input": "resources/gtcrn/test_wavs/mix.wav",
           "threads": 1, "warm": N_WARM, "timed": N_TIMED, "runs": []}
    for size in ["b", "s", "m"]:
        r = run(size, x)
        out["runs"].append(r)
        print(json.dumps(r, ensure_ascii=False), flush=True)
    ok = [r for r in out["runs"] if r["max_le_half_period"] and r["nonfinite_steps"] == 0]
    order = {"b": 0, "s": 1, "m": 2}  # 档位从小到大
    out["chosen"] = max(ok, key=lambda r: order[r["size"]])["size"] if ok else None
    (HERE / "fastenhancer_bench.json").write_text(json.dumps(out, ensure_ascii=False, indent=1))
    print("chosen", out["chosen"])


if __name__ == "__main__":
    main()
