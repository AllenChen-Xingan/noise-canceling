"""build_listen.py 的测试：页面不泄露答案、听测目录只放该放的、左右与题序可复现、响度增益不超峰值。"""
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import build_listen as B  # noqa: E402


class Page(unittest.TestCase):
    def test_page_has_no_answers(self):
        page = B.PAGE.replace("__QUESTIONS__", "".join(B.question(f"q{i}", i) for i in range(1, 5)))
        B.check_listen_dir_page_only = None
        low = page.lower()
        for bad in B.FORBIDDEN:
            self.assertNotIn(bad, low)

    def test_check_rejects_extra_file_and_leaky_page(self):
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            (t / "index.html").write_text("x")
            (t / "q1_X.wav").write_bytes(b"")
            files = {"index.html": "", "q1_X.wav": ""}
            B.check_listen_dir(t, files, "ok")
            (t / "unblind-key.json").write_text("{}")
            with self.assertRaises(AssertionError):
                B.check_listen_dir(t, files, "ok")
            (t / "unblind-key.json").unlink()
            with self.assertRaises(AssertionError):
                B.check_listen_dir(t, files, "这是 GTCRN")


class Plan(unittest.TestCase):
    def test_reproducible_and_complete(self):
        a = B.plan(np.random.default_rng(B.SEED), 4)
        b = B.plan(np.random.default_rng(B.SEED), 4)
        self.assertEqual(a, b)
        self.assertEqual(sorted(i for i, _ in a), [0, 1, 2, 3])

    def test_limits(self):
        self.assertLessEqual(len(B.CLIPS) * 2, B.MAX_FILES)
        self.assertLessEqual(len(B.CLIPS) * 2 * 12, B.MAX_SECONDS)


class Gain(unittest.TestCase):
    def test_peak_cap(self):
        class Meter:
            def integrated_loudness(self, x):
                return -60.0  # 很轻 → 想放大 34 dB
        x = np.array([0.0, 0.5, -0.5], dtype=np.float32)
        g = B.match_gain(x, 16000, Meter())
        self.assertLessEqual(g * 0.5, B.PEAK + 1e-9)


if __name__ == "__main__":
    unittest.main()
