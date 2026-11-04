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


def exp006_rows(splits=("train",)):
    rows = []
    for f in sorted((EXP006 / "builds").glob("*/truth.jsonl")):
        rows += [r for r in map(json.loads, f.read_text().splitlines()) if r.get("split") in splits]
    return rows


def exp006_train():
    rows = exp006_rows(("train",))
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
    spk, dem, fns, esc, other, rir, room_noise = set(), [], set(), set(), [], set(), []

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
        elif k == "rir":
            rir.add(f"{src['dataset']}/{src['file']}#ch{src.get('ch', 0)}")
        elif k in ("ace-noise", "mcv-noise"):
            room_noise.append((src["kind"], src["file"], src.get("ch", 0), src["start"], src["len"], src["sr"]))
        elif k not in ("babble", "events"):
            raise ValueError(f"leak.used 不认识素材类型 {k!r}：先在这里登记它怎么查，再用")
        for key in ("talkers", "items"):
            for s in src.get(key, []):
                walk(s)
        for key in ("music", "rir"):
            if isinstance(src.get(key), dict):
                walk(src[key])

    for r in records:
        for t in r["targets"]:
            walk(t["src"])
        for s in r["sounds"]:
            walk(s["src"])
    return {"speakers": spk, "demand": dem, "fns": fns, "esc50": esc, "other": other, "rir": rir, "room_noise": room_noise}


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
        key = (env, ch)
        if key not in frames:
            frames[key] = sf.info(str(pools.DEMAND / env / f"ch{ch:02d}.wav")).frames
        if env in pools.EXP006_DEMAND and start < pools.EXP006_TRAIN_FRAC * frames[key]:
            hits.append(f"DEMAND {env}/ch{ch:02d} 起点 {a:.1f}s 落在前 60% 时段（exp006 训练）")
        if env not in pools.EXP006_DEMAND and start < pools.TEST_SPLIT_FRAC * frames[key]:
            hits.append(f"DEMAND {env}/ch{ch:02d} 起点 {a:.1f}s 落在测试份（前 {pools.TEST_SPLIT_FRAC:.0%}）")
    hits += [f"合成函数 {f}" for f in sorted(u["fns"] & tr["fns"])]
    return {
        "against": "exp006 全部 build 的 train 段", "train_rows": tr["rows"],
        "speakers": len(u["speakers"] & tr["speakers"]), "demand": sum(1 for h in hits if h.startswith("DEMAND")),
        "synth_fn": len(u["fns"] & tr["fns"]), "hits": hits[:50], "ok": not hits,
        "synth_fn_note": "按函数名比；真正的保证是本目录代码不 import exp006 sim.py（tests 里 test_no_exp006_import）",
        "exp006_eval_overlap": eval_overlap(u),
    }


def eval_overlap(u):
    """只报告、不判不过：与 exp006 test / seq 段（评测份，曾用来选场景分类器的滞回参数）的重叠。
    本集与它同池（LibriSpeech 奇数位、DEMAND 后 40%），所以 exp006 场景分类器不能在本集上评（README 第四节）。"""
    rows = exp006_rows(("test", "seq"))
    spk = {r["target_spk"] for r in rows if r.get("target_spk")}
    seg_s = float(json.loads((EXP006 / "recipes.json").read_text())["seq"]["segment_s"])
    ivs = {}
    for r in rows:
        noise = r.get("noise") or ""
        listed = isinstance(noise, list)  # seq 段的 noise 是列表（每小段一条，每小段 seg_s 秒）
        for item in (noise if listed else [noise]):
            for ids in re.findall(r"talkersx\d+\(([\d,]+)\)", item):  # 邻座说话人也算
                spk |= set(ids.split(","))
            m = DEMAND_RE.search(item)
            if m:
                t = float(m.group(3))
                ivs.setdefault((m.group(1), int(m.group(2))), []).append((t, t + (seg_s if listed else float(r.get("dur_s", 10.0)))))
    dem = sum(1 for env, ch, st, ln, sr in u["demand"]
              if any(st / sr < tb and ta < (st + ln) / sr for ta, tb in ivs.get((env, ch), [])))
    return {"rows": len(rows), "speakers": len(u["speakers"] & spk), "demand_segments": dem}


def exclude_table(records):
    """本集用过的素材（裁判原料，永不训练；016 S10 第 3 条）。"""
    u = used(records)
    return {
        "_说明": "exp013 评测用过的素材。任何训练配方取材前都要先减掉这张表（016 S10 第 3 条）",
        "librispeech_speakers": sorted(u["speakers"]),
        "demand_segments": [{"env": e, "ch": c, "start": s, "len": n, "sr": sr} for e, c, s, n, sr in sorted(set(u["demand"]))],
        "esc50_files": sorted(u["esc50"]),
        "synth_fns": sorted(u["fns"]),
        "rir_files": sorted(u["rir"]),
        "rooms_reserved": {"_说明": "按房间整体保留给评测：训练不只要避开用过的段，同一房间 / 麦位的其他 IR 与噪声也不用（自审 P2）",
                           "ace": sorted({f.split("/")[1] + "/" + f.split("/")[2] for f in u["rir"] if f.startswith("ace/")}),
                           "mcvampire": "整个数据集" if any(f.startswith("mcvampire/") for f in u["rir"]) else None},
        "room_noise_segments": [{"kind": k, "file": f, "ch": c, "start": s, "len": n, "sr": sr}
                                for k, f, c, s, n, sr in sorted(set(u["room_noise"]))],
        "other_segments": [{"file": f, "start": s, "len": n, "sr": 16000, "_说明": "重采样到 16 kHz 后的样本号"}
                           for f, s, n in sorted(set(u["other"]))],
    }
