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
# 一票否决（照 013 槽位）：吃字、人声被压低超线、数值失效。
# 吃字两种口径：旧 = 单段多错 > 1 个百分点（v1 成绩单的线）；D85 G-3 = 单段多错 ≥ 5 个字
VETO = {"V-EAT-ASR": ("le", 1.0), "V-SPLVL": ("ge", -3.0), "V-NAN": ("le", 0.0)}
EAT_WORDS = 5.0
# 约束（「不许比 GTCRN 差」的守住项）：字头、背景跳变、音色
GUARDS = {"V-ONSET": 1.0, "V-BGSTAB": 2.0, "V-TIMBRE": 1.0}  # 容许比基线多退的量（dB）〔建议值〕
BASE = "gtcrn"


def score(row, v):
    for s in row["scores"]:
        if s["v"] == v:
            return s
    return None


def val(row, v):
    s = score(row, v)
    return None if s is None else s["value"]


def eat_words(row):
    """吃字：比原声多错的字数 = 百分点 × 参考字数 / 100。"""
    s = score(row, "V-EAT-ASR")
    if s is None or s["value"] is None:
        return None
    n = (s.get("extra") or {}).get("ref_len")
    return None if n is None else s["value"] * n / 100.0


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


def vetoed(row, skip=(), eat_rule="pp"):
    if eat_rule == "words" and "V-EAT-ASR" not in skip:
        w = eat_words(row)
        if w is not None and w >= EAT_WORDS:
            return True
        skip = tuple(skip) + ("V-EAT-ASR",)
    for v, (op, t) in VETO.items():
        if v in skip:
            continue
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


def r_composite(row, base, skip=(), eat_rule="pp", rel_guard=True):
    """主奖励 = ESTOI 提升 + 0.002 × 降噪量（dB，封顶 12）；否决 → −1；守住项超线按量扣。"""
    e, nr = val(row, "V-ESTOI"), val(row, "V-NR")
    if e is None:
        return None
    if vetoed(row, skip, eat_rule):
        return -1.0
    nr_gain = 0.0 if nr is None else min(12.0, max(0.0, -nr))
    pen = guard_penalty(row, base) if rel_guard else 0.0
    return e + 0.002 * nr_gain - 0.01 * pen


def r_composite_no_asr(row, base):
    """同 composite，但完全不看吃字（逐段奖励里没有识别）。整批吃字只在训练中途的成绩单上查，本脚本不模拟。"""
    return r_composite(row, base, skip=("V-EAT-ASR",))


def r_composite_d85(row, base):
    """同 composite，吃字按 D85 G-3：单段多错 ≥ 5 个字才否决。"""
    return r_composite(row, base, eat_rule="words")


# 绝对阈值版守住项（不以 GTCRN 为基线，查基线敏感性）：字头 3 dB（rubric C3）· 背景跳变 4 dB（D85 G-5）· 音色 6 dB
ABS_GUARDS = {"V-ONSET": 3.0, "V-BGSTAB": 4.0, "V-TIMBRE": 6.0}


def r_composite_hard_guard(row, base):
    """不带识别 + 守住项改成硬约束：任一项比 GTCRN 多退超过容许量就否决（−1），不按量扣。"""
    if guard_penalty(row, base) > 0:
        return -1.0
    return r_composite_no_asr(row, base)


def r_composite_abs_guard(row, base):
    """不带识别 + 守住项用绝对阈值硬约束；不再按 GTCRN 扣分（`base` 不参与）。"""
    for v, t in ABS_GUARDS.items():
        x = val(row, v)
        if x is not None and x > t:
            return -1.0
    return r_composite(row, base, skip=("V-EAT-ASR",), rel_guard=False)


REWARDS = {
    "nr_only": r_nr_only,
    "estoi_only": r_estoi_only,
    "composite": r_composite,
    "composite_d85": r_composite_d85,
    "composite_no_asr": r_composite_no_asr,
    "composite_hard_guard": r_composite_hard_guard,
    "composite_abs_guard": r_composite_abs_guard,
}


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


def veto_parts(systems, table):
    """逐项否决率（各自的分母 = 该项有读数的段数）。"""
    parts = {}
    for s in systems:
        rows = [d[s] for d in table.values()]
        eat = [eat_words(r) for r in rows]
        eat = [w for w in eat if w is not None]
        spl = [val(r, "V-SPLVL") for r in rows]
        spl = [x for x in spl if x is not None]
        parts[s] = {
            "eat_ge5_words": [sum(w >= EAT_WORDS for w in eat), len(eat)],
            "eat_gt1pp": [sum(val(r, "V-EAT-ASR") > 1.0 for r in rows
                              if val(r, "V-EAT-ASR") is not None), len(eat)],
            "splvl_lt_-3": [sum(x < -3.0 for x in spl), len(spl)],
        }
    return parts


def analyse(systems, table):
    out = {"systems": systems, "clips": len(table), "rewards": {}, "veto_parts": veto_parts(systems, table)}
    for rn, rf in REWARDS.items():
        per = {j: ([], []) for j in JUDGES}
        top_onset_worse = top_n = no_feasible = 0
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
            # 讨好检查：奖励最高的系统，字头比 GTCRN 多退 > 1 dB 的比例。
            # 全部候选都被否决 → 记「无可行候选」，不参与；并列最高时取字头最好的那个（对奖励最有利的解读）
            best = max(rs)
            if best == -1.0:
                no_feasible += 1
                continue
            tied = [s for s, x in zip(systems, rs) if x == best]
            top = min(tied, key=lambda s: (val(d[s], "V-ONSET") is None, val(d[s], "V-ONSET") or 0.0))
            o, ob = val(d[top], "V-ONSET"), val(base, "V-ONSET")
            if o is not None and ob is not None:
                top_n += 1
                top_onset_worse += (o - ob) > 1.0
        res = {}
        for j, (ts, gs) in per.items():
            if not ts:
                res[j] = {"mean_tau": None, "ci95": None, "n_clips": 0, "n_scenes": 0}
                continue
            lo, hi = boot_ci(ts, gs)
            res[j] = {"mean_tau": sum(ts) / len(ts), "ci95": [lo, hi], "n_clips": len(ts),
                      "n_scenes": len(set(gs))}
        res["veto_frac"] = {s: sum(rf(d[s], d[BASE]) == -1.0 for d in table.values()) / len(table)
                            for s in systems} if rn.startswith("composite") else None
        res["top_onset_worse_frac"] = top_onset_worse / top_n if top_n else None
        res["top_onset_n"] = top_n
        res["no_feasible_clips"] = no_feasible
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
                        if r[j]["mean_tau"] is not None else f"{j[8:]} τ=无" for j in JUDGES)
        print(f"{rn:11s} {line} 字头退>1dB={r['top_onset_worse_frac']:.2f}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
