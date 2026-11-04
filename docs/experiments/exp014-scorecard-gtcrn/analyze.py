"""exp014 分析：读驱动导出的逐段分数 + exp013 片段清单，出几张对照表（analysis.md）。

成绩单本身（每场景过不过、拖后腿的声音）以 `scorecard.md` 为准；本脚本只补成绩单没有的比较：
1. 每个场景 × 档 × 设备：GTCRN 过不过、不过的项，以及原声直通在同一项上过不过（分清「这项对原声也判不过」）
2. Air 内置麦 vs 外接麦：同场景同档，每项 GTCRN 均值差（Air − 外接）
3. 人声与底噪同房间（chain.room.paired）vs 不同房间：每项 GTCRN 均值与不过段比例
4. 人耳候选：两个评分模型分歧的段 · V-CLICK 检出最多的段 · 每段不过项数

用法：python3 analyze.py <run 目录（含 clips.jsonl、scorecard.json）> <exp013 clips.jsonl> [--out analysis.md]
只用标准库（系统 python3 没有 numpy）。
"""

from __future__ import annotations

import argparse
import json
from collections import defaultdict
from pathlib import Path

# 只报不判的项：照常列，不算进「不过项数」（specs/rubric/scorecard.md 第四节第 3 条）
REPORT_ONLY = {"V-CLICK"}
LISTEN = ("V-NRMOS-SCOREQ", "V-NRMOS-DISTILL")
DISAGREE = 0.3  # 两个评分模型 Δ 相差多少送人耳（scorecard 第四节第 2 条）


def load_rows(path: Path) -> list[dict]:
    return [json.loads(x) for x in path.read_text().splitlines() if x.strip()]


def load_manifest(path: Path) -> dict[str, dict]:
    out = {}
    for line in path.read_text().splitlines():
        if line.strip():
            c = json.loads(line)
            out[c["id"]] = c
    return out


def score(row: dict, v: str) -> dict | None:
    for s in row["scores"]:
        if s["v"] == v:
            return s
    return None


def mean(xs: list[float]) -> float | None:
    xs = [x for x in xs if x is not None]
    return sum(xs) / len(xs) if xs else None


def fmt(x: float | None, nd: int = 2) -> str:
    return "—" if x is None else f"{x:+.{nd}f}"


def items_of(rows: list[dict]) -> list[str]:
    seen: list[str] = []
    for r in rows:
        for s in r["scores"]:
            if s["v"] not in seen:
                seen.append(s["v"])
    return seen


def fail_items(rows: list[dict], items: list[str]) -> list[str]:
    """一组段里有任一段不过的项（最差一段决定，scorecard 第四节第 1 条）。"""
    return [v for v in items if any((score(r, v) or {}).get("verdict") == "fail" for r in rows)]


def group(rows: list[dict], *keys: str) -> dict[tuple, list[dict]]:
    g: dict[tuple, list[dict]] = defaultdict(list)
    for r in rows:
        g[tuple(r[k] for k in keys)].append(r)
    return g


def scene_table(rows: list[dict], card: dict, system: str, base: str) -> list[str]:
    """表 1：每场景 × 档 × 设备，被测系统的判定 · 不过的项 · 其中原声直通也不过的项。"""
    status = {(s["scene"], s["tier"], s["device"], s["system"]): s["status"] for s in card["scenes"]}
    items = [v for v in items_of(rows) if v not in REPORT_ONLY]
    g = group(rows, "scene", "tier", "device", "system")
    out = [
        "| 场景 | 档 | 设备 | 判定 | 不过的项 | 其中原声直通也不过 |",
        "|---|---|---|---|---|---|",
    ]
    for (sc, ti, dev, sy) in sorted(k for k in g if k[3] == system):
        mine = fail_items(g[(sc, ti, dev, sy)], items)
        b = g.get((sc, ti, dev, base), [])
        both = [v for v in mine if v in fail_items(b, items)]
        out.append(
            f"| {sc} | {ti} | {dev} | {status.get((sc, ti, dev, sy), '?')} | "
            f"{' '.join(mine) or '—'} | {' '.join(both) or '—'} |"
        )
    return out


def item_counts(rows: list[dict], card: dict, systems: list[str]) -> list[str]:
    """表 2：每项在多少个「场景 × 档 × 设备」组合里不过，按系统分列。"""
    items = items_of(rows)
    g = group(rows, "scene", "tier", "device", "system")
    combos = sorted({k[:3] for k in g})
    out = ["| 项 | " + " | ".join(f"{s} 不过组合数" for s in systems) + " |",
           "|---|" + "---|" * len(systems)]
    for v in items:
        cells = []
        for sy in systems:
            n = sum(1 for c in combos if v in fail_items(g.get((*c, sy), []), [v]))
            cells.append(f"{n} / {len(combos)}")
        tag = "（只报不判）" if v in REPORT_ONLY else ""
        out.append(f"| {v}{tag} | " + " | ".join(cells) + " |")
    return out


