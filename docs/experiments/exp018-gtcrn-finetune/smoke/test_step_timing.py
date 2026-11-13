"""step_timing.py 的测试（标准库 unittest，本机 venv 没有 pytest）。

需要官方代码目录：环境变量 GTCRN_DIR 指向 resources/gtcrn；没给则跳过。
跑法：GTCRN_DIR=<resources/gtcrn> python -m unittest test_step_timing -v
"""
import json
import os
import sys
import tempfile
import unittest
import warnings

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import step_timing as st  # noqa: E402

GTCRN_DIR = os.environ.get("GTCRN_DIR")


class FallbackOpsTest(unittest.TestCase):
    def test_extracts_and_dedups_op_names(self):
        msgs = [
            "The operator 'aten::foo' is not currently supported on the MPS backend and will fall back to run on the CPU.",
            "The operator 'aten::foo' is not currently supported on the MPS backend and will fall back to run on the CPU.",
            "The operator 'aten::bar.out' is not currently supported on the MPS backend and will fall back to run on the CPU.",
            "something unrelated",
        ]
        caught = [warnings.WarningMessage(m, UserWarning, "x", 1) for m in msgs]
        self.assertEqual(st.fallback_ops(caught), ["aten::foo", "aten::bar.out"])


@unittest.skipUnless(GTCRN_DIR, "需要 GTCRN_DIR 指向官方代码目录")
class RunTest(unittest.TestCase):
    def test_cpu_tiny_run_reports_fields(self):
        res = st.run(GTCRN_DIR, "cpu", batch=1, seconds=0.5, warmup=1, steps=2, ckpt=None)
        self.assertEqual(res["steps"], 2)
        self.assertGreater(res["params"], 10_000)
        self.assertGreater(res["step_s_median"], 0)
        self.assertTrue(res["loss_finite"])
        self.assertEqual(res["mps_fallback_ops"], [])

    def test_refuses_to_overwrite(self):
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "t.json")
            args = ["--gtcrn-dir", GTCRN_DIR, "--batch", "1", "--seconds", "0.5",
                    "--warmup", "0", "--steps", "1", "--out", out]
            st.main(args)
            with open(out, encoding="utf-8") as f:
                self.assertIn("step_s_median", json.load(f))
            with self.assertRaises(SystemExit):
                st.main(args)


if __name__ == "__main__":
    unittest.main()
