"""exp009 · 用公开众包逐条评分，量「机器分 vs 众人」「单人 vs 众人」「要几人几条才够」。

只读 label 文本（urgent-challenge/vmc2026-track1-meta，固定 revision），不碰音频，不跑模型推理。
数据放仓外：~/vdn-data/public-mos/vmc2026-track1-meta/<rev>/<子集>/（fetch_labels.sh 下载并记哈希）。

用法：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp009-more-ears/crowd_ears.py [--out results/<名>.json]

口径（尽量对齐 exp007 定标 src/eval/src/calib.rs）：
- 配对 = 同一条源句（同一条带噪输入）的两个不同系统输出；方向一致率（剔除任一边差为 0 的配对）
- 区间 = 按源句聚类 bootstrap（种子 20261030）
- 「|Δ众人|≥1」子集 ≈ exp003「H3 同屏差 ≥ 1 级」
"""
import argparse
import itertools
import json
import random
import statistics as st
from collections import defaultdict
from pathlib import Path

import numpy as np
from scipy.stats import kendalltau, spearmanr

REV = "ebe09d8891b7dc1f77ddf1772e42bb22de3ef500"
DATA = Path.home() / "vdn-data/public-mos/vmc2026-track1-meta" / REV
SEED = 20261030
BOOT = 2000
PREDICTORS = ["dnsmos_ovrl", "utmos", "scoreq", "nisqa_mos", "distill_mos"]
AUTHOR = {"dnsmos_ovrl": 0.48, "n_pairs": 48, "n_screens": 12}  # exp007 calib 第 1 轮（作者一人 · exp003）

# 子集 → (说明, 许可)
SUBSETS = {
    "urgent2024-sqa/test": ("URGENT 2024 盲测 · 英语 · 23 系统 · 每条 8 票（P.808 众包）", "CC BY-NC-SA 4.0：只评测"),
    "urgent2025-sqa/test": ("URGENT 2025 盲测 · 多语 · 26 系统 · 只有均值", "随源集（NC）：只评测"),
    "tmhint-qi/test": ("TMHINT-QI 测试 · 中文 · 老式 DNN 增强 · 每条 3–11 票", "见 B 笔记 1.5"),
    "chime-7-udase-eval/test": ("CHiME-7 UDASE · 真实多人嘈杂 · 5 条件 · 每条 24 票（实验室 P.835 OVRL）", "CC BY-SA 4.0"),
}


def read_scp(p):
    d = {}
    if not p.exists():
        return d
    for line in p.read_text().splitlines():
        k, v = line.split(maxsplit=1)
        d[k] = v.strip()
    return d


def group_key(subset, uid, sysname, ref):
    """返回 (源句键, 系统键)；None = 不参与配对。"""
    if subset.startswith("tmhint"):
        if sysname.startswith("clean"):
            return None
        method, cond = sysname.split("-SNR", 1)
        return f"{ref}|SNR{cond}", method
    if subset.startswith("chime"):
        s, src = uid.split(":", 1)
        return src, s
    return ref, sysname


def load(subset):
    d = DATA / subset
    mos = {k: float(v) for k, v in read_scp(d / "mos_overall.scp").items()}
    sysm = read_scp(d / "utt2system")
    refm = read_scp(d / "utt2reference")
    raw = {}
    if (d / "meta.jsonl").exists():
        for line in (d / "meta.jsonl").open():
            j = json.loads(line)
            r = [x[1] for x in j.get("mos_overall", []) if x[1] is not None]
            if r:
                raw[j["uid"]] = r
    preds = {p: {k: float(v) for k, v in read_scp(d / f"{p}.scp").items()} for p in PREDICTORS}
    clips = {}
    for uid, m in mos.items():
        g = group_key(subset, uid, sysm.get(uid, uid.split(":")[0]), refm.get(uid, uid.split(":", 1)[-1]))
        if g is None:
            continue
        clips[uid] = {"src": g[0], "sys": g[1], "mos": m, "raw": raw.get(uid), **{p: preds[p].get(uid) for p in PREDICTORS}}
    by_src = defaultdict(list)
    for uid, c in clips.items():
        by_src[c["src"]].append(uid)
    pairs = [(a, b) for lst in by_src.values() for a, b in itertools.combinations(sorted(lst), 2)
             if clips[a]["sys"] != clips[b]["sys"]]
    return clips, pairs


