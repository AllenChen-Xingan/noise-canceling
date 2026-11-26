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


if __name__ == "__main__":
    unittest.main()
