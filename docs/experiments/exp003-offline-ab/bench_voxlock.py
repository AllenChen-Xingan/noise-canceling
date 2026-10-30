"""exp003 附：VoxLock 单线程逐块计时（口径对齐 exp001：真实带噪语音、单线程、先热身再计时、统计 p50/p99/p99.9/max）。

- 输入：resources/gtcrn/test_wavs/mix.wav（与 exp001 同一份），注册样本取它前 15 s 最响的 3 s（VoxLock 自己基准的取法）
- 计的是 Stream.push() 整次调用（chunk.onnx 推理 + 图外 OLA），每次推一块；块长扫 8 / 10 / 20 / 40 ms
- 「每 10 ms 音频耗时」= 每块耗时 × 10 / 块长，用来和 002 的 2 ms 线比
- 缓冲延迟：每次 push 后「已进未出」的样本数（n_in − n_out）的最大值，换算成 ms；这是算法上必须攒的量，
  不含块长本身与设备缓冲
- 精度：int8（官方默认）与 fp32
输出：docs/experiments/exp003-offline-ab/voxlock_bench.json
用法（项目根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp003-offline-ab/bench_voxlock.py
"""
import json
import platform
import sys
import time
from pathlib import Path

import numpy as np
import onnxruntime as ort
import soundfile as sf

from _paths import RES, ROOT

sys.path.insert(0, str(RES / "voxlock"))
from voxlock.core import SR, VoxLock, loud_sample  # noqa: E402  只调用，不复制

HERE = ROOT / "docs/experiments/exp003-offline-ab"
WAV = RES / "gtcrn/test_wavs/mix.wav"
MODEL_DIR = Path.home() / "vdn-data/models/voxlock"
N_WARM, N_TIMED = 200, 6000
BLOCKS_MS = [8, 10, 20, 40]


def stats(t):
    return {k: round(float(v), 3) for k, v in
            dict(p50=np.percentile(t, 50), p99=np.percentile(t, 99), p999=np.percentile(t, 99.9), max=t.max()).items()}


def run(vl, x, block_ms):
    n = SR * block_ms // 1000
    need = (N_WARM + N_TIMED) * n
    xx = np.tile(x, need // len(x) + 1)[:need]
    s = vl.stream(loud_sample(x))
    t = np.empty(N_TIMED)
    held, nonfinite = 0, 0
    for k in range(N_WARM + N_TIMED):
        c = xx[k * n: (k + 1) * n]
        t0 = time.perf_counter()
        y = s.push(c, frames_per_run=100)
        t1 = time.perf_counter()
        if k >= N_WARM:
            t[k - N_WARM] = (t1 - t0) * 1e3
            held = max(held, s.n_in - s.n_out)
            nonfinite += int(not np.all(np.isfinite(y)))
    st = stats(t)
    return {"block_ms": block_ms, "push_ms": st,
            "per_10ms_audio_ms": {k: round(v * 10 / block_ms, 3) for k, v in st.items()},
            "max_held_samples": int(held), "max_held_ms": round(held / SR * 1e3, 2), "nonfinite_blocks": nonfinite}


def main():
    x, sr = sf.read(WAV, dtype="float32")
    assert sr == SR
    out = {"onnxruntime": ort.__version__, "machine": platform.machine(), "platform": platform.platform(),
           "input": str(WAV.relative_to(RES.parent)), "threads": 1, "warm": N_WARM, "timed": N_TIMED, "runs": []}
    for prec in ["int8", "fp32"]:
        vl = VoxLock(model_dir=MODEL_DIR, precision=prec, threads=1)
        for b in BLOCKS_MS:
            r = {"precision": prec, **run(vl, x, b)}
            out["runs"].append(r)
            print(json.dumps(r, ensure_ascii=False), flush=True)
    (HERE / "voxlock_bench.json").write_text(json.dumps(out, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
