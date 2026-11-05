"""exp016 analyze.py 的测试：方向换算、配对单位、三态、排名门槛。

用法（项目根下）：python3 -m unittest discover -s docs/experiments/exp016-candidates/tests
"""
import json
import random
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import analyze as A  # noqa: E402


def row(clip, system, vals, scene="cafe", sounds=("babble",)):
    return {"clip": clip, "system": system, "scene": scene, "sounds": [{"cls": s, "group": "x"} for s in sounds],
            "scores": [{"v": v, "value": x} for v, x in vals.items()]}


class Sign(unittest.TestCase):
    def test_positive_means_candidate_better(self):
        better = {"V-NR": "越小越好", "V-ESTOI": "越大越好"}
        rows = [row("a", "gtcrn", {"V-NR": -5.0, "V-ESTOI": 0.1}), row("a", "c", {"V-NR": -8.0, "V-ESTOI": 0.3})]
        d = {x[3]: x[4] for x in A.diffs(rows, better, "c")}
        self.assertAlmostEqual(d["V-NR"], 3.0)  # 多压 3 dB = 好
        self.assertAlmostEqual(d["V-ESTOI"], 0.2)

    def test_skips_unknown_and_unscored(self):
        better = {"V-NR": "越小越好", "V-LAT": "越小越好", "V-EAT-ASR": "越小越好"}
        rows = [row("a", "gtcrn", {"V-NR": 1.0, "V-LAT": 16.0, "V-EAT-ASR": None}),
                row("a", "c", {"V-NR": None, "V-LAT": 60.0, "V-EAT-ASR": 0.1})]
        self.assertEqual(A.diffs(rows, better, "c"), [])


class Pairs(unittest.TestCase):
    def test_two_devices_are_one_unit(self):
        with tempfile.TemporaryDirectory() as t:
            p = Path(t) / "c.jsonl"
            p.write_text("\n".join(json.dumps(x) for x in [{"id": "s-c00-mba", "pair": "s-c00-ext"},
                                                          {"id": "s-c00-ext", "pair": "s-c00-mba"}]) + "\n")
            u = A.pairs_of(p)
            self.assertEqual(u["s-c00-mba"], u["s-c00-ext"])

    def test_ci_and_state(self):
        rng = random.Random(0)
        n, m, lo, hi, _ = A.ci({f"u{i}": [1.0 + 0.01 * i, 1.0] for i in range(10)}, rng)
        self.assertEqual(n, 20)
        self.assertGreater(lo, 0)
        self.assertEqual(A.state(lo, hi), "候选好")
        self.assertEqual(A.state(-1, 1), "分不出")
        self.assertEqual(A.state(-2, -1), "候选差")
        self.assertEqual(A.ci({"u": [1.0, 2.0]}, rng)[2], None, "只有一个单位时不给区间")


class Merge(unittest.TestCase):
    def card(self, rid, rules="v2", qa=True):
        return {"meta": {"run_id": rid, "rules": rules, "suite": "s.json", "qa": qa},
                "scenes": [{"items": [{"v": "V-NR", "better": "lower", "unit": "dB"}]}]}

    def test_two_runs_pair_across(self):
        a = ([row("x", "gtcrn", {"V-NR": -5.0})], self.card("g"))
        b = ([row("x", "c", {"V-NR": -8.0})], self.card("k"))
        rows, card = A.merge([a, b])
        self.assertEqual(card["meta"]["run_id"], "g+k")
        self.assertEqual(a[1]["meta"]["run_id"], "g", "不改调用方的 card")
        d = A.diffs(rows, A.better_map(card), "c")
        self.assertEqual([x[4] for x in d], [3.0])

    def test_refuses_mismatch_and_duplicates(self):
        g = ([row("x", "gtcrn", {"V-NR": -5.0})], self.card("g"))
        for bad in (self.card("k", rules="v1"), self.card("k", qa=False)):
            with self.assertRaises(SystemExit):
                A.merge([g, ([row("x", "c", {"V-NR": 1.0})], bad)])
        with self.assertRaises(SystemExit):
            A.merge([g, ([row("x", "gtcrn", {"V-NR": 1.0})], self.card("k"))])

    def test_refuses_missing_meta_dup_in_first_run_units_and_tags(self):
        bare = self.card("g")
        del bare["meta"]["rules"]
        with self.assertRaises(SystemExit):
            A.merge([([row("x", "gtcrn", {"V-NR": 1.0})], bare)])
        with self.assertRaises(SystemExit):  # 第一个 run 内部重复
            A.merge([([row("x", "gtcrn", {"V-NR": 1.0})] * 2, self.card("g"))])
        g = ([row("x", "gtcrn", {"V-NR": 1.0})], self.card("g"))
        other_unit = self.card("k")
        other_unit["scenes"][0]["items"][0]["unit"] = "dBFS"
        with self.assertRaises(SystemExit):
            A.merge([g, ([row("x", "c", {"V-NR": 1.0})], other_unit)])
        with self.assertRaises(SystemExit):  # 同一片段场景标签不同
            A.merge([g, ([row("x", "c", {"V-NR": 1.0}, scene="home-quiet")], self.card("k"))])


class Rank(unittest.TestCase):
    def test_top_behind_needs_units_and_positive_interval(self):
        cells = [{"state": "候选好", "units": 4, "effect": 9.0}, {"state": "候选好", "units": 5, "effect": 1.0},
                 {"state": "分不出", "units": 9, "effect": 5.0}, {"state": "候选好", "units": 6, "effect": 2.0}]
        self.assertEqual([c["effect"] for c in A.top_behind(cells)], [2.0, 1.0])

    def test_effect_uses_global_item_sd(self):
        ds = [("a", "cafe", frozenset(["x"]), "V-NR", 1.0), ("b", "cafe", frozenset(["x"]), "V-NR", 3.0),
              ("c", "home-quiet", frozenset(["y"]), "V-NR", 2.0)]
        sd = A.item_sd(ds)["V-NR"]
        out = A.summarize(ds, {}, lambda d: [(d[1], d[3])], 1, A.item_sd(ds))
        cafe = next(x for x in out if x["key"] == ("cafe", "V-NR"))
        self.assertAlmostEqual(cafe["effect"], 2.0 / sd)


if __name__ == "__main__":
    unittest.main()
