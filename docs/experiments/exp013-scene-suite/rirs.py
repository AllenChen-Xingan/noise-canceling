"""exp013 · 实测房间冲激响应（第三版起，「一段 = 一个房间」，reverb-plan.md）。

三个来源（fetch.py --only rir；许可见 README 第二节）：
- ACE 单声道包：7 个房间 × 2 个麦位，每个麦位有 RIR + 同麦位录的环境声 / 风扇 / 多人说话 → **成对**
- McVAMPIRE：小面包车 8 个座位 × 11 个朝向 × 14 个车顶麦，6 个车载喇叭，14 档车速行驶噪声 → **成对**
- MIT IR Survey：271 处各 1 条 IR，没有噪声 → **非成对**，底噪按 T60 区间配（scenes.json `bed_t60`）

一段只选一个 Room；这段里所有点声源都从它取 RIR（ACE / MIT：同一条 IR，按距离只调直达部分幅度；
McVAMPIRE：按座位取各自实测 IR，同一个车顶麦）。直达窗长取 device_chain.json（022 第 2 步）。
"""
import json
import re
from dataclasses import dataclass, field
from functools import lru_cache
from pathlib import Path

import numpy as np
import scipy.io
import soundfile as sf

from device_chain import P as CP
from dsp import SR, resample, schroeder_t60
from rooms import Rir

HERE = Path(__file__).resolve().parent
RIR = Path.home() / "vdn-data/rir"
ACE = RIR / "ace-single/Single"
MCV = RIR / "mcvampire"
MIT = RIR / "mit-ir-survey/Audio"

# ACE 技术报告 arXiv 1606.03365 表 4「5 通道十字阵」一列的声源—麦距离（m）。单声道包用的是哪一个麦、
# 与十字阵中心差多少，报告里没逐字写，按十字阵中心取【推断】
ACE_DIST = {
    ("Office_1", 1): 1.25, ("Office_1", 2): 2.68, ("Office_2", 1): 1.09, ("Office_2", 2): 2.75,
    ("Meeting_Room_1", 1): 1.55, ("Meeting_Room_1", 2): 2.67, ("Meeting_Room_2", 1): 1.68, ("Meeting_Room_2", 2): 2.67,
    ("Lecture_Room_1", 1): 0.847, ("Lecture_Room_1", 2): 2.59, ("Lecture_Room_2", 1): 2.89, ("Lecture_Room_2", 2): 3.44,
    ("Building_Lobby", 1): 1.58, ("Building_Lobby", 2): 2.97,
}
MIT_DIST = 1.5  # MIT IR Survey 的声源—录音机距离：论文写约 1.5 m【推断，未核原文】
DIRECT_GAIN_DB = (-12.0, 18.0)  # 按距离调直达部分时的上下限，防止离谱外推【建议值】


@dataclass
class Measured:
    """一条实测 IR（16 kHz，直达峰幅度归一为 1）。"""
    dataset: str
    room: str
    file: str
    ch: int
    ir: np.ndarray
    dist_m: float  # 实测时的声源—麦距离
    peak: int = 0
    t60: float = 0.0
    drr: float = 0.0

    def __post_init__(self):
        # 直达声 = 第一个到达的强峰，不是全段最大值：车厢等小空间里早期反射可能比直达还强（MIT 列车 IR 实测如此）
        a = np.abs(self.ir)
        first = int(np.flatnonzero(a >= 0.5 * a.max())[0])
        self.peak = first + int(np.argmax(a[first : first + int(0.001 * SR) + 1]))
        self.ir = self.ir / self.ir[self.peak]
        w = int(round(CP["step2_direction"]["direct_window_ms"] * 1e-3 * SR))
        lo, hi = max(0, self.peak - w), self.peak + w + 1
        d = np.zeros_like(self.ir)
        d[lo:hi] = self.ir[lo:hi]
        self._direct, self._rest = d, self.ir - d
        self.t60 = schroeder_t60(self._rest[self.peak :])
        er = float(np.sum(self._rest**2))
        self.drr = 10 * np.log10(float(np.sum(d**2)) / er) if er > 0 else 99.0

    @property
    def valid(self):
        """可用的条件（三条都要满足）：①T60 估计不超过 IR 长度的 2.5 倍（MIT 的 IR 多被截到约 0.5 s，再长的估计不可信）；
        ②T60 估计失败（返回 0）时，只有直达明显占优（DRR ≥ 15 dB，真的几乎没混响）才算「短混响」，否则不用。"""
        if self.t60 <= 0:
            return self.drr >= 15.0
        return self.t60 <= 2.5 * len(self.ir) / SR and self.tail_end_db <= -30.0

    @property
    def tail_end_db(self):
        """③IR 结尾 50 ms 比混响开头 50 ms 低多少 dB。高于 −30 dB = 衰减没完就被截断（MIT 约一成如此），不用。"""
        r = self._rest[self.peak :]
        w = int(0.05 * SR)
        if len(r) < 3 * w:
            return 0.0 if np.any(r) else -120.0
        return float(10 * np.log10(np.mean(r[-w:] ** 2) / max(np.mean(r[:w] ** 2), 1e-20) + 1e-30))

    def at(self, dist, az):
        """把直达部分按 实测距离 / 目标距离 调幅度（反平方律），尾巴不动；返回 rooms.Rir。"""
        g_db = float(np.clip(20 * np.log10(self.dist_m / max(dist, 0.05)), *DIRECT_GAIN_DB))
        r = Rir(self._direct * 10 ** (g_db / 20), self._rest.copy(), self.t60, float(dist), float(az), (), self.peak)
        return r, {"kind": "rir", "dataset": self.dataset, "room": self.room, "file": self.file, "ch": self.ch,
                   "t60_s": round(self.t60, 3), "drr_meas_db": round(self.drr, 2), "direct_gain_db": round(g_db, 2)}


