"""混响一致性（reverb-plan.md 第三节）与实测 RIR 读取。需要 ~/vdn-data/rir/（fetch.py --only rir）；缺就整组跳过。"""
import copy
import json
import sys
import unittest
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import compose  # noqa: E402
import labels  # noqa: E402
import manifest  # noqa: E402
import reverb_check  # noqa: E402
import rirs  # noqa: E402

HAVE = rirs.ACE.exists() and rirs.MCV.exists() and rirs.MIT.exists()


def rec_for(scene, tier, idx, device="mba-builtin"):
    a, m = compose.make_clip(scene, tier, idx, device)
    lab = labels.label(a, m)
    audio = {k: "x" for k in manifest.AUDIO_KEYS if k != "stems"} | {"stems": {s["inst"]: "x" for s in m["sounds"]}}
    return manifest.record(m, lab, audio, {"samples": {}, "wav_sha256": {}}, len(a["mix"]), 16000, "external-mic")


def first(dataset, tier="common"):
    """找一段用了某个数据集房间的片段。"""
    for scene in compose.SCENES["scenes"]:
        for i in range(compose.SCENES["counts"][tier]):
            r = rec_for(scene, tier, i)
            if r["chain"]["room"]["dataset"] == dataset:
                return r
    raise AssertionError(f"没有用 {dataset} 的片段")


@unittest.skipUnless(HAVE, "缺 ~/vdn-data/rir（fetch.py --only rir），人工跑")
class Measured(unittest.TestCase):
    def test_direct_normalised_and_split(self):
        m = rirs.ace("Meeting_Room_1", 1)
        self.assertAlmostEqual(float(m.ir[m.peak]), 1.0, places=6)
        r, src = m.at(m.dist_m, 0)
        np.testing.assert_allclose(r.full, m.ir, atol=1e-9)  # 实测距离处不调
        self.assertEqual(src["direct_gain_db"], 0.0)

    def test_closer_means_more_direct(self):
        m = rirs.ace("Office_2", 2)
        near, s1 = m.at(0.5, 0)
        far, s2 = m.at(5.0, 0)
        self.assertGreater(s1["direct_gain_db"], 0)
        self.assertLess(s2["direct_gain_db"], 0)
        np.testing.assert_allclose(near.rest, far.rest)  # 尾巴不动

    def test_t60_matches_ace_metadata_order(self):
        """ACE 元数据里 Lecture_Room_2 混响最长、Office 最短；本线量的 T60 顺序应一致。"""
        t = {r: rirs.ace(r, 1).t60 for r in ("Office_1", "Meeting_Room_1", "Lecture_Room_1", "Lecture_Room_2")}
        self.assertLess(t["Office_1"], t["Lecture_Room_1"])
        self.assertLess(t["Lecture_Room_1"], t["Lecture_Room_2"])

    def test_truncated_mit_irs_excluded(self):
        self.assertTrue(all(m.valid for m in rirs.mit_all()))
        self.assertGreater(len(rirs.mit_all()), 200)

    def test_valid_filter_rejects_bad_irs(self):
        """回归（codex 第 1 轮 P2 #4）：独立造三条 IR，过滤器必须分得开。"""
        rng = np.random.default_rng(0)
        t = np.arange(int(0.5 * 16000)) / 16000

        def ir(t60, n=None):
            x = rng.standard_normal(len(t)) * np.exp(-6.91 * t / t60) * 0.3
            x[10] = 1.0
            return x

        ok = rirs.Measured("t", "r", "ok", 0, ir(0.4), 1.0)
        trunc = rirs.Measured("t", "r", "trunc", 0, ir(4.0), 1.0)  # 4 s 的衰减被截在 0.5 s
        self.assertTrue(ok.valid)
        self.assertFalse(trunc.valid)
        noisy = np.zeros(len(t))
        noisy[10] = 1.0
        noisy[200:] = rng.standard_normal(len(t) - 200) * 0.2  # 不衰减：估计失败，直达也不占优
        self.assertFalse(rirs.Measured("t", "r", "flat", 0, noisy, 1.0).valid)  # 被「结尾没衰减」这条拦下
        sparse = np.zeros(len(t))
        sparse[10], sparse[200] = 1.0, 0.9  # 只有一个直达窗外的强反射：T60 估不出（0），直达也不占优
        m = rirs.Measured("t", "r", "sparse", 0, sparse, 1.0)
        self.assertEqual(m.t60, 0.0)
        self.assertFalse(m.valid)
        dry = np.zeros(len(t))
        dry[10] = 1.0
        self.assertTrue(rirs.Measured("t", "r", "dry", 0, dry, 1.0).valid)  # 真的没混响

    def test_every_room_option_has_candidates(self):
        for key, S in compose.SCENES["scenes"].items():
            for o in S["rooms"]:
                if "mit" in o:
                    self.assertTrue(rirs.candidates(o, compose.SCENES["bed_t60"]), f"{key} {o['mit'][:2]}")

    def test_car_mic_near_seat(self):
        mic, mouth, _ = rirs.mcv_geometry()
        d = np.linalg.norm(mouth[:, None] - mic[None], axis=2).min(1)
        self.assertTrue(np.all((d > 0.25) & (d < 0.55)))  # 和笔记本麦到嘴的 0.4–0.8 m 同量级


