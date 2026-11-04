"""exp013 · 片段清单：拼每段一行的记录，并校验字段。

字段正本：scorecard 在 vdn-eval 里定的 Rust 类型，导出到 `specs/schema/eval-scene-clip.schema.json`（clip-manifest.md v1）。
那份 schema 还没导出时，按 clip-manifest.md 第一节表格的字段名校验（测试比对本文件 FIELDS 与那张表）。
"""
import json
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
SCHEMA = ROOT / "specs/schema/eval-scene-clip.schema.json"
DOC = HERE / "clip-manifest.md"

FIELDS = [
    "id", "scene", "tier", "device", "mic_mode", "pair", "lang", "polarity", "dur_samples", "sr", "audio",
    "has_clean_ref", "additive", "targets", "sounds", "nominal", "measured", "owner_iv", "interf_iv", "chain", "seed", "hash",
]
AUDIO_KEYS = ["mix", "owner", "owner_dry", "stems", "floor", "noise", "interf"]
SOUND_FIELDS = ["inst", "cls", "group", "is_speech", "iv", "level_dbfs", "rel_db", "dist_m", "azimuth_deg", "src", "license"]


def doc_fields(section):
    """从 clip-manifest.md 某一节的表格第一列取出反引号里的字段名。"""
    text = DOC.read_text()
    m = re.search(rf"^## {section}.*?(?=^## )", text, re.S | re.M)
    names = []
    for line in m.group(0).splitlines():
        if line.startswith("| `"):
            first = line.split("|")[1]
            names += re.findall(r"`([a-z_]+)`", first)
    return names


def clip_id(scene, tier, idx, device):
    return f"{scene}-{tier[0]}{idx:02d}-{'mba' if device == 'mba-builtin' else 'ext'}"


def record(meta, lab, audio_paths, hashes, n, sr, other_device):
    s, t, i = meta["scene"], meta["tier"], meta["idx"]
    chain = {k: (round(v, 4) if isinstance(v, float) else v) for k, v in meta["chain"].items()}
    nominal = {k: (None if v is None else round(float(v), 3)) for k, v in meta["nominal"].items()}
    return {
        "id": clip_id(s, t, i, meta["device"]),
        "scene": s, "tier": t, "device": meta["device"], "mic_mode": meta["chain"]["mic_mode"],
        "pair": clip_id(s, t, i, other_device), "lang": meta["lang"], "polarity": meta["polarity"],
        "dur_samples": int(n), "sr": int(sr), "audio": audio_paths, "has_clean_ref": True, "additive": bool(meta["additive"]),
        "targets": lab["targets"], "sounds": lab["sounds"], "nominal": nominal, "measured": lab["measured"],
        "owner_iv": lab["owner_iv"], "interf_iv": lab["interf_iv"], "chain": chain, "seed": int(meta["seed"]), "hash": hashes,
    }


# 已请 scorecard 加进 Rust 类型、schema 还没有的字段（2026-11-03 发出）。schema 有了就自动按 schema 查，这里的豁免不再生效
PENDING = {"Target": {"src"}, "Measured": {"snr_all_db", "owner_level_dbfs"}, "Sound": {"direction_db"}}
TYPES = {"string": str, "integer": int, "boolean": bool, "object": dict, "array": list, "number": (int, float)}


def schema_path():
    """正本合并进本仓后读本仓；合并前读旁边 scorecard worktree 导出的那份。"""
    if SCHEMA.exists():
        return SCHEMA
    side = ROOT.parent / "noise_canceling-scorecard" / "specs/schema" / SCHEMA.name
    return side if side.exists() else None


def _type_ok(val, ts):
    for t in ts:
        if t == "null" and val is None:
            return True
        if t in TYPES and isinstance(val, TYPES[t]) and not (t in ("integer", "number") and isinstance(val, bool)):
            return True
    return False


