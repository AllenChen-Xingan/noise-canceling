"""exp009 · 用公开众包逐条评分，量「机器分 vs 众人」「单个听众 vs 众人」「要几人几条才够」。

只读纯文本标签（先跑 fetch_labels.py），不碰音频、不跑模型推理，几十秒跑完，不占名额。
用法：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp009-more-ears/crowd_ears.py
输出：results/crowd_ears.{json,md}

口径（对齐 exp007 定标 src/eval/src/calib.rs）：
- 配对 = 同一条源句（同一条带噪输入）的两个不同系统输出；比较方向，任一边差为 0 的配对剔除
- 区间 = 按源句聚类 bootstrap（种子 20261030）
- 「|Δ众人|≥1」≈ exp003「H3 同屏差 ≥ 1 级」

数据要点：
- CHiME-7 UDASE：用逐听众 CSV 的 OVRL 一张量表（vmc2026 meta 的 MOS 把 SIG/BAK/OVRL 三表混在一起，
  见 resources/urgent2026_challenge_track2/scripts/data/chime-7-udase-eval/data_prep.py:42-58），32 名固定听众，
  每段话 5 个条件由同一组 8 人听完 → 能算「同一个人比较两个系统」，最像作者的处境
- URGENT 2024：每条 8 票，但没有听众 ID，同一配对两边的票多半来自不同的人 → 「单票」只是下界参照
"""
import csv
import itertools
import json
from collections import defaultdict
from pathlib import Path

import numpy as np
from scipy.stats import kendalltau, spearmanr

ROOT = Path.home() / "vdn-data/public-mos"
REV = "ebe09d8891b7dc1f77ddf1772e42bb22de3ef500"
META = ROOT / "vmc2026-track1-meta" / REV
CHIME_RAW = ROOT / "chime7-udase-listening/raw_results_listening_test.csv"
SEED = 20261030
BOOT = 2000
PRED = ["dnsmos_ovrl", "scoreq", "distill_mos", "utmos", "nisqa_mos"]
# 作者一人（exp007 calib 第 1 轮，exp003）：48 对；12 屏，其中有信息的 9 屏
AUTHOR = {"dnsmos_ovrl": 0.48, "dnsmos_ci95": [0.26, 0.70], "n_pairs": 48, "n_screens": 12}

DESC = {
    "chime7": "CHiME-7 UDASE · 真实多人嘈杂（英语）· 5 条件 · 32 名固定听众、每条 8 人 · P.835 OVRL · CC BY-SA 4.0",
    "urgent2024": "URGENT 2024 盲测 · 英语 · 23 系统 · 每条 8 票（P.808 众包，无听众 ID）· CC BY-NC-SA 4.0 只评测",
    "urgent2025": "URGENT 2025 盲测 · 多语 · 26 系统 · 只有均值（无逐票）· 随源集 NC 只评测",
    "tmhint": "TMHINT-QI 测试 · 中文 · 老式 DNN 增强 · 每条 3–11 票（无同人同句配对）",
}


def scp(p):
    if not p.exists():
        return {}
    return {k: v for k, v in (l.split(maxsplit=1) for l in p.read_text().splitlines())}


class DS:
    """一个数据集：clips 有 src / sys / mos / 预测器分；raw = 逐票（list）或逐听众（dict）。"""

    def __init__(self, name, uids, src, sysn, mos, raw, preds):
        self.name, self.uids = name, uids
        self.src, self.sys = np.array(src), np.array(sysn)
        self.mos = np.array(mos, float)
        self.raw = raw
        self.preds = {p: np.array(v, float) for p, v in preds.items()}
        pa, pb = [], []
        by = defaultdict(list)
        for i, s in enumerate(src):
            by[s].append(i)
        for lst in by.values():
            for a, b in itertools.combinations(lst, 2):
                if sysn[a] != sysn[b]:
                    pa.append(a)
                    pb.append(b)
        self.pa, self.pb = np.array(pa), np.array(pb)
        _, self.psrc = np.unique(self.src[self.pa], return_inverse=True)


