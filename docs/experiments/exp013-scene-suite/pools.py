"""exp013 · 素材索引与切池（防泄漏的切法见 README 第四节）。

- LibriSpeech test-clean：40 人按编号字符串排序，exp006 训练用偶数位（ids[0::2]）。本集只用奇数位 20 人，
  再分两池：OWNER（奇数位里的偶数位，10 人）只当主说话人；INTERF（其余 10 人）只当旁人 / babble / 电视 / 广播
- DEMAND：exp006 用过的 6 个环境只取每路后 40% 时段（起点 ≥ 0.6 × 总长）；没被合成用过的环境只取后一半（前一半是测试份，TEST_SPLIT_FRAC）
- ESC-50：只用 fetch.py 下载登记过的文件（materials.json），按类别分
- 中文：只用 exp003 登记过许可的两段 Commons 朗读（CC BY-SA 4.0）
"""
import hashlib
import json
from functools import lru_cache
from pathlib import Path

import numpy as np
import soundfile as sf

from dsp import SR, resample

HERE = Path(__file__).resolve().parent
DATA = Path.home() / "vdn-data"
LIBRI = DATA / "librispeech/LibriSpeech/test-clean"
DEMAND = DATA / "demand"
ESC = DATA / "esc50"
MUSIC = DATA / "music/Scott_Joplin_-_Maple_Leaf_Rag.ogg"
ZH = [DATA / "zh/zh_popobing_landxia.ogg", DATA / "zh/zh_braille_commusic.ogg"]
ZH_LICENSE = "CC-BY-SA-4.0"



def _exp006_rules():
    """exp006 的切分规则从它自己的 recipes.json 读（正本），不手抄：用过哪些 DEMAND 环境、train 占前多少比例。"""
    r = json.loads((HERE.parent / "exp006-scene-sim/recipes.json").read_text())
    envs = set()
    for srcs in r["noise_sources"].values():
        for s in srcs:
            if s.get("kind") == "demand":
                envs.add(s["env"])
            envs |= {b for b in s.get("bed", []) if not b.startswith("synth")}
    return envs, float(r["demand_split"]["train_frac"])


EXP006_DEMAND, EXP006_TRAIN_FRAC = _exp006_rules()


@lru_cache(maxsize=1)
def libri():
    spk = {}
    for f in sorted(LIBRI.rglob("*.flac")):
        spk.setdefault(f.name.split("-")[0], []).append(f)
    ids = sorted(spk)
    exp006_train = ids[0::2]
    ours = ids[1::2]
    return {
        "owner": {s: spk[s] for s in ours[0::2]},
        "interf": {s: spk[s] for s in ours[1::2]},
        "exp006_train": set(exp006_train),
    }


def read_mono(path, start=0, frames=-1):
    a, sr = sf.read(str(path), start=start, frames=frames, dtype="float32", always_2d=True)
    return resample(a[:, 0].astype(np.float64), sr), sr


def utterance(path):
    x, _ = read_mono(path)
    return x


# 从没被合成用过的 DEMAND 环境：按时段切（016 S10①「无说话人的按录音场次 / 时段」）——前一半留作测试份
# （只测、不当任何合成原料），后一半当裁判原料。大总管 2026-11-03 定（016 第 45 行由大总管改为指向 S10①）
TEST_SPLIT_FRAC = 0.5


def demand_window(env):
    """返回 (可用起点下限比例, 1.0)：exp006 用过的环境只用后 40%；其余环境只用后一半（前一半是测试份）。"""
    return (EXP006_TRAIN_FRAC, 1.0) if env in EXP006_DEMAND else (TEST_SPLIT_FRAC, 1.0)


