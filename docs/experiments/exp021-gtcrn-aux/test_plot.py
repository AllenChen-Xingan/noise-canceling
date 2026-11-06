"""plot.py 的测试：python3 -m unittest docs/experiments/exp021-gtcrn-aux/test_plot.py"""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plot  # noqa: E402


def row(system, scene, **vals):
    return {"system": system, "scene": scene, "scores": [{"v": v, "value": x} for v, x in vals.items()]}


class Plot(unittest.TestCase):
    def test_parse_name(self):
        self.assertEqual(plot.parse_name("dfn3-om6-fm12"), ("dfn3", -6, -12))
        self.assertEqual(plot.parse_name("gtcrn-raw-om3-fm24"), ("gtcrn-raw", -3, -24))
        self.assertEqual(plot.parse_name("dfn3-nomix"), ("dfn3", None, None))
        self.assertIsNone(plot.parse_name("gtcrn"))

    def test_series_axes_and_scene_filter(self):
        rows = [row("m-om6-fm12", "cafe", **{"V-NR": -8.0}), row("m-om3-fm12", "cafe", **{"V-NR": -6.0}),
                row("m-om6-fm18", "cafe", **{"V-NR": -10.0}), row("m-nomix", "cafe", **{"V-NR": -20.0}),
                row("m-om6-fm12", "bus", **{"V-NR": 99.0})]
        mean = plot.means(rows, {"V-NR"}, {"cafe"})
        self.assertEqual(plot.series(mean, "V-NR", "m", "open"), [("不混回", -20.0), ("-6", -8.0), ("-3", -6.0)])
        self.assertEqual(plot.series(mean, "V-NR", "m", "floor"), [("不混回", -20.0), ("-18", -10.0), ("-12", -8.0)])
        s = plot.svg(mean, ["V-NR"], ["m"], "t")
        self.assertTrue(s.startswith("<svg") and "polyline" in s)


if __name__ == "__main__":
    unittest.main()
