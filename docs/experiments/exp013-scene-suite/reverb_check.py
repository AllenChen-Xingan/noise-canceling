"""exp013 · 混响一致性检查（reverb-plan.md 第三节）：同一段里人声与底噪是不是「同一个房间」。

不做盲估：稳态底噪估不出 T60，语音盲估误差约 0.25 s（ACE 挑战赛报告）。改成只读清单比对：
1. 每个点声源（主说话人、要留的人、有方位的干扰、带房间的扩散声）都必须带 `src.rir`，且来自这段的房间
2. 成对房间（ACE / McVAMPIRE）：底噪必须是同一房间（同麦位 / 同车顶麦）录的噪声
3. 非成对房间（MIT）：所有 RIR 是同一条；RIR 的 T60 落在底噪名义区间 ×[0.8, 1.25]（暂定），缺名义值判红
返回错误列表，空 = 通过。build.py 生成后逐段跑，不过就停。
"""
import json
from pathlib import Path

import rirs

HERE = Path(__file__).resolve().parent
SCENES = json.loads((HERE / "scenes.json").read_text())
SOUNDS = json.loads((HERE / "sounds.json").read_text())["classes"]


def _rirs_of(rec):
    """(谁, rir 出处) 列表：主说话人、要留的人、每个干扰及其内部（babble 每个人、广播提示音）。"""
    out = [(f"targets[{i}]", t["src"].get("rir")) for i, t in enumerate(rec["targets"])]
    for s in rec["sounds"]:
        c = SOUNDS[s["cls"]]
        if s["src"].get("kind") == "babble":
            out += [(f"{s['inst']}.talkers[{j}]", t.get("rir")) for j, t in enumerate(s["src"]["talkers"])]
        elif c["directional"] or c["source"]["kind"] in ("speech",) or "rir" in s["src"]:
            # 按类别判「应当有房间」：有方位的点声源、所有语音（含扩散的广播、隔墙声），不看清单里有没有
            out.append((s["inst"], s["src"].get("rir")))
        if s["cls"] == "pa-chime":
            out.append((s["inst"], s["src"].get("rir")))
    return out


def check(rec, bed_t60=None):
    bed_t60 = bed_t60 or SCENES["bed_t60"]
    room = rec["chain"].get("room")
    if not room:
        return ["chain.room 缺失：不知道这段用的是哪个房间"]
    errs = []
    used = _rirs_of(rec)
    for who, r in used:
        if not r:
            errs.append(f"{who} 没有 rir 出处")
    used = [(w, r) for w, r in used if r]
    bed = next((s for s in rec["sounds"] if s["cls"] == room["bed_cls"]), None)
    if bed is None:
        errs.append(f"找不到底噪 {room['bed_cls']}")
    ds = room["dataset"]
    if ds == "ace":
        for who, r in used:
            if r["dataset"] != "ace" or not r["file"].startswith(f"{room['room']}/{room['cfg']}/"):
                errs.append(f"{who} 的 RIR {r['dataset']}:{r['file']} 不在房间 {room['room_id']}")
        if bed and not (bed["src"].get("kind") == "ace-noise" and bed["src"]["room"] == room["room"] and bed["src"]["cfg"] == room["cfg"]):
            errs.append(f"底噪不是 {room['room_id']} 同麦位录的：{bed['src']}")
    elif ds == "mcv":
        for who, r in used:
            if r["dataset"] != "mcvampire" or r["ch"] != room["mic"]:
                errs.append(f"{who} 的 RIR 不是同一个车顶麦（要 ch {room['mic']}，是 {r['dataset']} ch {r.get('ch')}）")
        if bed and not (bed["src"].get("kind") == "mcv-noise" and bed["src"]["ch"] == room["mic"]
                        and bed["src"]["speed_kph"] == room["speed_kph"]):
            errs.append(f"车内底噪不是同一个车顶麦 / 同一车速录的：{bed['src']}")
        own = rec["targets"][0]["src"].get("rir") or {}
        if own.get("file") != f"ir_mouth_pos{room['seat']}_0.wav":
            errs.append(f"主说话人没用本座位（{room['seat']}）正前方的 IR：{own.get('file')}")
    elif ds == "mit":
        files = {r["file"] for _, r in used}
        want = room["room_id"].split("/", 1)[1]
        if files - {want}:
            errs.append(f"非成对房间里混了别的 IR：{sorted(files - {want})}")
        for _, r in used[:1]:
            ok, why = rirs.bed_ok(bed_t60, room["bed_cls"], r["t60_s"])
            if not ok:
                errs.append(f"混响对不上：{why}")
    else:
        errs.append(f"不认识的房间来源 {ds}")
    return errs
