"""exp005 第一步：合成「主人 + 邻座」素材（16 kHz 单声道），三轨分存 + 时间表。

设计（005 decision「PSE 专属量法」：三轨分存、时间表、干扰者远场 / 主人近场）：
- 主人 = LibriSpeech test-clean 里总时长最长的那位说话人（固定，跨所有段同一人；每段用不同的句子）
- 邻座 = 其他 LibriSpeech 说话人
- 近场 / 远场：主人卷一个短混响（RT60 0.15 s、直达声强），邻座卷长混响（RT60 0.6 s、直达声弱）。
  RIR 是合成的（指数衰减高斯噪声 + 直达脉冲，固定种子）——没有实测 RIR，**这是简化**
- 电平：主人有声段 RMS 定在 -23 dBFS；SIR = 主人有声 RMS / 每位邻座有声 RMS；SNR = 主人有声 RMS / 底噪 RMS
- 底噪：DEMAND PCAFETER（自助餐厅，自带远处人声嘈杂）；c07 不加底噪（算「无噪格」的 IRL）
- 每段从冷启动开始（模拟一场新会议）

8 段（见 SCEN 表）：一人轮流说 ×3 · 邻桌多人 ×2 · 开头别人先说 25 s / 45 s · 只有主人 + 底噪
输出：<主目录>/_tmp/20261031-exp005/src/<id>_{mix,owner,interf,noise}.wav；本目录 clips.json（时间表、参数、哈希）
用法（项目根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp005-p2-speaker/make_clips.py
"""
import hashlib
import json
import sys
from pathlib import Path

import numpy as np
import soundfile as sf
from scipy.signal import fftconvolve

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "exp003-offline-ab"))
from _paths import MAIN  # noqa: E402

OUT = MAIN / "_tmp/20261031-exp005/src"
DATA = Path.home() / "vdn-data"
LIBRI = DATA / "librispeech/LibriSpeech/test-clean"
SR = 16000
OWNER_DB = -23.0
rng = np.random.default_rng(20261031)

# id, 场景, 时长 s, 参数
SCEN = [
    ("c01", "one", 80, {"sir": 6, "snr": 10, "overlap": 0.2, "nb_sex": "F"}),
    ("c02", "one", 80, {"sir": 3, "snr": 10, "overlap": 0.2, "nb_sex": "M"}),
    ("c03", "multi", 80, {"sir": 9, "snr": 10, "n_nb": 3}),
    ("c04", "other_first", 100, {"sir": 6, "snr": 10, "overlap": 0.2, "first_s": 25, "nb_sex": "F"}),
    ("c05", "other_first", 120, {"sir": 6, "snr": 10, "overlap": 0.2, "first_s": 45, "nb_sex": "M"}),
    ("c06", "owner_only", 80, {"snr": 10}),
    ("c07", "one", 80, {"sir": 6, "snr": None, "overlap": 0.2, "nb_sex": "F"}),
    ("c08", "multi", 80, {"sir": 6, "snr": 10, "n_nb": 4}),
]


def rms(a):
    return float(np.sqrt(np.mean(a.astype(np.float64) ** 2)) + 1e-12)


