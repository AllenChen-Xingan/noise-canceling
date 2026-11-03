"""exp008 · 生成 benchmark 条件格（单类 · 多类 · 整体），两套互相独立的生成器 A / B。

为什么两套：同一个格子规格（如「SNR 0 dB · 咖啡馆底噪」）用两套机理不同、同样说得过去的生成器各造一份。
如果系统排名在 A、B 之间就对不上，这格量到的是生成器自己的特征，不是降噪好坏（synth-standard.md S7 生成器稳健性）。

- 生成器 A：exp006 `sim.py` 的部件（三频带 + 6 个早期反射的合成 RIR · LibriSpeech test 侧说话人 · DEMAND 一组环境）
- 生成器 B：本文件自写（单频带 Polack 指数尾 RIR、直混比按 Sabine 临界距离推 · LibriSpeech train 侧说话人 · DEMAND 另一组环境 / 另一种合成）
- **配对设计**：同一个单类轴上，第 i 段的所有档位共用同一份素材（同一句话、同一段噪声、同一个 RIR），只改这一个因素（S2 可控）
- 真值三轨分存：owner / interf / noise，线性设备链分别作用在每一轨上，三轨相加 = mix（S3）
- 音频写主目录 `_tmp/exp008/<build>/`（仓外）；清单、参数、种子、哈希写 `docs/experiments/exp008-bench-validity/builds/<build>/`（进仓，CLAUDE.md 第五节第 1 条）

用法（worktree 根下）：
  PY=<主目录>/_tmp/exp002-venv/bin/python
  $PY docs/experiments/exp008-bench-validity/cells.py [--build <id>] [--n 6] [--only S-SNR,S-NOISE]
"""
import argparse
import hashlib
import json
import secrets
import sys
import time
from pathlib import Path

import numpy as np
import soundfile as sf
from scipy import signal

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "exp006-scene-sim"))
import sim  # noqa: E402  exp006 生成器的部件（只调用，不复制）

SR = sim.SR
MAIN = Path("/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling")
OUT_ROOT = MAIN / "_tmp/exp008"
CLIP_S = 8.0
HOP = 256  # 与 GTCRN / vdn-eval 同 hop，逐帧标签按它对齐

# ---------------- 格子规格（benchmark.md 第三节的 MVP 子集） ----------------
BASE = {"snr_db": 10.0, "noise": "cafe", "room": "office", "dist_m": 0.5, "sir_db": None, "level_dbfs": -26.0, "device": "laptop"}

AXES = {
    "S-SNR": ("snr_db", [-5.0, 0.0, 5.0, 10.0, 20.0]),
    "S-NOISE": ("noise", ["stationary", "cafe", "babble", "clatter", "traffic", "music"]),
    "S-RT60": ("room", ["car", "office", "cafe", "hall"]),
    "S-SIR": ("sir_db", [None, 20.0, 10.0, 5.0, 0.0, -6.0]),
}
# 多类：只挑交互最可能的两组（benchmark.md 第三节 3.2 的 M01 · M04）
MULTI = {
    "M-SNRxNOISE": [{"snr_db": s, "noise": k} for k in ("stationary", "babble") for s in (0.0, 10.0)],
    "M-SIRxSNR": [{"sir_db": i, "snr_db": s} for i in (10.0, 0.0) for s in (5.0, 20.0)],
}
# 整体：一个场景，按 015 的「两条独立时间轴」排，参数从分布里抽（不配对）
HOLISTIC = {"H-CAFE-CALL": {"dur_s": 30.0}}

ROOMS_B = {  # 生成器 B 的房间：单一 T60（取 A 三频带的中频）+ 体积，用于 Sabine 临界距离
    "car": (0.08, 3.0), "office": (0.35, 60.0), "cafe": (0.7, 250.0), "hall": (1.3, 1500.0), "meeting": (0.5, 90.0),
}


