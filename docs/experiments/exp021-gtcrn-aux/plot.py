#!/usr/bin/env python3
"""exp021 取舍曲线：标准库直接写 SVG（本机没有 matplotlib）。

  plot.py --out 图.svg --items V-ONSET,V-BGSTAB,V-SPLVL [--scenes a,b,c] <clips.jsonl>...

系统名形如 <模型>-om<说话时>-fm<停顿>（exp021-remix 起的名）或 <模型>-nomix。
小图矩阵：每行一个评分项；左列扫「说话时混回量」（停顿取默认 −12），右列扫「停顿下限」（说话时取默认 −6）；
每个模型一条线，「不混回」画在最左。y = 所选场景所有段的平均值。
"""

import json
import re
import sys
from collections import defaultdict

DEF_OPEN, DEF_FLOOR = -6, -12
NAME = re.compile(r"^(?P<m>.+?)-(?:om(?P<o>\d+)-fm(?P<f>\d+)|(?P<nomix>nomix))$")
COLORS = ["#2a6fdb", "#d9822b", "#2f9e44", "#c92a2a"]
LABEL = {"V-ONSET": "字头损失 dB（低好）", "V-BGSTAB": "背景稳（低好）", "V-SPLVL": "人声电平 dB（高好）",
         "V-PUMPV": "人声忽大忽小 dB（低好）", "V-NR": "降噪量 dB（低好）", "V-ESTOI": "可懂度变化（高好）"}


def parse_name(s):
    """→ (模型, 说话时 dB, 停顿 dB)；不混回 → (模型, None, None)；不认得 → None。"""
    m = NAME.match(s)
    if not m:
        return None
    if m.group("nomix"):
        return m.group("m"), None, None
    return m.group("m"), -int(m.group("o")), -int(m.group("f"))


def means(rows, items, scenes):
    acc = defaultdict(list)
    for r in rows:
        p = parse_name(r["system"])
        if p is None or (scenes and r["scene"] not in scenes):
            continue
        for s in r["scores"]:
            if s["v"] in items and s.get("value") is not None:
                acc[(p, s["v"])].append(s["value"])
    return {k: sum(v) / len(v) for k, v in acc.items()}


def series(mean, item, model, axis):
    """axis = "open" | "floor"：返回 [(x 标签, y)]，不混回在最前。"""
    pts = []
    nm = mean.get(((model, None, None), item))
    if nm is not None:
        pts.append(("不混回", nm))
    keys = sorted({k[0] for k in mean if k[0][0] == model and k[0][1] is not None and k[1] == item},
                  key=lambda p: p[1] if axis == "open" else p[2])
    for p in keys:
        if axis == "open" and p[2] == DEF_FLOOR:
            pts.append((f"{p[1]}", mean[(p, item)]))
        if axis == "floor" and p[1] == DEF_OPEN:
            pts.append((f"{p[2]}", mean[(p, item)]))
    return pts


def svg(mean, items, models, title):
    W, H, PL, PT, GAP = 300, 150, 70, 40, 30
    width = PL + 2 * (W + GAP) + 140
    height = PT + len(items) * (H + GAP + 20) + 20
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" font-family="sans-serif" font-size="11">',
         f'<rect width="100%" height="100%" fill="#fff"/><text x="10" y="20" font-size="14">{title}</text>']
    for c, ax in enumerate(["open", "floor"]):
        head = "说话时混回量 dB（停顿 −12）" if ax == "open" else "停顿下限 dB（说话时 −6）"
        o.append(f'<text x="{PL + c * (W + GAP)}" y="{PT - 8}">{head}</text>')
    for r, it in enumerate(items):
        y0 = PT + r * (H + GAP + 20)
        allv = [v for (k, i), v in mean.items() if i == it]
        if not allv:
            continue
        lo, hi = min(allv), max(allv)
        pad = (hi - lo) * 0.1 or 0.5
        lo, hi = lo - pad, hi + pad
        o.append(f'<text x="5" y="{y0 + H / 2}" transform="rotate(-90 12 {y0 + H / 2})">{LABEL.get(it, it)}</text>')
        for c, ax in enumerate(["open", "floor"]):
            x0 = PL + c * (W + GAP)
            o.append(f'<rect x="{x0}" y="{y0}" width="{W}" height="{H}" fill="none" stroke="#999"/>')
            o.append(f'<text x="{x0 - 4}" y="{y0 + 10}" text-anchor="end">{hi:.2f}</text>')
            o.append(f'<text x="{x0 - 4}" y="{y0 + H}" text-anchor="end">{lo:.2f}</text>')
            for mi, m in enumerate(models):
                pts = series(mean, it, m, ax)
                if not pts:
                    continue
                n = len(pts)
                xy = [(x0 + (j + 0.5) * W / n, y0 + H - (v - lo) / (hi - lo) * H) for j, (_, v) in enumerate(pts)]
                col = COLORS[mi % len(COLORS)]
                o.append(f'<polyline fill="none" stroke="{col}" stroke-width="2" points="'
                         + " ".join(f"{a:.1f},{b:.1f}" for a, b in xy) + '"/>')
                for (a, b), (lab, _) in zip(xy, pts):
                    o.append(f'<circle cx="{a:.1f}" cy="{b:.1f}" r="3" fill="{col}"/>')
                    if mi == 0:
                        o.append(f'<text x="{a:.1f}" y="{y0 + H + 13}" text-anchor="middle">{lab}</text>')
    for mi, m in enumerate(models):
        y = PT + 10 + mi * 16
        x = PL + 2 * (W + GAP)
        o.append(f'<rect x="{x}" y="{y - 8}" width="10" height="10" fill="{COLORS[mi % len(COLORS)]}"/>'
                 f'<text x="{x + 14}" y="{y}">{m}</text>')
    o.append("</svg>")
    return "\n".join(o) + "\n"


def main(argv):
    def opt(k, d=None):
        return argv[argv.index(k) + 1] if k in argv else d
    out, items = opt("--out"), opt("--items", "").split(",")
    scenes = set(filter(None, opt("--scenes", "").split(",")))
    files = [a for a in argv if a.endswith(".jsonl")]
    if not out or not files or not items[0]:
        print(__doc__)
        return 2
    rows = [json.loads(l) for f in files for l in open(f, encoding="utf-8") if l.strip()]
    mean = means(rows, set(items), scenes)
    models = sorted({k[0][0] for k in mean})
    title = "全部场景" if not scenes else "场景：" + "、".join(sorted(scenes))
    with open(out, "x", encoding="utf-8") as f:
        f.write(svg(mean, items, models, title))
    print(f"图 → {out}（{len(models)} 个模型，{len(items)} 项）")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