def _read(path, ch=0):
    x, sr = sf.read(str(path), always_2d=True, dtype="float32")
    return resample(x[:, ch].astype(np.float64), sr), sr


@lru_cache(maxsize=None)
def ace(room, cfg):
    code = next((ACE / room / str(cfg)).glob("*_RIR.wav"))
    x, _ = _read(code)
    return Measured("ace", room, f"{room}/{cfg}/{code.name}", 0, x, ACE_DIST[(room, cfg)])


@lru_cache(maxsize=None)
def mit_all():
    out = []
    for f in sorted(MIT.glob("*.wav")):
        x, _ = _read(f)
        m = Measured("mit", re.sub(r"^h\d+_", "", f.stem).rsplit("_", 1)[0], f.name, 0, x, MIT_DIST)
        if m.valid:
            out.append(m)
    return out


@lru_cache(maxsize=1)
def mcv_geometry():
    g = scipy.io.loadmat(str(MCV / "geometry.mat"), simplify_cells=True)
    mic = np.array([m["pos"] for m in g["mic"]], float)
    mouth = np.array([m["centerpos"] for m in g["mouth"]], float)
    ls = np.array([m.get("pos", m.get("centerpos")) for m in g["ls"]], float)
    return mic, mouth, ls


@lru_cache(maxsize=None)
def mcv(name, ch, dist):
    x, _ = _read(MCV / "ir" / name, ch)
    return Measured("mcvampire", "minivan", name, ch, x, dist)


@dataclass
class Room:
    """一段用的房间。kind：ace / mcv / mit。"""
    kind: str
    rid: str
    paired: bool
    t60: float
    info: dict = field(default_factory=dict)

    # ---- 点声源 ----
    def rir(self, rng, role, dist, az):
        if self.kind == "mcv":
            return self._mcv_rir(rng, role, az)
        m = self.info["ir"]
        return m.at(dist, az)

    def _mcv_rir(self, rng, role, az):
        mic, mouth, ls = mcv_geometry()
        ch = self.info["mic"]
        seat = self.info["seat"]
        if role == "owner":
            name, pos = f"ir_mouth_pos{seat}_0.wav", mouth[seat - 1]
        elif role == "nav" or role == "dash":  # 导航语音、转向灯：前排车载喇叭
            k = int(rng.integers(1, 3))
            name, pos = f"ir_ls{k}.wav", ls[k - 1]
        else:  # 旁人说话、咳嗽等：别的座位，随机朝向
            others = [s for s in range(1, 9) if s != seat]
            s = 1 if role == "talker" and seat != 1 else others[int(rng.integers(len(others)))]
            o = [-90, -60, -45, -30, -15, 0, 15, 30, 45, 60, 90][int(rng.integers(11))]
            name, pos = f"ir_mouth_pos{s}_{o}.wav", mouth[s - 1]
        d = float(np.linalg.norm(pos - mic[ch]))
        m = mcv(name, ch, d)
        r, info = m.at(d, az)  # 实测几何距离，直达不调（增益 0 dB）
        return r, info

    # ---- 同房间底噪 ----
    def noise(self, rng, n):
        if self.kind == "ace":
            room, cfg = self.info["room"], self.info["cfg"]
            t = self.info["noise_types"][int(rng.integers(len(self.info["noise_types"])))]
            f = next((ACE / room / str(cfg)).glob(f"*_Noise_{t}.wav"))
            return _segment(rng, f, 0, n, {"kind": "ace-noise", "room": room, "cfg": cfg, "type": t})
        if self.kind == "mcv":
            sp = self.info["speed_kph"]
            f = MCV / "driving_noise" / f"{sp}kph.wav"
            return _segment(rng, f, self.info["mic"], n, {"kind": "mcv-noise", "speed_kph": sp, "ch": self.info["mic"]})
        raise ValueError("非成对房间没有同房间底噪")

    def manifest(self):
        return {"room_id": self.rid, "dataset": self.kind, "paired": self.paired, "room_t60_s": round(self.t60, 3),
                **{k: v for k, v in self.info.items() if k in ("room", "cfg", "seat", "mic", "speed_kph", "bed_cls")}}


