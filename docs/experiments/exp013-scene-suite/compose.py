"""exp013 · 组一段：按 scenes.json 抽声音 → 造干轨 → 房间 + 设备链 → 设电平 → 相加。

随机数分两路，让 `mba-builtin` 与 `external-mic` 两个版本用完全相同的素材（配对）：
- rng_mat = default_rng([seed, 0])：抽声音、素材、房间、时间位置
- rng_dev = default_rng([seed, 1])：设备链参数（Chain 第一个抽的是主说话人电平，两版相同）
电平定义（README 第三节）：
- SNR = 主说话人活动电平（gated）− 稳态干扰（底噪 + 有规律 + babble + 设备底噪）整段功率；突发声不算进 SNR，单独按 rel_db 设
- SIR = 主说话人活动电平 − 听得清的旁人语音之和的活动电平
都在设备链第 1–3 步之后、第 6 步整段增益之前设定；整段增益不改变比值。
"""
import hashlib
import json
from pathlib import Path

import numpy as np
from scipy import signal

import pools
import tracks
from device_chain import Chain
from dsp import SR, gated_level_db
from rooms import make_rir

HERE = Path(__file__).resolve().parent
SCENES = json.loads((HERE / "scenes.json").read_text())
SOUNDS = json.loads((HERE / "sounds.json").read_text())["classes"]
STEADY_GROUPS = ("bed", "pattern")
HEADROOM = 10 ** (-1 / 20)  # −1 dBFS


def _u(rng, r):
    return float(rng.uniform(r[0], r[1]))


def _tiered(v, tier):
    return v[tier] if isinstance(v, dict) else v


def clip_seed(scene, tier, idx):
    h = hashlib.sha256(f"{SCENES['base_seed']}|{scene}|{tier}|{idx}".encode()).hexdigest()
    return int(h[:12], 16)


def resolve_cls(cls):
    """素材缺失时按 sounds.json 的 fallback 换类别（只有下载类底噪有 fallback）。"""
    c = SOUNDS[cls]
    src = c["source"]
    if src["kind"] == "demand" and not all(pools.demand_available(e) for e in src["envs"]):
        if "fallback" not in src:
            raise FileNotFoundError(f"{cls} 缺素材且没有 fallback")
        return resolve_cls(src["fallback"])
    return cls


def pick_sounds(rng, S, tier):
    t = 0 if tier == "common" else 1
    bed = resolve_cls(S["bed"][int(rng.integers(len(S["bed"])))])
    chosen = [e for e in S["sounds"] if rng.random() < e["p"][t]]
    if tier == "stress" and len(chosen) < SCENES["stress_min_extras"]:
        rest = sorted((e for e in S["sounds"] if e not in chosen and e["p"][t] > 0), key=lambda e: -e["p"][t])
        chosen += rest[: SCENES["stress_min_extras"] - len(chosen)]
    return bed, chosen


