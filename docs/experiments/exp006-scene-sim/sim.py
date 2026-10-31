"""exp006 场景合成器（模拟环境第一版，005 第三节 C / D41）：按参数表 recipes.json 批量生成带真值标签的 16 kHz 片段。

每段 = 目标说话人（近讲，经房间脉冲响应）+ 一种细场景的噪音 + 设备效果，真值写进 truth.jsonl。
三份数据：train（调分类器）· test（报准确率 / 混淆矩阵，说话人与噪音时段都和 train 不重叠）· seq（切换序列，量抖动与切换延迟）。

- 房间脉冲响应：自己合成（直达声 + 早期反射 + 三频带指数衰减尾巴），不下载 RIR 库
- 噪音：DEMAND 实录（PCAFETER / SCAFE / PRESTO / DKITCHEN / STRAFFIC / TBUS）+ 合成（底噪、空调、键盘、杯碟、音乐、多人 babble）
- 设备：麦克风频响倾斜、蓝牙窄带（降到 8 kHz 再升回，模拟 HFP CVSD）、削波
- 随机数：每段一个独立种子 = 分组种子 + 序号，可单段重现；多进程生成结果与单进程相同

输出：音频 → <worktree>/_tmp/20261031-exp006/<build>/{train,test,seq}/（git 忽略）
      真值、文件哈希、参数表快照 → docs/experiments/exp006-scene-sim/builds/<build>/（提交，CLAUDE.md 第五节第 1 条）
用法（worktree 根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp006-scene-sim/sim.py [--build <id>] [--only train,test,seq] [--n 20]
"""
import argparse
import hashlib
import json
import os
import secrets
import time
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import numpy as np
import soundfile as sf
from scipy import signal

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
RECIPES = json.loads((HERE / "recipes.json").read_text())
SR = RECIPES["sr"]
DATA = Path.home() / "vdn-data"
LIBRI = DATA / "librispeech/LibriSpeech/test-clean"
DEMAND = DATA / "demand"
MUSIC_FILE = DATA / "music/Scott_Joplin_-_Maple_Leaf_Rag.ogg"


# ---------------- 小工具 ----------------
def rms(a):
    return float(np.sqrt(np.mean(np.asarray(a, np.float64) ** 2)) + 1e-12)


def db(x):
    return 20 * np.log10(max(x, 1e-12))


def u(rng, lo_hi):
    lo, hi = lo_hi
    return float(rng.uniform(lo, hi))


def fit(a, n):
    """把 a 截 / 循环到 n 个样本。"""
    if len(a) >= n:
        return a[:n]
    reps = int(np.ceil(n / max(len(a), 1)))
    return np.tile(a, reps)[:n]


def butter(x, kind, f, order=2):
    sos = signal.butter(order, f, btype=kind, fs=SR, output="sos")
    return signal.sosfilt(sos, x)


def pink(rng, n):
    w = np.fft.rfft(rng.standard_normal(n))
    f = np.arange(len(w))
    w[1:] /= np.sqrt(f[1:])
    w[0] = 0
    x = np.fft.irfft(w, n)
    return x / rms(x)


# ---------------- 房间脉冲响应 ----------------
def make_rir(rng, room, dist):
    """合成 RIR。直达声幅度 1；尾巴能量按「临界距离」定直混比：DRR = 20·log10(rc / dist)，rc 取 0.8 m（小房间量级）。"""
    rt = RECIPES["rooms"][room]["rt60"]
    if max(rt) <= 0:
        return np.array([1.0])
    n = int(SR * max(0.08, 1.2 * max(rt)))
    t = np.arange(n) / SR
    tail = np.zeros(n)
    bands = [("lowpass", 500), ("bandpass", (500, 4000)), ("highpass", 4000)]
    for (kind, f), r in zip(bands, rt):
        nb = butter(rng.standard_normal(n), kind, f, 4)
        tail += nb * np.exp(-6.9 * t / max(r, 0.02))
    tail[: int(0.004 * SR)] = 0  # 尾巴从 4 ms 后开始
    rc = 0.8 * (0.5 if room == "car" else 1.0)
    drr_db = 20 * np.log10(rc / dist)
    tail *= np.sqrt(10 ** (-drr_db / 10) / np.sum(tail ** 2))
    h = tail
    h[0] += 1.0
    for _ in range(6):  # 早期反射：5–30 ms 内 6 个离散反射
        k = int(rng.uniform(0.005, 0.03) * SR)
        h[k] += rng.uniform(-0.5, 0.5) * min(1.0, rc / dist)
    return h


