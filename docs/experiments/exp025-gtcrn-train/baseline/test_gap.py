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
            gap.main(["weak", str(src), "--out", str(Path(d) / "o")])
            self.assertTrue((Path(d) / "o/gap-weak.md").exists())
            self.assertEqual(json.loads((Path(d) / "o/gap-weak.json").read_text())["system"], "gtcrn")
            with self.assertRaises(FileExistsError):
                gap.main(["weak", str(src), "--out", str(Path(d) / "o")])


if __name__ == "__main__":
    unittest.main()


def compare_json(by_scene, system_a="gtcrn", system_b="dfn3"):
    """by_scene = {场景: [(项, 判断, 差值均值, better)]} → `vdn-eval compare --by-scene` 的 compare.json 形状。"""
    scopes = [{"scope": "all", "items": []}]
    for s, items in by_scene.items():
        scopes.append({"scope": s, "pairs": 10, "items": [
            {"v": v, "call": c, "better": b, "diff": {"mean": d, "lo": d - 0.1, "hi": d + 0.1, "groups": 5}} for v, c, d, b in items]})
    return {"meta": {"a_system": system_a, "b_system": system_b}, "scopes": scopes}


def baseline_card(report_only=("V-TSOS",)):
    """只用来读「哪些项是判定项」的基线 scorecard.json。"""
    items = [{"v": v, "report_only": v in report_only, "mean": 0.0, "better": "lower", "status": "pass"}
             for v in ("V-NR", "V-ESTOI", "V-TSOS")]
    return {"scenes": [{"scene": "cafe", "tier": "common", "device": "mba-builtin", "system": "gtcrn",
                        "items": items, "listen": {"models": []}}]}


class Uplift(unittest.TestCase):
    def test_ranks_by_up_count_then_normalized_gain(self):
        cj = compare_json({
            "street-outdoor": [("V-NR", "up", -2.0, "lower"), ("V-ESTOI", "up", 0.05, "higher")],
            "transit-hub": [("V-NR", "up", -1.0, "lower"), ("V-ESTOI", "unsure", 0.01, "higher")],
            "cafe": [("V-NR", "up", -3.0, "lower"), ("V-ESTOI", "unsure", 0.0, "higher")],
            "home-quiet": [("V-NR", "down", 1.0, "lower"), ("V-ESTOI", "unsure", -0.01, "higher")],
        })
        u = gap.uplift_table(cj, baseline_card())
        self.assertEqual([r["scene"] for r in u["rows"]], ["street-outdoor", "cafe", "transit-hub", "home-quiet"])
        self.assertEqual((u["rows"][0]["ups"], u["rows"][-1]["downs"]), (2, 1))
        self.assertEqual(u["candidate"], "dfn3")

    def test_report_only_items_and_all_scope_ignored(self):
        cj = compare_json({"cafe": [("V-TSOS", "up", -5.0, "lower")], "street-outdoor": [("V-NR", "up", -1.0, "lower")]})
        u = gap.uplift_table(cj, baseline_card())
        rows = {r["scene"]: r for r in u["rows"]}
        self.assertEqual(rows["cafe"]["ups"], 0, "只报不判的项不算")
        self.assertEqual(rows["street-outdoor"]["ups"], 1)

    def test_excluded_items_are_dropped_and_listed(self):
        cj = compare_json({"cafe": [("V-NR", "up", -1.0, "lower"), ("V-ESTOI", "up", 0.1, "higher")]})
        u = gap.uplift_table(cj, baseline_card(), exclude=("V-ESTOI",))
        self.assertEqual(u["rows"][0]["ups"], 1)
        self.assertEqual(u["excluded_items"], ["V-ESTOI"])

    def test_needs_by_scene_compare(self):
        with self.assertRaises(ValueError):
            gap.uplift_table({"meta": {}, "scopes": [{"scope": "all", "items": []}]}, baseline_card())

    def test_verdict_uses_same_overlap_rule(self):
        order = list(gap.PROVISIONAL) + ["home-quiet"]
        cj = compare_json({s: [("V-NR", "up" if k < 4 else "unsure", -(5 - k), "lower")] for k, s in enumerate(order)})
        u = gap.uplift_table(cj, baseline_card())
        self.assertEqual(sorted(u["top"]), sorted(gap.PROVISIONAL))
        self.assertEqual(u["verdict"], "一致")


ALL15 = ["street-outdoor", "transit-hub", "car-passenger", "cafe", "home-quiet", "open-office", "canteen", "cowork",
         "home-family", "hotel-room", "library-classroom", "meeting-room", "phone-booth", "rail-air-cabin", "bus-metro-cabin"]


class Completeness(unittest.TestCase):
    """codex 第 1 轮 P1 #1 · #2：数据不全不下结论；第 TOP_K 名边界有并列时，重合数是个范围，跨线就不下结论。"""

    def test_min_scenes_matches_task_list(self):
        root = Path(__file__).resolve().parents[3]
        suite = json.loads((root / "src/eval/suite/scenes.json").read_text())
        self.assertEqual(gap.MIN_SCENES, len({t["real"]["scene"] for t in suite["tasks"]}))
        self.assertEqual(gap.SLICES, 4)

    def test_weak_partial_scenes_is_insufficient(self):
        g = gap.gap_table(scorecard(SCENES), "gtcrn")  # 只有 6 个场景
        self.assertEqual(g["verdict"], "数据不足")
        self.assertTrue(g["incomplete"])

    def test_weak_full_scenes_gives_verdict(self):
        g = gap.gap_table(scorecard(ALL15), "gtcrn")
        self.assertEqual(g["incomplete"], [])
        self.assertIn(g["verdict"], ("一致", "不一致"))

    def test_weak_missing_slice_or_missing_clips_is_insufficient(self):
        sc = scorecard(ALL15)
        sc["scenes"] = [c for c in sc["scenes"] if not (c["scene"] == "cafe" and c["tier"] == "stress")]
        self.assertEqual(gap.gap_table(sc, "gtcrn")["verdict"], "数据不足")
        sc = scorecard(ALL15)
        sc["missing"] = [{"what": "cafe-c00-mba"}]
        self.assertEqual(gap.gap_table(sc, "gtcrn")["verdict"], "数据不足")

    def test_boundary_tie_spanning_line_is_undecided(self):
        # 前 3 名是临时选定的 3 个；第 4、5 名同分，一个是临时选定的 cafe，一个不是 → 重合 3 或 4，都 ≥ 3 → 仍可判「一致」
        rows = [{"scene": s, "key": k} for s, k in [("street-outdoor", 9), ("transit-hub", 8), ("car-passenger", 7),
                                                     ("cafe", 5), ("home-quiet", 5), ("open-office", 1)]]
        self.assertEqual(gap._verdict(rows, lambda r: r["key"]), ((3, 4), "一致"))
        # 前 2 名是临时选定的；第 3–5 名三者同分，只有 1 个是临时选定 → 重合 2 或 3，跨线 → 「不确定」
        rows = [{"scene": s, "key": k} for s, k in [("street-outdoor", 9), ("transit-hub", 8), ("car-passenger", 5),
                                                     ("home-quiet", 5), ("open-office", 5), ("cowork", 1)]]
        self.assertEqual(gap._verdict(rows, lambda r: r["key"]), ((2, 3), "不确定（第 4 名边界并列）"))

    def test_uplift_partial_scenes_is_insufficient(self):
        cj = compare_json({"cafe": [("V-NR", "up", -1.0, "lower")]})
        self.assertEqual(gap.uplift_table(cj, baseline_card())["verdict"], "数据不足")
