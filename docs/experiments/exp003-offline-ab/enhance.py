"""exp003 第二步：把每段带噪录音逐帧流式地过 DPDFNet-2 与 GTCRN，做延迟对齐与响度统一。

做法与 exp001/bench.py 一致（state 按 ONNX 元数据初始化、GTCRN cache 全零、自写 STFT/OLA），
区别是这里要真的把输出拼回成音频：
- 逐帧：每次只喂一帧（DPDFNet 320/160 vorbis 窗；GTCRN 512/256 周期 sqrt-Hann），状态回灌，和实时一致
- 开头补 n_fft 个零，让第一个真实样本也有完整的重叠相加；结尾补零把模型内部延迟冲出来
- 对齐：DPDFNet 输出比输入晚 4 帧（4×160=640 样本），GTCRN 0 帧；按此平移，再用互相关核对（结果写进 JSON）
- 响度：三个版本都用 BS.1770（pyloudnorm）归一到同一目标 LUFS；若某版本峰值超 -1 dBFS，
  同一段的三个版本一起再降同样的分贝，保持三者响度相同
输入：_tmp/20261030-exp003/src/<id>_noisy.wav（make_clips.py 产出）
输出：_tmp/20261030-exp003/raw/<id>_{noisy,dpdfnet2,gtcrn}.wav（对齐后、未归一，给打分用）
      _tmp/20261030-exp003/audio/<id>_<随机名>.wav（归一后，给对比页用）
      docs/experiments/exp003-offline-ab/align.json
用法（项目根下）：_tmp/exp002-venv/bin/python docs/experiments/exp003-offline-ab/enhance.py
"""
import json
import secrets
from pathlib import Path

import numpy as np
import onnxruntime as ort
import pyloudnorm
import soundfile as sf

from _paths import RES, ROOT  # resources/ 只在主目录
HERE = Path(__file__).resolve().parent
OUT = ROOT / "_tmp/20261030-exp003"
DPDF = RES / "weights/dpdfnet/dpdfnet2.onnx"
GTCRN = RES / "gtcrn/stream/onnx_models/gtcrn_simple.onnx"
SR = 16000
TARGET_LUFS = -26.0
PEAK_MAX_DB = -1.0
DPDF_DELAY_FRAMES = 4  # mask 缓冲 2 帧 + DF 2 帧，见 docs/research/002_model-baseline/decision.md「反对它的最强依据」


def session(path):
    so = ort.SessionOptions()
    so.intra_op_num_threads = 1
    so.inter_op_num_threads = 1
    so.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
    return ort.InferenceSession(str(path), sess_options=so, providers=["CPUExecutionProvider"])


def vorbis(n):
    i = np.arange(n)
    return np.sin(0.5 * np.pi * np.sin(np.pi * (i + 0.5) / n) ** 2).astype(np.float32)


def sqrt_hann_periodic(n):
    return np.sqrt(0.5 - 0.5 * np.cos(2 * np.pi * np.arange(n) / n)).astype(np.float32)


class Dpdfnet:
    name = "dpdfnet2"

    def __init__(self):
        self.sess = session(DPDF)
        m = self.sess.get_modelmeta().custom_metadata_map
        assert int(m["sample_rate"]) == SR
        self.n_fft, self.hop = int(m["n_fft"]), int(m["hop_length"])
        self.win = vorbis(self.n_fft)
        self.delay_frames = DPDF_DELAY_FRAMES
        st = np.zeros(int(m["state_size"]), dtype=np.float32)
        e = np.array(m["erb_norm_init"].split(","), dtype=np.float32)
        s = np.array(m["spec_norm_init"].split(","), dtype=np.float32)
        st[: len(e)] = e
        st[len(e): len(e) + len(s)] = s
        self.state = st

    def step(self, spec):
        feed = {"spec": np.stack([spec.real, spec.imag], -1)[None, None].astype(np.float32), "state_in": self.state}
        y, self.state = self.sess.run(["spec_e", "state_out"], feed)
        y = y.reshape(-1, 2)
        return y[:, 0] + 1j * y[:, 1]


class Gtcrn:
    name = "gtcrn"

    def __init__(self):
        self.sess = session(GTCRN)
        self.n_fft, self.hop = 512, 256
        self.win = sqrt_hann_periodic(self.n_fft)  # 分析与合成都用周期窗（002 decision 风险 3）
        self.delay_frames = 0
        self.caches = {i.name: np.zeros(i.shape, dtype=np.float32) for i in self.sess.get_inputs()[1:]}

    def step(self, spec):
        feed = {"mix": np.stack([spec.real, spec.imag], -1)[None, :, None].astype(np.float32), **self.caches}
        enh, c1, c2, c3 = self.sess.run(["enh", "conv_cache_out", "tra_cache_out", "inter_cache_out"], feed)
        self.caches = {"conv_cache": c1, "tra_cache": c2, "inter_cache": c3}
        y = enh.reshape(-1, 2)
        return y[:, 0] + 1j * y[:, 1]


