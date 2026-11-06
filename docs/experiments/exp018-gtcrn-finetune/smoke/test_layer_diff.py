"""layer_diff.py 的测试：CPU 自比每层差为 0、找不到「第一个坏层」；元组输出取第一个张量。

跑法：GTCRN_DIR=<resources/gtcrn> python -m unittest test_layer_diff -v
"""
import os
import sys
import unittest

import torch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import layer_diff as ld  # noqa: E402

GTCRN_DIR = os.environ.get("GTCRN_DIR")


class FirstTensorTest(unittest.TestCase):
    def test_tuple(self):
        a = torch.ones(2)
        self.assertIs(ld.first_tensor((a, torch.zeros(1))), a)
        self.assertIsNone(ld.first_tensor((None, "x")))


@unittest.skipUnless(GTCRN_DIR, "需要 GTCRN_DIR 指向官方代码目录")
class DiffTest(unittest.TestCase):
    def test_cpu_self_diff_is_zero(self):
        for mode in ("eval", "train"):
            r = ld.diff(GTCRN_DIR, None, "cpu", mode, seconds=0.25)
            self.assertIsNone(r["first_bad"], mode)
            self.assertEqual(r["output_rel"], 0.0)
            self.assertGreater(len(r["layers"]), 20)
            self.assertTrue(any(l["type"] == "GRU" for l in r["layers"]))


if __name__ == "__main__":
    unittest.main()
