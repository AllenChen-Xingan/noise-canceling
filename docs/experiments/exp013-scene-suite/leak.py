"""exp013 · 防泄漏检查：本集用到的素材与 exp006 训练素材求交集（016 S10）。

三类都按两条路查（任一条命中即算交集）：
1. 说话人：exp006 训练池（test-clean 排序后偶数位，按规则算）∪ exp006 训练清单里记下的 target_spk
2. DEMAND 时段：同环境同通道的时间区间与 exp006 训练段重叠；另按规则：exp006 用过的环境，起点必须 ≥ 总长 60%
3. 合成函数：本集 src 里的函数名不许出现在 exp006 `sim.py` 的取材 / 合成函数名单里
纯标准库 + pools，不读音频内容。
"""
import json
import re
from pathlib import Path

import soundfile as sf

import pools

HERE = Path(__file__).resolve().parent
EXP006 = HERE.parent / "exp006-scene-sim"
DEMAND_RE = re.compile(r"DEMAND/(\w+)/ch(\d+)@([\d.]+)s")


def exp006_train():
    rows = []
    for f in sorted((EXP006 / "builds").glob("*/truth.jsonl")):
        rows += [r for r in map(json.loads, f.read_text().splitlines()) if r.get("split") == "train"]
    spk = {r["target_spk"] for r in rows if r.get("target_spk")} | pools.libri()["exp006_train"]
    ivs = {}
    for r in rows:
        m = DEMAND_RE.search(r.get("noise") or "")
        if m:
            t = float(m.group(3))
            ivs.setdefault((m.group(1), int(m.group(2))), []).append((t, t + float(r.get("dur_s", 10.0))))
    fns = set(re.findall(r"^def (synth_\w+|file_music|speech_track|demand|talkers|noise_for)\(", (EXP006 / "sim.py").read_text(), re.M))
    return {"speakers": spk, "demand": ivs, "fns": fns, "rows": len(rows)}


def used(records):
    """从清单记录里收集用过的素材（也用来写 exclude.json）。"""
    spk, dem, fns, esc, other = set(), [], set(), set(), []

    def walk(src):
        if not isinstance(src, dict):
            return
        k = src.get("kind")
        if k == "librispeech":
            spk.add(src["spk"])
        elif k == "demand":
            dem.append((src["env"], src["ch"], src["start"], src["len"], src["sr"]))
        elif k == "synth":
            fns.add(src["fn"])
        elif k == "esc50":
            esc.add(src["file"])
        elif k in ("commons-zh", "music"):
            other.append((src["file"], src["start"], src["len"]))
        for key in ("talkers", "items"):
            for s in src.get(key, []):
                walk(s)
        if "music" in src and isinstance(src["music"], dict):
            walk(src["music"])

    for r in records:
        for t in r["targets"]:
            walk(t["src"])
        for s in r["sounds"]:
            walk(s["src"])
    return {"speakers": spk, "demand": dem, "fns": fns, "esc50": esc, "other": other}


def check(records):
    tr = exp006_train()
    u = used(records)
    hits = []
    hits += [f"说话人 {s}" for s in sorted(u["speakers"] & tr["speakers"])]
    frames = {}
    for env, ch, start, ln, sr in u["demand"]:
        a, b = start / sr, (start + ln) / sr
        for ta, tb in tr["demand"].get((env, ch), []):
            if a < tb and ta < b:
                hits.append(f"DEMAND {env}/ch{ch:02d} {a:.1f}–{b:.1f}s 与训练段 {ta:.1f}–{tb:.1f}s 重叠")
        if env in pools.EXP006_DEMAND:
            key = (env, ch)
            if key not in frames:
                frames[key] = sf.info(str(pools.DEMAND / env / f"ch{ch:02d}.wav")).frames
            if start < pools.EXP006_TRAIN_FRAC * frames[key]:
                hits.append(f"DEMAND {env}/ch{ch:02d} 起点 {a:.1f}s 落在前 60% 时段")
    hits += [f"合成函数 {f}" for f in sorted(u["fns"] & tr["fns"])]
    return {
        "against": "exp006 全部 build 的 train 段", "train_rows": tr["rows"],
        "speakers": len(u["speakers"] & tr["speakers"]), "demand": sum(1 for h in hits if h.startswith("DEMAND")),
        "synth_fn": len(u["fns"] & tr["fns"]), "hits": hits[:50], "ok": not hits,
    }


def exclude_table(records):
    """本集用过的素材（裁判原料，永不训练；016 S10 第 3 条）。"""
    u = used(records)
    return {
        "_说明": "exp013 评测用过的素材。任何训练配方取材前都要先减掉这张表（016 S10 第 3 条）",
        "librispeech_speakers": sorted(u["speakers"]),
        "demand_segments": [{"env": e, "ch": c, "start": s, "len": n, "sr": sr} for e, c, s, n, sr in sorted(set(u["demand"]))],
        "esc50_files": sorted(u["esc50"]),
        "other_segments": [{"file": f, "start": s, "len": n} for f, s, n in sorted(set(u["other"]))],
    }
