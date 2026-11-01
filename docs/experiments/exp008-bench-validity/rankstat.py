"""exp008 · 排名效度统计（synth-standard.md S7 生成器稳健性 · S8 人耳预测效度的低功效版）。

S7  每个格（cell × 档位）、每个指标：系统在生成器 A 上的均值排名 vs 在 B 上的排名 → Kendall τ_b（τ_AB）。
    参照上限：同一生成器内两半片段（偶数号 vs 奇数号）的 τ（split-half 信度）。τ_AB 明显低于信度 = 排名被生成器左右。
    区间：A、B 各自按片段有放回重采样 2000 次（种子 20261031），取 2.5 / 97.5 分位。
    四种系统集合：全体（含作弊天花板）· 去掉天花板与参数变体 · 每族一个代表（6 个）· 只看神经模型（8 个）；后两种只看点估计
S8  exp003 人耳（build f8c3，3 个系统 × 12 屏，会话打折）：配对规则同 vdn-eval calib（最清楚 / 最自然被选者胜同屏其余；
    能用 > 勉强 > 不能用 差 ≥ 1 级）。用「近似匹配的合成格」上的系统均值预测每一对的胜负，算一致率；
    对照组 = 同一指标直接在 exp003 片段上算（scores.json），也就是 calib round 1 的口径。**3 个系统、48 对，功效很低，只作方向参考。**

用法：python3 docs/experiments/exp008-bench-validity/rankstat.py <build>
输出：results/<build>-rankstat.json + 屏幕摘要
"""
import itertools
import json
import random
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
EXP003 = HERE.parent / "exp003-offline-ab"

METRICS = {  # 指标: 方向（+1 越大越好；-1 越小越好；0 = 越接近 0 越好）
    "si_sdr": 1, "estoi": 1, "dnsmos_ovrl": 1, "dnsmos_sig": 1, "dnsmos_bak": 1,
    "noise_att_db": -1, "nonowner_att_db": -1, "spl_db": 0,
}
CEILING = {"oracle_irm"}
VARIANTS = {"gtcrn_mix05"}
REPS = ["passthrough", "wiener_dd", "gtcrn_vctk", "gtcrn", "dpdf8", "oracle_irm"]  # 每族一个代表（C 地板 · C · A′ · A · B · O）
NEURAL = {"gtcrn_vctk", "gtcrn", "ulunas", "fe_s", "fe_m", "dpdf_base", "dpdf2", "dpdf8"}  # 只看神经模型：去掉 DSP 与地板后排名难得多


def kendall_tau_b(x, y):
    n = len(x)
    c = d = tx = ty = 0
    for i, j in itertools.combinations(range(n), 2):
        a, b = x[i] - x[j], y[i] - y[j]
        if a == 0 and b == 0:
            continue
        if a == 0:
            tx += 1
        elif b == 0:
            ty += 1
        elif (a > 0) == (b > 0):
            c += 1
        else:
            d += 1
    den = ((c + d + tx) * (c + d + ty)) ** 0.5
    return (c - d) / den if den else None


def score(v, direction):
    return None if v is None else (v * direction if direction else -abs(v))


def load(build):
    """第一轮 + 第二轮（-scores-r2*.jsonl，按系统分批）合并；同一 (段, 系统) 只留一条，后来的覆盖先来的。"""
    rows = {}
    for p in [HERE / "results" / f"{build}-scores.jsonl"] + sorted((HERE / "results").glob(f"{build}-scores-r2*.jsonl")):
        if p.exists():
            for l in open(p):
                r = json.loads(l)
                rows[(r["cell"], r["gen"], r["clip"], json.dumps(r["level"]), r["system"])] = r
    return [r for r in rows.values() if not r["error"]]


def means(rows, systems, metric, clips=None):
    acc = defaultdict(list)
    for r in rows:
        if r["system"] in systems and (clips is None or r["clip"] in clips):
            v = score(r["metrics"].get(metric), METRICS[metric])
            if v is not None:
                acc[r["system"]].append(v)
    return {s: sum(v) / len(v) for s, v in acc.items() if v}


def tau_between(ma, mb):
    ss = sorted(set(ma) & set(mb))
    if len(ss) < 3:
        return None
    return kendall_tau_b([ma[s] for s in ss], [mb[s] for s in ss])


