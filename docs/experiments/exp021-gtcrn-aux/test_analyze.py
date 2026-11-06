"""analyze.py 的测试：python3 -m unittest docs/experiments/exp021-gtcrn-aux/test_analyze.py"""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import analyze  # noqa: E402


def row(clip, system, scene="cafe", **vals):
    return {"clip": clip, "system": system, "scene": scene,
            "scores": [{"v": v, "value": x, "verdict": "pass"} for v, x in vals.items()]}


class Check(unittest.TestCase):
    def test_identical_passes(self):
        a = [row("c1", "A", **{"V-NR": -7.0, "V-ONSET": 1.0})]
        b = [row("c1", "B", **{"V-NR": -7.0, "V-ONSET": 1.0})]
        _, ok = analyze.check(a, "A", b, "B")
        self.assertTrue(ok)

    def test_any_diff_fails(self):
        a = [row("c1", "A", **{"V-NR": -7.0})]
        b = [row("c1", "B", **{"V-NR": -7.0000001})]
        st, ok = analyze.check(a, "A", b, "B")
        self.assertFalse(ok)
        self.assertEqual(st["V-NR"]["diff"], 1)

    def test_expected_items_reported_not_judged(self):
        a = [row("c1", "A", **{"V-NR": -7.0, "V-SIM-GATE": 0.9})]
        b = [row("c1", "B", **{"V-NR": -7.0, "V-SIM-GATE": 0.1})]
        st, ok = analyze.check(a, "A", b, "B")
        self.assertTrue(ok)
        self.assertEqual(st["V-SIM-GATE"]["diff"], 1)

    def test_missing_clip_fails(self):
        a = [row("c1", "A", **{"V-NR": 1.0}), row("c2", "A", **{"V-NR": 1.0})]
        b = [row("c1", "B", **{"V-NR": 1.0})]
        _, ok = analyze.check(a, "A", b, "B")
        self.assertFalse(ok)

    def test_none_vs_value_fails(self):
        a = [row("c1", "A", **{"V-ESTOI": None})]
        b = [row("c1", "B", **{"V-ESTOI": 0.5})]
        _, ok = analyze.check(a, "A", b, "B")
        self.assertFalse(ok)


class Pair(unittest.TestCase):
    def test_mean_ci_and_scenes(self):
        a = [row("c1", "A", "cafe", **{"V-NR": -6.0}), row("c2", "A", "bus", **{"V-NR": -8.0})]
        b = [row("c1", "B", "cafe", **{"V-NR": -7.0}), row("c2", "B", "bus", **{"V-NR": -8.0})]
        r = analyze.pair(a, "A", b, "B", ["V-NR"])["V-NR"]
        self.assertEqual((r["n"], r["nz"]), (2, 1))
        self.assertAlmostEqual(r["mean"], -0.5)
        self.assertLess(r["lo"], r["mean"])
        self.assertAlmostEqual(r["scenes"]["cafe"], -1.0)


class Load(unittest.TestCase):
    def test_gz_and_plain_same(self):
        import gzip, json, tempfile
        rows = [row("c1", "A", **{"V-NR": -7.0})]
        text = "".join(json.dumps(r) + "\n" for r in rows)
        with tempfile.TemporaryDirectory() as d:
            p, g = os.path.join(d, "c.jsonl"), os.path.join(d, "c.jsonl.gz")
            open(p, "w", encoding="utf-8").write(text)
            with gzip.open(g, "wt", encoding="utf-8") as f:
                f.write(text)
            self.assertEqual(analyze.load(p), analyze.load(g))


class Oracle(unittest.TestCase):
    def test_picks_best_within_constraints(self):
        cur = {"D": {"cafe": {"V-ESTOI": 0.02, "V-ONSET": 2.0, "V-BGSTAB": 2.0}},
               "X": {"cafe": {"V-ESTOI": 0.05, "V-ONSET": 3.5, "V-BGSTAB": 1.0}},
               "Y": {"cafe": {"V-ESTOI": 0.04, "V-ONSET": 2.8, "V-BGSTAB": 1.5}},
               "Z": {"cafe": {"V-ESTOI": 0.045, "V-ONSET": 2.5, "V-BGSTAB": 2.5}}}
        r = analyze.oracle(cur, "D", onset_max=3.0, bg_up=0.0)
        self.assertEqual(r["cafe"][0], "Y")
        self.assertAlmostEqual(r["cafe"][1], 0.02)
        r = analyze.oracle(cur, "D", onset_max=3.0, onset_up=0.5)
        self.assertEqual(r["cafe"][0], "D")

    def test_reports_infeasible_instead_of_default(self):
        cur = {"D": {"car": {"V-ESTOI": 0.02, "V-ONSET": 3.4, "V-BGSTAB": 2.0}},
               "X": {"car": {"V-ESTOI": 0.05, "V-ONSET": 4.0, "V-BGSTAB": 1.0}}}
        self.assertEqual(analyze.oracle(cur, "D", onset_max=3.0)["car"], (None, 0.0))


class OracleMissing(unittest.TestCase):
    def test_unknown_is_not_feasible_but_not_applicable_passes(self):
        cur = {"D": {"q": {"V-ESTOI": 0.02, "V-ONSET": 2.0, "V-BGSTAB": None},
                     "u": {"V-ESTOI": 0.02, "V-ONSET": None, "V-BGSTAB": 1.0}},
               "X": {"q": {"V-ESTOI": 0.03, "V-ONSET": 2.5, "V-BGSTAB": None},
                     "u": {"V-ESTOI": 0.05, "V-ONSET": 2.5, "V-BGSTAB": 1.0}}}
        r = analyze.oracle(cur, "D", onset_max=3.0)
        self.assertEqual(r["q"][0], "X")   # 背景稳两边都没有：不适用，放行
        self.assertEqual(r["u"][0], None)  # 默认档字头缺分、候选有：未知，不选（也不报错）


class Curve(unittest.TestCase):
    def test_means_by_scene_and_all(self):
        rows = [row("c1", "S", "cafe", **{"V-NR": -6.0}), row("c2", "S", "cafe", **{"V-NR": -8.0}),
                row("c3", "S", "bus", **{"V-NR": -10.0, "V-ONSET": None})]
        c = analyze.curve(rows, {"V-NR", "V-ONSET"})
        self.assertAlmostEqual(c["S"]["cafe"]["V-NR"], -7.0)
        self.assertAlmostEqual(c["S"]["_all"]["V-NR"], -8.0)
        self.assertIsNone(c["S"]["bus"]["V-ONSET"])


if __name__ == "__main__":
    unittest.main()
