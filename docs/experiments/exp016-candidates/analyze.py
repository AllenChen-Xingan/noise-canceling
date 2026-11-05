"""exp016 配对分析：同一段上「候选 − GTCRN」的差，按评分项 · 场景 · 声音拆开，给平均与 95% 区间。

口径（按 025 待拍板默认 G-1「平均 + 95% 区间三态」推进，待人确认）：
- 差值统一成「正 = 候选比 GTCRN 好」：越大越好的项取 候选 − GTCRN，越小越好的项取 GTCRN − 候选。方向取成绩单里每项写明的
  `better`（复用 exp014 analyze.better_map，不另抄）
- 区间：成对片段自助重抽样（同一份素材的 Air 内置麦与外接麦两段算一个单位，`pair` 字段），2000 次、种子固定，取 2.5 / 97.5 百分位
- 三态：区间整体 > 0 → 「候选好」；整体 < 0 → 「候选差」；跨 0 → 「分不出」
- 不比的项：V-NAN（全 0）· V-LAT（外部系统延迟只是声明值）· 判「不知道」的段不进该项
- 排「GTCRN 落后最多」用标准化效应 = 格子平均 ÷ 该项在全部段上差值的标准差（各项单位不同，用同一把全局尺子，
  不用格子内标准差——那会偏向波动小的格子）；只排区间整体 > 0、且配对单位 ≥ MIN_UNITS 的格子。这个排序方式是本脚本的选择（推断），
  表里同时给原单位的平均。格子很多、未做多重比较校正：前 5 处是「值得先看」，不是统计显著的定论

用法：python3 analyze.py <run 目录>[,<run 目录>…] <exp013 clips.jsonl> --out <analysis.md> [--json <analysis.json>]
多个 run（如 GTCRN 一侧与候选一侧分开跑）按 `merge` 合并：规则版本 · 场景集 · 是否带评分模型须一致，同一（片段, 系统）不许出现两次
"""
from __future__ import annotations

import argparse
import json
import random
from collections import defaultdict
from pathlib import Path

import importlib.util

HERE = Path(__file__).resolve().parent
# exp014 的 analyze.py 与本文件同名：按路径载入、换个模块名，免得自己导入自己
_spec = importlib.util.spec_from_file_location("exp014_analyze", HERE.parent / "exp014-scorecard-gtcrn/analyze.py")
_exp014 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_exp014)
better_map = _exp014.better_map  # 每项方向取自成绩单（exp014 正本）

BASE = "gtcrn"
SKIP = {"V-NAN", "V-LAT"}
N_BOOT = 2000
SEED = 20261105
MIN_UNITS = 5  # 拆到 场景 × 声音 时，配对单位（一份素材的两个设备版本算一个）少于这么多不排名


def load(run: Path):
    rows = [json.loads(l) for l in (run / "clips.jsonl").read_text().splitlines() if l.strip()]
    card = json.loads((run / "scorecard.json").read_text())
    return rows, card


SAME_META = ("rules", "suite", "qa")  # 合并的几个 run 这几项必须都有、且相同


def merge(loaded):
    """[(rows, card)] → (rows, card)：card 取第一个的场景表（方向 · 单位），meta.run_id 用「+」连起来。
    拒绝：元数据缺项或不同 · 方向或单位不同 · 任一 run 内或跨 run 的（片段, 系统）重复 · 同一片段的场景 / 声音标签不同。"""
    card = json.loads(json.dumps(loaded[0][1]))
    first = card["meta"]
    for k in SAME_META:
        if first.get(k) is None:
            raise SystemExit(f"run {first.get('run_id')} 的元数据缺 {k}，不能配对")
    rows, seen, tags = [], set(), {}
    for i, (rs, c) in enumerate(loaded):
        m = c["meta"]
        if i:
            for k in SAME_META:
                if m.get(k) != first[k]:
                    raise SystemExit(f"run {m.get('run_id')} 的 {k} = {m.get(k)!r}，与 {first['run_id']} 的 {first[k]!r} 不同，不能配对")
            if better_map(c) != better_map(card) or units(c) != units(card):
                raise SystemExit(f"run {m.get('run_id')} 的评分项方向或单位与第一个 run 不同")
            card["meta"]["run_id"] += "+" + m["run_id"]
        for r in rs:
            key = (r["clip"], r["system"])
            if key in seen:
                raise SystemExit(f"{key} 出现两次（run {m.get('run_id')}），不知道用哪个")
            seen.add(key)
            t = (r["scene"], sorted(x["cls"] for x in r["sounds"]))
            if tags.setdefault(r["clip"], t) != t:
                raise SystemExit(f"片段 {r['clip']} 在两个 run 里的场景 / 声音标签不同")
            rows.append(r)
    return rows, card


