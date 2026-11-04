"""022 两个小工具的测试。用法：python3 -I -m unittest discover -s <本目录>"""
import math
import os
import plistlib
import struct
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_devices  # noqa: E402
import read_tunings  # noqa: E402


def be(vals):
    return struct.pack(">%df" % len(vals), *vals)


class CheckDevicesTest(unittest.TestCase):
    def test_real_table_passes(self):
        with open(check_devices.DEFAULT, encoding="utf-8") as fh:
            text = fh.read()
        keys = check_devices.read_keys(check_devices.key_section(text))
        self.assertIn("mba-builtin", keys)
        self.assertIn("non-laptop-phone", keys)
        self.assertEqual(check_devices.problems(keys), [])
        # 下游按整份文件读；整份读出的必须正好是键表那一节
        self.assertEqual(check_devices.read_keys(text), keys)
        for mode in ("standard", "voice-isolation", "wide-spectrum", "unknown"):
            self.assertNotIn(mode, keys)

    def test_stray_backtick_row_outside_key_section_is_caught(self):
        text = ("## 一、键表\n| `mba-builtin` | x |\n"
                "## 二、模式\n| `standard` | y |\n")
        self.assertEqual(check_devices.stray_rows(text), ["standard"])
        self.assertEqual(check_devices.read_keys(check_devices.key_section(text)),
                         ["mba-builtin"])

    def test_duplicate_and_bad_name(self):
        errs = check_devices.problems(["mba-builtin", "mba-builtin", "MBA_x"])
        self.assertTrue(any("重复" in e for e in errs))
        self.assertTrue(any("格式" in e for e in errs))

    def test_parent_without_children(self):
        errs = check_devices.problems(["non-laptop"])
        self.assertTrue(any("子键" in e for e in errs))

    def test_empty_table(self):
        self.assertTrue(check_devices.problems([]))

    def test_row_pattern_ignores_prose(self):
        text = "| 键 | 说明 |\n| `a-b` | x |\n说到 `c` 不算\n"
        self.assertEqual(check_devices.read_keys(text), ["a-b"])


class ReadTuningsTest(unittest.TestCase):
    def make_tree(self, root, aid, re_rows, im_rows):
        strips = os.path.join(root, "AID%d" % aid, "DSP", "Strips")
        graphs = os.path.join(root, "AID%d" % aid, "DSP", "Graphs")
        os.makedirs(strips)
        os.makedirs(graphs)
        beam = {
            "RealCoefficients": {"Mic%d" % i: be(r) for i, r in enumerate(re_rows)},
            "ImagCoefficients": {"Mic%d" % i: be(r) for i, r in enumerate(im_rows)},
        }
        with open(os.path.join(strips, "builtin_microphone_general_aufx_bmfd.plist"), "wb") as fh:
            plistlib.dump({"Beam0": beam, "nMicsUsed": len(re_rows)}, fh)
        with open(os.path.join(graphs, "builtin_microphone_general.dspg"), "w") as fh:
            fh.write("box dcBlocker (aufx dccb appl) 1 1\nbox BF (aufx bmfd appl) 2 4\n"
                     "box split split 1 3\n")

    def test_delay_and_sum_has_zero_db_coherent_gain(self):
        n = 481
        w = [1.0 / 3] * n
        with tempfile.TemporaryDirectory() as root:
            self.make_tree(root, 7, [w, w, w], [[0.0] * n] * 3)
            rows = read_tunings.scan(root)
        self.assertEqual(len(rows), 1)
        r = rows[0]
        self.assertEqual(r["aid"], 7)
        self.assertEqual(r["units"], ["dccb", "bmfd"])
        # 等权求和 w=1/3：‖w‖ = 1/√3 → −4.77 dB；|Σw| = 1 → 0 dB；相对白噪声增益 0 dB
        self.assertAlmostEqual(r["norm_db"][0], -4.77, places=2)
        self.assertAlmostEqual(r["in_phase_db"][0], 0.0, places=2)
        self.assertAlmostEqual(r["wng_rel_db"][0], 0.0, places=2)

    def test_superdirective_weights_three_quantities_exact(self):
        n = 481
        with tempfile.TemporaryDirectory() as root:
            self.make_tree(root, 3, [[2.0] * n, [-2.0] * n, [1.0] * n], [[0.0] * n] * 3)
            r = read_tunings.scan(root)[0]
        # w = (2, −2, 1)：‖w‖ = 3 → +9.54；|Σw| = 1 → 0；1/(3·√3) → −14.31 dB
        self.assertAlmostEqual(r["norm_db"][0], 9.54, places=2)
        self.assertAlmostEqual(r["in_phase_db"][0], 0.0, places=2)
        self.assertAlmostEqual(r["wng_rel_db"][0], -14.31, places=2)

    def test_complex_weights_use_magnitude_of_sum(self):
        n = 481
        with tempfile.TemporaryDirectory() as root:
            # w = (1, j)：|Σw| = √2 → +3.01；‖w‖ = √2；相对 = √2/(√2·√2) → −3.01
            self.make_tree(root, 5, [[1.0] * n, [0.0] * n], [[0.0] * n, [1.0] * n])
            r = read_tunings.scan(root)[0]
        self.assertAlmostEqual(r["in_phase_db"][0], 3.01, places=2)
        self.assertAlmostEqual(r["wng_rel_db"][0], -3.01, places=2)

    def test_bad_length_rejected(self):
        with self.assertRaises(ValueError):
            read_tunings.f32_be(b"\x00\x01\x02")

    def test_sorted_by_aid_number_not_text(self):
        n = 481
        w = [[0.5] * n] * 2
        z = [[0.0] * n] * 2
        with tempfile.TemporaryDirectory() as root:
            self.make_tree(root, 10, w, z)
            self.make_tree(root, 9, w, z)
            self.assertEqual([r["aid"] for r in read_tunings.scan(root)], [9, 10])

    @unittest.skipUnless(os.path.isdir(read_tunings.ROOT), "本机没有调音目录")
    def test_real_system_tunings_readable(self):
        rows = read_tunings.scan()
        self.assertGreater(len(rows), 10)
        self.assertTrue(all("bmfd" in r["units"] for r in rows if r["units"]))


if __name__ == "__main__":
    unittest.main()
