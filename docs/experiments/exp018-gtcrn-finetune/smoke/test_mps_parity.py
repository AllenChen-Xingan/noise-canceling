"""mps_parity.py 的测试（标准库 unittest）。

CPU 自比必须全过（误差为 0）；人为扰动必须被判不过——证明判据会红。
需要 GTCRN_DIR 指向官方代码目录；没给则跳过需要模型的那几条。
跑法：GTCRN_DIR=<resources/gtcrn> python -m unittest test_mps_parity -v
"""
import os
import sys
import unittest

import torch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mps_parity as mp  # noqa: E402
import step_timing as st  # noqa: E402

GTCRN_DIR = os.environ.get("GTCRN_DIR")


class RelTest(unittest.TestCase):
    def test_rel_zero_and_scale(self):
        a = torch.ones(10)
        self.assertEqual(mp.rel(a, a), 0.0)
        self.assertAlmostEqual(mp.rel(a * 1.1, a), 0.1, places=6)


class VerdictTest(unittest.TestCase):
    def _ok(self):
        return dict(forward_rel=0.0, grad_rel_max=0.0, grad_any_nan_dev=False,
                    train_loss_finite=True, train_loss_rel_max=0.0, train_weight_rel=0.0)

    def test_all_zero_passes(self):
        self.assertTrue(mp.verdict(self._ok())["pass"])

    def test_nan_grad_fails(self):
        r = self._ok()
        r["grad_any_nan_dev"] = True
        self.assertFalse(mp.verdict(r)["pass"])

    def test_forward_drift_fails(self):
        r = self._ok()
        r["forward_rel"] = 0.05
        self.assertFalse(mp.verdict(r)["checks"]["forward"])


@unittest.skipUnless(GTCRN_DIR, "需要 GTCRN_DIR 指向官方代码目录")
class CompareTest(unittest.TestCase):
    def test_cpu_vs_cpu_is_exact(self):
        torch.manual_seed(0)
        GTCRN, HybridLoss = st.load_official(GTCRN_DIR)
        r = mp.compare(GTCRN(), "cpu", HybridLoss, n_batch=1, seconds=0.5, steps=2, lr=1e-4)
        v = mp.verdict(r)
        self.assertTrue(v["pass"], r)
        self.assertEqual(r["forward_rel"], 0.0)


if __name__ == "__main__":
    unittest.main()
