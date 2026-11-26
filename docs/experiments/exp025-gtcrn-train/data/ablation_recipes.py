"""配方消融：从 recipe.json 生成 5 份「只翻一个旋钮」的副本（2026-11-26 大总管交代）。

用途：训练侧做单旋钮消融。正本是 recipe.json；本文件是这 5 份的唯一清单（VARIANTS），
落盘的 JSON 全部由它生成，不手改——改了配方就重跑一次，再用 --check 核对。

    python3 ablation_recipes.py            # 覆盖写 recipes/ablation/*.json
    python3 ablation_recipes.py --check    # 只核对：缺文件或与当前配方不一致就退 1

每份只做两件事：把 knobs[knob]["on"] 翻成目标值；在顶层加 _ablation，记下基准字节的
sha256、旋钮名、目标值。其余字段（含每个旋钮的 src、noise_pools、speech）逐字段保持原样。
翻过去与基准相同（旋钮已经在目标值）时报错：那样的副本不叫消融，只会和基准混在一起。
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BASE = HERE / "recipe.json"
OUT_DIR = HERE / "recipes" / "ablation"

# 这一份清单是这 5 个名字、这 5 个旋钮的唯一出处；测试与训练侧都从这里取
VARIANTS = [
    ("no-eq", "eq", False),
    ("no-clip", "clip", False),
    ("no-n_noises", "n_noises", False),
    ("no-reverb", "reverb", False),
    ("high_snr-on", "high_snr", True),
]


def sha256(b):
    return hashlib.sha256(b).hexdigest()


def variant(base_bytes, knob, on):
    """基准字节 → 一份消融配方（dict）。只改 knobs[knob].on，另加 _ablation 记账。"""
    d = json.loads(base_bytes)
    knobs = d.get("knobs")
    if not isinstance(knobs, dict) or knob not in knobs:
        raise ValueError(f"基准配方里没有旋钮 {knob}：消融清单与 recipe.json 已经对不上")
    cur = knobs[knob]
    if not isinstance(cur, dict) or "on" not in cur:
        raise ValueError(f"旋钮 {knob} 没有 on 开关，翻不动：{cur!r}")
    if cur["on"] == on:
        raise ValueError(f"旋钮 {knob} 已经是 {on}，再翻一次就和基准一样，不算消融")
    cur["on"] = on
    d["_ablation"] = {"base_sha256": sha256(base_bytes), "knob": knob, "on": on}
    return d


def render(d):
    """dict → 落盘字节。固定写法，同样内容每次生成的字节相同。"""
    return (json.dumps(d, ensure_ascii=False, indent=1) + "\n").encode("utf-8")


def render_all(base_bytes):
    """{文件名: 字节}：5 份全部在内存里生成，写盘与核对走同一份结果。"""
    return {f"{name}.json": render(variant(base_bytes, knob, on)) for name, knob, on in VARIANTS}


def write_all(base=BASE, out_dir=OUT_DIR):
    """生成并覆盖写入 5 份文件，返回写出的路径列表。"""
    base = Path(base)
    out_dir = Path(out_dir)
    got = render_all(base.read_bytes())
    out_dir.mkdir(parents=True, exist_ok=True)
    paths = []
    for fn, data in got.items():
        p = out_dir / fn
        p.write_bytes(data)
        paths.append(p)
    return paths


def check(base=BASE, out_dir=OUT_DIR):
    """核对已落盘的文件：返回 [(变体名, "missing" | "differs")]，空列表表示全部一致。"""
    base = Path(base)
    out_dir = Path(out_dir)
    try:
        got = render_all(base.read_bytes())
    except (OSError, ValueError) as e:
        print(f"读基准配方 {base} 失败：{e}", file=sys.stderr)
        return [(name, "missing") for name, _, _ in VARIANTS]
    bad = []
    for name, _, _ in VARIANTS:
        fn = f"{name}.json"
        p = out_dir / fn
        if not p.is_file():
            bad.append((name, "missing"))
        elif p.read_bytes() != got[fn]:
            bad.append((name, "differs"))
    return bad


def main(argv=None):
    ap = argparse.ArgumentParser(description="从 recipe.json 生成单旋钮消融配方")
    ap.add_argument("--base", type=Path, default=BASE, help="基准配方（默认 data/recipe.json）")
    ap.add_argument("--out-dir", type=Path, default=OUT_DIR, help="输出目录（默认 data/recipes/ablation）")
    ap.add_argument("--check", action="store_true", help="只核对，不写文件；不一致退 1")
    a = ap.parse_args(argv)
    try:
        if a.check:
            bad = check(a.base, a.out_dir)
            for name, why in bad:
                print(f"{'缺文件' if why == 'missing' else '不一致'}: {a.out_dir / (name + '.json')}", file=sys.stderr)
            if bad:
                print(f"{len(bad)} 份要重新生成：python3 ablation_recipes.py", file=sys.stderr)
                return 1
            print(f"{len(VARIANTS)} 份消融配方与 {a.base} 一致")
            return 0
        paths = write_all(a.base, a.out_dir)
    except ValueError as e:
        print(e, file=sys.stderr)
        return 2
    for p in paths:
        print(p)
    return 0


if __name__ == "__main__":
    sys.exit(main())
