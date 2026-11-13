"""exp022 · 评估器对人耳：分开报真阳性率（抓到坏的）与真阴性率（不冤枉好的）。

数据（全部已在仓里，不重跑模型）：
- 人耳：exp003 build f8c3 答卷，12 屏正式题 × 3 个模型，1 名听者，会话打折（全同陷阱没过）
  - 「开会能用吗」ok / marginal / no → 两种切法：宽（no = 不过）· 严（no 或 marginal = 不过）
  - 「哪份吃字」勾选 → 勾了 = 不过
- 评估器：
  - 听感：exp009 对作者实际听到的那份音频打的 SCOREQ · Distill-MOS，Δ = 输出 − 带噪原声
  - 吃字：exp003 scores.json 的语音识别 删 + 替，比原声多错几个词 / 字
- 过关线一律运行时从正本 `src/eval/src/thresholds/table.rs` 读，脚本里不写数字

用法：python3 docs/experiments/exp022-judge-tpr-tnr/tpr_tnr.py   （秒级，不要名额）
输出：results/tpr_tnr.json · results/tpr_tnr.md
"""
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE))
import stats as S  # noqa: E402

ANSWERS = ROOT / "docs/experiments/exp003-offline-ab/results/20261030-human-build-20261030110755-f8c3.json"
PRED = ROOT / "docs/experiments/exp009-more-ears/results/exp003_predictors.json"
SCORES = ROOT / "docs/experiments/exp003-offline-ab/scores.json"
TABLE = ROOT / "src/eval/src/thresholds/table.rs"
OUT = HERE / "results"
F, P, U = S.FAIL, S.PASS, S.UNDECIDED


# ── 过关线：从 Rust 正本读 ──────────────────────────────────────────
def _param(src, lines_id, key):
    i = src.find(f'id: "{lines_id}"')
    if i < 0:
        raise KeyError(f"table.rs 里没有 {lines_id}")
    block = src[i:src.find("};", i)]
    m = re.search(r'p\(\s*"' + re.escape(key) + r'",\s*Some\(([-0-9.]+)\)', block)
    if not m:
        m = re.search(r'p\(\s*"' + re.escape(key) + r'",\s*None,\s*&\[\(([-0-9.]+)', block)
    if not m:
        raise KeyError(f"table.rs 的 {lines_id} 里没有参数 {key}")
    return float(m.group(1))


def read_lines(path=TABLE):
    src = path.read_text(encoding="utf-8")
    return {
        "scoreq_min_v1": _param(src, "V-NRMOS-SCOREQ", "min"),
        "distill_min_v1": _param(src, "V-NRMOS-DISTILL", "min"),
        "nrmos_slack_v2": _param(src, "SC-NRMOS-DELTA", "max"),
        "eat_pp_v1": _param(src, "V-EAT-ASR", "max"),
        "eat_tokens_v2": _param(src, "V-EAT-ASR", "fail_chars"),
    }


# ── 判定函数（单测覆盖）────────────────────────────────────────────
def delta_verdict(delta, line):
    return P if delta >= line - 1e-12 else F


def both(a, b):
    return a if a == b else U


def _extra(c, noisy):
    return (c["del"] + c["sub"]) - (noisy["del"] + noisy["sub"])


def eat_tokens(c, noisy, lim):
    """D85 题 3：单段比原声多错（删 + 替）≥ lim 个词 / 字 = 不过。"""
    return F if _extra(c, noisy) >= lim else P


def eat_pp(c, noisy, lim):
    """rubric V7：多错百分点 > lim = 不过。"""
    return F if _extra(c, noisy) / max(1, c["ref_len"]) * 100 > lim + 1e-12 else P


def human_usable(v, strict):
    if v not in ("ok", "marginal", "no"):
        return None
    if v == "no":
        return F
    return F if (strict and v == "marginal") else P


# ── 组表 ────────────────────────────────────────────────────────────
def build_rows(answers, pred, scores, lines):
    rows = []
    for t in answers["trials"]:
        if t.get("kind") != "real":
            continue
        clip, bm = t["clip"], t["by_model"]
        eaten = set(bm.get("eaten") or [])
        for model in t["slots"].values():
            p, n = pred[clip][model], pred[clip]["noisy"]
            sc, sn = scores["per_clip"][clip][model], scores["per_clip"][clip]["noisy"]
            dq, dd = p["scoreq"] - n["scoreq"], p["distill_mos"] - n["distill_mos"]
            v1q = delta_verdict(dq, lines["scoreq_min_v1"])
            v1d = delta_verdict(dd, lines["distill_min_v1"])
            slack = -lines["nrmos_slack_v2"]
            rows.append({
                "clip": clip, "model": model,
                "usable": bm["usable"].get(model),
                "h_loose": human_usable(bm["usable"].get(model), strict=False),
                "h_strict": human_usable(bm["usable"].get(model), strict=True),
                "h_eaten": F if model in eaten else P,
                "d_scoreq": round(dq, 4), "d_distill": round(dd, 4),
                "extra_tokens": _extra(sc, sn), "ref_len": sc["ref_len"],
                "j_scoreq_v1": v1q, "j_distill_v1": v1d, "j_both_v1": both(v1q, v1d),
                "j_scoreq_slack": delta_verdict(dq, slack),
                "j_distill_slack": delta_verdict(dd, slack),
                "j_eat_tokens": eat_tokens(sc, sn, lines["eat_tokens_v2"]),
                "j_eat_pp": eat_pp(sc, sn, lines["eat_pp_v1"]),
            })
    return rows


