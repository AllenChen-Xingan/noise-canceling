#!/usr/bin/env python3
"""exp021 混回重测的检查表（D90 · 过线标准照调研 030 decision.md 第 21 行）。只用标准库。

  checklist.py <clips.jsonl> <默认系统> <候选系统>... [--sheet12 <−12 档系统>] [--out 表.md]

每个候选档对默认档逐段配对（差 = 候选 − 默认，95% 区间正态近似、每段当独立样本），逐条判：
  1. 五个弱场景（街边 · 候车厅 · 车内 · 咖啡馆 · 共享办公）可懂度（V-ESTOI）场景均值差各 ≥ +0.008
  2. 听感分不降（V-NRMOS-DISTILL · V-NRMOS-SCOREQ，高好）：全部段、五个弱场景合起来，两处配对区间上界都 ≥ 0
     （「不降」= 没有确定变差；口径是本脚本定的，不是 030 原文写的）
  3. 字头（V-ONSET）每个场景均值升 ≤ 0.6 dB
  4. 背景稳（V-BGSTAB，低好）不许变差：全部段配对区间下界 ≤ 0
  5. 旁人（V-INTF，低好）不许变差：同上
--sheet12 另给一张单独检查表（D91 · K11）：弱场景可懂度、两个听感分、吃字（单段多错 ≥ 5 字否决的段数，
按片段列出新增的）、字头、人声电平（几个场景均值越过 −3 dB 线）、忽大忽小（V-PUMPV）。
"""

import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from analyze import by_clip, load, pair  # noqa: E402

WEAK = ["street-outdoor", "transit-hub", "car-passenger", "cafe", "cowork"]
WEAK_ZH = {"street-outdoor": "街边", "transit-hub": "候车厅", "car-passenger": "车内", "cafe": "咖啡馆", "cowork": "共享办公"}
ESTOI_MIN, ONSET_UP, SPLVL_LINE, EAT_VETO = 0.008, 0.6, -3.0, 5
MOS = ["V-NRMOS-DISTILL", "V-NRMOS-SCOREQ"]
ITEMS = ["V-ESTOI", "V-ONSET", "V-BGSTAB", "V-INTF", "V-SPLVL", "V-PUMPV"] + MOS


def only(rows, scenes):
    return [r for r in rows if r["scene"] in scenes]


def eat_chars(s):
    """比原声多错几个字；与成绩单 rules_v2.rs `eat_chars` 同算法：百分点 ÷ 100 × 参考字数，四舍五入。"""
    v, n = s.get("value"), (s.get("extra") or {}).get("ref_len")
    return None if v is None or n is None else round(v / 100.0 * n)


def eat_vetoes(rows, system):
    """{片段: 多错字数}，只收 ≥ EAT_VETO 的段；另返回判「不知道」的段数。"""
    out, unk = {}, 0
    for clip, sc in by_clip(rows, system).items():
        s = sc.get("V-EAT-ASR")
        n = None if s is None else eat_chars(s)
        if n is None:
            unk += 1
        elif n >= EAT_VETO:
            out[clip] = n
    return out, unk


def scene_means(rows, system, item):
    acc = defaultdict(list)
    for r in rows:
        if r["system"] != system:
            continue
        for s in r["scores"]:
            if s["v"] == item and s.get("value") is not None:
                acc[r["scene"]].append(s["value"])
    return {k: sum(v) / len(v) for k, v in acc.items()}


def judge(rows, default, cand):
    """返回 [(条目, 是否过, 说明)]。"""
    allp = pair(rows, default, rows, cand, ITEMS)
    weakp = pair(only(rows, WEAK), default, only(rows, WEAK), cand, MOS)
    res = []
    es = allp["V-ESTOI"]["scenes"]
    bad = [w for w in WEAK if es.get(w) is None or es[w] < ESTOI_MIN]
    res.append(("弱场景可懂度各 ≥ +0.008", not bad,
                " · ".join(f"{WEAK_ZH[w]} {es.get(w, float('nan')):+.4f}" for w in WEAK)))
    for v in MOS:
        a, w = allp.get(v), weakp.get(v)
        ok = a is not None and w is not None and a["hi"] >= 0 and w["hi"] >= 0
        res.append((f"听感分 {v} 不降", ok, f"全部 {fmt_ci(a)} · 弱场景 {fmt_ci(w)}"))
    on = allp["V-ONSET"]["scenes"]
    over = sorted((k for k, x in on.items() if x > ONSET_UP), key=lambda k: -on[k])
    res.append(("字头每场景升 ≤ 0.6 dB", not over,
                f"最大 {max(on.values()):+.2f} dB；越线 {len(over)} 个场景" + (f"：{fmt_sc(on, over)}" if over else "")))
    for v, name in (("V-BGSTAB", "背景稳"), ("V-INTF", "旁人")):
        a = allp.get(v)
        res.append((f"{name}不变差", a is not None and a["lo"] <= 0, f"全部 {fmt_ci(a)}（低好）"))
    return res


