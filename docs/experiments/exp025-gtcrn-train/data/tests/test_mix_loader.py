"""mix.py + shard.py + loader.py：旋钮逐个生效、目标只留早期混响（调正本 rooms.early_rir）、端到端可复现、续训批次一致。
夹具在临时目录现造（几秒长的合成波形，总共 < 1 MB），不用真数据集。"""
import copy
import json
import shutil
import sys
import tempfile
import unittest
from unittest import mock
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import loader  # noqa: E402
import mix  # noqa: E402
import shard  # noqa: E402
import split  # noqa: E402

SR = 16000
RECIPE = json.loads((HERE / "recipe.json").read_text())


def knobs(**over):
    k = copy.deepcopy(RECIPE["knobs"])
    for name, v in over.items():
        k[name].update(v)
    return k


def all_off():
    return knobs(**{n: {"on": False} for n in RECIPE["knobs"]})


def speech(n=SR * 3, seed=0):
    rng = np.random.default_rng(seed)
    t = np.arange(n) / SR
    env = (np.sin(2 * np.pi * 2 * t) > 0).astype(float)  # 0.25 s 说 / 0.25 s 停
    return 0.1 * env * np.sin(2 * np.pi * 220 * t) + 0.001 * rng.standard_normal(n)


def rir(n=SR // 2, t60=0.4, seed=1):
    rng = np.random.default_rng(seed)
    h = rng.standard_normal(n) * np.exp(-6.9 * np.arange(n) / (t60 * SR)) * 0.05
    h[:100] = 0
    h[100] = 1.0
    return h


class Mix(unittest.TestCase):
    N = SR * 2

    def ex(self, K, seed=0, ir=None):
        rng = np.random.default_rng(seed)
        noise = [(np.random.default_rng(9).standard_normal(SR * 3) * 0.05, False, "n")]
        return mix.make_example(rng, speech(), noise, ir, K, self.N)

    def test_deterministic(self):
        a, b = self.ex(knobs(), 3, rir()), self.ex(knobs(), 3, rir())
        np.testing.assert_array_equal(a[0], b[0])
        np.testing.assert_array_equal(a[1], b[1])

    def test_target_is_early_reverb_from_rooms(self):
        """目标 = 人声卷 rooms.early_rir 的结果：和正本独立算一遍逐样本相等。"""
        K = knobs(eq={"on": False}, level={"on": False}, clip={"on": False}, n_noises={"on": False}, snr={"on": False})
        h = rir()
        rng = np.random.default_rng(5)
        noisy, clean, meta = mix.make_example(rng, speech(self.N), [(np.zeros(SR) + 1e-3, False, "n")], h, K, self.N)
        full, early, m = mix.to_rir(h)
        self.assertEqual(meta["reverb"]["early_ms"], mix.EARLY_REVERB_MS)
        cut = m.peak + int(round(mix.EARLY_REVERB_MS * 1e-3 * SR))
        self.assertTrue(np.all(early[cut + 2 * int(SR * 1e-3) :] == 0), "早期 RIR 在 100 ms 之后应为 0")
        self.assertGreater(np.sum(full[cut:] ** 2), 0)
        from scipy.signal import fftconvolve

        want = fftconvolve(speech(self.N), early)[: self.N]
        g = np.dot(clean, want) / np.dot(want, want)
        np.testing.assert_allclose(clean, (g * want).astype(np.float32), atol=1e-6)

    def test_snr_hits_target(self):
        K = knobs(eq={"on": False}, clip={"on": False}, n_noises={"on": False}, reverb={"on": False}, level={"on": False})
        for seed in range(5):
            rng = np.random.default_rng(seed)
            n = np.random.default_rng(9).standard_normal(SR * 3) * 0.05
            s = speech(self.N)
            noisy, clean, meta = mix.make_example(rng, s, [(n, False, "n")], None, K, self.N)
            noise = noisy - clean
            got = mix.gated_level_db(clean) - mix.power_db(noise)
            self.assertAlmostEqual(got, meta["snr_db"], places=1)
            self.assertTrue(-5 <= meta["snr_db"] <= 15)

    def test_wind_lower_bound(self):
        K = knobs()
        lows = [mix.draw_snr(np.random.default_rng(i), K["snr"], True) for i in range(400)]
        self.assertLess(min(lows), -5)
        self.assertGreaterEqual(min(lows), -10)

    def test_level_distribution(self):
        K = knobs(clip={"on": False})
        lv = [self.ex(K, s)[2]["level_dbfs"] for s in range(200)]
        self.assertAlmostEqual(float(np.mean(lv)), -28, delta=2.5)
        self.assertTrue(-60 <= min(lv) and max(lv) <= -10)

    def test_clip_only_input(self):
        K = knobs(clip={"on": True, "prob": 1.0, "thr_lo": 0.5, "thr_hi": 0.5})
        noisy, clean, meta = self.ex(K, 1)
        self.assertLessEqual(np.max(np.abs(noisy)), meta["clip_thr"] + 1e-6)
        self.assertGreater(np.max(np.abs(clean)), 0)

    def test_n_noises(self):
        K = knobs(n_noises={"on": True, "p": {"3": 1.0}})
        self.assertEqual(self.ex(K)[2]["n_noises"], 3)
        self.assertEqual(self.ex(knobs(n_noises={"on": False}))[2]["n_noises"], 1)

    def test_near_silent(self):
        K = knobs(near_silent={"on": True, "frac": 1.0})
        noisy, clean, meta = self.ex(K)
        self.assertTrue(meta["near_silent"])
        self.assertEqual(float(np.max(np.abs(clean))), 0.0)
        self.assertEqual(meta["owner_ivs"], [])
        self.assertGreater(float(np.max(np.abs(noisy))), 0)

    def test_all_off_is_plain_mix(self):
        noisy, clean, meta = self.ex(all_off())
        self.assertIsNone(meta["reverb"])
        self.assertEqual(meta["snr_db"], 5.0)
        self.assertEqual(meta["level_dbfs"], -28.0)
        self.assertIsNone(meta["clip_thr"])

    def test_owner_ivs_follow_speech(self):
        noisy, clean, meta = self.ex(all_off())
        ivs = meta["owner_ivs"]
        self.assertGreater(len(ivs), 3)
        self.assertTrue(all(0 <= a < b <= self.N for a, b in ivs))

    def test_target_and_replay_frac_agree(self):
        """codex 第 3 轮 P2 #4（第 5 条）：抽样只用 replay.frac；target_scene.frac 是第二处表示，两者之和必须为 1。"""
        r = json.loads((HERE / "recipe.json").read_text())
        K = r["knobs"]
        self.assertAlmostEqual(K["target_scene"]["frac"] + K["replay"]["frac"], 1.0, places=2)
        with tempfile.TemporaryDirectory() as d:
            K["target_scene"]["frac"] = 0.9
            p = Path(d) / "r.json"
            p.write_text(json.dumps(r, ensure_ascii=False))
            with self.assertRaises(ValueError):
                loader.load_recipe(p)

    def test_every_knob_switchable_with_source(self):
        for k, v in RECIPE["knobs"].items():
            self.assertIn("on", v, k)
            self.assertTrue(v["src"], k)

    def test_noise_pool_keys_are_exp013_scenes(self):
        scenes = json.loads((mix.EXP013 / "scenes.json").read_text())["scenes"]
        for k in RECIPE["noise_pools"]:
            if not k.startswith("_"):
                self.assertIn(k, scenes)

    def test_no_literal_100ms(self):
        """早期混响 100 ms 的正本在 rooms.py：本目录代码不许另写这个数。"""
        for f in ("mix.py", "loader.py", "shard.py"):
            self.assertNotRegex((HERE / f).read_text(), r"\b100\s*\*\s*1e-3|\b0\.1\s*\*\s*SR|100\s*\*\s*16000\s*/\s*1000")


def tone(path, sec, sr=SR, ch=1, seed=0):
    path.parent.mkdir(parents=True, exist_ok=True)
    x = speech(int(sec * sr), seed) if "flac" in path.suffix else np.random.default_rng(seed).standard_normal(int(sec * sr)) * 0.05
    sf.write(str(path), np.stack([x] * ch, 1) if ch > 1 else x, sr)


def make_tree(root):
    for i, spk in enumerate(("100", "200", "300", "400", "500", "600", "700", "800", "900", "1000")):
        tone(root / f"train-raw/librispeech/LibriSpeech/train-clean-100/{spk}/1/{spk}-1-0000.flac", 2.5, seed=i)
    tone(root / "train-raw/qut-noise/QUT-NOISE/STREET-CITY-1.wav", 8, sr=48000, ch=2, seed=11)
    tone(root / "train-raw/qut-noise/QUT-NOISE/CAFE-CAFE-1.wav", 8, sr=48000, ch=2, seed=12)
    for j in range(10):
        tone(root / f"train-raw/tum-wind/wind/rec{j}.wav", 1, seed=20 + j)
    for k, (user, cls) in enumerate((u, c) for u in ("user0001", "user0002", "user0003") for c in ("Subway", "SubwayStation", "CrowdedIndoor")):
        tone(root / f"train-raw/cochlscene/CochlScene/Train/{cls}/{cls}_{user}_14983413_{k:03d}.wav", 1, seed=50 + k)
    d = root / "train-raw/openslr28/RIRS_NOISES/simulated_rirs/smallroom"
    lines = []
    for room in range(1, 11):
        f = d / f"Room{room:03d}/Room{room:03d}-00001.wav"
        f.parent.mkdir(parents=True, exist_ok=True)
        sf.write(str(f), rir(SR // 4, 0.3, room), SR, subtype="FLOAT")
        lines.append(f"--rir-id r{room} --room-id Room{room:03d} RIRS_NOISES/simulated_rirs/smallroom/{f.parent.name}/{f.name}")  # 上游格式
    (d / "rir_list").write_text("\n".join(lines) + "\n")
    tone(root / "vdn-data/demand/STRAFFIC/ch01.wav", 6, seed=30)
    for i, spk in enumerate(("1272", "1462", "2033", "2414")):  # 终测原料池来源：不进分片，但交集检查要查它
        tone(root / f"train-raw/librispeech/LibriSpeech/{'dev-clean' if i < 2 else 'test-other'}/{spk}/1/{spk}-1-0000.flac", 1, seed=40 + i)


class EndToEnd(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        t = Path(cls.tmp.name)
        root = t / "data"
        make_tree(root)
        R = split.load_rules()
        for r in R["sources"].values():
            r.pop("expect", None)  # 夹具只有几个文件，不按上游数目核
            r.pop("room_lists_sha256", None)  # 夹具的 rir_list 不是上游原文件（钉哈希另有 test_openslr28_rooms）
        cls.splits = t / "splits"
        names = shard.default_sources()  # 要分片的来源 = recipe.json 里出现的全部来源（夹具必须全覆盖）
        for n in names + ["librispeech-dev-clean-test-other"]:  # 终测原料池：只写表，不进分片
            rows = split.split_source(root, n, R["sources"][n], R["seed"], R["splits"], workers=2)
            split.write(n, R["sources"][n], R["seed"], rows, cls.splits)
        cls.shards = t / "shards"
        orig = split.OUT
        cls.idx = shard.build(root, cls.shards, names, cls.splits)
        assert split.OUT == orig
        recipe = dict(RECIPE)
        recipe["knobs"] = knobs(target_scene={"scene": "street-outdoor"})
        cls.recipe = t / "recipe.json"
        cls.recipe.write_text(json.dumps(recipe, ensure_ascii=False))
        cls.builds = t / "builds"
        cls.bid = loader.register_build(cls.shards, cls.recipe, cls.splits, cls.builds)
        cls.kw = {"builds_dir": cls.builds}

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_shards_16k_and_only_train_dev(self):
        for src, d in self.idx["sources"].items():
            self.assertTrue(set(d) - {"kind"} <= {"train", "dev"}, src)
        self.assertIn("train", self.idx["sources"]["qut-noise"])
        p = loader.Pools(self.shards, json.loads(self.recipe.read_text()), self.idx, "train")
        x = p.read(self.idx["sources"]["qut-noise"]["train"]["members"][0])
        self.assertEqual(len(x), 4 * SR)  # 8 s 的前 50% = 4 s，48 kHz 立体声 → 16 kHz 单声道

    def test_build_json_records_noise_scenes(self):
        """噪声池漏了来源要看得见：build.json 记下每份实际有成员的场景名（排序）与跳过的来源（排序）。"""
        bj = json.loads((self.builds / self.bid / "build.json").read_text())
        self.assertEqual(sorted(bj["noise_scenes"]), ["dev", "train"])
        self.assertIn("transit-hub", bj["noise_scenes"]["train"])
        self.assertIn("cochlscene", bj["missing"])  # 夹具里 cochlscene 的片段全落在训练份
        self.assertEqual(bj["missing"], sorted(bj["missing"]))

    def test_build_json_valid(self):
        bj = json.loads((self.builds / self.bid / "build.json").read_text())
        self.assertTrue(bj["valid"])
        self.assertTrue(bj["leak_check"]["ok"])

    def test_leak_check_covers_every_rule_source(self):
        """codex 第 1 轮 P1 #2：终测原料池的人声来源不进分片，以前登记时就不查它。规则里每个来源都要查。"""
        bj = json.loads((self.builds / self.bid / "build.json").read_text())
        self.assertEqual(bj["leak_checked_sources"], sorted(split.load_rules()["sources"]))
        seen = {}
        real = loader.leakcheck.check
        with tempfile.TemporaryDirectory() as d, mock.patch.object(loader.leakcheck, "check", lambda t: seen.update(t) or real(t)):
            loader.register_build(self.shards, self.recipe, self.splits, Path(d))
        self.assertIn("librispeech-dev-clean-test-other", seen)

    def test_register_refuses_missing_table(self):
        with tempfile.TemporaryDirectory() as d:
            sp = Path(d) / "splits"
            shutil.copytree(self.splits, sp)
            for f in sp.glob("librispeech-dev-clean-test-other.*"):
                f.unlink()
            with self.assertRaises(SystemExit):
                loader.register_build(self.shards, self.recipe, sp, Path(d) / "builds")

    def test_build_id_covers_index_and_gpu_code(self):
        """codex 第 1 轮 P1 #1：build_id 要随分片索引（成员偏移）和真正混音的 gpu_mix.py 变。"""
        bid, parts, _ = loader.build_info(self.shards, self.recipe)
        self.assertEqual(loader.EXP013, mix.EXP013)
        for f in ("gpu_mix.py", "mix.py", "loader.py", "shard.py", "leakcheck.py", "split.py",
                  "exp013/dsp.py", "exp013/rooms.py", "exp013/rirs.py"):  # codex 第 2 轮 P1 #1：mix 直接调 exp013 这三个
            self.assertIn(f, parts["code_sha256"])
        with tempfile.TemporaryDirectory() as d:
            sh = Path(d) / "shards"
            shutil.copytree(self.shards, sh)
            idx = json.loads((sh / "index.json").read_text())
            m = idx["sources"]["qut-noise"]["train"]["members"][0]
            k = next(k for k, v in m.items() if isinstance(v, int) and not isinstance(v, bool))
            m[k] += 1
            (sh / "index.json").write_text(json.dumps(idx))
            self.assertNotEqual(loader.build_info(sh, self.recipe)[0], bid)

    def test_unregistered_build_refused(self):
        with self.assertRaises(SystemExit):
            loader.TrainStream(self.shards, self.recipe, builds_dir=self.builds / "nope")

    def test_stream_reproducible_and_worker_split(self):
        a = loader.TrainStream(self.shards, self.recipe, seed=7, **self.kw)
        b0 = loader.TrainStream(self.shards, self.recipe, seed=7, worker_id=0, num_workers=2, **self.kw)
        b1 = loader.TrainStream(self.shards, self.recipe, seed=7, worker_id=1, num_workers=2, **self.kw)
        ia, i0, i1 = iter(a), iter(b0), iter(b1)
        seq = [next(ia) for _ in range(4)]
        inter = [next(i0), next(i1), next(i0), next(i1)]
        for x, y in zip(seq, inter):
            np.testing.assert_array_equal(x["noisy"], y["noisy"])
        self.assertEqual(seq[0]["noisy"].shape, (64000,))
        self.assertEqual(a.build_id, self.bid)

    def test_replay_flag_and_scene(self):
        ds = loader.TrainStream(self.shards, self.recipe, seed=1, **self.kw)
        it = iter(ds)
        ms = [next(it)["meta"] for _ in range(30)]
        self.assertTrue(any(m["replay"] for m in ms))
        self.assertTrue(all((m["scene"] != "street-outdoor") == m["replay"] for m in ms))

    def test_dev_fixed(self):
        d1 = loader.DevSet(self.shards, self.recipe, n=5, **self.kw)
        d2 = loader.DevSet(self.shards, self.recipe, n=5, **self.kw)
        np.testing.assert_array_equal(d1[3]["clean"], d2[3]["clean"])
        with self.assertRaises(IndexError):
            d1[5]

    def test_batch_k_only_depends_on_seed_k(self):
        ds = loader._Base(self.shards, self.recipe, seed=3, **self.kw)
        b2 = loader.batch(ds, 2, 4)
        self.assertEqual(b2["noisy"].shape, (4, 64000))
        again = loader.batch(loader._Base(self.shards, self.recipe, seed=3, **self.kw), 2, 4)
        np.testing.assert_array_equal(b2["noisy"], again["noisy"])
        self.assertEqual([m["i"] for m in b2["meta"]], [8, 9, 10, 11])

    def test_torch_iterable_start_batch(self):
        try:
            import torch  # noqa: F401
        except ImportError:
            self.skipTest("本机无 torch；云上必跑（trainer 的对接测试也覆盖）")
        it = loader.torch_iterable(self.shards, self.recipe, seed=3, batch_size=2, start_batch=0, render="cpu", **self.kw)
        g = iter(it)
        seq = [next(g) for _ in range(3)]
        it2 = loader.torch_iterable(self.shards, self.recipe, seed=3, batch_size=2, start_batch=2, render="cpu", **self.kw)
        b = next(iter(it2))
        self.assertEqual(b["batch_index"], 2)
        self.assertTrue(torch.equal(b["noisy"], seq[2]["noisy"]))

    def test_gpu_path_dataloader_workers_and_parity(self):
        """GPU 路：DataLoader 2 个 worker + start_batch=1 出的批 = 单进程顺序取的第 1、2 批；渲染后与 CPU 路 ≤ 1e-4。"""
        try:
            import torch
            from torch.utils.data import DataLoader
        except ImportError:
            self.skipTest("本机无 torch；用仓库 .venv 跑")
        dev = "cuda" if torch.cuda.is_available() else "cpu"
        it = loader.torch_iterable(self.shards, self.recipe, seed=4, batch_size=3, start_batch=1, **self.kw)
        dl = iter(DataLoader(it, batch_size=None, num_workers=2))
        got = [next(dl) for _ in range(2)]
        ds = loader._Base(self.shards, self.recipe, seed=4, **self.kw)
        for k, b in zip((1, 2), got):
            self.assertEqual(int(b["batch_index"]), k)
            out = loader.render_on_gpu(b, dev)
            ref = loader.batch(ds, k, 3)
            self.assertLessEqual(float((out["noisy"].cpu() - torch.from_numpy(ref["noisy"])).abs().max()), 1e-4)
            self.assertLessEqual(float((out["clean"].cpu() - torch.from_numpy(ref["clean"])).abs().max()), 1e-4)
            self.assertEqual(out["owner_ivs"], ref["owner_ivs"])
            self.assertEqual(out["replay"].cpu().tolist(), ref["replay"].tolist())
            self.assertEqual([m["i"] for m in out["meta"]], [3 * k, 3 * k + 1, 3 * k + 2])


if __name__ == "__main__":
    unittest.main()
