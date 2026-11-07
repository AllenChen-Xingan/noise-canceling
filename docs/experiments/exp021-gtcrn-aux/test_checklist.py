"""checklist.py 的测试：python3 -m unittest docs/experiments/exp021-gtcrn-aux/test_checklist.py"""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import checklist as C  # noqa: E402


TH = {"V-ONSET": 3.0, "V-SPLVL": -3.0}  # 假数据里的成绩单过关线


def row(clip, system, scene, eat=None, ref_len=20.0, **vals):
    scores = [{"v": v.replace("_", "-"), "value": x, "verdict": "pass", "threshold": TH.get(v.replace("_", "-"))}
              for v, x in vals.items()]
    if eat is not None:
        scores.append({"v": "V-EAT-ASR", "value": eat, "extra": {"ref_len": ref_len}, "verdict": "pass"})
    return {"clip": clip, "system": system, "scene": scene, "scores": scores}


BASE = {"V_ESTOI": 0.5, "V_ONSET": 2.0, "V_BGSTAB": 1.0, "V_INTF": -5.0, "V_SPLVL": -1.0, "V_PUMPV": 1.0,
        "V_NRMOS_DISTILL": 3.0, "V_NRMOS_SCOREQ": 3.0}


def world(delta, n=4, eat_d=0.0, eat_c=0.0):
    """每个弱场景 n 段；候选档各项 = 默认 + delta（字典），每段加一点抖动免得区间为 0。"""
    rows = []
    for sc in C.WEAK:
        for i in range(n):
            j = 0.001 * i
            d = {k: v + j for k, v in BASE.items()}
            c = {k: v + j + delta.get(k, 0.0) for k, v in BASE.items()}
            rows.append(row(f"{sc}-{i}", "D", sc, eat=eat_d, **d))
            rows.append(row(f"{sc}-{i}", "X", sc, eat=eat_c if i == 0 else eat_d, **c))
    return rows


GOOD = {"V_ESTOI": 0.01, "V_ONSET": 0.5, "V_BGSTAB": -0.1, "V_INTF": -0.1, "V_NRMOS_DISTILL": 0.1,
        "V_NRMOS_SCOREQ": 0.1}


def states(rows):
    return {k: s for k, s, _ in C.judge(rows, "D", "X")}