def _walk(node, val, path, errs, defs, name=None):
    """不依赖 jsonschema 包的递归校验：$ref · type · enum · required · properties · additionalProperties · items · min/maxItems · minimum。"""
    if "$ref" in node:
        ref = node["$ref"].split("/")[-1]
        return _walk(defs[ref], val, path, errs, defs, ref)
    if "type" in node and not _type_ok(val, node["type"] if isinstance(node["type"], list) else [node["type"]]):
        errs.append(f"{path} 类型应为 {node['type']}")
        return
    if "enum" in node and val not in node["enum"]:
        errs.append(f"{path} = {val!r} 不在枚举里")
    if "minimum" in node and isinstance(val, (int, float)) and val < node["minimum"]:
        errs.append(f"{path} 小于 {node['minimum']}")
    if isinstance(val, dict):
        props = node.get("properties", {})
        errs += [f"{path}.{k} 缺" for k in node.get("required", []) if k not in val]
        extra = node.get("additionalProperties", True)
        for k, v in val.items():
            if k in props:
                _walk(props[k], v, f"{path}.{k}", errs, defs)
            elif extra is False:
                if k not in PENDING.get(name, set()):
                    errs.append(f"{path}.{k} 多出")
            elif isinstance(extra, dict):
                _walk(extra, v, f"{path}.{k}", errs, defs)
    if isinstance(val, list):
        if "minItems" in node and len(val) < node["minItems"] or "maxItems" in node and len(val) > node["maxItems"]:
            errs.append(f"{path} 长度 {len(val)} 不合")
        if "items" in node:
            for i, v in enumerate(val):
                _walk(node["items"], v, f"{path}[{i}]", errs, defs)


def extra_checks(rec):
    """schema 管不到的四条（scorecard 2026-11-03）。"""
    errs = []
    insts = [s["inst"] for s in rec["sounds"]]
    if sorted(insts) != sorted(rec["audio"]["stems"]):
        errs.append("sounds 的 inst 与 audio.stems 不一一对应")
    for h in ("samples", "wav_sha256"):
        if sorted(rec["hash"][h]["stems"]) != sorted(rec["audio"]["stems"]):
            errs.append(f"hash.{h}.stems 的键与 audio.stems 不一致")
    n = rec["dur_samples"]
    ivs = [("owner_iv", rec["owner_iv"]), ("interf_iv", rec["interf_iv"])]
    ivs += [(f"targets[{i}].iv", t["iv"]) for i, t in enumerate(rec["targets"])]
    ivs += [(f"sounds[{s['inst']}].iv", s["iv"]) for s in rec["sounds"]]
    for name, iv in ivs:
        if any(not (0 <= a < b <= n) for a, b in iv):
            errs.append(f"{name} 有区间不满足 起 < 止 ≤ dur_samples")
    if rec["sr"] != 16000:
        errs.append("sr 不是 16000")
    if rec["polarity"] == "suppress" and len(rec["targets"]) != 1:
        errs.append("suppress 段 targets 不是恰好 1 个")
    return errs


def validate(rec):
    """返回错误列表（空 = 通过）。有 schema 按 schema + extra_checks；没有就按 FIELDS（clip-manifest.md）。"""
    sp = schema_path()
    if sp:
        schema = json.loads(sp.read_text())
        errs = []
        _walk(schema, rec, "clip", errs, schema.get("$defs", {}), "SceneClip")
        return errs + extra_checks(rec)
    errs = [f"缺字段 {k}" for k in FIELDS if k not in rec] + [f"多出字段 {k}" for k in rec if k not in FIELDS]
    errs += [f"audio 缺 {k}" for k in AUDIO_KEYS if k not in rec.get("audio", {})]
    for s in rec.get("sounds", []):
        errs += [f"sounds[{s.get('inst')}] 缺 {k}" for k in SOUND_FIELDS if k not in s]
    return errs + (extra_checks(rec) if not errs else [])
