"""TUM 风噪（Zenodo 6687982）的切分单位与分层（2026-11-25 看云上清单后补）。

上游说明原文只有一句："Wind Noise Dataset including artificially generated and mobilephone recorded samples."
包内结构（按 HTTP Range 读 zip 目录得来，不下音频）：
- artifitial_wind_noise/{train,eval,test}/{const,gusts}_NN.wav：100 个，上游拼写如此；说明说是「artificially generated」→ 合成那一层
- mobilephone_wind_noise/{normal_wind,strong_wind}/<前缀>_<序号>.wav：378 个，32 个前缀，其中 20 个前缀两个文件夹都有
  → 前缀当一次录音（027：同一次录音被切成多个文件时按录音切）。上游没写前缀含义；按前缀并只会让单位更粗，不会跨份
"""
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import split  # noqa: E402

T = "train-raw/tum-wind"


def rule():
    return split.load_rules()["sources"]["tum-wind"]


class TumUnits(unittest.TestCase):
    def test_artificial_is_synth_stratum(self):
        import re

        rx = re.compile(rule()["stratum_regex"])
        self.assertTrue(rx.search(f"{T}/artifitial_wind_noise/eval/gusts_01.wav"))
        self.assertFalse(rx.search(f"{T}/mobilephone_wind_noise/normal_wind/000_001.wav"))

    def test_phone_prefix_is_one_recording(self):
        a = split.unit_of(f"{T}/mobilephone_wind_noise/normal_wind/000_001.wav", rule())
        b = split.unit_of(f"{T}/mobilephone_wind_noise/strong_wind/000_005.wav", rule())
        c = split.unit_of(f"{T}/mobilephone_wind_noise/normal_wind/001_001.wav", rule())
        self.assertEqual(a, b)
        self.assertNotEqual(a, c)

    def test_artificial_files_own_units(self):
        """合成的每个文件各是一个单位；train / eval / test 里同名的 const_01 是不同文件。"""
        u = {split.unit_of(f"{T}/artifitial_wind_noise/{d}/const_01.wav", rule()) for d in ("train", "eval", "test")}
        self.assertEqual(len(u), 3)

    def test_expect_exact(self):
        self.assertEqual(rule()["expect"].get("files"), 478)


if __name__ == "__main__":
    unittest.main()
