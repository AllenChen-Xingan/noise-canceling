"""exp016 配对分析：同一段上「候选 − GTCRN」的差，按评分项 · 场景 · 声音拆开，给平均与 95% 区间。

口径（按 025 待拍板默认 G-1「平均 + 95% 区间三态」推进，待人确认）：
- 差值统一成「正 = 候选比 GTCRN 好」：越大越好的项取 候选 − GTCRN，越小越好的项取 GTCRN − 候选。方向取成绩单里每项写明的
  `better`（复用 exp014 analyze.better_map，不另抄）
- 区间：成对片段自助重抽样（同一份素材的 Air 内置麦与外接麦两段算一个单位，`pair` 字段），2000 次、种子固定，取 2.5 / 97.5 百分位
- 三态：区间整体 > 0 → 「候选好」；整体 < 0 → 「候选差」；跨 0 → 「分不出」
- 不比的项：V-NAN（全 0）· V-LAT（外部系统延迟只是声明值）· 判「不知道」的段不进该项
- 排「GTCRN 落后最多」用标准化效应（平均 ÷ 标准差），因为各项单位不同；这个排序方式是本脚本的选择（推断），表里同时给原单位的平均

用法：python3 analyze.py <run 目录> <exp013 clips.jsonl> --out <analysis.md> [--json <analysis.json>]
"""
from __future__ import annotations

import argparse
import json
import random
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "exp014-scorecard-gtcrn"))
from analyze import better_map  # noqa: E402  exp014：每项方向取自成绩单

BASE = "gtcrn"
SKIP = {"V-NAN", "V-LAT"}
N_BOOT = 2000
SEED = 20261105
MIN_N = 6  # 拆到 场景 × 声音 时，少于这么多段不排名


def load(run: Path):
    rows = [json.loads(l) for l in (run / "clips.jsonl").read_text().splitlines() if l.strip()]
    card = json.loads((run / "scorecard.json").read_text())
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


def summarize(ds, pair, key, seed):
    rng = random.Random(seed)
    out = []
    for k, vu in sorted(group(ds, pair, key).items(), key=lambda kv: str(kv[0])):
        n, m, lo, hi, sd = ci(vu, rng)
        out.append({"key": k, "n": n, "units": len(vu), "mean": m, "lo": lo, "hi": hi, "sd": sd, "state": state(lo, hi),
                    "effect": (m / sd) if sd > 0 else 0.0})
    return out


def analyze(run: Path, manifest: Path, cands: list[str]):
    rows, card = load(run)
    better, unit, pair = better_map(card), units(card), pairs_of(manifest)
    res = {"run_id": card["meta"]["run_id"], "base": BASE, "candidates": {}}
    for c in cands:
        ds = diffs(rows, better, c)
        res["candidates"][c] = {
            "by_item": summarize(ds, pair, lambda d: [d[3]], SEED),
            "by_scene_item": summarize(ds, pair, lambda d: [(d[1], d[3])], SEED + 1),
            "by_sound_item": summarize(ds, pair, lambda d: [(s, d[3]) for s in d[2]], SEED + 2),
            "by_scene_sound_item": summarize(ds, pair, lambda d: [(d[1], s, d[3]) for s in d[2]], SEED + 3),
        }
    res["better"], res["unit"] = better, unit
    return res


def f(x, nd=2):
    return "—" if x is None else f"{x:+.{nd}f}"


def top_behind(rows_, k=5):
    """GTCRN 落后最多：区间整体 > 0（候选好）且段数 ≥ MIN_N，按标准化效应降序。"""
    ok = [r for r in rows_ if r["state"] == "候选好" and r["n"] >= MIN_N]
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
        L += ["", f"### GTCRN 落后最多的前 5 处（场景 × 声音 × 评分项，段数 ≥ {MIN_N}，按标准化效应排）", "",
              "| 场景 | 声音 | 评分项 | 段数 | 平均 | 95% 区间 | 标准化效应 |", "|---|---|---|---|---|---|---|"]
        for x in top_behind(r["by_scene_sound_item"]):
            s, snd, v = x["key"]
            L.append(f"| {s} | {snd} | {v} | {x['n']} | {f(x['mean'])} {unit.get(v, '')} | [{f(x['lo'])}, {f(x['hi'])}] | {x['effect']:.2f} |")
        L.append("")
    return "\n".join(L) + "\n"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run", type=Path)
    ap.add_argument("manifest", type=Path)
    ap.add_argument("--cands", default="dpdfnet2,dfn3-ll")
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--json", type=Path)
    a = ap.parse_args()
    res = analyze(a.run, a.manifest, a.cands.split(","))
    a.out.write_text(render(res))
    if a.json:
        a.json.write_text(json.dumps(res, ensure_ascii=False, separators=(",", ":"), default=list) + "\n")
    print(f"已写 {a.out.name}")


if __name__ == "__main__":
    main()
