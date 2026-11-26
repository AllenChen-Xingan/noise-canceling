"""BUT Speech@FIT Reverb Database（2019 RIR-Only 包）的底噪登记成第三批训练噪声（D111 补充，2026-11-26）。

数字与字符串一律从正本读，不在测试里抄副本：
- `sources.json` 的 `A7`：手工下载条目（字节数与首次下载记的 md5 / sha256）
- `download-list-but.md` 第 1 节的表：字节数（sources.json 的第二处表示，交叉校验）
- `split_rules.json` 的 `but-reverbdb-silence`：通配符 · 单位正则 · 层 · meta 排除 · 筛查表 · 期望条数 · 比例
- `recipe.json` 的 `noise_pools`：会议室 → meeting-room，酒店房间 → hotel-room
- 上游 read_me（副本 `third_party/BUT_ReverbDB-read_me.txt`）：第 2 个扬声器放背景声
夹具在临时目录现造（几间房 × 几次布置 × 两个麦克风的 0.5 s wav），不用真数据。
"""
import copy
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "tests"))
import fetch  # noqa: E402
import fixtures_but  # noqa: E402
import split  # noqa: E402

MD = HERE / "download-list-but.md"
SRC = "but-reverbdb-silence"
GRP = "A7"
META = {"file": "../../spk_meta.txt", "regex": "^\\$EnvSpk2", "why": "read_me：第 2 个扬声器放的是背景声（如收音机）"}


def feat_rule(**over):
    """带真规则里那几条通用字段的最小规则：只看一条特性时用它，不依赖 split_rules.json 已改到哪一步。"""
    r = {"group": GRP, "unit": "file", "glob": [fixtures_but.ROOT + "/**/silence/*.wav"],
         "frac": {"train": 0.9, "dev": 0.1, "final": 0.0, "d63": 0.0}}
    r.update(over)
    return r


def group():
    return fetch.load_sources()[GRP]


def rule():
    return split.load_rules()["sources"][SRC]


def md_bytes():
    """download-list-but.md 第 1 节表：文件名 → 字节数（去掉千位分隔符）。"""
    sec = MD.read_text().split("## 1.")[1].split("## 2.")[0]
    return {n: int(b.replace(",", "")) for n, b in re.findall(r"^\| (\S+\.tgz) \| \S+ \| ([\d,]+) \|", sec, re.M)}


class Sources(unittest.TestCase):
    """A7 是手工下载条目（上游没给校验码，首次下载由 cloudrun 记 md5 / sha256）。"""

    def setUp(self):
        self.g = group()
        self.f = self.g["files"][0]

    def test_one_archive_only(self):
        self.assertEqual(len(self.g["files"]), 1)
        self.assertEqual(self.f["name"], "BUT_ReverbDB_rel_19_06_RIR-Only.tgz")
        self.assertEqual(self.f["url"], "http://merlin.fit.vutbr.cz/ReverbDB/BUT_ReverbDB_rel_19_06_RIR-Only.tgz")

    def test_bytes_match_download_list(self):
        got = md_bytes()
        self.assertEqual(set(got), {self.f["name"]})
        self.assertEqual(self.f["bytes"], got[self.f["name"]])

    def test_hashes_are_hex_of_right_length(self):
        self.assertRegex(self.f["md5"], r"^[0-9a-f]{32}$")
        self.assertRegex(self.f["sha256"], r"^[0-9a-f]{64}$")

    def test_role_license_attribution(self):
        self.assertEqual(self.g["role"], "train")
        self.assertTrue(self.g["train_ok"])
        self.assertEqual(self.g["license"], "Apache-2.0")
        self.assertIn("D111", self.g["license_src"])
        self.assertIn("third_party/BUT_ReverbDB-read_me.txt", self.g["license_src"])
        self.assertIn("Copyright 2018 Brno University of Technology", self.g["attribution"])
        self.assertIn("IEEE JSTSP", self.g["attribution"])

    def test_manual_says_bytes_only_and_what_not_to_unpack(self):
        m = self.g["manual"]
        self.assertIn("md5", m)
        self.assertIn("sha256", m)
        self.assertIn("D57", m)  # 重放 LibriSpeech 的包不下
        self.assertIn("download-list-but.md", m)