def sha256(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def active_rms(a, thr_db=-40.0):
    """有声段 RMS：20 ms 帧里高于最响帧 thr_db 的帧。"""
    f = 320
    n = len(a) // f
    e = np.mean(a[: n * f].reshape(n, f).astype(np.float64) ** 2, axis=1)
    m = 10 * np.log10(e + 1e-12) > 10 * np.log10(e.max() + 1e-12) + thr_db
    return float(np.sqrt(e[m].mean()) + 1e-12)


def make_rir(rt60, drr_db, length_s=0.8):
    n = int(length_s * SR)
    t = np.arange(n) / SR
    tail = rng.standard_normal(n) * np.exp(-6.9 * t / rt60)
    tail[: int(0.002 * SR)] = 0  # 2 ms 预延迟
    tail /= np.sqrt(np.sum(tail ** 2))
    h = tail * 10 ** (-drr_db / 20)
    h[0] += 1.0
    return h.astype(np.float32)


RIR_NEAR = make_rir(0.15, 12.0)
RIR_FAR = make_rir(0.6, -3.0)


def libri():
    sex = {}
    for line in (LIBRI.parent / "SPEAKERS.TXT").read_text().splitlines():
        if line.startswith(";"):
            continue
        p = [s.strip() for s in line.split("|")]
        if len(p) >= 3 and p[2] == "test-clean":
            sex[p[0]] = p[1]
    trans = {}
    for t in LIBRI.rglob("*.trans.txt"):
        for line in t.read_text().splitlines():
            k, v = line.split(" ", 1)
            trans[k] = v
    by = {}
    for f in sorted(LIBRI.rglob("*.flac")):
        spk = f.name.split("-")[0]
        by.setdefault(spk, []).append({"path": f, "id": f.stem, "dur": sf.info(str(f)).duration})
    return sex, trans, by


SEX, TRANS, BY = libri()
OWNER = max(BY, key=lambda s: sum(u["dur"] for u in BY[s]))
owner_pool = [u for u in BY[OWNER] if 3.0 <= u["dur"] <= 12.0]
rng.shuffle(owner_pool)
owner_next = [0]
NB_USED = set()


def load(u):
    a, sr = sf.read(str(u["path"]), dtype="float32")
    assert sr == SR
    return a


def next_owner():
    u = owner_pool[owner_next[0] % len(owner_pool)]
    owner_next[0] += 1
    return u


def pick_neighbors(n, sex=None):
    c = [s for s in BY if s != OWNER and s not in NB_USED and (sex is None or SEX.get(s) == sex)]
    c.sort()
    sel = list(rng.choice(c, size=n, replace=False))
    NB_USED.update(sel)
    return sel


def place(track, a, start):
    s = int(start * SR)
    e = min(len(track), s + len(a))
    if e > s:
        track[s:e] += a[: e - s]
    return e / SR


def build(cid, kind, dur, p):
    n = int(dur * SR)
    own = np.zeros(n, np.float32)
    nbs = {}
    tl = {"owner": [], "interf": []}
    owner_utts = []

    def put_owner(t):
        u = next_owner()
        a = load(u)
        a = a / active_rms(a) * 10 ** (OWNER_DB / 20)
        end = place(own, a, t)
        tl["owner"].append([round(t, 3), round(min(end, dur), 3)])
        owner_utts.append({"id": u["id"], "text": TRANS.get(u["id"], "")})
        return end

    def put_nb(spk, t, level_db):
        pool = [u for u in BY[spk] if 2.0 <= u["dur"] <= 10.0]
        u = pool[int(rng.integers(len(pool)))]
        a = load(u)
        a = a / active_rms(a) * 10 ** (level_db / 20)
        nbs.setdefault(spk, np.zeros(n, np.float32))
        end = place(nbs[spk], a, t)
        tl["interf"].append([round(t, 3), round(min(end, dur), 3), spk])
        return end

    t = 0.5
    if kind in ("one", "other_first"):
        nb = pick_neighbors(1, p.get("nb_sex"))[0]
        lvl = OWNER_DB - p["sir"]
        if kind == "other_first":
            while t < p["first_s"]:
                t = put_nb(nb, t, lvl) + rng.uniform(0.3, 0.9)
            t = max(t, p["first_s"])
        turn = 0
        while t < dur - 3:
            if turn % 2 == 0:
                e = put_owner(t)
            else:
                e = put_nb(nb, t, lvl)
            # 接话：间隔 0.3–1.2 s；一定比例提前 0.5–1.5 s 插话（重叠）
            if rng.random() < p["overlap"]:
                t = e - rng.uniform(0.5, 1.5)
            else:
                t = e + rng.uniform(0.3, 1.2)
            turn += 1
        nbl = [nb]
    elif kind == "multi":
        nbl = pick_neighbors(p["n_nb"])
        lvl = OWNER_DB - p["sir"]
        # 邻桌：几个人轮流说，几乎不停（间隔 0.1–0.6 s）
        tn, k = 0.3, 0
        while tn < dur - 2:
            tn = put_nb(nbl[k % len(nbl)], tn, lvl) + rng.uniform(0.1, 0.6)
            k += 1
        # 主人：说一句、停 2–6 s
        while t < dur - 3:
            t = put_owner(t) + rng.uniform(2.0, 6.0)
    elif kind == "owner_only":
        nbl = []
        while t < dur - 3:
            t = put_owner(t) + rng.uniform(0.4, 2.0)
    own_r = fftconvolve(own, RIR_NEAR)[:n].astype(np.float32)
    interf = np.zeros(n, np.float32)
    for a in nbs.values():
        interf += fftconvolve(a, RIR_FAR)[:n].astype(np.float32)
    # 远场混响后重定电平：每位邻座的有声 RMS = 主人有声 RMS − SIR（多人时逐人定）
    if nbs:
        g = 10 ** ((OWNER_DB - p["sir"]) / 20) / np.mean([active_rms(fftconvolve(a, RIR_FAR)[:n]) for a in nbs.values()])
        interf *= g
    own_r *= 10 ** (OWNER_DB / 20) / active_rms(own_r)
    if p.get("snr") is not None:
        noise_all, nsr = sf.read(str(DATA / "demand/PCAFETER/ch01.wav"), dtype="float32")
        assert nsr == SR
        off = int(rng.integers(0, len(noise_all) - n))
        noise = noise_all[off:off + n].copy()
        noise *= 10 ** ((OWNER_DB - p["snr"]) / 20) / rms(noise)
    else:
        noise = np.zeros(n, np.float32)
    mix = own_r + interf + noise
    peak = float(np.abs(mix).max())
    if peak > 0.99:
        k = 0.99 / peak
        mix, own_r, interf, noise = mix * k, own_r * k, interf * k, noise * k
    files = {}
    for name, a in (("mix", mix), ("owner", own_r), ("interf", interf), ("noise", noise)):
        f = OUT / f"{cid}_{name}.wav"
        sf.write(str(f), a.astype(np.float32), SR, subtype="FLOAT")
        files[name] = {"file": f.name, "sha256": sha256(f)}
    tl["owner"].sort()
    tl["interf"].sort()
    return {
        "id": cid, "kind": kind, "dur_s": dur, "params": p, "owner_speaker": OWNER,
        "neighbors": [str(s) for s in nbl], "timeline": tl, "owner_utts": owner_utts, "files": files,
    }


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    clips = [build(cid, kind, dur, p) for cid, kind, dur, p in SCEN]
    meta = {
        "seed": 20261031, "sr": SR, "owner_speaker": OWNER, "owner_sex": SEX.get(OWNER),
        "owner_db": OWNER_DB, "rir": {"near": {"rt60": 0.15, "drr_db": 12}, "far": {"rt60": 0.6, "drr_db": -3}},
        "noise": "DEMAND PCAFETER ch01", "out_dir": str(OUT.relative_to(MAIN)), "clips": clips,
    }
    (HERE / "clips.json").write_text(json.dumps(meta, ensure_ascii=False, indent=1) + "\n")
    for c in clips:
        o = sum(e - s for s, e in c["timeline"]["owner"])
        i = sum(e - s for s, e, _ in c["timeline"]["interf"])
        print(c["id"], c["kind"], f"owner {o:.1f}s interf {i:.1f}s", c["neighbors"])


if __name__ == "__main__":
    main()