class Judge(unittest.TestCase):
    def test_all_pass(self):
        got = states(world(GOOD))
        self.assertTrue(all(s == C.PASS for s in got.values()), got)

    def test_estoi_below_line_fails(self):
        self.assertEqual(states(world({**GOOD, "V_ESTOI": 0.007}))["弱场景可懂度各 ≥ +0.008"], C.FAIL)

    def test_onset_over_fails(self):
        self.assertEqual(states(world({**GOOD, "V_ONSET": 0.61}))["字头每场景升 ≤ 0.6 dB"], C.FAIL)

    def test_mos_drop_and_bg_worse_fail(self):
        got = states(world({**GOOD, "V_NRMOS_DISTILL": -0.1, "V_BGSTAB": 0.2, "V_INTF": 0.3}))
        self.assertEqual(got["听感分 V-NRMOS-DISTILL 不降"], C.FAIL)
        self.assertEqual(got["听感分 V-NRMOS-SCOREQ 不降"], C.PASS)
        self.assertEqual(got["背景稳不变差"], C.FAIL)
        self.assertEqual(got["旁人不变差"], C.FAIL)

    def test_interval_across_zero_is_tbd_not_pass(self):
        # codex 第 1 轮 P1 #1：区间跨 0 不能当「不变差」
        self.assertEqual(C.ci_state({"lo": -0.3, "hi": 0.1}, True), C.TBD)
        self.assertEqual(C.ci_state({"lo": -0.1, "hi": 0.3}, False), C.TBD)
        self.assertEqual(C.ci_state({"lo": 0.0, "hi": 0.3}, True), C.PASS)
        self.assertEqual(C.ci_state({"lo": -0.3, "hi": -0.1}, False), C.PASS)
        self.assertEqual(C.ci_state(None, True), C.TBD)

    def test_unchanged_mos_passes(self):
        # 候选与默认逐段相同 → 区间 [0, 0]：下界 ≥ 0 算过（证明没降）
        self.assertEqual(states(world({**GOOD, "V_NRMOS_DISTILL": 0.0}))["听感分 V-NRMOS-DISTILL 不降"], C.PASS)

    def test_missing_onset_scene_is_tbd(self):
        rows = world(GOOD)
        for r in rows:
            if r["system"] == "X" and r["scene"] == "cafe":
                r["scores"] = [s for s in r["scores"] if s["v"] != "V-ONSET"]
        self.assertEqual(states(rows)["字头每场景升 ≤ 0.6 dB"], C.TBD)

    def test_missing_weak_scene_estoi_is_tbd(self):
        rows = [r for r in world(GOOD) if not (r["system"] == "X" and r["scene"] == "cafe")]
        self.assertEqual(states(rows)["弱场景可懂度各 ≥ +0.008"], C.TBD)


    def test_one_segment_missing_one_side_blocks_pass(self):
        # codex 第 2 轮 P2 #1：场景里只缺一段也不许判过
        for v in ("V-NRMOS-SCOREQ", "V-BGSTAB", "V-ONSET"):
            rows = world(GOOD)
            r = next(r for r in rows if r["system"] == "X" and r["clip"] == "cafe-1")
            r["scores"] = [s for s in r["scores"] if s["v"] != v]
            got = states(rows)
            key = {"V-NRMOS-SCOREQ": "听感分 V-NRMOS-SCOREQ 不降", "V-BGSTAB": "背景稳不变差",
                   "V-ONSET": "字头每场景升 ≤ 0.6 dB"}[v]
            self.assertEqual(got[key], C.TBD, v)

    def test_missing_on_both_sides_is_not_missing(self):
        rows = world(GOOD)
        for r in rows:
            if r["clip"] == "cafe-1":
                r["scores"] = [s for s in r["scores"] if s["v"] != "V-BGSTAB"]
        self.assertEqual(states(rows)["背景稳不变差"], C.PASS)


class RustSource(unittest.TestCase):
    """吃字否决线和字数还原公式的正本在 Rust；这里读源码比对，正本一改这条就红（codex 第 2 轮 P2 #2）。"""
    ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))

    def read(self, rel):
        with open(os.path.join(self.ROOT, rel), encoding="utf-8") as f:
            return f.read()

    def test_veto_line_matches_table(self):
        import re
        src = self.read("src/eval/src/thresholds/table.rs")
        block = src[src.index('id: "V-EAT-ASR"'):]
        m = re.search(r'"fail_chars",\s*Some\(([0-9.]+)\)', block)
        self.assertIsNotNone(m, "table.rs 里找不到 V-EAT-ASR 的 fail_chars")
        self.assertEqual(float(m.group(1)), C.EAT_VETO)

    def test_formula_matches_rules_v2(self):
        src = self.read("src/eval/src/scorecard/rules_v2.rs")
        body = src[src.index("pub fn eat_chars"):].split("}", 1)[0]
        # 公式：百分点 ÷ 100 × ref_len，再 f64::round（半整数远离 0）；checklist.eat_chars 照此实现
        self.assertIn('s.value? / 100.0 * extra(s, "ref_len")?).round()', body)
        src2 = self.read("src/eval/src/scorecard/rules_v2.rs")
        self.assertIn("Some(n) if n >= lim => f += 1", src2)  # 「≥ 线」才否决


