"""split.py + leakcheck.py：切法确定、不跨份、和 027 表一致；交集检查能抓到故意放进去的评测素材（负对照）。
夹具在临时目录现造（几十个极短 wav / flac，<100 KB），不用真数据集。"""
import json
import re
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import leakcheck  # noqa: E402
import split  # noqa: E402

DECISION = HERE.parents[2] / "research/027_gtcrn-finetune/decision.md"


def tone(path, n=1600, sr=16000):
    path.parent.mkdir(parents=True, exist_ok=True)
    sf.write(str(path), np.sin(np.arange(n) * 0.05).astype("float32") * 0.1, sr)


class Allocate(unittest.TestCase):
    ORDER = ["train", "dev", "final", "d63"]

    def test_sum_exact(self):
        for n in range(0, 60):
            c = split.allocate(n, {"train": 0.7, "dev": 0.1, "final": 0.1, "d63": 0.1}, self.ORDER)
            self.assertEqual(sum(c.values()), n)

    def test_deterministic_and_order_free(self):
        us = [f"s{i}" for i in range(40)]
        f = {"train": 0.8, "dev": 0.1, "final": 0.0, "d63": 0.1}
        a = split.assign_units(us, f, 1, "x", self.ORDER)
        b = split.assign_units(list(reversed(us)), f, 1, "x", self.ORDER)
        self.assertEqual(a, b)
        self.assertEqual(sum(v == "train" for v in a.values()), 32)
        self.assertNotEqual(a, split.assign_units(us, f, 2, "x", self.ORDER))

    def test_strata_each_split(self):
        us = [f"r{i}" for i in range(10)] + [f"s{i}" for i in range(10)]
        st = {u: u[0] for u in us}
        a = split.assign_units(us, {"train": 0.7, "dev": 0.1, "final": 0.1, "d63": 0.1}, 0, "t", self.ORDER, st)
        for layer in "rs":
            self.assertEqual(sum(1 for u, k in a.items() if u[0] == layer and k == "train"), 7)

    def test_bad_frac(self):
        with self.assertRaises(ValueError):
            split.assign_units(["a"], {"train": 0.5}, 0, "x", self.ORDER)

    def test_time_segments_cover(self):
        seg = split.time_segments(1000, [["train", 0.0, 0.5], ["dev", 0.5, 0.6], ["final", 0.6, 0.8], ["d63", 0.8, 1.0]])
        self.assertEqual(seg, {"train": [0, 500], "dev": [500, 600], "final": [600, 800], "d63": [800, 1000]})


class Rules(unittest.TestCase):
    """split_rules.json 是 027 第六节表的第二处表示：逐行比对比例。"""

    def test_matches_027_table(self):
        text = DECISION.read_text()
        sec = text.split("## 六、训练数据")[1]
        want = {"LibriSpeech train-clean-100": "librispeech-train-clean-100", "LibriSpeech dev-clean + test-other": "librispeech-dev-clean-test-other",
                "QUT-NOISE": "qut-noise", "TUM 风噪": "tum-wind", "OpenSLR 28 冲激响应": "openslr28-rir"}
        R = split.load_rules()["sources"]
        for row in re.findall(r"^\| (.+?) \| .+? \| (.+?) \|", sec, re.M):
            name, cell = row
            if name not in want:
                continue
            rule = R[want[name]]
            pct = [float(x) / 100 if x not in ("—", "") else 0.0 for x in re.findall(r"(\d+|—)%?", cell)][:4]
            if rule["unit"] == "time":
                got = [b - a for _, a, b in rule["time"]]
            else:
                got = [rule["frac"][k] for k in ("train", "dev", "final", "d63")]
            self.assertEqual([round(x, 3) for x in got], [round(x, 3) for x in pct], name)
            want.pop(name)
        self.assertEqual(want, {}, "027 表里这些行没找到")


