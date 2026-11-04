"""exp013 · 生成 vdn-eval 场景任务文件 `src/eval/suite/scenes.json`（地点 × 档 × 设备：15 × 2 × 2 = 60 条）。
`mba-builtin` 是主线（过 MacBook Air 内置麦模拟）；`external-mic` 是同一批素材的「无设备链对照」，
成绩单拿两者比，看同场景下 Air 麦让分数掉多少（大总管 2026-11-03 转 scorecard 的要求）。

任务文件是派生物：地点与档位来自本目录 scenes.json，片段来自某个 build 的 clips.jsonl。不手改，改了重跑本脚本。
graders 由 scorecard 填（D81；与 scorecard 2026-11-03 商定）。重跑时保留任务文件里已有的 graders。
`real.kind = "scene"` 的读法由 scorecard 在 vdn-eval 里加（RealSource::Scene）。
用法：python3 docs/experiments/exp013-scene-suite/tasks.py --build <build-id>
"""
import argparse
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = ROOT / "src/eval/suite/scenes.json"
SCENES = json.loads((HERE / "scenes.json").read_text())


def catalog_name(key):
    """中文名取自 021 主表（正本），不在这里另写。"""
    for line in (ROOT / "docs/research/021_meeting-scenes/catalog.md").read_text().splitlines():
        if line.startswith(f"| `{key}` |"):
            return line.split("|")[2].strip()
    raise KeyError(key)


def build_tasks(build, old=None):
    old = {t["id"]: t for t in (old or {}).get("tasks", [])}
    clips = f"docs/experiments/exp013-scene-suite/builds/{build}/clips.jsonl"
    tasks = []
    combos = [(d, k, t) for d in SCENES["devices"] for k in SCENES["scenes"] for t in ("common", "stress")]
    for device, key, tier in combos:
        S = SCENES["scenes"][key]
        ext = device != "mba-builtin"
        tid = f"S-{key}-{tier}" + ("-ext" if ext else "")
        keep_all = S.get("polarity") == "preserve_all"
        tasks.append({
            "id": tid,
            "set": "capability",
            "polarity": "preserve" if keep_all else "suppress",
            "origin": f"exp013 build {build} · 021 地点 {key}（{catalog_name(key)}）· {'常见档' if tier == 'common' else '压力档'}",
            "desc": ("近处几个人都要留（会议模式，干净参考 = 所有要留的人）" if keep_all else "只留主说话人")
                    + f"；{SCENES['counts'][tier]} 段 × {SCENES['clip_s']:g} s；设备 {device}"
                    + ("（无设备链对照：与 mba-builtin 同素材，只差 MacBook Air 麦模拟）" if ext else ""),
            "real": {"kind": "scene", "clips": clips, "scene": key, "tier": tier, "device": device},
            "graders": old.get(tid, {}).get("graders", []),
        })
    return {
        "name": "exp013-scenes",
        "doc": "场景测试集（exp013）：15 个开会地点 × 常见 / 压力两档，每条任务展开成该地点该档的全部片段；"
               "片段清单格式见 docs/experiments/exp013-scene-suite/clip-manifest.md。graders 由 scorecard 填。",
        "seed": SCENES["base_seed"],
        "defaults": {"trials": 1},
        "tasks": tasks,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--build", required=True)
    a = ap.parse_args()
    if not (HERE / "builds" / a.build / "clips.jsonl").exists():
        raise SystemExit(f"没有 build {a.build}")
    old = json.loads(OUT.read_text()) if OUT.exists() else None
    OUT.write_text(json.dumps(build_tasks(a.build, old), ensure_ascii=False, indent=2) + "\n")
    print(f"写入 {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
