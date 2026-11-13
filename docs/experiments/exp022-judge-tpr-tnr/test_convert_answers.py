"""convert_answers.py 的测试：答卷 → calib 输入 → 混淆矩阵（Python 口径与 judge.rs 逐格比对过）。"""
import json
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import convert_answers as C  # noqa: E402
import tpr_tnr as T  # noqa: E402

KEY = json.loads((HERE / "builds/20261113-122017-3126/unblind-key.json").read_text())


def fake_answers(usable="no", eaten="yes", build=KEY["build"]):
    return {"build": build, "answers": [{"q": q["q"], "usable": usable, "eaten": eaten} for q in KEY["questions"]]}


def full_path(answers):
    a, s, p, w = C.convert(answers, KEY, ref_len=lambda *_: 20)
    rows = T.build_rows(a, p, s, T.read_lines())
    return a, w, rows


class Convert(unittest.TestCase):
    def test_end_to_end_counts(self):
        a, w, rows = full_path(fake_answers())
        n_first = sum(q["retest_of"] is None for q in KEY["questions"])
        self.assertEqual(len(rows), n_first, "重测题不进")
        self.assertEqual(len(w["retests"]), len(KEY["questions"]) - n_first)
        # 人全说不能用：评估器判不过的那几条是 TP，其余是 FN；没有人判过的条目
        res = {r["judge_key"] + r["human_key"]: r for r in T.evaluate(rows, reps=10)}
        c = res["j_scoreq_slackh_loose"]["confusion"]
        self.assertEqual(c["tn"] + c["fp"], 0)
        listen_fail = sum(q["judge"]["d_scoreq"] < -0.1 for q in KEY["questions"] if q["retest_of"] is None)
        self.assertEqual(c["tp"], listen_fail)
        # 吃字：评估器「多错」原样带过去
        eat = res["j_eat_counth_eaten"]["confusion"]
        eat_fail = sum(q["judge"]["eat_extra"] >= 5 for q in KEY["questions"] if q["retest_of"] is None)
        self.assertEqual(eat["tp"], eat_fail)

    def test_eaten_mapping(self):
        _, _, rows = full_path(fake_answers(eaten="unsure"))
        self.assertTrue(all(r["h_eaten"] is None for r in rows), "听不出 = 判不了")
        _, _, rows = full_path(fake_answers(eaten="no"))
        self.assertTrue(all(r["h_eaten"] == "pass" for r in rows))

    def test_weights_carried(self):
        _, w, _ = full_path(fake_answers())
        self.assertTrue(all(0 < v["inclusion_prob"] < 1 for v in w["weights"].values()))
        self.assertIn("usable_cut", w["prereg"])

    def test_mismatch_raises(self):
        with self.assertRaises(ValueError):
            C.convert(fake_answers(build="other"), KEY, ref_len=lambda *_: 20)
        bad = fake_answers()
        bad["answers"].append({"q": 99, "usable": "ok"})
        with self.assertRaises(KeyError):
            C.convert(bad, KEY, ref_len=lambda *_: 20)
        with self.assertRaises(ValueError):
            C.convert({"build": KEY["build"], "answers": []}, KEY, ref_len=lambda *_: 20)


if __name__ == "__main__":
    unittest.main()
