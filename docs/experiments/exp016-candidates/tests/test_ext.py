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
        self.assertEqual(list(got), ["mix", "bg", "trunc", "clean"])
        mix = _ext.read16(self.root / "c0-mix.wav")
        bg = _ext.read16(self.root / "c0-noise.wav") + _ext.read16(self.root / "c0-interf.wav")
        np.testing.assert_array_equal(got["mix"], mix)
        np.testing.assert_array_equal(got["bg"], bg.astype(np.float32))
        self.assertTrue(np.all(got["trunc"][800:] == 0) and np.array_equal(got["trunc"][:800], mix[:800]))
        np.testing.assert_array_equal(got["clean"], _ext.read16(self.root / "c0-owner.wav"))  # = Rust clip.clean（owner 轨）
        # 非 additive 不给 bg；不在 picks 不给 trunc；clean 每段都给
        c2 = clip("c1", "cafe", self.root, additive=False)
        self.assertEqual([k for k, _ in _ext.inputs_of(c2, self.root, set())], ["mix", "clean"])

    def test_hash_mismatch_is_rejected(self):
        c = clip("c0", "cafe", self.root)
        c["hash"]["wav_sha256"]["mix"] = "0" * 64
        with self.assertRaises(ValueError):
            _ext.inputs_of(c, self.root, set())


class RustWavParity(unittest.TestCase):
    """Python 读 wav 与 Rust `wav::read` 同一归一：整型按 v / 2^(位深−1)，浮点原样；多声道取第一声道（codex 016 脚本第 1 轮 P2 #6）。"""

    def test_rust_formula_is_what_we_mirror(self):
        src = (_ext.ROOT / "src/eval/src/wav.rs").read_text()
        self.assertIn("1.0 / (1u64 << (spec.bits_per_sample - 1)) as f32", src)
        self.assertIn(".step_by(ch)", src)

    def test_pcm16_and_float_and_stereo(self):
        with tempfile.TemporaryDirectory() as t:
            ints = np.array([-32768, -1, 0, 1, 12345, 32767], np.int16)
            p = Path(t) / "i.wav"
            sf.write(str(p), ints, _ext.SR, subtype="PCM_16")
            np.testing.assert_array_equal(_ext.read16(p), ints.astype(np.float32) * np.float32(1 / 32768))
            f = np.array([0.1, -0.7, 1e-8], np.float32)
            sf.write(str(p), f, _ext.SR, subtype="FLOAT")
            np.testing.assert_array_equal(_ext.read16(p), f)
            st = np.stack([f, -f], 1)
            sf.write(str(p), st, _ext.SR, subtype="FLOAT")
            np.testing.assert_array_equal(_ext.read16(p), f)


class Fingerprint(unittest.TestCase):
    def test_mismatch_is_refused(self):
        with tempfile.TemporaryDirectory() as t:
            d = Path(t)
            fp = _ext.fingerprint({"w": "a"}, lambda: {"thr": 1})
            self.assertIn("docs/experiments/exp016-candidates/_ext.py", fp["code_sha256"])
            _ext.pin_fingerprint(d, fp)
            _ext.pin_fingerprint(d, fp)  # 一样：通过
            with self.assertRaises(SystemExit) as e:
                _ext.pin_fingerprint(d, _ext.fingerprint({"w": "b"}, lambda: {"thr": 1}))
            self.assertIn("model", str(e.exception))

    def test_legacy_outputs_without_fingerprint_are_refused(self):
        with tempfile.TemporaryDirectory() as t:
            out, sd = Path(t) / "out", Path(t) / "sd"
            _ext.write_out(out / "mix" / "a.wav", np.zeros(10), 10)
            with self.assertRaises(SystemExit) as e:
                _ext.pin_fingerprint(sd, {"x": 1}, out)
            self.assertIn("没有运行指纹", str(e.exception))
            self.assertEqual(list(sd.glob("*")), [], "不许留临时文件或指纹")

    def test_bad_existing_output_is_refused(self):
        with tempfile.TemporaryDirectory() as t:
            p = Path(t) / "a.wav"
            sf.write(str(p), np.zeros(9, np.float32), _ext.SR, subtype="FLOAT")
            with self.assertRaises(SystemExit):
                _ext.check_existing(p, 10)
            sf.write(str(p), np.zeros(10, np.float32), _ext.SR, subtype="FLOAT")
            _ext.check_existing(p, 10)


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

    def test_identity_chain_is_aligned_and_lookahead_is_declared(self):
        import run_dfn3ll as r
        la = r.resampler_lookahead()
        self.assertGreater(la, 0, "soxr 流式应有滤波器前视")
        self.assertEqual(r.total_delay(), r.SHIFT + la)
        x = (np.random.default_rng(3).standard_normal(16000) * 0.1).astype(np.float32)
        self.assertEqual(_ext.xcorr_lag(x, r.identity_chain(x)), 0)
        # 换一段信号再量：入口截半自检用的容差 1e-4 下，前视不超过声明值
        self.assertLessEqual(_ext.measure_lookahead(r.identity_chain, x), la)

    def test_scheduling_delay_covers_lookahead_and_frame_wait(self):
        import run_dfn3ll as r
        d = r.scheduling_delay()
        # 至少 = 算法前视（声明延迟）+ 攒满一帧（160 @16 k）减 1。soxr 流式内部攒大块，实测 HQ 约 1393（87 ms），
        # 只记录、不设上限：调度延迟取决于重采样器实现，本实验不比（README「延迟口径」）
        self.assertGreaterEqual(d, r.total_delay() + 160 - 1)

    def test_lib_constants(self):
        import run_dfn3ll as r
        self.assertEqual(r.SHIFT, 160)

    def test_native_rate_matches_rust_io_rate(self):
        """DFN3 写进 run.json 的 native_rate = Rust 正本 IO_RATE；输入 16 kHz = MODEL_RATE（第二处表示的比对）。"""
        import run_dfn3ll as r
        rate = (_ext.ROOT / "src/types/src/rate.rs").read_text()
        self.assertIn(f"pub const IO_RATE: u32 = {r.SR_MODEL // 1000}_{r.SR_MODEL % 1000:03d};", rate)
        self.assertIn(f"pub const MODEL_RATE: u32 = {r.SR_IN // 1000}_{r.SR_IN % 1000:03d};", rate)


