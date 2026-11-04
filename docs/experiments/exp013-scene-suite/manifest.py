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


def _check_schema(rec, schema):
    """不依赖 jsonschema 包的最小校验：必填字段、不许多余字段（schema 里 additionalProperties 为 false 时）、顶层类型。"""
    errs = []
    props = schema.get("properties", {})
    for k in schema.get("required", []):
        if k not in rec:
            errs.append(f"缺字段 {k}")
    if schema.get("additionalProperties") is False:
        errs += [f"多出字段 {k}" for k in rec if k not in props]
    tmap = {"string": str, "integer": int, "boolean": bool, "object": dict, "array": list, "number": (int, float)}
    for k, p in props.items():
        if k in rec and rec[k] is not None and "type" in p:
            ts = p["type"] if isinstance(p["type"], list) else [p["type"]]
            if not any(t == "null" and rec[k] is None or (t in tmap and isinstance(rec[k], tmap[t])) for t in ts):
                errs.append(f"{k} 类型应为 {ts}")
    return errs


def validate(rec):
    """返回错误列表（空 = 通过）。有 schema 按 schema，没有按 FIELDS。"""
    if SCHEMA.exists():
        return _check_schema(rec, json.loads(SCHEMA.read_text()))
    errs = [f"缺字段 {k}" for k in FIELDS if k not in rec] + [f"多出字段 {k}" for k in rec if k not in FIELDS]
    errs += [f"audio 缺 {k}" for k in AUDIO_KEYS if k not in rec.get("audio", {})]
    for s in rec.get("sounds", []):
        errs += [f"sounds[{s.get('inst')}] 缺 {k}" for k in SOUND_FIELDS if k not in s]
    if set(rec.get("hash", {}).get("wav_sha256", {})) != set(rec.get("audio", {})):
        errs.append("hash.wav_sha256 的键与 audio 不一致")
    return errs
