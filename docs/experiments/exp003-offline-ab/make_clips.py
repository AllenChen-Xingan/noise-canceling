"""exp003 第一步：准备 12 段带噪录音（16 kHz 单声道），有干净参考的一并写出。

来源（详见 README「素材清单」）：
- 现成 3 段：resources/urgent2026_challenge_track2/assets/sp0{3,4,5}_casino_sn*.wav（带干净参考 sp0x.wav），截到 12 s
- 合成 9 段：目标语音 = LibriSpeech test-clean（英文 7 段）或 Wikimedia Commons 中文朗读（2 段）；
  干扰 = 其他 LibriSpeech 说话人（单个干扰者或 3–6 路 babble）+ DEMAND 底噪（PCAFETER / SCAFE / DKITCHEN）或公有领域钢琴曲
合成约定：
- 目标语音前面留 1.0 s、后面留 0.5 s 只有噪音的段落（给「非语音段残留电平」用，也让模型状态先稳定下来）
- SNR = 目标语音有声区间（去掉前后留白）的 RMS 对 全部干扰之和 的 RMS，单位 dB
- 底噪 bed_db：底噪相对人声干扰的电平（例如 -10 表示底噪比干扰人声低 10 dB）；纯非人声段没有人声干扰，底噪就是全部干扰
- 随机数固定种子，重跑结果相同
输出：_tmp/20261030-exp003/src/<id>_noisy.wav、<id>_clean.wav；docs/experiments/exp003-offline-ab/clips.json
用法（项目根下）：_tmp/exp002-venv/bin/python docs/experiments/exp003-offline-ab/make_clips.py
"""
import hashlib
import json
from pathlib import Path

import librosa
import numpy as np
import soundfile as sf

from _paths import MAIN, ROOT  # resources/ 只在主目录（MAIN）
HERE = Path(__file__).resolve().parent
OUT = ROOT / "_tmp/20261030-exp003/src"
DATA = Path.home() / "vdn-data"
LIBRI = DATA / "librispeech/LibriSpeech/test-clean"
SR = 16000
PRE, POST = 1.0, 0.5
rng = np.random.default_rng(20261030)


def load(path, start=None, dur=None, channel=0):
    info = sf.info(str(path))
    kw = {}
    if start is not None:
        kw["start"] = int(start * info.samplerate)
        kw["frames"] = int(dur * info.samplerate)
    a, sr = sf.read(str(path), dtype="float32", always_2d=True, **kw)
    a = a[:, channel]
    if sr != SR:
        a = librosa.resample(a, orig_sr=sr, target_sr=SR).astype(np.float32)
    return a


def rms(a):
    return float(np.sqrt(np.mean(a.astype(np.float64) ** 2)) + 1e-12)


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


# ---------- LibriSpeech 索引 ----------
def libri_index():
    spk_sex = {}
    for line in (LIBRI.parent / "SPEAKERS.TXT").read_text().splitlines():
        if line.startswith(";"):
            continue
        p = [s.strip() for s in line.split("|")]
        if len(p) >= 3 and p[2] == "test-clean":
            spk_sex[p[0]] = p[1]
    utts = []
    for f in sorted(LIBRI.rglob("*.flac")):
        spk = f.name.split("-")[0]
        utts.append({"path": f, "spk": spk, "sex": spk_sex.get(spk, "?"), "dur": sf.info(str(f)).duration})
    trans = {}
    for t in LIBRI.rglob("*.trans.txt"):
        for line in t.read_text().splitlines():
            k, v = line.split(" ", 1)
            trans[k] = v
    return utts, trans


UTTS, TRANS = libri_index()
USED_SPK = set()


def pick_target(sex):
    cands = [u for u in UTTS if u["sex"] == sex and 8.0 <= u["dur"] <= 10.0 and u["spk"] not in USED_SPK]
    u = cands[rng.integers(len(cands))]
    USED_SPK.add(u["spk"])
    return u


def talker_stream(n, exclude):
    """取一个没用过的说话人，把他的多条语句首尾相接成 n 个样本长。"""
    spks = sorted({u["spk"] for u in UTTS} - exclude - USED_SPK)
    spk = spks[rng.integers(len(spks))]
    exclude.add(spk)
    mine = [u for u in UTTS if u["spk"] == spk]
    order = rng.permutation(len(mine))
    buf, used = [], []
    total = 0
    off = int(rng.integers(0, SR * 2))  # 随机起点，避免所有干扰者同时开口
    for i in order:
        a = load(mine[i]["path"])
        buf.append(a)
        used.append(mine[i]["path"].stem)
        total += len(a)
        if total - off >= n:
            break
    s = np.concatenate(buf)[off: off + n]
    s = np.pad(s, (0, n - len(s)))
    return s / rms(s), spk, used