def reverb(x, h):
    return signal.fftconvolve(x, h)[: len(x)]


# ---------------- 语音（LibriSpeech）----------------
def libri_index():
    spk = {}
    for f in sorted(LIBRI.rglob("*.flac")):
        spk.setdefault(f.name.split("-")[0], []).append(str(f))
    ids = sorted(spk)
    return {"train": {s: spk[s] for s in ids[0::2]}, "test": {s: spk[s] for s in ids[1::2]}}


SPK = libri_index()


def speech_track(rng, split, n, spk=None, gap=(0.25, 0.8)):
    """一位说话人的若干句子首尾相接（中间留停顿），填满 n 个样本。返回 (波形, 说话人号)。"""
    spk = spk or rng.choice(sorted(SPK[split]))
    files = SPK[split][spk]
    out = np.zeros(n)
    pos = 0
    order = rng.permutation(len(files))
    i = 0
    while pos < n:
        a, _ = sf.read(files[order[i % len(order)]], dtype="float32")
        a = a / rms(a)
        m = min(len(a), n - pos)
        out[pos: pos + m] = a[:m]
        pos += m + int(u(rng, gap) * SR)
        i += 1
    return out, spk


def active_mask(x, frame=320, thr_db=-35):
    nf = len(x) // frame
    e = 10 * np.log10(np.mean(x[: nf * frame].reshape(nf, frame) ** 2, axis=1) + 1e-12)
    m = np.repeat(e > e.max() + thr_db, frame)
    return np.concatenate([m, np.zeros(len(x) - len(m), bool)])


