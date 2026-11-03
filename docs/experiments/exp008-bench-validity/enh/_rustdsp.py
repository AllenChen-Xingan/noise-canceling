"""exp008 enh：RNNoise（nnnoiseless）/ WebRTC NS（sonora-ns）共用驱动。

Rust 小 bin 源码在 enh/rust/{rnnoise_raw,webrtc_ns_raw}/，只读写 raw f32le（stdin→stdout），
不做延迟补偿；本模块负责 wav 读写、16↔48 k 重采样、延迟补偿、延迟核对和记录。
"""
import argparse
import hashlib
import json
import platform
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import numpy as np
import scipy
import soundfile as sf
from scipy.signal import resample_poly

MAIN = Path("/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling")
HERE = Path(__file__).resolve().parent
EXP = HERE.parent
CARGO_TARGET = MAIN / "_tmp/exp008-cargo"
SR = 16000

SYSTEMS = {
    "rnnoise": dict(
        family="E", sr_model=48000, crate="rnnoise_raw", bin="exp008-rnnoise-raw", args=[],
        # 48 k 下 RNNoise 输出比输入晚 1 帧（480 样本，窗 960 / 帧移 480 的重叠相加）= 16 k 下 160
        delay=160,
        code_source="resources/nnnoiseless（jneem/nnnoiseless 0.5.2，git 49fa1945e997f0fee4814a5038703fb40aaf756c）",
        license="BSD-3-Clause", license_source="resources/nnnoiseless/COPYING",
        weights_path="resources/nnnoiseless/src/weights.rnn",
        notes="DenoiseState::process_frame，输入按 ±32768 量级缩放；权重编进源码（RNNoise 2017 原始模型）；外面 resample_poly 16→48→16 k",
    ),
    "webrtc_ns18": dict(
        family="C*", sr_model=16000, crate="webrtc_ns_raw", bin="exp008-webrtc-ns-raw", args=["18"],
        delay=None,  # 由 delay 探针定，见 DELAY_NS
        code_source="resources/sonora/crates/sonora-ns（dignifiedquire/sonora 0.2.0，git 373f1d73fd4429ef38dd8854b3162f40e0eb7eb0）",
        license="BSD-3-Clause", license_source="resources/sonora/LICENSE",
        weights_path=None,
        notes="NoiseSuppressor::with_level(K18dB)，单频带 16 k，每帧 analyze→process（同 sonora capture 路径），输入按 ±32768 量级",
    ),
    "webrtc_ns6": dict(
        family="C*", sr_model=16000, crate="webrtc_ns_raw", bin="exp008-webrtc-ns-raw", args=["6"],
        delay=None,
        code_source="resources/sonora/crates/sonora-ns（dignifiedquire/sonora 0.2.0，git 373f1d73fd4429ef38dd8854b3162f40e0eb7eb0）",
        license="BSD-3-Clause", license_source="resources/sonora/LICENSE",
        weights_path=None,
        notes="NoiseSuppressor::with_level(K6dB)，参数变体；其余同 webrtc_ns18",
    ),
}
# WebRTC NS：256 点 FFT、160 帧移，重叠 96 → 输出晚 96 样本（16 k）
DELAY_NS = 96
for k in ("webrtc_ns18", "webrtc_ns6"):
    SYSTEMS[k]["delay"] = DELAY_NS


