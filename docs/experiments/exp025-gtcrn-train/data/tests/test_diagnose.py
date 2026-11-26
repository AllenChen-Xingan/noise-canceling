"""diagnose.py（候选噪声片段的诊断）的判据：逐秒电平 · 频谱峰 · 平稳度 · 命令行。

- 平稳度那一项不许另写一份：diagnose 直接调 `screen.level_std_db`，这里核两处给同一个数
- 频谱峰：已知频率的正弦 + 少量噪声，峰要落在 ±2 Hz 内、电平远高于中位数；两个音都要找得到；
  白噪下取 5 个峰，两两至少隔 20 Hz
- 命令行：清单里每个文件一行，逐秒电平与峰都写进去，并打一行汇总
"""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import diagnose  # noqa: E402
import screen  # noqa: E402

SR = 16000


def tone(freq, sec=3.0, amp=0.5, noise_amp=1e-4, sr=SR, seed=0):
    t = np.arange(int(sec * sr)) / sr
    return amp * np.sin(2 * np.pi * freq * t) + noise_amp * np.random.default_rng(seed).standard_normal(t.size)


def noise(sec, amp=0.1, seed=0, sr=SR):
    return amp * np.random.default_rng(seed).standard_normal(int(sec * sr))


class PerSecond(unittest.TestCase):
    def test_second_two_twenty_db_louder(self):
        x = noise(3, seed=1)
        x[SR:2 * SR] *= 10.0
        lv = diagnose.per_second_dbfs(x, SR)
        self.assertEqual(len(lv), 3)
        self.assertAlmostEqual(lv[1] - lv[0], 20.0, delta=0.5)
        self.assertAlmostEqual(lv[2] - lv[0], 0.0, delta=1.5)

    def test_partial_second_dropped(self):
        self.assertEqual(len(diagnose.per_second_dbfs(noise(2.5, seed=2), SR)), 2)

    def test_short_clip_raises(self):
        with self.assertRaises(ValueError):
            diagnose.per_second_dbfs(np.zeros(SR - 1), SR)
        with self.assertRaises(ValueError):
            diagnose.per_second_dbfs(np.zeros(0), SR)

    def test_silent_second_is_finite(self):
        """整秒静音会算出 -inf：要压到有限值，否则 jsonl 写不出合法 JSON。"""
        x = noise(2, seed=3)
        x[SR:] = 0.0
        lv = diagnose.per_second_dbfs(x, SR)
        self.assertTrue(np.all(np.isfinite(lv)), lv)


class Peaks(unittest.TestCase):
    def test_one_khz_sine(self):
        peaks = diagnose.spectral_peaks(tone(1000.0), SR)
        f, level = peaks[0]
        self.assertAlmostEqual(f, 1000.0, delta=2.0)
        self.assertGreater(level, 20.0)  # 远高于中位数

    def test_two_tones_both_found(self):
        t = np.arange(2 * SR) / SR
        x = 0.3 * np.sin(2 * np.pi * 50 * t) + 0.3 * np.sin(2 * np.pi * 120 * t)
        x += 1e-5 * np.random.default_rng(3).standard_normal(t.size)
        fs = [f for f, _ in diagnose.spectral_peaks(x, SR, k=3)]
        self.assertTrue(any(abs(f - 50) <= 2 for f in fs), fs)
        self.assertTrue(any(abs(f - 120) <= 2 for f in fs), fs)

    def test_peaks_at_least_20hz_apart(self):
        fs = [f for f, _ in diagnose.spectral_peaks(noise(3, seed=4), SR, k=5)]
        self.assertEqual(len(fs), 5)
        for i in range(len(fs)):
            for j in range(i + 1, len(fs)):
                self.assertGreaterEqual(abs(fs[i] - fs[j]), 20.0, fs)

    def test_peaks_are_local_maxima(self):
        x = tone(1000.0)
        f, power = diagnose.welch_power(x, SR)
        lvl = 10 * np.log10(np.maximum(power, 1e-300))
        for hz, _ in diagnose.spectral_peaks(x, SR):
            i = int(np.argmin(np.abs(f - hz)))
            self.assertGreaterEqual(lvl[i], lvl[i - 1], hz)
            self.assertGreaterEqual(lvl[i], lvl[i + 1], hz)

    def test_band_limits_respected(self):
        fs = [f for f, _ in diagnose.spectral_peaks(noise(2, seed=5), SR, k=3, fmin=200.0, fmax=2000.0)]
        self.assertTrue(all(200.0 <= f <= 2000.0 for f in fs), fs)


class WholeClip(unittest.TestCase):
    def test_level_std_db_is_screens_definition(self):
        """平稳度只有一个正本（screen.level_std_db）：两边必须给同一个数。"""
        x = noise(4, seed=6)
        self.assertEqual(diagnose.diagnose(x, SR)["level_std_db"], screen.level_std_db(x, SR))

    def test_short_clip_has_no_level_std(self):
        row = diagnose.diagnose(noise(1.5, seed=7), SR)
        self.assertIsNone(row["level_std_db"])
        self.assertEqual(len(row["per_second_dbfs"]), 1)
        self.assertTrue(row["peaks"])


class Cli(unittest.TestCase):
    def test_one_line_per_file(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            names = ["a.wav", "b.wav"]
            for i, n in enumerate(names):
                sf.write(str(root / n), noise(3, seed=10 + i), SR)
            (root / "list.txt").write_text("\n".join(names) + "\n", encoding="utf-8")
            out = root / "out.jsonl"
            r = subprocess.run([sys.executable, str(HERE / "diagnose.py"), "--root", str(root),
                                "--list", str(root / "list.txt"), "--out", str(out)],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            rows = [json.loads(ln) for ln in out.read_text(encoding="utf-8").splitlines()]
            self.assertEqual([row["path"] for row in rows], names)
            for row in rows:
                self.assertEqual(len(row["per_second_dbfs"]), 3)
                self.assertEqual(len(row["peaks"]), 3)
                self.assertTrue(np.isfinite(row["level_std_db"]))
            self.assertIn("中位数", r.stdout)
            self.assertIn("频谱峰", r.stdout)

    def test_comments_and_blank_lines_skipped(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            sf.write(str(root / "a.wav"), noise(3, seed=12), SR)
            (root / "list.txt").write_text("# 注释\n\na.wav\n", encoding="utf-8")
            out = root / "out.jsonl"
            r = subprocess.run([sys.executable, str(HERE / "diagnose.py"), "--root", str(root),
                                "--list", str(root / "list.txt"), "--out", str(out)],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(len(out.read_text(encoding="utf-8").splitlines()), 1)


if __name__ == "__main__":
    unittest.main()
