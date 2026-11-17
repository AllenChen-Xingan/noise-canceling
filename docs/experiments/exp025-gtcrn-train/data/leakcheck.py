"""exp025 · 训练 build 和 exp013 评测素材求交集（027 第六节隔离规则第 1、4 条）：不为空即红，该 build 作废。

正本：exp013 正式 build `20261103230756-17f5` 的 `exclude.json`，它由 exp013 `leak.exclude_table` 生成。
这里只读它、不手抄：exclude.json 里每个键都必须有一条对应的检查（CHECKS），多出不认识的键直接红——
exp013 以后在 leak.py 里加了新素材类，这里会先红，逼着补检查。
查的范围是「训练 · 开发」两份能取到的全部原料（不只是某次抽到的），所以现场混音怎么抽都不会漏。
另查「终测原料池」也不许碰评测素材（终测场景要和开发评测独立，027 第五节）。
"""
import json
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
EXP013 = HERE.parents[1] / "exp013-scene-suite"
EXCLUDE_BUILD = "20261103230756-17f5"
EXCLUDE = EXP013 / "builds" / EXCLUDE_BUILD / "exclude.json"
GUARDED = ("train", "dev", "final")  # d63 测试份本来就不进任何合成原料，不在训练 build 里


def load_exclude(path=EXCLUDE):
    return json.loads(Path(path).read_text())


def pool_rows(splits):
    """splits = {来源名: 切分表行列表}；返回落在 GUARDED 份里的 (来源, 行, 份, 段界或 None)。"""
    for src, rows in splits.items():
        for r in rows:
            if "segments" in r:
                for k, seg in r["segments"].items():
                    if k in GUARDED:
                        yield src, r, k, seg
            elif r.get("split") in GUARDED:
                yield src, r, r["split"], None


def chk_speakers(ex, splits):
    bad = set(ex["librispeech_speakers"])
    return [f"说话人 {r['unit']}（{src} {k}）是 exp013 评测说话人"
            for src, r, k, _ in pool_rows(splits) if src.startswith("librispeech") and r["unit"] in bad]


DEMAND_RE = re.compile(r"demand/(\w+)/ch(\d+)\.wav$")


def chk_demand(ex, splits):
    segs = {}
    for s in ex["demand_segments"]:
        segs.setdefault((s["env"], int(s["ch"])), []).append((s["start"] / s["sr"], (s["start"] + s["len"]) / s["sr"]))
    hits = []
    for src, r, k, seg in pool_rows(splits):
        m = DEMAND_RE.search(r["path"])
        if not m:
            continue
        if seg is None:
            hits.append(f"{r['path']} 整个文件进了 {k}：DEMAND 只许按时段用")
            continue
        a, b = seg[0] / r["sr"], seg[1] / r["sr"]
        for ea, eb in segs.get((m.group(1), int(m.group(2))), []):
            if a < eb and ea < b:
                hits.append(f"DEMAND {m.group(1)}/ch{m.group(2)} {k} 段 {a:.1f}–{b:.1f}s 与评测段 {ea:.1f}–{eb:.1f}s 重叠")
    return hits


def _basenames(splits):
    return {Path(r["path"]).name.lower(): (src, r["path"]) for src, r, _, _ in pool_rows(splits)}


def chk_files(key, prefix_of):
    def chk(ex, splits):
        names = _basenames(splits)
        hits = []
        for item in ex[key]:
            nm = Path(prefix_of(item)).name.lower()
            if nm in names:
                hits.append(f"{key} {nm} 出现在训练原料 {names[nm][1]}")
        return hits
    return chk


def chk_datasets(ex, splits):
    """exp013 独占的数据集（实测 RIR 三套 · ESC-50 · Commons 中文 · 音乐）整个不许进训练原料：按路径里的目录名查。"""
    hits = []
    for src, r, k, _ in pool_rows(splits):
        parts = {p.lower() for p in Path(r["path"]).parts}
        for d in ("ace-single", "ace-data", "mit-ir-survey", "mcvampire", "esc50", "zh", "music"):
            if d in parts:
                hits.append(f"{r['path']}（{src} {k}）来自 exp013 独占的 {d}")
    return hits


def chk_rooms(ex, splits):
    """按房间整体保留给评测的（ACE 房间 · McVAMPIRE 整个数据集）：训练原料里不许有这些数据集的任何文件。
    数据集级比房间级更严，这里直接按数据集查（认不出按最严处理，CLAUDE.md 二）。"""
    return chk_datasets(ex, splits)


def chk_synth(ex, splits, code_dir=HERE):
    """exp013 的合成函数（synth_sounds.py）不许被训练代码 import：真正的保证是本目录不 import synth_sounds。"""
    hits = []
    for f in sorted(Path(code_dir).glob("*.py")):
        if re.search(r"^\s*(import|from)\s+synth_sounds\b", f.read_text(), re.M):
            hits.append(f"{f.name} import 了 exp013 synth_sounds（合成函数 {len(ex['synth_fns'])} 个归评测）")
    return hits


CHECKS = {
    "_说明": None,
    "librispeech_speakers": chk_speakers,
    "demand_segments": chk_demand,
    "esc50_files": chk_files("esc50_files", lambda s: s),
    "synth_fns": chk_synth,
    "rir_files": chk_files("rir_files", lambda s: s.split("#")[0]),
    "rooms_reserved": chk_rooms,
    "room_noise_segments": chk_files("room_noise_segments", lambda s: s["file"]),
    "other_segments": chk_files("other_segments", lambda s: s["file"]),
}


def check(splits, ex=None):
    """返回 {"ok", "hits", "per_key"}；exclude.json 里有不认识的键即红。"""
    ex = ex if ex is not None else load_exclude()
    unknown = sorted(set(ex) - set(CHECKS))
    hits = [f"exclude.json 有不认识的键 {k}：先在 leakcheck.CHECKS 里登记它怎么查" for k in unknown]
    per = {}
    for k, fn in CHECKS.items():
        if fn is None or k not in ex:
            continue
        h = fn(ex, splits)
        per[k] = len(h)
        hits += h
    return {"against": f"exp013 build {EXCLUDE_BUILD} exclude.json", "ok": not hits, "hits": hits[:200], "n_hits": len(hits), "per_key": per,
            "rows": sum(1 for _ in pool_rows(splits))}