def stream(model, x):
    """逐帧处理；返回与 x 等长、已按模型内部延迟平移好的输出。"""
    n_fft, hop, win = model.n_fft, model.hop, model.win
    pad_front = n_fft
    tail = (model.delay_frames + 2) * hop + n_fft
    xp = np.concatenate([np.zeros(pad_front, np.float32), x, np.zeros(tail, np.float32)])
    n_frames = (len(xp) - n_fft) // hop + 1
    y = np.zeros(len(xp) + n_fft, np.float32)
    for k in range(n_frames):
        seg = xp[k * hop: k * hop + n_fft]
        spec = np.fft.rfft(seg * win)
        out = model.step(spec)
        if not np.all(np.isfinite(out)):
            raise RuntimeError(f"{model.name}: 第 {k} 帧输出非有限值")
        y[k * hop: k * hop + n_fft] += np.fft.irfft(out, n=n_fft).astype(np.float32) * win
    shift = model.delay_frames * hop
    return y[pad_front + shift: pad_front + shift + len(x)]


def xcorr_lag(a, b, max_lag=2000):
    """返回 b 相对 a 的滞后样本数（正 = b 落后）。用 FFT 互相关，只在 ±max_lag 内找峰。"""
    n = len(a) + len(b)
    nf = 1 << (n - 1).bit_length()
    c = np.fft.irfft(np.fft.rfft(b, nf) * np.conj(np.fft.rfft(a, nf)), nf)
    lags = np.concatenate([np.arange(0, max_lag + 1), np.arange(-max_lag, 0)])
    vals = np.concatenate([c[: max_lag + 1], c[-max_lag:]])
    return int(lags[np.argmax(vals)])


def main():
    manifest = json.loads((HERE / "clips.json").read_text())
    (OUT / "raw").mkdir(parents=True, exist_ok=True)
    (OUT / "audio").mkdir(parents=True, exist_ok=True)
    meter = pyloudnorm.Meter(SR)
    align, files = {}, {}
    for clip in manifest["clips"]:
        cid = clip["id"]
        x, sr = sf.read(OUT / "src" / f"{cid}_noisy.wav", dtype="float32")
        assert sr == SR and x.ndim == 1
        ref = None
        if clip.get("has_ref"):
            ref, _ = sf.read(OUT / "src" / f"{cid}_clean.wav", dtype="float32")
        versions = {"noisy": x}
        for M in (Dpdfnet, Gtcrn):
            m = M()  # 每段重新建状态，互不串
            versions[m.name] = stream(m, x)
        # 对齐核对：与干净参考（有则用）或带噪输入做互相关，期望滞后 0
        probe = ref if ref is not None else x
        align[cid] = {k: xcorr_lag(probe, v) for k, v in versions.items() if k != "noisy"}
        align[cid]["probe"] = "clean" if ref is not None else "noisy"
        for k, v in versions.items():
            sf.write(OUT / "raw" / f"{cid}_{k}.wav", v, SR, subtype="FLOAT")
        # 响度统一
        gains = {k: TARGET_LUFS - meter.integrated_loudness(v) for k, v in versions.items()}
        peaks = {k: 20 * np.log10(np.max(np.abs(v)) + 1e-12) + gains[k] for k, v in versions.items()}
        extra = min(0.0, PEAK_MAX_DB - max(peaks.values()))
        files[cid] = {"lufs_target": TARGET_LUFS + extra, "versions": {}}
        for k, v in versions.items():
            y = v * 10 ** ((gains[k] + extra) / 20)
            fname = f"{cid}_{secrets.token_hex(3)}.wav"  # 文件名不暴露模型
            sf.write(OUT / "audio" / fname, y, SR, subtype="PCM_16")
            files[cid]["versions"][k] = {
                "file": f"audio/{fname}",
                "lufs_after": round(float(meter.integrated_loudness(y)), 2),
                "peak_dbfs": round(float(20 * np.log10(np.max(np.abs(y)) + 1e-12)), 2),
                "gain_db": round(float(gains[k] + extra), 2),
            }
        print(cid, align[cid], {k: v["lufs_after"] for k, v in files[cid]["versions"].items()}, flush=True)
    (HERE / "align.json").write_text(json.dumps({"lag_samples": align, "files": files}, ensure_ascii=False, indent=1))
    (OUT / "files.json").write_text(json.dumps(files, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