def load_meta(sub, name, keyfn):
    d = META / sub
    mos = scp(d / "mos_overall.scp")
    sysm, refm = scp(d / "utt2system"), scp(d / "utt2reference")
    raw = {}
    if (d / "meta.jsonl").exists():
        for line in (d / "meta.jsonl").open():
            j = json.loads(line)
            raw[j["uid"]] = [x[1] for x in j.get("mos_overall", []) if x[1] is not None]
    P = {p: scp(d / f"{p}.scp") for p in PRED}
    rows = []
    for u, m in mos.items():
        g = keyfn(u, sysm.get(u, ""), refm.get(u, ""))
        if g and all(u in P[p] for p in PRED):
            rows.append((u, g[0], g[1], float(m)))
    return DS(name, [r[0] for r in rows], [r[1] for r in rows], [r[2] for r in rows], [r[3] for r in rows],
              [raw.get(r[0]) for r in rows], {p: [float(P[p][r[0]]) for r in rows] for p in PRED})


def load_chime():
    D = defaultdict(dict)
    for r in csv.DictReader(CHIME_RAW.open()):
        if r["scale"] == "OVRL" and r["condition"].lower() != "ref":
            D[f"{r['condition'].upper()}:{r['sample'].rsplit('.', 1)[0]}"][r["csv"]] = float(r["rating"])
    P = {p: scp(META / "chime-7-udase-eval/test" / f"{p}.scp") for p in PRED}
    uids = sorted(u for u in D if all(u in P[p] for p in PRED))
    return DS("chime7", uids, [u.split(":", 1)[1] for u in uids], [u.split(":")[0] for u in uids],
              [np.mean(list(D[u].values())) for u in uids], [D[u] for u in uids],
              {p: [float(P[p][u]) for u in uids] for p in PRED})


def tm_key(u, s, ref):
    if s.startswith("clean"):
        return None
    method, cond = s.split("-SNR", 1)
    return f"{ref}|SNR{cond}", method


def agree(x, y, pa, pb, mask=None):
    """方向一致：返回 (每对是否计入, 每对是否一致)。x / y 是逐条分数（可含 nan）。"""
    dx, dy = x[pa] - x[pb], y[pa] - y[pb]
    ok = (dx != 0) & (dy != 0) & ~np.isnan(dx) & ~np.isnan(dy)
    if mask is not None:
        ok &= mask
    return ok, (np.sign(dx) == np.sign(dy))


def rate(ok, hit):
    n = int(ok.sum())
    return (float(hit[ok].mean()) if n else float("nan")), n


def cluster_ci(ok, hit, cl, rng):
    n_cl = cl.max() + 1
    s = np.bincount(cl, weights=(ok & hit).astype(float), minlength=n_cl)
    n = np.bincount(cl, weights=ok.astype(float), minlength=n_cl)
    idx = rng.integers(0, n_cl, size=(BOOT, n_cl))
    b = s[idx].sum(1) / np.maximum(n[idx].sum(1), 1)
    return [round(float(np.percentile(b, 2.5)), 3), round(float(np.percentile(b, 97.5)), 3)]


def dist(v):
    v = np.array(v, float)
    return {"n": len(v), "min": r3(v.min()), "p10": r3(np.percentile(v, 10)), "median": r3(np.median(v)),
            "p90": r3(np.percentile(v, 90)), "max": r3(v.max()), "mean": r3(v.mean())}


def r3(x):
    return round(float(x), 3)


def part_a(ds, rng):
    out = {}
    big = np.abs(ds.mos[ds.pa] - ds.mos[ds.pb]) >= 1
    for p, x in ds.preds.items():
        ok, hit = agree(x, ds.mos, ds.pa, ds.pb)
        a, n = rate(ok, hit)
        okb = ok & big
        ab, nb = rate(okb, hit)
        systems = sorted(set(ds.sys))
        sx = [ds.mos[ds.sys == s].mean() for s in systems]
        sy = [x[ds.sys == s].mean() for s in systems]
        out[p] = {"pairs": n, "agree": r3(a), "ci95": cluster_ci(ok, hit, ds.psrc, rng),
                  "big_gap_pairs": nb, "big_gap_agree": r3(ab) if nb else None,
                  "utt_srcc": r3(spearmanr(x, ds.mos)[0]), "sys_srcc": r3(spearmanr(sx, sy)[0]),
                  "sys_ktau": r3(kendalltau(sx, sy)[0]), "n_systems": len(systems)}
    return out


