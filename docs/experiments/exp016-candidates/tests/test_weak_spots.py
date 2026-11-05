"""weak_spots.py 的测试：只收所有候选一致的格子、单位门槛、跳过镜像项、排序、拒绝旧目录。"""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import weak_spots as W  # noqa: E402


def rec(state, units=5, effect=1.0, mean=1.0):
    return {"state": state, "units": units, "effect": effect, "mean": mean, "lo": mean - 0.1, "hi": mean + 0.1, "n": units * 2}


class Cells(unittest.TestCase):
    def test_needs_every_candidate(self):
        a = {("cafe", "music", "V-NR"): rec("候选好"), ("cafe", "music", "V-ESTOI"): rec("候选好")}
        b = {("cafe", "music", "V-NR"): rec("候选好"), ("cafe", "music", "V-ESTOI"): rec("分不出")}
        got = W.cells({"x": a, "y": b}, W.GOOD)
        self.assertEqual([v for v, _ in got[("cafe", "music")]], ["V-NR"])

    def test_units_threshold(self):
        a = {("cafe", "music", "V-NR"): rec("候选好", units=4)}
        self.assertEqual(dict(W.cells({"x": a}, W.GOOD)), {})

    def test_skips_mirror_and_report_only(self):
        a = {("cafe", "music", v): rec("候选差") for v in ("V-VACUUM", "V-CLICK", "V-ONSET")}
        got = W.cells({"x": a}, W.BAD)
        self.assertEqual([v for v, _ in got[("cafe", "music")]], ["V-ONSET"])

    def test_rank_by_count_then_effect(self):
        g = {("a", "s"): [("V-NR", {"x": rec("候选好", effect=3.0)})],
             ("b", "s"): [("V-NR", {"x": rec("候选好", effect=1.0)}), ("V-ESTOI", {"x": rec("候选好", effect=1.0)})],
             ("c", "s"): [("V-NR", {"x": rec("候选好", effect=2.0)})]}
        self.assertEqual([k for k, _ in W.ranked(g)], [("b", "s"), ("a", "s"), ("c", "s")])


class Cli(unittest.TestCase):
    def test_refuses_existing_dir_and_writes_new(self):
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            js = {"run_id": "r1", "unit": {"V-NR": "dB"}, "candidates": {"x": {"by_scene_sound_item": [
                {"key": ["cafe", "music", "V-NR"], **rec("候选好")}]}}}
            (t / "a.json").write_text(json.dumps(js))
            cmd = [sys.executable, str(HERE.parent / "weak_spots.py"), str(t / "a.json"), "--out"]
            self.assertNotEqual(subprocess.run(cmd + [str(t)], capture_output=True).returncode, 0)
            self.assertEqual(subprocess.run(cmd + [str(t / "o")], capture_output=True).returncode, 0)
            self.assertIn("cafe | music | 1 | V-NR", (t / "o" / "weak_spots.md").read_text())


if __name__ == "__main__":
    unittest.main()
