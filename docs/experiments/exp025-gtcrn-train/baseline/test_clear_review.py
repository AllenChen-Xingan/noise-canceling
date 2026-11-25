import datetime as dt
import hashlib
import json
import tempfile
import unittest
from pathlib import Path

import clear_review as cr

NOW = dt.datetime(2026, 11, 25, 12, 0, 0, tzinfo=dt.timezone.utc)
REVIEW = "费用未知，停发：连不上识别服务：curl: (28) timed out"


def ledger(d, **kw):
    p = Path(d) / "ledger.json"
    v = {"spent_usd": 0.1053, "calls": 1170, "reserved_usd": 0.000405, "review": REVIEW}
    v.update(kw)
    p.write_text(json.dumps(v))
    return p, hashlib.sha256(p.read_bytes()).hexdigest()


class ClearReview(unittest.TestCase):
    def test_price_comes_from_rust_constant(self):
        self.assertEqual(cr.price_per_second(), 0.000008)

    def test_clears_and_books_worst_case(self):
        with tempfile.TemporaryDirectory() as d:
            p, h = ledger(d)
            new, backup = cr.clear(p, h, 1, 12.0, "大总管定", now=NOW, price=0.000008)
            self.assertNotIn("review", new)
            self.assertAlmostEqual(new["spent_usd"], 0.1053 + 12 * 0.000008, places=15)
            self.assertEqual((new["calls"], new["reserved_usd"]), (1171, 0.0))
            adj = new["adjustments"][-1]
            self.assertEqual((adj["review"], adj["worst_count"], adj["backup_sha256"]), (REVIEW, 1, h))
            self.assertIn("按最坏估", adj["note"])
            self.assertEqual(backup.read_bytes(), json.dumps({"spent_usd": 0.1053, "calls": 1170, "reserved_usd": 0.000405, "review": REVIEW}).encode())
            self.assertEqual(json.loads(p.read_text()), new)

    def test_refuses_and_changes_nothing(self):
        with tempfile.TemporaryDirectory() as d:
            p, h = ledger(d)
            before = p.read_bytes()
            with self.assertRaises(SystemExit):
                cr.clear(p, "0" * 64, 1, 12.0, "x", now=NOW)  # sha 对不上
            p2, h2 = ledger(d, review=None)
            with self.assertRaises(SystemExit):
                cr.clear(p2, h2, 1, 12.0, "x", now=NOW)  # 没有 review
            p, h = ledger(d)
            cr.clear(p, h, 1, 12.0, "x", now=NOW)
            p, h = ledger(d)
            with self.assertRaises(FileExistsError):
                cr.clear(p, h, 1, 12.0, "x", now=NOW)  # 同一秒的备份已存在：不覆盖
            self.assertEqual(p.read_bytes(), before)


    def test_bad_input_refused_before_any_file_is_written(self):
        """codex 本分支第 1 轮 P1 #3 · P2 #4：非有限秒数、adjustments 格式不对，都在写盘前拒绝，目录内容不变。"""
        for kw, extra in ((dict(audio_s=float("nan")), {}), (dict(audio_s=float("inf")), {}),
                          (dict(audio_s=1e309), {}), (dict(audio_s=12.0), {"adjustments": None})):
            with tempfile.TemporaryDirectory() as d:
                p, h = ledger(d, **extra)
                before = {f.name: f.read_bytes() for f in Path(d).iterdir() if f.name != "ledger.json.lock"}
                with self.assertRaises(SystemExit):
                    cr.clear(p, h, 1, kw["audio_s"], "x", now=NOW, price=0.000008)
                after = {f.name: f.read_bytes() for f in Path(d).iterdir() if f.name != "ledger.json.lock"}
                self.assertEqual(after, before, kw)


if __name__ == "__main__":
    unittest.main()
