"""真规则与仓里两份筛查表的核对（第五批：Room112 补 250 Hz 单音检查 · 新来源名 `-b5` · 新筛查表）。

从 `tests/test_but.py` 分出来的（那个文件超 600 行前搬出来，覆盖范围不减）：
- `RealRule`：`split_rules.json` 里 `but-reverbdb-silence-b5`（第五批的 4 间房）的通配符 · 期望条数 · 文字与决策对得上；
  除 `screen_file` 与 `src` 外与 `-b4` 逐字段相同（第五批只换了筛查表）
- `Names`：三个来源名的分工——`but-reverbdb-silence` 只含 Room112，`-b4` 是第四批的 4 间房，`-b5` 是第五批；
  两个旧名字的规则一个字节都不能动（旧表 build 10aa3a9f · bc0e9e7b 在用）；`-b5` 的表已带回进仓
  （build df759bb8d606a209，顶层 `未建表` 里已经没有它）；配方与分片只看新名字
- `RealScreen`：仓里第五批那份筛查表 1270 行（C236 620 + Room112 248 + 三间办公室 402）；Room112 的 248 条由
  老表 + `screen/room112-tone.jsonl` 逐字节可复现；其余 1022 条与老表逐字节相同；Room112 keep 188 · 因单音新丢 21

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

SRC = "but-reverbdb-silence-b5"  # 第五批的 4 间房（Room112 + 三间办公室，Room112 补了单音检查）
B4 = "but-reverbdb-silence-b4"  # 第四批的 4 间房（旧表 build bc0e9e7b 在用）
OLD = "but-reverbdb-silence"  # 只含 Room112 的最早来源（旧表 build 10aa3a9f 在用）
SCREEN = HERE / "screen" / "but-reverbdb-silence-b5.jsonl"  # 第五批的筛查表（-b5 用）
SCREEN_B4 = HERE / "screen" / "but-reverbdb-silence.jsonl"  # 第四批及更早的筛查表（OLD / -b4 用，不改）
SCREEN_RAW = HERE / "screen" / "office-3mic-screen.jsonl"  # screen.py 的原始输出（证据，不改写）
TONE_RAW = HERE / "screen" / "office-3mic-tone.jsonl"  # diagnose.py --tone 的原始输出（证据，不改写）
ROOM_TONE = HERE / "screen" / "room112-tone.jsonl"  # Room112 的单音补测（证据，不改写）
ROOM112_RE = re.compile(r"/Hotel_SkalskyDvur_Room112/")
ROOMS = (fixtures_but.ROOM112, *fixtures_but.OFFICES)


def rule():
    return split.load_rules()["sources"][SRC]


def is_office(path):
    return path.split("/")[2] in fixtures_but.OFFICES


def is_room(path):
    return ROOM112_RE.search(path) is not None


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
        self.assertIn("#221", self.r["src"])  # 房间单音检查这一条 issue
        self.assertIn("21", self.r["src"])
        self.assertIn("5%", self.r["src"])
        self.assertIn("-b4", self.r["src"])  # 为什么不用旧名字
        self.assertIn("bc0e9e7b", self.r["src"])  # -b4 的表谁在用

    def test_expect_files_is_728(self):
        """Room112 248 + 三间办公室 480（480 是 meta 排除之前数的，78 个收音机布置还没减）。"""
        self.assertEqual(self.r["expect"]["files"], 248 + 480)

    def test_dev_note_says_four_strata(self):
        note = self.r["_开发份"]
        self.assertIn("四个层", note)
        self.assertIn("min_dev_per_stratum", note)
        self.assertIn("同一间房", note)  # 每间房只有一间：开发份只测同房不同场次

    def test_rule_is_b4_rule_except_screen_file_and_src(self):
        """第五批只换了筛查表：-b5 与 -b4 除 `screen_file` 与 `src` 外逐字段相同（比例 · 层 · 排除 · 期望条数都不许动）。"""
        b4 = split.load_rules()["sources"][B4]
        self.assertEqual({k: v for k, v in self.r.items() if k not in ("screen_file", "src")},
                         {k: v for k, v in b4.items() if k not in ("screen_file", "src")})
        self.assertNotEqual(self.r["screen_file"], b4["screen_file"])
        self.assertNotEqual(self.r["src"], b4["src"])


class Names(unittest.TestCase):
    """三个来源名的分工（大总管 87 2026-11-26；第五批 traindata 2026-11-26）：

    - `but-reverbdb-silence`：只含 Room112，规则一个字节都不能动（旧表 build 10aa3a9f 在用）
    - `-b4`：第四批的 4 间房，规则一个字节都不能动（旧表 build bc0e9e7b 在用）
    - `-b5`：第五批的 4 间房，只换了筛查表（Room112 补了单音检查）；云上跑完的表已带回进仓
      （build df759bb8d606a209），顶层 `未建表` 里不该再有它
    """

    def setUp(self):
        self.rules = split.load_rules()["sources"]

    def test_old_rules_match_their_tables(self):
        """两个旧名字的规则必须与仓里那两张表对得上：动它一个字节，那两份 build 的 `rule_sha256` 就红。"""
        for name in (OLD, B4):
            s, _ = split.load_split(name, HERE / "splits")
            self.assertEqual(s["source"], name)
            self.assertEqual(split.rule_sha(self.rules[name]), s["rule_sha256"],
                             f"{name} 的规则被改过：仓里那张表的 rule_sha256 对不上了")

    def test_old_rule_is_only_the_hotel_room(self):
        """旧名字只认 Room112：三间办公室一条也不许进它的通配符。"""
        text = json.dumps(self.rules[OLD], ensure_ascii=False)
        for room in fixtures_but.OFFICES:
            self.assertNotIn(room, text)
        self.assertEqual(len(self.rules[OLD]["glob"]), 1)

    def test_screen_files_are_split_by_batch(self):
        """筛查表按批次：OLD / -b4 共用老表，-b5 用新表（Room112 的行并了单音结论）。"""
        self.assertEqual(self.rules[OLD]["screen_file"], self.rules[B4]["screen_file"])
        self.assertEqual(self.rules[B4]["screen_file"], "screen/but-reverbdb-silence.jsonl")
        self.assertEqual(self.rules[SRC]["screen_file"], "screen/but-reverbdb-silence-b5.jsonl")

    def load_table(self):
        """`-b5` 的表已进仓：顶层没有 `未建表` 标记 · 能按摘要里的哈希读回 · 按当前规则切的。
        返回 (规则, 摘要, 行)。"""
        R = split.load_rules()
        self.assertNotIn(SRC, R.get("未建表", {}),
                         "顶层 `未建表` 里还有 -b5：音频已切回、表已进仓，那一行该删（表回来了却留着标记）")
        r = R["sources"][SRC]
        s, rows = split.load_split(SRC, HERE / "splits")  # 顺带核 jsonl 的 sha256
        self.assertEqual(s["rule_sha256"], split.rule_sha(r), "表是按旧规则切的")
        self.assertEqual(s["seed"], R["seed"])
        self.assertEqual(s["files"], len(rows))
        return r, s, rows

    def test_table_is_back(self):
        """表回来了：563 条 · 20 个单位 · meta 排除 78 · 筛查排除 87，且与规则里的上游条数对得上。"""
        r, s, _ = self.load_table()
        self.assertEqual(s["files"], 563)
        self.assertEqual(s["units"], 20)
        self.assertEqual(s["excluded_meta"], 78)
        self.assertEqual(s["excluded_screen"], 87)
        # 上游条数 − 两次排除 = 表里条数：三处一起动，改一处就对不上
        self.assertEqual(r["expect"]["files"] - s["excluded_meta"] - s["excluded_screen"], s["files"])

    def test_table_only_has_train_and_dev(self):
        """D106：只作训练噪声，不给终测（final）与测试份（d63）；两份的合计正好是整张表。"""
        _, s, rows = self.load_table()
        self.assertEqual(set(s["per_split"]), {"train", "dev"})
        self.assertEqual({r["split"] for r in rows}, {"train", "dev"})
        self.assertEqual(sum(v["files"] for v in s["per_split"].values()), s["files"])
        self.assertEqual(sum(v["units"] for v in s["per_split"].values()), s["units"])

    def test_every_room_has_a_dev_unit(self):
        """四间房各是一个层（`min_dev_per_stratum` = 1）：少一间的开发份，9:1 会把整类挤进训练份。"""
        _, _, rows = self.load_table()
        dev = {room_of_unit(r["unit"]) for r in rows if r["split"] == "dev"}
        self.assertEqual(dev, set(ROOMS), "有房间一个开发份单位都没有")

    def test_screen_drops_stay_out_of_the_table(self):
        """Room112 进表的正是新筛查表 keep 的那 188 条：新表判 drop 的一条也不许进（含因单音新丢的 21 条）。"""
        _, _, rows = self.load_table()
        paths = {r["path"] for r in rows}
        new = screen_verdicts(SCREEN)
        old = screen_verdicts(SCREEN_B4)
        room = {p for p in new if ROOM112_RE.search(p)}
        kept = {p for p in room if new[p] == "keep"}
        self.assertEqual(paths & room, kept, "Room112 进表的那批和新筛查表的 keep 对不上")
        self.assertEqual(len(paths & room), 188)
        # 因单音新丢的 21 条：新表 drop、老表还是 keep（老表按铁律不改，谁在用谁读它）
        newly = {p for p in room if old[p] == "keep" and new[p] == "drop"}
        self.assertEqual(len(newly), 21)
        self.assertEqual(newly & paths, set(), "因单音新丢的文件进了切分表")

    def test_tables_that_exist_are_cut_with_the_current_rule(self):
        """除 `未建表` 里的来源外，每张切分表的 rule_sha256 都要等于当前规则（旧表没被原地覆盖）。"""
        pending = set(split.load_rules().get("未建表", {}))
        for name in sorted(self.rules):
            if name in pending:
                continue
            s, _ = split.load_split(name, HERE / "splits")  # 顺带核 jsonl 的哈希
            self.assertEqual(s["rule_sha256"], split.rule_sha(self.rules[name]),
                             f"{name}：切分表不是按当前规则切的")

    def test_recipe_pools_point_at_the_new_name_only(self):
        """BUT 的两个噪声池只认新名字：指向旧名字就说明配方改漏了（旧名字没有第五批的筛查表）。"""
        recipe = json.loads((HERE / "recipe.json").read_text())
        but = [(scene, it["source"]) for scene, items in recipe["noise_pools"].items()
               if not scene.startswith("_") for it in items
               if "but-reverbdb" in it["source"]]
        self.assertEqual({scene for scene, _ in but}, {"hotel-room", "meeting-room"})
        self.assertEqual({src for _, src in but}, {SRC}, "配方里还有指向旧名字的噪声池")
        self.assertIn(SRC, recipe["noise_pools"]["_说明"])

    def test_default_sources_use_the_new_name(self):
        """分片默认来源从 recipe.json 导出：只该出现新名字（旧名字没有第五批的筛查表，切了也用不上）。"""
        names = shard_sources()
        self.assertIn(SRC, names)
        self.assertNotIn(OLD, names)
        self.assertNotIn(B4, names)

    def test_scene_notes_registers_the_new_name(self):
        """同一段声音的识别规则每个名字都要有：少一个，新名字的「不同内容」时长会按麦克风条数虚高。"""
        same = json.loads((HERE / "scene_notes.json").read_text())["same_content"]
        for name in (OLD, B4, SRC):
            self.assertEqual(same[name]["regex"], same[SRC]["regex"])
            self.assertEqual(same[name]["group"], same[SRC]["group"])
        self.assertIn("-b4", same[SRC]["why"])


def room_of_unit(unit):
    """单位写法是「房间/扬声器布置」（规则 `unit_regex` 的正本）：斜杠前那一段就是房间。"""
    return unit.split("/")[0]


def screen_verdicts(path):
    """筛查表里 路径 → 结论（keep / drop）：比对的是仓里那两份真文件，不在测试里另算一份。"""
    rows = [json.loads(ln) for ln in Path(path).read_text(encoding="utf-8").splitlines() if ln.strip()]
    return {r["path"]: r["verdict"] for r in rows}


def shard_sources():
    """分片默认来源：走正本 `shard.default_sources`（它读 recipe.json），不在测试里另算一份。"""
    import shard

    return shard.default_sources()


class RealScreen(unittest.TestCase):
    """第五批那份筛查表 1270 行：C236 620 + Room112 248 + 三间办公室 402。

    Room112 的 248 条由老表 + Room112 单音 jsonl 可逐字节复现（keep 188 · drop 60，其中因单音新丢 21）；
    办公室那 402 条与两份原始证据对得上；其余 1022 条与老表逐字节相同。
    """

    def setUp(self):
        self.text = SCREEN.read_text(encoding="utf-8")
        self.lines = [ln for ln in self.text.splitlines() if ln.strip()]
        self.rows = [json.loads(ln) for ln in self.lines]
        self.office = [r for r in self.rows if is_office(r["path"])]
        self.room = [r for r in self.rows if is_room(r["path"])]
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
        self.assertEqual(len(self.room), 248)
        self.assertEqual(len(self.office), 402)
        self.assertEqual(len(self.room) + len(self.office) + 78, rule()["expect"]["files"])

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

    def test_office_rows_still_reproduce_from_raw_evidence(self):
        """办公室 402 条与两份原始证据逐字节对得上（揭盲表之坑：原始证据不改写）。"""
        res = screen_tone.merge(screen_tone.read_jsonl(SCREEN_RAW), screen_tone.read_jsonl(TONE_RAW),
                                screen_tone.load_drop_where(), screen_tone.load_present_db())
        got = [json.dumps(r, ensure_ascii=False) for r in res["rows"]]
        want = [ln for ln in self.lines if is_office(json.loads(ln)["path"])]
        self.assertEqual(got, want)
        self.assertEqual((res["total"], res["keep"], res["drop"]), (402, 375, 27))

    def test_every_room_row_has_tone_where(self):
        for r in self.room:
            self.assertIn("tone_where", r.get("measures", {}), r["path"])

    def test_room_rows_with_a_tone_are_drop(self):
        """单音判据的正本是 screen_rules.json 的 tone_check.drop_where：命中就一条都不能留在训练里。"""
        drop_where = set(screen_tone.load_drop_where())
        toned = [r for r in self.room if r["measures"]["tone_where"] != "none"]
        self.assertEqual(len(toned), 38, "Room112 的 248 条里后来几秒有 250 Hz 单音的该是 38 条")
        for r in toned:
            self.assertIn(r["measures"]["tone_where"], drop_where, r["path"])
            self.assertEqual(r["verdict"], "drop", r["path"])

    def test_room_keep_count_and_new_drops(self):
        """老表 keep 209 条 → 第五批 keep 188 条；因单音新丢 21 条（traindata 已核）。"""
        self.assertEqual(sum(1 for r in self.room if r["verdict"] == "keep"), 188)
        self.assertEqual(len(self.room) - 188, 60)
        self.assertEqual(21 + 39, 60)  # 60 = 老表本来就 drop 的 39 + 因单音新丢的 21

    def test_room_rows_are_byte_exact_from_old_table_and_tone(self):
        """老表 + Room112 单音 jsonl 跑 screen_tone.splice：这 248 行逐字节相同（生成器的真产物测试）。"""
        base = screen_tone.read_jsonl(SCREEN_B4)
        res = screen_tone.splice(base, base, screen_tone.read_jsonl(ROOM_TONE),
                                 screen_tone.load_drop_where(), screen_tone.load_present_db(),
                                 r"/Hotel_SkalskyDvur_Room112/")
        got = [json.dumps(r, ensure_ascii=False) for r in res["rows"] if is_room(r["path"])]
        want = [ln for ln in self.lines if is_room(json.loads(ln)["path"])]
        self.assertEqual(got, want)
        self.assertEqual((res["keep"], res["tone_dropped"]), (188, 21))

    def test_non_room_lines_are_byte_identical_to_the_old_table(self):
        old = {json.loads(ln)["path"]: ln for ln in SCREEN_B4.read_text(encoding="utf-8").splitlines() if ln.strip()}
        for ln in self.lines:
            path = json.loads(ln)["path"]
            if not is_room(path):
                self.assertEqual(ln, old[path], path)

    def test_old_screen_table_is_not_rewritten(self):
        """老表（OLD / -b4 在用）一个字节都不该被第五批动过：Room112 那些行仍然没有单音结果。"""
        old_room = [json.loads(ln) for ln in SCREEN_B4.read_text(encoding="utf-8").splitlines()
                    if ln.strip() and is_room(json.loads(ln)["path"])]
        self.assertEqual(len(old_room), 248)
        self.assertFalse(any("tone_where" in r.get("measures", {}) for r in old_room))
        self.assertEqual(sum(1 for r in old_room if r["verdict"] == "keep"), 209)


if __name__ == "__main__":
    unittest.main()
