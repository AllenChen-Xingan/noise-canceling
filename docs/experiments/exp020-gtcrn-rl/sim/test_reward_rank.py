#!/usr/bin/env python3
"""reward_rank.py 的测试：python3 -m unittest discover -s docs/experiments/exp020-gtcrn-rl/sim"""

import json
import tempfile
import unittest
from pathlib import Path

import reward_rank as rr


def row(clip, system, **vals):
    return {"clip": clip, "system": system, "scene": clip.split("-")[0],
            "scores": [{"v": k.replace("_", "-"), "value": v} for k, v in vals.items()]}


class T(unittest.TestCase):
    def test_kendall(self):
        self.assertEqual(rr.kendall_tau([1, 2, 3], [1, 2, 3]), 1.0)
        self.assertEqual(rr.kendall_tau([1, 2, 3], [3, 2, 1]), -1.0)
        self.assertIsNone(rr.kendall_tau([1, 1], [2, 2]))

    def test_veto_and_guard(self):
        base = row("a-c0", "gtcrn", V_ONSET=1.0, V_ESTOI=0.02, V_NR=-6.0)
        ok = row("a-c0", "x", V_ONSET=1.5, V_ESTOI=0.03, V_NR=-6.0)
        eat = row("a-c0", "x", V_ONSET=1.0, V_ESTOI=0.10, V_NR=-6.0, V_EAT_ASR=5.0)
        clip = row("a-c0", "x", V_ONSET=4.0, V_ESTOI=0.03, V_NR=-6.0)
        self.assertEqual(rr.r_composite(eat, base), -1.0)  # 吃字一票否决
        # 字头多退 3 dB（容许 1）→ 扣 2 × 0.01，排到不削字头的同分系统后面
        self.assertLess(rr.r_composite(clip, base), rr.r_composite(ok, base))
        # 降噪量封顶：压到 −60 dB 也只按 12 dB 算
        deep = row("a-c0", "x", V_ONSET=1.0, V_ESTOI=0.03, V_NR=-60.0)
        self.assertAlmostEqual(rr.r_composite(deep, base), 0.03 + 0.002 * 12)

    def test_eat_words_rule(self):
        base = row("a-c0", "gtcrn", V_ESTOI=0.02)
        r = row("a-c0", "x", V_ESTOI=0.03, V_NR=-6.0, V_EAT_ASR=12.5)
        r["scores"][-1]["extra"] = {"ref_len": 24.0}  # 12.5% × 24 = 3 个字
        self.assertAlmostEqual(rr.eat_words(r), 3.0)
        self.assertEqual(rr.r_composite(r, base), -1.0)  # 旧口径 > 1 个百分点 → 否决
        self.assertGreater(rr.r_composite_d85(r, base), 0.0)  # D85：不到 5 个字 → 不否决
        r["scores"][-1]["value"] = 25.0  # 6 个字
        self.assertEqual(rr.r_composite_d85(r, base), -1.0)

    def test_no_asr_and_abs_guard(self):
        base = row("a-c0", "gtcrn", V_ONSET=1.0, V_ESTOI=0.02)
        r = row("a-c0", "x", V_ESTOI=0.03, V_NR=-6.0, V_ONSET=2.0, V_EAT_ASR=50.0)
        r["scores"][-1]["extra"] = {"ref_len": 24.0}  # 12 个字
        self.assertGreater(rr.r_composite_no_asr(r, base), 0.0)  # 不看吃字
        self.assertGreater(rr.r_composite_hard_guard(r, base), 0.0)  # 字头多退 1 dB，在容许内
        self.assertGreater(rr.r_composite_abs_guard(r, base), 0.0)  # 字头 2 dB < 3 dB
        r["scores"][2]["value"] = 3.5  # 字头 3.5 dB
        self.assertEqual(rr.r_composite_abs_guard(r, base), -1.0)
        self.assertEqual(rr.r_composite_hard_guard(r, base), -1.0)

    def test_abs_guard_ignores_gtcrn(self):
        r = row("a-c0", "x", V_ESTOI=0.03, V_NR=-6.0, V_ONSET=2.5)
        b1 = row("a-c0", "gtcrn", V_ONSET=0.0)
        b2 = row("a-c0", "gtcrn", V_ONSET=2.5)
        self.assertEqual(rr.r_composite_abs_guard(r, b1), rr.r_composite_abs_guard(r, b2))

    def test_no_feasible_excluded(self):
        rows = []
        for s, o in [("gtcrn", 4.0), ("a", 5.0), ("b", 6.0)]:  # 全部字头 > 3 dB → 绝对阈值下全否决
            rows.append(row("s0-c0", s, V_ESTOI=0.02, V_NR=-5.0, V_ONSET=o,
                            V_NRMOS_DISTILL=0.1, V_NRMOS_SCOREQ=0.1))
        table = {"s0-c0": {r["system"]: r for r in rows}}
        res = rr.analyse(["a", "b", "gtcrn"], table)
        g = res["rewards"]["composite_abs_guard"]
        self.assertEqual(g["no_feasible_clips"], 1)
        self.assertEqual(g["top_onset_n"], 0)

    def test_end_to_end(self):
        rows = []
        for i in range(4):
            c = f"s{i % 2}-c{i}"
            for s, e, m in [("gtcrn", 0.02, 0.2), ("good", 0.05, 0.4), ("bad", 0.0, 0.0)]:
                rows.append(row(c, s, V_ESTOI=e, V_NR=-5.0, V_ONSET=1.0,
                                V_NRMOS_DISTILL=m, V_NRMOS_SCOREQ=m))
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "c.jsonl"
            p.write_text("\n".join(json.dumps(r) for r in rows))
            (Path(d) / "runs.json").write_text(json.dumps({"all": "c.jsonl"}))
            systems, table = rr.load(Path(d) / "runs.json")
        self.assertEqual(len(table), 4)
        res = rr.analyse(systems, table)
        self.assertEqual(res["rewards"]["estoi_only"]["V-NRMOS-DISTILL"]["mean_tau"], 1.0)
        self.assertEqual(res["rewards"]["composite"]["system_tau_V-NRMOS-SCOREQ"], 1.0)


if __name__ == "__main__":
    unittest.main()
