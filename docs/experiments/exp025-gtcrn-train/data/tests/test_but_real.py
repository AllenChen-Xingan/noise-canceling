"""真规则与仓里那份筛查表的核对（第四批：加三间办公室 · C236 不用 · 会议室池改办公室底噪）。

从 `tests/test_but.py` 分出来的（那个文件超 600 行前搬出来，覆盖范围不减）：
- `RealRule`：`split_rules.json` 里 `but-reverbdb-silence` 的通配符 · 期望条数 · 文字与决策对得上；顶层 `未建表` 那一行
- `RealScreen`：仓里那份筛查表 1270 行（C236 620 + Room112 248 + 三间办公室 402）；办公室 402 条与两份原始证据逐字节对得上

数字与字符串一律从正本读（规则 json · 筛查表 · 原始证据），不在测试里抄第二份；只有「上游数出来多少条」这类
一次性事实写成常数，并在注释里写清怎么来的。夹具（`tests/fixtures_but.py`）与共用函数（`tests/test_but.py`）都现造。
"""
import fnmatch
import json
import re
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "tests"))
import fixtures_but  # noqa: E402
import screen_tone  # noqa: E402
import split  # noqa: E402

SRC = "but-reverbdb-silence"
SCREEN = HERE / "screen" / "but-reverbdb-silence.jsonl"
SCREEN_RAW = HERE / "screen" / "office-3mic-screen.jsonl"  # screen.py 的原始输出（证据，不改写）
TONE_RAW = HERE / "screen" / "office-3mic-tone.jsonl"  # diagnose.py --tone 的原始输出（证据，不改写）
ROOMS = (fixtures_but.ROOM112, *fixtures_but.OFFICES)


def rule():
    return split.load_rules()["sources"][SRC]


def is_office(path):
    return path.split("/")[2] in fixtures_but.OFFICES


class RealRule(unittest.TestCase):
    """真规则：Room112 全部麦克风 + 三间办公室只取 01 · 11 · 21；C236 不用（D114）。"""

    def setUp(self):
        self.r = rule()

    def test_globs_cover_four_rooms_and_never_c236(self):
        g = self.r["glob"]
        self.assertIsInstance(g, list)
        self.assertEqual(len(g), 4, "四间房四条通配符：Room112 + L207 + L212 + Q301")
        text = json.dumps(g, ensure_ascii=False)
        self.assertNotIn(fixtures_but.C236, text)
        for room in ROOMS:
            self.assertIn(room, text)

    def test_office_globs_take_only_mics_01_11_21(self):
        """三间办公室的通配符写死 `[012]1`：31 路麦克风只取 01 · 11 · 21（同一段声音，取 3 路够）。"""
        office = [g for g in self.r["glob"] if any(room in g for room in fixtures_but.OFFICES)]
        self.assertEqual(len(office), 3)
        for g in office:
            self.assertIn("/[012]1/silence/*.wav", g)
            # `[012]1` 按 glob 的字面意思只认 01 · 11 · 21：拿 02 / 31 试一遍
            head = g.replace("[012]1", "{}")
            self.assertTrue(fnmatch.fnmatch(head.format("01"), g))
            for mic in fixtures_but.OFFICE_MICS:
                self.assertEqual(fnmatch.fnmatch(head.format(mic), g), mic in fixtures_but.OFFICE_PICKED, mic)

    def test_hotel_room_glob_keeps_every_mic(self):
        hotel = [g for g in self.r["glob"] if fixtures_but.ROOM112 in g]
        self.assertEqual(len(hotel), 1)
        self.assertIn("/MicID*/*/*/silence/*.wav", hotel[0])

    def test_strata_are_the_four_rooms(self):
        """每间房一个层：四间房 = 四个层，各自至少一个开发份单位（min_dev_per_stratum=1）。"""
        units = {f"{room}/SpkID01_20171223_S" for room in ROOMS}
        self.assertEqual(set(split.unit_strata(units, self.r).values()), set(ROOMS))
        self.assertEqual(self.r["min_dev_per_stratum"], 1)

    def test_src_names_rooms_and_decisions(self):
        self.assertIn("Room112", self.r["expect"]["src"])
        self.assertIn("480", self.r["expect"]["src"])  # 办公室 480 个文件（3 间 × 3 个麦克风位置）
        self.assertIn("meeting-room", self.r["src"])
        self.assertIn("hotel-room", self.r["src"])
        self.assertIn("D114", self.r["src"])  # C236 为什么不用
        self.assertIn("01 · 11 · 21", self.r["src"])  # 只取哪几个麦克风

    def test_expect_files_is_728(self):
        """Room112 248 + 三间办公室 480（480 是 meta 排除之前数的，78 个收音机布置还没减）。"""
        self.assertEqual(self.r["expect"]["files"], 248 + 480)

    def test_dev_note_says_four_strata(self):
        note = self.r["_开发份"]
        self.assertIn("四个层", note)
        self.assertIn("min_dev_per_stratum", note)
        self.assertIn("同一间房", note)  # 每间房只有一间：开发份只测同房不同场次

    def test_marked_pending_until_the_table_comes_back(self):
        """规则改了，音频在云上：本机切不了，顶层 `未建表` 记着，表带回进仓后删那一行。"""
        marker = split.load_rules()["未建表"][SRC]
        self.assertIn("旧表", marker)
        self.assertIn("Room112", marker)
        self.assertIn("第四批", marker)
        self.assertIn("删这一行", marker)


