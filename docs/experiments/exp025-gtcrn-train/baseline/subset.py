"""exp025 · 从场景任务清单裁出一个小测试集（冒烟用，≤ 5 段）。

读 `src/eval/suite/scenes.json` 与它指向的 `clips.jsonl`，只留给定的片段 id：写一份子集 `clips.jsonl`（原行原样，不改字段）
和一份子集任务清单（只留覆盖到这些片段的任务，`real.clips` 改指向子集）。音频路径不变，仍读原 build 的音频。

用法：python subset.py --suite src/eval/suite/scenes.json --clips id1,id2 --out <目录>  → <目录>/clips.jsonl + suite.json（只新建）
"""
import argparse
import json
from pathlib import Path

MAX_CLIPS = 5  # 本机冒烟上限（大总管 2026-11-25：本机只做 ≤ 5 段冒烟）


def subset(suite, clip_lines, ids):
    """返回 (子集 clips 行, 子集任务清单 dict)。`clip_lines` = 原 clips.jsonl 的行。"""
    if not 0 < len(ids) <= MAX_CLIPS:
        raise ValueError(f"片段数要在 1–{MAX_CLIPS} 之间，收到 {len(ids)}")
    want = set(ids)
    rows = [(json.loads(l), l) for l in clip_lines if l.strip()]
    keep = [(r, l) for r, l in rows if r["id"] in want]
    missing = want - {r["id"] for r, _ in keep}
    if missing:
        raise ValueError(f"清单里没有这些片段：{sorted(missing)}")
    keys = {(r["scene"], r["tier"], r["device"]) for r, _ in keep}
    tasks = [t for t in suite["tasks"] if (t["real"]["scene"], t["real"]["tier"], t["real"]["device"]) in keys]
    return [l for _, l in keep], {**suite, "tasks": tasks}


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--suite", default="src/eval/suite/scenes.json")
    ap.add_argument("--clips", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    suite = json.loads(Path(a.suite).read_text())
    srcs = {t["real"]["clips"] for t in suite["tasks"]}
    if len(srcs) != 1:
        raise ValueError(f"任务清单指向不止一份 clips.jsonl：{sorted(srcs)}")
    lines, sub = subset(suite, Path(srcs.pop()).read_text().splitlines(), [c for c in a.clips.split(",") if c])
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    jl = out / "clips.jsonl"
    for t in sub["tasks"]:
        t["real"]["clips"] = str(jl.resolve())
    with open(jl, "x") as f:
        f.write("\n".join(lines) + "\n")
    with open(out / "suite.json", "x") as f:
        f.write(json.dumps(sub, ensure_ascii=False, indent=1) + "\n")
    print(f"子集：{len(lines)} 段 · {len(sub['tasks'])} 条任务 → {out}")


if __name__ == "__main__":
    main()