class Finalize(unittest.TestCase):
    def test_incomplete_outputs_are_refused(self):
        with tempfile.TemporaryDirectory() as t:
            out, builds = Path(t) / "out", Path(t) / "builds"
            cs = [{"id": "a", "scene": "cafe", "has_clean_ref": True, "additive": True, "dur_samples": 10}]
            _ext.write_out(out / "mix" / "a.wav", np.zeros(10), 10)
            args = type("A", (), {"limit": 0})()
            with self.assertRaises(SystemExit) as e:
                _ext.finalize("x", out, builds, _ext.CLIPS, cs, {"a"}, 0, {}, None, args)
            self.assertIn("输出不全", str(e.exception))
            self.assertFalse((out / "run.json").exists())

    def test_finalize_writes_run_json_and_rejects_stray_dirs(self):
        with tempfile.TemporaryDirectory() as t:
            out, builds = Path(t) / "out", Path(t) / "builds"
            cs = [{"id": "a", "scene": "cafe", "has_clean_ref": True, "additive": True, "dur_samples": 10}]
            for k in _ext.KINDS:
                _ext.write_out(out / k / "a.wav", np.zeros(10), 10)
            self.assertEqual(sorted(p.name for p in out.glob("*/*")), ["a.wav"] * len(_ext.KINDS), "不许留 .part")
            args = type("A", (), {"limit": 0})()
            (out / "shards").mkdir()
            with self.assertRaises(SystemExit) as e:
                _ext.finalize("x", out, builds, _ext.CLIPS, cs, {"a"}, 0, {}, None, args)
            self.assertIn("入口不认", str(e.exception))
            (out / "shards").rmdir()
            # 坏 wav：收尾逐个校验，拒绝，且不写 run.json
            bad = out / "bg" / "a.wav"
            sf.write(str(bad), np.zeros(9, np.float32), _ext.SR, subtype="FLOAT")
            with self.assertRaises(SystemExit):
                _ext.finalize("x", out, builds, _ext.CLIPS, cs, {"a"}, 7, {"m": 1}, None, args)
            self.assertFalse((out / "run.json").exists())
            import shutil
            shutil.rmtree(builds, ignore_errors=True)
            _ext.write_out(bad, np.zeros(10), 10)
            _ext.finalize("x", out, builds, _ext.CLIPS, cs, {"a"}, 7, {"m": 1}, None, args)
            rj = json.loads((out / "run.json").read_text())
            self.assertNotIn("native_rate", rj)
            self.assertEqual((rj["schema"], rj["latency_samples"], rj["inputs"]), (_ext.SCHEMA, 7, list(_ext.KINDS)))
            self.assertEqual(len((builds / "hashes.txt").read_text().splitlines()), 3)
            with self.assertRaises(SystemExit):  # 不覆盖
                _ext.finalize("x", out, builds, _ext.CLIPS, cs, {"a"}, 7, {}, None, args)

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
