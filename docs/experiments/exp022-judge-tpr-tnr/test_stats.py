"""stats.py 与 tpr_tnr.py 判定函数的测试：python3 -m unittest discover -s docs/experiments/exp022-judge-tpr-tnr -p 'test_*.py'"""
import math
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import stats as S  # noqa: E402
import tpr_tnr as T  # noqa: E402

F, P, U = S.FAIL, S.PASS, S.UNDECIDED


def mk(tp, fn, tn, fp, und=0):
    return [(F, F)] * tp + [(F, P)] * fn + [(P, P)] * tn + [(P, F)] * fp + [(F, U)] * und


class Confusion(unittest.TestCase):
    def test_counts_and_rates(self):
        # evals-faq 的例子：人判 10 个坏的抓到 8 个；100 个好的放过 95 个
        r = S.rates(S.confusion(mk(8, 2, 95, 5)))
        self.assertAlmostEqual(r["tpr"], 0.8)
        self.assertAlmostEqual(r["tnr"], 0.95)
        self.assertAlmostEqual(r["raw_agreement"], 103 / 110)
        self.assertAlmostEqual(r["balanced_accuracy"], 0.875)

    def test_always_pass_inflates_raw_agreement(self):
        # 5% 坏的、评估器一律判过：总一致率 95%，但一个坏的都没抓到
        r = S.rates(S.confusion(mk(0, 5, 95, 0)))
        self.assertAlmostEqual(r["raw_agreement"], 0.95)
        self.assertEqual(r["tpr"], 0.0)
        self.assertAlmostEqual(r["youden_j"], 0.0)

    def test_undecided_excluded_and_counted(self):
        r = S.rates(S.confusion(mk(1, 1, 1, 1, und=3)))
        self.assertEqual(r["n_decided"], 4)
        self.assertEqual(r["n_undecided"], 3)

    def test_bad_label_raises(self):
        with self.assertRaises(ValueError):
            S.confusion([("maybe", F)])

    def test_empty_class_is_nan(self):
        r = S.rates(S.confusion(mk(0, 0, 3, 1)))
        self.assertTrue(math.isnan(r["tpr"]))
        self.assertTrue(math.isnan(r["youden_j"]))


class Wilson(unittest.TestCase):
    def test_known_value(self):
        lo, hi = S.wilson(8, 10)
        self.assertAlmostEqual(lo, 0.4902, places=3)
        self.assertAlmostEqual(hi, 0.9433, places=3)

    def test_edges(self):
        self.assertEqual(S.wilson(0, 5)[0], 0.0)
        self.assertEqual(S.wilson(5, 5)[1], 1.0)
        self.assertTrue(math.isnan(S.wilson(0, 0)[0]))


class RoganGladen(unittest.TestCase):
    def test_recovers_truth(self):
        # 真不过率 0.3，TPR 0.8，TNR 0.9 → 评估器判不过率 = 0.3·0.8 + 0.7·0.1 = 0.31
        self.assertAlmostEqual(S.rogan_gladen(0.31, 0.8, 0.9), 0.3)

    def test_chance_judge_has_no_solution(self):
        self.assertIsNone(S.rogan_gladen(0.4, 0.5, 0.5))
        self.assertIsNone(S.rogan_gladen(0.4, 0.3, 0.6))

    def test_clamped(self):
        self.assertEqual(S.rogan_gladen(0.0, 0.8, 0.9), 0.0)


class Bootstrap(unittest.TestCase):
    def test_deterministic_and_brackets_point(self):
        clusters = {f"c{i}": mk(1, 0, 1, 0) if i % 2 else mk(0, 1, 1, 0) for i in range(12)}
        stat = lambda xs: S.rates(S.confusion(xs))["tpr"]  # noqa: E731
        a = S.cluster_bootstrap(clusters, stat, reps=500)
        b = S.cluster_bootstrap(clusters, stat, reps=500)
        self.assertEqual(a, b)
        self.assertLessEqual(a[0], 0.5)
        self.assertGreaterEqual(a[1], 0.5)


class Verdicts(unittest.TestCase):
    def test_delta_judge(self):
        self.assertEqual(T.delta_verdict(-0.1, -0.1), P)
        self.assertEqual(T.delta_verdict(-0.11, -0.1), F)
        self.assertEqual(T.delta_verdict(0.0, 0.0), P)

    def test_two_judges_v1(self):
        self.assertEqual(T.both(P, P), P)
        self.assertEqual(T.both(F, F), F)
        self.assertEqual(T.both(P, F), U)

    def test_eat_judge(self):
        noisy = {"del": 1, "sub": 1, "ref_len": 100}
        self.assertEqual(T.eat_count({"del": 4, "sub": 2, "ref_len": 100}, noisy, 5), P)  # 多 4 个
        self.assertEqual(T.eat_count({"del": 5, "sub": 2, "ref_len": 100}, noisy, 5), F)  # 多 5 个
        self.assertEqual(T.eat_pp({"del": 2, "sub": 1, "ref_len": 100}, noisy, 1.0), P)  # 多 1 个百分点
        self.assertEqual(T.eat_pp({"del": 3, "sub": 1, "ref_len": 100}, noisy, 1.0), F)

    def test_human_usable_cuts(self):
        self.assertEqual(T.human_usable("no", strict=False), F)
        self.assertEqual(T.human_usable("marginal", strict=False), P)
        self.assertEqual(T.human_usable("marginal", strict=True), F)
        self.assertIsNone(T.human_usable("", strict=True))


class Lines(unittest.TestCase):
    """过关线只从 Rust 正本读：读得到、读出的值能在原文找到；正本缺参数时必须报错，不许静默用默认值。"""

    def test_reads_real_table(self):
        lines = T.read_lines()
        self.assertEqual(set(lines), {"scoreq_min_v1", "distill_min_v1", "nrmos_slack_v2",
                                      "eat_pp_v1", "eat_count_v2"})
        src = T.TABLE.read_text(encoding="utf-8")
        blk = src[src.index('id: "V-EAT-ASR"'):]
        self.assertIn(f'"fail_chars",\n            Some({lines['eat_count_v2']})', blk)

    def test_missing_param_raises(self):
        with self.assertRaises(KeyError):
            T._param('id: "V-X",\n params: &[p("min", Some(0.0), &[])],\n};', "V-X", "max")
        with self.assertRaises(KeyError):
            T._param("", "V-NOPE", "min")


if __name__ == "__main__":
    unittest.main()
