"""exp013 · 造每种声音的「干轨」（还没过房间与设备链、电平未定）。

每个函数返回 (x, src)：x 长 n 个 16 kHz 样本；src 是素材出处（进清单 sounds[].src，S10⑦）。
说话人池见 pools.py：主说话人只从 OWNER 池取，旁人 / babble / 电视 / 广播只从 INTERF 池取。
"""
import numpy as np

import pools
import synth_sounds as ss
from dsp import SR, intervals


def _u(rng, r):
    return float(rng.uniform(r[0], r[1]))


def speech_fill(rng, files, n, start, end, gap):
    """把一位说话人的句子首尾相接放进 [start, end)，句间停顿 gap 秒。返回 (x, 句子 id 列表)。"""
    x = np.zeros(n)
    utts = []
    pos = start
    order = rng.permutation(len(files))
    for k in order:
        if pos >= end - SR // 2:
            break
        y = pools.utterance(files[k])
        m = min(len(y), end - pos)
        seg = y[:m].copy()
        if m < len(y):  # 截断的句子尾巴淡出 20 ms
            f = min(m, int(0.02 * SR))
            seg[-f:] *= np.linspace(1, 0, f)
        x[pos : pos + m] += seg
        utts.append(files[k].stem)
        pos += m + int(_u(rng, gap) * SR)
    return x, utts


def owner_track(rng, n, lang, lead_s, gap, spk=None):
    lead = int(lead_s * SR)
    if lang == "zh":
        y, src = pools.zh_segment(rng, n - lead)
        x = np.zeros(n)
        x[lead:] = y
        f = int(0.02 * SR)
        x[lead : lead + f] *= np.linspace(0, 1, f)
        return x, {**src, "spk": "zh:" + src["file"]}
    pool = pools.libri()["owner"]
    spk = spk or sorted(pool)[int(rng.integers(len(pool)))]
    x, utts = speech_fill(rng, pool[spk], n, lead, n, gap)
    return x, {"kind": "librispeech", "spk": spk, "utts": utts}


def co_target_track(rng, n, gap, exclude, lead_s):
    pool = pools.libri()["owner"]
    cands = [s for s in sorted(pool) if s not in exclude]
    spk = cands[int(rng.integers(len(cands)))]
    start = int(_u(rng, (lead_s, 4.0)) * SR)  # 开头留空对所有要留的人都成立（codex 第 3 轮 P2 #4）
    x, utts = speech_fill(rng, pool[spk], n, start, n, gap)
    return x, {"kind": "librispeech", "spk": spk, "utts": utts}


def _interf_spk(rng, exclude=()):
    pool = pools.libri()["interf"]
    cands = [s for s in sorted(pool) if s not in exclude]
    s = cands[int(rng.integers(len(cands)))]
    return s, pool[s]


def talker_track(rng, n, gap, exclude=()):
    spk, files = _interf_spk(rng, exclude)
    start = int(_u(rng, (0.5, 5.0)) * SR)
    end = min(n, start + int(_u(rng, (4.0, 10.0)) * SR))
    x, utts = speech_fill(rng, files, n, start, end, gap)
    return x, {"kind": "librispeech", "spk": spk, "utts": utts}


def tv_track(rng, n):
    spk, files = _interf_spk(rng)
    x, utts = speech_fill(rng, files, n, 0, n, (0.1, 0.4))
    src = {"kind": "librispeech", "spk": spk, "utts": utts}
    if rng.random() < 0.4:  # 节目里垫着音乐
        m, msrc = pools.music_segment(rng, n)
        x = x + 0.3 * m / (np.std(m) + 1e-12) * np.std(x)
        src["music"] = msrc
    y, p = ss.speaker(x, rng, "tv")
    return y, {**src, **p}