class RealScreen(unittest.TestCase):
    """仓里那份筛查表 1270 行：C236 620 + Room112 248 + 三间办公室 402。每条都有结论；办公室那 402 条与两份原始证据对得上。"""

    def setUp(self):
        self.lines = [ln for ln in SCREEN.read_text(encoding="utf-8").splitlines() if ln.strip()]
        self.rows = [json.loads(ln) for ln in self.lines]
        self.office = [r for r in self.rows if is_office(r["path"])]
        self.globs = [fnmatch.translate(g) for g in rule()["glob"]]

    def matched(self, path):
        return any(re.match(t, path) for t in self.globs)

    def test_line_count(self):
        self.assertEqual(len(self.lines), 1270)  # C236 620 + Room112 248 + 办公室 402
        self.assertEqual(len({r["path"] for r in self.rows}), len(self.rows), "同一文件出现两次")

    def test_every_row_has_a_verdict(self):
        for r in self.rows:
            self.assertIn(r.get("verdict"), ("keep", "drop"), r.get("path"))

    def test_room_rows_count(self):
        """酒店房间 248 条、办公室 402 条；办公室 402 + 收音机排除的 78 = 期望条数里的那 480。"""
        hotel = [r for r in self.rows if fixtures_but.ROOM112 in r["path"]]
        self.assertEqual(len(hotel), 248)
        self.assertEqual(len(self.office), 402)
        self.assertEqual(len(hotel) + len(self.office) + 78, rule()["expect"]["files"])

    def test_every_room_row_matched_by_a_glob(self):
        for r in self.rows:
            if r["path"].split("/")[2] in ROOMS:
                self.assertTrue(self.matched(r["path"]), r["path"])

    def test_globs_match_no_c236_row(self):
        """C236 不用（D114）：真规则的通配符一条也不许认领它的行。"""
        c236 = [r for r in self.rows if fixtures_but.C236 in r["path"]]
        self.assertTrue(c236, "筛查表里该有 C236 的行（历史证据），只是没人认领")
        for r in c236:
            self.assertFalse(self.matched(r["path"]), r["path"])

    def test_office_rows_are_mics_01_11_21_only(self):
        self.assertEqual(len(self.office), 402)
        self.assertEqual({r["path"].split("/")[5] for r in self.office}, set(fixtures_but.OFFICE_PICKED))

    def test_every_office_row_has_tone_where(self):
        for r in self.office:
            self.assertIn("tone_where", r.get("measures", {}), r["path"])

    def test_office_rows_with_a_tone_are_drop(self):
        """单音判据的正本是 screen_rules.json 的 tone_check.drop_where：命中就一条都不能留在训练里。"""
        drop_where = set(screen_tone.load_drop_where())
        toned = [r for r in self.office if r["measures"]["tone_where"] != "none"]
        self.assertTrue(toned, "402 条里该有带单音的行，否则这条不是空检查")
        for r in toned:
            self.assertIn(r["measures"]["tone_where"], drop_where, r["path"])
            self.assertEqual(r["verdict"], "drop", r["path"])

    def test_office_keep_count_is_375(self):
        self.assertEqual(sum(1 for r in self.office if r["verdict"] == "keep"), 375)

    def test_merge_reproduces_office_rows_byte_exactly(self):
        """两份原始证据重跑 screen_tone.merge，结果与仓里这 402 条逐字节相同（揭盲表之坑：原始证据不改写）。"""
        res = screen_tone.merge(screen_tone.read_jsonl(SCREEN_RAW), screen_tone.read_jsonl(TONE_RAW),
                                screen_tone.load_drop_where())
        got = [json.dumps(r, ensure_ascii=False) for r in res["rows"]]
        want = [ln for ln in self.lines if is_office(json.loads(ln)["path"])]
        self.assertEqual(len(got), 402)
        self.assertEqual(got, want)
        self.assertEqual((res["total"], res["keep"], res["drop"]), (402, 375, 27))


if __name__ == "__main__":
    unittest.main()
