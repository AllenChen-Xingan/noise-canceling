"""dsp：电平函数、区间、T60、哈希。"""
import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
import dsp  # noqa: E402
from rooms import make_rir  # noqa: E402

SR = dsp.SR


def tone(sec, amp=1.0, f=1000.0):
    return amp * np.sin(2 * np.pi * f * np.arange(int(sec * SR)) / SR)


class Levels(unittest.TestCase):
    def test_p56_sine_with_silence(self):
        """满幅正弦 RMS = −3.01 dBFS；中间夹一半静音，活动电平不变。"""
        x = np.concatenate([tone(2), np.zeros(2 * SR), tone(2)])
        self.assertAlmostEqual(dsp.p56_level_db(x), -3.01, delta=0.3)
        self.assertAlmostEqual(dsp.gated_level_db(x), -3.01, delta=0.05)

    def test_p56_scales_linearly(self):
        rng = np.random.default_rng(0)
        x = rng.standard_normal(3 * SR) * (np.arange(3 * SR) % SR < SR // 2)
        a, b = dsp.p56_level_db(x), dsp.p56_level_db(0.1 * x)
        self.assertAlmostEqual(a - b, 20.0, delta=0.05)

    def test_p56_short_loud_burst(self):
        """回归：曾取「最后一个满足 15.9 dB 余量的门限」，一段 50 ms 的响声会把电平抬高约 14 dB（旧版 −21.4，应为 −35.6）。
        信号从头到尾都在响（安静段是连续正弦），所以活动电平应等于整段功率。"""
        x = np.concatenate([tone(6, 0.01), tone(0.05, 0.3), tone(4, 0.01)])
        self.assertAlmostEqual(dsp.p56_level_db(x), 10 * np.log10(np.mean(x**2)), delta=1.0)


class Intervals(unittest.TestCase):
    def test_guard_and_bounds(self):
        x = np.zeros(2 * SR)
        x[SR : SR + 3200] = tone(0.2)
        iv = dsp.intervals(x)
        self.assertEqual(len(iv), 1)
        s, e = iv[0]
        self.assertEqual(s, SR - 2 * dsp.FRAME)
        self.assertEqual(e, SR + 3200 + 2 * dsp.FRAME)

    def test_union_overlap(self):
        a = [[0, 10], [20, 30]]
        b = [[5, 25]]
        self.assertEqual(dsp.iv_union(a, b), [[0, 30]])
        self.assertEqual(dsp.iv_overlap(a, b), 10)


class Rooms(unittest.TestCase):
    def test_t60_close_to_nominal(self):
        rng = np.random.default_rng(1)
        for t60 in (0.3, 0.7, 1.5):
            r = make_rir(rng, (8, 6, 3), t60, 0.5, 0)
            self.assertLess(abs(dsp.schroeder_t60(r.rest) / t60 - 1), 0.25, t60)

    def test_outdoor_has_no_tail(self):
        r = make_rir(np.random.default_rng(2), (100, 100, 10), 0.0, 1.0, 0)
        self.assertEqual(dsp.schroeder_t60(r.rest), 0.0)

    def test_direct_split_sums(self):
        r = make_rir(np.random.default_rng(3), (5, 4, 3), 0.5, 1.0, 45)
        np.testing.assert_allclose(r.direct + r.rest, r.full)
        self.assertAlmostEqual(float(np.max(np.abs(r.direct))), 1.0, places=6)


class Trim(unittest.TestCase):
    def test_tail_trimmed(self):
        """回归（codex 第 1 轮 P1 #2）：ESC 片段的混响尾巴要截掉，只留 60 ms。"""
        import pools

        rng = np.random.default_rng(0)
        x = np.concatenate([tone(0.5), rng.standard_normal(SR) * np.exp(-np.arange(SR) / (0.1 * SR)) * 0.5])
        y, end = pools.trim_tail(x)
        self.assertLess(end, int(0.95 * SR))  # 0.5 s 的声音 + 约 0.3 s 衰到 −30 dB + 60 ms，原长 1.5 s
        self.assertEqual(float(y[-1]), 0.0)


class Hash(unittest.TestCase):
    def test_sample_hash_is_float32_bytes(self):
        x = np.array([0.1, -0.2], np.float64)
        self.assertEqual(dsp.sample_hash(x), dsp.sample_hash(x.astype(np.float32)))


if __name__ == "__main__":
    unittest.main()