def sha256(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def build(cfg):
    d = HERE / "rust" / cfg["crate"]
    env = dict(__import__("os").environ, CARGO_TARGET_DIR=str(CARGO_TARGET))
    subprocess.run(["cargo", "build", "--release", "--offline", "--quiet"], cwd=d, env=env, check=True)
    exe = CARGO_TARGET / "release" / cfg["bin"]
    assert exe.exists(), exe
    assert not (d / "target").exists(), "仓内出现 target/，停"
    return exe


def run_raw(exe, args, x):
    r = subprocess.run([str(exe), *args], input=x.astype("<f4").tobytes(), capture_output=True, check=True)
    return np.frombuffer(r.stdout, dtype="<f4").astype(np.float64)


def enhance(exe, cfg, x, delay):
    """x: 16 k float. 返回与 x 等长、已补偿 delay（16 k 样本）的输出。"""
    n = len(x)
    if cfg["sr_model"] == 48000:
        x48 = resample_poly(x, 3, 1)
        y48 = run_raw(exe, cfg["args"], x48)
        y = resample_poly(y48, 1, 3)
    else:
        y = run_raw(exe, cfg["args"], x)
    y = y[delay:delay + n]
    if len(y) < n:
        y = np.pad(y, (0, n - len(y)))
    return y


def xcorr_lag(y, ref, maxlag=2000):
    a = y - y.mean()
    b = ref - ref.mean()
    n = len(a) + len(b)
    nf = 1 << (n - 1).bit_length()
    c = np.fft.irfft(np.fft.rfft(a, nf) * np.conj(np.fft.rfft(b, nf)), nf)
    lags = np.r_[np.arange(0, maxlag + 1), np.arange(-maxlag, 0)]
    vals = np.r_[c[: maxlag + 1], c[-maxlag:]]
    return int(lags[np.argmax(vals)])


def main(sid):
    cfg = SYSTEMS[sid]
    ap = argparse.ArgumentParser()
    ap.add_argument("build")
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--audio-root", default=str(MAIN / "_tmp/exp008"))
    a = ap.parse_args()
    bdir = EXP / "builds" / a.build
    rows = [json.loads(l) for l in open(bdir / "truth.jsonl")]
    if a.limit:
        rows = rows[: a.limit]
    src = Path(a.audio_root) / a.build
    dst = Path(a.audio_root) / f"{a.build}-enh" / sid
    exe = build(cfg)
    delay = cfg["delay"]
    t0 = time.time()

    def one(r):
        rel = r["files"]["mix"]["path"]
        out = dst / rel
        if out.exists():
            return "skip"
        x, sr = sf.read(src / rel, dtype="float64")
        assert sr == SR and x.ndim == 1
        y = enhance(exe, cfg, x, delay)
        assert len(y) == len(x)
        out.parent.mkdir(parents=True, exist_ok=True)
        tmp = out.with_suffix(".part.wav")
        sf.write(tmp, y.astype(np.float32), SR, subtype="FLOAT")
        tmp.rename(out)
        return "ok"

    fails = []
    with ThreadPoolExecutor(max(1, a.jobs)) as ex:
        futs = {ex.submit(one, r): r["files"]["mix"]["path"] for r in rows}
        res = []
        for f, rel in futs.items():
            try:
                res.append(f.result())
            except Exception as e:  # noqa: BLE001
                fails.append({"path": rel, "error": repr(e)})
    secs = time.time() - t0

    # 延迟核对：前 6 段，输出 vs owner 互相关 ±2000
    lags = []
    for r in rows[:6]:
        rel = r["files"]["mix"]["path"]
        y = sf.read(dst / rel, dtype="float64")[0]
        o = sf.read(src / r["files"]["owner"]["path"], dtype="float64")[0]
        lags.append({"path": rel, "lag": xcorr_lag(y, o)})
    ok = all(abs(l["lag"]) <= 1 for l in lags)

    n_written = sum(1 for r in json.loads(json.dumps([x["files"]["mix"]["path"] for x in rows])) if (dst / r).exists())
    w = []
    if cfg["weights_path"]:
        p = MAIN / cfg["weights_path"]
        w.append({"path": cfg["weights_path"], "sha256": sha256(p), "bytes": p.stat().st_size,
                  "license": cfg["license"], "license_source": cfg["license_source"]})
    rec = {
        "id": sid, "family": cfg["family"], "causal": True, "sr_model": cfg["sr_model"],
        "weights": w, "code_source": cfg["code_source"], "license": cfg["license"],
        "license_source": cfg["license_source"],
        "delay_comp_samples": delay,
        "delay_check": {"method": "前 6 段输出 vs _owner.wav 互相关，±2000 样本", "lags": lags, "pass": ok},
        "runtime": f"python {platform.python_version()} · numpy {np.__version__} · scipy {scipy.__version__} · "
                   f"{cfg['bin']} sha256 {sha256(exe)}（{subprocess.run(['rustc', '--version'], capture_output=True, text=True).stdout.strip()}）",
        "bin_source": f"docs/experiments/exp008-bench-validity/enh/rust/{cfg['crate']}/",
        "n_written": n_written, "n_rows": len(rows), "failures": fails,
        "seconds": round(secs, 1), "jobs": a.jobs, "limit": a.limit, "notes": cfg["notes"],
    }
    if not a.limit:
        (bdir / "enh").mkdir(exist_ok=True)
        (bdir / "enh" / f"{sid}.json").write_text(json.dumps(rec, ensure_ascii=False, indent=1) + "\n")
    print(json.dumps({k: rec[k] for k in ("id", "n_written", "n_rows", "seconds", "delay_comp_samples")}, ensure_ascii=False))
    print("delay_check", "PASS" if ok else "FAIL", [l["lag"] for l in lags])
    if fails:
        print("failures", fails[:5], file=sys.stderr)
    return 0 if (ok and not fails) else 1