def device_table(rows: list[dict], system: str) -> list[str]:
    """表 3：Air 内置麦 − 外接麦，每项均值差（全部场景合并，再按场景类列最差项）。"""
    items = [v for v in items_of(rows) if v not in REPORT_ONLY]
    sel = [r for r in rows if r["system"] == system]
    by = group(sel, "device")
    air, ext = by.get(("mba-builtin",), []), by.get(("external-mic",), [])
    out = ["| 项 | Air 均值 | 外接均值 | 差（Air − 外接） | Air 不过段 | 外接不过段 |",
           "|---|---|---|---|---|---|"]
    for v in items:
        a = [(score(r, v) or {}).get("value") for r in air]
        e = [(score(r, v) or {}).get("value") for r in ext]
        fa = sum(1 for r in air if (score(r, v) or {}).get("verdict") == "fail")
        fe = sum(1 for r in ext if (score(r, v) or {}).get("verdict") == "fail")
        ma, me = mean(a), mean(e)
        d = None if ma is None or me is None else ma - me
        out.append(f"| {v} | {fmt(ma)} | {fmt(me)} | {fmt(d)} | {fa} / {len(air)} | {fe} / {len(ext)} |")
    return out


def device_by_scene(rows: list[dict], system: str, card: dict) -> list[str]:
    """表 4：每场景 × 档，Air 与外接的判定和不过项数（只看被测系统）。"""
    status = {(s["scene"], s["tier"], s["device"], s["system"]): s for s in card["scenes"]}
    items = [v for v in items_of(rows) if v not in REPORT_ONLY]
    g = group([r for r in rows if r["system"] == system], "scene", "tier", "device")
    out = ["| 场景 | 档 | Air 判定 | Air 不过项数 | 外接判定 | 外接不过项数 | Air 多出的不过项 |",
           "|---|---|---|---|---|---|---|"]
    for sc, ti in sorted({k[:2] for k in g}):
        a = fail_items(g.get((sc, ti, "mba-builtin"), []), items)
        e = fail_items(g.get((sc, ti, "external-mic"), []), items)
        sa = status.get((sc, ti, "mba-builtin", system), {}).get("status", "?")
        se = status.get((sc, ti, "external-mic", system), {}).get("status", "?")
        extra = [v for v in a if v not in e]
        out.append(f"| {sc} | {ti} | {sa} | {len(a)} | {se} | {len(e)} | {' '.join(extra) or '—'} |")
    return out


def paired_table(rows: list[dict], manifest: dict, system: str) -> list[str]:
    """表 5：人声与底噪同房间 vs 不同房间（只看 Air 设备链的段，外接也一并列）。"""
    items = [v for v in items_of(rows) if v not in REPORT_ONLY]
    sel = [r for r in rows if r["system"] == system]
    flag = {}
    for r in sel:
        m = manifest.get(r["clip"], {})
        flag[r["clip"]] = bool(((m.get("chain") or {}).get("room") or {}).get("paired"))
    yes = [r for r in sel if flag[r["clip"]]]
    no = [r for r in sel if not flag[r["clip"]]]
    out = [f"同房间 {len(yes)} 段 · 不同房间 {len(no)} 段（{system}，两种设备合计）", "",
           "| 项 | 同房间均值 | 不同房间均值 | 差（同 − 不同） | 同房间不过比例 | 不同房间不过比例 |",
           "|---|---|---|---|---|---|"]
    for v in items:
        my = mean([(score(r, v) or {}).get("value") for r in yes])
        mn = mean([(score(r, v) or {}).get("value") for r in no])
        fy = sum(1 for r in yes if (score(r, v) or {}).get("verdict") == "fail")
        fn = sum(1 for r in no if (score(r, v) or {}).get("verdict") == "fail")
        d = None if my is None or mn is None else my - mn
        py = f"{fy}/{len(yes)}" if yes else "—"
        pn = f"{fn}/{len(no)}" if no else "—"
        out.append(f"| {v} | {fmt(my)} | {fmt(mn)} | {fmt(d)} | {py} | {pn} |")
    return out


def disagreements(rows: list[dict], system: str) -> list[dict]:
    """两个评分模型分歧的段：Δ 方向相反，或相差 ≥ 0.3。按相差从大到小。"""
    out = []
    for r in rows:
        if r["system"] != system:
            continue
        a, b = (score(r, v) or {} for v in LISTEN)
        x, y = a.get("value"), b.get("value")
        if x is None or y is None:
            continue
        opposite = (x >= 0) != (y >= 0)
        if opposite or abs(x - y) >= DISAGREE:
            out.append({"clip": r["clip"], "scene": r["scene"], "tier": r["tier"],
                        "scoreq": x, "distill": y, "gap": abs(x - y), "opposite": opposite})
    return sorted(out, key=lambda d: (-d["opposite"], -d["gap"]))


