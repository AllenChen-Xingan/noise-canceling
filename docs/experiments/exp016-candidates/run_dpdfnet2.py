"""exp016 · DPDFNet-2 离线跑 exp013 场景测试集，写外部输出目录（格式见 _ext.py 头注释）。

模型与逐帧流式沿用 exp003 `enhance.py` 的 `Dpdfnet` 类与 `stream()`（导入，不复制）：
16 kHz 原生、320/160 vorbis 窗、state 向量回灌、每次只喂一帧；输出比输入晚 4 帧（640 样本），`stream()` 已按此平移对齐。
标称延迟（16 kHz 样本）= 4 帧 × 160 + 窗长 320 = 960（60 ms；015 notes/A 表「两段串联 4 帧 · ① 60 ms」）。
每段新建状态（与内置模型每段 `Engine::new` 同口径），会话只建一次。

用法（项目根下，exp002-venv 有 onnxruntime / soundfile / pyloudnorm）：
  <主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp016-candidates/run_dpdfnet2.py [--shard k/N] [--run-id ID]
  … 全部分片跑完后同一 run-id 加 --finalize
"""
import sys
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent / "exp003-offline-ab"))
import _ext  # noqa: E402
import _paths  # noqa: E402  exp003：主目录
import enhance  # noqa: E402  exp003：Dpdfnet · stream · DPDF 路径

NAME = "dpdfnet2"
HOP, N_FFT, DELAY_FRAMES = 160, 320, enhance.DPDF_DELAY_FRAMES
NOMINAL_LATENCY = DELAY_FRAMES * HOP + N_FFT

_model = None


def model():
    global _model
    if _model is None:
        _model = enhance.Dpdfnet()
        assert (_model.n_fft, _model.hop) == (N_FFT, HOP), (_model.n_fft, _model.hop)
        _model.state0 = _model.state.copy()
    return _model


def process(x):
    m = model()
    m.state = m.state0.copy()  # 每段从初始状态开始
    return enhance.stream(m, np.asarray(x, dtype=np.float32)).astype(np.float32)


def setup():
    return {"weights": str(enhance.DPDF.relative_to(_paths.MAIN)), "weights_sha256": _ext.sha256_file(enhance.DPDF),
            "onnxruntime_threads": 1, "delay_frames": DELAY_FRAMES}


INFO = {"repo": "DPDFNet（resources/DPDFNet）", "weights": "resources/weights/dpdfnet/dpdfnet2.onnx",
        "weights_sha256": "4f0ee28935b4a32abecc717d745416976565834d839601acf43031094b4dc94c",
        "mode": "逐帧流式 · ORT CPU 单线程 · 16 kHz 原生 · 每段重置状态", "resample": "无",
        "nominal_latency_ms": NOMINAL_LATENCY / 16.0, "runner": "docs/experiments/exp016-candidates/run_dpdfnet2.py"}

if __name__ == "__main__":
    if _ext.sha256_file(enhance.DPDF) != INFO["weights_sha256"]:
        raise SystemExit("dpdfnet2.onnx 的 sha256 与记录不符")
    _ext.main(NAME, process, NOMINAL_LATENCY, INFO, setup)
