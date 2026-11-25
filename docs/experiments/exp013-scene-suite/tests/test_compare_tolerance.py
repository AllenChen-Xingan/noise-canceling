"""compare_builds.py 的容差模式（跨平台可复现的验收，2026-11-25 大总管定）：
抽几段两边都有的音频，逐轨比：样本数相等；每样本差 ≤ TOL_ULP（4）× 整轨峰值处的 32 位浮点最小单位（第 3 版）；
整轨差信号比原信号低 ≥ TOL_DIFF_DB（120 dB）；manifest 素材 / 代码 / 参数哈希逐位相等。
（原阈值「逐样本绝对差 ≤ 1e-9」比 wav 的 32 位浮点精度细 60 倍，看到结果后改，见 crosscheck-20261125.md。）

在临时目录造两份迷你 build（2 段 × 3 条轨 + 1 条分轨，float32 wav）：
- 只差浮点最后一位 · 整段 2e-9 的相对增益差 · 接近 0 的样本差 1e-15 → 一致
- 一条轨有样本差 5 个最小单位 → 不一致，并点名哪段哪轨；差信号只低 100 dB → 不一致，原因里有 dB
- 样本数不同 · 素材索引不同 · 抽的段只在一边 → 不一致
"""
import copy
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
from scipy.io import wavfile

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import compare_builds as cb  # noqa: E402

SR = 16000
CLIPS = ["a-c00-ext", "a-c00-mba"]
TRACKS = ["mix", "owner", "owner_early"]
MAN = {"material_index": {"sha256": "a" * 64, "files": 1994}, "code_sha256": {"x.py": "c"},
       "params_sha256": {"p": "d"}, "versions": {"numpy": "2.5.4"}, "base_seed": 1, "clips": 2}


def _rec(cid, bid):
    rel = lambda k: f"_tmp/exp013/{bid}/{cid}/{k}.wav"  # noqa: E731
    return {"id": cid, "audio": {**{k: rel(k) for k in TRACKS}, "stems": {"bed0": rel("stem-bed0")}}}


class Tolerance(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        rng = np.random.default_rng(3)
        # 幅度 0.01：float32 在这里的最小单位约 9e-10，2e-9 的差写得进 wav
        self.sig = {(c, k): (0.01 * rng.standard_normal(SR)).astype("<f4") for c in CLIPS for k in TRACKS + ["stem-bed0"]}

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def _build(self, name, sig=None, clips=CLIPS, man=MAN):
        sig = self.sig if sig is None else sig
        root, bdir = self.tmp / f"{name}-audio", self.tmp / name
        bdir.mkdir()
        recs = [_rec(c, name) for c in clips]
        for r in recs:
            for k, p in [(k, r["audio"][k]) for k in TRACKS] + [("stem-bed0", r["audio"]["stems"]["bed0"])]:
                (root / p).parent.mkdir(parents=True, exist_ok=True)
                wavfile.write(str(root / p), SR, sig[(r["id"], k)])
        (bdir / "clips.jsonl").write_text("".join(json.dumps(r) + "\n" for r in recs))
        (bdir / "manifest.json").write_text(json.dumps(man))
        return bdir, root

    def _check(self, **kw):
        self.n = getattr(self, "n", 0) + 1  # 同一测试里多次比较：每次新目录
        return cb.tolerance_check(*self._build(f"A{self.n}"), *self._build(f"B{self.n}", **kw), CLIPS)

    def test_thresholds(self):
        self.assertEqual((cb.TOL_ULP, cb.TOL_DIFF_DB), (4, 120))
        self.assertFalse(hasattr(cb, "TOL_ABS"), "旧阈值 1e-9 已删")

    def test_last_bit_differences_are_consistent(self):
        sig = copy.deepcopy(self.sig)
        for x in sig.values():
            small = np.abs(x) < 1e-3  # 这些样本的 float32 最小单位 ≤ 1.2e-10
            x[small] = np.nextafter(x[small], np.float32(1))
        self.assertEqual(self._check(sig=sig), [])

    def test_whole_track_gain_2e9_is_consistent(self):
        """跨平台实测的情形（meeting-room-s02-mba）：整段增益差约 2e-9，存成 32 位浮点后响的样本差整数个最小单位。"""
        sig = copy.deepcopy(self.sig)
        for k in sig:
            sig[k] = (sig[k].astype(np.float64) * (1 + 2e-9)).astype("<f4")
        self.assertEqual(self._check(sig=sig), [])

    def test_tiny_difference_on_near_zero_samples_is_consistent(self):
        """第 2 版的失误：接近 0 的样本最小单位极小，1e-15 的差折成几千万个最小单位。第 3 版按整轨峰值算，必须判一致。"""
        sig = copy.deepcopy(self.sig)
        x = sig[("a-c00-ext", "owner")]
        x[:50] = np.float32(1e-30)
        y = x.copy()
        y[:50] = np.float32(1e-30 + 1e-15)
        sig[("a-c00-ext", "owner")] = y
        self.sig[("a-c00-ext", "owner")] = x
        self.assertEqual(self._check(sig=sig), [])

    def test_five_ulp_is_inconsistent(self):
        sig = copy.deepcopy(self.sig)
        x = sig[("a-c00-mba", "owner_early")]
        i = int(np.argmax(np.abs(x)))
        for _ in range(5):
            x[i] = np.nextafter(x[i], np.float32(np.inf))
        errs = self._check(sig=sig)
        self.assertEqual(len(errs), 1, errs)
        self.assertIn("a-c00-mba", errs[0])
        self.assertIn("owner_early", errs[0])
        self.assertIn("最小单位", errs[0])

    def test_diff_only_100db_down_is_inconsistent(self):
        sig = copy.deepcopy(self.sig)
        x = sig[("a-c00-ext", "owner")].astype(np.float64)
        noise = np.random.default_rng(9).standard_normal(len(x))
        noise *= np.sqrt(np.mean(x**2)) * 10 ** (-100 / 20) / np.sqrt(np.mean(noise**2))
        sig[("a-c00-ext", "owner")] = (x + noise).astype("<f4")
        errs = self._check(sig=sig)
        self.assertTrue(any("a-c00-ext" in e and "dB" in e for e in errs), errs)

    def test_sample_count_material_and_missing_clip(self):
        sig = copy.deepcopy(self.sig)
        sig[("a-c00-ext", "mix")] = sig[("a-c00-ext", "mix")][:-1]
        self.assertTrue(any("样本数" in e for e in self._check(sig=sig)))
        man = json.loads(json.dumps(MAN))
        man["material_index"]["sha256"] = "b" * 64
        self.assertTrue(any("material_index" in e for e in self._check(man=man)))
        self.assertTrue(any("a-c00-mba" in e and "B" in e for e in self._check(clips=["a-c00-ext"])))


if __name__ == "__main__":
    unittest.main()