def demand_segment(rng, env, n):
    """随机通道、随机起点（在允许的时段里）取 n 个 16 kHz 样本。"""
    ch = int(rng.integers(1, 17))
    path = DEMAND / env / f"ch{ch:02d}.wav"
    info = sf.info(str(path))
    need = int(np.ceil(n * info.samplerate / SR)) + 64
    lo_f, hi_f = demand_window(env)
    lo, hi = int(info.frames * lo_f), int(info.frames * hi_f) - need
    if hi <= lo:
        raise ValueError(f"{env} 时段不够 {n} 个样本")
    start = int(rng.integers(lo, hi))
    x, sr = read_mono(path, start, need)
    src = {"kind": "demand", "env": env, "ch": ch, "start": start, "len": need, "sr": int(sr)}
    return x[:n], src


def demand_available(env):
    return (DEMAND / env / "ch01.wav").exists()


@lru_cache(maxsize=1)
def esc50():
    rec = json.loads((HERE / "materials.json").read_text())["files"]
    out = {}
    for key, v in rec.items():
        if key.startswith("esc50/") and (DATA / key).exists():
            out.setdefault(v["cls"], []).append(key)
    return {k: sorted(v) for k, v in out.items()}


def esc50_clip(rng, category):
    files = esc50().get(category)
    if not files:
        raise FileNotFoundError(f"ESC-50 类别 {category} 没有已下载的文件（先跑 fetch.py）")
    key = files[int(rng.integers(len(files)))]
    x, _ = read_mono(DATA / key)
    x, end = trim_tail(x)
    return x, {"kind": "esc50", "file": key.split("/", 1)[1], "trim_end": end}


def trim_tail(x, rel_db=-30.0, keep_ms=60, fade_ms=20):
    """截掉原录音的混响尾巴（reverb-plan.md 第二节）：最后一个能量 > 最大帧 rel_db 的 20 ms 帧之后只留 keep_ms，再淡出。
    片段内部的原录音混响去不掉，是已知限制。"""
    f = 320
    n = len(x) // f
    e = np.mean(x[: n * f].reshape(n, f) ** 2, axis=1) if n else np.zeros(0)
    if not e.size or e.max() <= 0:
        return x, len(x)
    last = int(np.flatnonzero(e > e.max() * 10 ** (rel_db / 10))[-1])
    end = min(len(x), (last + 1) * f + int(keep_ms * 1e-3 * SR))
    y = x[:end].copy()
    k = min(end, int(fade_ms * 1e-3 * SR))
    y[end - k :] *= np.linspace(1, 0, k)
    return y, end


@lru_cache(maxsize=4)
def _long(path):
    x, _ = read_mono(path)
    return x


def zh_segment(rng, n):
    i = int(rng.integers(len(ZH)))
    x = _long(ZH[i])
    s = int(rng.integers(0, len(x) - n))
    return x[s : s + n], {"kind": "commons-zh", "file": ZH[i].name, "start": s, "len": n}


def music_segment(rng, n):
    x = _long(MUSIC)
    s = int(rng.integers(0, len(x) - n))
    return x[s : s + n], {"kind": "music", "file": MUSIC.name, "start": s, "len": n}


def index_hash():
    """素材索引哈希：所有可能被取用的文件的相对路径 + 字节数（不读内容，秒级）。"""
    L = libri()
    files = [f for pool in ("owner", "interf") for fs in L[pool].values() for f in fs]
    files += sorted(DEMAND.glob("*/ch*.wav")) + [DATA / k for v in esc50().values() for k in v] + ZH + [MUSIC]
    rir = DATA / "rir"  # 实测 RIR、同房间噪声、McVAMPIRE 几何（codex 第 2 轮 P2 #4）
    files += sorted(rir.glob("ace-single/Single/*/*/*.wav")) + sorted(rir.glob("mit-ir-survey/Audio/*.wav"))
    files += sorted(rir.glob("mcvampire/ir/*.wav")) + sorted(rir.glob("mcvampire/driving_noise/*.wav")) + [rir / "mcvampire/geometry.mat"]
    rows = sorted(f"{Path(f).relative_to(DATA)}\t{Path(f).stat().st_size}" for f in files)
    return hashlib.sha256("\n".join(rows).encode()).hexdigest(), len(rows)