def bed(kind, n):
    src = {
        "PCAFETER": DATA / "demand/PCAFETER/ch01.wav",
        "SCAFE": DATA / "demand/SCAFE/ch01.wav",
        "DKITCHEN": DATA / "demand/DKITCHEN/ch01.wav",
        "MUSIC": DATA / "music/Scott_Joplin_-_Maple_Leaf_Rag.ogg",
    }[kind]
    total = sf.info(str(src)).duration
    dur = n / SR + 0.5
    start = float(rng.uniform(5.0, total - dur - 5.0))
    a = load(src, start, dur)[:n]
    return a / rms(a), {"file": str(src).replace(str(Path.home()), "~"), "start_s": round(start, 2)}


def synth(cid, desc, category, target, snr_db, n_talkers, bed_kind, bed_db, lang="en"):
    if lang == "en":
        u = target
        clean_core = load(u["path"])
        tgt_info = {"file": str(u["path"]).replace(str(Path.home()), "~"), "spk": u["spk"], "sex": u["sex"],
                    "text": TRANS[u["path"].stem]}
    else:
        path, start, dur = target
        clean_core = load(path, start, dur)
        tgt_info = {"file": str(path).replace(str(Path.home()), "~"), "start_s": start, "dur_s": dur}
    clean_core = clean_core / np.max(np.abs(clean_core)) * 0.5
    n = int((PRE + POST) * SR) + len(clean_core)
    clean = np.zeros(n, np.float32)
    clean[int(PRE * SR): int(PRE * SR) + len(clean_core)] = clean_core
    exclude = {target["spk"]} if lang == "en" else set()
    talkers, tinfo = np.zeros(n, np.float32), []
    for _ in range(n_talkers):
        s, spk, used = talker_stream(n, exclude)
        talkers += s
        tinfo.append({"spk": spk, "utts": used})
    if n_talkers:
        talkers /= rms(talkers)
    binfo = None
    if bed_kind:
        b, binfo = bed(bed_kind, n)
        interf = talkers + b * 10 ** (bed_db / 20) if n_talkers else b
    else:
        interf = talkers
    interf = interf / rms(interf) * rms(clean_core) * 10 ** (-snr_db / 20)
    noisy = clean + interf
    peak = np.max(np.abs(noisy))
    g = 0.9 / peak if peak > 0.9 else 1.0
    sf.write(OUT / f"{cid}_noisy.wav", noisy * g, SR, subtype="FLOAT")
    sf.write(OUT / f"{cid}_clean.wav", clean * g, SR, subtype="FLOAT")
    return {"id": cid, "desc": desc, "category": category, "lang": lang, "snr_db": snr_db, "has_ref": True,
            "kind": "synthetic", "dur_s": round(n / SR, 2), "speech_start_s": PRE,
            "speech_end_s": round(PRE + len(clean_core) / SR, 2),
            "target": tgt_info, "interferers": tinfo, "bed": bed_kind, "bed_db": bed_db, "bed_src": binfo}


def ready_made(cid, desc, category, noisy_rel, clean_rel, snr_db, crop=12.0):
    noisy = load(MAIN / noisy_rel)[: int(crop * SR)]
    clean = load(MAIN / clean_rel)[: int(crop * SR)]
    sf.write(OUT / f"{cid}_noisy.wav", noisy, SR, subtype="FLOAT")
    sf.write(OUT / f"{cid}_clean.wav", clean, SR, subtype="FLOAT")
    return {"id": cid, "desc": desc, "category": category, "lang": "en", "snr_db": snr_db, "has_ref": True,
            "kind": "ready-made", "dur_s": round(len(noisy) / SR, 2),
            "noisy_src": noisy_rel, "clean_src": clean_rel,
            "noisy_sha256": sha256(MAIN / noisy_rel), "clean_sha256": sha256(MAIN / clean_rel)}


def loudest(a, sec=3.0):
    """a 里能量最大的连续 sec 秒（10 ms 帧滑窗）。"""
    n, w = len(a) // 160, int(sec * SR / 160)
    e = (a[: n * 160].reshape(n, 160).astype(np.float64) ** 2).mean(1)
    st = int(np.convolve(e, np.ones(w), "valid").argmax()) * 160
    return a[st: st + int(sec * SR)]