# ---------------- 生成器 B 的部件 ----------------
def rir_b(rng, room, dist):
    """Polack 模型：单频带指数衰减白噪尾 + 直达；直混比按临界距离 rc = 0.057·sqrt(V/T60)（Sabine，全向源）。"""
    t60, vol = ROOMS_B[room]
    n = int(SR * max(0.1, 1.5 * t60))
    t = np.arange(n) / SR
    tail = rng.standard_normal(n) * np.exp(-6.9 * t / t60)
    tail[: int(0.002 * SR)] = 0
    rc = 0.057 * np.sqrt(vol / t60)
    drr_db = 20 * np.log10(rc / dist)
    tail *= np.sqrt(10 ** (-drr_db / 10) / np.sum(tail ** 2))
    tail[0] += 1.0
    return tail


def tilt_noise(rng, n):
    """B 的稳态噪声：随机谱斜率的有色噪声（−3…−9 dB/倍频程）+ 慢起伏，不含叶片音（与 A 的 synth_hvac 机理不同）。"""
    w = np.fft.rfft(rng.standard_normal(n))
    f = np.maximum(np.arange(len(w)) * SR / n, 20.0)
    slope = rng.uniform(-9, -3)
    w *= (f / 1000.0) ** (slope / 20 / np.log10(2))
    x = np.fft.irfft(w, n)
    return x / sim.rms(x), f"tilt_noise({slope:.1f}dB/oct)"


def demand_any(rng, env, n, part):
    """DEMAND 任一环境的一段；part = 'a'（前 60%）/ 'b'（后 40%）。A、B 用不同环境，part 只为防同段复用。"""
    return sim.demand(rng, env, "train" if part == "a" else "test", n)


NOISE_A = {  # 名义噪声类型 → 生成器 A 的素材
    "stationary": lambda rng, n, room: sim.synth_hvac(rng, n),
    "cafe": lambda rng, n, room: demand_any(rng, "PCAFETER", n, "a"),
    "babble": lambda rng, n, room: _babble(rng, n, room, "test", sim.make_rir),
    "clatter": lambda rng, n, room: demand_any(rng, "DKITCHEN", n, "a"),
    "traffic": lambda rng, n, room: demand_any(rng, "STRAFFIC", n, "a"),
    "music": lambda rng, n, room: sim.synth_music(rng, n, room),
}
NOISE_B = {
    "stationary": lambda rng, n, room: tilt_noise(rng, n),
    "cafe": lambda rng, n, room: demand_any(rng, "SCAFE", n, "b"),
    "babble": lambda rng, n, room: _babble(rng, n, room, "train", lambda r, rm, d: rir_b(r, rm, d)),
    "clatter": lambda rng, n, room: sim.synth_dishes(rng, n, room),
    "traffic": lambda rng, n, room: demand_any(rng, "TBUS", n, "b"),
    "music": lambda rng, n, room: sim.file_music(rng, n),
}


def _babble(rng, n, room, split, rir_fn):
    """远处 6 人 babble（两套生成器说话人不相交、RIR 机理不同）。"""
    pool = sorted(sim.SPK[split])
    out = np.zeros(n)
    for s in rng.choice(pool, size=6, replace=False):
        a, _ = sim.speech_track(rng, split, n, spk=s, gap=(0.1, 1.5))
        a = np.roll(a, int(rng.integers(0, n)))
        d = rng.uniform(2.0, 4.0)
        out += sim.reverb(a, rir_fn(rng, room, d)) / d
    return out, f"babble6({split})"


def device_chain(x, kind):
    """线性设备链（同一份作用在三轨上，三轨仍可相加）：laptop = 100 Hz 二阶高通 + 7 kHz 低通（内置麦量级的假设；按 D62 开发阶段不实测，发版前试用时再核）。"""
    if kind == "none":
        return x
    if kind == "laptop":
        x = sim.butter(x, "highpass", 100, 2)
        return sim.butter(x, "lowpass", 7000, 2)
    raise ValueError(kind)


def rir_stats(h):
    """实测 RIR：Schroeder 反向积分，T20 外推 T60（−5…−25 dB 线性拟合）；直混比 = 直达 ±2.5 ms 能量 ÷ 其余。"""
    if len(h) < 2:
        return {"t60_s": 0.0, "drr_db": None}
    e = np.cumsum(h[::-1] ** 2)[::-1]
    edc = 10 * np.log10(e / e[0] + 1e-30)
    t = np.arange(len(h)) / SR
    sel = (edc <= -5) & (edc >= -25)
    t60 = float(-60 / np.polyfit(t[sel], edc[sel], 1)[0]) if sel.sum() > 10 else None
    k = int(0.0025 * SR)
    d = float(np.sum(h[:k] ** 2))
    r = float(np.sum(h[k:] ** 2))
    return {"t60_s": None if t60 is None else round(t60, 3), "drr_db": round(10 * np.log10(d / max(r, 1e-30)), 2)}