def agree_flags(pairs, clips, fa, fb):
    """fa / fb(uid) → 分数；返回 [(源句, 是否一致)]，剔除平局。"""
    out = []
    for a, b in pairs:
        xa, xb, ya, yb = fa(a), fb(a), fa(b), fb(b)
        if None in (xa, xb, ya, yb):
            continue
        dx, dy = xa - ya, xb - yb
        if dx == 0 or dy == 0:
            continue
        out.append((clips[a]["src"], (dx > 0) == (dy > 0)))
    return out


def cluster_ci(flags, rng):
    if not flags:
        return None
    by = defaultdict(list)
    for s, f in flags:
        by[s].append(f)
    keys = list(by)
    sums = np.array([sum(by[k]) for k in keys], float)
    ns = np.array([len(by[k]) for k in keys], float)
    idx = rng.integers(0, len(keys), size=(BOOT, len(keys)))
    boots = sums[idx].sum(1) / ns[idx].sum(1)
    return {"n": int(ns.sum()), "agree": round(float(sums.sum() / ns.sum()), 3),
            "ci95": [round(float(np.percentile(boots, 2.5)), 3), round(float(np.percentile(boots, 97.5)), 3)]}


def analyse(subset):
    rng = np.random.default_rng(SEED)
    pyr = random.Random(SEED)
    clips, pairs = load(subset)
    res = {"desc": SUBSETS[subset][0], "license": SUBSETS[subset][1], "clips": len(clips), "pairs": len(pairs),
           "sources": len({c["src"] for c in clips.values()}), "systems": len({c["sys"] for c in clips.values()})}
    crowd = lambda u: clips[u]["mos"]
    big = [(a, b) for a, b in pairs if abs(clips[a]["mos"] - clips[b]["mos"]) >= 1]

    # A · 机器分 vs 众人均值（配对一致率）+ 语句级 / 系统级相关
    A = {}
    for p in PREDICTORS:
        f = lambda u, p=p: clips[u][p]
        if all(clips[u][p] is None for u in clips):
            continue
        us = [u for u in clips if clips[u][p] is not None]
        sysm = defaultdict(lambda: [[], []])
        for u in us:
            sysm[clips[u]["sys"]][0].append(clips[u]["mos"])
            sysm[clips[u]["sys"]][1].append(clips[u][p])
        sx = [st.mean(v[0]) for v in sysm.values()]
        sy = [st.mean(v[1]) for v in sysm.values()]
        A[p] = {"all": cluster_ci(agree_flags(pairs, clips, f, crowd), rng),
                "big_gap": cluster_ci(agree_flags(big, clips, f, crowd), rng),
                "utt_srcc": round(float(spearmanr([clips[u]["mos"] for u in us], [clips[u][p] for u in us])[0]), 3),
                "sys_srcc": round(float(spearmanr(sx, sy)[0]), 3),
                "sys_ktau": round(float(kendalltau(sx, sy)[0]), 3)}
    res["A_predictor_vs_crowd"] = A

    has_raw = sum(1 for c in clips.values() if c["raw"] and len(c["raw"]) >= 2) > 0.8 * len(clips)
    if not has_raw:
        res["note"] = "无逐人评分：只做 A"
        return res

    # B · 「一个人当裁判」：每条随机抽 1 个单人分，当作独自一人的耳朵（20 次抽样取均值）
    B = {}
    draws = 20
    lone_vs_rest = []
    for p in list(A) + ["crowd_loo"]:
        vals, flags_all = [], []
        for _ in range(draws):
            pick = {u: pyr.randrange(len(c["raw"])) for u, c in clips.items() if c["raw"]}
            lone = lambda u: clips[u]["raw"][pick[u]] if u in pick else None
            if p == "crowd_loo":
                other = lambda u: (st.mean(clips[u]["raw"][:pick[u]] + clips[u]["raw"][pick[u] + 1:])
                                   if u in pick and len(clips[u]["raw"]) > 1 else None)
            else:
                other = lambda u, p=p: clips[u][p]
            fl = agree_flags(pairs, clips, other, lone)
            flags_all += fl
            vals.append(sum(f for _, f in fl) / max(len(fl), 1))
        B[p] = {"agree_mean": round(st.mean(vals), 3), "agree_sd_over_draws": round(st.pstdev(vals), 3)}
    res["B_lone_listener"] = B

    # C · 评审团人数曲线：k 人均值 vs 另外不重叠的 H 人均值（同一条的票随机拆两半）
    n_min = min(len(c["raw"]) for c in clips.values() if c["raw"])
    H = n_min // 2
    C = {"held_out_panel": H, "rows": []}
    if H >= 1:
        for k in sorted({1, 2, 3, 4, 6, 8, 12} & set(range(1, n_min - H + 1))):
            vals = {"k": k}
            accs, paccs = [], {p: [] for p in A}
            for _ in range(10):
                split = {}
                for u, c in clips.items():
                    if not c["raw"] or len(c["raw"]) < n_min:
                        continue
                    r = c["raw"][:]
                    pyr.shuffle(r)
                    split[u] = (st.mean(r[:k]), st.mean(r[k:k + H]))
                pk = lambda u: split[u][0] if u in split else None
                ph = lambda u: split[u][1] if u in split else None
                fl = agree_flags(pairs, clips, pk, ph)
                accs.append(sum(f for _, f in fl) / max(len(fl), 1))
                for p in A:
                    fl2 = agree_flags(pairs, clips, lambda u, p=p: clips[u][p], pk)
                    paccs[p].append(sum(f for _, f in fl2) / max(len(fl2), 1))
            vals["panel_k_vs_heldout"] = round(st.mean(accs), 3)
            vals["predictor_vs_panel_k"] = {p: round(st.mean(v), 3) for p, v in paccs.items()}
            C["rows"].append(vals)
    res["C_panel_size"] = C

    # D · 只有 48 对（12 屏 × 4 对）时，「DNSMOS vs 一个人」的一致率会抖多大；作者的 0.48 落在第几分位
    by_src = defaultdict(list)
    for a, b in pairs:
        by_src[clips[a]["src"]].append((a, b))
    srcs = [s for s, v in by_src.items() if len(v) >= 4]
    sims = []
    for _ in range(2000):
        ss = pyr.sample(srcs, AUTHOR["n_screens"])
        sub = [pp for s in ss for pp in pyr.sample(by_src[s], 4)]
        pick = {u: pyr.randrange(len(clips[u]["raw"])) for pp in sub for u in pp}
        fl = agree_flags(sub, clips, lambda u: clips[u]["dnsmos_ovrl"], lambda u: clips[u]["raw"][pick[u]])
        if fl:
            sims.append(sum(f for _, f in fl) / len(fl))
    sims = np.array(sims)
    res["D_author_like_48pairs"] = {
        "dnsmos_vs_lone_listener_48pairs_p2.5_p50_p97.5": [round(float(np.percentile(sims, q)), 3) for q in (2.5, 50, 97.5)],
        "author_dnsmos_agree": AUTHOR["dnsmos_ovrl"],
        "share_of_sims_at_or_below_author": round(float((sims <= AUTHOR["dnsmos_ovrl"]).mean()), 3)}

    # E · 系统排名：用 m 条 × k 人评，和全体众人的系统排名比 Kendall τ
    systems = sorted({c["sys"] for c in clips.values()})
    full = {s: st.mean(c["mos"] for c in clips.values() if c["sys"] == s) for s in systems}
    srcs_all = sorted({c["src"] for c in clips.values()})
    idx = {(c["src"], c["sys"]): u for u, c in clips.items()}
    E = []
    for m in [5, 10, 20, 50, 100]:
        if m > len(srcs_all):
            continue
        for k in [1, 2, 4, 8]:
            if k > n_min:
                continue
            taus = []
            for _ in range(50):
                ss = pyr.sample(srcs_all, m)
                est = {}
                for s in systems:
                    v = [st.mean(pyr.sample(clips[idx[(x, s)]]["raw"], k)) for x in ss
                         if (x, s) in idx and clips[idx[(x, s)]]["raw"] and len(clips[idx[(x, s)]]["raw"]) >= k]
                    if v:
                        est[s] = st.mean(v)
                common = [s for s in systems if s in est]
                if len(common) >= 3:
                    taus.append(kendalltau([full[s] for s in common], [est[s] for s in common])[0])
            if taus:
                E.append({"clips_per_system": m, "raters_per_clip": k, "ratings_per_system": m * k,
                          "ktau_vs_full_crowd_median": round(float(np.median(taus)), 3),
                          "ktau_p10": round(float(np.percentile(taus, 10)), 3)})
    res["E_system_ranking"] = E
    return res


