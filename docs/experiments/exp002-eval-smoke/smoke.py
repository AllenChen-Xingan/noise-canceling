"""exp002 冒烟：在本机给几段公开样例打分并计时。

用法（项目根目录下）：
    _tmp/exp002-venv/bin/python docs/experiments/exp002-eval-smoke/smoke.py > docs/experiments/exp002-eval-smoke/results.json

DNSMOS 直接调用 resources/DNS-Challenge 里的 ComputeScore（只调用，不复制代码）。
"""
import json
import platform
import sys
import time
from pathlib import Path

import librosa
import numpy as np
import onnxruntime
import pyloudnorm
import soundfile as sf
from pesq import pesq
from pystoi import stoi

ROOT = Path(__file__).resolve().parents[3]
DNSMOS_DIR = ROOT / "resources/DNS-Challenge/DNSMOS"
sys.path.insert(0, str(DNSMOS_DIR))
from dnsmos_local import ComputeScore  # noqa: E402

CLIPS = {
    "gtcrn_mix": "resources/gtcrn/test_wavs/mix.wav",
    "gtcrn_enh": "resources/gtcrn/test_wavs/enh.wav",
    "dfn_clean": "resources/DeepFilterNet/assets/clean_freesound_33711.wav",
    "dfn_noisy_snr0": "resources/DeepFilterNet/assets/noisy_snr0.wav",
}
# 有参考指标：只有 DeepFilterNet 这一对有干净参考
REF_PAIRS = [("dfn_noisy_snr0", "dfn_clean"), ("dfn_clean", "dfn_clean")]


def load16k(path):
    audio, sr = sf.read(ROOT / path, dtype="float32", always_2d=False)
    if audio.ndim > 1:
        audio = audio.mean(axis=1)
    if sr != 16000:
        audio = librosa.resample(audio, orig_sr=sr, target_sr=16000)
    return audio


def si_sdr(est, ref):
    n = min(len(est), len(ref))
    est, ref = est[:n] - est[:n].mean(), ref[:n] - ref[:n].mean()
    target = (np.dot(est, ref) / np.dot(ref, ref)) * ref
    noise = est - target
    return float(10 * np.log10(np.sum(target**2) / np.sum(noise**2)))


def main():
    out = {
        "machine": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "processor": platform.processor(),
            "python": sys.version.split()[0],
            "onnxruntime": onnxruntime.__version__,
            "providers_available": onnxruntime.get_available_providers(),
            "librosa": librosa.__version__,
            "numpy": np.__version__,
        },
        "clips": {},
        "ref_pairs": [],
    }

    t0 = time.perf_counter()
    scorer = ComputeScore(
        str(DNSMOS_DIR / "DNSMOS/sig_bak_ovr.onnx"), str(DNSMOS_DIR / "DNSMOS/model_v8.onnx")
    )
    out["dnsmos_load_s"] = round(time.perf_counter() - t0, 3)

    meter = pyloudnorm.Meter(16000)
    for name, path in CLIPS.items():
        audio = load16k(path)
        dur = len(audio) / 16000
        t = time.perf_counter()
        s = scorer(str(ROOT / path), 16000, False)
        el = time.perf_counter() - t
        out["clips"][name] = {
            "path": path,
            "duration_s": round(dur, 3),
            "SIG": round(float(s["SIG"]), 3),
            "BAK": round(float(s["BAK"]), 3),
            "OVRL": round(float(s["OVRL"]), 3),
            "P808_MOS": round(float(s["P808_MOS"]), 3),
            "dnsmos_time_s": round(el, 3),
            "dnsmos_rtf": round(el / dur, 4),
            "LUFS": round(float(meter.integrated_loudness(audio)), 2),
        }

    for est_name, ref_name in REF_PAIRS:
        est, ref = load16k(CLIPS[est_name]), load16k(CLIPS[ref_name])
        n = min(len(est), len(ref))
        t = time.perf_counter()
        p = pesq(16000, ref[:n], est[:n], "wb")
        tp = time.perf_counter() - t
        t = time.perf_counter()
        st = stoi(ref[:n], est[:n], 16000, extended=False)
        es = stoi(ref[:n], est[:n], 16000, extended=True)
        ts = time.perf_counter() - t
        out["ref_pairs"].append({
            "est": est_name, "ref": ref_name,
            "PESQ_wb": round(float(p), 3), "STOI": round(float(st), 4), "ESTOI": round(float(es), 4),
            "SI_SDR_dB": round(si_sdr(est, ref), 2) if est_name != ref_name else "inf(自比)",
            "pesq_time_s": round(tp, 3), "stoi_estoi_time_s": round(ts, 3),
        })
    out["total_s"] = round(time.perf_counter() - t0, 3)
    print(json.dumps(out, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
