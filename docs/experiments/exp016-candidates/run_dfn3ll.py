"""exp016 · DeepFilterNet3 低延迟版（官方 `DeepFilterNet3_ll_onnx`）离线跑 exp013 场景测试集，写外部输出目录。

运行方式：Hush 预编译的 `libweya_nc.dylib`（内嵌 libDF，读模型包里的 config.ini，`model = deepfilternet3` 放行；
015 notes/G-system-ladder）经 ctypes 逐帧调用。人定 2026-10-31：Hush 预编译 dylib 只作对照使用，不进产品。
- 模型 48 kHz、帧 480（10 ms）、`df_lookahead = conv_lookahead = 0`（包内 config.ini:51,54）
- 整条链路流式、因果：16 kHz 每 160 样本一块 → soxr 流式升到 48 kHz → 攒满 480 交 dylib → soxr 流式降回 16 kHz。
  不用 dylib 自带的线性重采样（以 48 kHz 建会话，它不再转）
- `atten_lim_db = 100`（不设衰减下限，纯模型输出；与 NoNoise-Mac meeting 档同）
- 延迟：soxr 流式重采样按样本序号对齐（自己扣掉了滤波器延迟），代价是**偷看滤波器那一截未来**；
  所以对齐只平移 libDF 的一帧 480 @48 k（= 160 @16 k），声明延迟 = 160 + 重采样一来一回的前视
  （用「恒等模型」截半实测，容差 1e-7，覆盖整个滤波器）。实时管线里这截前视就是要等的延迟。
  `_ext.probe` 再在真模型上量前视与对齐，滞后 ≠ 0 即报错
每段 `weya_nc_reset` 并新建两个重采样器（与内置模型每段新建 Engine 同口径）。

用法（项目根下，exp002-venv 有 soxr / soundfile）：
  <主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp016-candidates/run_dfn3ll.py [--shard k/N] [--run-id ID]
  … 全部分片跑完后同一 run-id 加 --finalize
"""
import ctypes
import sys
from pathlib import Path

import numpy as np
import soxr

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import _ext  # noqa: E402

NAME = "dfn3-ll"
MAIN = _ext.main_root()
LIB = MAIN / "resources/Hush/deployment/lib/libweya_nc.dylib"
MODEL = MAIN / "resources/DeepFilterNet/models/DeepFilterNet3_ll_onnx.tar.gz"
SHA = {"lib": "4bcd38634000d456ad68db4b6ff97fe4a462542aa65ac5f910fbf374ca33fbaa",
       "model": "5998e58e8ba0e09bb76986ef97b84afa065a571ef282d4a1222f341e3251cf3a"}
SR_IN, SR_MODEL, BLOCK = 16000, 48000, 160
FRAME = 480
MODEL_DELAY_48K = 480  # libDF：输出比输入晚 fft_size − hop_size = 960 − 480
ATTEN_LIM_DB = 100.0
QUALITY = "HQ"
TAIL = 4000  # 结尾补零把链路延迟冲出来（16 kHz 样本，> 实测延迟）

_lib = _model = _sess = None


def lib():
    global _lib, _model, _sess
    if _lib is None:
        L = ctypes.CDLL(str(LIB))
        L.weya_nc_model_load_from_path.argtypes = [ctypes.c_char_p]
        L.weya_nc_model_load_from_path.restype = ctypes.c_void_p
        L.weya_nc_session_create.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_float]
        L.weya_nc_session_create.restype = ctypes.c_void_p
        for f in ("weya_nc_get_frame_length", "weya_nc_get_sample_rate", "weya_nc_get_input_sample_rate"):
            getattr(L, f).argtypes = [ctypes.c_void_p]
            getattr(L, f).restype = ctypes.c_size_t
        fp = ctypes.POINTER(ctypes.c_float)
        L.weya_nc_process_frame.argtypes = [ctypes.c_void_p, fp, fp]
        L.weya_nc_process_frame.restype = ctypes.c_float
        L.weya_nc_reset.argtypes = [ctypes.c_void_p]
        _model = L.weya_nc_model_load_from_path(str(MODEL).encode())
        if not _model:
            raise SystemExit(f"dylib 载入模型失败：{MODEL.name}")
        _sess = L.weya_nc_session_create(_model, SR_MODEL, ATTEN_LIM_DB)
        if not _sess:
            raise SystemExit("dylib 建会话失败")
        got = (L.weya_nc_get_frame_length(_sess), L.weya_nc_get_sample_rate(_sess), L.weya_nc_get_input_sample_rate(_sess))
        if got != (FRAME, SR_MODEL, SR_MODEL):
            raise SystemExit(f"帧长 / 采样率与预期不符：{got}")
        _lib = L
    return _lib


