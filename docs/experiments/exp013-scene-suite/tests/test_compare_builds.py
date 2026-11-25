"""compare_builds.py：同一批声音、不同 build id → 一致；改一条轨哈希 / 素材索引 / 少一段 → 报出来。
用仓里旧 build 17f5 的清单（不需要音频），在临时目录里造副本。"""
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import compare_builds  # noqa: E402

OLD = HERE / "builds/20261103230756-17f5"


class CompareBuilds(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.a = self.tmp / "a"
        shutil.copytree(OLD, self.a)

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def _b(self, edit=None, man=None):
        b = self.tmp / "b"
        shutil.copytree(self.a, b)
        lines = [json.loads(x) for x in (b / "clips.jsonl").read_text().splitlines()]
        for r in lines:  # 换了 build id：路径全变
            r["audio"] = json.loads(json.dumps(r["audio"]).replace("20261103230756-17f5", "OTHER-ID"))
        if edit:
            lines = edit(lines)
        (b / "clips.jsonl").write_text("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in lines))
        if man:
            m = json.loads((b / "manifest.json").read_text())
            man(m)
            (b / "manifest.json").write_text(json.dumps(m))
        return b

    def test_same_sounds_different_id_is_consistent(self):
        self.assertEqual(compare_builds.diff_builds(self.a, self._b()), [])

    def test_track_hash_change_is_reported(self):
        def edit(ls):
            ls[3]["hash"]["samples"]["owner"] = "0" * 64
            return ls
        errs = compare_builds.diff_builds(self.a, self._b(edit))
        self.assertEqual(len(errs), 1)
        self.assertIn("hash.samples", errs[0])

    def test_material_index_and_missing_clip_reported(self):
        def edit(ls):
            return ls[1:]

        def man(m):
            m["material_index"]["sha256"] = "f" * 64
        errs = compare_builds.diff_builds(self.a, self._b(edit, man))
        self.assertTrue(any("material_index" in e for e in errs))
        self.assertTrue(any("只在 A" in e for e in errs))


if __name__ == "__main__":
    unittest.main()