# ---------------- 噪音源 ----------------
def demand(rng, env, split, n):
    d = DEMAND / env
    ch = int(rng.integers(1, 17))
    path = d / f"ch{ch:02d}.wav"
    info = sf.info(str(path))
    total = info.frames
    frac = RECIPES["demand_split"]["train_frac"]
    lo, hi = (0, int(total * frac)) if split == "train" else (int(total * frac), total)
    need = int(n * info.samplerate / SR) + 16
    start = int(rng.integers(lo, max(lo + 1, hi - need)))
    a, sr = sf.read(str(path), start=start, frames=need, dtype="float32", always_2d=True)
    a = a[:, 0].astype(np.float64)
    if sr != SR:
        g = np.gcd(sr, SR)
        a = signal.resample_poly(a, SR // g, sr // g)
    return fit(a, n), f"DEMAND/{env}/ch{ch:02d}@{start / sr:.1f}s"


def synth_room(rng, n):
    x = butter(pink(rng, n), "lowpass", u(rng, (2000, 6000)))
    hum_f = rng.choice([50, 60])
    t = np.arange(n) / SR
    x = x / rms(x) + 0.3 * sum(np.sin(2 * np.pi * hum_f * k * t + rng.uniform(0, 6)) / k for k in (1, 2, 3))
    return x, f"synth_room(hum={hum_f})"


def synth_hvac(rng, n):
    t = np.arange(n) / SR
    brown = np.cumsum(rng.standard_normal(n))
    brown = butter(brown - np.mean(brown), "highpass", 20)
    x = 0.6 * brown / rms(brown) + pink(rng, n)
    x = butter(x, "lowpass", u(rng, (800, 3000)))
    mod = 1 + u(rng, (0.05, 0.25)) * np.sin(2 * np.pi * u(rng, (0.1, 0.5)) * t)
    blade = u(rng, (30, 120))
    x = x / rms(x) * mod + 0.25 * sum(np.sin(2 * np.pi * blade * k * t) / k for k in (1, 2, 3, 4))
    return x, f"synth_hvac(blade={blade:.0f}Hz)"


def synth_keyboard(rng, n, room):
    x = np.zeros(n)
    t = 0.0
    while t < n / SR:
        burst = u(rng, (0.8, 3.0))
        rate = u(rng, (4, 9))
        end = t + burst
        while t < end and t < n / SR:
            k = int(t * SR)
            L = int(u(rng, (0.002, 0.008)) * SR)
            click = butter(rng.standard_normal(L), "bandpass", (u(rng, (800, 1500)), u(rng, (4000, 7000)))) * np.exp(-np.arange(L) / (L / 4))
            thump = np.sin(2 * np.pi * u(rng, (120, 300)) * np.arange(L * 3) / SR) * np.exp(-np.arange(L * 3) / L) * 0.3
            seg = np.zeros(L * 3)
            seg[:L] += click
            seg += thump
            m = min(len(seg), n - k)
            x[k: k + m] += seg[:m] * u(rng, (0.5, 1.0))
            t += rng.exponential(1 / rate)
        t += u(rng, (0.3, 2.0))
    x = reverb(x, make_rir(rng, room, u(rng, (0.5, 1.5)))) + 0.01 * pink(rng, n)
    return x, "synth_keyboard"


def synth_dishes(rng, n, room):
    x = np.zeros(n)
    t = rng.exponential(0.5)
    rate = u(rng, (0.6, 3.0))
    while t < n / SR:
        k = int(t * SR)
        L = int(0.4 * SR)
        tt = np.arange(L) / SR
        hit = sum(np.sin(2 * np.pi * u(rng, (1200, 8000)) * tt) * np.exp(-tt / u(rng, (0.03, 0.3))) * rng.uniform(0.2, 1)
                  for _ in range(int(rng.integers(3, 8))))
        hit[: int(0.003 * SR)] += rng.standard_normal(int(0.003 * SR)) * 0.5
        m = min(L, n - k)
        x[k: k + m] += hit[:m] * u(rng, (0.3, 1.0))
        t += rng.exponential(1 / rate)
    x = reverb(x, make_rir(rng, room, u(rng, (1.0, 3.0)))) + 0.03 * pink(rng, n)
    return x, "synth_dishes"


def synth_music(rng, n, room):
    """合成音乐：随机调、和弦进行、加法合成的「钢琴」旋律 + 低音 + 可选鼓。不追求好听，要的是音乐的统计特征（稳定音高、节拍）。"""
    t_n = n / SR
    bpm = u(rng, (70, 140))
    beat = 60 / bpm
    root = 48 + int(rng.integers(0, 12))
    prog = [[0, 4, 7], [5, 9, 12], [7, 11, 14], [9, 12, 16]]
    order = rng.permutation(4)
    x = np.zeros(n)
    bright = u(rng, (0.4, 0.9))

    def note(midi, start, dur, amp):
        f = 440 * 2 ** ((midi - 69) / 12)
        L = int(dur * SR)
        k = int(start * SR)
        if k >= n:
            return
        tt = np.arange(L) / SR
        env = np.exp(-tt / (0.3 + dur * 0.5)) * np.minimum(1, tt / 0.005)
        w = sum(bright ** h * np.sin(2 * np.pi * f * (h + 1) * tt) for h in range(6) if f * (h + 1) < SR / 2)
        m = min(L, n - k)
        x[k: k + m] += amp * env[:m] * w[:m]

    bar = 0
    t = 0.0
    while t < t_n:
        chord = prog[order[bar % 4]]
        note(root - 12 + chord[0], t, beat * 4, 0.8)
        for b in range(4):
            for c in chord:
                note(root + c, t + b * beat, beat * 0.9, 0.25)
            for s in range(int(rng.integers(1, 3))):
                note(root + 12 + int(rng.choice([0, 2, 4, 5, 7, 9, 11, 12])), t + b * beat + s * beat / 2, beat / 2, 0.4)
        bar += 1
        t += beat * 4
    if rng.random() < 0.6:  # 鼓
        t = 0.0
        while t < t_n:
            k = int(t * SR)
            L = int(0.15 * SR)
            tt = np.arange(L) / SR
            kick = np.sin(2 * np.pi * (50 + 80 * np.exp(-tt / 0.02)) * tt) * np.exp(-tt / 0.08)
            m = min(L, n - k)
            x[k: k + m] += 0.9 * kick[:m]
            hk = int((t + beat / 2) * SR)
            if hk < n:
                hl = min(int(0.05 * SR), n - hk)
                x[hk: hk + hl] += 0.15 * butter(rng.standard_normal(hl), "highpass", 6000) * np.exp(-np.arange(hl) / (0.01 * SR))
            t += beat
    x = reverb(x, make_rir(rng, room, u(rng, (1.5, 4.0))))
    return x, f"synth_music(bpm={bpm:.0f})"


def file_music(rng, n):
    info = sf.info(str(MUSIC_FILE))
    need = int(n * info.samplerate / SR) + 16
    start = int(rng.integers(0, info.frames - need))
    a, sr = sf.read(str(MUSIC_FILE), start=start, frames=need, dtype="float32", always_2d=True)
    a = a.mean(axis=1).astype(np.float64)
    g = np.gcd(sr, SR)
    return fit(signal.resample_poly(a, SR // g, sr // g), n), f"MapleLeafRag@{start / sr:.1f}s"


def talkers(rng, split, n, k, room, exclude):
    pool = [s for s in sorted(SPK[split]) if s != exclude]
    out = np.zeros(n)
    used = []
    for s in rng.choice(pool, size=k, replace=False):
        a, _ = speech_track(rng, split, n, spk=s, gap=(0.1, 1.5))
        a = np.roll(a, int(rng.integers(0, n)))
        d = u(rng, RECIPES["interferer_distance_m"])
        out += reverb(a, make_rir(rng, room, d)) * (0.8 / d) * u(rng, (0.6, 1.0))
        used.append(str(s))
    return out, used


def noise_for(rng, fine, split, n, room, target_spk):
    """按细场景抽一种噪音源，返回 (噪音波形, 来源描述, 干扰说话人数)。"""
    srcs = [s for s in RECIPES["noise_sources"][fine] if s.get("only_split", split) == split]
    src = srcs[int(rng.integers(0, len(srcs)))]
    kind = src["kind"]
    if kind == "demand":
        x, desc = demand(rng, src["env"], split, n)
        return x, desc, -1  # -1 = 实录 babble，人数未知（多人）
    if kind == "synth_room":
        return (*synth_room(rng, n), 0)
    if kind == "synth_hvac":
        return (*synth_hvac(rng, n), 0)
    if kind == "synth_keyboard":
        return (*synth_keyboard(rng, n, room), 0)
    if kind == "synth_dishes":
        return (*synth_dishes(rng, n, room), 0)
    if kind == "synth_music":
        return (*synth_music(rng, n, room), 0)
    if kind == "file_music":
        return (*file_music(rng, n), 0)
    k = int(rng.integers(src["talkers"][0], src["talkers"][1] + 1))
    x, used = talkers(rng, split, n, k, room, target_spk)
    desc = f"{kind}x{k}({','.join(used)})"
    if kind == "talkers":
        bed_name = rng.choice(src["bed"])
        bed = synth_room(rng, n)[0] if bed_name == "synth_room" else demand(rng, bed_name, split, n)[0]
        x = x / rms(x) + bed / rms(bed) * 10 ** (u(rng, src["bed_db"]) / 20)
        desc += f"+bed:{bed_name}"
    return x, desc, k


# ---------------- 设备 ----------------
def pick_device(rng):
    p = RECIPES["device"]
    names = ["none", "bt_narrow", "clip"]
    return names[int(rng.choice(3, p=np.array([p[k] for k in names]) / sum(p[k] for k in names)))]


def apply_device(rng, x, dev):
    p = RECIPES["device"]
    tilt = u(rng, p["mic_tilt_db_per_oct"])
    # 麦克风倾斜：以 1 kHz 为轴，用一阶搁架近似（低频 −tilt·2 倍频程、高频 +tilt·2 倍频程）
    hs = signal.butter(1, 2000, "highpass", fs=SR, output="sos")
    ls = signal.butter(1, 500, "lowpass", fs=SR, output="sos")
    g_hi, g_lo = 10 ** (tilt * 2 / 20) - 1, 10 ** (-tilt * 2 / 20) - 1
    x = x + g_hi * signal.sosfilt(hs, x) + g_lo * signal.sosfilt(ls, x)
    info = {"mic_tilt_db_per_oct": round(tilt, 2)}
    if dev == "bt_narrow":
        y = signal.resample_poly(x, 1, 2)
        y = np.round(y * 2 ** 11) / 2 ** 11  # 粗量化，模拟窄带编码的颗粒
        x = signal.resample_poly(y, 2, 1)[: len(x)]
    elif dev == "clip":
        drive = u(rng, p["clip_drive_db"])
        peak = np.max(np.abs(x)) + 1e-9
        x = np.clip(x / peak * 10 ** (drive / 20) * 0.5, -1, 1)
        info["clip_drive_db"] = round(drive, 1)
    return x, info


# ---------------- 一段 ----------------
def make_clip(job):
    split, idx, seed = job["split"], job["idx"], job["seed"]
    rng = np.random.default_rng(seed)
    n = int(RECIPES["clip_s"] * SR)
    fines = list(RECIPES["fine"])
    w = np.array([RECIPES["fine"][f]["weight"] for f in fines])
    fine = fines[int(rng.choice(len(fines), p=w / w.sum()))]
    cfg = RECIPES["fine"][fine]
    room = str(rng.choice(cfg["rooms"]))
    has_target = rng.random() >= RECIPES["no_target_prob"]
    tgt_spk = None
    target = np.zeros(n)
    if has_target:
        pre = int(u(rng, (0.0, 1.5)) * SR)
        sp, tgt_spk = speech_track(rng, split, n - pre)
        target[pre:] = sp
        target = reverb(target, make_rir(rng, room, u(rng, RECIPES["target_distance_m"])))
        drift = u(rng, RECIPES["target_drift_db"])  # 说话人前后挪动 / 转头：慢变化的电平
        curve = signal.resample(rng.standard_normal(6), n)
        curve = curve / (np.max(np.abs(curve)) + 1e-9) * drift / 2
        target *= 10 ** (curve / 20)
    noise, desc, n_talk = noise_for(rng, fine, split, n, room, tgt_spk)
    level = u(rng, RECIPES["target_level_dbfs"])
    if has_target:
        act = active_mask(target)
        g = 10 ** (level / 20) / rms(target[act])
        target *= g
        snr = u(rng, cfg["snr_db"])
        noise *= rms(target[act]) / rms(noise) / 10 ** (snr / 20)
    else:
        snr = None
        noise *= 10 ** ((level - u(rng, (cfg["snr_db"][0], cfg["snr_db"][1]))) / 20) / rms(noise)
    dev = pick_device(rng)
    noisy, dev_info = apply_device(rng, target + noise, dev)
    peak = float(np.max(np.abs(noisy)))
    if peak > 0.99 and dev != "clip":
        noisy *= 0.99 / peak
        target *= 0.99 / peak
    cid = f"{split[:2]}{idx:04d}"
    truth = {"id": cid, "split": split, "seed": seed, "fine": fine, "user": cfg["user"], "room": room,
             "rt60_mid": RECIPES["rooms"][room]["rt60"][1], "has_target": bool(has_target), "target_spk": tgt_spk,
             "snr_db": None if snr is None else round(snr, 2), "level_dbfs": round(level, 1), "noise": desc,
             "n_interferers": n_talk, "device": dev, **dev_info, "dur_s": RECIPES["clip_s"]}
    return cid, noisy, target if has_target else None, truth


def make_seq(job):
    """切换序列：同一说话人说 60 s，背景按 4 个不同细场景轮换，段间噪音交叉淡化 1 s。"""
    split, idx, seed = "test", job["idx"], job["seed"]
    rng = np.random.default_rng(seed)
    S = RECIPES["seq"]
    seg_n = int(S["segment_s"] * SR)
    xf = int(S["xfade_s"] * SR)
    n = seg_n * S["segments"]
    fines = list(RECIPES["fine"])
    chosen = []
    while len(chosen) < S["segments"]:
        f = fines[int(rng.integers(0, len(fines)))]
        if not chosen or (f != chosen[-1] and RECIPES["fine"][f]["user"] != RECIPES["fine"][chosen[-1]]["user"] or rng.random() < 0.3 and f != chosen[-1]):
            chosen.append(f)
    room = "living"
    sp, spk = speech_track(rng, split, n)
    target = reverb(sp, make_rir(rng, room, 0.5))
    act = active_mask(target)
    target *= 10 ** (-26 / 20) / rms(target[act])
    noise = np.zeros(n + xf)
    timeline = []
    descs = []
    for k, f in enumerate(chosen):
        cfg = RECIPES["fine"][f]
        r = str(rng.choice(cfg["rooms"]))
        x, desc, _ = noise_for(rng, f, split, seg_n + xf, r, spk)
        snr = u(rng, cfg["snr_db"])
        x *= rms(target[act]) / rms(x) / 10 ** (snr / 20)
        ramp = np.ones(seg_n + xf)
        if k > 0:
            ramp[:xf] = np.linspace(0, 1, xf)
        ramp[seg_n:] = np.linspace(1, 0, xf)
        s0 = k * seg_n
        noise[s0: s0 + seg_n + xf] += x * ramp
        timeline.append({"start_s": k * S["segment_s"], "end_s": (k + 1) * S["segment_s"], "fine": f, "user": cfg["user"], "snr_db": round(snr, 2)})
        descs.append(desc)
    noisy = target + noise[:n]
    noisy, dev_info = apply_device(rng, noisy, "none")
    peak = float(np.max(np.abs(noisy)))
    if peak > 0.99:
        noisy *= 0.99 / peak
        target *= 0.99 / peak
    cid = f"sq{idx:04d}"
    truth = {"id": cid, "split": "seq", "seed": seed, "timeline": timeline, "noise": descs, "target_spk": spk,
             "device": "none", **dev_info, "dur_s": n / SR}
    return cid, noisy, target, truth


def run_job(job):
    cid, noisy, clean, truth = (make_seq if job["split"] == "seq" else make_clip)(job)
    out = Path(job["out"])
    sf.write(out / f"{cid}_noisy.wav", noisy.astype(np.float32), SR, subtype="PCM_16")
    if clean is not None and job["split"] != "train":
        sf.write(out / f"{cid}_clean.wav", clean.astype(np.float32), SR, subtype="PCM_16")
    return truth


def sha256(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--build", default=None)
    ap.add_argument("--only", default="train,test,seq")
    ap.add_argument("--n", type=int, default=None, help="每组只生成前 n 段（冒烟用）")
    a = ap.parse_args()
    build = a.build or time.strftime("%Y%m%d%H%M%S") + "-" + secrets.token_hex(2)
    audio_root = ROOT / "_tmp/20261031-exp006" / build
    doc_root = HERE / "builds" / build
    doc_root.mkdir(parents=True, exist_ok=True)
    jobs = []
    for split in a.only.split(","):
        (audio_root / split).mkdir(parents=True, exist_ok=True)
        cnt = RECIPES["count"][split] if a.n is None else min(a.n, RECIPES["count"][split])
        for i in range(cnt):
            jobs.append({"split": split, "idx": i, "seed": RECIPES["seed"][split] * 1000 + i, "out": str(audio_root / split)})
    t0 = time.time()
    with ProcessPoolExecutor(max_workers=max(1, (os.cpu_count() or 4) // 2)) as ex:
        truths = list(ex.map(run_job, jobs, chunksize=4))
    with open(doc_root / "truth.jsonl", "w") as f:
        for t in truths:
            f.write(json.dumps(t, ensure_ascii=False) + "\n")
    files = {str(p.relative_to(audio_root)): sha256(p) for p in sorted(audio_root.rglob("*.wav"))}
    (doc_root / "files.json").write_text(json.dumps(files, indent=0))
    (doc_root / "recipes.snapshot.json").write_text((HERE / "recipes.json").read_text())
    (doc_root / "build.json").write_text(json.dumps({"build": build, "audio_dir": str(audio_root.relative_to(ROOT)), "n": len(truths),
                                                     "seconds": round(time.time() - t0, 1), "only": a.only, "n_limit": a.n}, indent=1))
    print(build, len(truths), f"{time.time() - t0:.1f}s")


if __name__ == "__main__":
    main()