def active_level(x, thr_db=-35):
    m = sim.active_mask(x, frame=320, thr_db=thr_db)
    return sim.rms(x[m]) if m.any() else sim.rms(x)


def frame_labels(owner_dry, interf_dry):
    """逐 hop 标签：0 无人声 · 1 只有主讲人 · 2 只有干扰者 · 3 重叠。真值规则写死：干净轨 −35 dB（相对本轨最大帧）能量门限，与模型输入不同源（012 §3.4 限定 3）。"""
    def act(x):
        if x is None or not np.any(x):
            return np.zeros(len(owner_dry) // HOP, bool)
        nf = len(x) // HOP
        e = 10 * np.log10(np.mean(x[: nf * HOP].reshape(nf, HOP) ** 2, axis=1) + 1e-12)
        return e > e.max() - 35
    o, i = act(owner_dry), act(interf_dry)
    return (o.astype(int) + 2 * i.astype(int)).tolist()


# ---------------- 一段 ----------------
def make_clip(gen, cond, mat_seed, n):
    """gen = 'A' / 'B'；cond = 完整条件（BASE 覆盖后）；mat_seed = 素材种子（配对设计：同轴各档共用）。"""
    rng = np.random.default_rng(mat_seed)
    split = "test" if gen == "A" else "train"
    rir_fn = sim.make_rir if gen == "A" else (lambda r, rm, d: rir_b(r, rm, d))
    noise_tab = NOISE_A if gen == "A" else NOISE_B
    owner_dry, spk = sim.speech_track(rng, split, n)
    h_o = rir_fn(rng, cond["room"], cond["dist_m"])
    owner = sim.reverb(owner_dry, h_o)
    # 干扰者（邻座）：1.5 m，独立时间轴（停顿更长），按主动电平定 SIR
    rng_i = np.random.default_rng(mat_seed + 7)
    pool = [s for s in sorted(sim.SPK[split]) if s != spk]
    nb = str(rng_i.choice(pool))
    interf_dry, _ = sim.speech_track(rng_i, split, n, spk=nb, gap=(0.6, 2.5))
    interf_dry = np.roll(interf_dry, int(rng_i.integers(0, n)))
    interf = sim.reverb(interf_dry, rir_fn(rng_i, cond["room"], 1.5))
    rng_n = np.random.default_rng(mat_seed + 13)
    noise, ndesc = noise_tab[cond["noise"]](rng_n, n, cond["room"])
    noise = sim.fit(np.asarray(noise, np.float64), n)
    # 定义点（S2）：电平、SIR、SNR 一律在**设备链之后**（= 麦克风输出、模型输入）量。先过线性设备链，再定增益。
    # 先定后过会让实测 SNR 偏离名义值（smoke：最多 6.3 dB，设备链的高 / 低通吃掉的噪声能量比语音多）
    owner = device_chain(owner, cond["device"])
    interf = device_chain(interf, cond["device"])
    noise = device_chain(noise, cond["device"])
    g = 10 ** (cond["level_dbfs"] / 20) / active_level(owner)
    owner *= g
    lo = active_level(owner)
    if cond["sir_db"] is None:
        interf = np.zeros(n)
        interf_dry = None
    else:
        interf *= lo / active_level(interf) * 10 ** (-cond["sir_db"] / 20)
    noise *= lo / sim.rms(noise) * 10 ** (-cond["snr_db"] / 20)
    tracks = {"owner": owner, "interf": interf, "noise": noise}
    mix = tracks["owner"] + tracks["interf"] + tracks["noise"]
    peak = float(np.max(np.abs(mix)))
    if peak > 0.99:  # 不削波：整体降，记录（削波另有专门的格）
        for k in tracks:
            tracks[k] *= 0.99 / peak
        mix *= 0.99 / peak
    labels = frame_labels(owner_dry, interf_dry)
    meta = {"speaker": spk, "neighbor": nb if cond["sir_db"] is not None else None, "noise_src": ndesc,
            "rir": rir_stats(h_o), "peak_scaled": peak > 0.99}
    return mix, tracks, labels, meta


def cond_of(over):
    c = dict(BASE)
    c.update(over)
    return c


def jobs(n_clips, only):
    out = []
    for cell, (key, levels) in AXES.items():
        for gen in ("A", "B"):
            for i in range(n_clips):
                for lv in levels:
                    out.append({"cell": cell, "gen": gen, "clip": i, "level": lv, "cond": cond_of({key: lv}),
                                "mat_seed": 20261031 * 10 + hash_int(cell, gen, i)})
    for cell, conds in MULTI.items():
        for gen in ("A", "B"):
            for i in range(n_clips):
                for c in conds:
                    out.append({"cell": cell, "gen": gen, "clip": i, "level": c, "cond": cond_of(c),
                                "mat_seed": 20261031 * 10 + hash_int(cell, gen, i)})
    return [j for j in out if not only or j["cell"] in only]


def hash_int(*parts):
    return int(hashlib.sha256("/".join(map(str, parts)).encode()).hexdigest()[:8], 16)


def level_tag(lv):
    if isinstance(lv, dict):
        return "_".join(f"{k}{'none' if v is None else v}" for k, v in sorted(lv.items()))
    return "none" if lv is None else str(lv)


def sha256(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def pcm_sha256(x):
    """内容哈希：对写入的 float32 样本算，不对 wav 文件算——libsndfile 的 float wav 带 PEAK 块（含时间戳），同样的样本两次写出文件哈希不同（smoke 实测）。"""
    return hashlib.sha256(np.ascontiguousarray(x, dtype="<f4").tobytes()).hexdigest()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--build")
    ap.add_argument("--n", type=int, default=6)
    ap.add_argument("--only", default="")
    ap.add_argument("--out-root", default=str(OUT_ROOT))
    ap.add_argument("--docs-root", default=str(HERE / "builds"), help="清单写哪；冒烟时指到仓外，免得试错产物进仓")
    a = ap.parse_args()
    build = a.build or time.strftime("%Y%m%d%H%M%S") + "-" + secrets.token_hex(2)
    only = [s for s in a.only.split(",") if s]
    root = Path(a.out_root) / build
    n = int(CLIP_S * SR)
    rows = []
    t0 = time.time()
    for j in jobs(a.n, only):
        mix, tracks, labels, meta = make_clip(j["gen"], j["cond"], j["mat_seed"], n)
        d = root / j["gen"] / j["cell"]
        d.mkdir(parents=True, exist_ok=True)
        stem = f"c{j['clip']:02d}_{level_tag(j['level'])}"
        files = {}
        for k, x in (("mix", mix), *tracks.items()):
            p = d / f"{stem}_{k}.wav"
            sf.write(p, x.astype(np.float32), SR, subtype="FLOAT")
            files[k] = {"path": str(p.relative_to(root)), "pcm_sha256": pcm_sha256(x)}
        rows.append({**{k: j[k] for k in ("cell", "gen", "clip", "mat_seed")}, "level": j["level"], "cond": j["cond"],
                     "files": files, "labels_hop": HOP, "labels": labels, **meta})
    docs = Path(a.docs_root) / build
    docs.mkdir(parents=True, exist_ok=True)
    with open(docs / "truth.jsonl", "w") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    manifest = {"build": build, "audio_root": str(root.relative_to(MAIN)) if root.is_relative_to(MAIN) else "(仓外临时目录)", "sr": SR, "clip_s": CLIP_S, "n_clips": a.n,
                "base": BASE, "axes": {k: v[1] for k, v in AXES.items()}, "multi": MULTI, "count": len(rows),
                "script_sha256": sha256(__file__), "sim_sha256": sha256(sim.__file__), "seconds": round(time.time() - t0, 1)}
    (docs / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=1))
    print(json.dumps({k: manifest[k] for k in ("build", "count", "seconds")}, ensure_ascii=False))


if __name__ == "__main__":
    main()
