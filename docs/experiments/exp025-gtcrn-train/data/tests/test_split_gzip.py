"""切分表超 10 MB 时存成 gzip（2026-11-25：openslr28-rir.jsonl 16.4 MB，被 auto-commit hook 的 10 MB 线挡在库外）。

摘要里的 jsonl_sha256 始终是解压后正文的哈希：压不压缩不改变切分表的身份，云上旧摘要照样能核。
"""
import gzip
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import split  # noqa: E402

RULE = {"unit": "file", "frac": {"train": 1.0}}
ROWS = [{"path": f"a/{i:03d}.wav", "unit": f"a/{i:03d}.wav", "split": "train", "frames": 16000, "sr": 16000} for i in range(50)]


class Gzip(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.out = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_big_table_written_gzipped_and_reads_back(self):
        with mock.patch.object(split, "GZIP_OVER", 100):
            s = split.write("x", RULE, 1, ROWS, self.out)
        self.assertFalse((self.out / "x.jsonl").exists())
        gz = (self.out / "x.jsonl.gz").read_bytes()
        self.assertEqual(split.sha256_bytes(gzip.decompress(gz)), s["jsonl_sha256"])
        s2, rows = split.load_split("x", self.out)
        self.assertEqual(rows, sorted(ROWS, key=lambda r: r["path"]))
        with mock.patch.object(split, "GZIP_OVER", 100):
            split.write("x", RULE, 1, ROWS, self.out)  # 同内容重写，字节逐位相同（gzip 不带时间戳）
        self.assertEqual((self.out / "x.jsonl.gz").read_bytes(), gz)

    def test_small_table_stays_plain(self):
        split.write("x", RULE, 1, ROWS, self.out)
        self.assertTrue((self.out / "x.jsonl").exists())
        self.assertFalse((self.out / "x.jsonl.gz").exists())

    def test_gzip_tamper_red(self):
        with mock.patch.object(split, "GZIP_OVER", 100):
            split.write("x", RULE, 1, ROWS, self.out)
        p = self.out / "x.jsonl.gz"
        p.write_bytes(gzip.compress(gzip.decompress(p.read_bytes()).replace(b'"train"', b'"dev"', 1), mtime=0))
        with self.assertRaises(SystemExit):
            split.load_split("x", self.out)

    def test_both_forms_present_red(self):
        split.write("x", RULE, 1, ROWS, self.out)
        body = (self.out / "x.jsonl").read_bytes()
        (self.out / "x.jsonl.gz").write_bytes(gzip.compress(body, mtime=0))
        with self.assertRaises(SystemExit):
            split.load_split("x", self.out)

    def test_cloud_plain_table_compressed_keeps_summary(self):
        """云上旧版 write 出的 .jsonl，压成 .jsonl.gz 后摘要不用改。"""
        s = split.write("x", RULE, 1, ROWS, self.out)
        jl = self.out / "x.jsonl"
        (self.out / "x.jsonl.gz").write_bytes(gzip.compress(jl.read_bytes(), mtime=0))
        jl.unlink()
        self.assertEqual(split.load_split("x", self.out)[0], s)


class RepoTablesUnderHookLimit(unittest.TestCase):
    def test_every_split_file_under_10mb(self):
        """auto-commit hook 不收超 10 MB 的文件：仓里的切分表一个都不许超。"""
        d = HERE / "splits"
        for p in d.glob("*") if d.exists() else []:
            self.assertLessEqual(p.stat().st_size, 10 * 1024 * 1024, p.name)

    def test_every_summary_has_its_table(self):
        d = HERE / "splits"
        for js in d.glob("*.json") if d.exists() else []:
            n = js.stem
            self.assertEqual(sum((d / f"{n}{e}").exists() for e in (".jsonl", ".jsonl.gz")), 1, n)
            self.assertEqual(json.loads(js.read_text())["source"], n)


if __name__ == "__main__":
    unittest.main()
