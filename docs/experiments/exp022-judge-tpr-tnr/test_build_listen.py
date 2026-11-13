"""build_listen.py 纯函数的测试（不碰音频）：python3 -m unittest discover -s docs/experiments/exp022-judge-tpr-tnr -p 'test_*.py'"""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import build_listen as B  # noqa: E402
import tpr_tnr as T  # noqa: E402


def row(q, d, extra_pp, ref_len=20):
    return {"scores": [
        {"v": "V-NRMOS-SCOREQ", "value": q}, {"v": "V-NRMOS-DISTILL", "value": d},
        {"v": "V-EAT-ASR", "value": extra_pp, "extra": {"ref_len": ref_len}}]}


def pool(n_fail, n_pass):
    out = []
    for i in range(n_fail + n_pass):
        v = "fail" if i < n_fail else "pass"
        out.append({"run": "r", "system": "s", "clip": f"x-c{i:02d}-ext", "judge": {"verdict": v}})
        out.append({"run": "r", "system": "s", "clip": f"x-c{i:02d}-mba", "judge": {"verdict": v}})
    return out


class Judge(unittest.TestCase):
    def test_lines_from_table(self):
        L = T.read_lines()
        slack, lim = -L["nrmos_slack_v2"], L["eat_count_v2"]
        self.assertEqual(B.judge_of(row(slack, 0, 0), L)["verdict"], "pass")
        self.assertEqual(B.judge_of(row(slack - 0.01, 0, 0), L)["verdict"], "fail")
        # 多错 lim 个字：百分点 = lim / ref_len × 100
        self.assertEqual(B.judge_of(row(0, 0, lim / 20 * 100), L)["verdict"], "fail")
        self.assertEqual(B.judge_of(row(0, 0, (lim - 1) / 20 * 100), L)["verdict"], "pass")
        self.assertIsNone(B.judge_of({"scores": []}, L))


class Select(unittest.TestCase):
    def test_strata_unique_sources_and_retests(self):
        order, sizes, frac = B.select(pool(30, 300), 9, 2, 1)
        self.assertEqual(len(order), 20)
        first = [p for p in order if p["retest_of"] is None]
        self.assertEqual(sum(p["stratum"] == "fail" for p in first), 9)
        self.assertEqual(len({B.source_of(p["clip"]) for p in first}), 18, "同一源片段只出一次")
        self.assertEqual(sizes, {"fail": 60, "pass": 600})
        self.assertAlmostEqual(frac["fail"], 9 / 60)
        rts = [(i + 1, p) for i, p in enumerate(order) if p["retest_of"]]
        self.assertEqual({p["stratum"] for _, p in rts}, {"fail", "pass"})
        for pos, p in rts:
            src = order[p["retest_of"] - 1]
            self.assertEqual(src["clip"], p["clip"])
            self.assertGreaterEqual(pos - p["retest_of"], 9, "重测离原题至少隔半套题")

    def test_deterministic(self):
        a = [p["clip"] for p in B.select(pool(30, 300), 9, 2, 7)[0]]
        b = [p["clip"] for p in B.select(pool(30, 300), 9, 2, 7)[0]]
        self.assertEqual(a, b)

    def test_not_enough_raises(self):
        with self.assertRaises(ValueError):
            B.select(pool(3, 300), 9, 2, 1)


class Forbidden(unittest.TestCase):
    def test_hits_and_template_clean(self):
        self.assertTrue(B.forbidden_hits([("x", "gtcrn 输出")]))
        self.assertFalse(B.forbidden_hits([("q01.wav", "q01.wav"), ("q01_orig.wav", "q01_orig.wav")]))
        tpl = (Path(B.__file__).parent / "listen_template.html").read_text()
        self.assertEqual(B.forbidden_hits([("tpl", tpl)]), [])


if __name__ == "__main__":
    unittest.main()