def s7(rows):
    systems_all = sorted({r["system"] for r in rows})
    sets = {"all": set(systems_all), "no_ceiling_variant": set(systems_all) - CEILING - VARIANTS, "reps": set(REPS) & set(systems_all),
            "neural": NEURAL & set(systems_all)}
    cells = defaultdict(list)
    for r in rows:
        cells[(r["cell"], json.dumps(r["level"]))].append(r)
    rng = random.Random(20261031)
    out = []
    for (cell, lv), rs in sorted(cells.items()):
        A = [r for r in rs if r["gen"] == "A"]
        B = [r for r in rs if r["gen"] == "B"]
        clips_a = sorted({r["clip"] for r in A})
        clips_b = sorted({r["clip"] for r in B})
        for metric in METRICS:
            row = {"cell": cell, "level": json.loads(lv), "metric": metric}
            for name, ss in sets.items():
                ma, mb = means(A, ss, metric), means(B, ss, metric)
                t = tau_between(ma, mb)
                if t is None:
                    continue
                ev, od = set(clips_a[0::2]), set(clips_a[1::2])
                sh_a = tau_between(means(A, ss, metric, ev), means(A, ss, metric, od))
                ev, od = set(clips_b[0::2]), set(clips_b[1::2])
                sh_b = tau_between(means(B, ss, metric, ev), means(B, ss, metric, od))
                # 同样本量的对照：A 半边 × B 半边（3 段 vs 3 段）四种组合的平均 τ，和 split-half 一样大小，才能公平相比
                halves_a = [set(clips_a[0::2]), set(clips_a[1::2])]
                halves_b = [set(clips_b[0::2]), set(clips_b[1::2])]
                hh = [tau_between(means(A, ss, metric, ha), means(B, ss, metric, hb)) for ha in halves_a for hb in halves_b]
                hh = [x for x in hh if x is not None]
                t_half = sum(hh) / len(hh) if hh else None
                d = {"n_sys": len(set(ma) & set(mb)), "tau_ab": round(t, 3),
                     "tau_ab_half": None if t_half is None else round(t_half, 3),
                     "split_half_a": None if sh_a is None else round(sh_a, 3),
                     "split_half_b": None if sh_b is None else round(sh_b, 3)}
                if name not in ("reps", "neural"):
                    boots = []
                    for _ in range(2000):
                        ca = [rng.choice(clips_a) for _ in clips_a]
                        cb = [rng.choice(clips_b) for _ in clips_b]
                        ma2 = _boot_means(A, ss, metric, ca)
                        mb2 = _boot_means(B, ss, metric, cb)
                        tb = tau_between(ma2, mb2)
                        if tb is not None:
                            boots.append(tb)
                    boots.sort()
                    d["ci95"] = [round(boots[int(0.025 * len(boots))], 3), round(boots[int(0.975 * len(boots)) - 1], 3)]
                row[name] = d
            out.append(row)
    return out


def _boot_means(rows, systems, metric, clip_sample):
    by = defaultdict(dict)
    for r in rows:
        if r["system"] in systems:
            v = score(r["metrics"].get(metric), METRICS[metric])
            if v is not None:
                by[r["system"]][r["clip"]] = v
    out = {}
    for s, d in by.items():
        vals = [d[c] for c in clip_sample if c in d]
        if vals:
            out[s] = sum(vals) / len(vals)
    return out


# ---------------- S8 ----------------
EXP003_MODEL = {"gtcrn": "gtcrn", "fastenhancer_m": "fe_m", "dpdfnet2": "dpdf2"}
# exp003 片段 → 近似匹配的合成格（cell, level）。都是「近似」：噪声素材、说话人、具体 SNR / SIR 都对不上，只对上大类
MATCH = {
    "c01": ("S-SNR", 5.0), "c02": ("S-SNR", 0.0), "c03": ("S-SNR", -5.0),
    "c04": ("M-SIRxSNR", {"sir_db": 10.0, "snr_db": 5.0}), "c05": ("S-SIR", 0.0),
    "c06": ("M-SNRxNOISE", {"snr_db": 0.0, "noise": "babble"}), "c07": ("M-SNRxNOISE", {"snr_db": 0.0, "noise": "babble"}),
    "c08": ("M-SNRxNOISE", {"snr_db": 10.0, "noise": "babble"}), "c09": ("S-NOISE", "clatter"), "c10": ("S-NOISE", "music"),
    "c11": ("M-SNRxNOISE", {"snr_db": 0.0, "noise": "babble"}), "c12": ("M-SIRxSNR", {"sir_db": 10.0, "snr_db": 5.0}),
}
EXP003_METRIC = {"si_sdr": "SI-SDR", "estoi": "ESTOI", "dnsmos_ovrl": "OVRL", "dnsmos_sig": "SIG", "dnsmos_bak": "BAK", "noise_att_db": "noise_att_db"}
RANK = {"ok": 2, "marginal": 1, "no": 0}


