#!/usr/bin/env python3
"""exp020 离线排序模拟：几种「只由 DSP verifier 组成」的奖励，给同一段录音上的几个系统排序，
和两个留出的评分模型（Distill-MOS · SCOREQ，D81：只打分、不当奖励）的排序比一致性。

不跑任何模型，只读成绩单已有的逐段读数（clips.jsonl）。用法：
    python3 reward_rank.py <runs.json> <输出.json>
runs.json：{"系统名": "clips.jsonl 的路径（相对 runs.json 所在目录或仓库根）", ...}
同一系统在多个文件里出现时取第一份。只保留五个系统都有读数的段。
"""

import json
import math
import random
import sys
from pathlib import Path

JUDGES = ["V-NRMOS-DISTILL", "V-NRMOS-SCOREQ"]  # 留出的裁判，不进任何奖励
# 一票否决（照 v2 规则与 013 槽位）：吃字、人声被压低超线、数值失效
VETO = {"V-EAT-ASR": ("le", 1.0), "V-SPLVL": ("ge", -3.0), "V-NAN": ("le", 0.0)}
# 约束（「不许比 GTCRN 差」的守住项）：字头、背景跳变、音色
GUARDS = {"V-ONSET": 1.0, "V-BGSTAB": 2.0, "V-TIMBRE": 1.0}  # 容许比基线多退的量（dB）〔建议值〕
BASE = "gtcrn"


def val(row, v):
    for s in row["scores"]:
        if s["v"] == v:
            return s["value"]
    return None


def load(runs_json):
    root = Path(runs_json).resolve().parent
    spec = json.loads(Path(runs_json).read_text())
    table = {}  # clip -> system -> row
    for _, path in spec.items():
        p = Path(path)
        if not p.is_absolute():
            cand = root / p
            p = cand if cand.exists() else Path.cwd() / p
        for line in p.read_text().splitlines():
            r = json.loads(line)
            table.setdefault(r["clip"], {}).setdefault(r["system"], r)
    systems = sorted({s for c in table.values() for s in c})
    full = {c: d for c, d in table.items() if len(d) == len(systems)}
    return systems, full


# ---- 奖励 ----
def r_nr_only(row, base):
    """只看降噪量：越压越高分（已知可刷，作反例）。"""
    x = val(row, "V-NR")
    return None if x is None else -x


def r_estoi_only(row, base):
    return val(row, "V-ESTOI")


def vetoed(row):
    for v, (op, t) in VETO.items():
        x = val(row, v)
        if x is None:
            continue
        if (op == "le" and x > t) or (op == "ge" and x < t):
            return True
    return False


def guard_penalty(row, base):
    """守住项：比 GTCRN 多退超过容许量，每超 1 dB 扣 1 个奖励单位（0.01 ESTOI）。"""
    pen = 0.0
    for v, tol in GUARDS.items():
        x, b = val(row, v), val(base, v)
        if x is None or b is None:
            continue
        pen += max(0.0, (x - b) - tol)
    return pen


def r_composite(row, base):
    """主奖励 = ESTOI 提升 + 0.002 × 降噪量（dB，封顶 12）；否决 → −1；守住项超线按量扣。"""
    e, nr = val(row, "V-ESTOI"), val(row, "V-NR")
    if e is None:
        return None
    if vetoed(row):
        return -1.0
    nr_gain = 0.0 if nr is None else min(12.0, max(0.0, -nr))
    return e + 0.002 * nr_gain - 0.01 * guard_penalty(row, base)


REWARDS = {"nr_only": r_nr_only, "estoi_only": r_estoi_only, "composite": r_composite}


# ---- 一致性 ----
def kendall_tau(a, b):
    """Kendall τ-b（处理并列）；不足 2 个有效对时返回 None。"""
    n = len(a)
    conc = disc = ta = tb = 0
    for i in range(n):
        for j in range(i + 1, n):
            da, db = a[i] - a[j], b[i] - b[j]
            if da == 0 and db == 0:
                continue
            if da == 0:
                ta += 1
            elif db == 0:
                tb += 1
            elif (da > 0) == (db > 0):
                conc += 1
            else:
                disc += 1
    den = math.sqrt((conc + disc + ta) * (conc + disc + tb))
    return None if den == 0 else (conc - disc) / den


