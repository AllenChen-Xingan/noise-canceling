"""subset.py 的测试：用仓里真实的 scenes.json 与 build 6323 清单（不需要音频）。"""
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]  # baseline → exp025 → experiments → docs → 仓库根
sys.path.insert(0, str(HERE))
import subset  # noqa: E402

SUITE = ROOT / "src/eval/suite/scenes.json"


def _load():
    s = json.loads(SUITE.read_text())
    lines = (ROOT / s["tasks"][0]["real"]["clips"]).read_text().splitlines()
    return s, lines


class Subset(unittest.TestCase):
    def test_keeps_lines_verbatim_and_only_covering_tasks(self):
        s, lines = _load()
        ids = ["cafe-c00-mba", "cafe-c01-mba", "street-outdoor-c00-ext"]
        got, sub = subset.subset(s, lines, ids)
        self.assertEqual(sorted(json.loads(l)["id"] for l in got), sorted(ids))
        self.assertTrue(all(l in lines for l in got), "子集行必须是原行原样")
        keys = {(t["real"]["scene"], t["real"]["tier"], t["real"]["device"]) for t in sub["tasks"]}
        self.assertEqual(keys, {("cafe", "common", "mba-builtin"), ("street-outdoor", "common", "external-mic")})

    def test_limits_and_unknown_ids(self):
        s, lines = _load()
        with self.assertRaises(ValueError):
            subset.subset(s, lines, [])
        with self.assertRaises(ValueError):
            subset.subset(s, lines, [f"cafe-c0{i}-mba" for i in range(6)])
        with self.assertRaises(ValueError):
            subset.subset(s, lines, ["no-such-clip"])

    def test_main_writes_new_files_pointing_at_subset(self):
        with tempfile.TemporaryDirectory() as d:
            out = Path(d) / "s"
            import os
            cwd = os.getcwd()
            os.chdir(ROOT)
            try:
                subset.main(["--suite", str(SUITE), "--clips", "cafe-c00-mba", "--out", str(out)])
                with self.assertRaises(FileExistsError):
                    subset.main(["--suite", str(SUITE), "--clips", "cafe-c00-mba", "--out", str(out)])
            finally:
                os.chdir(cwd)
            sub = json.loads((out / "suite.json").read_text())
            self.assertEqual(len(sub["tasks"]), 1)
            self.assertEqual(sub["tasks"][0]["real"]["clips"], str((out / "clips.jsonl").resolve()))


if __name__ == "__main__":
    unittest.main()