@unittest.skipUnless(HAVE, "缺 ~/vdn-data/rir（fetch.py --only rir），人工跑")
class Consistency(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ace = first("ace")
        cls.mit = first("mit")
        cls.car = rec_for("car-passenger", "common", 0)

    def test_real_clips_pass(self):
        for r in (self.ace, self.mit, self.car):
            self.assertEqual(reverb_check.check(r), [], r["id"])

    def test_bed_t60_table_covers_every_bed(self):
        tbl = compose.SCENES["bed_t60"]
        for key, S in compose.SCENES["scenes"].items():
            for o in S["rooms"]:
                for b in o.get("bed", []):
                    self.assertIn(b, tbl, f"{key} 的底噪 {b} 没有名义 T60")

    # ---- 负对照（必须判红）----
    def test_neg_ace_other_room(self):
        r = copy.deepcopy(self.ace)
        t = next(s for s in r["sounds"] if "rir" in s["src"]) if any("rir" in s["src"] for s in r["sounds"]) else r["targets"][0]
        t["src"]["rir"]["file"] = "Lecture_Room_2/1/Single_403a_1_RIR.wav" if not r["chain"]["room"]["room"] == "Lecture_Room_2" else "Office_1/1/x.wav"
        self.assertTrue(reverb_check.check(r))

    def test_neg_ace_bed_from_other_room(self):
        r = copy.deepcopy(self.ace)
        bed = next(s for s in r["sounds"] if s["cls"] == "bed-ace")
        bed["src"]["cfg"] = 3 - bed["src"]["cfg"]
        self.assertTrue(reverb_check.check(r))

    def test_neg_car_bed_with_long_rir(self):
        """车内底噪配 1.0 s 的 RIR 必须判红（短混响类：RIR T60 ≤ 0.3 s）。"""
        ok, _ = rirs.bed_ok(compose.SCENES["bed_t60"], "bed-bus", 1.0)
        self.assertFalse(ok)
        r = copy.deepcopy(self.mit)
        r["chain"]["room"]["bed_cls"] = "bed-street"
        bed = next(s for s in r["sounds"] if s["group"] == "bed")
        bed["cls"] = "bed-street"
        for t in r["targets"]:
            t["src"]["rir"]["t60_s"] = 1.0
        self.assertTrue(reverb_check.check(r))

    def test_neg_car_other_mic(self):
        r = copy.deepcopy(self.car)
        r["targets"][0]["src"]["rir"]["ch"] = (r["chain"]["room"]["mic"] + 1) % 14
        self.assertTrue(reverb_check.check(r))

    def test_neg_missing_rir_and_missing_bed_value(self):
        r = copy.deepcopy(self.mit)
        r["targets"][0]["src"].pop("rir")
        self.assertTrue(reverb_check.check(r))
        tbl = dict(compose.SCENES["bed_t60"])
        tbl.pop(self.mit["chain"]["room"]["bed_cls"])
        self.assertTrue(reverb_check.check(self.mit, tbl))

    def test_neg_mit_mixed_irs(self):
        r = copy.deepcopy(self.mit)
        r["targets"][0]["src"]["rir"]["file"] = "h999_Other_1txts.wav"
        self.assertTrue(reverb_check.check(r))


if __name__ == "__main__":
    unittest.main()
