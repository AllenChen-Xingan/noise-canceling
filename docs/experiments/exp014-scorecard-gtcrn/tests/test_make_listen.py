"""make_listen.py 的测试：给人听的目录里没有揭盲表、不覆盖、A/B 指向正确的源文件。"""

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import make_listen as L  # noqa: E402


class TestMakeListen(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        d = Path(self.tmp.name)
        self.wavs = d / "wavs"
        self.wavs.mkdir()
        for kind, body in (("mix", b"MIX"), ("gtcrn", b"GTCRN"), ("clean", b"CLEAN")):
            (self.wavs / f"cafe-c00-mba#abcd-{kind}-0001.wav").write_bytes(body)
        self.out, self.key = d / "listen" / "b1", d / "repo" / "b1"
        self.picks = [{"clip": "cafe-c00-mba", "why": "测试", "listen_for": "字头"}]

    def tearDown(self):
        self.tmp.cleanup()

    def test_no_key_in_listen_dir_and_ab_maps_to_source(self):
        key = L.make(self.wavs, self.picks, self.out, self.key, seed=1)
        names = sorted(p.name for p in self.out.iterdir())
        self.assertEqual(names, ["01-A.wav", "01-B.wav", "listen.md"])
        text = (self.out / "listen.md").read_text()
        self.assertNotIn("GTCRN", text)
        it = key["items"][0]
        for ab in "AB":
            body = (self.out / f"01-{ab}.wav").read_bytes()
            self.assertEqual(body, b"GTCRN" if it[ab]["is"] == "GTCRN" else b"MIX")
        self.assertTrue((self.key / "unblind-key.json").exists())
        self.assertEqual(json.loads((self.key / "unblind-key.json").read_text())["seed"], 1)

    def test_refuses_to_overwrite(self):
        L.make(self.wavs, self.picks, self.out, self.key, seed=1)
        with self.assertRaises(SystemExit):
            L.make(self.wavs, self.picks, self.out, self.key, seed=1)


if __name__ == "__main__":
    unittest.main()
