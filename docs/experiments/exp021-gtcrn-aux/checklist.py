#!/usr/bin/env python3
"""exp021 混回重测的检查表（D90 · 过线标准照调研 030 decision.md 第 21 行）。只用标准库。

  checklist.py <clips.jsonl> <默认系统> <候选系统>... [--sheet12 <−12 档系统>] [--out 表.md]

每个候选档对默认档逐段配对（差 = 候选 − 默认，95% 区间正态近似、每段当独立样本），逐条判，结果三态：
过 / 不过 / 待测（证据不够下结论，不当成过）。
  1. 五个弱场景（街边 · 候车厅 · 车内 · 咖啡馆 · 共享办公）可懂度（V-ESTOI）场景均值差各 ≥ +0.008；缺场景 = 待测
  2. 听感分不降（V-NRMOS-DISTILL · V-NRMOS-SCOREQ，高好）：全部段、五个弱场景合起来各一个配对区间
  4. 背景稳（V-BGSTAB，低好）不许变差 · 5. 旁人（V-INTF，低好）不许变差：全部段一个配对区间
     2 · 4 · 5 的判法（本脚本定的，030 原文没写，codex 第 1 轮 P1 #1）：
     区间整体在好的一侧（高好：下界 ≥ 0；低好：上界 ≤ 0）= 过；整体在坏的一侧 = 不过；跨 0 = 待测。
     没设容忍幅度，所以「过」要求证明不变差，比「没证明变差」严。
  3. 字头（V-ONSET）每个场景均值升 ≤ 0.6 dB
  缺分：任何一段只有一边有分（单边缺测），该条不许判过，最多判待测；两边都没分的段 = 不考这项
--sheet12 另给一张单独检查表（D91 · K11）：弱场景可懂度、两个听感分、吃字（单段多错 ≥ 5 字否决的段数，
按「通过 / 否决 / 不知道」列出两档之间的转移）、字头、人声电平（几个场景均值越过 −3 dB 线）、忽大忽小（V-PUMPV）。
"""

import math
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from analyze import by_clip, load, pair  # noqa: E402

WEAK = ["street-outdoor", "transit-hub", "car-passenger", "cafe", "cowork"]
WEAK_ZH = {"street-outdoor": "街边", "transit-hub": "候车厅", "car-passenger": "车内", "cafe": "咖啡馆", "cowork": "共享办公"}
ESTOI_MIN, ONSET_UP = 0.008, 0.6  # 030 的过线标准（decision.md 第 21 行）
EAT_VETO = 5  # 正本：src/eval/src/thresholds/table.rs `EAT_ASR` 的 fail_chars；test_checklist 读 Rust 源码比对
MOS = ["V-NRMOS-DISTILL", "V-NRMOS-SCOREQ"]
ITEMS = ["V-ESTOI", "V-ONSET", "V-BGSTAB", "V-INTF", "V-SPLVL", "V-PUMPV"] + MOS
PASS, FAIL, TBD = "过", "不过", "待测"


def only(rows, scenes):
    return [r for r in rows if r["scene"] in scenes]


def round_half_away(x):
    """与 Rust `f64::round` 相同：半整数远离 0 取整（Python 的 round 是取偶，codex 第 1 轮 P2 #4）。"""
    return math.copysign(math.floor(abs(x) + 0.5), x)


def eat_chars(s):
    """比原声多错几个字；与成绩单 rules_v2.rs `eat_chars` 同算法：百分点 ÷ 100 × 参考字数，半整数远离 0 取整。"""
    v, n = s.get("value"), (s.get("extra") or {}).get("ref_len")
    return None if v is None or n is None else int(round_half_away(v / 100.0 * n))


def eat_state(rows, system):
    """{片段: ("通过" | "否决" | "不知道", 多错字数或 None)}。"""
    out = {}
    for clip, sc in by_clip(rows, system).items():
        s = sc.get("V-EAT-ASR")
        n = None if s is None else eat_chars(s)
        out[clip] = ("不知道", None) if n is None else ("否决" if n >= EAT_VETO else "通过", n)
    return out


def eat_vetoes(rows, system):
    """{片段: 多错字数}，只收否决的段；另返回判「不知道」的段数。"""
    st = eat_state(rows, system)
    return {c: n for c, (k, n) in st.items() if k == "否决"}, sum(1 for k, _ in st.values() if k == "不知道")


