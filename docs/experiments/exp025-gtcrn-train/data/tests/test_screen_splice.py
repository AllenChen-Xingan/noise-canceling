"""`screen_tone.py --only-regex` + `--base`：只并一段，其余行原样带过（第五批 Room112 补单音检查用它）。

为什么只并一段：Room112 的 248 条才补了 250 Hz 单音检查，其余 1022 条（C236 620 + 三间办公室 402）已经有结论。
整表重并会把老结论再写一遍、还可能换掉字节；只并匹配的那一段，其余行逐字节照抄，改哪儿看得见。

核的事：
- 匹配的行换成 `merge` 的结果，顺序照 `--base`；不匹配的行逐字节不动
- 对不上就停：`--screen` 里匹配的清单与 `--base` 不同（少 / 多 / 顺序不同）· `--tone` 里有不匹配的行 · 没有 `--base`
- `--check FILE` 用同一份 jsonl 写法重算再与文件逐字节比，不同就退非零
- 真产物：`screen/but-reverbdb-silence-b5.jsonl` 由仓里两份输入（老筛查表 + Room112 单音 jsonl）重算得逐字节相同；
  Room112 248 条里 keep 188、因单音新丢 21，每条都有 `measures.tone_where`；其余 1022 条与老表逐字节相同
"""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "tests"))
import screen_tone  # noqa: E402
from test_screen_tone import PRESENT_DB, screen_row, tone_row  # noqa: E402  行形状只从那里取一份

BASE = HERE / "screen" / "but-reverbdb-silence.jsonl"
NEW = HERE / "screen" / "but-reverbdb-silence-b5.jsonl"
TONE = HERE / "screen" / "room112-tone.jsonl"
ROOM = "/Hotel_SkalskyDvur_Room112/"
REGEX = r"/Hotel_SkalskyDvur_Room112/"
FIXTURE_REGEX = r"^room/"  # 夹具里的路径（room/1.wav）与真路径不同，正则跟着换


def spl(base, screen, tone, only_regex=FIXTURE_REGEX):
    return screen_tone.splice(base, screen, tone, screen_tone.load_drop_where(),
                              PRESENT_DB, only_regex)


class Splice(unittest.TestCase):
    def setUp(self):
        self.base = [screen_row("keep/a.wav"), screen_row("room/1.wav"), screen_row("keep/b.wav"),
                     screen_row("room/2.wav", verdict="drop", reasons=["不平稳"])]
        self.screen = [self.base[1], self.base[3]]
        self.tone = [tone_row("room/1.wav", "none", []), tone_row("room/2.wav", "later_too", [2])]

    def test_only_matching_rows_are_replaced(self):
        res = spl(self.base, self.screen, self.tone)
        rows = res["rows"]
        self.assertEqual([r["path"] for r in rows], [r["path"] for r in self.base])  # 顺序照 base
        self.assertEqual(rows[0], self.base[0])  # 不匹配的原样带过（同一个对象也没关系：merge 只改匹配的行）
        self.assertEqual(rows[2], self.base[2])
        self.assertNotIn("tone_where", self.base[1].get("measures", {}))  # 输入列表本身不被改写
        self.assertEqual(self.base[1]["verdict"], "keep")
        self.assertIn("tone_where", rows[1]["measures"])
        self.assertEqual(rows[1]["measures"]["tone_where"], "none")
        self.assertEqual(rows[3]["measures"]["tone_where"], "later_too")
        self.assertEqual(rows[3]["verdict"], "drop")

    def test_summary_counts_only_the_merged_part(self):
        res = spl(self.base, self.screen, self.tone)
        self.assertEqual((res["total"], res["keep"], res["drop"], res["tone_dropped"]), (2, 1, 1, 0))

    def test_jsonl_text_keeps_passthrough_bytes(self):
        text = screen_tone.jsonl_text(spl(self.base, self.screen, self.tone)["rows"])
        lines = text.splitlines()
        self.assertEqual(lines[0], json.dumps(self.base[0], ensure_ascii=False))
        self.assertEqual(lines[2], json.dumps(self.base[2], ensure_ascii=False))

    def test_missing_matching_row_stops(self):
        with self.assertRaises(SystemExit) as cm:
            spl(self.base, self.screen[:1], self.tone)
        self.assertIn("room/2.wav", str(cm.exception))

    def test_extra_matching_row_stops(self):
        extra = screen_row("room/3.wav")
        with self.assertRaises(SystemExit) as cm:
            spl(self.base, self.screen + [extra], self.tone)
        self.assertIn("room/3.wav", str(cm.exception))

    def test_order_change_stops(self):
        with self.assertRaises(SystemExit) as cm:
            spl(self.base, list(reversed(self.screen)), self.tone)
        self.assertIn("对不上", str(cm.exception))

    def test_tone_row_outside_the_regex_stops(self):
        with self.assertRaises(SystemExit) as cm:
            spl(self.base, self.screen, self.tone + [tone_row("keep/a.wav", "none", [])])
        self.assertIn("keep/a.wav", str(cm.exception))

    def test_bad_regex_stops(self):
        with self.assertRaises(SystemExit) as cm:
            spl(self.base, self.screen, self.tone, only_regex="(")
        self.assertIn("正则", str(cm.exception))


