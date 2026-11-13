"""exp022 · 评估器对人耳的混淆矩阵统计（纯标准库，可单测）。

约定（照 Hamel《evals-faq》）：「阳性」= 有缺陷 = 不过。人耳是真值。
- 真阳性率 TPR = 人说不过的里面，评估器也判不过的比例（抓到坏的）
- 真阴性率 TNR = 人说过的里面，评估器也判过的比例（不冤枉好的）
- 评估器判「待测」的条目不进 TPR/TNR，单独报覆盖率
"""
import math
import random

FAIL, PASS, UNDECIDED = "fail", "pass", "undecided"


def wilson(k, n, z=1.959964):
    """二项比例的 Wilson 95% 区间；n = 0 返回 (nan, nan)。"""
    if n == 0:
        return (float("nan"), float("nan"))
    p = k / n
    den = 1 + z * z / n
    mid = (p + z * z / (2 * n)) / den
    half = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / den
    return (max(0.0, mid - half), min(1.0, mid + half))


def confusion(items):
    """items: [(human, judge)]，human ∈ {fail, pass}，judge ∈ {fail, pass, undecided}。"""
    c = {"tp": 0, "fn": 0, "tn": 0, "fp": 0, "und_hfail": 0, "und_hpass": 0}
    for h, j in items:
        if h not in (FAIL, PASS) or j not in (FAIL, PASS, UNDECIDED):
            raise ValueError(f"bad label {h!r} / {j!r}")
        if j == UNDECIDED:
            c["und_hfail" if h == FAIL else "und_hpass"] += 1
        elif h == FAIL:
            c["tp" if j == FAIL else "fn"] += 1
        else:
            c["tn" if j == PASS else "fp"] += 1
    return c


def ratio(k, n):
    return k / n if n else float("nan")


def rates(c):
    """混淆矩阵 → TPR · TNR（带 Wilson 区间）· 总一致率 · 「全判过」基线一致率 · 平衡准确率。"""
    nf, np_ = c["tp"] + c["fn"], c["tn"] + c["fp"]
    n = nf + np_
    tpr, tnr = ratio(c["tp"], nf), ratio(c["tn"], np_)
    return {
        "n_decided": n,
        "n_undecided": c["und_hfail"] + c["und_hpass"],
        "human_fail": nf,
        "human_pass": np_,
        "tpr": tpr,
        "tpr_ci95": wilson(c["tp"], nf),
        "tnr": tnr,
        "tnr_ci95": wilson(c["tn"], np_),
        "raw_agreement": ratio(c["tp"] + c["tn"], n),
        # 评估器什么都不看、一律判过时的总一致率：类别不平衡时总一致率会被它撑高
        "always_pass_agreement": ratio(np_, n),
        "balanced_accuracy": (tpr + tnr) / 2 if nf and np_ else float("nan"),
        # Youden J = TPR + TNR − 1：0 = 和抛硬币一样，≤ 0 时 Rogan-Gladen 校正无解
        "youden_j": tpr + tnr - 1 if nf and np_ else float("nan"),
    }


def rogan_gladen(judge_fail_rate, tpr, tnr):
    """按评估器误差校正「真实不过比例」：θ = (p_obs + TNR − 1) / (TPR + TNR − 1)，截到 [0, 1]。

    J = TPR + TNR − 1 ≤ 0 时评估器不比抛硬币好，校正无解，返回 None。"""
    j = tpr + tnr - 1
    if not (j > 0):
        return None
    return min(1.0, max(0.0, (judge_fail_rate + tnr - 1) / j))


def cluster_bootstrap(items_by_cluster, stat, reps=10000, seed=20261030):
    """按簇（同一片段的几个模型一起抽）重采样，返回 stat 的 2.5% / 97.5% 分位（跳过 nan）。"""
    keys = sorted(items_by_cluster)
    rng = random.Random(seed)
    vals = []
    for _ in range(reps):
        sample = []
        for _ in keys:
            sample.extend(items_by_cluster[keys[rng.randrange(len(keys))]])
        v = stat(sample)
        if v is not None and not math.isnan(v):
            vals.append(v)
    if not vals:
        return (float("nan"), float("nan"))
    vals.sort()
    q = lambda p: vals[min(len(vals) - 1, int(p * (len(vals) - 1) + 0.5))]  # noqa: E731
    return (q(0.025), q(0.975))