def nav_track(rng, n):
    spk, files = _interf_spk(rng)
    start = int(_u(rng, (1.0, 7.0)) * SR)
    x, utts = speech_fill(rng, files[:], n, start, min(n, start + int(_u(rng, (2.0, 4.0)) * SR)), (0.3, 0.6))
    y, p = ss.speaker(x, rng, "nav")
    return y, {"kind": "librispeech", "spk": spk, "utts": utts, **p}


def announce_track(rng, n):
    """广播：先一段提示音、再人声。返回 (人声, src, 提示音轨)；提示音不是语音，compose 另记成 pa-chime。"""
    spk, files = _interf_spk(rng)
    chime = ss.pa_chime(rng)
    t0 = int(_u(rng, (0.5, 4.0)) * SR)
    c = np.zeros(n)
    c[t0 : t0 + len(chime)] = chime[: n - t0]
    s = t0 + len(chime) + int(0.3 * SR)
    sp, utts = speech_fill(rng, files, n, s, min(n, s + int(_u(rng, (3.0, 6.0)) * SR)), (0.3, 0.6))
    y, p = ss.speaker(sp, rng, "announce")
    return y, {"kind": "librispeech", "spk": spk, "utts": utts, **p}, (c, {"kind": "synth", "fn": "pa_chime", "at": t0})


def muffled_track(rng, n, gap):
    x, src = talker_track(rng, n, gap)
    y, p = ss.speaker(x, rng, "muffled")
    return y, {**src, **p}


def babble_tracks(rng, n, k_range):
    """k 个旁人各自连续说话（互相重叠），返回 [(x, src)]，由 compose 各自加房间后相加。"""
    k = int(rng.integers(k_range[0], k_range[1] + 1))
    out, used = [], []
    pool = pools.libri()["interf"]
    for _ in range(k):
        spk = sorted(pool)[int(rng.integers(len(pool)))]
        x, utts = speech_fill(rng, pool[spk], n, int(_u(rng, (0, 1.0)) * SR), n, (0.1, 0.5))
        out.append((x, {"kind": "librispeech", "spk": spk, "utts": utts}))
        used.append(spk)
    return out


def continuous_track(rng, source, n):
    kind = source["kind"]
    if kind == "synth":
        x, p = ss.CONTINUOUS[source["fn"]](rng, n)
        return x, {"kind": "synth", "fn": source["fn"], "params": p}
    if kind == "demand":
        env = source["envs"][int(rng.integers(len(source["envs"])))]
        return pools.demand_segment(rng, env, n)
    if kind == "music":
        return pools.music_segment(rng, n)
    raise ValueError(f"连续声不支持 {kind}")


def _onsets(rng, n, count, length, owner_dry):
    """一半放在主说话人停顿里、一半随机（README 第一节〔建议值〕）。"""
    iv = intervals(owner_dry, guard=0)
    pauses, prev = [], 0
    for s, e in iv + [[n, n]]:
        if s - prev > int(0.3 * SR):
            pauses.append((prev, s))
        prev = e
    out = []
    for i in range(count):
        if i % 2 == 0 and pauses:
            a, b = pauses[int(rng.integers(len(pauses)))]
            hi = max(a + 1, b - min(length, b - a))
            out.append(int(rng.integers(a, hi)))
        else:
            out.append(int(rng.integers(0, max(1, n - length))))
    return out


def event_track(rng, source, n, count, owner_dry):
    x = np.zeros(n)
    occ = []
    clips = []
    for _ in range(count):
        if source["kind"] == "esc50":
            y, src = pools.esc50_clip(rng, source["category"])
        else:
            y, p = ss.EVENTS[source["fn"]](rng)
            src = {"kind": "synth", "fn": source["fn"], "params": p}
        clips.append((y, src))
    for (y, src), t in zip(clips, _onsets(rng, n, count, max(len(c[0]) for c in clips), owner_dry)):
        m = min(len(y), n - t)
        x[t : t + m] += y[:m]
        occ.append({**src, "at": int(t)})
    return x, {"kind": "events", "items": occ}
