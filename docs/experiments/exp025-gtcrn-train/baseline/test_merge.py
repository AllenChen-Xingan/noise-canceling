"""merge.py 的测试：云上一份（吃字判不了）+ 本机一份（带吃字）→ 合并后每段的吃字取本机、其余取云上；两边逐项比对的读数；
缺段、多段、系统不同、本机没出吃字都报错；只新建不覆盖。"""
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge  # noqa: E402

ITEMS = ["V-NAN", "V-EAT-ASR", "V-ESTOI", "V-NRMOS-SCOREQ"]


def score(v, value, verdict="pass", extra=None):
    return {"v": v, "veto": v in ("V-NAN", "V-EAT-ASR"), "value": value, "threshold": 0.0, "verdict": verdict, "extra": extra or {}}


def row(clip, vals, system="gtcrn"):
    return {"clip": clip, "scene": "cafe", "tier": "common", "device": "mba-builtin", "system": system, "sounds": [],
            "scores": [score(v, vals.get(v), "unknown" if vals.get(v) is None else "pass") for v in ITEMS]}


def cloud(n=3):
    return [row(f"cafe-c0{i}-mba", {"V-NAN": 0.0, "V-ESTOI": 0.1 * i, "V-NRMOS-SCOREQ": 0.2}) for i in range(n)]


def local(n=3, d=0.0):
    return [row(f"cafe-c0{i}-mba", {"V-NAN": 0.0, "V-EAT-ASR": 1.0 * i, "V-ESTOI": 0.1 * i + d, "V-NRMOS-SCOREQ": 0.2 + d})
            for i in range(n)]


