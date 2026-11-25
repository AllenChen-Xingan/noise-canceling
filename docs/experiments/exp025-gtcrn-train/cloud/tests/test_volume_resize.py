"""扩网络盘：要 --yes、只能变大、不超过上限、过花费检查；账本按扩盘时刻换单价。假接口，不连网。"""

import datetime as dt
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
sys.path.insert(0, HERE)

import config as C  # noqa: E402
import ledger as L  # noqa: E402
from test_cloud import Base  # noqa: E402


class TestVolumeResize(Base):
    def setUp(self):
        super().setUp()
        self.run_cli("volume-create", "--yes")
        self.vid = next(iter(self.fake.volumes))

    def test_needs_yes(self):
        with self.assertRaises(SystemExit):
            self.run_cli("volume-resize", "--size", "200")
        self.assertEqual(self.fake.volumes[self.vid]["size"], C.VOLUME_SIZE_GB)

    def test_grows_and_logs_new_rate(self):
        out, _ = self.run_cli("volume-resize", "--size", "200", "--yes")
        self.assertIn("200 GB", out)
        self.assertEqual(self.fake.volumes[self.vid]["size"], 200)
        row = L.read_rows(self.ledger)[-1]
        self.assertEqual((row["action"], row["resource_id"]), ("volume_resize", self.vid))
        self.assertAlmostEqual(float(row["usd_per_hr"]), L.volume_hourly(200), places=6)
        self.assertAlmostEqual(L.volume_reserve(L.read_rows(self.ledger)), L.volume_hourly(200) * C.VOLUME_RESERVE_HOURS,
                               places=2)  # 预留按新大小

    def test_refuses_shrink_same_or_over_cap(self):
        for size in (C.VOLUME_SIZE_GB - 10, C.VOLUME_SIZE_GB, C.VOLUME_MAX_GB + 1):
            with self.subTest(size), self.assertRaises(SystemExit):
                self.run_cli("volume-resize", "--size", str(size), "--yes")
        self.assertFalse([c for c in self.fake.calls if c[0] == "PATCH"])

    def test_over_budget_never_patches(self):
        with self.assertRaises(SystemExit):
            import rp_ops as O
            with __import__("unittest").mock.patch.object(C, "SPEND_CAP_USD", 10.6):
                O.resize_volume(self.cli, 200, ledger_path=self.ledger)
        self.assertFalse([c for c in self.fake.calls if c[0] == "PATCH"])

    def test_spend_switches_rate_at_resize_time(self):
        t0 = dt.datetime(2026, 11, 1, tzinfo=dt.timezone.utc)
        rows = [{"action": "volume_create", "resource_id": "v", "usd_per_hr": "0.01", "started_utc": L.iso(t0)},
                {"action": "volume_resize", "resource_id": "v", "usd_per_hr": "0.02",
                 "started_utc": L.iso(t0 + dt.timedelta(hours=10))}]
        self.assertAlmostEqual(L.volume_spent(rows, t0 + dt.timedelta(hours=30)), 10 * 0.01 + 20 * 0.02)


if __name__ == "__main__":
    import unittest
    unittest.main()