def to_md(R):
    L = ["# exp009 · 众人耳朵 · 结果", "", f"数据：urgent-challenge/vmc2026-track1-meta@{REV[:7]}（只用 label 文本）· 种子 {SEED} · bootstrap {BOOT}", ""]
    L += ["## A · 机器分 vs 众人均值（同源句配对方向一致率 · 按源句聚类 95% 区间）", "",
          "| 数据集 | 指标 | 配对 | 一致率 [95%] | \\|Δ众人\\|≥1 的配对 | 语句 SRCC | 系统 SRCC | 系统 τ |", "|---|---|---|---|---|---|---|---|"]
    for s, r in R.items():
        for p, a in r["A_predictor_vs_crowd"].items():
            g = a["big_gap"] or {"agree": "-", "ci95": ["-", "-"], "n": 0}
            L.append(f"| {s} | {p} | {a['all']['n']} | {a['all']['agree']} [{a['all']['ci95'][0]}, {a['all']['ci95'][1]}] | "
                     f"{g['agree']} [{g['ci95'][0]}, {g['ci95'][1]}]（{g['n']}）| {a['utt_srcc']} | {a['sys_srcc']} | {a['sys_ktau']} |")
    L += ["", "## B · 只有一个人当裁判（每条随机抽 1 个单人分，20 次抽样均值）", "",
          "| 数据集 | 对手 | 与「一个人」的一致率 |", "|---|---|---|"]
    for s, r in R.items():
        for p, b in r.get("B_lone_listener", {}).items():
            L.append(f"| {s} | {'其余众人（留一）' if p == 'crowd_loo' else p} | {b['agree_mean']}（抽样 sd {b['agree_sd_over_draws']}）|")
    L += ["", "## C · 评审团人数：k 人均值 vs 另一组不重叠的 H 人", ""]
    for s, r in R.items():
        c = r.get("C_panel_size")
        if not c:
            continue
        L += [f"**{s}**（H = {c['held_out_panel']}）", "", "| k 人 | k 人 vs H 人 | " + " | ".join(f"{p} vs k 人" for p in c["rows"][0]["predictor_vs_panel_k"]) + " |",
              "|---|---|" + "---|" * len(c["rows"][0]["predictor_vs_panel_k"])]
        for row in c["rows"]:
            L.append(f"| {row['k']} | {row['panel_k_vs_heldout']} | " + " | ".join(str(v) for v in row["predictor_vs_panel_k"].values()) + " |")
        L.append("")
    L += ["## D · 像作者那样只有 48 对（12 屏 × 4 对）+ 一个人时，DNSMOS 一致率的抽样分布", "",
          "| 数据集 | 2.5% / 50% / 97.5% | 作者实测 | 模拟里 ≤ 作者的比例 |", "|---|---|---|---|"]
    for s, r in R.items():
        d = r.get("D_author_like_48pairs")
        if d:
            L.append(f"| {s} | {' / '.join(map(str, d['dnsmos_vs_lone_listener_48pairs_p2.5_p50_p97.5']))} | {d['author_dnsmos_agree']} | {d['share_of_sims_at_or_below_author']} |")
    L += ["", "## E · 系统排名要多少评分：m 条 × k 人 vs 全体众人（Kendall τ 中位数 / 10 分位）", ""]
    for s, r in R.items():
        if r.get("E_system_ranking"):
            L += [f"**{s}**（{r['systems']} 个系统）", "", "| 每系统条数 m | 每条人数 k | 每系统评分数 | τ 中位 | τ 10 分位 |", "|---|---|---|---|---|"]
            for e in r["E_system_ranking"]:
                L.append(f"| {e['clips_per_system']} | {e['raters_per_clip']} | {e['ratings_per_system']} | {e['ktau_vs_full_crowd_median']} | {e['ktau_p10']} |")
            L.append("")
    return "\n".join(L) + "\n"


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=None)
    a = ap.parse_args()
    R = {s: analyse(s) for s in SUBSETS}
    here = Path(__file__).resolve().parent
    out = Path(a.out) if a.out else here / "results/crowd_ears.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps({"rev": REV, "seed": SEED, "results": R}, ensure_ascii=False, indent=1))
    out.with_suffix(".md").write_text(to_md(R))
    print(out.with_suffix(".md").read_text())