class Merge(unittest.TestCase):
    def test_eat_from_local_rest_from_cloud(self):
        rows, rep = merge.merge(cloud(), local(d=1e-6))
        for i, r in enumerate(rows):
            s = {x["v"]: x for x in r["scores"]}
            self.assertEqual(s["V-EAT-ASR"]["value"], 1.0 * i)
            self.assertEqual(s["V-EAT-ASR"]["verdict"], "pass")
            self.assertAlmostEqual(s["V-ESTOI"]["value"], 0.1 * i)  # 云上那份
        self.assertEqual([x["v"] for x in rows[0]["scores"]], ITEMS, "项的顺序不变")
        self.assertAlmostEqual(rep["V-ESTOI"]["max_abs_diff"], 1e-6, places=12)
        self.assertEqual(rep["V-ESTOI"]["pairs"], 3)
        self.assertNotIn("V-EAT-ASR", rep, "吃字不比（云上判不了）")

    def test_parity_thresholds_and_verdict_flips(self):
        self.assertEqual((merge.PARITY_ABS, merge.PARITY_FLIP), (1e-3, 0.01))
        _, rep = merge.merge(cloud(), local(d=1e-6))
        self.assertEqual(merge.parity_errors(rep), [])
        _, rep = merge.merge(cloud(), local(d=2e-3))  # 差 2e-3 > 1e-3
        self.assertTrue(any("V-ESTOI" in e for e in merge.parity_errors(rep)))
        loc = local()
        loc[0]["scores"][2]["verdict"] = "fail"  # 3 段里 1 段判定不同 = 33% > 1%
        _, rep = merge.merge(cloud(), loc)
        self.assertEqual(rep["V-ESTOI"]["verdict_flips"], 1)
        self.assertTrue(any("判定" in e for e in merge.parity_errors(rep)))

    def test_one_sided_values_fail_and_flip_share_uses_all_clips(self):
        """codex 第 1 轮 P2 #3：一边有值一边没值算对拍不过；判定不同的占比按全部段数算，不按「两边都有值」的段数算。"""
        loc = local()
        loc[2]["scores"][2]["value"] = None  # 本机 V-ESTOI 这段没值，云上有
        _, rep = merge.merge(cloud(), loc)
        self.assertEqual((rep["V-ESTOI"]["clips"], rep["V-ESTOI"]["pairs"], rep["V-ESTOI"]["one_sided"]), (3, 2, 1))
        self.assertTrue(any("V-ESTOI" in e and "一边" in e for e in merge.parity_errors(rep)))
        rows_c = [row(f"cafe-c{i:02d}-mba", {"V-NAN": 0.0, "V-ESTOI": None if i >= 2 else 0.1}) for i in range(200)]
        rows_l = [row(f"cafe-c{i:02d}-mba", {"V-NAN": 0.0, "V-EAT-ASR": 1.0, "V-ESTOI": None if i >= 2 else 0.1}) for i in range(200)]
        rows_l[0]["scores"][2]["verdict"] = "fail"  # 200 段里 1 段判定不同 = 0.5% ≤ 1%
        _, rep = merge.merge(rows_c, rows_l)
        self.assertEqual(rep["V-ESTOI"]["verdict_flips"], 1)
        self.assertEqual(merge.parity_errors(rep), [])

    def test_parity_only_compares_runs_that_both_have_eat(self):
        """codex 第 5 轮 P2 #6：识别上云后两边都有吃字值；对拍模式比全部项（含吃字），不合并、不要求哪边没值。"""
        a, b = local(), local(d=1e-6)
        rep = merge.parity(a, b)
        self.assertIn("V-EAT-ASR", rep)
        self.assertEqual(rep["V-EAT-ASR"]["max_abs_diff"], 0.0)
        self.assertAlmostEqual(rep["V-ESTOI"]["max_abs_diff"], 1e-6, places=12)
        self.assertEqual(merge.parity_errors(rep), [])
        with self.assertRaises(ValueError):
            merge.merge(a, b)  # 合并模式照旧拒绝「云上已有吃字」
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            for side, rows in (("c", a), ("l", b)):
                (d / side).mkdir()
                (d / side / "clips.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rows))
                (d / side / "scorecard.json").write_text(json.dumps({"missing": []}))
            merge.main(["--cloud", str(d / "c/clips.jsonl"), "--local", str(d / "l/clips.jsonl"), "--out", str(d / "p"), "--parity-only"])
            self.assertFalse((d / "p/clips.jsonl").exists(), "对拍模式不写合并结果")
            self.assertIn("V-EAT-ASR", json.loads((d / "p/merge-report.json").read_text())["items"])

    def test_audio_identity_checked_in_both_modes(self):
        """codex 第 11 轮 P2 #2：两边都记了 mix_sha256 且不同 → 合并、对拍都拒绝；有一边没记 → 照做，但记「核不了」的段数。"""
        c, l = cloud(), local()
        for r in c:
            r["mix_sha256"] = "a" * 64
        l2 = [dict(r, mix_sha256="b" * 64) for r in l]
        with self.assertRaises(ValueError):
            merge.merge(c, l2)
        with self.assertRaises(ValueError):
            merge.parity(c, l2)
        same = [dict(r, mix_sha256="a" * 64) for r in l]
        _, rep = merge.merge(c, same)
        self.assertEqual(rep["_identity"]["unverified_pairs"], 0)
        _, rep = merge.merge(c, l)  # 本机那份没记
        self.assertEqual(rep["_identity"]["unverified_pairs"], 3)
        self.assertEqual(merge.parity_errors(rep), [], "核不了不算对拍不过，只单列")

    def test_mismatched_clip_sets_or_systems_are_errors(self):
        with self.assertRaises(ValueError):
            merge.merge(cloud(3), local(2))
        with self.assertRaises(ValueError):
            merge.merge(cloud(2), local(3))
        loc = local()
        loc[0]["system"] = "ref"
        with self.assertRaises(ValueError):
            merge.merge(cloud(), loc)

    def test_local_without_eat_is_error(self):
        loc = local()
        loc[1]["scores"][1] = score("V-EAT-ASR", None, "unknown")
        with self.assertRaises(ValueError):
            merge.merge(cloud(), loc)

    def test_cloud_already_has_eat_is_error(self):
        cl = cloud()
        cl[0]["scores"][1] = score("V-EAT-ASR", 3.0)
        with self.assertRaises(ValueError):
            merge.merge(cl, local())

    def _write(self, d, missing_cloud=(), missing_local=()):
        for side, rows, miss in (("c", cloud(), missing_cloud), ("l", local(), missing_local)):
            (d / side).mkdir()
            (d / side / "clips.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rows))
            (d / side / "scorecard.json").write_text(json.dumps({"missing": list(miss)}))
        return ["--cloud", str(d / "c/clips.jsonl"), "--local", str(d / "l/clips.jsonl")]

    def test_missing_clips_on_either_side_refuse_merge(self):
        for mc, ml in (([{"what": "x"}], []), ([], [{"what": "y"}])):
            with tempfile.TemporaryDirectory() as d:
                args = self._write(Path(d), mc, ml)
                with self.assertRaises(ValueError):
                    merge.main(args + ["--out", str(Path(d) / "m")])

    def test_main_writes_new_files(self):
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            args = self._write(d)
            merge.main(args + ["--out", str(d / "m")])
            # 给 vdn-eval --from-clips 读的缺段记录（空）
            self.assertEqual(json.loads((d / "m/scorecard.json").read_text())["missing"], [])
            got = [json.loads(l) for l in (d / "m/clips.jsonl").read_text().splitlines()]
            self.assertEqual(len(got), 3)
            self.assertIn("V-ESTOI", json.loads((d / "m/merge-report.json").read_text())["items"])
            with self.assertRaises(FileExistsError):
                merge.main(args + ["--out", str(d / "m")])


if __name__ == "__main__":
    unittest.main()
