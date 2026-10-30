"""exp001 v2：按自审意见重测。

与 v1（bench.py）的区别：
- 输入是真实带噪语音（resources/gtcrn/test_wavs/mix.wav，16 kHz），逐帧做 STFT 后喂模型
- DPDFNet 的 state 按 ONNX 元数据初始化（归一化两段填初值，其余为 0）；GTCRN 的 cache 全零
- 每帧检查输出是否全是有限值；统计输出与输入能量比，确认模型「在工作」而不是原样输出或静音
- 分别计时：模型 run，以及 numpy 的 STFT + iSTFT（在 Rust 里会更快，这里只给量级）
用法：uv run --no-project --with onnxruntime --with numpy python bench2.py
"""
import time
import wave
from pathlib import Path

import numpy as np
import onnxruntime as ort

ROOT = Path(__file__).resolve().parents[3]
WAV = ROOT / "resources/gtcrn/test_wavs/mix.wav"
W = ROOT / "resources/weights/dpdfnet"
G = ROOT / "resources/gtcrn/stream/onnx_models"
N_WARM, N_FRAMES = 200, 6000


def load_wav(path):
    with wave.open(str(path)) as w:
        sr = w.getframerate()
        x = np.frombuffer(w.readframes(w.getnframes()), dtype=np.int16).astype(np.float32) / 32768.0
    return x, sr


def resample_linear(x, sr_in, sr_out):
    if sr_in == sr_out:
        return x
    t_out = np.arange(int(len(x) * sr_out / sr_in)) * sr_in / sr_out
    return np.interp(t_out, np.arange(len(x)), x).astype(np.float32)


def vorbis(n):
    i = np.arange(n)
    return np.sin(0.5 * np.pi * np.sin(np.pi * (i + 0.5) / n) ** 2).astype(np.float32)


def session(path):
    so = ort.SessionOptions()
    so.intra_op_num_threads = 1
    so.inter_op_num_threads = 1
    so.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
    return ort.InferenceSession(str(path), sess_options=so, providers=["CPUExecutionProvider"])


def dpdfnet_init_state(sess):
    m = sess.get_modelmeta().custom_metadata_map
    st = np.zeros(int(m["state_size"]), dtype=np.float32)
    e = np.array(m["erb_norm_init"].split(","), dtype=np.float32)
    s = np.array(m["spec_norm_init"].split(","), dtype=np.float32)
    st[: len(e)] = e
    st[len(e): len(e) + len(s)] = s
    return st, m


def stats(t):
    return dict(p50=np.percentile(t, 50), p99=np.percentile(t, 99), p999=np.percentile(t, 99.9), max=t.max())


def run(name, path, kind):
    sess = session(path)
    if kind == "dpdfnet":
        state, meta = dpdfnet_init_state(sess)
        sr, n_fft, hop = int(meta["sample_rate"]), int(meta["n_fft"]), int(meta["hop_length"])
        win = vorbis(n_fft)
    else:  # gtcrn
        sr, n_fft, hop = 16000, 512, 256
        win = np.sqrt(np.hanning(n_fft + 1)[:-1]).astype(np.float32)  # 周期 sqrt-Hann
        caches = {i.name: np.zeros(i.shape, dtype=np.float32) for i in sess.get_inputs()[1:]}
    x, sr0 = load_wav(WAV)
    x = resample_linear(x, sr0, sr)
    need = (N_WARM + N_FRAMES) * hop + n_fft
    x = np.tile(x, need // len(x) + 1)[:need]
    names = [o.name for o in sess.get_outputs()]
    ins = [i.name for i in sess.get_inputs()]
    t_model, t_dsp = np.empty(N_FRAMES), np.empty(N_FRAMES)
    nonfinite, e_in, e_out = 0, 0.0, 0.0
    ola = np.zeros(n_fft, dtype=np.float32)
    for k in range(N_WARM + N_FRAMES):
        t0 = time.perf_counter()
        spec = np.fft.rfft(x[k * hop: k * hop + n_fft] * win)
        if kind == "dpdfnet":
            feed = {ins[0]: np.stack([spec.real, spec.imag], -1)[None, None].astype(np.float32), ins[1]: state}
        else:
            feed = {ins[0]: np.stack([spec.real, spec.imag], -1)[None, :, None].astype(np.float32), **caches}
        t1 = time.perf_counter()
        res = sess.run(names, feed)
        t2 = time.perf_counter()
        y = res[0].reshape(-1, 2)
        frame = np.fft.irfft(y[:, 0] + 1j * y[:, 1], n=n_fft).astype(np.float32) * win
        ola = np.roll(ola, -hop)
        ola[-hop:] = 0
        ola += frame
        t3 = time.perf_counter()
        if kind == "dpdfnet":
            state = res[1]
        else:
            for i_name, o_name in zip(ins[1:], names[1:]):
                caches[i_name] = res[names.index(o_name)]
        if k >= N_WARM:
            j = k - N_WARM
            t_model[j] = (t2 - t1) * 1e3
            t_dsp[j] = ((t1 - t0) + (t3 - t2)) * 1e3
            nonfinite += int(not np.all(np.isfinite(res[0])))
            e_in += float(np.sum(np.abs(spec) ** 2))
            e_out += float(np.sum(y ** 2))
    sm, sd = stats(t_model), stats(t_dsp)
    ratio_db = 10 * np.log10(e_out / e_in) if e_in > 0 and e_out > 0 else float("nan")
    print(f"{name}\tsr={sr} n_fft={n_fft} hop={hop}({hop / sr * 1e3:.0f} ms)\tnonfinite_frames={nonfinite}\tout/in energy={ratio_db:.1f} dB")
    print("  model ms  " + " ".join(f"{k}={v:.3f}" for k, v in sm.items()))
    print("  stft+istft ms (numpy) " + " ".join(f"{k}={v:.3f}" for k, v in sd.items()))


if __name__ == "__main__":
    print("onnxruntime", ort.__version__, "| input", WAV.name)
    for m in ["baseline", "dpdfnet2", "dpdfnet4", "dpdfnet8", "dpdfnet2_48khz_hr", "dpdfnet8_48khz_hr"]:
        run(m, W / f"{m}.onnx", "dpdfnet")
    run("gtcrn_simple", G / "gtcrn_simple.onnx", "gtcrn")
