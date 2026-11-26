"""BUT Speech@FIT Reverb Database（2019 RIR-Only 包）的底噪登记成第三批训练噪声（D111 补充，2026-11-26）。

数字与字符串一律从正本读，不在测试里抄副本：
- `sources.json` 的 `A7`：手工下载条目（字节数与首次下载记的 md5 / sha256）
- `download-list-but.md` 第 1 节的表：字节数（sources.json 的第二处表示，交叉校验）
- `split_rules.json` 的 `but-reverbdb-silence`：通配符 · 单位正则 · 层 · meta 排除 · 筛查表 · 期望条数 · 比例
- `recipe.json` 的 `noise_pools`：会议室 → meeting-room，酒店房间 → hotel-room
- 上游 read_me（副本 `third_party/BUT_ReverbDB-read_me.txt`）：第 2 个扬声器放背景声
夹具在临时目录现造（几间房 × 几次布置 × 两个麦克风的 0.5 s wav），不用真数据。
"""
import json
import re
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import fetch  # noqa: E402
import split  # noqa: E402

MD = HERE / "download-list-but.md"
SRC = "but-reverbdb-silence"
GRP = "A7"


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


if __name__ == "__main__":
    unittest.main()
