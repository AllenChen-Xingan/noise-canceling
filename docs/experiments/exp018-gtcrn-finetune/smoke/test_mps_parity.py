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

    @unittest.skipUnless(torch.backends.mps.is_available(), "本机没有 MPS")
    def test_rel_accepts_mps_tensors(self):
        # 回归：先 .double() 再 .cpu() 会在 MPS 上报错（MPS 不支持 float64）
        a = torch.ones(4, device="mps")
        self.assertEqual(mp.rel(a, a), 0.0)


class VerdictTest(unittest.TestCase):
    def _ok(self):
        return dict(forward_rel=0.0, grad_rel_model=0.0, grad_share_max=0.0, grad_any_nan_dev=False,
                    train_loss_finite=True, train_loss_rel_max=0.0, train_weight_rel=0.0,
                    export_unpatched=True, export_eval_rel=0.0)

    def test_all_zero_passes(self):
        self.assertTrue(mp.verdict(self._ok())["pass"])

    def test_nan_grad_fails(self):
        r = self._ok()
        r["grad_any_nan_dev"] = True
        self.assertFalse(mp.verdict(r)["pass"])

    def test_model_grad_drift_fails(self):
        r = self._ok()
        r["grad_rel_model"] = 0.8  # 2026-11-05 修复前实测约 0.84
        self.assertFalse(mp.verdict(r)["checks"]["grad"])

    def test_patched_export_fails(self):
        r = self._ok()
        r["export_unpatched"] = False
        self.assertFalse(mp.verdict(r)["checks"]["export_eval"])

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
        self.assertTrue(r["export_unpatched"])
        self.assertGreater(r["train_state_keys"], len(list(GTCRN().parameters())))  # 含 BN 统计


if __name__ == "__main__":
    unittest.main()
