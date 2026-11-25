"""逐键比两个训练检查点的 `model`（续训核对用，plan-h100-smoke.md 第四节乙 3）。

    python cmp_ckpt.py <检查点 1> <检查点 2> [--tol 0] [--ignore-step]

通过 = 键集合相同、每个张量形状与类型相同、两边全是有限数、步数相同（除非 --ignore-step）、
且最大绝对差 ≤ tol。默认 tol = 0，即要求逐值相同；CUDA 上用一个小容差，并把最大差记进报告。
防误判通过：缺键、多键、NaN / inf、空模型、容差非法，一律判不过。打印 JSON，退出码 0 = 通过。
"""
from __future__ import annotations

import argparse
import json
import math
import sys

import torch


def compare(a: dict, b: dict, tol: float, ignore_step: bool = False) -> dict:
    ma, mb = a.get("model"), b.get("model")
    problems: list[str] = []
    if not isinstance(ma, dict) or not isinstance(mb, dict):
        return {"pass": False, "problems": ["检查点里没有 model 字典"]}
    if not ma or not mb:
        problems.append("model 为空")
    only_a, only_b = sorted(set(ma) - set(mb)), sorted(set(mb) - set(ma))
    if only_a or only_b:
        problems.append("键集合不同")
    if not ignore_step:
        sa, sb = a.get("step"), b.get("step")
        # 两边都缺 step 时 None == None 会放过（codex 第 1 轮 #5）：要求是非负整数且相同
        if not all(isinstance(x, int) and not isinstance(x, bool) and x >= 0 for x in (sa, sb)):
            problems.append(f"步数缺失或不是非负整数：{sa!r} vs {sb!r}")
        elif sa != sb:
            problems.append(f"步数不同：{sa} vs {sb}")
    max_diff, diff_keys, bad_keys = 0.0, [], []
    for k in sorted(set(ma) & set(mb)):
        x, y = ma[k], mb[k]
        if not (torch.is_tensor(x) and torch.is_tensor(y)) or x.shape != y.shape or x.dtype != y.dtype:
            bad_keys.append(k)
            continue
        if x.is_floating_point() or x.is_complex():
            if not (torch.isfinite(x).all() and torch.isfinite(y).all()):
                bad_keys.append(k)
                continue
            up = torch.complex128 if x.is_complex() else torch.float64  # 复数不能 double()，会丢虚部（codex 第 2 轮 #3）
            d = float((x.to(up) - y.to(up)).abs().max()) if x.numel() else 0.0
        else:
            d = float((x.long() - y.long()).abs().max()) if x.numel() else 0.0
        if d > 0:
            diff_keys.append(k)
        max_diff = max(max_diff, d)
    if bad_keys:
        problems.append("形状 / 类型不同或含非有限数")
    ok = not problems and max_diff <= tol
    return {"pass": bool(ok), "max_abs_diff": max_diff, "tol": tol, "n_keys": len(set(ma) & set(mb)),
            "differing_keys": diff_keys, "bad_keys": bad_keys, "only_in_1": only_a, "only_in_2": only_b,
            "step": [a.get("step"), b.get("step")], "problems": problems}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="逐键比两个检查点的 model")
    ap.add_argument("ckpt1")
    ap.add_argument("ckpt2")
    ap.add_argument("--tol", type=float, default=0.0, help="最大绝对差容差，默认 0（逐值相同）")
    ap.add_argument("--ignore-step", action="store_true", help="不要求步数相同")
    a = ap.parse_args(argv)
    if not math.isfinite(a.tol) or a.tol < 0:
        print(json.dumps({"pass": False, "problems": [f"容差非法：{a.tol}"]}, ensure_ascii=False))
        return 2
    load = lambda p: torch.load(p, map_location="cpu", weights_only=False)  # noqa: E731
    r = compare(load(a.ckpt1), load(a.ckpt2), a.tol, a.ignore_step)
    r["files"] = [a.ckpt1, a.ckpt2]
    print(json.dumps(r, ensure_ascii=False, indent=2))
    return 0 if r["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
