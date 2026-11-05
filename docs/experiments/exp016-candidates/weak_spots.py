"""exp016 · GTCRN 弱在哪：把几份 analysis.json 的「场景 × 声音 × 评分项」格子合起来，给训练线用。

- 弱项：每个候选都判「候选好」（区间整体在 GTCRN 更差的一侧）且配对单位 ≥ MIN_UNITS 的格子。
- 守住项：每个候选都判「候选差」的格子（GTCRN 在这里比所有候选好，加训时别退）。
- 不看：V-VACUUM（= V-NR 反向，同一个数）· V-CLICK（只报不判）· V-LAT · V-NAN（不进配对或全 0）。
排序：按格子数多少排「场景 × 声音」，同数按最小标准化效应。未做多重比较校正，是「先看哪里」，不是定论。

用法（项目根下）：python3 docs/experiments/exp016-candidates/weak_spots.py <analysis.json>[,<analysis.json>…] --out <新目录>
"""
import argparse
import json
from collections import defaultdict
from pathlib import Path

SKIP = {"V-VACUUM", "V-CLICK", "V-LAT", "V-NAN"}
MIN_UNITS = 5
GOOD, BAD = "候选好", "候选差"


def load(path):
    d = json.loads(Path(path).read_text())
    (name, c), = d["candidates"].items()  # 每份 analysis.json 只放一个候选
    return name, d["run_id"], {tuple(r["key"]): r for r in c["by_scene_sound_item"]}


def cells(cands, state):
    """所有候选都判 state、单位都够的格子 → {(场景, 声音): [(评分项, {候选: 记录})]}。"""
    names = list(cands)
    common = set.intersection(*(set(cands[n]) for n in names))
    out = defaultdict(list)
    for k in sorted(common):
        if k[2] in SKIP:
            continue
        recs = {n: cands[n][k] for n in names}
        if all(r["state"] == state and r["units"] >= MIN_UNITS for r in recs.values()):
            out[(k[0], k[1])].append((k[2], recs))
    return out


def ranked(groups):
    def key(item):
        (_, items) = item
        return (-len(items), -min(abs(r["effect"]) for _, recs in items for r in recs.values()))
    return sorted(groups.items(), key=key)


def fmt(r, unit):
    return f"{r['mean']:+.3g} {unit} [{r['lo']:+.3g}, {r['hi']:+.3g}]"


def render(names, runs, units, weak, guard, top):
    L = [f"# GTCRN 弱在哪（由 weak_spots.py 生成，勿手改）", "",
         f"来源 run：{' · '.join(f'`{r}`' for r in runs)}。候选：{' · '.join(names)}。",
         f"弱项 = 每个候选都比 GTCRN 好（95% 区间不跨 0）、配对单位 ≥ {MIN_UNITS}；守住项 = 每个候选都比 GTCRN 差。"
         f"不看 {' · '.join(sorted(SKIP))}。差值正 = 候选更好。未做多重比较校正；全部是合成音频。", ""]
    for title, groups in (("弱项", weak), ("守住项（加训时别退）", guard)):
        L += [f"## {title}：{len(groups)} 个「场景 × 声音」，{sum(len(v) for v in groups.values())} 个格子", "",
              "| 场景 | 声音 | 格子数 | 评分项 |", "|---|---|---|---|"]
        for (sc, so), items in ranked(groups)[:top]:
            L.append(f"| {sc} | {so} | {len(items)} | {' · '.join(sorted(v for v, _ in items))} |")
        L += ["", f"### {title}：标准化效应最大的 {top} 个格子", "",
              "| 场景 | 声音 | 评分项 | 段数（单位） | " + " | ".join(names) + " |",
              "|---|---|---|---|" + "---|" * len(names)]
        flat = [(sc, so, v, recs) for (sc, so), items in groups.items() for v, recs in items]
        flat.sort(key=lambda x: -min(abs(r["effect"]) for r in x[3].values()))
        for sc, so, v, recs in flat[:top]:
            r0 = recs[names[0]]
            L.append(f"| {sc} | {so} | {v} | {r0['n']}（{r0['units']}） | "
                     + " | ".join(fmt(recs[n], units.get(v, "")) for n in names) + " |")
        L.append("")
    return "\n".join(L)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("analyses", help="几份 analysis.json，逗号分隔")
    ap.add_argument("--out", type=Path, required=True, help="新目录（已存在就拒绝）")
    ap.add_argument("--top", type=int, default=20)
    a = ap.parse_args()
    if a.out.exists():
        raise SystemExit(f"{a.out} 已存在：每次用新目录")
    cands, runs, units = {}, [], {}
    for p in a.analyses.split(","):
        name, run, recs = load(p)
        if name in cands:
            raise SystemExit(f"候选 {name} 出现两次")
        cands[name], runs = recs, runs + [run]
        units.update(json.loads(Path(p).read_text())["unit"])
    weak, guard = cells(cands, GOOD), cells(cands, BAD)
    a.out.mkdir(parents=True)
    (a.out / "weak_spots.md").write_text(render(list(cands), runs, units, weak, guard, a.top))
    js = {"runs": runs, "candidates": list(cands), "min_units": MIN_UNITS, "skip": sorted(SKIP),
          "weak": {f"{k[0]}|{k[1]}": {v: {n: r for n, r in recs.items()} for v, recs in items} for k, items in weak.items()},
          "guard": {f"{k[0]}|{k[1]}": {v: {n: r for n, r in recs.items()} for v, recs in items} for k, items in guard.items()}}
    (a.out / "weak_spots.json").write_text(json.dumps(js, ensure_ascii=False, indent=1))
    print(f"已写 {a.out}：弱项 {len(weak)} 组 · 守住项 {len(guard)} 组")


if __name__ == "__main__":
    main()
