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

    def test_sides_balanced(self):
        for seed in range(50):
            got = [x for _, x in B.plan(np.random.default_rng(seed), 4)]
            self.assertEqual(sum(got), 2, seed)

    def test_limits(self):
        self.assertLessEqual(len(B.CLIPS) * 2, B.MAX_FILES)
        self.assertLessEqual(len(B.CLIPS) * 2 * 12, B.MAX_SECONDS)


class Gain(unittest.TestCase):
    class Meter:  # 假响度计：响度 = 20·log10(峰值)，够测增益逻辑
        def integrated_loudness(self, x):
            return 20 * np.log10(float(np.max(np.abs(x))))

    def test_equal_loudness_when_one_side_hits_peak(self):
        quiet = np.array([0.0, 0.01, -0.01], dtype=np.float32)
        loud = np.array([0.0, 0.9, -0.9], dtype=np.float32)
        (gq, gl), target = B.pair_gains([quiet, loud], self.Meter())
        m = self.Meter()
        self.assertAlmostEqual(m.integrated_loudness(quiet * gq), m.integrated_loudness(loud * gl), places=6)
        self.assertLessEqual(max(np.max(np.abs(quiet * gq)), np.max(np.abs(loud * gl))), B.PEAK + 1e-9)

    def test_target_is_lufs_when_room(self):
        x = np.array([0.0, 0.01], dtype=np.float32)
        _, target = B.pair_gains([x, x], self.Meter())
        self.assertAlmostEqual(target, B.LUFS)


if __name__ == "__main__":
    unittest.main()
