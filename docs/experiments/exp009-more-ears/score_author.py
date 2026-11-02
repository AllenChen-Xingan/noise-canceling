"""exp009 · verifier V-AUTHOR-VS-CROWD：作者一人的盲听，与「众人耳训出的预测器」一致到什么程度，放进众人单人分布里看。

步骤：
1. 对 regen_exp003.py 重新生成的、作者实际听到的那份音频（响度统一后的 audio/）打 SCOREQ（natural · NR）、Distill-MOS、DNSMOS
2. 核对：重算的 DNSMOS OVRL 与 exp003 scores.json 逐条比，确认音频就是作者听的那份
3. 按 exp007 定标的配对口径（src/eval/src/calib.rs extract：最清楚 / 最自然被选者对同屏其余 + H3 差 ≥ 1 级）
   算作者与各预测器的一致率；两种平局口径都给（calib：预测器打平记 0.5；众人分析：剔除）
4. 判据（事先写死，见 README「预注册」）：作者对 SCOREQ 的一致率（剔除平局）落在
   「CHiME-7 固定听众 × 48 对」抽样分布 2.5% 分位之下 → 记「作者偏离大众：有证据（SCOREQ 代理）」；否则「无证据」

用法（要名额）：<主目录>/_tmp/exp009-venv/bin/python docs/experiments/exp009-more-ears/score_author.py
"""
import json
import sys
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
EXP003 = HERE.parent / "exp003-offline-ab"
sys.path.insert(0, str(EXP003))
from _paths import RES, ROOT  # noqa: E402

REGEN = ROOT / "_tmp/20261031-exp009/exp003-regen"
ANSWERS = EXP003 / "results/20261030-human-build-20261030110755-f8c3.json"
SCORES003 = EXP003 / "scores.json"
CROWD = HERE / "results/crowd_ears.json"
MODELS = ["dpdfnet2", "gtcrn", "fastenhancer_m"]
SEED = 20261030
GRADE = {"ok": 2, "marginal": 1, "no": 0}


def predict_all():
    files = json.loads((REGEN / "files.json").read_text())
    sys.path.insert(0, str(RES / "DNS-Challenge/DNSMOS"))
    from dnsmos_local import ComputeScore
    d = RES / "DNS-Challenge/DNSMOS/DNSMOS"
    dns = ComputeScore(str(d / "sig_bak_ovr.onnx"), str(d / "model_v8.onnx"))
    import scoreq
    sq = scoreq.Scoreq(data_domain="natural", mode="nr")
    import distillmos
    import torch
    dm = distillmos.ConvTransformerSQAModel()
    dm.eval()
    out = {}
    for cid, c in files.items():
        out[cid] = {}
        for m in ["noisy"] + MODELS:
            p = REGEN / c["versions"][m]["file"]
            x, sr = sf.read(p, dtype="float32")
            s = dns(str(p), sr, False)  # 与 exp003 score.py 同调用
            with torch.no_grad():
                dmos = float(dm(torch.from_numpy(x)[None, :]).squeeze())
            out[cid][m] = {"scoreq": round(float(sq.predict(test_path=str(p), ref_path=None)), 4),
                           "distill_mos": round(dmos, 4), "OVRL": round(float(s["OVRL"]), 3)}
            print(cid, m, out[cid][m], flush=True)
    return out


def extract_pairs(ans):
    """与 calib.rs extract 同口径：只用 real 屏。"""
    pairs = []
    for t in ans["trials"]:
        if t["kind"] != "real":
            continue
        clip, models = t["clip"], list(t["slots"].values())
        for q in ["clearer", "natural"]:
            w = t["by_model"].get(q)
            if w in models:
                pairs += [(clip, w, l, q) for l in models if l != w]
        g = {m: GRADE[v] for m, v in (t["by_model"].get("usable") or {}).items() if v in GRADE}
        pairs += [(clip, a, b, "h3") for a in g for b in g if g[a] - g[b] >= 1]
    return pairs


def agreement(pairs, S, key, rng):
    rows = []
    for clip, w, l, src in pairs:
        dw, dl = S[clip][w][key], S[clip][l][key]
        rows.append((clip, src, 0.5 if abs(dw - dl) < 1e-9 else float(dw > dl)))
    screens = sorted({r[0] for r in rows})

    def rate(scr, drop_ties):
        v = [r[2] for s in scr for r in rows if r[0] == s and not (drop_ties and r[2] == 0.5)]
        return float(np.mean(v)) if v else float("nan"), len(v)

    res = {}
    for name, drop in [("calib_ties_half", False), ("ties_dropped", True)]:
        point, n = rate(screens, drop)
        bs = [rate(list(rng.choice(screens, len(screens))), drop)[0] for _ in range(10000)]
        res[name] = {"agree": round(point, 3), "n": n, "screens": len(screens),
                     "ci95": [round(float(np.nanpercentile(bs, 2.5)), 3), round(float(np.nanpercentile(bs, 97.5)), 3)]}
    res["by_source"] = {s: round(float(np.mean([r[2] for r in rows if r[1] == s])), 3) for s in ["clearer", "natural", "h3"]
                        if any(r[1] == s for r in rows)}
    return res


def main():
    cache = HERE / "results/exp003_predictors.json"
    if cache.exists() and "--rescore" not in sys.argv:
        S = json.loads(cache.read_text())
    else:
        S = predict_all()
        cache.write_text(json.dumps(S, ensure_ascii=False, indent=1))
    old = json.loads(SCORES003.read_text())["per_clip"]
    d_ovrl = max(abs(S[c][m]["OVRL"] - old[c][m]["OVRL"]) for c in S for m in ["noisy"] + MODELS)
    rng = np.random.default_rng(SEED)
    pairs = extract_pairs(json.loads(ANSWERS.read_text()))
    crowd = json.loads(CROWD.read_text())["results"]["chime7"]["listener"]
    ref48 = crowd["author_like_48"]
    R = {"regen_dnsmos_max_abs_delta_vs_exp003": round(d_ovrl, 3), "pairs": len(pairs),
         "author_vs": {k: agreement(pairs, S, k, rng) for k in ["OVRL", "scoreq", "distill_mos"]},
         "crowd_reference_chime7_48pairs": {"scoreq": ref48["scoreq"], "dnsmos_ovrl": ref48["dnsmos_ovrl"],
                                            "single_vs_loo": ref48["single_vs_loo"]}}
    a = R["author_vs"]["scoreq"]["ties_dropped"]["agree"]
    lo = ref48["scoreq"]["p2.5"]
    R["verdict"] = {"verifier": "V-AUTHOR-VS-CROWD", "author_scoreq_agree": a, "threshold_p2.5": lo,
                    "author_deviates": bool(a < lo),
                    "text": "作者偏离大众：有证据（SCOREQ 代理）" if a < lo else "作者偏离大众：无证据（在单人 48 对的正常波动内）",
                    "caveats": ["众人分布来自 CHiME-7（英语、ACR 相减）；作者是 2AFC + 中英混合会议片段",
                                "SCOREQ 在我们片段上的准确度未测（尚无判据），偏离也可能是模型不在分布内",
                                "exp003 会话陷阱 trap-identical 未过（会话打折）"]}
    (HERE / "results/author_vs_crowd.json").write_text(json.dumps(R, ensure_ascii=False, indent=1))
    print(json.dumps(R, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