def top_by(rows: list[dict], system: str, v: str, n: int) -> list[tuple[str, float]]:
    vals = [(r["clip"], (score(r, v) or {}).get("value")) for r in rows if r["system"] == system]
    vals = [(c, x) for c, x in vals if x is not None]
    return sorted(vals, key=lambda t: -t[1])[:n]


def fails_per_clip(rows: list[dict], system: str) -> list[tuple[str, list[str]]]:
    out = []
    for r in rows:
        if r["system"] != system:
            continue
        f = [s["v"] for s in r["scores"] if s["verdict"] == "fail" and s["v"] not in REPORT_ONLY]
        out.append((r["clip"], f))
    return sorted(out, key=lambda t: -len(t[1]))


def gap_vs_base(rows: list[dict], system: str, base: str) -> list[str]:
    """表 6：每项「被测系统 − 原声直通」的均值差（同一批段，配对），看模型把这项拉好还是拉坏。"""
    items = [v for v in items_of(rows) if v not in REPORT_ONLY]
    b = {(r["clip"]): r for r in rows if r["system"] == base}
    out = ["| 项 | 好的方向 | 配对段数 | 均值差（模型 − 原声直通） |", "|---|---|---|---|"]
    for v in items:
        ds, better = [], "?"
        for r in rows:
            if r["system"] != system or r["clip"] not in b:
                continue
            s, t = score(r, v), score(b[r["clip"]], v)
            if s and t and s["value"] is not None and t["value"] is not None:
                ds.append(s["value"] - t["value"])
        better = better_of(rows, v)
        out.append(f"| {v} | {better} | {len(ds)} | {fmt(mean(ds))} |")
    return out


def better_of(rows: list[dict], v: str) -> str:
    """从一段的分数、过线值和判定推出方向：过且分数 > 过线 → 越大越好；不过且分数 > 过线 → 越小越好。"""
    for r in rows:
        s = score(r, v)
        if not s or s["value"] is None or s["threshold"] is None or s["value"] == s["threshold"]:
            continue
        above = s["value"] > s["threshold"]
        if s["verdict"] == "pass":
            return "越大越好" if above else "越小越好"
        if s["verdict"] == "fail":
            return "越小越好" if above else "越大越好"
    return "?"


def build(run: Path, manifest_path: Path, system: str = "gtcrn", base: str = "passthrough") -> str:
    rows = load_rows(run / "clips.jsonl")
    card = json.loads((run / "scorecard.json").read_text())
    man = load_manifest(manifest_path)
    systems = [s for s in card["meta"]["systems"]]
    L = [f"# exp014 分析表（run `{card['meta']['run_id']}`，由 analyze.py 生成，勿手改）", ""]
    L += ["## 1. 每个场景：" + system + " 不过的项，以及原声直通是否也不过", ""]
    L += scene_table(rows, card, system, base) + [""]
    L += ["## 2. 每项在多少个组合里不过（组合 = 场景 × 档 × 设备）", ""]
    L += item_counts(rows, card, systems) + [""]
    L += [f"## 3. {system}：模型 − 原声直通（同一批段配对）", ""]
    L += gap_vs_base(rows, system, base) + [""]
    L += [f"## 4. Air 内置麦 vs 外接麦（{system}，全部场景合并）", ""]
    L += device_table(rows, system) + [""]
    L += [f"## 5. Air vs 外接，按场景（{system}）", ""]
    L += device_by_scene(rows, system, card) + [""]
    L += ["## 6. 人声与底噪同房间 vs 不同房间", ""]
    L += paired_table(rows, man, system) + [""]
    dis = disagreements(rows, system)
    L += [f"## 7. 两个评分模型分歧的段（{system}，共 {len(dis)} 段，列前 12）", "",
          "| 段 | 场景 | 档 | SCOREQ Δ | Distill Δ | 相差 | 方向相反 |", "|---|---|---|---|---|---|---|"]
    for d in dis[:12]:
        L.append(f"| {d['clip']} | {d['scene']} | {d['tier']} | {fmt(d['scoreq'])} | "
                 f"{fmt(d['distill'])} | {d['gap']:.2f} | {'是' if d['opposite'] else '否'} |")
    L += ["", f"## 8. V-CLICK 检出最多的段（{system}，个/分钟）", ""]
    for c, x in top_by(rows, system, "V-CLICK", 5):
        L.append(f"- {c}：{x:.0f}")
    L += ["", f"## 9. 不过项最多的段（{system}，前 10）", ""]
    for c, f in fails_per_clip(rows, system)[:10]:
        L.append(f"- {c}：{len(f)} 项（{' '.join(f)}）")
    return "\n".join(L) + "\n"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("run", type=Path)
    ap.add_argument("manifest", type=Path)
    ap.add_argument("--system", default="gtcrn")
    ap.add_argument("--base", default="passthrough")
    ap.add_argument("--out", type=Path)
    a = ap.parse_args()
    text = build(a.run, a.manifest, a.system, a.base)
    if a.out:
        a.out.write_text(text)
    else:
        print(text)


if __name__ == "__main__":
    main()
