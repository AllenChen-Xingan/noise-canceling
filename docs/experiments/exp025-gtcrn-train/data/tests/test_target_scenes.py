"""target_scene.scene 支持多个场景（2026-11-25 大总管交代，人定 D106：先补数据，再针对最弱场景加训）。

scene 可以是 null（全部噪声池等概率）· 一个场景名 · 场景名列表。列表时：每个样本先按 replay.frac 决定是否回放；
不回放就在列表里等概率抽一个，回放就在列表以外的场景里等概率抽。单个场景名和只有一项的列表抽样完全相同
（不多消耗随机数），已登记 build 的样本流因此不变。
"""
import collections
import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import loader  # noqa: E402

SCENES = ["cafe", "canteen", "street-outdoor", "transit-hub"]


def ts(scene, on=True):
    return {"on": on, "scene": scene, "frac": 0.667}


RP = {"on": True, "frac": 0.333}


def draws(scene, n=6000, rp=RP, scenes=SCENES):
    out = []
    for i in range(n):
        rng = np.random.default_rng([7, i])
        out.append(loader.pick_scene(rng, ts(scene), rp, scenes))
    return out


class PickScene(unittest.TestCase):
    def test_list_targets_equal_and_replay_third(self):
        got = draws(["transit-hub", "cafe"])
        rep = [s for s, r in got if r]
        tgt = [s for s, r in got if not r]
        self.assertAlmostEqual(len(rep) / len(got), 0.333, delta=0.02)
        self.assertEqual(set(tgt), {"transit-hub", "cafe"})
        self.assertEqual(set(rep), {"canteen", "street-outdoor"})
        c = collections.Counter(tgt)
        self.assertAlmostEqual(c["cafe"] / len(tgt), 0.5, delta=0.03)
        c = collections.Counter(rep)
        self.assertAlmostEqual(c["canteen"] / len(rep), 0.5, delta=0.04)

    def test_single_name_same_as_one_item_list(self):
        """单个名字与一项列表：场景、是否回放、之后的随机数都相同（不多抽一次）。"""
        for i in range(200):
            a, b = np.random.default_rng([3, i]), np.random.default_rng([3, i])
            self.assertEqual(loader.pick_scene(a, ts("cafe"), RP, SCENES), loader.pick_scene(b, ts(["cafe"]), RP, SCENES))
            self.assertEqual(a.random(), b.random())

    def test_single_name_keeps_old_stream(self):
        """单个名字的抽法与改动前逐样本相同：先抽是否回放，回放才再抽一个场景。"""
        for i in range(200):
            a, b = np.random.default_rng([5, i]), np.random.default_rng([5, i])
            got = loader.pick_scene(a, ts("cafe"), RP, SCENES)
            others = [s for s in SCENES if s != "cafe"]
            rep = bool(b.random() < RP["frac"])
            want = (others[int(b.integers(len(others)))] if rep else "cafe", rep)
            self.assertEqual(got, want)
            self.assertEqual(a.random(), b.random())

    def test_null_or_off_uniform_over_all(self):
        got = draws(None, n=4000)
        self.assertFalse(any(r for _, r in got))
        c = collections.Counter(s for s, _ in got)
        self.assertEqual(set(c), set(SCENES))
        rng = np.random.default_rng(0)
        s, r = loader.pick_scene(rng, ts(["cafe"], on=False), RP, SCENES)
        self.assertFalse(r)

    def test_all_scenes_listed_no_replay(self):
        got = draws(list(SCENES), n=500)
        self.assertFalse(any(r for _, r in got))

    def test_replay_off_only_targets(self):
        got = draws(["cafe", "canteen"], n=500, rp={"on": False, "frac": 0.333})
        self.assertEqual({s for s, _ in got}, {"cafe", "canteen"})
        self.assertFalse(any(r for _, r in got))

    def test_unknown_scene_refused(self):
        with self.assertRaises(SystemExit):
            loader.pick_scene(np.random.default_rng(0), ts(["cafe", "rail-air-cabin"]), RP, SCENES)


class RealRecipe(unittest.TestCase):
    """D113（人定 2026-11-26）：这一批加训 3 个目标场景；会议室等人听过 C236 再定，暂不进配方。"""

    WANT = ["rail-air-cabin", "canteen", "transit-hub"]

    def setUp(self):
        self.r = json.loads((HERE / "recipe.json").read_text())

    def test_target_scene_is_this_batch(self):
        self.assertEqual(self.r["knobs"]["target_scene"]["scene"], self.WANT)

    def test_target_scene_src_names_d113(self):
        """来自 4 个最弱场景（会议室待池），这一批只写 3 个；两条限制也写在 src 里。"""
        src = self.r["knobs"]["target_scene"]["src"]
        self.assertIn("D113", src)
        self.assertIn("meeting-room", src)
        self.assertIn("地铁车厢", src)
        self.assertIn("PCAFETER", src)

    def test_every_target_scene_has_a_pool(self):
        """通用检查：目标场景名必须都能在 noise_pools 里找到，否则训练时 loader 会停。"""
        pools = {k for k in self.r["noise_pools"] if not k.startswith("_")}
        for s in self.r["knobs"]["target_scene"]["scene"]:
            self.assertIn(s, pools)


class RecipeShape(unittest.TestCase):
    def bad(self, scene):
        r = json.loads((HERE / "recipe.json").read_text())
        r["knobs"]["target_scene"]["scene"] = scene
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "r.json"
            p.write_text(json.dumps(r, ensure_ascii=False))
            return loader.load_recipe(p)

    def test_good_shapes_load(self):
        for s in (None, "cafe", ["cafe"], ["cafe", "transit-hub"]):
            self.assertEqual(self.bad(s)["knobs"]["target_scene"]["scene"], s)

    def test_bad_shapes_refused(self):
        for s in ([], ["cafe", "cafe"], [1], "", 3, {"cafe": 1}):
            with self.assertRaises(ValueError, msg=repr(s)):
                self.bad(s)


if __name__ == "__main__":
    unittest.main()