def chime_listener_parts(ds, rng):
    """固定听众：①单人 vs 留一众人 ②预测器 vs 单人 ③评审团人数曲线 ④像作者那样只有 48 对"""
    listeners = sorted({l for d in ds.raw for l in d})
    M = np.full((len(ds.uids), len(listeners)), np.nan)
    li = {l: j for j, l in enumerate(listeners)}
    for i, d in enumerate(ds.raw):
        for l, v in d.items():
            M[i, li[l]] = v
    tot = np.nansum(M, 1)
    cnt = (~np.isnan(M)).sum(1)
    single_vs_loo, pred_vs_single = [], {p: [] for p in ds.preds}
    ties = []
    for j in range(len(listeners)):
        y = M[:, j]
        loo = np.where(np.isnan(y), np.nan, (tot - np.nan_to_num(y)) / np.maximum(cnt - 1, 1))
        has = ~np.isnan(y[ds.pa]) & ~np.isnan(y[ds.pb])
        ties.append(float(((y[ds.pa] - y[ds.pb]) == 0)[has].mean()))
        a, _ = rate(*agree(y, loo, ds.pa, ds.pb))
        single_vs_loo.append(a)
        for p, x in ds.preds.items():
            pred_vs_single[p].append(rate(*agree(x, y, ds.pa, ds.pb))[0])
    res = {"listeners": len(listeners), "own_tie_rate_mean": r3(np.mean(ties)),
           "single_vs_loo": dist(single_vs_loo), "pred_vs_single": {p: dist(v) for p, v in pred_vs_single.items()},
           "pred_vs_crowd_minus_vs_single_median": {}}

    # ③ 评审团人数：每段话的 8 人里随机抽 k 人当评审团，另抽 4 个不重叠的人当「独立众人」
    by_src = defaultdict(list)
    for i, s in enumerate(ds.src):
        by_src[s].append(i)
    curve = []
    for k in [1, 2, 3, 4]:
        acc, pacc = [], {p: [] for p in ds.preds}
        for _ in range(50):
            panel = np.full(len(ds.uids), np.nan)
            held = np.full(len(ds.uids), np.nan)
            for s, idx in by_src.items():
                who = np.where(~np.isnan(M[idx[0]]))[0]
                pick = rng.permutation(who)
                panel[idx] = np.nanmean(M[np.ix_(idx, pick[:k])], 1)
                held[idx] = np.nanmean(M[np.ix_(idx, pick[k:k + 4])], 1)
            acc.append(rate(*agree(panel, held, ds.pa, ds.pb))[0])
            for p, x in ds.preds.items():
                pacc[p].append(rate(*agree(x, panel, ds.pa, ds.pb))[0])
        curve.append({"k": k, "panel_vs_heldout4": r3(np.mean(acc)), "pred_vs_panel": {p: r3(np.mean(v)) for p, v in pacc.items()}})
    res["panel_curve"] = curve

    # ④ 像作者：一个固定听众、12 屏 × 4 对 = 48 对；预测器 vs 这个人的一致率抽样分布
    psrc_pairs = defaultdict(list)
    for t, s in enumerate(ds.psrc):
        psrc_pairs[s].append(t)
    sims = {p: [] for p in ["dnsmos_ovrl", "scoreq"]}
    sims_loo = []
    for _ in range(4000):
        j = rng.integers(len(listeners))
        y = M[:, j]
        cand = [s for s, ts in psrc_pairs.items() if not np.isnan(y[ds.pa[ts[0]]])]
        ss = rng.choice(cand, AUTHOR["n_screens"], replace=False)
        sel = np.concatenate([rng.choice(psrc_pairs[s], 4, replace=False) for s in ss])
        pa, pb = ds.pa[sel], ds.pb[sel]
        for p in sims:
            sims[p].append(rate(*agree(ds.preds[p], y, pa, pb))[0])
        loo = (tot - np.nan_to_num(y)) / np.maximum(cnt - 1, 1)
        sims_loo.append(rate(*agree(y, loo, pa, pb))[0])
    sims = {p: np.array(v)[~np.isnan(v)] for p, v in sims.items()}
    res["author_like_48"] = {
        p: {"p2.5": r3(np.percentile(v, 2.5)), "p50": r3(np.percentile(v, 50)), "p97.5": r3(np.percentile(v, 97.5)),
            "share_le_0.48": r3((v <= AUTHOR["dnsmos_ovrl"]).mean())} for p, v in sims.items()}
    sl = np.array(sims_loo)[~np.isnan(sims_loo)]
    res["author_like_48"]["single_vs_loo"] = {"p2.5": r3(np.percentile(sl, 2.5)), "p50": r3(np.percentile(sl, 50)),
                                              "p97.5": r3(np.percentile(sl, 97.5))}

    # ⑤ 要多少对，单人一致率的 95% 区间半宽才 ≤ 0.10（每屏 4 对，按屏聚类，对每个听众平均）
    need = []
    for n_pairs in [48, 100, 200, 400]:
        widths = []
        for j in range(len(listeners)):
            y = M[:, j]
            cand = [s for s, ts in psrc_pairs.items() if not np.isnan(y[ds.pa[ts[0]]])]
            n_scr = n_pairs // 4
            vals = []
            for _ in range(200):
                ss = rng.choice(cand, n_scr, replace=n_scr > len(cand))
                sel = np.concatenate([rng.choice(psrc_pairs[s], 4, replace=False) for s in ss])
                vals.append(rate(*agree(ds.preds["scoreq"], y, ds.pa[sel], ds.pb[sel]))[0])
            vals = np.array(vals)[~np.isnan(vals)]
            widths.append((np.percentile(vals, 97.5) - np.percentile(vals, 2.5)) / 2)
        need.append({"pairs": n_pairs, "half_width_mean": r3(np.mean(widths)), "note": "n>128 屏时屏有放回抽，偏乐观"})
    res["pairs_needed"] = need
    return res