def eat_moves(rows, default, cand):
    """两档之间吃字状态的转移：{(默认状态, 候选状态): [片段]}，只收状态变了的；另返回只在一边出现的片段数。"""
    a, b = eat_state(rows, default), eat_state(rows, cand)
    moves = defaultdict(list)
    for c in sorted(set(a) & set(b)):
        if a[c][0] != b[c][0]:
            moves[(a[c][0], b[c][0])].append((c, b[c][1]))
    return dict(moves), len(set(a) ^ set(b))


def line_of(rows, item):
    """成绩单记在每段分数里的过关线（V-ONSET 3 dB、V-SPLVL −3 dB 等）；不手抄。各段不一致就报错。"""
    th = {s.get("threshold") for r in rows for s in r["scores"] if s["v"] == item and s.get("threshold") is not None}
    if len(th) != 1:
        raise ValueError(f"{item} 的过关线不唯一：{sorted(th)}")
    return th.pop()


def scene_means(rows, system, item):
    acc = defaultdict(list)
    for r in rows:
        if r["system"] != system:
            continue
        for s in r["scores"]:
            if s["v"] == item and s.get("value") is not None:
                acc[r["scene"]].append(s["value"])
    return {k: sum(v) / len(v) for k, v in acc.items()}


def ci_state(r, higher_better):
    """配对区间 → 过 / 不过 / 待测（见文件头 2 · 4 · 5）。没有区间 = 待测。"""
    if r is None:
        return TBD
    lo, hi = (r["lo"], r["hi"]) if higher_better else (-r["hi"], -r["lo"])
    return PASS if lo >= 0 else FAIL if hi < 0 else TBD


def worst(states):
    return FAIL if FAIL in states else TBD if TBD in states else PASS


def one_sided(rows, default, cand, item):
    """两档里只有一边有分的段（单边缺测）。两边都没分 = 这段不考这项，不算缺测（codex 第 2 轮 P2 #1）。"""
    a, b = by_clip(rows, default), by_clip(rows, cand)
    has = lambda d, c: d.get(c, {}).get(item, {}).get("value") is not None  # noqa: E731
    return sorted(c for c in set(a) | set(b) if has(a, c) != has(b, c))


def guard(state, miss):
    """有单边缺测时不许判过：过 → 待测；不过照旧不过。"""
    return TBD if miss and state == PASS else state


def note(miss):
    return f"；单边缺分 {len(miss)} 段：{', '.join(miss[:5])}{' …' if len(miss) > 5 else ''}" if miss else ""


def judge(rows, default, cand):
    """返回 [(条目, 过 / 不过 / 待测, 说明)]。"""
    allp = pair(rows, default, rows, cand, ITEMS)
    weakp = pair(only(rows, WEAK), default, only(rows, WEAK), cand, MOS)
    miss = {v: one_sided(rows, default, cand, v) for v in ITEMS}
    res = []
    es = allp.get("V-ESTOI", {}).get("scenes", {})
    st = [TBD if es.get(w) is None else PASS if es[w] >= ESTOI_MIN else FAIL for w in WEAK]
    m = one_sided(only(rows, WEAK), default, cand, "V-ESTOI")
    res.append(("弱场景可懂度各 ≥ +0.008", guard(worst(st), m),
                " · ".join(f"{WEAK_ZH[w]} {fmt_x(es.get(w), 4)}" for w in WEAK) + note(m)))
    for v in MOS:
        a, w = allp.get(v), weakp.get(v)
        res.append((f"听感分 {v} 不降", guard(worst([ci_state(a, True), ci_state(w, True)]), miss[v]),
                    f"全部 {fmt_ci(a)} · 弱场景 {fmt_ci(w)}" + note(miss[v])))
    on = allp.get("V-ONSET", {}).get("scenes", {})
    over = sorted((k for k, x in on.items() if x > ONSET_UP), key=lambda k: -on[k])
    why = (f"最大 {max(on.values()):+.2f} dB；" if on else "") + f"越线 {len(over)} 个场景" \
        + (f"：{fmt_sc(on, over)}" if over else "") + note(miss["V-ONSET"])
    res.append(("字头每场景升 ≤ 0.6 dB", FAIL if over else guard(PASS if on else TBD, miss["V-ONSET"]), why))
    for v, name in (("V-BGSTAB", "背景稳"), ("V-INTF", "旁人")):
        a = allp.get(v)
        res.append((f"{name}不变差", guard(ci_state(a, False), miss[v]), f"全部 {fmt_ci(a)}（低好）" + note(miss[v])))
    return res