class Meta(unittest.TestCase):
    """`exclude_if_meta`：上游 read_me 说第 2 个扬声器放背景声，那种布置的底噪不许当训练噪声。"""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.files = fixtures_but.make_tree(self.root)
        self.r = feat_rule(exclude_if_meta=META)

    def tearDown(self):
        self.tmp.cleanup()

    def run_src(self):
        d = {}
        rows = split.split_source(self.root, SRC, self.r, split.load_rules()["seed"], split.load_rules()["splits"],
                                  workers=2, excluded=d)
        return rows, d

    def test_radio_setup_excluded_and_counted(self):
        rows, d = self.run_src()
        radio = set(fixtures_but.radio_files())
        self.assertTrue(radio, "夹具里要有一次放收音机的布置")
        self.assertEqual({r["path"] for r in rows}, set(self.files) - radio)
        self.assertEqual(d["excluded_meta"], len(radio))

    def test_clean_setups_kept(self):
        """`$EnvSpk1` 不算命中：只有第 2 个及以后的扬声器才是背景声源。"""
        rows, _ = self.run_src()
        keep = {r["path"] for r in rows}
        for p in self.files:
            if p not in set(fixtures_but.radio_files()):
                self.assertIn(p, keep, p)

    def test_missing_meta_stops(self):
        """没有 spk_meta.txt 就不能判：静默留着会把收音机当噪声（绝不悄悄放过）。"""
        rel = fixtures_but.radio_files()[0]
        fixtures_but.meta_path(self.root, rel).unlink()
        with self.assertRaises(SystemExit) as cm:
            self.run_src()
        self.assertIn("spk_meta.txt", str(cm.exception))

    def test_write_records_excluded_meta(self):
        """摘要（进仓的那份）里要看得见排除了多少个文件。"""
        rows, d = self.run_src()
        out = self.root / "splits"
        s = split.write(SRC, self.r, split.load_rules()["seed"], rows, out, d)
        self.assertEqual(s["excluded_meta"], len(fixtures_but.radio_files()))
        self.assertEqual(json.loads((out / f"{SRC}.json").read_text())["excluded_meta"], s["excluded_meta"])

    def test_no_field_no_change(self):
        """没有这个字段的规则行为不变（老来源不受影响）。"""
        plain, d = feat_rule(), {}
        got = split.split_source(self.root, SRC, plain, split.load_rules()["seed"], split.load_rules()["splits"],
                                 workers=2, excluded=d)
        self.assertEqual(len(got), len(self.files))
        self.assertEqual(d.get("excluded_meta", 0), 0)

