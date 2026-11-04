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
        r = subprocess.run([sys.executable, str(HERE / "build.py"), "--build", "t0", "--n", "1", "--workers", "4",
                            "--builds-dir", str(t / "b"), "--audio-root", str(t / "a")], capture_output=True, text=True)
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
        self.assertTrue(self.man["leak_check"]["ok"], self.man["leak_check"]["hits"])

    def test_leak_negative_controls(self):
        bad_spk = sorted(pools.libri()["exp006_train"])[0]
        fake = json.loads(json.dumps(self.recs[0]))
        fake["targets"][0]["src"] = {"kind": "librispeech", "spk": bad_spk, "utts": []}
        fake["sounds"].append({"inst": "x", "src": {"kind": "demand", "env": "PCAFETER", "ch": 1, "start": 0, "len": 16000, "sr": 16000}})
        lk = leak.check([fake])
        self.assertFalse(lk["ok"])
        self.assertEqual(lk["speakers"], 1)
        self.assertGreaterEqual(lk["demand"], 1)

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

        self.assertTrue(bad(lambda r: r.update(scene="moon")) if manifest.schema_path() else True)
        self.assertTrue(bad(lambda r: r.update(junk=1)))
        self.assertTrue(bad(lambda r: r["owner_iv"].append([0, r["dur_samples"] + 1])))
        self.assertTrue(bad(lambda r: r["audio"]["stems"].update(ghost="x")))
        self.assertTrue(bad(lambda r: r["targets"].append(r["targets"][0])))
        if manifest.schema_path():
            self.assertTrue(bad(lambda r: r["measured"].update(junk=1)))
            self.assertTrue(bad(lambda r: r["sounds"][0].update(level_dbfs="loud")))

    def test_sound_classes_known(self):
        for r in self.recs:
            for s in r["sounds"]:
                self.assertIn(s["cls"], labels.SOUNDS)

    def test_task_file_shape(self):
        t = tasks.build_tasks("t0")
        self.assertEqual(len(t["tasks"]), 60)
        keys = {(x["real"]["scene"], x["real"]["tier"], x["real"]["device"]) for x in t["tasks"]}
        self.assertEqual(keys, {(s, tr, d) for s in compose.SCENES["scenes"] for tr in ("common", "stress") for d in compose.SCENES["devices"]})
        self.assertEqual(len({x["id"] for x in t["tasks"]}), 60)


class Static(unittest.TestCase):
    def test_sounds_referenced_exist(self):
        for key, S in compose.SCENES["scenes"].items():
            for c in S["bed"] + [e["cls"] for e in S["sounds"]]:
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