def dfn_frame(fr):
    L = lib()
    inp = np.ascontiguousarray(fr, dtype=np.float32)
    out = np.zeros(FRAME, np.float32)
    fp = ctypes.POINTER(ctypes.c_float)
    L.weya_nc_process_frame(_sess, inp.ctypes.data_as(fp), out.ctypes.data_as(fp))
    return out


def chain(x, frame_fn):
    """流式链路：返回原始（未对齐）16 kHz 输出，长度 ≥ len(x) + TAIL − 一点余量。"""
    up = soxr.ResampleStream(SR_IN, SR_MODEL, 1, dtype="float32", quality=QUALITY)
    dn = soxr.ResampleStream(SR_MODEL, SR_IN, 1, dtype="float32", quality=QUALITY)
    xp = np.concatenate([np.asarray(x, np.float32), np.zeros(TAIL, np.float32)])
    buf, outs = np.zeros(0, np.float32), []
    for i in range(0, len(xp), BLOCK):
        buf = np.concatenate([buf, up.resample_chunk(xp[i:i + BLOCK])])
        while len(buf) >= FRAME:
            outs.append(dn.resample_chunk(frame_fn(buf[:FRAME])))
            buf = buf[FRAME:]
    return np.concatenate(outs) if outs else np.zeros(0, np.float32)


SHIFT = MODEL_DELAY_48K * SR_IN // SR_MODEL  # 对齐平移（16 kHz 样本）


def identity_chain(x):
    return chain(x, lambda fr: fr)[: len(x)]


def resampler_lookahead():
    """重采样一来一回的前视（16 kHz 样本）：恒等模型过宽带噪声，截半实测；同时核对它按样本对齐（滞后 0）。"""
    x = np.random.default_rng(0).standard_normal(SR_IN * 2).astype(np.float32) * 0.1
    lag = _ext.xcorr_lag(x, identity_chain(x))
    if lag != 0:
        raise SystemExit(f"soxr 流式重采样不再按样本对齐（滞后 {lag}），本脚本的平移要重算")
    return _ext.measure_lookahead(identity_chain, x, tol=1e-7)


_D = None


def total_delay():
    """声明延迟：libDF 一帧 + 重采样前视。"""
    global _D
    if _D is None:
        _D = SHIFT + resampler_lookahead()
    return _D


def process(x):
    lib().weya_nc_reset(_sess)
    raw = chain(x, dfn_frame)
    y = raw[SHIFT:SHIFT + len(x)]
    if len(y) != len(x):
        raise RuntimeError(f"链路输出不够长：{len(raw)} < {SHIFT} + {len(x)}")
    return y.astype(np.float32)


def setup():
    return {"resampler": f"soxr {soxr.__version__} ResampleStream {QUALITY}", "resampler_lookahead_16k": resampler_lookahead(),
            "model_delay_48k": MODEL_DELAY_48K, "align_shift_16k": SHIFT, "declared_latency_16k": total_delay(), "atten_lim_db": ATTEN_LIM_DB,
            "lib_sha256": SHA["lib"], "model_sha256": SHA["model"]}


INFO = {"repo": "DeepFilterNet（resources/DeepFilterNet，上游 d375b2d8）· 运行库 Hush libweya_nc（上游 9f6414e9）",
        "weights": "resources/DeepFilterNet/models/DeepFilterNet3_ll_onnx.tar.gz", "weights_sha256": SHA["model"],
        "lib_sha256": SHA["lib"], "mode": "逐帧流式 · 48 kHz · 零前瞻（_ll）· atten_lim 100 dB · 每段重置",
        "resample": f"soxr 流式 {QUALITY} 16→48→16 kHz", "runner": "docs/experiments/exp016-candidates/run_dfn3ll.py"}

if __name__ == "__main__":
    for k, p in (("lib", LIB), ("model", MODEL)):
        if _ext.sha256_file(p) != SHA[k]:
            raise SystemExit(f"{p.name} 的 sha256 与记录不符")
    # 标称延迟 = libDF 一帧 + 实测的重采样前视；_ext.probe 再在真模型上量前视
    _ext.main(NAME, process, total_delay(), INFO, setup)
