"""exp006 盲听揭盲：页面里没有答案（只导出「字母 → 槽位 slotK」），这里用仓里的对照表把槽位换成版本，再汇总。

对照表：docs/experiments/exp006-scene-sim/builds/<数据 build>/pages/<页面 build>/unblind-key.json（生成页面时当场提交）
答卷：人导出的 JSON，交回后先原样放进 docs/experiments/exp006-scene-sim/results/ 并提交，再跑本脚本

汇总（每个版本）：最清楚 / 最自然被选次数 · 开会 能用 / 勉强 / 不能用 · 吃字次数 · 各毛病标签次数；
两两（同一段内）：修音链开 vs 关（polish vs base、both vs route）、路由开 vs 关（route vs base、both vs polish）——
先比「开会」评级，平级再看谁被选为最清楚（同 exp003）；检验题：全同题应答「分不出」，劣化题不该选到劣化版。

用法：python3 docs/experiments/exp006-scene-sim/unblind.py <答卷.json> [--data-build 20261030214530-44df]
输出：同目录写 <答卷>.unblinded.json，并打印汇总
"""
import json
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
VERS = ["base", "route", "polish", "both"]
RANK = {"ok": 2, "marginal": 1, "no": 0}


def main():
    ans_path = Path(sys.argv[1])
    data_build = sys.argv[sys.argv.index("--data-build") + 1] if "--data-build" in sys.argv else "20261030214530-44df"
    ans = json.loads(ans_path.read_text())
    key = json.loads((HERE / "builds" / data_build / "pages" / ans["build"] / "unblind-key.json").read_text())
    traps = {t["id"]: t["names"] for t in key["traps"]}
    trap_kind = {t["id"]: t.get("kind", t["id"].removeprefix("trap-")) for t in key["traps"]}

    def ver(trial, slot):  # slot 形如 "slot2"；"same" / "none" 原样
        if not isinstance(slot, str) or not slot.startswith("slot"):
            return slot
        i = int(slot[4:])
        return traps[trial["trap"]][i] if trial["kind"] == "trap" else key["key"][trial["clip"]][i]

    per = {v: {"clearer_best": 0, "natural_best": 0, "eaten": 0, "usable": Counter(), "tags": Counter()} for v in VERS}
    rows, trap_res = [], []
    for t in ans["trials"]:
        bm = t["by_model"]
        u = {ver(t, s): x for s, x in bm["usable"].items()}
        r = {"clip": t.get("clip"), "kind": t["kind"], "trap": t.get("trap"),
             "clearer": ver(t, bm["clearer"]), "natural": ver(t, bm["natural"]),
             "eaten": [ver(t, s) for s in bm["eaten"]], "usable": u,
             "tags": {ver(t, s): v for s, v in bm["tags"].items()}, "note": t.get("note", "")}
        rows.append(r)
        if t["kind"] == "trap":
            if trap_kind[t["trap"]] == "identical":
                ok = r["clearer"] == "same" and r["natural"] == "same"
            else:
                ok = r["clearer"] != "degraded" and r["natural"] != "degraded"
            trap_res.append({"trap": t["trap"], "kind": trap_kind[t["trap"]], "pass": ok, "clearer": r["clearer"], "natural": r["natural"]})
            continue
        if t["kind"] != "real":
            continue
        for q in ("clearer", "natural"):
            if r[q] in per:
                per[r[q]][q + "_best"] += 1
        for v in r["eaten"]:
            if v in per:
                per[v]["eaten"] += 1
        for v, x in u.items():
            if v in per:
                per[v]["usable"][x] += 1
        for v, tg in r["tags"].items():
            if v in per:
                per[v]["tags"].update(tg)

    def h2h(a, b):
        o = {a: 0, b: 0, "tie": 0}
        for r in rows:
            if r["kind"] != "real":
                continue
            ua, ub = RANK.get(r["usable"].get(a), -1), RANK.get(r["usable"].get(b), -1)
            if ua > ub or (ua == ub and r["clearer"] == a):
                o[a] += 1
            elif ub > ua or (ua == ub and r["clearer"] == b):
                o[b] += 1
            else:
                o["tie"] += 1
        return o

    out = {"page_build": ans["build"], "data_build": data_build, "answer_file": ans_path.name,
           "per_version": {v: {**p, "usable": dict(p["usable"]), "tags": dict(p["tags"])} for v, p in per.items()},
           "head_to_head": {f"{a}_vs_{b}": h2h(a, b) for a, b in [("polish", "base"), ("both", "route"), ("route", "base"), ("both", "polish"), ("both", "base")]},
           "traps": trap_res, "trials": rows}
    dst = ans_path.with_suffix(".unblinded.json")
    dst.write_text(json.dumps(out, ensure_ascii=False, indent=1))
    print(json.dumps({k: out[k] for k in ("per_version", "head_to_head", "traps")}, ensure_ascii=False, indent=1))
    print("→", dst)


if __name__ == "__main__":
    main()