def sheet12(rows, default, cand):
    """−12 档单独检查表的各行：[(项, 默认档, −12 档, 说明)]。"""
    out = []
    p = pair(rows, default, rows, cand, ITEMS)
    es = p["V-ESTOI"]["scenes"]
    out.append(("弱场景可懂度（配对差）", "—", " · ".join(f"{WEAK_ZH[w]} {fmt_x(es.get(w), 4)}" for w in WEAK), ""))
    wp = pair(only(rows, WEAK), default, only(rows, WEAK), cand, MOS)
    for v in MOS:
        out.append((f"听感分 {v}", "—", f"全部 {fmt_ci(p.get(v))} · 弱场景 {fmt_ci(wp.get(v))}", "高好"))
    v0, u0 = eat_vetoes(rows, default)
    v1, u1 = eat_vetoes(rows, cand)
    moves, odd = eat_moves(rows, default, cand)
    desc = "；".join(f"{a}→{b} {len(cs)} 段：" + ", ".join(c + (f"（{n} 字）" if b == "否决" else "") for c, n in cs)
                    for (a, b), cs in sorted(moves.items())) or "无变化"
    out.append(("吃字否决段数（单段多错 ≥ 5 字）", f"{len(v0)}（不知道 {u0}）", f"{len(v1)}（不知道 {u1}）",
                desc + (f"；只在一档出现的片段 {odd} 个" if odd else "")))
    on = p["V-ONSET"]["scenes"]
    out.append(("字头（配对差，每场景）", "—", f"全部 {fmt_ci(p['V-ONSET'])}；最大 {max(on.values()):+.2f} dB",
                f"升 > 0.6 dB 的场景 {sum(1 for x in on.values() if x > ONSET_UP)} 个"))
    ol = line_of(rows, "V-ONSET")
    o0, o1 = scene_means(rows, default, "V-ONSET"), scene_means(rows, cand, "V-ONSET")
    out.append((f"字头场景均值超过成绩单 {ol:g} dB 线的场景数", str(sum(1 for x in o0.values() if x > ol)),
                str(sum(1 for x in o1.values() if x > ol)), "、".join(sorted(k for k, x in o1.items() if x > ol))))
    sl = line_of(rows, "V-SPLVL")
    s0, s1 = scene_means(rows, default, "V-SPLVL"), scene_means(rows, cand, "V-SPLVL")
    l0 = sorted(k for k, x in s0.items() if x < sl)
    l1 = sorted(k for k, x in s1.items() if x < sl)
    out.append((f"人声电平越过 {sl:g} dB 线的场景数", str(len(l0)), str(len(l1)), "、".join(l1) or "无"))
    out.append(("忽大忽小（配对差）", "—", fmt_ci(p.get("V-PUMPV")), "低好；按 D85 只报不判"))
    return out


def fmt_x(x, nd):
    return "缺" if x is None else f"{x:+.{nd}f}"


def fmt_ci(r):
    return "—" if r is None else f"{r['mean']:+.4f} [{r['lo']:+.4f}, {r['hi']:+.4f}]（{r['n']} 段）"


def fmt_sc(d, keys):
    return "、".join(f"{WEAK_ZH.get(k, k)} {d[k]:+.2f}" for k in keys)


def coverage(rows, default, cands):
    """每档每项有分的段数（缺分的段不进配对，codex 第 1 轮 P2 #3）。"""
    L = ["| 项 | " + " | ".join(f"`{s}`" for s in [default] + cands) + " |", "|---" * (len(cands) + 2) + "|"]
    for v in ITEMS + ["V-EAT-ASR"]:
        n = [sum(1 for sc in by_clip(rows, s).values() if sc.get(v, {}).get("value") is not None)
             for s in [default] + cands]
        L.append(f"| {v} | " + " | ".join(map(str, n)) + " |")
    return L


def render(rows, default, cands, s12):
    L = [f"默认档 `{default}`；差 = 候选 − 默认；95% 区间正态近似，每段当独立样本。判定三态：过 / 不过 / 待测。", "",
         "## 各项有分的段数", ""] + coverage(rows, default, cands) + [""]
    for c in cands:
        res = judge(rows, default, c)
        verdict = {PASS: "全部过线", FAIL: "未过线", TBD: "待测"}[worst([s for _, s, _ in res])]
        L += [f"## `{c}`：{verdict}", "", "| 条目 | 判定 | 读数 |", "|---|---|---|"]
        L += [f"| {k} | {s} | {why} |" for k, s, why in res]
        L.append("")
    if s12:
        L += [f"## −12 档单独检查表：`{s12}`", "", "| 项 | 默认档 | −12 档 | 说明 |", "|---|---|---|---|"]
        L += [f"| {a} | {b} | {c} | {d} |" for a, b, c, d in sheet12(rows, default, s12)]
        L.append("")
    return "\n".join(L)


def main(argv):
    argv = list(argv)
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
