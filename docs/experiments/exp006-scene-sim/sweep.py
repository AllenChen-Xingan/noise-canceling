"""exp006 第四步：滞回与后验平滑扫参（D41：细场景逻辑靠模拟环境标定）。

对每组 (ema_s, confirm_s, min_confidence, min_dwell_s) 跑一遍 Rust 的 scene_offline（test + seq），用 eval.py 打分。
选法（写在这里，不事后挑）：先要求 seq 上「切换延迟 p90 ≤ 6 s」且「始终没切对的换场景 ≤ 10%」，
在满足的组里取 seq 正确时间占比最高者；并列时取 test 抖动（多余切换）更少者。
助听器参照（ondevice 线转述，待核）：ReSound 判定新环境 ≤ 4 s，Phonak 每 0.4 s 出一次概率。

用法：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp006-scene-sim/sweep.py <build>
输出：docs/experiments/exp006-scene-sim/builds/<build>/sweep.json（全表 + 选中组）
"""
import itertools
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE))
import eval as ev  # noqa: E402

BIN = ROOT / "target/release/examples/scene_offline"


def run(build, split, args):
    d = ROOT / "_tmp/20261031-exp006" / build
    out = subprocess.run([str(BIN), str(d / f"list_{split}.txt"), *args], capture_output=True, text=True, check=True).stdout
    rows = {}
    for line in out.splitlines():
        r = json.loads(line)
        rows.setdefault(r["id"], []).append(r)
    return rows


def main():
    build = sys.argv[1]
    truth = {json.loads(l)["id"]: json.loads(l) for l in (HERE / "builds" / build / "truth.jsonl").read_text().splitlines()}
    grid = list(itertools.product([0.5, 1.0, 1.5], [1.0, 1.5, 2.0, 3.0], [0.4, 0.5, 0.6], [3.0, 5.0]))
    res = []
    for ema, confirm, minconf, dwell in grid:
        args = ["--ema", str(ema), "--confirm", str(confirm), "--minconf", str(minconf), "--dwell", str(dwell)]
        t = ev.eval_test(truth, run(build, "test", args))
        s = ev.eval_seq(truth, run(build, "seq", args))
        res.append({"ema_s": ema, "confirm_s": confirm, "min_confidence": minconf, "min_dwell_s": dwell,
                    "seq_time_correct": s["time_correct_frac"], "seq_delay_median": s["change_delay_s"]["median"],
                    "seq_delay_p90": s["change_delay_s"]["p90"], "seq_never": s["changes_never_reached_correct"],
                    "seq_extra_switches": s["extra_switches_total"], "test_jitter_mean": t["jitter_test"]["mean_extra_switches"],
                    "test_committed_after_5s": t["acc_committed_after_5s"]["fine_class"],
                    "test_committed_after_5s_user": t["acc_committed_after_5s"]["user_scene"]})
        print(json.dumps(res[-1]))
    n_changes = 3 * len([k for k in truth if k.startswith("sq")])
    ok = [r for r in res if r["seq_delay_p90"] <= 6.0 and r["seq_never"] <= 0.1 * n_changes]
    pool = ok or res
    best = max(pool, key=lambda r: (r["seq_time_correct"], -r["test_jitter_mean"]))
    (HERE / "builds" / build / "sweep.json").write_text(json.dumps({"rule": __doc__.split("选法")[1].split("\n")[0], "met_constraints": bool(ok),
                                                                   "best": best, "all": res}, indent=1, ensure_ascii=False))
    print("BEST", json.dumps(best))


if __name__ == "__main__":
    main()
