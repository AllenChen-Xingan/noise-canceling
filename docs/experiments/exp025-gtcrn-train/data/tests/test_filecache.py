"""split 的提速改动：先认单位再算哈希（认不出立刻停）；逐文件读数缓存（重跑只算变了的文件）。
起因：2026-11-25 云上 split 算了 1800 s 哈希后才在 OpenSLR 28 的房间号上失败。"""
import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import filecache  # noqa: E402
import split  # noqa: E402

RULE = {"unit": "file", "glob": "w/*.wav", "frac": {"train": 0.5, "dev": 0.5}, "group": "T"}
ROOM = {"unit": "room", "glob": "w/*.wav", "room_lists": ["w/rir_list"], "frac": {"train": 1.0}, "group": "T"}


class Base(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        for i in range(4):
            p = self.root / f"w/f{i}.wav"
            p.parent.mkdir(parents=True, exist_ok=True)
            sf.write(str(p), np.full(800 + i, 0.01 * (i + 1), "float32"), 16000)
        self.calls = 0
        self.orig = filecache.measure

        def counted(p):
            self.calls += 1
            return self.orig(p)

        filecache.measure = counted

    def tearDown(self):
        filecache.measure = self.orig
        self.tmp.cleanup()

    def run_split(self, rule, cache=None):
        return split.split_source(self.root, "t", rule, 1, ["train", "dev"], workers=2, cache=cache)


class Order(Base):
    def test_unknown_room_stops_before_hashing(self):
        (self.root / "w/rir_list").write_text("--rir-id 1 --room-id R1 w/f0.wav\n")
        with self.assertRaises(SystemExit):
            self.run_split(ROOM)
        self.assertEqual(self.calls, 0)


class Cache(Base):
    def test_rerun_reuses_and_matches(self):
        path = self.root / "cache/c.jsonl"
        a = self.run_split(RULE, filecache.FileCache(path))
        self.assertEqual(self.calls, 4)
        c = filecache.FileCache(path)
        b = self.run_split(RULE, c)
        self.assertEqual(self.calls, 4)
        self.assertEqual((c.hits, c.misses), (4, 0))
        self.assertEqual(a, b)

    def test_changed_file_recomputed(self):
        path = self.root / "cache/c.jsonl"
        self.run_split(RULE, filecache.FileCache(path))
        sf.write(str(self.root / "w/f1.wav"), np.zeros(5000, "float32"), 16000)
        c = filecache.FileCache(path)
        rows = {r["path"]: r for r in self.run_split(RULE, c)}
        self.assertEqual((c.hits, c.misses), (3, 1))
        self.assertEqual(rows["w/f1.wav"]["frames"], 5000)

    def test_half_written_line_ignored(self):
        path = self.root / "cache/c.jsonl"
        self.run_split(RULE, filecache.FileCache(path))
        with open(path, "a") as f:
            f.write('{"path": "w/f0.wav", "byt')
        c = filecache.FileCache(path)
        self.run_split(RULE, c)
        self.assertEqual(c.misses, 0)

    def test_large_file_path_same_readings(self):
        p = self.root / "w/f2.wav"
        small = filecache.measure(p)
        old, filecache.SMALL = filecache.SMALL, 0
        try:
            self.assertEqual(filecache.measure(p), small)
        finally:
            filecache.SMALL = old
        import hashlib

        self.assertEqual(small[0], hashlib.sha256(p.read_bytes()).hexdigest())
        self.assertEqual(json.loads(json.dumps(small[1])), 802)


if __name__ == "__main__":
    unittest.main()