def urgent_parts(ds, rng):
    """无听众 ID：单票（多半不同人）vs 留一；系统排名要多少评分。"""
    R = ds.raw
    nmin = min(len(r) for r in R)
    A = np.array([r[:nmin] for r in R], float)
    res = {"ratings_per_clip": nmin}
    vals, pv = [], {p: [] for p in ds.preds}
    for _ in range(20):
        pick = rng.integers(0, nmin, len(R))
        one = A[np.arange(len(R)), pick]
        loo = (A.sum(1) - one) / (nmin - 1)
        vals.append(rate(*agree(one, loo, ds.pa, ds.pb))[0])
        for p, x in ds.preds.items():
            pv[p].append(rate(*agree(x, one, ds.pa, ds.pb))[0])
    res["random_single_vote_vs_loo"] = r3(np.mean(vals))
    res["pred_vs_random_single_vote"] = {p: r3(np.mean(v)) for p, v in pv.items()}

    systems = sorted(set(ds.sys))
    full = np.array([ds.mos[ds.sys == s].mean() for s in systems])
    srcs = sorted(set(ds.src))
    cell = {(ds.src[i], ds.sys[i]): i for i in range(len(ds.uids))}
    rows = []
    for m in [5, 10, 20, 50, 100]:
        for k in [1, 2, 4, 8]:
            if k > nmin:
                continue
            taus = []
            for _ in range(100):
                ss = rng.choice(srcs, m, replace=False)
                est = []
                for s in systems:
                    v = [A[cell[(x, s)], rng.choice(nmin, k, replace=False)].mean() for x in ss if (x, s) in cell]
                    est.append(np.mean(v))
                taus.append(kendalltau(full, est)[0])
            rows.append({"clips_per_system": m, "votes_per_clip": k, "votes_per_system": m * k,
                         "ktau_median": r3(np.median(taus)), "ktau_p10": r3(np.percentile(taus, 10))})
    res["system_ranking"] = rows
    return res


