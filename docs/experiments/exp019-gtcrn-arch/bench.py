"""测每帧推理耗时：单线程、流式、一次一帧（16 ms 一帧，GTCRN 的实际节奏）。

两种跑法（019 / 020 的教训：两种不能混比）：
- burst：连续推帧，只比相对快慢
- paced：每 16 ms 推一帧，接近真实节奏（帧间空闲会降频，单帧更慢）

各候选轮流跑（base, A, base, B ...），减少机器状态漂移。每次跑写一个新目录，不覆盖旧的。
用法：python bench.py --mode burst --frames 500 --rounds 3 --out runs/
"""
import argparse
import json
import platform
import secrets
import statistics
import subprocess
import time
from datetime import datetime
from pathlib import Path

import torch

import model
import variants

HOP_S = 0.016


def machine_info():
    def sh(*cmd):
        try:
            return subprocess.run(cmd, capture_output=True, text=True, timeout=5).stdout.strip()
        except Exception as e:  # 拿不到就记下原因，不挡测速
            return f"error: {e}"
    return {
        "torch": torch.__version__,
        "threads": torch.get_num_threads(),
        "python": platform.python_version(),
        "machine": platform.machine(),
        "cpu": sh("sysctl", "-n", "machdep.cpu.brand_string"),
        "os": sh("sw_vers", "-productVersion"),
        "power": sh("pmset", "-g", "batt").splitlines()[0] if True else "",
    }


def run_one(m, frames, paced, warmup=100):
    st = m.init_state(1)
    x = torch.randn(frames + warmup, 1, 257, 1, 2) * 0.1
    times = []
    with torch.inference_mode():
        nxt = time.perf_counter()
        for i in range(frames + warmup):
            if paced:
                nxt += HOP_S
            t0 = time.perf_counter_ns()
            _, st = m(x[i], st)
            dt = time.perf_counter_ns() - t0
            if i >= warmup:
                times.append(dt / 1e6)
            if paced:
                rest = nxt - time.perf_counter()
                if rest > 0:
                    time.sleep(rest)
    times.sort()
    q = lambda p: times[min(len(times) - 1, int(p * len(times)))]
    return {"n": len(times), "mean_ms": statistics.fmean(times),
            "p50_ms": q(0.5), "p90_ms": q(0.9), "p99_ms": q(0.99), "max_ms": times[-1]}


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=["burst", "paced"], default="burst")
    ap.add_argument("--frames", type=int, default=500)
    ap.add_argument("--rounds", type=int, default=3)
    ap.add_argument("--names", default=",".join(variants.NAMES))
    ap.add_argument("--out", default="runs")
    a = ap.parse_args(argv)

    torch.set_num_threads(1)
    names = a.names.split(",")
    models = {n: model.make(n) for n in names}
    run_id = datetime.now().strftime("%Y%m%d-%H%M%S") + "-" + a.mode + "-" + secrets.token_hex(3)
    out = Path(a.out) / run_id
    out.mkdir(parents=True, exist_ok=False)
    info = machine_info() | {"mode": a.mode, "frames": a.frames, "rounds": a.rounds,
                             "hop_ms": HOP_S * 1000, "run_id": run_id}
    (out / "machine.json").write_text(json.dumps(info, ensure_ascii=False, indent=1))
    with open(out / "bench.jsonl", "w") as f:
        for r in range(a.rounds):
            for n in names:
                rec = {"round": r, "name": n} | run_one(models[n], a.frames, a.mode == "paced")
                f.write(json.dumps(rec) + "\n")
                f.flush()
                print(json.dumps(rec))
    print("写入", out)
    return out


if __name__ == "__main__":
    main()
