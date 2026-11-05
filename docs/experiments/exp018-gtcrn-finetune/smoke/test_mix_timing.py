"""mix_timing.py 的测试（标准库 unittest）：用临时合成的小 wav，不依赖本机数据。

跑法：python -m unittest test_mix_timing -v
"""
import os
import sys
import tempfile
import unittest

import numpy as np
import soundfile as sf

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mix_timing as mt  # noqa: E402


class MixMathTest(unittest.TestCase):
    def test_snr_is_hit(self):
        rng = np.random.default_rng(0)
        clean = rng.standard_normal(16000).astype(np.float32) * 0.01
        noise = rng.standard_normal(16000).astype(np.float32) * 0.01
        y = mt.mix_at_snr(clean, noise, 5.0)
        got = 10 * np.log10(np.mean(clean ** 2) / np.mean((y - clean) ** 2))
        self.assertAlmostEqual(got, 5.0, places=3)

    def test_clipping_guard(self):
        clean = np.ones(100, np.float32)
        y = mt.mix_at_snr(clean, np.ones(100, np.float32), 0.0)
        self.assertLessEqual(float(np.max(np.abs(y))), 0.9 + 1e-6)

    def test_resample_and_fit(self):
        x = np.zeros(44100, np.float32)
        self.assertEqual(len(mt.to_16k(x, 44100)), 16000)
        self.assertEqual(len(mt.fit(np.ones(3, np.float32), 10)), 10)


class RunTest(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.TemporaryDirectory()
        rng = np.random.default_rng(1)
        self.speech = [os.path.join(self.d.name, f"s{i}.flac") for i in range(2)]
        self.noise = [os.path.join(self.d.name, "n.wav")]
        self.rir = [os.path.join(self.d.name, "h.wav")]
        for p in self.speech:
            sf.write(p, rng.standard_normal(16000 * 3).astype(np.float32) * 0.1, 16000)
        sf.write(self.noise[0], rng.standard_normal(44100).astype(np.float32) * 0.1, 44100)
        sf.write(self.rir[0], np.exp(-np.arange(3200) / 400.0).astype(np.float32), 32000)

    def tearDown(self):
        self.d.cleanup()

    def test_single_process(self):
        r = mt.run(self.speech, self.noise, self.rir, clips=6, seconds=2.0, workers=0, p_rir=1.0)
        self.assertTrue(r["all_finite"])
        self.assertTrue(r["all_len_ok"])
        self.assertEqual(r["rir_used"], 6)

    def test_pool_matches_lengths(self):
        r = mt.run(self.speech, self.noise, [], clips=4, seconds=1.0, workers=2, p_rir=0.5)
        self.assertTrue(r["all_len_ok"])
        self.assertEqual(r["rir_used"], 0)


if __name__ == "__main__":
    unittest.main()
