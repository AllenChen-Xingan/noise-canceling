"""click_diag.py 的测试：参数与 Rust 正本一致；合成信号上检出孤立尖脉冲、不检出原声自带的瞬态。"""
import sys
import unittest
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import click_diag as C  # noqa: E402


class Params(unittest.TestCase):
    def test_match_rust_table(self):
        got = C.params_from_rust(C.TABLE.read_text())
        self.assertEqual((got["k"], got["resid_frac"], got["floor_db"]), (C.K, C.RESID_FRAC, C.FLOOR_DB))

    def test_parser(self):
        txt = 'pub const CLICK: Lines = Lines {\n p("k", Some(3.0), &[]),\n p("floor_db", Some(-1.5), &[]),\n};'
        self.assertEqual(C.params_from_rust(txt), {"k": 3.0, "floor_db": -1.5})


class Detect(unittest.TestCase):
    def setUp(self):
        rng = np.random.default_rng(0)
        n = C.SR
        t = np.arange(n) / C.SR
        self.clean = (0.3 * np.sin(2 * np.pi * 200 * t)).astype(np.float32)
        self.bg = (0.002 * rng.standard_normal(n)).astype(np.float32)
        self.mix = self.clean + self.bg

    def test_spike_in_output_is_a_click(self):
        out = self.mix.copy()
        out[8000] += 0.5
        self.assertIn(7999, C.click_positions(out, self.mix, self.clean, self.bg))

    def test_transient_already_in_input_is_not(self):
        mix = self.mix.copy()
        mix[8000] += 0.5
        self.assertEqual(C.click_positions(mix.copy(), mix, self.clean, self.bg), [])

    def test_clean_passthrough_has_none(self):
        self.assertEqual(C.click_positions(self.mix.copy(), self.mix, self.clean, self.bg), [])


if __name__ == "__main__":
    unittest.main()