class SplitTree(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        for spk in ("100", "200", "300", "400", "500", "600", "700", "800", "900", "1000"):
            for u in range(2):
                tone(self.root / f"train-raw/librispeech/LibriSpeech/train-clean-100/{spk}/1/{spk}-1-{u:04d}.flac")
        d = self.root / "train-raw/openslr28/RIRS_NOISES/simulated_rirs/smallroom"
        lines = []
        for room in range(1, 11):
            for pos in range(2):
                f = d / f"Room{room:03d}/Room{room:03d}-{pos:05d}.wav"
                tone(f, 400)
                lines.append(f"--rir-id small-{room}-{pos} --room-id Room{room:03d} --rir-rspecifier RIRS_NOISES/simulated_rirs/smallroom/{f.parent.name}/{f.name}")
        (d / "rir_list").write_text("\n".join(lines) + "\n")
        self.R = split.load_rules()
        for r in self.R["sources"].values():
            r.pop("expect", None)  # 夹具只有几个文件，不按上游数目核（数目检查另有 Expect 测试）
            r.pop("room_lists_sha256", None)  # 夹具的 rir_list 不是上游原文件（钉哈希另有 test_openslr28_rooms）

    def tearDown(self):
        self.tmp.cleanup()

    def run_src(self, name):
        return split.split_source(self.root, name, self.R["sources"][name], self.R["seed"], self.R["splits"], workers=2)

    def test_speaker_never_crosses(self):
        rows = self.run_src("librispeech-train-clean-100")
        by = {}
        for r in rows:
            by.setdefault(r["unit"], set()).add(r["split"])
        self.assertTrue(all(len(v) == 1 for v in by.values()))
        self.assertEqual(sum(1 for v in by.values() if "train" in v), 8)
        self.assertTrue(all(len(r["sha256"]) == 64 for r in rows))

    def test_room_never_crosses(self):
        rows = self.run_src("openslr28-rir")
        by = {}
        for r in rows:
            by.setdefault(r["unit"], set()).add(r["split"])
        self.assertEqual(len(by), 10)
        self.assertTrue(all(len(v) == 1 for v in by.values()))

    def test_room_missing_from_list_red(self):
        tone(self.root / "train-raw/openslr28/RIRS_NOISES/simulated_rirs/smallroom/Room099/Room099-00000.wav", 400)
        with self.assertRaises(SystemExit):
            self.run_src("openslr28-rir")

    def test_write_no_overwrite_and_tamper(self):
        rows = self.run_src("librispeech-train-clean-100")
        out = self.root / "splits"
        rule = self.R["sources"]["librispeech-train-clean-100"]
        split.write("librispeech-train-clean-100", rule, 1, rows, out)
        split.write("librispeech-train-clean-100", rule, 1, rows, out)  # 同内容可以重写
        rows[0]["split"] = "dev" if rows[0]["split"] != "dev" else "train"
        with self.assertRaises(SystemExit):
            split.write("librispeech-train-clean-100", rule, 1, rows, out)
        jl = out / "librispeech-train-clean-100.jsonl"
        jl.write_text(jl.read_text().replace('"train"', '"dev"', 1))
        with self.assertRaises(SystemExit):
            split.load_split("librispeech-train-clean-100", out)


class Expect(unittest.TestCase):
    """通配符写错时 split 不写表：文件数 / 说话人数对不上上游公布的数目就停。"""

    def test_every_source_has_expect_with_src(self):
        for n, r in split.load_rules()["sources"].items():
            self.assertIn("expect", r, n)
            self.assertTrue(r["expect"].get("src"), n)
            self.assertTrue({"files", "min_files"} & set(r["expect"]), n)

    def test_exact_count_mismatch_stops(self):
        rule = {"unit": "file", "expect": {"files": 3}}
        split.check_expect("x", rule, ["a.wav", "b.wav", "c.wav"])
        with self.assertRaises(SystemExit):
            split.check_expect("x", rule, ["a.wav", "b.wav"])

    def test_min_count(self):
        rule = {"unit": "file", "expect": {"min_files": 2}}
        split.check_expect("x", rule, ["a.wav", "b.wav", "c.wav"])
        with self.assertRaises(SystemExit):
            split.check_expect("x", rule, ["a.wav"])

    def test_speaker_count(self):
        rule = {"unit": "speaker", "expect": {"files": 3, "units": 2}}
        fs = ["L/19/1/a.flac", "L/19/2/b.flac", "L/26/1/c.flac"]
        split.check_expect("x", rule, fs)
        with self.assertRaises(SystemExit):
            split.check_expect("x", rule, ["L/19/1/a.flac", "L/19/2/b.flac", "L/19/3/c.flac"])


def row(path, split_=None, segments=None, unit=None, sr=16000):
    r = {"path": path, "sr": sr, "unit": unit or path}
    if segments:
        r["segments"] = segments
    else:
        r["split"] = split_
    return r


class Leak(unittest.TestCase):
    def setUp(self):
        self.ex = leakcheck.load_exclude()

    def clean_pool(self):
        return {"librispeech-train-clean-100": [row("train-raw/librispeech/LibriSpeech/train-clean-100/19/1/19-1-0000.flac", "train", unit="19")],
                "demand-train": [row("vdn-data/demand/DKITCHEN/ch01.wav", segments={"train": [0, 2_000_000], "dev": [2_000_000, 2_400_000]})],
                "openslr28-rir": [row("train-raw/openslr28/RIRS_NOISES/simulated_rirs/smallroom/Room001/Room001-00001.wav", "train")]}

    def test_clean_ok(self):
        r = leakcheck.check(self.clean_pool(), self.ex)
        self.assertTrue(r["ok"], r["hits"])

    def test_speaker_caught(self):
        p = self.clean_pool()
        spk = self.ex["librispeech_speakers"][0]
        p["librispeech-train-clean-100"].append(row(f"x/{spk}/1/{spk}-1-0.flac", "dev", unit=spk))
        self.assertFalse(leakcheck.check(p, self.ex)["ok"])

    def test_demand_overlap_caught(self):
        s = self.ex["demand_segments"][0]
        p = self.clean_pool()
        p["demand-train"].append(row(f"vdn-data/demand/{s['env']}/ch{s['ch']:02d}.wav", segments={"train": [s["start"], s["start"] + 10]}))
        r = leakcheck.check(p, self.ex)
        self.assertFalse(r["ok"])
        self.assertEqual(r["per_key"]["demand_segments"], 1)

    def test_d63_segment_not_counted(self):
        s = self.ex["demand_segments"][0]
        p = self.clean_pool()
        p["demand-train"].append(row(f"vdn-data/demand/{s['env']}/ch{s['ch']:02d}.wav", segments={"d63": [s["start"], s["start"] + 10]}))
        self.assertTrue(leakcheck.check(p, self.ex)["ok"])

    def test_exp013_dataset_caught(self):
        p = self.clean_pool()
        p["openslr28-rir"].append(row("vdn-data/rir/mit-ir-survey/Audio/h001.wav", "final"))
        self.assertFalse(leakcheck.check(p, self.ex)["ok"])

    def test_rir_basename_caught(self):
        f = self.ex["rir_files"][0].split("#")[0]
        p = self.clean_pool()
        p["openslr28-rir"].append(row("train-raw/openslr28/x/" + Path(f).name, "train"))
        self.assertFalse(leakcheck.check(p, self.ex)["ok"])

    def test_unknown_key_red(self):
        ex = dict(self.ex, new_kind=[1])
        self.assertFalse(leakcheck.check(self.clean_pool(), ex)["ok"])

    def test_synth_import_red(self):
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "x.py").write_text("import numpy\nfrom synth_sounds import thing\n")
            self.assertTrue(leakcheck.chk_synth(self.ex, {}, d))
        self.assertEqual(leakcheck.chk_synth(self.ex, {}), [], "本目录代码 import 了 synth_sounds")

    def test_exclude_reproduces_from_leak_py(self):
        """exclude.json 必须等于 exp013 leak.exclude_table（正本）对同一 build 清单的输出：防有人手改 exclude.json。"""
        sys.path.insert(0, str(leakcheck.EXP013))
        import leak  # noqa: E402

        recs = [json.loads(x) for x in (leakcheck.EXCLUDE.parent / "clips.jsonl").read_text().splitlines()]
        again = json.loads(json.dumps(leak.exclude_table(recs), ensure_ascii=False))
        self.assertEqual(again, self.ex)


if __name__ == "__main__":
    unittest.main()
