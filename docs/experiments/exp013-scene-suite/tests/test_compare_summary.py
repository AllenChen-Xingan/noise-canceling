"""compare_builds.py 的「逐轨摘要」模式（跨平台按容差核对，2026-11-25 大总管定）。

在临时目录造两份迷你 build（2 段、各 3 条轨 + 1 条分轨），音频按 float32 wav 写：
- 位翻转级差异（个别样本差一个 float32 最小单位）+ 参数最后一位不同 → 一致
- 一条轨整体压低 1e-6（相对）→ 不一致
- 样本数不同 · 参数相对差 1e-9 · 素材索引不同 · 少一段 → 不一致
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
TRACKS = ["mix", "owner", "owner_early"]


def _rec(cid, bid):
    rel = lambda k: f"_tmp/exp013/{bid}/{cid}/{k}.wav"  # noqa: E731
    return {
        "id": cid,
        "audio": {**{k: rel(k) for k in TRACKS}, "stems": {"bed0": rel("stem-bed0")}},
        "hash": {"samples": {}, "wav_sha256": {}},
        "chain": {"strength": 0.123456789012345, "flat": False},
        "nominal": {"snr_db": 10.0},
    }


MAN = {"material_index": {"sha256": "a" * 64, "files": 1994}, "code_sha256": {"x.py": "c"}, "params_sha256": {"p": "d"},
       "versions": {"numpy": "2.5.4"}, "base_seed": 1, "clips": 2}


class Summary(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        rng = np.random.default_rng(3)
        self.sig = {(c, k): (0.1 * rng.standard_normal(SR)).astype("<f4") for c in ("a-c00-ext", "a-c00-mba")
                    for k in TRACKS + ["stem-bed0"]}

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def _build(self, name, sig=None, recs=None, man=None):
        """写一份 build：返回 (build 目录, 音频根)。"""
        sig = self.sig if sig is None else sig
        root, bdir = self.tmp / f"{name}-audio", self.tmp / name
        bdir.mkdir()
        rs = recs if recs is not None else [_rec(c, name) for c in ("a-c00-ext", "a-c00-mba")]
        for r in rs:
            for k, p in [(k, r["audio"][k]) for k in TRACKS] + [("stem-bed0", r["audio"]["stems"]["bed0"])]:
                f = root / p
                f.parent.mkdir(parents=True, exist_ok=True)
                wavfile.write(str(f), SR, sig[(r["id"], k)])
        (bdir / "clips.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rs))
        (bdir / "manifest.json").write_text(json.dumps(man or MAN))
        return bdir, root

    def _cmp(self, b, **kw):
        a = cb.summarize(*self._build("A"))
        return cb.diff_summaries(a, cb.summarize(*self._build("B", **kw)))

    def test_bitflip_and_last_digit_params_are_consistent(self):
        sig = copy.deepcopy(self.sig)
        for (c, k), x in sig.items():
            x[::997] = np.nextafter(x[::997], np.float32(1))  # 每 997 个样本差一个 float32 最小单位
        recs = [_rec(c, "B") for c in ("a-c00-ext", "a-c00-mba")]
        recs[0]["chain"]["strength"] = 0.123456789012345 * (1 + 4e-16)
        self.assertEqual(self._cmp("B", sig=sig, recs=recs), [])

    def test_real_attenuation_1e6_is_inconsistent(self):
        sig = copy.deepcopy(self.sig)
        sig[("a-c00-mba", "owner")] = (sig[("a-c00-mba", "owner")] * (1 - 1e-6)).astype("<f4")
        errs = self._cmp("B", sig=sig)
        self.assertTrue(errs)
        self.assertTrue(all("a-c00-mba" in e and "owner" in e for e in errs), errs)

    def test_sample_count_difference_is_inconsistent(self):
        sig = copy.deepcopy(self.sig)
        sig[("a-c00-ext", "mix")] = sig[("a-c00-ext", "mix")][:-1]
        self.assertTrue(any("样本数" in e for e in self._cmp("B", sig=sig)))

    def test_param_relative_diff_1e9_is_inconsistent(self):
        recs = [_rec(c, "B") for c in ("a-c00-ext", "a-c00-mba")]
        recs[1]["chain"]["strength"] = 0.123456789012345 * (1 + 1e-9)
        self.assertTrue(any("chain" in e for e in self._cmp("B", recs=recs)))

    def test_material_index_and_missing_clip_are_inconsistent(self):
        man = json.loads(json.dumps(MAN))
        man["material_index"]["sha256"] = "b" * 64
        recs = [_rec("a-c00-ext", "B")]
        errs = self._cmp("B", recs=recs, man=man)
        self.assertTrue(any("material_index" in e for e in errs))
        self.assertTrue(any("a-c00-mba" in e and "只在" in e for e in errs))

    def test_summary_round_trips_through_json(self):
        s = cb.summarize(*self._build("A"))
        self.assertEqual(cb.diff_summaries(s, json.loads(json.dumps(s))), [])
        self.assertEqual(set(s["clips"]["a-c00-ext"]["tracks"]), set(TRACKS) | {"stems/bed0"})


if __name__ == "__main__":
    unittest.main()
