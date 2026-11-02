"""exp009 · 第二段事后诊断（预注册之外，结论只作线索）：作者与机器的分歧从哪来。

1. 三个评分模型在作者那 48 对上彼此一致多少——机器之间打架，还是作者与所有机器都不同
2. 作者答卷里各模型的胜负，与各评分模型的平均分排名对照

用法：<主目录>/_tmp/exp009-venv/bin/python docs/experiments/exp009-more-ears/diagnose_author.py
输入：results/exp003_predictors.json（score_author.py 生成）· exp003 答卷
输出：results/author_diagnosis.json
"""
import collections
import itertools
import json
from pathlib import Path

import numpy as np

from score_author import ANSWERS, MODELS, extract_pairs

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
    (HERE / "results/author_diagnosis.json").write_text(json.dumps(R, ensure_ascii=False, indent=1))
    print(json.dumps(R, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
