"""exp016 离线脚本公共模块的测试（不跑任何降噪模型）。

用法（项目根下）：<主目录>/_tmp/exp002-venv/bin/python -m unittest discover -s docs/experiments/exp016-candidates/tests
"""
import json
import re
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import _ext  # noqa: E402

RUST = _ext.ROOT / "src/eval/src/system/external.rs"


def clip(cid, scene, root, n=1600, additive=True, has_ref=True, seed=0):
    rng = np.random.default_rng(seed)
    a, h = {}, {}
    for k in ("mix", "owner", "noise", "interf"):
        p = root / f"{cid}-{k}.wav"
        sf.write(str(p), (rng.standard_normal(n) * 0.1).astype(np.float32), _ext.SR, subtype="FLOAT")
        a[k], h[k] = p.name, _ext.sha256_file(p)
    return {"id": cid, "scene": scene, "dur_samples": n, "additive": additive, "has_clean_ref": has_ref,
            "audio": a, "hash": {"wav_sha256": h}}


class CrossCheckRust(unittest.TestCase):
    """Python 里的格式常量是第二处表示：与 Rust 正本 external.rs 比对。"""

    def test_schema_and_kinds_match_rust(self):
        src = RUST.read_text()
        self.assertIn(f'pub const SCHEMA: &str = "{_ext.SCHEMA}";', src)
        kinds = re.findall(r'Kind::\w+ => "(\w+)"', src)
        self.assertEqual(tuple(kinds), _ext.KINDS)

    def test_clips_path_matches_scenes_json(self):
        self.assertEqual(_ext.scenes_clips_path(), _ext.CLIPS)


class Inputs(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.TemporaryDirectory()
        self.root = Path(self.d.name)

    def tearDown(self):
        self.d.cleanup()

    def test_trunc_picks_first_clip_with_ref_per_scene(self):
        cs = [{"id": "a0", "scene": "cafe", "has_clean_ref": False}, {"id": "a1", "scene": "cafe", "has_clean_ref": True},
              {"id": "a2", "scene": "cafe", "has_clean_ref": True}, {"id": "b0", "scene": "home-quiet", "has_clean_ref": True}]
        self.assertEqual(_ext.trunc_picks(cs), {"a1", "b0"})

    def test_inputs_follow_rust_kind_input(self):
        c = clip("c0", "cafe", self.root, n=1601)
        got = dict(_ext.inputs_of(c, self.root, {"c0"}))
        self.assertEqual(list(got), ["mix", "bg", "trunc"])
        mix = _ext.read16(self.root / "c0-mix.wav")
        bg = _ext.read16(self.root / "c0-noise.wav") + _ext.read16(self.root / "c0-interf.wav")
        np.testing.assert_array_equal(got["mix"], mix)
        np.testing.assert_array_equal(got["bg"], bg.astype(np.float32))
        self.assertTrue(np.all(got["trunc"][800:] == 0) and np.array_equal(got["trunc"][:800], mix[:800]))
        # 非 additive 不给 bg；不在 picks 不给 trunc
        c2 = clip("c1", "cafe", self.root, additive=False)
        self.assertEqual([k for k, _ in _ext.inputs_of(c2, self.root, set())], ["mix"])

    def test_hash_mismatch_is_rejected(self):
        c = clip("c0", "cafe", self.root)
        c["hash"]["wav_sha256"]["mix"] = "0" * 64
        with self.assertRaises(ValueError):
            _ext.inputs_of(c, self.root, set())


class Lookahead(unittest.TestCase):
    def test_measure_lookahead_of_shift(self):
        x = (np.random.default_rng(1).standard_normal(16000) * 0.1).astype(np.float32)
        for k in (0, 1, 300):
            f = lambda v, k=k: np.concatenate([v[k:], np.zeros(k, np.float32)])  # 看 k 样本的未来
            self.assertEqual(_ext.measure_lookahead(f, x), k)

    def test_xcorr_lag(self):
        x = (np.random.default_rng(2).standard_normal(8000)).astype(np.float32)
        self.assertEqual(_ext.xcorr_lag(x, np.concatenate([np.zeros(37, np.float32), x])[:8000]), 37)


class Dfn3Chain(unittest.TestCase):
    """DFN3 低延迟版的重采样链路（不载入 dylib，用恒等模型）：因果、对齐后前视 = 总延迟、无滞后。"""

    def test_identity_chain_is_causal_and_aligned(self):
        import run_dfn3ll as r
        d = r.resampler_delay()
        self.assertGreater(d, 0)
        x = (np.random.default_rng(3).standard_normal(16000) * 0.1).astype(np.float32)
        ident = lambda v: r.chain(v, lambda fr: fr)[d:d + len(v)]
        y = ident(x)
        self.assertEqual(_ext.xcorr_lag(x, y), 0)
        la = _ext.measure_lookahead(ident, x)
        self.assertLessEqual(la, d, f"恒等链路前视 {la} > 延迟 {d}：链路不因果")


class Finalize(unittest.TestCase):
    def test_incomplete_outputs_are_refused(self):
        with tempfile.TemporaryDirectory() as t:
            out, builds = Path(t) / "out", Path(t) / "builds"
            cs = [{"id": "a", "scene": "cafe", "has_clean_ref": True, "additive": True}]
            _ext.write_out(out / "mix" / "a.wav", np.zeros(10), 10)
            args = type("A", (), {"limit": 0})()
            with self.assertRaises(SystemExit) as e:
                _ext.finalize("x", out, builds, _ext.CLIPS, cs, {"a"}, 0, {}, None, args)
            self.assertIn("输出不全", str(e.exception))
            self.assertFalse((out / "run.json").exists())

    def test_write_out_rejects_bad_shape_and_nan(self):
        with tempfile.TemporaryDirectory() as t:
            with self.assertRaises(ValueError):
                _ext.write_out(Path(t) / "a.wav", np.zeros(9), 10)
            y = np.zeros(10)
            y[3] = np.nan
            with self.assertRaises(ValueError):
                _ext.write_out(Path(t) / "a.wav", y, 10)


if __name__ == "__main__":
    unittest.main()
