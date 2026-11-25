"""gap.py 的测试：用手造的 scorecard.json（几张卡）核排名、不过数、方向、并列、格子太少、只新建与核对判据。"""
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gap  # noqa: E402

SCENES = ["street-outdoor", "transit-hub", "car-passenger", "cafe", "home-quiet", "open-office"]


def card(scene, tier, device, items, listen=(), system="gtcrn"):
    """items = [(项, 均值, better, status, report_only)]"""
    mk = lambda v, m, b, st, ro=False: {"v": v, "mean": m, "better": b, "status": st, "report_only": ro}  # noqa: E731
    return {"scene": scene, "tier": tier, "device": device, "system": system,
            "items": [mk(*i) for i in items], "listen": {"models": [mk(*i) for i in listen]}}


def scorecard(bad_order, fails=None, extra=None):
    """`bad_order`：从最差到最好的场景。每个场景 2 档 × 2 设备，每张卡 3 项 + 1 个听感模型，均值按名次单调。"""
    fails = fails or {}
    cards = []
    n = len(bad_order)
    for k, s in enumerate(bad_order):
        good = k / (n - 1)  # 0 = 最差
        for t in ("common", "stress"):
            for d in ("mba-builtin", "external-mic"):
                st = "fail" if fails.get(s) else "pass"
                cards.append(card(s, t, d, [
                    ("V-NR", -10 * good, "lower", st),  # 越小越好：最差的值最大
                    ("V-ESTOI", good, "higher", "pass"),
                    ("V-TSOS", 99.0 if k == 0 else 0.0, "lower", "pass", True),  # 只报不判：不该算进去
                ], listen=[("V-NRMOS-SCOREQ", good, "higher", "pass")]))
    return {"scenes": cards + (extra or [])}


class Gap(unittest.TestCase):
    def test_constants_are_the_documented_ones(self):
        self.assertEqual((gap.TOP_K, gap.MIN_OVERLAP, gap.MIN_CELLS), (4, 3, 10))
        self.assertEqual(gap.PROVISIONAL, ("street-outdoor", "transit-hub", "car-passenger", "cafe"))

    def test_weakness_follows_means_in_both_directions(self):
        order = ["home-quiet", "open-office", "cafe", "car-passenger", "transit-hub", "street-outdoor"]
        g = gap.gap_table(scorecard(order), "gtcrn")
        self.assertEqual([r["scene"] for r in g["rows"]], order)
        self.assertAlmostEqual(g["rows"][0]["weakness"], 1.0)
        self.assertAlmostEqual(g["rows"][-1]["weakness"], 0.0)
        self.assertEqual(g["rows"][0]["cells"], 12)  # 3 个判定格 × 4 切片，只报不判那项不算

    def test_fail_count_ranks_before_weakness(self):
        order = ["home-quiet", "open-office", "cafe", "car-passenger", "transit-hub", "street-outdoor"]
        g = gap.gap_table(scorecard(order, fails={"street-outdoor": True}), "gtcrn")
        self.assertEqual(g["rows"][0]["scene"], "street-outdoor")
        self.assertEqual(g["rows"][0]["fails"], 4)  # V-NR 不过 × 4 切片

    def test_verdict_consistent_when_provisional_are_weakest(self):
        g = gap.gap_table(scorecard(list(gap.PROVISIONAL) + ["home-quiet", "open-office"]), "gtcrn")
        self.assertEqual(sorted(g["top"]), sorted(gap.PROVISIONAL))
        self.assertEqual(g["verdict"], "一致")

    def test_verdict_inconsistent_when_overlap_below_line(self):
        g = gap.gap_table(scorecard(["home-quiet", "open-office", "street-outdoor", "transit-hub", "car-passenger", "cafe"]), "gtcrn")
        self.assertEqual(len(g["overlap"]), 2)
        self.assertEqual(g["verdict"], "不一致")

    def test_ties_share_rank_and_scenes_with_few_cells_are_excluded(self):
        self.assertEqual(gap._ranks({"a": 1.0, "b": 1.0, "c": 2.0}), {"a": 0.25, "b": 0.25, "c": 1.0})
        thin = [card("phone-booth", "common", "mba-builtin", [("V-NR", 50.0, "lower", "fail")])]
        g = gap.gap_table(scorecard(SCENES, extra=thin), "gtcrn")
        self.assertNotIn("phone-booth", [r["scene"] for r in g["rows"]])
        self.assertEqual(g["excluded"], [{"scene": "phone-booth", "cells": 1}])

    def test_other_systems_ignored_and_missing_system_is_error(self):
        other = [card(s, "common", "mba-builtin", [("V-NR", 99.0, "lower", "fail")], system="ref") for s in SCENES]
        g = gap.gap_table(scorecard(SCENES, extra=other), "gtcrn")
        self.assertTrue(all(r["fails"] == 0 for r in g["rows"]))
        with self.assertRaises(ValueError):
            gap.gap_table(scorecard(SCENES), "nope")

    def test_out_writes_new_files_only(self):
        with tempfile.TemporaryDirectory() as d:
            src = Path(d) / "scorecard.json"
            src.write_text(json.dumps(scorecard(SCENES)))
            gap.main([str(src), "--out", str(Path(d) / "o")])
            self.assertTrue((Path(d) / "o/gap.md").exists())
            self.assertEqual(json.loads((Path(d) / "o/gap.json").read_text())["system"], "gtcrn")
            with self.assertRaises(FileExistsError):
                gap.main([str(src), "--out", str(Path(d) / "o")])


if __name__ == "__main__":
    unittest.main()