def _segment(rng, f, ch, n, src):
    info = sf.info(str(f))
    need = int(np.ceil(n * info.samplerate / SR)) + 64
    start = int(rng.integers(0, info.frames - need))
    x, sr = sf.read(str(f), start=start, frames=need, always_2d=True, dtype="float32")
    y = resample(x[:, ch].astype(np.float64), sr)[:n]
    return y, {**src, "file": f.name, "start": start, "len": need, "sr": int(sr)}


def bed_ok(bed_t60, bed_cls, t60):
    """非成对：RIR 的 T60 是否和底噪名义区间对得上（reverb-plan.md 第三节；×[0.8, 1.25] 暂定）。返回 (过, 说明)。"""
    spec = bed_t60.get(bed_cls)
    if spec is None:
        return False, f"{bed_cls} 没有名义 T60（缺值判红）"
    if spec == "agnostic":
        return True, "合成扩散底噪，与房间无关"
    lo, hi = spec
    tol = bed_t60["_tolerance"]
    if hi < bed_t60["_short_s"]:
        return t60 <= bed_t60["_short_max_s"], f"短混响类：RIR T60 {t60:.2f} s ≤ {bed_t60['_short_max_s']} s"
    ok = lo * tol[0] <= t60 <= hi * tol[1]
    return ok, f"RIR T60 {t60:.2f} s 对 {bed_cls} 名义 {lo}–{hi} s ×{tol}"


def candidates(opt, bed_t60):
    """一个房间选项能展开成哪些 (Room, 底噪类别)。"""
    if "ace" in opt:
        out = []
        for cfg in opt.get("cfg", [1, 2]):
            m = ace(opt["ace"], cfg)
            out.append((Room("ace", f"ace/{opt['ace']}/{cfg}", True, m.t60,
                             {"room": opt["ace"], "cfg": cfg, "ir": m, "noise_types": opt["noise"], "bed_cls": "bed-ace"}), "bed-ace"))
        return out
    if "mcv" in opt:
        return [(None, "bed-car-mcv")]  # 车内在 choose 时再抽座位与车速
    out = []
    for m in mit_all():
        if m.room in opt["mit"]:
            for b in opt["bed"]:
                if bed_ok(bed_t60, b, m.t60)[0]:
                    out.append((Room("mit", f"mit/{m.file}", False, m.t60, {"ir": m, "bed_cls": b}), b))
    return out


def choose(rng, S, tier, bed_t60):
    """按 scenes.json 的 rooms 选项给一段抽房间。返回 Room（info 里带 bed_cls）。"""
    opts = [o for o in S["rooms"] if tier in o.get("tiers", ["common", "stress"])]
    o = opts[int(rng.integers(len(opts)))]
    if "mcv" in o:
        mic, mouth, _ = mcv_geometry()
        seat = int(o["seats"][int(rng.integers(len(o["seats"])))])
        ch = int(np.argmin(np.linalg.norm(mic - mouth[seat - 1], axis=1)))  # 离这个座位最近的车顶麦（0.3–0.5 m）
        lo, hi = o["speed_kph"][tier]
        speeds = [s for s in [0] + list(range(30, 151, 10)) if lo <= s <= hi]
        sp = speeds[int(rng.integers(len(speeds)))]
        t60 = mcv(f"ir_mouth_pos{seat}_0.wav", ch, 0.4).t60
        return Room("mcv", f"mcv/seat{seat}/mic{ch + 1}", True, t60, {"seat": seat, "mic": ch, "speed_kph": sp, "bed_cls": "bed-car-mcv"})
    cands = candidates(o, bed_t60)
    if not cands:
        raise ValueError(f"房间选项 {json.dumps(o, ensure_ascii=False)} 在 {tier} 档没有一条 T60 对得上的 IR")
    room, _ = cands[int(rng.integers(len(cands)))]
    return room
