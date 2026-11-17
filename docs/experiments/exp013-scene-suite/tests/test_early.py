"""早期回声参考轨（owner_early.wav，D99 第 2 条）的自检。

- 截断长度的正本是 rooms.EARLY_REVERB_MS：early_rir 默认用它，直达部分原样、截点之后全零、不改入参
- 加这一路不抽随机数：旧 build 的各轨逐样本可重现（对 build 20261103230756-17f5 清单里的样本哈希）
- 旧 build 的仓内文件逐字节不变（对 tests/old_builds.sha256，生成于加早期参考之前）
- 早期参考与主参考同电平系数：差值只在直达后 EARLY_REVERB_MS 之后的尾巴上
需要本机素材时（~/vdn-data/）缺就跳过并写明原因。
"""
import hashlib
import json
import sys
import unittest
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import compose  # noqa: E402
import rirs  # noqa: E402
import rooms  # noqa: E402
from dsp import SR, sample_hash  # noqa: E402

OLD = "20261103230756-17f5"
HAVE = rirs.ACE.exists() and rirs.MCV.exists() and rirs.MIT.exists() and Path.home().joinpath("vdn-data/librispeech").exists()


def _old_recs():
    out = {}
    for line in (HERE / "builds" / OLD / "clips.jsonl").read_text().splitlines():
        r = json.loads(line)
        out[r["id"]] = r
    return out


class EarlyRir(unittest.TestCase):
    def setUp(self):
        rng = np.random.default_rng(7)
        self.r = rooms.make_rir(rng, (6.0, 4.0, 3.0), 0.6, 1.0, 30.0)

    def test_constant_is_the_default(self):
        self.assertEqual(rooms.EARLY_REVERB_MS, 100)
        a = rooms.early_rir(self.r)
        b = rooms.early_rir(self.r, ms=rooms.EARLY_REVERB_MS)
        np.testing.assert_array_equal(a.rest, b.rest)

    def test_cut_exact_and_direct_kept(self):
        e = rooms.early_rir(self.r)
        cut = self.r.delay + int(round(rooms.EARLY_REVERB_MS * 1e-3 * SR))
        np.testing.assert_array_equal(e.direct, self.r.direct)
        np.testing.assert_array_equal(e.rest[:cut], self.r.rest[:cut])
        self.assertFalse(np.any(e.rest[cut:]))
        self.assertTrue(np.any(self.r.rest[cut:]), "测试房间的尾巴要长过截点，否则测不出截断")
        self.assertEqual((e.t60, e.dist_m, e.azimuth_deg, e.delay), (self.r.t60, self.r.dist_m, self.r.azimuth_deg, self.r.delay))

    def test_input_not_mutated(self):
        before = self.r.rest.copy()
        rooms.early_rir(self.r)
        np.testing.assert_array_equal(self.r.rest, before)

    def test_make_rir_ignores_constant(self):
        """make_rir 的直达 / 其余拆分不读 EARLY_REVERB_MS（旧 build 的行为不随它变）。"""
        old = rooms.EARLY_REVERB_MS
        try:
            rooms.EARLY_REVERB_MS = 37
            r2 = rooms.make_rir(np.random.default_rng(7), (6.0, 4.0, 3.0), 0.6, 1.0, 30.0)
        finally:
            rooms.EARLY_REVERB_MS = old
        np.testing.assert_array_equal(r2.direct, self.r.direct)
        np.testing.assert_array_equal(r2.rest, self.r.rest)


class OldBuildsUntouched(unittest.TestCase):
    def test_repo_files_byte_identical(self):
        want = {}
        for line in (HERE / "tests/old_builds.sha256").read_text().splitlines():
            h, p = line.split(None, 1)
            want[p] = h
        self.assertTrue(want)
        for p, h in want.items():
            got = hashlib.sha256((HERE / "builds" / p).read_bytes()).hexdigest()
            self.assertEqual(got, h, p)


@unittest.skipUnless(HAVE, "缺 ~/vdn-data 素材（fetch.py），人工跑")
class Reproduce(unittest.TestCase):
    # 只留一人（有房间）· 全留（多人各自早期版本）· 户外近零混响 · 无设备链对照
    CLIPS = ["cafe-c00-mba", "meeting-room-s01-mba", "street-outdoor-c00-ext"]

    @classmethod
    def setUpClass(cls):
        recs = _old_recs()
        cls.out = {}
        for cid in cls.CLIPS:
            r = recs[cid]
            idx = int(cid.split("-")[-2][1:])
            cls.out[cid] = (r, *compose.make_clip(r["scene"], r["tier"], idx, r["device"]))

    def test_old_tracks_sample_identical(self):
        for cid, (r, a, _) in self.out.items():
            h = r["hash"]["samples"]
            for k in ("mix", "owner", "owner_dry", "floor", "noise", "interf"):
                self.assertEqual(sample_hash(a[k]), h[k], f"{cid} {k}")
            for k, v in a["stems"].items():
                self.assertEqual(sample_hash(v), h["stems"][k], f"{cid} stem {k}")
            self.assertNotIn("owner_early", h, "旧 build 不应有早期参考")

    def test_early_differs_only_after_cut(self):
        """单人段：早期参考能量不超过主参考；两者之差（晚期尾巴）非零，且与早期参考几乎不相关。"""
        r, a, m = self.out["cafe-c00-mba"]
        e, o = np.asarray(a["owner_early"]), np.asarray(a["owner"])
        self.assertEqual(e.shape, o.shape)
        self.assertLessEqual(float(np.sum(e**2)), float(np.sum(o**2)) * 1.001)
        d = o - e
        self.assertGreater(float(np.sum(d**2)), 0.0)
        # 早期参考 ⊂ 主参考：两者的残差与早期参考几乎不相关（只差晚期尾巴）
        rho = float(np.dot(d, e) / np.sqrt(np.sum(d**2) * np.sum(e**2)))
        self.assertLess(abs(rho), 0.2, rho)

    def test_preserve_all_sums_every_target(self):
        r, a, m = self.out["meeting-room-s01-mba"]
        self.assertGreater(len(m["targets"]), 1)
        s = sum(t["x_early"] for t in m["targets"])
        np.testing.assert_array_equal(np.asarray(a["owner_early"]), s)

    def test_no_rng_consumed(self):
        """point_early 不碰两路随机数。"""
        r = _old_recs()["cafe-c00-mba"]
        S = compose.SCENES["scenes"][r["scene"]]
        rm, rd = np.random.default_rng([1, 0]), np.random.default_rng([1, 1])
        sc = compose.Scene(rm, rd, S, r["tier"], r["device"])
        rir, _ = sc.room.rir(rm, "owner", 0.6, 0.0)
        s0, s1 = rm.bit_generator.state, rd.bit_generator.state
        sc.point_early(np.ones(SR), rir)
        self.assertEqual(rm.bit_generator.state, s0)
        self.assertEqual(rd.bit_generator.state, s1)


if __name__ == "__main__":
    unittest.main()
