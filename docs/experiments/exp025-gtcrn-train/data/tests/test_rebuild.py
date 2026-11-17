"""rebuild_exp013.py 与 exp013_inputs.py 的本机可测部分：哈希比对逻辑、版本核对、原料清单和 pools 正本一致。
真正的重建在云上跑（D97）：人工步骤见 data/README.md 第六节。"""
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import exp013_inputs  # noqa: E402
import rebuild_exp013 as rb  # noqa: E402


class Compare(unittest.TestCase):
    def write(self, d, name, rows):
        p = Path(d) / name
        p.write_text("".join(f"{h}  _tmp/exp013/{rb.BUILD}/{k}\n" for k, h in rows.items()))
        return p

    def test_same_extra_differ(self):
        with tempfile.TemporaryDirectory() as d:
            old = self.write(d, "a", {"c1/mix.wav": "1" * 64, "c1/owner.wav": "2" * 64})
            self.assertTrue(rb.compare(old, self.write(d, "b", {"c1/mix.wav": "1" * 64, "c1/owner.wav": "2" * 64, "c1/owner_early.wav": "3" * 64}))["ok"])
            r = rb.compare(old, self.write(d, "c", {"c1/mix.wav": "1" * 64, "c1/owner.wav": "f" * 64}))
            self.assertFalse(r["ok"])
            self.assertEqual(r["differ_first"], ["c1/owner.wav"])
            self.assertFalse(rb.compare(old, self.write(d, "e", {"c1/mix.wav": "1" * 64}))["ok"])

    def test_old_build_has_4540(self):
        self.assertEqual(len(rb.read_hashes(rb.OLD / "hashes.txt")), 4540)

    def test_version_diff(self):
        self.assertEqual(rb.version_diff(rb.versions()), {})
        self.assertIn("numpy", rb.version_diff(dict(rb.versions(), numpy="0.0")))

    def test_code_unchanged_since_build(self):
        """本分支上 exp013 代码与旧 build 记的哈希一致；evalcmp 的改动合进来后若变了，这条会红，要核旧输出仍逐位一致。"""
        self.assertEqual(rb.code_drift(), [])


class Inputs(unittest.TestCase):
    def test_record_well_formed(self):
        rec = json.loads(exp013_inputs.OUT.read_text())
        self.assertEqual(rec["build"], rb.BUILD)
        self.assertEqual(len(rec["files"]), 1994)
        man = json.loads((rb.OLD / "manifest.json").read_text())["material_index"]
        self.assertEqual(exp013_inputs.index_of(rec["files"]), man["sha256"])
        self.assertTrue(all(len(r["sha256"]) == 64 for r in rec["files"]))

    def test_verify_against_local_if_present(self):
        pools = exp013_inputs.load_pools()
        if not pools.LIBRI.exists():
            self.skipTest("本机无 ~/vdn-data（云上由 rebuild_exp013.py 跑 --verify）")
        rec = json.loads(exp013_inputs.OUT.read_text())
        rec["files"] = rec["files"][:20]  # 抽前 20 个，单测不读满 2 GB
        self.assertEqual(exp013_inputs.verify(pools, rec), [])
        rec["files"][0] = dict(rec["files"][0], sha256="0" * 64)
        self.assertEqual(len(exp013_inputs.verify(pools, rec)), 1)


if __name__ == "__main__":
    unittest.main()