class RealFile(unittest.TestCase):
    """真产物：老表 + Room112 单音 jsonl → `screen/but-reverbdb-silence-b5.jsonl`，逐字节可复现。"""

    def setUp(self):
        self.base = screen_tone.read_jsonl(BASE)
        self.res = spl(self.base, self.base, screen_tone.read_jsonl(TONE), only_regex=REGEX)
        self.text = screen_tone.jsonl_text(self.res["rows"])

    def test_regenerated_bytes_are_the_committed_file(self):
        self.assertTrue(NEW.is_file(), f"没有 {NEW.name}：用 screen_tone.py --only-regex 生成它")
        self.assertEqual(self.text.encode("utf-8"), NEW.read_bytes())

    def test_non_room_lines_are_byte_identical(self):
        got = self.text.splitlines()
        want = BASE.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(got), len(want))
        for i, (a, b) in enumerate(zip(want, got), 1):
            if ROOM not in json.loads(a)["path"]:
                self.assertEqual(a, b, f"第 {i} 行不是 Room112 却被改了")

    def test_room112_counts(self):
        room = [r for r in self.res["rows"] if ROOM in r["path"]]
        self.assertEqual(len(room), 248)
        self.assertEqual(sum(1 for r in room if r["verdict"] == "keep"), 188)
        self.assertEqual(self.res["tone_dropped"], 21)
        self.assertEqual(self.res["keep"], 188)
        self.assertEqual(self.res["drop"], 60)

    def test_every_room_row_has_tone_where(self):
        for r in self.res["rows"]:
            if ROOM in r["path"]:
                self.assertIn("tone_where", r.get("measures", {}), r["path"])


class Cli(unittest.TestCase):
    def run_cli(self, args):
        return subprocess.run([sys.executable, str(HERE / "screen_tone.py")] + args,
                              capture_output=True, text=True)

    def args(self):
        return ["--base", str(BASE), "--screen", str(BASE), "--tone", str(TONE),
                "--only-regex", REGEX]

    def test_check_passes_on_the_committed_file(self):
        r = self.run_cli(self.args() + ["--check", str(NEW)])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("逐字节一致", r.stdout)
        self.assertIn("共 248 条：keep 188 · drop 60（其中因单音新丢 21 条）", r.stdout)

    def test_check_fails_on_a_copy_with_one_changed_line(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "changed.jsonl"
            lines = NEW.read_text(encoding="utf-8").splitlines()
            lines[0] = lines[0].replace("{", "{ ", 1)  # 还是合法 json，只是字节变了
            p.write_text("\n".join(lines) + "\n", encoding="utf-8")
            r = self.run_cli(self.args() + ["--check", str(p)])
            self.assertNotEqual(r.returncode, 0)
            self.assertIn("第 1 行", r.stderr)

    def test_out_and_check_write_the_same_bytes(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "out.jsonl"
            r = self.run_cli(self.args() + ["--out", str(p), "--check", str(NEW)])
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(p.read_bytes(), NEW.read_bytes())

    def test_needs_base_with_only_regex(self):
        r = self.run_cli(["--screen", str(BASE), "--tone", str(TONE), "--only-regex", REGEX,
                          "--out", str(Path(tempfile.gettempdir()) / "x.jsonl")])
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("--base", r.stderr)

    def test_needs_out_or_check(self):
        r = self.run_cli(self.args())
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("--out", r.stderr)

    def test_refuses_to_overwrite_the_base(self):
        args = ["--base", str(BASE), "--screen", str(NEW), "--tone", str(TONE),
                "--only-regex", REGEX, "--out", str(BASE)]
        r = self.run_cli(args)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("base", r.stderr)


if __name__ == "__main__":
    unittest.main()