JUDGES = [
    ("听感 · SCOREQ 单段 Δ ≥ 0（旧规则 v1 的单段线）", "j_scoreq_v1", ["h_loose", "h_strict"]),
    ("听感 · Distill-MOS 单段 Δ ≥ 0", "j_distill_v1", ["h_loose", "h_strict"]),
    ("听感 · 两模型合判（v1：一过一不过 = 待测）", "j_both_v1", ["h_loose", "h_strict"]),
    ("听感 · SCOREQ 单段 Δ ≥ −0.1（D85 的线若用在单段上）", "j_scoreq_slack", ["h_loose", "h_strict"]),
    ("听感 · Distill-MOS 单段 Δ ≥ −0.1", "j_distill_slack", ["h_loose", "h_strict"]),
    ("吃字 · 识别多错 ≥ 5 个词/字（D85 题 3）", "j_eat_tokens", ["h_eaten"]),
    ("吃字 · 识别多错 > 1 个百分点（rubric V7）", "j_eat_pp", ["h_eaten"]),
]
HUMAN = {
    "h_loose": "人耳「开会能用」宽切：no = 不过",
    "h_strict": "人耳「开会能用」严切：no 或 marginal = 不过",
    "h_eaten": "人耳勾选「吃字」= 不过",
}


def evaluate(rows, reps=10000):
    out = []
    for name, jk, hks in JUDGES:
        for hk in hks:
            items = [(r[hk], r[jk]) for r in rows if r[hk] is not None]
            by_clip = {}
            for r in rows:
                if r[hk] is not None:
                    by_clip.setdefault(r["clip"], []).append((r[hk], r[jk]))
            c = S.confusion(items)
            res = S.rates(c)
            fail_obs = (c["tp"] + c["fp"]) / res["n_decided"] if res["n_decided"] else float("nan")
            res.update({
                "judge": name, "judge_key": jk, "human": HUMAN[hk], "human_key": hk,
                "confusion": c,
                "tpr_ci95_cluster": S.cluster_bootstrap(
                    by_clip, lambda xs: S.rates(S.confusion(xs))["tpr"], reps),
                "tnr_ci95_cluster": S.cluster_bootstrap(
                    by_clip, lambda xs: S.rates(S.confusion(xs))["tnr"], reps),
                "judge_fail_rate": fail_obs,
                "human_fail_rate": res["human_fail"] / res["n_decided"] if res["n_decided"] else float("nan"),
                "rogan_gladen_fail_rate": S.rogan_gladen(fail_obs, res["tpr"], res["tnr"]),
            })
            out.append(res)
    return out


def _f(x):
    if x is None:
        return "无解"
    return "—" if x != x else f"{x:.2f}"


def _ci(t):
    return f"[{_f(t[0])}, {_f(t[1])}]"


def to_markdown(res, lines, n_rows):
    s = ["# exp022 结果 · 评估器对人耳的真阳性率 / 真阴性率\n",
         "由 `tpr_tnr.py` 生成，勿手改。「阳性」= 不过。真阳性率 = 人说不过的里面评估器也判不过；"
         "真阴性率 = 人说过的里面评估器也判过。\n",
         f"- 样本：{n_rows} 条（12 屏 × 3 个模型），**1 名听者**，会话打折（全同陷阱没过）。"
         "同一屏 3 条不独立，所以另给按屏重采样的区间（10000 次，种子 20261030）",
         f"- 过关线（运行时读 `src/eval/src/thresholds/table.rs`）：`{json.dumps(lines, ensure_ascii=False)}`",
         "- 「全判过时的一致率」= 评估器一律判过能拿到的总一致率；总一致率不高于它，说明总一致率没有信息量",
         "- 「校正后不过比例」= Rogan-Gladen：按本表 TPR/TNR 校正评估器判出的不过比例；TPR + TNR ≤ 1 时无解\n",
         "| 评估器 | 人耳真值 | 人不过 / 人过 | 待测 | TP FN TN FP | 真阳性率 [Wilson] [按屏] | 真阴性率 [Wilson] [按屏] | 总一致率 | 全判过时的一致率 | TPR+TNR−1 | 评估器判不过 / 人判不过 / 校正后 |",
         "|---|---|---|---|---|---|---|---|---|---|---|"]
    for r in res:
        c = r["confusion"]
        s.append(
            f"| {r['judge']} | {r['human']} | {r['human_fail']} / {r['human_pass']} | {r['n_undecided']} "
            f"| {c['tp']} {c['fn']} {c['tn']} {c['fp']} "
            f"| {_f(r['tpr'])} {_ci(r['tpr_ci95'])} {_ci(r['tpr_ci95_cluster'])} "
            f"| {_f(r['tnr'])} {_ci(r['tnr_ci95'])} {_ci(r['tnr_ci95_cluster'])} "
            f"| {_f(r['raw_agreement'])} | {_f(r['always_pass_agreement'])} | {_f(r['youden_j'])} "
            f"| {_f(r['judge_fail_rate'])} / {_f(r['human_fail_rate'])} / {_f(r['rogan_gladen_fail_rate'])} |")
    return "\n".join(s) + "\n"


def main():
    lines = read_lines()
    rows = build_rows(json.loads(ANSWERS.read_text()), json.loads(PRED.read_text()),
                      json.loads(SCORES.read_text()), lines)
    res = evaluate(rows)
    OUT.mkdir(exist_ok=True)
    (OUT / "tpr_tnr.json").write_text(json.dumps(
        {"lines": lines, "rows": rows, "results": res}, ensure_ascii=False, indent=1) + "\n")
    (OUT / "tpr_tnr.md").write_text(to_markdown(res, lines, len(rows)))
    print(f"写出 {OUT / 'tpr_tnr.md'}（{len(rows)} 条）")


if __name__ == "__main__":
    main()
