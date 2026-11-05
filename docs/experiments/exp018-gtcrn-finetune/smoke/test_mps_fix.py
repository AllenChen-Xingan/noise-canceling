"""mps_fix.py 的测试。

1. CPU 上：换写法后的卷积与原卷积逐点一致（前向 + 梯度），覆盖 GTCRN 用到的 dilation 2 · 5
2. 换写法不改 state_dict（键、形状、数值），官方检查点照常载入
3. MPS 上（有才跑）：原卷积在 MPS 上差约 100%（复现问题，防「问题已修」时还在白绕），
   换写法后 MPS 与 CPU 一致
4. 整个 GTCRN：只换了 en_convs.3 / en_convs.4 两个
跑法：GTCRN_DIR=<resources/gtcrn> python -m unittest test_mps_fix -v
"""
import copy
import os
import sys
import unittest

import torch
from torch import nn

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mps_fix as fx  # noqa: E402

GTCRN_DIR = os.environ.get("GTCRN_DIR")
HAS_MPS = torch.backends.mps.is_available()


def rel(a, b):
    a, b = a.detach().cpu().double(), b.detach().cpu().double()
    return float((a - b).norm() / (b.norm() + 1e-12))


def make(d):
    torch.manual_seed(d)
    return nn.Conv2d(16, 16, (3, 3), stride=1, padding=(0, 1), dilation=(d, 1), groups=16)


class CpuEquivalenceTest(unittest.TestCase):
    def test_forward_and_grad_match(self):
        for d in (2, 5):
            ref = make(d)
            new = copy.deepcopy(ref)
            self.assertEqual(fx.patch(new), [""])
            x1 = torch.randn(2, 16, 40, 33, requires_grad=True)
            x2 = x1.detach().clone().requires_grad_()
            y1, y2 = ref(x1), new(x2)
            self.assertEqual(y1.shape, y2.shape)
            self.assertLess(rel(y2, y1), 1e-6)
            y1.square().sum().backward()
            y2.square().sum().backward()
            self.assertLess(rel(new.weight.grad, ref.weight.grad), 1e-5)
            self.assertLess(rel(x2.grad, x1.grad), 1e-5)

    def test_time_padding_also_matches(self):
        ref = nn.Conv2d(8, 8, (3, 3), padding=(4, 1), dilation=(2, 1), groups=8)
        new = copy.deepcopy(ref)
        fx.patch(new)
        x = torch.randn(1, 8, 20, 9)
        self.assertLess(rel(new(x), ref(x)), 1e-6)

    def test_state_dict_unchanged_and_untouched_layers(self):
        m = nn.Sequential(make(2), nn.Conv2d(16, 16, 3, padding=1), make(1))
        before = {k: v.clone() for k, v in m.state_dict().items()}
        self.assertEqual(fx.patch(m), ["0"])
        after = m.state_dict()
        self.assertEqual(list(before), list(after))
        for k in before:
            self.assertTrue(torch.equal(before[k], after[k]))


@unittest.skipUnless(HAS_MPS, "本机没有 MPS")
class MpsTest(unittest.TestCase):
    def test_bug_still_present_and_fix_works(self):
        ref = make(2)
        x = torch.randn(1, 16, 40, 33)
        y_cpu = ref(x)
        broken = copy.deepcopy(ref).to("mps")
        self.assertGreater(rel(broken(x.to("mps")), y_cpu), 0.1,
                           "MPS 原卷积已经算对了：上游可能修了，考虑去掉 mps_fix")
        fixed = copy.deepcopy(ref)
        fx.patch(fixed)
        fixed = fixed.to("mps")
        self.assertLess(rel(fixed(x.to("mps")), y_cpu), 1e-5)


@unittest.skipUnless(GTCRN_DIR, "需要 GTCRN_DIR 指向官方代码目录")
class GtcrnTest(unittest.TestCase):
    def test_patches_exactly_two_encoder_convs(self):
        sys.path.insert(0, GTCRN_DIR)
        from gtcrn import GTCRN  # type: ignore
        m = GTCRN()
        self.assertEqual(sorted(fx.patch(m)),
                         ["encoder.en_convs.3.depth_conv", "encoder.en_convs.4.depth_conv"])


if __name__ == "__main__":
    unittest.main()