class Eat(unittest.TestCase):
    def test_same_rule_as_rules_v2(self):
        # 25 百分点 × 20 字 = 5 字：恰好否决；24 → 4.8 → 5 也否决（四舍五入）；22 → 4.4 → 4 不否决
        self.assertEqual(C.eat_chars({"value": 25.0, "extra": {"ref_len": 20.0}}), 5)
        self.assertEqual(C.eat_chars({"value": 24.0, "extra": {"ref_len": 20.0}}), 5)
        self.assertEqual(C.eat_chars({"value": 22.0, "extra": {"ref_len": 20.0}}), 4)
        self.assertIsNone(C.eat_chars({"value": None, "extra": {}}))

    def test_half_rounds_away_from_zero_like_rust(self):
        # codex 第 1 轮 P2 #4：22.5 % × 20 字 = 4.5 字；Rust f64::round → 5（否决），Python round → 4
        self.assertEqual(C.eat_chars({"value": 22.5, "extra": {"ref_len": 20.0}}), 5)
        self.assertEqual(C.eat_chars({"value": -22.5, "extra": {"ref_len": 20.0}}), -5)
        self.assertEqual(C.eat_chars({"value": 17.5, "extra": {"ref_len": 20.0}}), 4)  # 3.5 → 4

    def test_new_vetoes_listed(self):
        rows = world(GOOD, eat_d=0.0, eat_c=30.0)
        lines = {a: (b, c, d) for a, b, c, d in C.sheet12(rows, "D", "X")}
        b, c, d = lines["吃字否决段数（单段多错 ≥ 5 字）"]
        self.assertTrue(b.startswith("0"))
        self.assertTrue(c.startswith("5"))
        self.assertIn("通过→否决 5 段", d)
        self.assertIn("street-outdoor-0（6 字）", d)

    def test_unknown_not_counted_as_new_or_gone(self):
        # codex 第 1 轮 P2 #3：否决→不知道 不能算「消失」，不知道→否决 不能算「新增」
        rows = world(GOOD, eat_d=30.0, eat_c=30.0)
        for r in rows:
            if r["system"] == "X" and r["clip"] == "cafe-0":
                r["scores"] = [s for s in r["scores"] if s["v"] != "V-EAT-ASR"]
        moves, odd = C.eat_moves(rows, "D", "X")
        self.assertEqual(list(moves), [("否决", "不知道")])
        self.assertEqual(odd, 0)

    def test_splvl_line_count(self):
        rows = world({"V_SPLVL": -2.5})
        lines = {a: (b, c) for a, b, c, _ in C.sheet12(rows, "D", "X")}
        self.assertEqual(lines["人声电平越过 -3 dB 线的场景数"], ("0", "5"))

    def test_onset_line_count_from_data(self):
        rows = world({**GOOD, "V_ONSET": 1.5})  # 默认 2.0 → 3.5，越过数据里记的 3 dB 线
        lines = {a: (b, c) for a, b, c, _ in C.sheet12(rows, "D", "X")}
        self.assertEqual(lines["字头场景均值超过成绩单 3 dB 线的场景数"], ("0", "5"))

    def test_line_must_be_unique(self):
        rows = world(GOOD)
        next(s for s in rows[0]["scores"] if s["v"] == "V-ONSET")["threshold"] = 99.0
        with self.assertRaises(ValueError):
            C.line_of(rows, "V-ONSET")
        self.assertEqual(C.line_of(world(GOOD), "V-SPLVL"), -3.0)


class Cli(unittest.TestCase):
    def test_writes_new_and_refuses_existing_out(self):
        import json
        import tempfile
        with tempfile.TemporaryDirectory() as t:
            src, out = os.path.join(t, "c.jsonl"), os.path.join(t, "x.md")
            with open(src, "w") as f:
                f.writelines(json.dumps(r) + "\n" for r in world({"V_ESTOI": 0.01}))
            self.assertEqual(C.main([src, "D", "X", "--sheet12", "X", "--out", out]), 0)
            with open(out) as f:
                text = f.read()
            self.assertIn("## `X`：全部过线", text)
            self.assertIn("−12 档单独检查表", text)
            with self.assertRaises(FileExistsError):
                C.main([src, "D", "X", "--out", out])


if __name__ == "__main__":
    unittest.main()