def units(card) -> dict[str, str]:
    out = {}
    for s in card["scenes"]:
        for i in s["items"] + (s.get("listen") or {}).get("models", []):
            out.setdefault(i["v"], i.get("unit", ""))
    return out


def pairs_of(manifest: Path) -> dict[str, str]:
    """片段 id → 配对单位 id（同一份素材的两个设备版本取字典序较小的那个 id）。"""
    out = {}
    for l in manifest.read_text().splitlines():
        if l.strip():
            c = json.loads(l)
            out[c["id"]] = min(c["id"], c.get("pair") or c["id"])
    return out


def diffs(rows, better, cand):
    """[(片段, 场景, 声音类别集合, 评分项, 正向差)]：只收两边都有数的。"""
    by = defaultdict(dict)
    meta = {}
    for r in rows:
        by[r["clip"]][r["system"]] = {x["v"]: x["value"] for x in r["scores"]}
        meta[r["clip"]] = (r["scene"], frozenset(s["cls"] for s in r["sounds"]))
    out = []
    for clip, sy in by.items():
        if BASE not in sy or cand not in sy:
            continue
        for v, b in sy[cand].items():
            a = sy[BASE].get(v)
            if v in SKIP or a is None or b is None or v not in better:
                continue
            sign = 1.0 if better[v] == "越大越好" else -1.0
            out.append((clip, *meta[clip], v, sign * (b - a)))
    return out


def ci(vals_by_unit: dict[str, list[float]], rng: random.Random):
    """成对单位自助重抽样的平均与 95% 区间。返回 (段数, 平均, 下, 上, 标准差)。"""
    units_ = list(vals_by_unit)
    flat = [x for u in units_ for x in vals_by_unit[u]]
    n = len(flat)
    m = sum(flat) / n
    sd = (sum((x - m) ** 2 for x in flat) / (n - 1)) ** 0.5 if n > 1 else 0.0
    if len(units_) < 2:
        return n, m, None, None, sd
    boots = []
    for _ in range(N_BOOT):
        pick = [vals_by_unit[units_[rng.randrange(len(units_))]] for _ in units_]
        s = [x for p in pick for x in p]
        boots.append(sum(s) / len(s))
    boots.sort()
    return n, m, boots[int(0.025 * N_BOOT)], boots[int(0.975 * N_BOOT) - 1], sd


def state(lo, hi):
    if lo is None:
        return "样本太少"
    return "候选好" if lo > 0 else "候选差" if hi < 0 else "分不出"


def group(ds, pair, key):
    """按 key(行) 分组，每组再按配对单位归堆。"""
    g = defaultdict(lambda: defaultdict(list))
    for d in ds:
        for k in key(d):
            g[k][pair.get(d[0], d[0])].append(d[4])
    return g


def item_sd(ds) -> dict[str, float]:
    """每个评分项在全部段上差值的标准差（标准化效应的分母）。"""
    by = defaultdict(list)
    for d in ds:
        by[d[3]].append(d[4])
    out = {}
    for v, xs in by.items():
        m = sum(xs) / len(xs)
        out[v] = (sum((x - m) ** 2 for x in xs) / (len(xs) - 1)) ** 0.5 if len(xs) > 1 else 0.0
    return out


def summarize(ds, pair, key, seed, sd_of=None):
    rng = random.Random(seed)
    sd_of = sd_of or item_sd(ds)
    out = []
    for k, vu in sorted(group(ds, pair, key).items(), key=lambda kv: str(kv[0])):
        n, m, lo, hi, sd = ci(vu, rng)
        v = k[-1] if isinstance(k, tuple) else k
        g = sd_of.get(v, 0.0)
        out.append({"key": k, "n": n, "units": len(vu), "mean": m, "lo": lo, "hi": hi, "sd": sd, "state": state(lo, hi),
                    "effect": (m / g) if g > 0 else 0.0})
    return out


def analyze(runs: list[Path], manifest: Path, cands: list[str]):
    rows, card = merge([load(r) for r in runs])
    better, unit, pair = better_map(card), units(card), pairs_of(manifest)
    res = {"run_id": card["meta"]["run_id"], "base": BASE, "candidates": {}}
    for c in cands:
        ds = diffs(rows, better, c)
        g = item_sd(ds)
        res["candidates"][c] = {
            "by_item": summarize(ds, pair, lambda d: [d[3]], SEED, g),
            "by_scene_item": summarize(ds, pair, lambda d: [(d[1], d[3])], SEED + 1, g),
            "by_sound_item": summarize(ds, pair, lambda d: [(s, d[3]) for s in d[2]], SEED + 2, g),
            "by_scene_sound_item": summarize(ds, pair, lambda d: [(d[1], s, d[3]) for s in d[2]], SEED + 3, g),
        }
    res["better"], res["unit"] = better, unit
    return res