def human_pairs():
    d = json.load(open(EXP003 / "results/20261030-human-build-20261030110755-f8c3.json"))
    pairs = []
    for t in d["trials"]:
        if t["kind"] != "real":
            continue
        bm = t["by_model"]
        models = [m for m in t["slots"].values()]
        for q in ("clearer", "natural"):
            w = bm.get(q)
            if w and w != "same":
                pairs += [(t["clip"], w, l, q) for l in models if l != w]
        u = bm.get("usable", {})
        for a, b in itertools.combinations(models, 2):
            if a in u and b in u and RANK[u[a]] != RANK[u[b]]:
                w, l = (a, b) if RANK[u[a]] > RANK[u[b]] else (b, a)
                pairs.append((t["clip"], w, l, "h3"))
    return pairs


def s8(rows):
    pairs = human_pairs()
    sc = json.load(open(EXP003 / "scores.json"))["per_clip"]
    out = {}
    for metric, m3 in EXP003_METRIC.items():
        syn_hit = syn_n = clip_hit = clip_n = 0
        for clip, w, l, _ in pairs:
            cell, lv = MATCH[clip]
            sel = [r for r in rows if r["cell"] == cell and json.dumps(r["level"], sort_keys=True) == json.dumps(lv, sort_keys=True)]
            mm = means(sel, {EXP003_MODEL[w], EXP003_MODEL[l]}, metric)
            if len(mm) == 2 and mm[EXP003_MODEL[w]] != mm[EXP003_MODEL[l]]:
                syn_n += 1
                syn_hit += mm[EXP003_MODEL[w]] > mm[EXP003_MODEL[l]]
            try:
                vw, vl = sc[clip][w].get(m3), sc[clip][l].get(m3)
            except KeyError:
                vw = vl = None
            if vw is not None and vl is not None and vw != vl:
                clip_n += 1
                clip_hit += score(vw, METRICS[metric]) > score(vl, METRICS[metric])
        out[metric] = {"pairs": len(pairs), "synthetic_cell": {"n": syn_n, "agree": round(syn_hit / syn_n, 3) if syn_n else None},
                       "on_exp003_clip": {"n": clip_n, "agree": round(clip_hit / clip_n, 3) if clip_n else None}}
    return out


def main():
    build = sys.argv[1]
    rows = load(build)
    r7 = s7(rows)
    r8 = s8(rows)
    (HERE / "results" / f"{build}-rankstat.json").write_text(json.dumps({"build": build, "S7": r7, "S8": r8}, ensure_ascii=False, indent=1))
    # 摘要：每个指标跨格的 τ_AB 与 split-half 中位数
    med = lambda v: sorted(v)[len(v) // 2] if v else None  # noqa: E731
    print("S7（no_ceiling_variant，系统数 / τ_AB 中位 / 信度 A、B 中位 / 下界 > 0 的格数）")
    for metric in METRICS:
        xs = [r["no_ceiling_variant"] for r in r7 if r["metric"] == metric and "no_ceiling_variant" in r]
        if not xs:
            continue
        print(f"  {metric:16s} n={xs[0]['n_sys']:2d} τ_AB={med([x['tau_ab'] for x in xs])} "
              f"信度A={med([x['split_half_a'] for x in xs if x['split_half_a'] is not None])} "
              f"信度B={med([x['split_half_b'] for x in xs if x['split_half_b'] is not None])} "
              f"下界>0: {sum(x['ci95'][0] > 0 for x in xs)}/{len(xs)}")
    print("S7（neural，8 个神经模型，τ_AB 中位 / 信度 A、B 中位）")
    for metric in METRICS:
        xs = [r["neural"] for r in r7 if r["metric"] == metric and "neural" in r]
        if xs:
            print(f"  {metric:16s} τ_AB={med([x['tau_ab'] for x in xs])} 信度A={med([x['split_half_a'] for x in xs if x['split_half_a'] is not None])} "
                  f"信度B={med([x['split_half_b'] for x in xs if x['split_half_b'] is not None])}")
    print("S8（exp003 人耳配对一致率：合成格预测 vs 片段自身）")
    for m, v in r8.items():
        print(f"  {m:16s} 合成格 {v['synthetic_cell']}  片段自身 {v['on_exp003_clip']}")


if __name__ == "__main__":
    main()
