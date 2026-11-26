"""同一批录音被两个场景共用时，抽样只算一次权重（2026-11-25，D106 的已知偏差）。

cowork 与 open-office 的噪声池都是 `/CrowdedIndoor/`，成员完全相同。以前 target_scene 为 null 时按场景等概率抽，
这批录音的权重是其他来源的两倍；回放的「其他场景」和目标列表里点名两个也都一样。
现在：成员完全相同的场景并成一组（loader.scene_groups，按 (来源, key) 排序后的元组比），组名取字母序第一个；
三处抽样都只在组名上抽，返回的也是组名。没有重复组时随机数与结果和改动前逐样本相同。
"""
import collections
import json
import sys
import unittest
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import loader  # noqa: E402

RECIPE = json.loads((HERE / "recipe.json").read_text())
# cowork 与 open-office 共用同一批拥挤室内录音；cafe · street-outdoor 各自一份
SCENES = ["cafe", "cowork", "open-office", "street-outdoor"]
SAME = {"cowork": "cowork", "open-office": "cowork"}


def ts(scene, on=True):
    return {"on": on, "scene": scene, "frac": 0.667}


RP = {"on": True, "frac": 0.333}


def draws(scene, same, n=6000, scenes=SCENES, rp=RP):
    out = []
    for i in range(n):
        rng = np.random.default_rng([7, i])
        out.append(loader.pick_scene(rng, ts(scene), rp, scenes, same))
    return out


def old_pick_scene(rng, ts, rp, scenes):
    """改动前的抽法（正本 = 2026-11-25 之前的 loader.pick_scene）：没有重复组时结果必须逐样本相同。"""
    scenes = sorted(scenes)
    if not (ts["on"] and ts["scene"]):
        return scenes[int(rng.integers(len(scenes)))], False
    targets = [ts["scene"]] if isinstance(ts["scene"], str) else list(ts["scene"])
    miss = [t for t in targets if t not in scenes]
    if miss:
        raise SystemExit(f"目标场景 {miss} 没有噪声池")
    others = [s for s in scenes if s not in targets]
    replay = bool(rp["on"] and others and rng.random() < rp["frac"])
    if replay:
        return others[int(rng.integers(len(others)))], True
    return (targets[int(rng.integers(len(targets)))] if len(targets) > 1 else targets[0]), False


class DedupDraws(unittest.TestCase):
    def test_null_scene_counts_group_once(self):
        """target_scene 为 null：三个不同的池各占 1/3；open-office 一次也不出现（报出的是组名）。"""
        got = draws(None, SAME)
        c = collections.Counter(s for s, _ in got)
        self.assertEqual(set(c), {"cafe", "cowork", "street-outdoor"})
        for s in ("cafe", "cowork", "street-outdoor"):
            self.assertAlmostEqual(c[s] / len(got), 1 / 3, delta=0.03)

    def test_replay_others_counted_once(self):
        """目标 cafe、回放的「其他场景」里有重复的一对：两个不同的池各占回放的一半。"""
        got = draws("cafe", SAME)
        rep = [s for s, r in got if r]
        self.assertAlmostEqual(len(rep) / len(got), 0.333, delta=0.02)
        c = collections.Counter(rep)
        self.assertEqual(set(c), {"cowork", "street-outdoor"})
        for s in c:
            self.assertAlmostEqual(c[s] / len(rep), 0.5, delta=0.04)

    def test_target_list_of_group_equals_representative(self):
        """目标列表点名同一组的两个场景，和只点组名完全一样（结果与之后的随机数都不变）。"""
        for i in range(200):
            a, b = np.random.default_rng([11, i]), np.random.default_rng([11, i])
            self.assertEqual(loader.pick_scene(a, ts(["cowork", "open-office"]), RP, SCENES, SAME),
                             loader.pick_scene(b, ts("cowork"), RP, SCENES, SAME))
            self.assertEqual(a.random(), b.random())

    def test_unknown_scene_still_refused(self):
        with self.assertRaises(SystemExit):
            loader.pick_scene(np.random.default_rng(0), ts(["cowork", "rail-air-cabin"]), RP, SCENES, SAME)


