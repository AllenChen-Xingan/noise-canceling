"""exp025 · 两条路对拍：内置流式 `gtcrn` 与 `external:<整段版原版>` 的同一张成绩单，逐段逐项比 `value`。

为什么可能不同：两条路的 STFT 边界、执行顺序、float32 舍入不同。差超过容差 = 两条路口径不同，甲路（外部离线输出）
的读数就不能拿来对原版（`docs/experiments/exp025-gtcrn-train/eval-plan.md` 第二节第 2 步的过线条件）。

用法（项目根下，任一带 numpy 的 python3 都行，本脚本只用标准库）：
  python3 docs/experiments/exp025-gtcrn-train/eval/parity.py --a <内置路>/clips.jsonl --b <外部路>/clips.jsonl
  一份文件里两个系统（`--system gtcrn,external:…` 一次跑出来的）：同一个文件传两次，点名两个系统
  python3 …/parity.py --a <run>/clips.jsonl --b <run>/clips.jsonl --system-a gtcrn --system-b gtcrn-offline

退出码：0 = 全部在容差内；1 = 有超差 · 有只在一边的片段 · 一项都没比成。
"""
import argparse
import json
from pathlib import Path

# 容差（绝对差，按 verifier 原单位）：两条路只在 STFT 边界与 float32 舍入上不同，读数噪声在 1e-6 量级；
# 而最小的判定线是 0.05（V-SIM-GATE 的 max）与 1 dB（V-ONSET / V-TIMBRE）。1e-3 远高于舍入、远低于判定线。
TOL = 1e-3

# 外部离线路**测不了**的项（`src/eval/src/system/external.rs:9-11` 公平性三条）。列出来，不假装比过。
SKIP = {
    "V-LAT": "延迟只是声明值，没有实测（external.rs:9 · provenance `latency_declared_only`）",
    "V-FRAME": "回放不耗时，`engine_times` 为空，最坏一帧判「不知道」（external.rs:10 · physical.rs:53）",
}


def load(path, system=None):
    """读一份成绩单 clips.jsonl → ({片段 id: {项: 值}}, 系统名)。值是 None（不知道 / 没测）也留着。"""
    p = Path(path)
    rows = [json.loads(line) for line in p.read_text().splitlines() if line.strip()]
    if not rows:
        raise SystemExit(f"{p} 是空的")
    systems = sorted({str(r.get("system")) for r in rows})
    if system is None:
        if len(systems) != 1:
            raise SystemExit(f"{p} 里有多个系统 {systems}；用 --system-a / --system-b 指定比哪两个")
        system = systems[0]
    if system not in systems:
        raise SystemExit(f"{p} 里没有系统 {system}（有：{systems}）")
    out = {}
    for r in rows:
        if r.get("system") != system:
            continue
        scores = out.setdefault(r["clip"], {})
        for s in r["scores"]:
            scores[s["v"]] = s.get("value")
    return out, system


def compare(a, b, tol=TOL):
    """a / b 都是 `load` 的字典。返回要打印的一切（不打印、不抛，方便测试）。"""
    only_a, only_b = sorted(set(a) - set(b)), sorted(set(b) - set(a))
    common = sorted(set(a) & set(b))
    items = sorted(({v for c in common for v in a[c]} | {v for c in common for v in b[c]}) - set(SKIP))
    rows, fails = [], []
    for it in items:
        pairs, max_diff, no_value = 0, 0.0, 0
        item_fails = []
        for c in common:
            va, vb = a[c].get(it), b[c].get(it)
            if va is None or vb is None:  # 不知道 / 没测：只计数，不算差
                no_value += 1
                continue
            d = abs(float(va) - float(vb))
            pairs += 1
            max_diff = max(max_diff, d)
            if d > tol:
                item_fails.append({"clip": c, "a": float(va), "b": float(vb), "diff": d})
        rows.append({"item": it, "pairs": pairs, "max_diff": max_diff, "no_value": no_value,
                     "fails": item_fails})
        fails.extend(dict(f, item=it) for f in item_fails)
    return {"rows": rows, "fails": fails, "only_a": only_a, "only_b": only_b,
            "compared": sum(r["pairs"] for r in rows)}


def render(rep, tol=TOL, names=("a", "b"), limit=20):
    """短表：项 · 配对段数 · 最大绝对差 · PASS/FAIL；再列跳过的项与超差的地方。"""
    lines = [f"逐项（容差 {tol:g}；配对 = 两边都有数的段；最多列出 {limit} 条超差）",
             f"  {'项':<16}{'配对':>6}{'最大绝对差':>16}  判定"]
    for r in rep["rows"]:
        if r["pairs"] == 0:
            verdict = "跳过：两边没有共同值"
        elif r["fails"]:
            verdict = f"FAIL（{len(r['fails'])} 段超差）"
        else:
            verdict = "PASS"
        lines.append(f"  {r['item']:<16}{r['pairs']:>6}{r['max_diff']:>16.3e}  {verdict}")
    lines.append(f"共比 {len(rep['rows'])} 项 · {rep['compared']} 对；跳过 {len(SKIP)} 项（外部离线路测不了）：")
    for k, why in SKIP.items():
        lines.append(f"  {k}：{why}")
    for key, other, tag in (("only_a", names[0], "--a"), ("only_b", names[1], "--b")):
        if rep[key]:
            lines.append(f"只在 {tag}（{other}）里出现的片段（{len(rep[key])} 段，配对不成立）：")
            lines.extend(f"  {c}" for c in rep[key][:limit])
    if rep["fails"]:
        lines.append(f"超差的 {len(rep['fails'])} 处（{names[0]} → {names[1]}）：")
        for f in rep["fails"][:limit]:
            lines.append(f"  {f['clip']} {f['item']}：{f['a']!r} → {f['b']!r}（差 {f['diff']:.3e}）")
    lines.append("判定：" + ("PASS（全部在容差内）" if not rep["fails"] and not rep["only_a"] and not rep["only_b"]
                            and rep["compared"] else "FAIL"))
    return "\n".join(lines)


def main(argv=None):
    ap = argparse.ArgumentParser(description="exp025 · 内置 gtcrn 与外部离线原版两条路逐项对拍")
    ap.add_argument("--a", required=True, help="基线成绩单 clips.jsonl（内置流式 gtcrn）")
    ap.add_argument("--b", required=True, help="对照成绩单 clips.jsonl（external:<整段版原版>）")
    ap.add_argument("--system-a", help="--a 里的系统名（文件里只有一个系统时可不给）")
    ap.add_argument("--system-b", help="--b 里的系统名（同上）")
    ap.add_argument("--tol", type=float, default=TOL, help=f"绝对差容差，默认 {TOL:g}（理由见文件头注释）")
    args = ap.parse_args(argv)
    a, sa = load(args.a, args.system_a)
    b, sb = load(args.b, args.system_b)
    rep = compare(a, b, args.tol)
    print(render(rep, args.tol, (sa, sb)))
    bad = rep["fails"] or rep["only_a"] or rep["only_b"] or not rep["compared"]
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
