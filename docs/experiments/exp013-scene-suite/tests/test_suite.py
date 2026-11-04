"""场景测试集自检（交代第 6 条）：在临时目录出一个小 build（每地点每档 1 段 × 2 个设备 = 60 段，约 3 s），逐项查。

- 同种子重生成样本哈希一致
- 实测 SNR / SIR 与名义值：管线核对（从 wav 读回、用生成时的定义重算）≤ 0.1 dB；独立算法（P.56）≥ 95% 段 ≤ 1 dB、全部 ≤ 1.5 dB
- 标注里每种声音的起止与分轨能量一致
- 15 个地点键与 021 主表一致，且每个都出了片段
- 与 exp006 训练素材零交集（另有两个负对照：故意放一个训练说话人 / 一段前 60% 的 DEMAND，必须被抓到）
- 清单字段与 clip-manifest.md 一致；分轨相加 = mix
- 场景任务文件与 scenes.json、build 一致
需要本机素材（~/vdn-data/，fetch.py 下载的也在内）；缺素材时整组跳过并写明原因。
"""
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
from scipy.io import wavfile

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import compose  # noqa: E402
import dsp  # noqa: E402
import labels  # noqa: E402
import leak  # noqa: E402
import manifest  # noqa: E402
import pools  # noqa: E402
import tasks  # noqa: E402

ROOT = HERE.parents[2]
HAVE_DATA = pools.LIBRI.exists() and pools.esc50()


def read(root, rel):
    sr, x = wavfile.read(str(Path(root) / rel))
    assert sr == dsp.SR
    return x.astype(np.float64)