class Screen(unittest.TestCase):
    """`screen_file`：screen.py 的 jsonl 里 verdict=drop 的不要；清单里每个文件都得有结论。"""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.files = fixtures_but.make_tree(self.root)
        self.r = feat_rule(exclude_if_meta=META, screen_file=str(self.root / "screen.jsonl"))
        self.kept = [p for p in self.files if p not in set(fixtures_but.radio_files())]

    def tearDown(self):
        self.tmp.cleanup()

    def run_src(self):
        d = {}
        rows = split.split_source(self.root, SRC, self.r, split.load_rules()["seed"], split.load_rules()["splits"],
                                  workers=2, excluded=d)
        return rows, d

    def test_drop_excluded_and_counted(self):
        dropped = self.kept[:2]
        fixtures_but.write_screen(self.root / "screen.jsonl", self.kept, drop=dropped)
        rows, d = self.run_src()
        self.assertEqual({r["path"] for r in rows}, set(self.kept) - set(dropped))
        self.assertEqual(d["excluded_screen"], len(dropped))

    def test_radio_files_need_no_verdict(self):
        """顺序是先 meta 后筛查：被 meta 排掉的文件不进筛查表，也不报「少了一条」。"""
        fixtures_but.write_screen(self.root / "screen.jsonl", self.kept)  # 不含收音机那一次布置
        rows, d = self.run_src()
        self.assertEqual(len(rows), len(self.kept))
        self.assertEqual(d["excluded_meta"], len(fixtures_but.radio_files()))
        self.assertEqual(d["excluded_screen"], 0)

    def test_file_missing_from_screen_stops(self):
        fixtures_but.write_screen(self.root / "screen.jsonl", self.kept[1:])
        with self.assertRaises(SystemExit) as cm:
            self.run_src()
        self.assertIn(self.kept[0], str(cm.exception))

    def test_missing_screen_file_stops(self):
        self.r["screen_file"] = str(self.root / "screen" / "nope.jsonl")
        with self.assertRaises(SystemExit) as cm:
            self.run_src()
        self.assertIn("screen.py", str(cm.exception))

    def test_bad_verdict_stops(self):
        p = fixtures_but.write_screen(self.root / "screen.jsonl", self.kept)
        p.write_text(p.read_text().replace('"keep"', '"maybe"', 1))
        with self.assertRaises(SystemExit):
            self.run_src()

class Strata(unittest.TestCase):
    """`stratum_unit_regex` + `min_dev_per_stratum`：按房间分层切，每层开发份至少一个单位。"""

    UNIT_REGEX = [{"match": "^train-raw/but-reverbdb/([^/]+)/MicID\\d+/([^/]+)/", "unit": "\\1/\\2",
                   "why": "房间 + 扬声器布置：同一次布置的 31 个麦克风同时录的是同一段底噪，必须同进一份"}]

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.files = fixtures_but.make_tree(self.root)
        self.r = feat_rule(unit_regex=self.UNIT_REGEX, unit_regex_required=True, exclude_if_meta=META,
                           stratum_unit_regex="^([^/]+)/", min_dev_per_stratum=1)

    def tearDown(self):
        self.tmp.cleanup()

    def run_src(self, rule=None):
        return split.split_source(self.root, SRC, rule or self.r, split.load_rules()["seed"],
                                  split.load_rules()["splits"], workers=2)

    def units(self):
        radio = set(fixtures_but.radio_files())
        return sorted({"/".join(p.split("/")[2:3] + p.split("/")[4:5]) for p in self.files if p not in radio})

    def test_unit_and_stratum_from_path(self):
        rows = self.run_src()
        units = {r["unit"] for r in rows}
        self.assertEqual(units, set(self.units()))
        st = split.unit_strata(units, self.r)
        self.assertEqual(set(st.values()), {fixtures_but.C236, fixtures_but.ROOM112})
        self.assertEqual(st[f"{fixtures_but.C236}/SpkID01_20190503_S"], fixtures_but.C236)

    def test_each_room_stratum_has_a_dev_unit(self):
        rows = self.run_src()
        dev = {r["unit"].split("/")[0] for r in rows if r["split"] == "dev"}
        self.assertEqual(dev, {fixtures_but.C236, fixtures_but.ROOM112})

    def test_without_min_dev_the_room_has_no_dev(self):
        """不是空检查：每个房间只有 2 个单位时，9:1 最大余数法会把开发份都挤掉。"""
        r = copy.deepcopy(self.r)
        r.pop("min_dev_per_stratum")
        rows = self.run_src(r)
        self.assertEqual({r0["split"] for r0 in rows}, {"train"})

    def test_min_dev_takes_highest_rank_from_train(self):
        """确定性：从训练份拿 rank_key 最大的补进开发份。"""
        rows = self.run_src()
        got = {r["unit"] for r in rows if r["split"] == "dev"}
        want = set()
        for room in (fixtures_but.C236, fixtures_but.ROOM112):
            us = [u for u in self.units() if u.startswith(room + "/")]
            want.add(max(us, key=lambda u: split.rank_key(split.load_rules()["seed"], SRC, u)))
        self.assertEqual(got, want)

    def test_no_unit_in_two_splits(self):
        by = {}
        for r in self.run_src():
            by.setdefault(r["unit"], set()).add(r["split"])
        self.assertTrue(all(len(v) == 1 for v in by.values()), by)

    def test_deterministic(self):
        a = [(r["path"], r["split"]) for r in self.run_src()]
        b = [(r["path"], r["split"]) for r in self.run_src()]
        self.assertEqual(a, b)

    def test_unrecognised_unit_stops(self):
        r = copy.deepcopy(self.r)
        r["stratum_unit_regex"] = "^(nomatch)/([^/]+)/"
        with self.assertRaises(SystemExit):
            self.run_src(r)

