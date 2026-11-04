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
            keys = check_devices.read_keys(fh.read())
        self.assertIn("mba-builtin", keys)
        self.assertIn("non-laptop-phone", keys)
        self.assertEqual(check_devices.problems(keys), [])

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
        # 等权求和：范数 = 1/sqrt(3) → −4.8 dB；同相增益/范数归一后 = 0 dB
        self.assertAlmostEqual(r["norm_db"][0], round(20 * math.log10(1 / math.sqrt(3)), 1))
        self.assertAlmostEqual(r["coherent_db"][0], 0.0)

    def test_superdirective_weights_show_negative_coherent_gain(self):
        n = 481
        with tempfile.TemporaryDirectory() as root:
            self.make_tree(root, 3, [[2.0] * n, [-2.0] * n, [1.0] * n], [[0.0] * n] * 3)
            r = read_tunings.scan(root)[0]
        self.assertGreater(r["norm_db"][0], 9.0)   # 范数 3 → +9.5 dB
        self.assertLess(r["coherent_db"][0], -10.0)

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