def f(x, nd=2):
    return "—" if x is None else f"{x:+.{nd}f}"


def top_behind(rows_, k=5):
    """GTCRN 落后最多：区间整体 > 0（候选好）且配对单位 ≥ MIN_UNITS，按标准化效应降序。"""
    ok = [r for r in rows_ if r["state"] == "候选好" and r["units"] >= MIN_UNITS]
    return sorted(ok, key=lambda r: -r["effect"])[:k]


def render(res) -> str:
    unit = res["unit"]
    L = [f"# exp016 配对分析（run `{res['run_id']}`，由 analyze.py 生成，勿手改）", "",
         f"差值 = 候选相对 {res['base']}，**正 = 候选更好**；区间 = 成对片段自助重抽样 95%（{N_BOOT} 次）。"
         "按 025 待拍板 G-1 默认推进，待人确认。", ""]
    for c, r in res["candidates"].items():
        L += [f"## {c} vs {res['base']}", "", "### 每个评分项（全部 480 段）", "",
              "| 评分项 | 方向 | 单位 | 段数 | 平均 | 95% 区间 | 判断 |", "|---|---|---|---|---|---|---|"]
        for x in r["by_item"]:
            v = x["key"]
            L.append(f"| {v} | {res['better'].get(v, '?')} | {unit.get(v, '')} | {x['n']} | {f(x['mean'])} | "
                     f"[{f(x['lo'])}, {f(x['hi'])}] | {x['state']} |")
        L += ["", "### 场景 × 评分项：判断不是「分不出」的格子", "",
              "| 场景 | 评分项 | 段数 | 平均 | 95% 区间 | 判断 |", "|---|---|---|---|---|---|"]
        for x in r["by_scene_item"]:
            if x["state"] in ("候选好", "候选差"):
                s, v = x["key"]
                L.append(f"| {s} | {v} | {x['n']} | {f(x['mean'])} {unit.get(v, '')} | [{f(x['lo'])}, {f(x['hi'])}] | {x['state']} |")
        L += ["", f"### GTCRN 落后最多的前 5 处（场景 × 声音 × 评分项，配对单位 ≥ {MIN_UNITS}，按标准化效应排；未做多重比较校正）", "",
              "| 场景 | 声音 | 评分项 | 段数（单位） | 平均 | 95% 区间 | 标准化效应 |", "|---|---|---|---|---|---|---|"]
        for x in top_behind(r["by_scene_sound_item"]):
            s, snd, v = x["key"]
            L.append(f"| {s} | {snd} | {v} | {x['n']}（{x['units']}） | {f(x['mean'])} {unit.get(v, '')} | [{f(x['lo'])}, {f(x['hi'])}] | {x['effect']:.2f} |")
        L += ["", f"### 反过来：候选落后 GTCRN 最多的前 5 处（同口径）", "",
              "| 场景 | 声音 | 评分项 | 段数（单位） | 平均 | 95% 区间 | 标准化效应 |", "|---|---|---|---|---|---|---|"]
        worse = sorted([x for x in r["by_scene_sound_item"] if x["state"] == "候选差" and x["units"] >= MIN_UNITS],
                       key=lambda x: x["effect"])[:5]
        for x in worse:
            s, snd, v = x["key"]
            L.append(f"| {s} | {snd} | {v} | {x['n']}（{x['units']}） | {f(x['mean'])} {unit.get(v, '')} | [{f(x['lo'])}, {f(x['hi'])}] | {x['effect']:.2f} |")
        L.append("")
    return "\n".join(L) + "\n"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("runs", help="一个或几个 run 目录，逗号分隔")
    ap.add_argument("manifest", type=Path)
    ap.add_argument("--cands", default="dpdfnet2,dfn3-ll")
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--json", type=Path)
    a = ap.parse_args()
    res = analyze([Path(x) for x in a.runs.split(",")], a.manifest, a.cands.split(","))
    a.out.write_text(render(res))
    if a.json:
        a.json.write_text(json.dumps(res, ensure_ascii=False, separators=(",", ":"), default=list) + "\n")
    print(f"已写 {a.out.name}")


if __name__ == "__main__":
    main()