class UnitFrom(unittest.TestCase):
    """#204：摘要里记下单位模板，读表的人不用再翻规则去猜单位长什么样。"""

    def summ(self, rule, name=SRC):
        return split.summary(name, rule, 1, [], b"")

    def test_present_for_regex_rules(self):
        s = self.summ(feat_rule(unit_regex=Strata.UNIT_REGEX))
        self.assertEqual(s["unit_from"], Strata.UNIT_REGEX[0]["unit"])

    def test_present_for_cochlscene(self):
        r = split.load_rules()["sources"]["cochlscene"]
        self.assertEqual(self.summ(r, "cochlscene")["unit_from"], r["unit_regex"][0]["unit"])

    def test_absent_for_speaker_rule(self):
        r = split.load_rules()["sources"]["librispeech-train-clean-100"]
        self.assertNotIn("unit_from", self.summ(r, "librispeech-train-clean-100"))

    def test_absent_for_time_rule(self):
        r = split.load_rules()["sources"]["qut-noise"]
        self.assertNotIn("unit_from", self.summ(r, "qut-noise"))

class Pools(unittest.TestCase):
    """第三批：BUT 库的会议室底噪 → meeting-room，酒店房间 → hotel-room（D111）。"""

    def setUp(self):
        self.recipe = json.loads((HERE / "recipe.json").read_text())
        self.pools = {k: v for k, v in self.recipe["noise_pools"].items() if not k.startswith("_")}

    def item(self, scene):
        items = [it for it in self.pools[scene] if it["source"] == SRC]
        self.assertEqual(len(items), 1, scene)
        return items[0]

    def test_two_pools_use_this_source(self):
        rules = split.load_rules()["sources"]
        self.assertIn(SRC, rules)
        self.assertIn(fixtures_but.C236, self.item("meeting-room")["regex"])
        self.assertIn(fixtures_but.ROOM112, self.item("hotel-room")["regex"])

    def test_regex_picks_only_its_room(self):
        p = {room: fixtures_but.wav_rel(room, "SpkID01_20190503_S", "01") for room in (fixtures_but.C236, fixtures_but.ROOM112)}
        for scene, room in (("meeting-room", fixtures_but.C236), ("hotel-room", fixtures_but.ROOM112)):
            rx = re.compile(self.item(scene)["regex"])
            for other, path in p.items():
                self.assertEqual(bool(rx.search(path)), other == room, f"{scene} 对 {other}")

    def test_note_names_third_batch(self):
        note = self.recipe["noise_pools"]["_说明"]
        self.assertIn("D111", note)
        self.assertIn("BUT", note)
        self.assertIn("同一间房", note)

    def test_target_scene_untouched(self):
        self.assertIsNone(self.recipe["knobs"]["target_scene"]["scene"])


class ScreenRules(unittest.TestCase):
    def test_hotel_room_scene(self):
        rules = json.loads((HERE / "screen_rules.json").read_text())
        sc = rules["scenes"]["hotel-room"]
        self.assertEqual(sc["top_k"], 5)
        self.assertTrue(sc["src"])
        self.assertIn("Inside, small room", sc["tags_any"])


if __name__ == "__main__":
    unittest.main()