def to_md(R):
    L = ["# exp009 · 众人耳朵 · 结果（crowd_ears.py 自动生成）", "",
         f"数据：`data-manifest.json`（vmc2026-track1-meta@{REV[:7]} + CHiME-7 逐听众 CSV）· 种子 {SEED} · bootstrap {BOOT}",
         f"作者一人参照（exp007 calib 第 1 轮）：DNSMOS OVRL 一致率 {AUTHOR['dnsmos_ovrl']} {AUTHOR['dnsmos_ci95']}，{AUTHOR['n_pairs']} 对", ""]
    L += ["## A · 机器分 vs 众人均值（同源句配对方向一致率 · 按源句聚类 95% 区间）", "",
          "| 数据集 | 指标 | 配对 | 一致率 [95%] | \\|Δ众人\\|≥1：一致率（对数） | 语句 SRCC | 系统 SRCC | 系统 τ（系统数） |",
          "|---|---|---|---|---|---|---|---|"]
    for name, r in R.items():
        for p, a in r["A"].items():
            L.append(f"| {name} | {p} | {a['pairs']} | {a['agree']} [{a['ci95'][0]}, {a['ci95'][1]}] | {a['big_gap_agree']}（{a['big_gap_pairs']}） | "
                     f"{a['utt_srcc']} | {a['sys_srcc']} | {a['sys_ktau']}（{a['n_systems']}） |")
    c = R["chime7"]["listener"]
    L += ["", "## B · CHiME-7：32 名固定听众，每人「同一段话比较两个条件」（最像作者）", "",
          f"听众自己打平的比例均值 {c['own_tie_rate_mean']}（已剔除）。", "",
          "| 比什么 | 最小 | p10 | 中位 | p90 | 最大 |", "|---|---|---|---|---|---|"]
    d = c["single_vs_loo"]
    L.append(f"| 单个听众 vs 其余 7 人均值 | {d['min']} | {d['p10']} | {d['median']} | {d['p90']} | {d['max']} |")
    for p, d in c["pred_vs_single"].items():
        L.append(f"| {p} vs 单个听众 | {d['min']} | {d['p10']} | {d['median']} | {d['p90']} | {d['max']} |")
    L += ["", "### 评审团人数：k 人均值 vs 另 4 个不重叠的人", "",
          "| k | k 人 vs 另 4 人 | " + " | ".join(f"{p} vs k 人" for p in PRED) + " |", "|---|---|" + "---|" * len(PRED)]
    for row in c["panel_curve"]:
        L.append(f"| {row['k']} | {row['panel_vs_heldout4']} | " + " | ".join(str(row["pred_vs_panel"][p]) for p in PRED) + " |")
    L += ["", "### 像作者那样：一个固定听众、12 屏 × 4 对 = 48 对（4000 次抽样）", "",
          "| 比什么 | 2.5% | 中位 | 97.5% | 抽样里 ≤ 0.48 的比例 |", "|---|---|---|---|---|"]
    for p in ["dnsmos_ovrl", "scoreq"]:
        a = c["author_like_48"][p]
        L.append(f"| {p} vs 这个人 | {a['p2.5']} | {a['p50']} | {a['p97.5']} | {a['share_le_0.48']} |")
    a = c["author_like_48"]["single_vs_loo"]
    L.append(f"| 这个人 vs 其余 7 人 | {a['p2.5']} | {a['p50']} | {a['p97.5']} | — |")
    L += ["", "### 单人要听多少对，SCOREQ 一致率的 95% 区间半宽才 ≤ 0.10（每屏 4 对）", "", "| 配对数 | 半宽均值 |", "|---|---|"]
    for row in c["pairs_needed"]:
        L.append(f"| {row['pairs']} | {row['half_width_mean']} |")
    u = R["urgent2024"]["votes"]
    L += ["", "## C · URGENT 2024（无听众 ID；单票两边多半是不同的人，只作下界参照）", "",
          f"单票 vs 留一 7 票均值：{u['random_single_vote_vs_loo']}；预测器 vs 单票：" +
          "，".join(f"{p} {v}" for p, v in u["pred_vs_random_single_vote"].items()), "",
          "### 23 个系统的排名要多少评分（m 条 × k 票，vs 全体众人；Kendall τ 中位 / 10 分位，100 次抽样）", "",
          "| 每系统条数 m | 每条票数 k | 每系统评分数 | τ 中位 | τ 10 分位 |", "|---|---|---|---|---|"]
    for e in u["system_ranking"]:
        L.append(f"| {e['clips_per_system']} | {e['votes_per_clip']} | {e['votes_per_system']} | {e['ktau_median']} | {e['ktau_p10']} |")
    return "\n".join(L) + "\n"


def main():
    rng = np.random.default_rng(SEED)
    sets = {
        "chime7": load_chime(),
        "urgent2024": load_meta("urgent2024-sqa/test", "urgent2024", lambda u, s, ref: (ref, s)),
        "urgent2025": load_meta("urgent2025-sqa/test", "urgent2025", lambda u, s, ref: (ref, s)),
        "tmhint": load_meta("tmhint-qi/test", "tmhint", tm_key),
    }
    R = {}
    for name, ds in sets.items():
        R[name] = {"desc": DESC[name], "clips": len(ds.uids), "pairs": len(ds.pa), "A": part_a(ds, rng)}
    R["chime7"]["listener"] = chime_listener_parts(sets["chime7"], rng)
    R["urgent2024"]["votes"] = urgent_parts(sets["urgent2024"], rng)
    here = Path(__file__).resolve().parent / "results"
    here.mkdir(exist_ok=True)
    (here / "crowd_ears.json").write_text(json.dumps({"seed": SEED, "author": AUTHOR, "results": R}, ensure_ascii=False, indent=1))
    (here / "crowd_ears.md").write_text(to_md(R))
    print(to_md(R))


if __name__ == "__main__":
    main()
