"""exp013 · 逐段标注与实测值。

- 区间规则（016 §4）：从**混响 + 设备后**的分轨算，20 ms 帧能量 > 全段最大帧 −35 dB，两端各外扩 2 帧
- 实测值用 ITU-T P.56 活动电平（dsp.p56_level_db），与生成时设增益的 gated 电平是两个算法（016 S2③）
- 实测 SNR = P.56(owner) − 稳态干扰（底噪 + 有规律 + babble + 设备底噪）整段功率；snr_all_db 另把突发声也算进去
- 实测 SIR = P.56(owner) − P.56(听得清的旁人语音之和)
"""
import json
from pathlib import Path

import numpy as np

from dsp import gated_level_db, intervals, iv_len, iv_overlap, iv_union, p56_level_db, schroeder_t60
from rooms import drr_db

HERE = Path(__file__).resolve().parent
SOUNDS = json.loads((HERE / "sounds.json").read_text())["classes"]


def is_steady(cls):
    c = SOUNDS[cls]
    return c["group"] in ("bed", "pattern") or (c["group"] == "speech" and not c["is_speech"])


def power_db(x):
    return 10 * np.log10(np.mean(np.asarray(x, float) ** 2) + 1e-20)


def measure(owner, stems, floor, cls_of):
    """从各轨算实测 SNR / SIR（只依赖音频与类别，不依赖生成时的任何电平）。"""
    lo = p56_level_db(owner)
    n = len(owner)
    steady = sum((stems[k] for k in stems if is_steady(cls_of[k])), np.zeros(n)) + floor
    nonspeech = sum((stems[k] for k in stems if not SOUNDS[cls_of[k]]["is_speech"]), np.zeros(n)) + floor
    speech = [stems[k] for k in stems if SOUNDS[cls_of[k]]["is_speech"]]
    out = {"snr_db": lo - power_db(steady), "snr_all_db": lo - power_db(nonspeech), "sir_db": None}
    if speech:
        out["sir_db"] = lo - p56_level_db(sum(speech))
    return out


def label(arrays, meta):
    """返回清单里与标注 / 实测有关的字段。"""
    owner = arrays["owner"]
    stems = arrays["stems"]
    cls_of = {s["inst"]: s["cls"] for s in meta["sounds"]}
    lo_gated = gated_level_db(owner)
    owner_iv = intervals(owner)
    sounds = []
    speech_ivs = []
    for s in meta["sounds"]:
        x = stems[s["inst"]]
        c = SOUNDS[s["cls"]]
        iv = intervals(x)
        lvl = gated_level_db(x)
        if c["is_speech"]:
            speech_ivs.append(iv)
        sounds.append({
            "inst": s["inst"], "cls": s["cls"], "group": c["group"], "is_speech": c["is_speech"], "iv": iv,
            "level_dbfs": round(lvl, 2), "rel_db": round(lvl - lo_gated, 2),
            "dist_m": None if s["dist_m"] is None else round(s["dist_m"], 3),
            "azimuth_deg": None if s["azimuth_deg"] is None else round(s["azimuth_deg"], 1),
            "src": s["src"], "license": c["license"],
            **({"direction_db": {k: round(v, 2) for k, v in s["att"].items()}} if s.get("att") else {}),
        })
    targets = []
    for t in meta["targets"]:
        targets.append({"spk": t["spk"], "dist_m": round(t["dist_m"], 3), "azimuth_deg": round(t["azimuth_deg"], 1),
                        "level_dbfs": round(gated_level_db(t["x"]), 2), "iv": intervals(t["x"]), "src": t["src"]})
    interf_iv = iv_union(*speech_ivs) if speech_ivs else []
    m = measure(owner, stems, arrays["floor"], cls_of)
    r = meta["owner_rir"]
    m.update({
        # T60 在混响部分上量（不含直达窗）：近距离时直达声会把 Schroeder 曲线前段拉陡
        "t60_s": round(schroeder_t60(r.rest), 3) if r.t60 >= 0.03 else 0.0, "drr_db": round(drr_db(r), 2),
        "overlap_ratio": round(iv_overlap(owner_iv, interf_iv) / max(iv_len(owner_iv), 1), 4),
        "owner_level_dbfs": round(p56_level_db(owner), 2),
    })
    m = {k: (None if v is None else round(float(v), 3)) for k, v in m.items()}
    return {"targets": targets, "sounds": sounds, "owner_iv": owner_iv, "interf_iv": interf_iv, "measured": m}
