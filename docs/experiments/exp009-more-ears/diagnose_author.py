"""exp009 · 第二段事后诊断（预注册之外，结论只作线索）：作者与机器的分歧从哪来。

1. 三个评分模型在作者那 48 对上彼此一致多少——机器之间打架，还是作者与所有机器都不同
2. 作者答卷里各模型的胜负，与各评分模型的平均分排名对照
3. 敏感性分析（评审后加，非预注册）：作者 48 对去重后按「同屏同一对模型」只算一次；
   参照分布改成与作者同构——一个 CHiME-7 固定听众 × 9 屏 × 每屏同一组 3 个条件的 3 对，4000 次

用法：<主目录>/_tmp/exp009-venv/bin/python docs/experiments/exp009-more-ears/diagnose_author.py
输入：results/exp003_predictors.json（score_author.py 生成）· exp003 答卷
输出：results/author_diagnosis.json
"""
import collections
import itertools
import json
from pathlib import Path

import numpy as np

from score_author import ANSWERS, MODELS, SEED, extract_pairs

HERE = Path(__file__).resolve().parent
KEYS = ["scoreq", "distill_mos", "OVRL"]


def sign_agree(S, pairs, a, b):
    v = []
    for x in pairs:
        c, w, l = x[0], x[1], x[2]
        sa, sb = np.sign(S[c][w][a] - S[c][l][a]), np.sign(S[c][w][b] - S[c][l][b])
        if sa and sb:
            v.append(bool(sa == sb))
    return {"agree": round(float(np.mean(v)), 3), "n": len(v)}


def dedup(pairs):
    """同屏同一对模型只留一次，方向按多数票；票数打平的丢掉。"""
    votes = collections.defaultdict(int)
    for x in pairs:
        c, w, l = x[0], x[1], x[2]
        a, b = sorted([w, l])
        votes[(c, a, b)] += 1 if w == a else -1
    return [(c, a, b) if v > 0 else (c, b, a) for (c, a, b), v in votes.items() if v != 0]


def sensitivity(S, pairs):
    uniq = dedup(pairs)
    R = {"distinct_pairs": len(uniq), "screens": len({u[0] for u in uniq})}
    au = {}
    for k in KEYS:
        v = [bool(S[c][w][k] > S[c][l][k]) for c, w, l in uniq if S[c][w][k] != S[c][l][k]]
        au[k] = {"agree": round(float(np.mean(v)), 3), "n": len(v)}
    R["author_vs_on_distinct"] = au
    R["machine_vs_machine_on_distinct"] = {f"{a}~{b}": sign_agree(S, uniq, a, b) for a, b in itertools.combinations(KEYS, 2)}
    dis = collections.Counter(tuple(sorted([w, l])) for c, w, l in uniq if S[c][w]["scoreq"] < S[c][l]["scoreq"])
    R["scoreq_disagreements_by_model_pair_distinct"] = {"~".join(k): n for k, n in dis.most_common()}

    import crowd_ears as C
    ds = C.load_chime()
    L = sorted({l for d in ds.raw for l in d})
    M = np.full((len(ds.uids), len(L)), np.nan)
    for i, d in enumerate(ds.raw):
        for l, v in d.items():
            M[i, L.index(l)] = v
    by = collections.defaultdict(dict)
    for i, (src, sysn) in enumerate(zip(ds.src, ds.sys)):
        by[src][sysn] = i
    conds = sorted(set(ds.sys))
    x = ds.preds["scoreq"]
    rng = np.random.default_rng(SEED)
    n_scr = R["screens"]
    sims = []
    for _ in range(4000):
        y = M[:, rng.integers(len(L))]
        T = list(rng.choice(conds, 3, replace=False))
        cand = [s for s, m in by.items() if all(t in m and not np.isnan(y[m[t]]) for t in T)]
        if len(cand) < n_scr:
            continue
        ok = []
        for s in rng.choice(cand, n_scr, replace=False):
            for a, b in itertools.combinations(T, 2):
                i, j = by[s][a], by[s][b]
                dy, dx = y[i] - y[j], x[i] - x[j]
                if dy and dx:
                    ok.append(np.sign(dy) == np.sign(dx))
        if ok:
            sims.append(float(np.mean(ok)))
    o = np.array(sims)
    a = au["scoreq"]["agree"]
    R["isomorphic_reference_scoreq"] = {
        "design": f"一个 CHiME-7 固定听众 × {n_scr} 屏 × 同一组 3 个条件的 3 对（去重计数），剔平局，4000 次，种子 {SEED}",
        "valid_sims": len(o),
        "p2.5": round(float(np.percentile(o, 2.5)), 3), "p50": round(float(np.percentile(o, 50)), 3),
        "p97.5": round(float(np.percentile(o, 97.5)), 3),
        "author_distinct_agree": a, "share_le_author": round(float((o <= a).mean()), 3),
        "author_below_p2.5": bool(a < np.percentile(o, 2.5)),
    }
    return R


def main():
    S = json.loads((HERE / "results/exp003_predictors.json").read_text())
    pairs = extract_pairs(json.loads(ANSWERS.read_text()))
    every = [(c, x, y) for c in S for x, y in itertools.combinations(["noisy"] + MODELS, 2)]
    R = {
        "machine_vs_machine_on_author_pairs": {f"{a}~{b}": sign_agree(S, pairs, a, b) for a, b in itertools.combinations(KEYS, 2)},
        "machine_vs_machine_all_pairs": {f"{a}~{b}": sign_agree(S, every, a, b) for a, b in itertools.combinations(KEYS, 2)},
        "author_win_loss": {m: {"win": sum(p[1] == m for p in pairs), "loss": sum(p[2] == m for p in pairs)} for m in MODELS},
        "author_pair_counts": {f"{w}>{l}": n for (w, l), n in collections.Counter((p[1], p[2]) for p in pairs).most_common()},
        "machine_mean_scores": {m: {k: round(float(np.mean([S[c][m][k] for c in S])), 2) for k in KEYS} for m in ["noisy"] + MODELS},
    }
    R.update(sensitivity(S, pairs))
    (HERE / "results/author_diagnosis.json").write_text(json.dumps(R, ensure_ascii=False, indent=1))
    print(json.dumps(R, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
