"""训练有没有用（判据第 4 版的另一半，大总管定 2026-11-25）：H100 乙段 A、B 各训 300 步后，
开发份的 hybrid 读数必须低于第 0 步。第 0 步 = 同代码 CPU 参照（known-dev-cpu.json，起点模型；B 零初始化时与 A 相同），
口径一致：数据字段（KNOWN_META_KEYS）逐项要和这次运行相同。跨设备差在冒烟里核过 ≤ 1e-3，所以要低过这个容差才算低。

    python dev_gate.py <运行目录> --known-dev known-dev-cpu.json [--step 300] [--tol 1e-3]

退出码：0 过 · 1 不过 · 2 不完整（缺读数、口径对不上、文件坏）。过不了就不开正式训练。
"""
from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

from smoke import KNOWN_META_KEYS as META_KEYS  # 正本在 smoke.py（与 --known-dev 同一份字段表）


def _incomplete(reason: str, **kw) -> dict:
    return {"verdict": "incomplete", "pass": False, "reason": reason, **kw}


def check(run_dir: Path, known: dict, step: int = 300, tol: float = 1e-3) -> dict:
    run_dir = Path(run_dir)
    meta = known.get("meta")
    h0 = (known.get("readings") or {}).get("hybrid")
    if not isinstance(meta, dict) or not (isinstance(h0, (int, float)) and math.isfinite(h0)):
        return _incomplete("参照文件没有 meta 或 hybrid 读数")
    if meta.get("device") not in (None, "cpu"):
        return _incomplete(f"参照不是 CPU 算的：{meta.get('device')}")
    try:
        rec = json.loads((run_dir / "run.json").read_text())
        rows = [json.loads(x) for x in (run_dir / "metrics.jsonl").read_text().splitlines() if x.strip()]
    except (OSError, ValueError) as e:
        return _incomplete(f"读不了运行记录：{e}")
    d = rec.get("config", {}).get("data", {})
    have = {**{k: d.get(k) for k in META_KEYS}, "build_id": rec.get("data_build_id")}
    bad = [k for k in META_KEYS if have.get(k) != meta.get(k)]
    if bad:
        return _incomplete("口径对不上：" + ", ".join(f"{k} 运行 {have.get(k)!r} ≠ 参照 {meta.get(k)!r}" for k in bad))
    hits = [r["dev"].get("hybrid") for r in rows if r.get("step") == step and isinstance(r.get("dev"), dict)]
    if not hits or not (isinstance(hits[-1], (int, float)) and math.isfinite(hits[-1])):
        return _incomplete(f"没有第 {step} 步的开发份 hybrid 读数")
    h = float(hits[-1])
    ok = h < h0 - tol
    return {"verdict": "pass" if ok else "fail", "pass": ok, "step": step, "dev_hybrid": h, "step0_hybrid": h0,
            "tol": tol, "drop": h0 - h,
            "rule": f"第 {step} 步开发份 hybrid < 第 0 步（CPU 参照）− {tol:g}；数据字段逐项相同（大总管定 2026-11-25）"}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="300 步开发份读数低于第 0 步")
    ap.add_argument("run_dir")
    ap.add_argument("--known-dev", required=True)
    ap.add_argument("--step", type=int, default=300)
    ap.add_argument("--tol", type=float, default=1e-3)
    a = ap.parse_args(argv)
    try:
        known = json.loads(Path(a.known_dev).read_text())
    except (OSError, ValueError) as e:
        r = _incomplete(f"读不了参照：{e}")
    else:
        r = check(Path(a.run_dir), known, a.step, a.tol) if math.isfinite(a.tol) and a.tol >= 0 else \
            _incomplete(f"容差非法：{a.tol}")
    print(json.dumps(r, ensure_ascii=False, indent=2))
    return {"pass": 0, "fail": 1}.get(r["verdict"], 2)


if __name__ == "__main__":
    sys.exit(main())
