"""exp006 第三步：评 Rust 场景分析（带平滑与滞回）在 test 与 seq 上的表现。

读 scene_offline 的输出（每 ≈0.5 s 一行），对照 truth.jsonl：
- **逐窗准确率**（t ≥ 2 s）：原始后验最大 / 平滑后最大 / 滞回后已生效场景，分细场景大类与用户层两级；混淆矩阵（已生效，大类）
- **设备**：窄带蓝牙、削波的检出率与误报率（按段末状态）
- **抖动**（test 组，每段 10 s 背景不变）：第一次生效之后又换了几次
- **切换**（seq 组，每条 60 s、背景换 3 次）：多余切换次数、每次真换场景后多久生效到正确的大类（切换延迟）、正确生效的时间占比

用法：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp006-scene-sim/eval.py <build> [--tag 名字] [--feat-suffix 后缀]
输出：打印 JSON；--tag 给了就写 docs/experiments/exp006-scene-sim/builds/<build>/eval_<tag>.json
"""
import argparse
import json
from collections import defaultdict
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
RECIPES = json.loads((HERE / "recipes.json").read_text())
CLASSES = list(RECIPES["fine"])
USER = {c: RECIPES["fine"][c]["user"] for c in CLASSES}
USERS = ["quiet", "cafe", "commute", "music"]


def cls_of_fine_id(fid):
    if fid <= 0 or fid > 21:
        return None
    return CLASSES[(fid - 1) // 3]


def load(build, split, suffix=""):
    truth = {json.loads(l)["id"]: json.loads(l) for l in (HERE / "builds" / build / "truth.jsonl").read_text().splitlines()}
    rows = defaultdict(list)
    for l in (ROOT / "_tmp/20261031-exp006" / build / f"feat_{split}{suffix}.jsonl").read_text().splitlines():
        r = json.loads(l)
        rows[r["id"]].append(r)
    return truth, rows


def eval_test(truth, rows):
    n = 0
    n5, hit5, hit5_u = 0, 0, 0
    hit = {"raw": 0, "smooth": 0, "committed": 0}
    hit_u = {"raw": 0, "smooth": 0, "committed": 0}
    conf = np.zeros((len(CLASSES), len(CLASSES)), int)
    conf_u = np.zeros((len(USERS), len(USERS)), int)
    switches = []
    dev_tp = defaultdict(int)
    dev_n = defaultdict(int)
    dev_fp = defaultdict(int)
    n_clips = 0
    for cid, rs in rows.items():
        t = truth[cid]
        y = CLASSES.index(t["fine"])
        n_clips += 1
        sw = sum(1 for r in rs if r["switched"]) - 1  # 第一次生效不算抖动
        switches.append(max(sw, 0))
        last = rs[-1]
        dev_n[t["device"]] += 1
        for d in ("bt_narrow", "clip"):
            if last["device"] == d:
                if t["device"] == d:
                    dev_tp[d] += 1
                else:
                    dev_fp[d] += 1
        for r in rs:
            if r["t"] < 2.0:
                continue
            n += 1
            preds = {"raw": int(np.argmax(r["raw"])), "smooth": int(np.argmax(r["smooth"])),
                     "committed": CLASSES.index(cls_of_fine_id(r["committed_fine"])) if cls_of_fine_id(r["committed_fine"]) else -1}
            for k, p in preds.items():
                hit[k] += p == y
                hit_u[k] += p >= 0 and USER[CLASSES[p]] == USER[CLASSES[y]]
            if r["t"] >= 5.0:
                n5 += 1
                hit5 += preds["committed"] == y
                hit5_u += preds["committed"] >= 0 and USER[CLASSES[preds["committed"]]] == USER[CLASSES[y]]
            if preds["committed"] >= 0:
                conf[y, preds["committed"]] += 1
                conf_u[USERS.index(USER[CLASSES[y]]), USERS.index(USER[CLASSES[preds["committed"]]])] += 1
    sw = np.array(switches)
    neg = {d: n_clips - dev_n[d] for d in ("bt_narrow", "clip")}
    return {
        "windows": n,
        "acc_fine_class": {k: round(v / n, 4) for k, v in hit.items()},
        "acc_user_scene": {k: round(v / n, 4) for k, v in hit_u.items()},
        "acc_committed_after_5s": {"fine_class": round(hit5 / max(n5, 1), 4), "user_scene": round(hit5_u / max(n5, 1), 4)},
        "per_class_recall_committed": {c: round(conf[i, i] / max(conf[i].sum(), 1), 3) for i, c in enumerate(CLASSES)},
        "confusion_committed": {"labels": CLASSES, "rows_truth_cols_pred": conf.tolist()},
        "confusion_user_committed": {"labels": USERS, "rows_truth_cols_pred": conf_u.tolist()},
        "jitter_test": {"clips": len(sw), "mean_extra_switches": round(float(sw.mean()), 3), "clips_with_any": int((sw > 0).sum()),
                        "max": int(sw.max()) if len(sw) else 0},
        "device": {d: {"recall": round(dev_tp[d] / max(dev_n[d], 1), 3), "false_alarm": round(dev_fp[d] / max(neg[d], 1), 3)} for d in ("bt_narrow", "clip")},
    }


def eval_seq(truth, rows):
    extra, delays, correct_time, total_time, missed = [], [], 0.0, 0.0, 0
    for cid, rs in rows.items():
        tl = truth[cid]["timeline"]

        def true_at(t):
            for seg in tl:
                if seg["start_s"] <= t < seg["end_s"]:
                    return seg["fine"]
            return tl[-1]["fine"]

        n_sw = sum(1 for r in rs if r["switched"])
        extra.append(n_sw - len(tl))  # 理想：开头 1 次 + 每次换场景 1 次 = len(tl)
        for seg in tl[1:]:
            d = None
            for r in rs:
                if r["t"] > seg["start_s"] and r["t"] <= seg["end_s"] and cls_of_fine_id(r["committed_fine"]) == seg["fine"]:
                    d = r["t"] - seg["start_s"]
                    break
            if d is None:
                missed += 1
            else:
                delays.append(d)
        for r in rs:
            total_time += 0.496
            correct_time += 0.496 * (cls_of_fine_id(r["committed_fine"]) == true_at(r["t"] - 0.25))
    d = np.array(delays) if delays else np.array([np.nan])
    return {"sequences": len(rows), "extra_switches_mean": round(float(np.mean(extra)), 2), "extra_switches_total": int(np.sum(extra)),
            "change_delay_s": {"median": round(float(np.median(d)), 2), "p90": round(float(np.percentile(d, 90)), 2), "n": len(delays)},
            "changes_never_reached_correct": missed, "time_correct_frac": round(correct_time / max(total_time, 1e-9), 4)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("build")
    ap.add_argument("--tag", default=None)
    ap.add_argument("--feat-suffix", default="")
    a = ap.parse_args()
    truth, rows = load(a.build, "test", a.feat_suffix)
    out = {"test": eval_test(truth, rows)}
    truth, rows = load(a.build, "seq", a.feat_suffix)
    out["seq"] = eval_seq(truth, rows)
    if a.tag:
        (HERE / "builds" / a.build / f"eval_{a.tag}.json").write_text(json.dumps(out, indent=1, ensure_ascii=False))
    print(json.dumps(out, ensure_ascii=False))


if __name__ == "__main__":
    main()
