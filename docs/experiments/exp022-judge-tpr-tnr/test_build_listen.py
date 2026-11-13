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
        order, sizes, tries = B.select(pool(30, 300), 9, 2, 1)
        self.assertEqual(len(order), 20)
        first = [p for p in order if p["retest_of"] is None]
        self.assertEqual(sum(p["stratum"] == "fail" for p in first), 9)
        self.assertEqual(len({B.source_of(p["clip"]) for p in first}), 18, "同一源片段只出一次")
        self.assertEqual(sizes, {"fail": 60, "pass": 600})
        self.assertGreaterEqual(tries, 1)
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

    def test_retest_when_first_half_is_one_stratum(self):
        # codex 第 2 轮 P2 #2：洗牌后前半段全是判不过层，判过层的原题也要被换到前半段，不能崩
        order = [{"clip": f"f{i}", "stratum": "fail"} for i in range(9)] + \
                [{"clip": f"p{i}", "stratum": "pass"} for i in range(9)]
        out = B.place_retests(order, 2, None)
        rts = [p for p in out if p["retest_of"]]
        self.assertEqual({p["stratum"] for p in rts}, {"fail", "pass"})
        for p in rts:
            self.assertLessEqual(p["retest_of"], 9, "原题在前半段")
            self.assertEqual(out[p["retest_of"] - 1]["clip"], p["clip"])


class Inclusion(unittest.TestCase):
    """codex 第 2 轮 P1 #1：入样概率按实际设计模拟，跨层共享源片段时偏离 n/N。"""

    def test_no_shared_sources_matches_n_over_n(self):
        strata = B.frame_of(pool(10, 10)[::2] + [dict(x, clip=x["clip"] + "z") for x in pool(0, 0)])
        pi = B.inclusion_probs(strata, 2, 3, 20000)
        for k in strata:
            for it in strata[k]:
                self.assertAlmostEqual(pi.get(B.item_key(it), 0), 2 / len(strata[k]), delta=0.02)

    def test_shared_source_lowers_probability(self):
        # 判不过层 2 条：源 A、源 B；判过层 4 条：源 A、C、D、E。每层抽 1。
        # 判过层抽到源 A 时，只要判不过层抽到 A 就要整套重抽 → 判过层的 A 入样概率低于 1/4
        mk = lambda clip, v: {"run": "r", "system": "s", "clip": clip, "judge": {"verdict": v}}  # noqa: E731
        p = [mk("A-ext", "fail"), mk("B-ext", "fail"),
             mk("A-mba", "pass"), mk("C-ext", "pass"), mk("D-ext", "pass"), mk("E-ext", "pass")]
        pi = B.inclusion_probs(B.frame_of(p), 1, 5, 40000)
        # 精确值：接受的套数 = 2×4 − 1 = 7，含「判过 A」的 1 套 → 1/7；其余判过条 2/7
        self.assertAlmostEqual(pi["r|s|A-mba"], 1 / 7, delta=0.01)
        self.assertAlmostEqual(pi["r|s|C-ext"], 2 / 7, delta=0.01)


class Key(unittest.TestCase):
    def test_prereg_and_required_fields(self):
        k = B.make_key("b", T.read_lines(), {"fail": 1, "pass": 1}, 1, [], "<html>", pool(1, 1))
        for f in B.REQUIRED_KEY_FIELDS:
            self.assertIn(f, k)
        for f in B.REQUIRED_PREREG:
            self.assertIn(f, k["prereg"])

    def test_missing_fields_rejected(self):
        k = B.make_key("b", T.read_lines(), {"fail": 1, "pass": 1}, 1, [{"q": 1}], "<html>", pool(1, 1))
        for f in B.REQUIRED_KEY_FIELDS:
            bad = dict(k)
            del bad[f]
            with self.assertRaises(ValueError, msg=f):
                B.check_key(bad)
        for f in B.REQUIRED_PREREG:
            for v in (None, "", "  "):
                bad = dict(k, prereg=dict(k["prereg"]))
                if v is None:
                    del bad["prereg"][f]
                else:
                    bad["prereg"][f] = v
                with self.assertRaises(ValueError, msg=f"prereg.{f}={v!r}"):
                    B.check_key(bad)


class Forbidden(unittest.TestCase):
    def test_hits_and_template_clean(self):
        self.assertTrue(B.forbidden_hits([("x", "gtcrn 输出")]))
        self.assertFalse(B.forbidden_hits([("q01.wav", "q01.wav"), ("q01_orig.wav", "q01_orig.wav")]))
        tpl = (Path(B.__file__).parent / "listen_template.html").read_text()
        self.assertEqual(B.forbidden_hits([("tpl", tpl)]), [])


if __name__ == "__main__":
    unittest.main()