def boot_ci(xs, groups, reps=2000, seed=20261105):
    """按场景整组重抽的 bootstrap 95% 区间（同场景的段不独立）。"""
    rng = random.Random(seed)
    by = {}
    for x, g in zip(xs, groups):
        by.setdefault(g, []).append(x)
    keys = sorted(by)
    means = []
    for _ in range(reps):
        pick = [x for k in (rng.choice(keys) for _ in keys) for x in by[k]]
        means.append(sum(pick) / len(pick))
    means.sort()
    return means[int(0.025 * reps)], means[int(0.975 * reps) - 1]


def analyse(systems, table):
    out = {"systems": systems, "clips": len(table), "rewards": {}}
    for rn, rf in REWARDS.items():
        per = {j: ([], []) for j in JUDGES}
        top_onset_worse = top_n = 0
        sys_sum = {s: [0.0, 0] for s in systems}
        for clip, d in table.items():
            base = d[BASE]
            rs = [rf(d[s], base) for s in systems]
            if any(x is None for x in rs):
                continue
            for s, x in zip(systems, rs):
                sys_sum[s][0] += x
                sys_sum[s][1] += 1
            for j in JUDGES:
                js = [val(d[s], j) for s in systems]
                if any(x is None for x in js):
                    continue
                t = kendall_tau(rs, js)
                if t is not None:
                    per[j][0].append(t)
                    per[j][1].append(d[BASE]["scene"])
            # 讨好检查：奖励最高的系统，字头比 GTCRN 多退 > 1 dB 的比例
            top = systems[max(range(len(rs)), key=lambda i: rs[i])]
            o, ob = val(d[top], "V-ONSET"), val(base, "V-ONSET")
            if o is not None and ob is not None:
                top_n += 1
                top_onset_worse += (o - ob) > 1.0
        res = {}
        for j, (ts, gs) in per.items():
            lo, hi = boot_ci(ts, gs)
            res[j] = {"mean_tau": sum(ts) / len(ts), "ci95": [lo, hi], "n_clips": len(ts),
                      "n_scenes": len(set(gs))}
        res["top_onset_worse_frac"] = top_onset_worse / top_n if top_n else None
        res["system_mean_reward"] = {s: v[0] / v[1] for s, v in sys_sum.items() if v[1]}
        out["rewards"][rn] = res
    # 系统级：奖励均值排序 vs 裁判均值排序（只有 len(systems) 个点）
    jm = {}
    for j in JUDGES:
        jm[j] = {}
        for s in systems:
            xs = [val(d[s], j) for d in table.values()]
            xs = [x for x in xs if x is not None]
            jm[j][s] = sum(xs) / len(xs)
    out["system_mean_judge"] = jm
    for rn in REWARDS:
        mr = out["rewards"][rn]["system_mean_reward"]
        for j in JUDGES:
            a = [mr[s] for s in systems]
            b = [jm[j][s] for s in systems]
            out["rewards"][rn][f"system_tau_{j}"] = kendall_tau(a, b)
    return out


def main(argv):
    if len(argv) != 3:
        print(__doc__)
        return 2
    systems, table = load(argv[1])
    res = analyse(systems, table)
    Path(argv[2]).write_text(json.dumps(res, ensure_ascii=False, indent=1) + "\n")
    for rn, r in res["rewards"].items():
        line = " ".join(f"{j[8:]} τ={r[j]['mean_tau']:+.3f} [{r[j]['ci95'][0]:+.3f},{r[j]['ci95'][1]:+.3f}]"
                        for j in JUDGES)
        print(f"{rn:11s} {line} 字头退>1dB={r['top_onset_worse_frac']:.2f}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