def sheet12(rows, default, cand):
    """−12 档单独检查表的各行：[(项, 默认档, −12 档, 说明)]。"""
    out = []
    p = pair(rows, default, rows, cand, ITEMS)
    es = p["V-ESTOI"]["scenes"]
    out.append(("弱场景可懂度（配对差）", "—", " · ".join(f"{WEAK_ZH[w]} {es.get(w, float('nan')):+.4f}" for w in WEAK), ""))
    wp = pair(only(rows, WEAK), default, only(rows, WEAK), cand, MOS)
    for v in MOS:
        out.append((f"听感分 {v}", "—", f"全部 {fmt_ci(p.get(v))} · 弱场景 {fmt_ci(wp.get(v))}", "高好"))
    v0, u0 = eat_vetoes(rows, default)
    v1, u1 = eat_vetoes(rows, cand)
    new = sorted(set(v1) - set(v0))
    out.append(("吃字否决段数（单段多错 ≥ 5 字）", f"{len(v0)}（不知道 {u0}）", f"{len(v1)}（不知道 {u1}）",
                "新增：" + (", ".join(f"{c}（{v1[c]} 字）" for c in new) if new else "无")
                + ("；消失：" + ", ".join(sorted(set(v0) - set(v1))) if set(v0) - set(v1) else "")))
    on = p["V-ONSET"]["scenes"]
    out.append(("字头（配对差，每场景）", "—", f"全部 {fmt_ci(p['V-ONSET'])}；最大 {max(on.values()):+.2f} dB",
                f"升 > 0.6 dB 的场景 {sum(1 for x in on.values() if x > ONSET_UP)} 个"))
    s0, s1 = scene_means(rows, default, "V-SPLVL"), scene_means(rows, cand, "V-SPLVL")
    l0 = sorted(k for k, x in s0.items() if x < SPLVL_LINE)
    l1 = sorted(k for k, x in s1.items() if x < SPLVL_LINE)
    out.append(("人声电平越过 −3 dB 线的场景数", str(len(l0)), str(len(l1)), "、".join(l1) or "无"))
    out.append(("忽大忽小（配对差）", "—", fmt_ci(p.get("V-PUMPV")), "低好；按 D85 只报不判"))
    return out


def fmt_ci(r):
    return "—" if r is None else f"{r['mean']:+.4f} [{r['lo']:+.4f}, {r['hi']:+.4f}]（{r['n']} 段）"


def fmt_sc(d, keys):
    return "、".join(f"{WEAK_ZH.get(k, k)} {d[k]:+.2f}" for k in keys)


def render(rows, default, cands, s12):
    L = [f"默认档 `{default}`；差 = 候选 − 默认；95% 区间正态近似，每段当独立样本。", ""]
    for c in cands:
        res = judge(rows, default, c)
        L += [f"## `{c}`：{'全部过线' if all(ok for _, ok, _ in res) else '未过线'}", "",
              "| 条目 | 过线 | 读数 |", "|---|---|---|"]
        L += [f"| {k} | {'是' if ok else '否'} | {why} |" for k, ok, why in res]
        L.append("")
    if s12:
        L += [f"## −12 档单独检查表：`{s12}`", "", "| 项 | 默认档 | −12 档 | 说明 |", "|---|---|---|---|"]
        L += [f"| {a} | {b} | {c} | {d} |" for a, b, c, d in sheet12(rows, default, s12)]
        L.append("")
    return "\n".join(L)


def main(argv):
    if len(argv) < 3:
        print(__doc__)
        return 2
    def opt(k):
        if k in argv:
            i = argv.index(k)
            v = argv[i + 1]
            del argv[i:i + 2]
            return v
        return None
    out, s12 = opt("--out"), opt("--sheet12")
    rows = load(argv[0])
    text = render(rows, argv[1], argv[2:], s12)
    if out:
        with open(out, "x", encoding="utf-8") as f:
            f.write(text)
    print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