@unittest.skipUnless(HAVE_DATA, "缺本机素材 ~/vdn-data（LibriSpeech / ESC-50），人工跑：先 fetch.py")
class SmallBuild(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        t = Path(cls.tmp.name)
        dev = [] if manifest.schema_path() else ["--dev"]  # 干净克隆里没有 scorecard 的 schema 时按开发冒烟跑
        r = subprocess.run([sys.executable, str(HERE / "build.py"), "--build", "t0", "--n", "1", "--workers", "4",
                            "--builds-dir", str(t / "b"), "--audio-root", str(t / "a")] + dev, capture_output=True, text=True)
        if r.returncode != 0:
            raise RuntimeError(r.stdout + r.stderr)
        cls.root = t / "a"
        cls.recs = [json.loads(line) for line in (t / "b/t0/clips.jsonl").read_text().splitlines()]
        cls.man = json.loads((t / "b/t0/manifest.json").read_text())

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_count_and_scene_keys(self):
        cat = re.findall(r"^\| `([a-z-]+)` \|", (ROOT / "docs/research/021_meeting-scenes/catalog.md").read_text(), re.M)
        self.assertEqual(sorted(compose.SCENES["scenes"]), sorted(k for k in cat if k != "other"))
        self.assertEqual({r["scene"] for r in self.recs}, set(compose.SCENES["scenes"]))
        self.assertEqual(len(self.recs), 15 * 2 * 2)

    def test_reproducible_sample_hash(self):
        for r in self.recs[::7]:
            arrays, _ = compose.make_clip(r["scene"], r["tier"], 0, r["device"])
            self.assertEqual(dsp.sample_hash(np.asarray(arrays["mix"], "<f4")), r["hash"]["samples"]["mix"], r["id"])
            self.assertEqual(dsp.sample_hash(read(self.root, r["audio"]["mix"]).astype("<f4")), r["hash"]["samples"]["mix"])

    def test_pairs_share_material(self):
        by = {r["id"]: r for r in self.recs}
        for r in self.recs:
            p = by[r["pair"]]
            self.assertEqual([s["src"] for s in r["sounds"]], [s["src"] for s in p["sounds"]], r["id"])
            self.assertEqual(r["nominal"], p["nominal"])

    def _stems(self, r):
        return {k: read(self.root, v) for k, v in r["audio"]["stems"].items()}

    def test_snr_sir_pipeline_from_files(self):
        """从 wav 读回，用生成时的定义（gated 主说话人电平 vs 整段功率）重算，必须对上名义值。"""
        for r in self.recs:
            owner, floor = read(self.root, r["audio"]["owner"]), read(self.root, r["audio"]["floor"])
            st = self._stems(r)
            cls = {s["inst"]: s["cls"] for s in r["sounds"]}
            lo = dsp.gated_level_db(owner)
            steady = sum((st[k] for k in st if labels.is_steady(cls[k])), np.zeros(len(owner))) + floor
            self.assertAlmostEqual(lo - labels.power_db(steady), r["nominal"]["snr_db"], delta=0.1, msg=r["id"])
            sp = [st[k] for k in st if labels.SOUNDS[cls[k]]["is_speech"]]
            if sp:
                self.assertAlmostEqual(lo - dsp.gated_level_db(sum(sp)), r["nominal"]["sir_db"], delta=0.1, msg=r["id"])

    def test_snr_sir_independent_p56(self):
        d = []
        for r in self.recs:
            m, n = r["measured"], r["nominal"]
            d.append(abs(m["snr_db"] - n["snr_db"]))
            if n["sir_db"] is not None:
                d.append(abs(m["sir_db"] - n["sir_db"]))
        d = np.array(d)
        self.assertLessEqual(d.max(), 1.5)
        self.assertGreaterEqual(np.mean(d <= 1.0), 0.95)

    def test_labels_match_stem_energy(self):
        for r in self.recs:
            st = self._stems(r)
            for s in r["sounds"]:
                x = st[s["inst"]]
                mask = np.zeros(len(x), bool)
                for a, b in s["iv"]:
                    mask[a:b] = True
                self.assertTrue(s["iv"], f"{r['id']} {s['inst']} 没有活动区间")
                out = np.sum(x[~mask] ** 2) / max(np.sum(x**2), 1e-30)
                self.assertLess(out, 0.05, f"{r['id']} {s['inst']} 区间外能量 {out:.3f}")
            owner = read(self.root, r["audio"]["owner"])
            self.assertEqual(r["owner_iv"], dsp.intervals(owner))

    def test_additive(self):
        for r in self.recs:
            mix = read(self.root, r["audio"]["mix"])
            parts = read(self.root, r["audio"]["owner"]) + read(self.root, r["audio"]["noise"]) + read(self.root, r["audio"]["interf"])
            if r["additive"]:
                res = 10 * np.log10(np.sum((mix - parts) ** 2) / np.sum(mix**2) + 1e-30)
                self.assertLess(res, -60, r["id"])
            st = self._stems(r)
            total = sum(st.values()) + read(self.root, r["audio"]["floor"]) + read(self.root, r["audio"]["owner"])
            self.assertLess(np.max(np.abs(total - parts)), 1e-4, r["id"])
            if r["tier"] == "common":
                self.assertTrue(r["additive"])

    def test_owner_dry_aligned_and_levelled(self):
        """回归（自审 P1）：owner_dry 曾比 owner 早到 n0 样本；「全留」段曾按所有人之和定电平。"""
        for r in self.recs:
            dry, own = read(self.root, r["audio"]["owner_dry"]), read(self.root, r["audio"]["owner"])
            if r["polarity"] == "suppress":
                # owner = 干轨 ⊛ RIR（⊛ FIR）。对齐正确时互相关在 0 附近（直达声）有强峰；旧 bug 时强峰在 +n0（19–37 样本）
                a, z = own[24000:88000], dry[24000:88000]
                lags = np.arange(-64, 65)
                xc = np.array([abs(np.dot(a[64 + k : len(a) - 64 + k], z[64 : len(z) - 64])) for k in lags])
                near = xc[(lags >= -1) & (lags <= (1 if r["device"] == "external-mic" else 8))].max()
                # 实测车内 IR 的早期反射和直达差不多强，阈值取 0.3（旧 bug 时 0 附近约 0.1）
                self.assertGreaterEqual(near, 0.3 * xc.max(), r["id"])
            lead = r["targets"][0]
            self.assertAlmostEqual(dsp.gated_level_db(dry), lead["level_dbfs"], delta=0.6, msg=r["id"])

    def test_headroom(self):
        """回归：合成舱噪的亚音频漂移曾让 mix 峰值到 5 倍满幅。整段峰值 ≤ −1 dBFS（削波段除外），调低量记在清单。"""
        for r in self.recs:
            pk = float(np.max(np.abs(read(self.root, r["audio"]["mix"]))))
            if r["additive"]:
                self.assertLessEqual(pk, 10 ** (-1 / 20) + 1e-6, r["id"])
            self.assertLessEqual(r["chain"]["headroom_db"], 0.0)

    def test_synth_crest_factor(self):
        import synth_sounds as ss

        rng = np.random.default_rng(0)
        for name, fn in ss.CONTINUOUS.items():
            x, _ = fn(rng, 12 * dsp.SR)
            crest = 20 * np.log10(np.max(np.abs(x))) - dsp.gated_level_db(x)  # 对活动段电平（咖啡机中间有静音）
            self.assertLess(crest, 20.0, name)

    def test_no_leak(self):
        lk = leak.check(self.recs)  # 重跑，不只看 manifest 里的标志
        self.assertTrue(lk["ok"], lk["hits"])
        self.assertEqual(lk, self.man["leak_check"])
        self.assertGreater(lk["train_rows"], 0)

    def test_leak_negative_controls(self):
        bad_spk = sorted(pools.libri()["exp006_train"])[0]
        fake = json.loads(json.dumps(self.recs[0]))
        fake["targets"][0]["src"] = {"kind": "librispeech", "spk": bad_spk, "utts": []}
        fake["sounds"].append({"inst": "x", "src": {"kind": "demand", "env": "PCAFETER", "ch": 1, "start": 0, "len": 16000, "sr": 16000}})
        lk = leak.check([fake])
        self.assertFalse(lk["ok"])
        self.assertEqual(lk["speakers"], 1)
        self.assertGreaterEqual(lk["demand"], 1)
        # 嵌套的说话人（babble 里、电视的配乐旁）、48 kHz 的 SCAFE 前 60%、exp006 的合成函数名
        f2 = json.loads(json.dumps(self.recs[0]))
        f2["sounds"] = [
            {"inst": "b", "src": {"kind": "babble", "talkers": [{"kind": "librispeech", "spk": bad_spk, "utts": []}]}},
            {"inst": "s", "src": {"kind": "demand", "env": "SCAFE", "ch": 2, "start": 48000 * 100, "len": 48000, "sr": 48000}},
            {"inst": "f", "src": {"kind": "events", "items": [{"kind": "synth", "fn": "synth_hvac"}]}},
        ]
        f2["sounds"].append({"inst": "t", "src": {"kind": "demand", "env": "OOFFICE", "ch": 1, "start": 0, "len": 16000, "sr": 16000}})
        lk2 = leak.check([f2])
        self.assertTrue(any("测试份" in h for h in lk2["hits"]), lk2["hits"])
        self.assertEqual((lk2["speakers"], lk2["synth_fn"]), (1, 1))
        self.assertGreaterEqual(lk2["demand"], 1)
        f3 = json.loads(json.dumps(self.recs[0]))
        f3["sounds"] = [{"inst": "u", "src": {"kind": "mystery"}}]
        with self.assertRaises(ValueError):
            leak.check([f3])

    def test_manifest_fields(self):
        self.assertEqual(manifest.FIELDS, manifest.doc_fields("一、"))
        self.assertEqual(manifest.SOUND_FIELDS + manifest.SOUND_OPTIONAL, manifest.doc_fields("二、"))
        for r in self.recs:
            self.assertEqual(manifest.validate(r), [], r["id"])
            for k in ("mix", "owner", "noise", "interf"):
                self.assertIn(k, r["hash"]["wav_sha256"])

    def test_validator_negative_controls(self):
        """校验器本身要能抓错：坏枚举、多余字段、越界区间、stems 键不对、suppress 却有 2 个 target。"""
        good = self.recs[0]
        self.assertEqual(manifest.validate(good), [])

        def bad(fn):
            r = json.loads(json.dumps(good))
            fn(r)
            return manifest.validate(r)

        self.assertTrue(bad(lambda r: r.update(junk=1)))
        self.assertTrue(bad(lambda r: r["owner_iv"].append([0, r["dur_samples"] + 1])))
        self.assertTrue(bad(lambda r: r["audio"]["stems"].update(ghost="x")))
        self.assertTrue(bad(lambda r: r["targets"].append(r["targets"][0])))
        if not manifest.schema_path():
            self.skipTest("scorecard 的 schema 不在本机：嵌套字段的负对照跳过（字段名负对照已跑）")
        self.assertTrue(bad(lambda r: r.update(scene="moon")))
        self.assertTrue(bad(lambda r: r["measured"].update(junk=1)))
        self.assertTrue(bad(lambda r: r["sounds"][0].update(level_dbfs="loud")))
        d = next((s for s in good["sounds"] if s.get("direction_db")), None)
        if d:
            self.assertTrue(bad(lambda r: next(s for s in r["sounds"] if s["inst"] == d["inst"])["direction_db"].pop("low")))

    def test_sound_classes_known(self):
        for r in self.recs:
            for s in r["sounds"]:
                self.assertIn(s["cls"], labels.SOUNDS)

    def test_same_build_id_refused(self):
        """回归（codex 第 2 轮 P1 #1）：同 build id 再跑一次必须直接失败，不碰已有音频。"""
        t = Path(self.tmp.name)
        before = sorted(p.stat().st_mtime_ns for p in (t / "a").rglob("mix.wav"))[:5]
        r = subprocess.run([sys.executable, str(HERE / "build.py"), "--build", "t0", "--n", "1",
                            "--builds-dir", str(t / "b"), "--audio-root", str(t / "a")], capture_output=True, text=True)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("已存在", r.stdout + r.stderr)
        self.assertEqual(before, sorted(p.stat().st_mtime_ns for p in (t / "a").rglob("mix.wav"))[:5])

    def test_small_build_cannot_make_task_file(self):
        """回归（codex 第 1 轮 P2 #5）：冒烟 build 只有每档 1 段，tasks.py 必须拒绝。"""
        errs = tasks.coverage_errors(Path(self.tmp.name) / "b/t0")
        self.assertTrue(any("应为" in e for e in errs))

    def test_level_check_in_manifest(self):
        lv = self.man["level_check"]
        self.assertTrue(lv["ok"], lv)
        import build

        bad = json.loads(json.dumps(self.recs[:3]))
        bad[0]["measured"]["snr_db"] = bad[0]["nominal"]["snr_db"] + 3
        self.assertFalse(build.level_check(bad)["ok"])

    def test_task_file_shape(self):
        t = tasks.build_tasks("t0")
        self.assertEqual(len(t["tasks"]), 60)
        keys = {(x["real"]["scene"], x["real"]["tier"], x["real"]["device"]) for x in t["tasks"]}
        self.assertEqual(keys, {(s, tr, d) for s in compose.SCENES["scenes"] for tr in ("common", "stress") for d in compose.SCENES["devices"]})
        self.assertEqual(len({x["id"] for x in t["tasks"]}), 60)


class Static(unittest.TestCase):
    def test_missing_material_raises(self):
        """回归（codex 第 2 轮 P2 #5）：不做替代，缺 DEMAND 素材必须报错。"""
        orig = pools.demand_available
        try:
            pools.demand_available = lambda env: False
            with self.assertRaises(FileNotFoundError):
                compose.resolve_cls("bed-street")
        finally:
            pools.demand_available = orig

    def test_index_hash_covers_rooms(self):
        """回归（codex 第 2 轮 P2 #4）：素材索引要包含实测 RIR 与同房间噪声。"""
        if not (pools.DATA / "rir").exists():
            self.skipTest("缺 ~/vdn-data/rir")
        _, n = pools.index_hash()
        self.assertGreater(n, 600)  # LibriSpeech 20 人 + DEMAND + ESC + 实测 RIR（ACE 56 + MIT 270 + McV 108）
    def test_sounds_referenced_exist(self):
        for key, S in compose.SCENES["scenes"].items():
            beds = [b for o in S["rooms"] for b in o.get("bed", [])] + (["bed-ace"] if any("ace" in o for o in S["rooms"]) else []) \
                + (["bed-car-mcv"] if any("mcv" in o for o in S["rooms"]) else [])
            for c in beds + [e["cls"] for e in S["sounds"]]:
                self.assertIn(c, labels.SOUNDS, key)

    def test_no_exp006_import(self):
        for f in HERE.glob("*.py"):
            self.assertNotRegex(f.read_text(), r"^\s*(import sim|from sim )", f.name)

    def test_files_under_600_lines(self):
        for f in list(HERE.glob("*.py")) + list((HERE / "tests").glob("*.py")):
            self.assertLessEqual(len(f.read_text().splitlines()), 600, f.name)

    def test_committed_task_file_matches(self):
        """仓里的 src/eval/suite/scenes.json 必须等于从 scenes.json + 它所指 build 重新生成的（graders 除外）。"""
        if not tasks.OUT.exists():
            self.skipTest("任务文件还没生成")
        cur = json.loads(tasks.OUT.read_text())
        build = cur["tasks"][0]["real"]["clips"].split("/")[-2]
        want = tasks.build_tasks(build, cur)
        self.assertEqual(cur, want)
        self.assertTrue((HERE / "builds" / build / "clips.jsonl").exists())


if __name__ == "__main__":
    unittest.main()
