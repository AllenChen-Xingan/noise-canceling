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
