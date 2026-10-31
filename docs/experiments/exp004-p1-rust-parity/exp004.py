"""exp004：Rust P1 离线原型（`vdn`）对拍 exp003 的 Python 流式结果，并计时。

分步（项目根下，用 exp003 的 venv；音频全写主目录 _tmp/20261030-exp004/，不进仓）：
  PY=<主目录>/_tmp/exp002-venv/bin/python
  $PY docs/experiments/exp004-p1-rust-parity/exp004.py src      # 用 exp003 make_clips.py 原样重生成 12 段素材
  $PY docs/experiments/exp004-p1-rust-parity/exp004.py py       # Python 参考：exp003 enhance.py 的 Gtcrn / FastEnhancer 原样跑
  $PY docs/experiments/exp004-p1-rust-parity/exp004.py rs       # Rust：target/release/vdn 跑 raw / safe / floor / fault 四种模式
  $PY docs/experiments/exp004-p1-rust-parity/exp004.py compare  # 逐值对比 + 三件保命效果 → results/compare.json
  $PY docs/experiments/exp004-p1-rust-parity/exp004.py bench    # Rust 与 Python 单线程每帧耗时 → results/bench.json

只调用 exp003 的脚本（import），不复制它们的代码。
"""
import hashlib
import json
import platform
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
EXP3 = HERE.parent / "exp003-offline-ab"
sys.path.insert(0, str(EXP3))
from _common import vad_mask  # noqa: E402
from _paths import MAIN, RES, ROOT  # noqa: E402

OUT = MAIN / "_tmp/20261030-exp004"
RESULTS = HERE / "results"
VDN = ROOT / "target/release/vdn"
SR = 16000
GTCRN_ONNX = RES / "gtcrn/stream/onnx_models/gtcrn_simple.onnx"
FE_ONNX = Path.home() / "vdn-data/models/fastenhancer-dns-v1.0.0/fastenhancer_m.onnx"
# 模型名 → (vdn --model, 模型文件, Rust 开头补零数, Python 文件后缀)
MODELS = {
    "gtcrn": ("gtcrn", GTCRN_ONNX, 256, "gtcrn"),  # Python stream() 第一帧是全零帧：Rust 先喂一个 hop 的零对齐
    "fastenhancer_m": ("fastenhancer-m", FE_ONNX, 0, "fastenhancer_m"),
}
# 保命关闭 / 默认全开 / 只留固定下限 / 默认全开 + 故障注入
MODES = {
    "raw": ["--no-safety"],
    "safe": [],
    "floor": ["--no-fast-open"],
    "fault": [],
}
FAULT_CLIPS = ["c05", "c06", "c10"]
FAULT_HOPS = {"nan": 100, "error": 200, "explode": 300, "stall": 400}


