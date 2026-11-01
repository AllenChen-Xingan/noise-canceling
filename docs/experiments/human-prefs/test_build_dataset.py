"""build_dataset.py 自测：不碰仓里的输出文件。

1. 现有答卷：揭盲结果与各实验 README 里人工汇总的数逐项对得上
2. exp006 接口：按页面导出格式（字母 → "slotK"）造一份假答卷，确认能揭盲、出配对、检验题判对
3. 两份逐字节相同的音频被分出胜负 → 该对标 invalid

用法：python3 docs/experiments/human-prefs/test_build_dataset.py
"""
import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import build_dataset as bd  # noqa: E402


def check(cond, msg):
    print(("ok   " if cond else "FAIL ") + msg)
    if not cond:
        check.failed = True


check.failed = False


def test_existing():
    trials, pairs = bd.collect()
    t3 = [r for r in trials if r["build"] == "20261030110755-f8c3" and r["kind"] == "real"]
    cnt = lambda q, m: sum(r["answers"][q] == m for r in t3)
    # exp003 results/README.md「逐项计数」
    check([cnt("clearer", m) for m in ("dpdfnet2", "gtcrn", "fastenhancer_m")] == [1, 5, 3], "exp003 最清楚 1/5/3")
    check([cnt("natural", m) for m in ("dpdfnet2", "gtcrn", "fastenhancer_m")] == [1, 4, 3], "exp003 最自然 1/4/3")
    no = lambda m: sum(r["answers"]["usable"].get(m) == "no" for r in t3)
    check([no(m) for m in ("dpdfnet2", "gtcrn", "fastenhancer_m")] == [8, 5, 7], "exp003 不能用 8/5/7")
    # exp005 results/human-feedback.md
    t5 = [r for r in trials if r["exp"] == "exp005" and r["kind"] == "real"]
    nat = [sum(r["answers"]["natural"] == m for r in t5) for m in ("raw", "personal", "aggressive")]
    pick = [sum(r["answers"]["pick"] == m for r in t5) for m in ("raw", "personal", "aggressive")]
    eat = [sum(m in r["answers"]["eaten"] for r in t5) for m in ("raw", "personal", "aggressive")]
    check(nat == [0, 7, 1], f"exp005 最自然 0/7/1（得 {nat}）")
    check(pick == [2, 3, 3], f"exp005 开会选 2/3/3（得 {pick}）")
    check(eat == [2, 0, 4], f"exp005 吃字 2/0/4（得 {eat}）")
    # 旧 build 可揭盲的两题（exp003 results/README.md「部分揭盲」）
    t68 = {r["clip"]: r for r in trials if r["build"] == "20261030102301-68ab" and not r.get("unblindable")}
    check(t68["c02"]["answers"]["clearer"] == "gtcrn" and t68["c01"]["answers"]["clearer"] == "dpdfnet2", "68ab c02→GTCRN、c01→DPDFNet-2")
    # 相同音频
    inv = [p for p in pairs if p["strength"] == "invalid"]
    check(len(inv) == 1 and inv[0]["clip"] == "c08" and inv[0]["dim"] == "meeting", "exp005 c08 激进 = 只有 GTCRN 同一文件，开会选择那一对标 invalid")