class NoDuplicates(unittest.TestCase):
    """没有重复组时：三处抽样（null · 单个目标 · 目标列表）与改动前逐样本相同，随机数用法也一样。"""

    SCENES = ["cafe", "canteen", "transit-hub", "street-outdoor"]

    def test_stream_unchanged(self):
        for i in range(200):
            for scene in (None, "cafe", ["transit-hub", "cafe"]):
                for same in (None, {s: s for s in self.SCENES}):
                    a, b = np.random.default_rng([13, i]), np.random.default_rng([13, i])
                    got = loader.pick_scene(a, ts(scene), RP, self.SCENES, same)
                    self.assertEqual(got, old_pick_scene(b, ts(scene), RP, self.SCENES), msg=f"{scene!r} {same!r}")
                    self.assertEqual(a.random(), b.random(), msg=f"{scene!r} 抽完之后的随机数流变了")


def cpath(cls, user="user0001", k=0):
    return f"train-raw/cochlscene/CochlScene/Train/{cls}/{cls}_{user}_14983413_{k:03d}.wav"


def member(path):
    return {"key": path, "shard": "s-00000.tar", "offset": 0, "bytes": 1024, "src_path": path, "unit": path}


def fake_idx():
    """最小分片索引（只填 members，不读音频）：配方引用的每个来源都要在，路径要能匹配 recipe.json 的正则。"""
    by_src = {
        "librispeech-train-clean-100": ["train-raw/librispeech/LibriSpeech/train-clean-100/19/1/19-1-0000.flac"],
        "openslr28-rir": ["train-raw/openslr28/RIRS_NOISES/simulated_rirs/smallroom/Room001/Room001-00001.wav"],
        "qut-noise": ["train-raw/qut-noise/QUT-NOISE/CAFE-CAFE-1.wav"],
        "tum-wind": ["train-raw/tum-wind/wind/mobilephone_wind_noise/rec0/000_001.wav"],
        "demand-train": ["vdn-data/demand/OHALLWAY/ch01.wav"],  # 配方里没有正则匹配它
        "cochlscene": [cpath("Subway"), cpath("SubwayStation"), cpath("CrowdedIndoor", k=0), cpath("CrowdedIndoor", k=1)],
    }
    src = {}
    for name, paths in by_src.items():
        kind = "speech" if name.startswith("librispeech") else "rir" if name.endswith("-rir") else "noise"
        src[name] = {"kind": kind, "train": {"members": [member(p) for p in paths], "shards": {}}}
    return {"sources": src}


class Groups(unittest.TestCase):
    """分组只从实际噪声池成员算出，不许手写场景名清单。"""

    def pools(self, split_="train"):
        pools, _ = loader.pool_members(fake_idx(), RECIPE, split_)
        return pools

    def test_only_identical_pools_grouped(self):
        g = loader.scene_groups(self.pools())
        multi = {k: v for k, v in g.items() if len(v) > 1}
        self.assertEqual(multi, {"cowork": ["cowork", "open-office"]})
        self.assertEqual(len(g), len(set(k for v in g.values() for k in v)), "每个场景必须恰好出现在一组里")
        for rep, names in g.items():
            self.assertEqual(rep, names[0])  # 组名 = 字母序第一个

    def test_pools_keep_it(self):
        p = loader.Pools(HERE, RECIPE, fake_idx(), "train")
        self.assertEqual({k: v for k, v in p.scene_groups.items() if len(v) > 1},
                         {"cowork": ["cowork", "open-office"]})

    def test_empty_split_has_no_group(self):
        got = loader.noise_scene_index(fake_idx(), RECIPE)
        self.assertEqual(got["scene_groups"]["train"], {"cowork": ["cowork", "open-office"]})
        self.assertEqual(got["scene_groups"]["dev"], {})


if __name__ == "__main__":
    unittest.main()
