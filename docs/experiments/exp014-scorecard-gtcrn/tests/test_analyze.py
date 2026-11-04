"""analyze.py 的单元测试：python3 -m unittest discover -s docs/experiments/exp014-scorecard-gtcrn/tests"""

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import analyze as A  # noqa: E402


def sc(v, value, verdict, threshold=0.0, veto=False):
    return {"v": v, "veto": veto, "value": value, "threshold": threshold, "verdict": verdict, "extra": {}}


def row(clip, system, device="mba-builtin", scene="cafe", tier="common", scores=()):
    return {"clip": clip, "scene": scene, "tier": tier, "device": device, "system": system,
            "sounds": [], "scores": list(scores)}


class TestAnalyze(unittest.TestCase):
    def test_fail_items_worst_clip_decides(self):
        rs = [row("a", "gtcrn", scores=[sc("V-NR", -7, "pass", -6)]),
              row("b", "gtcrn", scores=[sc("V-NR", -2, "fail", -6)])]
        self.assertEqual(A.fail_items(rs, ["V-NR"]), ["V-NR"])
        self.assertEqual(A.fail_items(rs[:1], ["V-NR"]), [])

    def test_better_of_from_threshold(self):
        lower = [row("a", "g", scores=[sc("V-NR", -2, "fail", -6)])]
        higher = [row("a", "g", scores=[sc("V-ESTOI", 0.1, "pass", -0.05)])]
        self.assertEqual(A.better_of(lower, "V-NR"), "越小越好")
        self.assertEqual(A.better_of(higher, "V-ESTOI"), "越大越好")
        self.assertEqual(A.better_of([row("a", "g", scores=[sc("X", None, "unknown")])], "X"), "?")

    def test_disagreements_opposite_first(self):
        rs = [row("big", "gtcrn", scores=[sc("V-NRMOS-SCOREQ", 0.9, "pass"), sc("V-NRMOS-DISTILL", 0.1, "pass")]),
              row("opp", "gtcrn", scores=[sc("V-NRMOS-SCOREQ", 0.1, "pass"), sc("V-NRMOS-DISTILL", -0.1, "fail")]),
              row("ok", "gtcrn", scores=[sc("V-NRMOS-SCOREQ", 0.2, "pass"), sc("V-NRMOS-DISTILL", 0.25, "pass")])]
        d = A.disagreements(rs, "gtcrn")
        self.assertEqual([x["clip"] for x in d], ["opp", "big"])

    def test_report_only_not_counted(self):
        rs = [row("a", "gtcrn", scores=[sc("V-CLICK", 30, "fail"), sc("V-NR", -2, "fail", -6)])]
        self.assertEqual(A.fails_per_clip(rs, "gtcrn"), [("a", ["V-NR"])])

    def test_gap_vs_base_is_paired(self):
        rs = [row("a", "gtcrn", scores=[sc("V-NR", -8, "pass", -6)]),
              row("a", "passthrough", scores=[sc("V-NR", 0, "fail", -6)]),
              row("b", "gtcrn", scores=[sc("V-NR", -4, "fail", -6)])]  # b 没有原声直通：不进配对
        t = A.gap_vs_base(rs, "gtcrn", "passthrough")
        self.assertIn("| V-NR | 越小越好 | 1 | -8.00 |", t)

    def test_build_end_to_end(self):
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            rs = [row("cafe-c00-mba", "gtcrn", scores=[sc("V-NR", -8, "pass", -6)]),
                  row("cafe-c00-mba", "passthrough", scores=[sc("V-NR", 0, "fail", -6)]),
                  row("cafe-c00-ext", "gtcrn", device="external-mic", scores=[sc("V-NR", -10, "pass", -6)])]
            (d / "clips.jsonl").write_text("\n".join(json.dumps(r) for r in rs) + "\n")
            card = {"meta": {"run_id": "t", "systems": ["passthrough", "gtcrn"]},
                    "scenes": [{"scene": "cafe", "tier": "common", "device": "mba-builtin",
                                "system": "gtcrn", "status": "pass"}]}
            (d / "scorecard.json").write_text(json.dumps(card))
            man = [{"id": "cafe-c00-mba", "chain": {"room": {"paired": True}}},
                   {"id": "cafe-c00-ext", "chain": {"room": {"paired": False}}}]
            (d / "m.jsonl").write_text("\n".join(json.dumps(m) for m in man) + "\n")
            text = A.build(d, d / "m.jsonl")
            self.assertIn("| cafe | common | mba-builtin | pass | — | — |", text)
            self.assertIn("| V-NR | -8.00 | -10.00 | +2.00 | 0 / 1 | 0 / 1 |", text)
            self.assertIn("同房间 1 段 · 不同房间 1 段", text)


if __name__ == "__main__":
    unittest.main()
