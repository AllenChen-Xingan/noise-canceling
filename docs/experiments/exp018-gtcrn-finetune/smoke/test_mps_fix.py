"""mps_fix.py 的测试。

1. CPU 上：换写法后的卷积与原卷积逐点一致（前向 + 梯度），覆盖 GTCRN 用到的 dilation 2 · 5
2. 换写法不改 state_dict（键、形状、数值），官方检查点照常载入
3. MPS 上（有才跑）：原卷积在 MPS 上差约 100%（复现问题，防「问题已修」时还在白绕），
   换写法后 MPS 与 CPU 一致
4. 转置卷积（解码器）：CPU 上等价；MPS 上原写法输入梯度错、换写法后对
5. 整个 GTCRN：只换了编码器 en_convs.3 / 4 与解码器 de_convs.0 / 1 四个
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


def make_t(d):
    torch.manual_seed(10 + d)  # 与 GTCRN 解码器同形：padding=(2d, 1)
    return nn.ConvTranspose2d(16, 16, (3, 3), stride=1, padding=(2 * d, 1), dilation=(d, 1), groups=16)


def grads_of(m, x):
    x = x.detach().clone().requires_grad_()
    m.zero_grad()
    y = m(x)
    y.square().sum().backward()
    return y.detach(), x.grad.detach(), m.weight.grad.detach()


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

    def test_transpose_forward_and_grad_match(self):
        for d in (2, 5):
            ref = make_t(d)
            new = copy.deepcopy(ref)
            self.assertEqual(fx.patch(new), [""])
            x = torch.randn(2, 16, 40, 33)
            for a, b in zip(grads_of(new, x), grads_of(ref, x)):
                self.assertEqual(a.shape, b.shape)
                self.assertLess(rel(a, b), 1e-5)

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

    def test_transpose_input_grad_bug_and_fix(self):
        # 转置卷积前向对，但对输入的梯度在 MPS 上错（它内部就是带空洞的逐通道卷积）
        ref = make_t(2)
        x = torch.randn(1, 16, 40, 33)
        y_c, gx_c, gw_c = grads_of(ref, x)
        y_b, gx_b, _ = grads_of(copy.deepcopy(ref).to("mps"), x.to("mps"))
        self.assertLess(rel(y_b, y_c), 1e-5)
        self.assertGreater(rel(gx_b, gx_c), 0.1, "MPS 转置卷积的输入梯度已经对了：上游可能修了")
        fixed = copy.deepcopy(ref)
        fx.patch(fixed)
        y_f, gx_f, gw_f = grads_of(fixed.to("mps"), x.to("mps"))
        for a, b in ((y_f, y_c), (gx_f, gx_c), (gw_f, gw_c)):
            self.assertLess(rel(a, b), 1e-4)


@unittest.skipUnless(GTCRN_DIR, "需要 GTCRN_DIR 指向官方代码目录")
class GtcrnTest(unittest.TestCase):
    def test_patches_exactly_four_dilated_depthwise_convs(self):
        sys.path.insert(0, GTCRN_DIR)
        from gtcrn import GTCRN  # type: ignore
        m = GTCRN()
        self.assertEqual(sorted(fx.patch(m)),
                         ["decoder.de_convs.0.depth_conv", "decoder.de_convs.1.depth_conv",
                          "encoder.en_convs.3.depth_conv", "encoder.en_convs.4.depth_conv"])


if __name__ == "__main__":
    unittest.main()