def enroll(c):
    """VoxLock「给定样本」模式的 3 s 注册音频：尽量取同一说话人、但不在本段里出现的干净语音。
    - LibriSpeech 段：同一说话人的另一条语句（不是目标句，取最长的一条），其中最响的 3 s
    - 中文段：同一份录音里 300–310 s 这一窗（目标段在 120 s 起）中最响的 3 s
    - 现成赌场段（c01–c03）：没有同一说话人的其他录音，只能用本段干净参考里最响的 3 s——和测试内容重叠，偏乐观，README 里注明
    不用随机数，不影响上面各段的生成结果。"""
    if c["kind"] == "ready-made":
        a, info = load(MAIN / c["clean_src"]), {"from": c["clean_src"], "overlap_with_clip": True}
    elif c["lang"] == "en":
        stem = Path(c["target"]["file"]).stem
        mine = sorted((u for u in UTTS if u["spk"] == c["target"]["spk"] and u["path"].stem != stem),
                      key=lambda u: (-u["dur"], u["path"].name))
        a, info = load(mine[0]["path"]), {"from": str(mine[0]["path"]).replace(str(Path.home()), "~"), "overlap_with_clip": False}
    else:
        f = Path(c["target"]["file"].replace("~", str(Path.home()), 1))
        a, info = load(f, 300.0, 10.0), {"from": c["target"]["file"], "start_s": 300.0, "dur_s": 10.0, "overlap_with_clip": False}
    s = loudest(a)
    sf.write(OUT / f"{c['id']}_enroll.wav", s / np.max(np.abs(s)) * 0.5, SR, subtype="FLOAT")
    return info


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    U = "resources/urgent2026_challenge_track2/assets/"
    clips = [
        ready_made("c01", "赌场环境（人声 + 机器声），SNR 5 dB", "babble-多人", U + "sp03_casino_sn5.wav", U + "sp03.wav", 5),
        ready_made("c02", "赌场环境（人声 + 机器声），SNR 0 dB", "babble-多人", U + "sp04_casino_sn0.wav", U + "sp04.wav", 0),
        ready_made("c03", "赌场环境（人声 + 机器声），SNR −5 dB", "babble-多人", U + "sp05_casino_sn-5.wav", U + "sp05.wav", -5),
        synth("c04", "邻座一人说话 + 咖啡馆露台底噪，SNR 5 dB", "babble-单个干扰者", pick_target("F"), 5, 1, "SCAFE", -10),
        synth("c05", "邻座一人说话 + 咖啡馆露台底噪，SNR 0 dB", "babble-单个干扰者", pick_target("M"), 0, 1, "SCAFE", -10),
        synth("c06", "邻桌 3 人说话 + 食堂底噪，SNR 0 dB", "babble-多人", pick_target("F"), 0, 3, "PCAFETER", -6),
        synth("c07", "邻桌 6 人说话 + 咖啡馆露台底噪，SNR −5 dB", "babble-多人", pick_target("M"), -5, 6, "SCAFE", -6),
        synth("c08", "邻桌 4 人说话 + 食堂底噪，SNR 10 dB", "babble-多人", pick_target("F"), 10, 4, "PCAFETER", -6),
        synth("c09", "厨房杯碟水声（非人声），SNR 0 dB", "非人声-杯碟", pick_target("M"), 0, 0, "DKITCHEN", 0),
        synth("c10", "背景钢琴曲（拉格泰姆），SNR 5 dB", "非人声-背景音乐", pick_target("F"), 5, 0, "MUSIC", 0),
        synth("c11", "中文 · 邻桌 3 人（英文）说话 + 食堂底噪，SNR 0 dB", "babble-多人", (DATA / "zh/zh_popobing_landxia.ogg", 120.0, 9.5), 0, 3, "PCAFETER", -6, lang="zh"),
        synth("c12", "中文 · 邻座一人（英文）说话 + 咖啡馆露台底噪，SNR 5 dB", "babble-单个干扰者", (DATA / "zh/zh_braille_commusic.ogg", 120.0, 9.5), 5, 1, "SCAFE", -10, lang="zh"),
    ]
    for c in clips:
        c["enroll"] = enroll(c)
    srcs = {}
    for p in [DATA / "demand/PCAFETER/ch01.wav", DATA / "demand/SCAFE/ch01.wav", DATA / "demand/DKITCHEN/ch01.wav",
              DATA / "music/Scott_Joplin_-_Maple_Leaf_Rag.ogg", DATA / "zh/zh_popobing_landxia.ogg",
              DATA / "zh/zh_braille_commusic.ogg", DATA / "librispeech/test-clean.tar.gz"]:
        srcs[str(p).replace(str(Path.home()), "~")] = sha256(p)
    (HERE / "clips.json").write_text(json.dumps({"seed": 20261030, "sr": SR, "clips": clips, "source_sha256": srcs},
                                                ensure_ascii=False, indent=1))
    for c in clips:
        print(c["id"], c["dur_s"], c["desc"])


if __name__ == "__main__":
    main()
