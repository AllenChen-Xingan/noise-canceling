"""power_parse.py 的测试（加测试是默认行为）。用法：python3 -m unittest tests/test_power_parse.py（在 apple-rs 目录下）。
用合成的 powermetrics 文本核对：计时窗向内收、按样本时长加权、缺功率项作废、空闲基线相减、没有计时窗的只展示不排名。"""
import json, os, subprocess, sys, tempfile, unittest
from datetime import datetime, timezone, timedelta

HERE = os.path.dirname(os.path.abspath(__file__))
PARSER = os.path.join(HERE, "..", "tools", "power_parse.py")
TZ = timezone(timedelta(hours=-8))


def block(t, cpu, elapsed=500.0, gpu=2, ane=0, drop=None):
    ts = datetime.fromtimestamp(t, TZ).strftime("%a %b %d %H:%M:%S %Y %z")
    lines = [f"*** Sampled system activity ({ts}) ({elapsed:.2f}ms elapsed) ***"]
    for k, v in (("CPU", cpu), ("GPU", gpu), ("ANE", ane)):
        if k != drop:
            lines.append(f"{k} Power: {v} mW")
    return "\n".join(lines) + "\n\n"


class PowerParse(unittest.TestCase):
    def run_parse(self, power_files, rt_rows):
        d = tempfile.mkdtemp()
        os.makedirs(os.path.join(d, "power"))
        for name, text in power_files.items():
            open(os.path.join(d, "power", name), "w").write(text)
        with open(os.path.join(d, "rt.jsonl"), "w") as f:
            for r in rt_rows:
                f.write(json.dumps(r) + "\n")
        out = subprocess.run([sys.executable, PARSER, d], capture_output=True, text=True, check=True).stdout
        return json.loads(out)

    def test_window_inward_and_idle_subtraction(self):
        t0 = 1_800_000_000
        idle = "".join(block(t0 + i, 100) for i in range(10))
        # 计时窗 [t0+100, t0+110]；窗外两头各有一个高功率样本（预热 / 事后），必须被排除
        run = block(t0 + 100, 900) + "".join(block(t0 + 101 + i, 150) for i in range(8)) + block(t0 + 110, 900)
        res = {"frames": 600, "pace_ms": 16, "p50_ms": 0.1, "p999_ms": 0.2, "max_ms": 0.3, "cpu_over_wall": 0.01,
               "win_start_unix": t0 + 100, "win_end_unix": t0 + 110, "wall_s": 10.0}
        out = self.run_parse({"idle_1.txt": idle, "r1_1.txt": run},
                             [{"round": 1, "set": 1, "file": "r1_1", "config": "x", "result": res}])
        row = out["rows"][0]
        self.assertTrue(row["ranked"])
        self.assertEqual(row["power_mw"]["CPU"], 150)  # 两头 900 mW 的样本被排除
        self.assertEqual(row["delta_mw"], 50)  # 150 − 100（GPU / ANE 相等相减为 0）
        self.assertAlmostEqual(row["uj_per_frame"], 50 * 10.0 / 600 * 1000, places=1)
        self.assertEqual(out["summary"][0]["config"], "x")

    def test_missing_power_item_invalidates(self):
        t0 = 1_800_000_000
        idle = "".join(block(t0 + i, 100) for i in range(5))
        run = "".join(block(t0 + 101 + i, 150, drop="GPU" if i == 3 else None) for i in range(8))
        res = {"frames": 600, "pace_ms": 16, "p50_ms": 0.1, "p999_ms": 0.2, "max_ms": 0.3, "cpu_over_wall": 0.01,
               "win_start_unix": t0 + 100, "win_end_unix": t0 + 110, "wall_s": 10.0}
        out = self.run_parse({"idle_1.txt": idle, "r1_1.txt": run},
                             [{"round": 1, "set": 1, "file": "r1_1", "config": "x", "result": res}])
        self.assertIn("invalid", out["rows"][0])
        self.assertEqual(out["summary"], [])

    def test_no_window_is_shown_not_ranked(self):
        t0 = 1_800_000_000
        idle = "".join(block(t0 + i, 100) for i in range(5))
        run = "".join(block(t0 + 50 + i, 150) for i in range(8))
        res = {"frames": 600, "pace_ms": 16, "p50_ms": 0.1, "p999_ms": 0.2, "max_ms": 0.3, "cpu_over_wall": 0.01}
        out = self.run_parse({"idle.txt": idle, "1.txt": run}, [{"set": 1, "config": "old", "result": res}])
        self.assertFalse(out["rows"][0]["ranked"])
        self.assertEqual(out["summary"], [])
        self.assertEqual(out["unranked_no_window"], ["old"])


if __name__ == "__main__":
    unittest.main()