def fake_exp006(page, pick_trap_identical, pick_degraded, extra_self=False):
    """按页面真实导出格式造答卷：slots 字母 → "slotK"，answer 是原始字母，by_model 是 "slotK"。"""
    data_build = bd.EXP / "exp006-scene-sim" / "builds" / "20261030214530-44df"
    key = bd.jload(data_build / "pages" / page / "unblind-key.json")
    trials = []
    slots = {L: f"slot{j}" for j, L in enumerate("ABCD")}
    letter = {v: k for k, v in slots.items()}
    for i, (clip, names) in enumerate(key["key"].items(), 1):
        best = "slot" + str(names.index("polish"))  # 假设每屏都选 polish
        trials.append({"order": i, "kind": "real", "clip": clip, "trap": None, "slots": slots,
                       "qorder": ["clearer", "natural"],
                       "answer": {"clearer": letter[best], "natural": "same", "eaten": ["none"],
                                  "usable": {L: "ok" for L in slots}, "tags": {}},
                       "by_model": {"clearer": best, "natural": "same", "eaten": ["none"],
                                    "usable": {s: "ok" for s in slots.values()}, "tags": {}},
                       "note": "假答卷", "heard": {L: True for L in slots}, "listen_s": {"A": 4.0, "ref": 3.0},
                       "plays": {"A": 1}, "switches": 1, "started_at": "x", "answered_at": "y"})
    for tr in key["traps"]:
        kind = tr.get("kind") or tr["id"].removeprefix("trap-")
        if kind == "identical":
            c = pick_trap_identical  # "same" 或 "slot2" 之类
        else:
            c = "slot" + str(tr["names"].index("degraded")) if pick_degraded else "slot" + str(next(i for i, n in enumerate(tr["names"]) if n != "degraded"))
        bm = {"clearer": c, "natural": c, "eaten": [], "usable": {s: "marginal" for s in slots.values()}, "tags": {}}
        trials.append({"order": len(trials) + 1, "kind": "trap", "clip": tr["clip"], "trap": tr["id"],
                       "slots": slots, "answer": {"clearer": letter.get(c, c), "natural": letter.get(c, c)}, "by_model": bm})
    if extra_self:
        trials.append({"order": len(trials) + 1, "kind": "self", "clip": None, "trap": None, "slots": slots, "by_model": {}})
    return {"exp": "exp006-scene-sim", "build": page, "started": "s", "finished": "f", "trials": trials}, data_build


def run_exp006(ans, data_build):
    with tempfile.TemporaryDirectory() as d:
        p = Path(d) / "fake.json"
        p.write_text(json.dumps(ans))
        old = bd.rel
        bd.rel = lambda x: str(x)  # 临时文件不在仓里
        try:
            return bd.exp006(p, data_build)
        finally:
            bd.rel = old


def test_exp006_interface():
    page = "20261030235644-6a99"
    recs = run_exp006(*fake_exp006(page, "same", True, extra_self=True))
    real = [r for r in recs if r["kind"] == "real"]
    check(len(recs) == 14, "exp006 自录题（kind = self）被跳过、不崩")
    check(len(real) == 12 and all(r["answers"]["clearer"] == "polish" for r in real), "exp006 揭盲：12 屏都揭成 polish")
    check(all(r["scene"]["fine"] for r in real), "exp006 场景标签取自 files.json")
    check(recs[0]["session"]["validity"] == "void", "exp006 劣化检验题选了劣化版 → 会话作废")
    for r in recs:
        r["rec_id"] = "t"
    pairs = [p for r in recs for p in bd.pairs_of(r)]
    check(all(p["identical_audio"] for p in pairs if p["strength"] in ("invalid", "consistency")), "exp006 相同音频的对只会标 invalid / consistency")
    check(any(p["identical_audio"] for p in pairs), "exp006 能识别出同一文件的候选（base = route 等）")
    # 全同检验题：选了某个槽位必须判没过（曾因 "same" 撞名永远判过）
    recs = run_exp006(*fake_exp006(page, "slot2", False))
    tr = [r for r in recs if r["kind"] == "trap" and r["trap"]["type"] == "trap-identical"][0]
    check(tr["trap"]["pass"] is False and tr["answers"]["clearer"] == "identical#3", "exp006 全同题选了槽位 2 → 判没过，答案揭成 identical#3")
    check(recs[0]["session"]["validity"] == "discounted", "exp006 全同题没过 → 会话打折")
    # 旧页面 traps 没有 kind 字段
    recs = run_exp006(*fake_exp006("20261030223232-700e", "same", True))
    types = sorted(r["trap"]["type"] for r in recs if r["kind"] == "trap")
    check(types == ["trap-degraded", "trap-identical"] and recs[0]["session"]["validity"] == "void",
          "旧页面（traps 无 kind）检验题类型判对，选劣化版 → 作废")


if __name__ == "__main__":
    test_existing()
    test_exp006_interface()
    sys.exit(1 if check.failed else 0)