def sha256(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def clips():
    return json.loads((EXP3 / "clips.json").read_text())["clips"]


def step_src():
    import make_clips  # exp003 的脚本；只改输出位置，别覆盖 exp003 的 clips.json

    make_clips.OUT = OUT / "src"
    make_clips.HERE = OUT
    make_clips.main()
    regen = json.loads((OUT / "clips.json").read_text())
    orig = json.loads((EXP3 / "clips.json").read_text())
    same = regen == orig
    files = {p.name: sha256(p) for p in sorted((OUT / "src").glob("*.wav"))}
    RESULTS.mkdir(exist_ok=True)
    (RESULTS / "src_manifest.json").write_text(json.dumps(
        {"clips_json_identical_to_exp003": same, "src_sha256": files}, ensure_ascii=False, indent=1))
    print("clips.json 与 exp003 一致：", same)


def step_py():
    import enhance  # exp003 的脚本

    (OUT / "py").mkdir(parents=True, exist_ok=True)
    for c in clips():
        x, sr = sf.read(OUT / "src" / f"{c['id']}_noisy.wav", dtype="float32")
        assert sr == SR
        y = {"gtcrn": enhance.stream(enhance.Gtcrn(), x), "fastenhancer_m": enhance.FastEnhancer().run(x)}
        for k, v in y.items():
            sf.write(OUT / "py" / f"{c['id']}_{k}.wav", v, SR, subtype="FLOAT")
        print("py", c["id"], flush=True)


def fault_arg(model):
    stall_ms = 40
    return ",".join(f"{h}:{k if k != 'stall' else f'stall={stall_ms}'}" for k, h in FAULT_HOPS.items())


def step_rs():
    assert VDN.is_file(), f"先 cargo build --release：{VDN}"
    for mode, extra in MODES.items():
        (OUT / "rs" / mode).mkdir(parents=True, exist_ok=True)
    for c in clips():
        src = OUT / "src" / f"{c['id']}_noisy.wav"
        for name, (arg, path, lead_in, _) in MODELS.items():
            for mode, extra in MODES.items():
                if mode == "fault" and c["id"] not in FAULT_CLIPS:
                    continue
                args = list(extra)
                if mode == "fault":
                    hop = 256 if name == "gtcrn" else 160
                    args += ["--inject", fault_arg(name), "--deadline-ms", str(hop / SR * 1e3 / 2)]
                dst = OUT / "rs" / mode / f"{c['id']}_{name}.wav"
                cmd = [str(VDN), "denoise", str(src), str(dst), "--model", arg, "--model-path", str(path),
                       "--lead-in", str(lead_in), "--json", str(dst.with_suffix(".json")), *args]
                subprocess.run(cmd, check=True, capture_output=True)
        print("rs", c["id"], flush=True)


def si_sdr(est, ref):
    import score  # exp003 的 si_sdr

    return score.si_sdr(est, ref)


HOLD_S = 0.1  # 与 SafetyConfig::default().recover_hold_s 一致


def seams(name, y, safe, x, half=32):
    """每个故障有两个接缝：切到原声（故障那一帧开头）、切回模型（hold 结束后 5 ms 交叉淡化开头）。
    在接缝前后 half 个样本内量二阶差分的最大绝对值（咔声在二阶差分上最显眼），
    和同一窗口里「无故障版」与「原声」两者中较大的那个比：比值 ≤ 1 说明接缝没有比两路信号本身更尖的跳变。"""
    hop = 256 if name == "gtcrn" else 160
    lat = 256 if name == "gtcrn" else 352
    lead = MODELS[name][2]
    hold = int(np.ceil(HOLD_S * SR / hop))
    d2 = lambda a, w: float(np.max(np.abs(np.diff(a[w], 2))))
    out = {}
    for kind, h in FAULT_HOPS.items():
        back = h + 1 + (0 if kind == "stall" else hold)  # 卡顿不复位，下一帧就切回
        for side, hh in (("enter", h), ("exit", back)):
            pos = hh * hop - lead - lat
            w = slice(max(pos - half, 0), pos + half)
            ref = max(d2(safe, w), d2(x, w))
            out[f"{kind}_{side}"] = {"pos": pos, "fault_d2": d2(y, w), "ref_d2": ref, "ratio": d2(y, w) / ref}
    return out


def parity(rs, py):
    d = rs - py
    return {
        "max_abs": float(np.max(np.abs(d))),
        "rms_err": float(np.sqrt(np.mean(d ** 2))),
        # 逐样本相同时误差为 0、SNR 无穷大（JSON 里记 null，看 identical）
        "snr_db": float(10 * np.log10(np.sum(py ** 2) / np.sum(d ** 2))) if np.any(d) else None,
        "identical": bool(not np.any(d)),
    }


def step_compare():
    out = {"per_clip": {}, "summary": {}}
    for c in clips():
        cid = c["id"]
        x, _ = sf.read(OUT / "src" / f"{cid}_noisy.wav", dtype="float32")
        ref, _ = sf.read(OUT / "src" / f"{cid}_clean.wav", dtype="float32")
        mask, nm = vad_mask(ref)
        ns = ~mask
        ns_sec = float(ns.sum() / SR)
        row = {"nonspeech_s": round(ns_sec, 2), "noisy_si_sdr": si_sdr(x, ref)}
        for name in MODELS:
            py, _ = sf.read(OUT / "py" / f"{cid}_{name}.wav", dtype="float32")
            r = {"py_si_sdr": si_sdr(py, ref)}
            for mode in MODES:
                p = OUT / "rs" / mode / f"{cid}_{name}.wav"
                if not p.exists():
                    continue
                y, _ = sf.read(p, dtype="float32")
                st = json.loads(p.with_suffix(".json").read_text())["stats"]
                m = {"si_sdr": si_sdr(y, ref), "stats": st,
                     "finite": bool(np.all(np.isfinite(y)))}
                if mode == "raw":
                    m["parity_vs_py"] = parity(y, py)
                if ns_sec >= 1.0:
                    m["nonspeech_atten_db"] = float(10 * np.log10(np.mean(y[:nm][ns] ** 2) / np.mean(x[:nm][ns] ** 2)))
                if mode == "fault":
                    safe, _ = sf.read(OUT / "rs" / "safe" / f"{cid}_{name}.wav", dtype="float32")
                    m["max_step"] = float(np.max(np.abs(np.diff(y))))
                    m["max_step_safe"] = float(np.max(np.abs(np.diff(safe))))
                    m["max_step_noisy"] = float(np.max(np.abs(np.diff(x))))
                    m["seams"] = seams(name, y, safe, x)
                r[mode] = m
            if ns_sec >= 1.0:
                r["py_nonspeech_atten_db"] = float(10 * np.log10(np.mean(py[:nm][ns] ** 2) / np.mean(x[:nm][ns] ** 2)))
            row[name] = r
        out["per_clip"][cid] = row
    # 汇总
    pc = out["per_clip"]
    for name in MODELS:
        s = {}
        par = [pc[c][name]["raw"]["parity_vs_py"] for c in pc]
        s["parity_max_abs_worst"] = max(p["max_abs"] for p in par)
        snrs = [p["snr_db"] for p in par if p["snr_db"] is not None]
        s["parity_identical_clips"] = sum(p["identical"] for p in par)
        s["parity_snr_db_min"] = min(snrs) if snrs else None
        s["parity_snr_db_median"] = float(np.median(snrs)) if snrs else None
        s["seam_ratio_max"] = max(v["ratio"] for c in FAULT_CLIPS for v in pc[c][name]["fault"]["seams"].values())
        for k in ("py",):
            s[f"{k}_si_sdr_mean"] = float(np.mean([pc[c][name]["py_si_sdr"] for c in pc]))
        for mode in ("raw", "safe", "floor"):
            s[f"{mode}_si_sdr_mean"] = float(np.mean([pc[c][name][mode]["si_sdr"] for c in pc]))
        s["si_sdr_diff_raw_minus_py_max_abs"] = max(abs(pc[c][name]["raw"]["si_sdr"] - pc[c][name]["py_si_sdr"]) for c in pc)
        ns_clips = [c for c in pc if "py_nonspeech_atten_db" in pc[c][name]]
        s["nonspeech_atten_db_median"] = {
            "py": float(np.median([pc[c][name]["py_nonspeech_atten_db"] for c in ns_clips])),
            **{m: float(np.median([pc[c][name][m]["nonspeech_atten_db"] for c in ns_clips])) for m in ("raw", "safe", "floor")},
            "n_clips": len(ns_clips),
        }
        s["nonspeech_atten_db_min"] = {m: float(min(pc[c][name][m]["nonspeech_atten_db"] for c in ns_clips))
                                       for m in ("raw", "safe", "floor")}
        s["voiced_hop_fraction_safe"] = float(np.mean([pc[c][name]["safe"]["stats"]["voiced_hops"] / pc[c][name]["safe"]["stats"]["hops"] for c in pc]))
        s["fault"] = {c: {k: pc[c][name]["fault"][k] for k in ("finite", "max_step", "max_step_safe", "max_step_noisy", "seams", "si_sdr")}
                      | {"stats": pc[c][name]["fault"]["stats"], "safe_si_sdr": pc[c][name]["safe"]["si_sdr"]}
                      for c in FAULT_CLIPS}
        out["summary"][name] = s
    out["fault_plan"] = {"clips": FAULT_CLIPS, "hops": FAULT_HOPS, "stall_ms": 40, "deadline": "hop 周期的一半"}
    RESULTS.mkdir(exist_ok=True)
    (RESULTS / "compare.json").write_text(json.dumps(out, ensure_ascii=False, indent=1))
    print(json.dumps(out["summary"], ensure_ascii=False, indent=1))


def py_bench_gtcrn(x, n_warm=200, n_timed=6000):
    """同 exp001 bench.py 的 GTCRN 口径：只计 sess.run。"""
    import onnxruntime as ort
    import enhance

    sess = enhance.session(GTCRN_ONNX)
    n_fft, hop = 512, 256
    win = enhance.sqrt_hann_periodic(n_fft)
    caches = {i.name: np.zeros(i.shape, dtype=np.float32) for i in sess.get_inputs()[1:]}
    need = (n_warm + n_timed) * hop + n_fft
    xx = np.tile(x, need // len(x) + 1)[:need]
    names = [o.name for o in sess.get_outputs()]
    t = np.empty(n_timed)
    for k in range(n_warm + n_timed):
        spec = np.fft.rfft(xx[k * hop: k * hop + n_fft] * win)
        feed = {"mix": np.stack([spec.real, spec.imag], -1)[None, :, None].astype(np.float32), **caches}
        t0 = time.perf_counter()
        res = sess.run(names, feed)
        t1 = time.perf_counter()
        caches = {"conv_cache": res[1], "tra_cache": res[2], "inter_cache": res[3]}
        if k >= n_warm:
            t[k - n_warm] = (t1 - t0) * 1e3
    return {"onnxruntime": ort.__version__, "model_ms": {k: float(v) for k, v in dict(
        p50=np.percentile(t, 50), p99=np.percentile(t, 99), p999=np.percentile(t, 99.9), max=t.max()).items()}}


def step_bench():
    import bench_fastenhancer  # exp003 的 FE 计时（同 exp001 口径）

    wav = RES / "gtcrn/test_wavs/mix.wav"
    x, sr = sf.read(wav, dtype="float32")
    assert sr == SR
    res = {"platform": platform.platform(), "machine": platform.machine(), "input": "resources/gtcrn/test_wavs/mix.wav",
           "warmup": 200, "steps": 6000, "rust": {}, "python": {}}
    # Rust 与 Python 交替跑 ROUNDS 轮（同一模型挨着跑），减少「先后跑、负载不同」的偏差；每轮各自热身 200、计 6000
    ROUNDS = 3
    for name in MODELS:
        res["rust"][name], res["python"][name] = [], []
    for r in range(ROUNDS):
        for name, (arg, path, _, _) in MODELS.items():
            j = OUT / f"bench_rs_{name}_r{r}.json"
            subprocess.run([str(VDN), "bench", "--model", arg, "--model-path", str(path), "--input", str(wav),
                            "--json", str(j)], check=True, capture_output=True)
            res["rust"][name].append(json.loads(j.read_text()))
            if name == "gtcrn":
                res["python"][name].append(py_bench_gtcrn(x))
            else:
                res["python"][name].append({"model_ms": bench_fastenhancer.run("m", x)["step_ms"]})
            print("round", r, name, flush=True)
    res["rounds"] = ROUNDS
    res["note"] = ("Rust model_ms = 适配器整次 process（GTCRN 含 Rust 的 STFT/iSTFT；FE-M 的 STFT 在图里）；"
                   "Python model_ms = 只计 sess.run（GTCRN 不含 numpy STFT）。口径对 GTCRN 偏向 Python")
    RESULTS.mkdir(exist_ok=True)
    (RESULTS / "bench.json").write_text(json.dumps(res, ensure_ascii=False, indent=1, default=str))
    for name in MODELS:
        for k in ("p50", "p999", "max"):
            print(name, k, "rust", [round(v["model_ms"][k], 3) for v in res["rust"][name]],
                  "python", [round(v["model_ms"][k], 3) for v in res["python"][name]])


if __name__ == "__main__":
    {"src": step_src, "py": step_py, "rs": step_rs, "compare": step_compare, "bench": step_bench}[sys.argv[1]]()