class Scene:
    """一段里的房间与设备链，以及「把一条干轨变成链后分轨」的办法。"""

    def __init__(self, rng_mat, rng_dev, S, tier, device):
        R = S["room"]
        self.dims = tuple(_u(rng_mat, d) for d in R["dims"])
        self.t60 = _u(rng_mat, R["t60"][tier])
        self.ch = Chain(rng_dev, device, tier)
        self.rng_mat, self.rng_dev = rng_mat, rng_dev

    def point(self, x, dist, az, src_h=1.2, continuous=False):
        """点声源：房间（第 1 步）+ 方位（第 2 步）+ 频响（第 3 步）。返回 (分轨, rir, 方位衰减)。"""
        rir = make_rir(self.rng_mat, self.dims, self.t60, dist, az, src_h)
        h, pad, att = self.ch.source_rir(rir, self.rng_dev)
        n = len(x)
        L = len(h) if continuous else 0  # 连续声循环补一段前奏，开头就是稳态混响
        pre = np.tile(x, L // n + 1)[-L:] if L else np.zeros(0)
        y = signal.fftconvolve(np.concatenate([pre, x]), h)[L + pad : L + pad + n]
        return self.ch.response(y), rir, att

    def diffuse(self, x, dist=None):
        """扩散声：可选先加房间（babble、广播、隔墙声），只做扩散场衰减 + 频响。"""
        if dist is not None:
            rir = make_rir(self.rng_mat, self.dims, self.t60, dist, _u(self.rng_mat, (-180, 180)))
            x = signal.fftconvolve(x, rir.full)[: len(x)]
        return self.ch.response(self.ch.diffuse(x))


def _scale_to(x, target_db, level_fn):
    cur = level_fn(x)
    return x * 10 ** ((target_db - cur) / 20) if cur > -200 else x


def _power_db(x):
    return 10 * np.log10(np.mean(np.asarray(x, float) ** 2) + 1e-20)


def make_clip(scene, tier, idx, device):
    """返回 dict：arrays（各轨）+ meta（清单字段的原料）。"""
    S = SCENES["scenes"][scene]
    seed = clip_seed(scene, tier, idx)
    rng_mat, rng_dev = np.random.default_rng([seed, 0]), np.random.default_rng([seed, 1])
    n = int(SCENES["clip_s"] * SR)
    gap = SCENES["gap_s"]
    lang = "zh" if idx in SCENES["zh_clips"][tier] else "en"
    sc = Scene(rng_mat, rng_dev, S, tier, device)
    if "owner_level_dbfs" in S:  # 地点自带电平分布（如图书馆压低声音），覆盖第 6 步默认；用 rng_mat 保证两设备相同
        sc.ch.params["owner_level_dbfs"] = _u(rng_mat, S["owner_level_dbfs"])
    from device_chain import P as CP

    # ---- 主说话人与要留的人 ----
    owner_dist = _u(rng_mat, CP["step1_room"]["owner_dist_m"]["range"])
    x_dry, osrc = tracks.owner_track(rng_mat, n, lang, SCENES["lead_silence_s"], gap)
    owner, orir, _ = sc.point(x_dry, owner_dist, 0.0)
    L_o = gated_level_db(owner)
    L_owner = L_o  # 主说话人自己的电平；「全留」时 L_o 会改成所有人之和，owner_dry 只含主说话人
    targets = [{"spk": osrc["spk"], "dist_m": orir.dist_m, "azimuth_deg": 0.0, "src": osrc, "x": owner}]
    polarity = S.get("polarity", "suppress")
    if polarity == "preserve_all":
        co = S["co_targets"]
        k = int(rng_mat.integers(co["count"][tier][0], co["count"][tier][1] + 1))
        for _ in range(k):
            xd, src = tracks.co_target_track(rng_mat, n, gap, {t["spk"] for t in targets})
            y, r, _ = sc.point(xd, _u(rng_mat, co["dist_m"]), _u(rng_mat, co["az_deg"]))
            y = _scale_to(y, L_o + _u(rng_mat, co["rel_db"]), gated_level_db)
            targets.append({"spk": src["spk"], "dist_m": r.dist_m, "azimuth_deg": r.azimuth_deg, "src": src, "x": y})
        L_o = gated_level_db(sum(t["x"] for t in targets))  # 「全留」时参照 = 所有要留的人之和

    # ---- 干扰声 ----
    bed_cls, chosen = pick_sounds(rng_mat, S, tier)
    sounds = []  # 每项：inst, cls, x（链后、未定电平）, dist, az, src, att

    def add(cls, x, src, dist=None, az=None, att=None):
        k = sum(1 for s in sounds if s["cls"] == cls)
        sounds.append({"inst": f"{cls}{k}", "cls": cls, "x": x, "dist_m": dist, "azimuth_deg": az, "src": src, "att": att})

    xb, sb = tracks.continuous_track(rng_mat, SOUNDS[bed_cls]["source"], n)
    add(bed_cls, sc.diffuse(xb), sb)
    for e in chosen:
        cls = resolve_cls(e["cls"])
        c = SOUNDS[cls]
        src_def = c["source"]
        dist = _u(rng_mat, _tiered(e.get("dist_m", SCENES["default_dist_m"]), tier))
        az = _u(rng_mat, e.get("az_deg", SCENES["default_az_deg"]))
        if src_def["kind"] == "babble":
            parts = tracks.babble_tracks(rng_mat, n, src_def["talkers"])
            y = sum(sc.diffuse(x, dist=_u(rng_mat, (3.0, 8.0))) for x, _ in parts)
            add(cls, y, {"kind": "babble", "talkers": [s for _, s in parts]})
            continue
        if src_def["kind"] == "speech":
            style = src_def["style"]
            if style == "talker":
                x, s = tracks.talker_track(rng_mat, n, gap)
            elif style == "tv":
                x, s = tracks.tv_track(rng_mat, n)
            elif style == "nav":
                x, s = tracks.nav_track(rng_mat, n)
            elif style == "announce":
                x, s, (xc, sc_src) = tracks.announce_track(rng_mat, n)
                add("pa-chime", sc.diffuse(xc, dist=dist), sc_src)
            else:
                x, s = tracks.muffled_track(rng_mat, n, gap)
        elif c["group"] == "event":
            cnt = int(rng_mat.integers(SCENES["event_count"][0], SCENES["event_count"][1] + 1))
            x, s = tracks.event_track(rng_mat, src_def, n, cnt, x_dry)
        else:
            x, s = tracks.continuous_track(rng_mat, src_def, n)
        if c["directional"]:
            y, r, att = sc.point(x, dist, az, continuous=c["group"] in STEADY_GROUPS)
            add(cls, y, s, r.dist_m, r.azimuth_deg, att)
        elif c.get("bypass_direction"):  # 麦口独立噪声（风噪）：不经波束的扩散场衰减，只过频响
            add(cls, sc.ch.response(x), s)
        else:
            y = sc.diffuse(x, dist=dist if src_def["kind"] == "speech" else None)
            add(cls, y, s)

    # ---- 设电平 ----
    nominal = {"owner_dist_m": orir.dist_m, "t60_s": sc.t60, "snr_db": _u(rng_mat, S["snr_db"][tier]), "sir_db": None,
               "owner_level_dbfs": sc.ch.params["owner_level_dbfs"]}
    steady = [s for s in sounds if SOUNDS[s["cls"]]["group"] in STEADY_GROUPS or (SOUNDS[s["cls"]]["group"] == "speech" and not SOUNDS[s["cls"]]["is_speech"])]
    speech = [s for s in sounds if SOUNDS[s["cls"]]["is_speech"]]
    events = [s for s in sounds if SOUNDS[s["cls"]]["group"] == "event"]
    bed = steady[0]
    for s in steady[1:]:
        s["rel_bed_db"] = _u(rng_mat, SCENES["pattern_rel_db"])
        s["x"] = _scale_to(s["x"], _power_db(bed["x"]) + s["rel_bed_db"], _power_db)  # 两边都按整段功率（间歇声如空调启停也一致）
    floor = sc.ch.floor(n, L_o, rng_dev)
    p_target = 10 ** ((L_o - nominal["snr_db"]) / 10)
    p_floor = float(np.mean(floor**2))
    floor_clamped = p_floor > 0.5 * p_target
    if floor_clamped:  # 底噪比名义 SNR 允许的总噪声还响：压到一半（记进清单）
        floor *= np.sqrt(0.5 * p_target / p_floor)
        p_floor = 0.5 * p_target
    st_sum = sum(s["x"] for s in steady)
    g = np.sqrt((p_target - p_floor) / max(np.mean(st_sum**2), 1e-20))
    for s in steady:
        s["x"] = s["x"] * g
    for s in events:
        s["rel_db"] = _u(rng_mat, SCENES["event_rel_db"][tier])
        s["x"] = _scale_to(s["x"], L_o + s["rel_db"], gated_level_db)
    if speech:
        nominal["sir_db"] = _u(rng_mat, S["sir_db"][tier])
        for s in speech[1:]:
            s["x"] = _scale_to(s["x"], gated_level_db(speech[0]["x"]) + _u(rng_mat, SCENES["speech_rel_db"]), gated_level_db)
        G = sum(s["x"] for s in speech)
        k = 10 ** ((L_o - nominal["sir_db"] - gated_level_db(G)) / 20)
        for s in speech:
            s["x"] = s["x"] * k

    # ---- 第 6–8 步：整段增益、削波 ----
    gain = 10 ** (sc.ch.level_gain_db(L_o) / 20)
    for t in targets:
        t["x"] = t["x"] * gain
    for s in sounds:
        s["x"] = s["x"] * gain
    floor = floor * gain
    owner_ref = sum(t["x"] for t in targets)
    noise = sum(s["x"] for s in sounds if not SOUNDS[s["cls"]]["is_speech"]) + floor
    interf = sum((s["x"] for s in speech), np.zeros(n))
    mix_lin = owner_ref + noise + interf
    # 余量：整段峰值超过 −1 dBFS 就整段调低（真机会削波；有意的削波只走第 8 步），调了多少记进清单
    peak = float(np.max(np.abs(mix_lin)))
    head = min(1.0, HEADROOM / max(peak, 1e-12))
    if head < 1.0:
        owner_ref, noise, interf, mix_lin, floor = (v * head for v in (owner_ref, noise, interf, mix_lin, floor))
        for t in targets:
            t["x"] = t["x"] * head
        for s in sounds:
            s["x"] = s["x"] * head
    mix, clipped = sc.ch.clip(mix_lin)
    dry_gain = gain * 10 ** ((L_owner - gated_level_db(x_dry)) / 20)  # 干轨与链后主说话人同电平
    # 干轨按直达声到达时刻后移，和 owner / mix 对齐（最小相位 FIR 的几个样本延迟不补）
    dry = np.concatenate([np.zeros(orir.delay), x_dry])[:n] * dry_gain * head
    arrays = {"mix": mix, "owner": owner_ref, "owner_dry": dry, "floor": floor, "noise": noise, "interf": interf,
              "stems": {s["inst"]: s["x"] for s in sounds}}
    meta = {"scene": scene, "tier": tier, "idx": idx, "device": device, "seed": seed, "lang": lang, "polarity": polarity,
            "nominal": nominal, "targets": targets, "sounds": sounds, "chain": {**sc.ch.params, "floor_clamped": bool(floor_clamped),
            "room_dims_m": list(sc.dims), "headroom_db": round(20 * np.log10(head), 3), "owner_dry_delay_samples": int(orir.delay)}, "additive": not clipped, "owner_rir": orir,
            "bed": bed_cls}
    return arrays, meta
