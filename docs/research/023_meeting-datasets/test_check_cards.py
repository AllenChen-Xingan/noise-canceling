#!/usr/bin/env python3
"""check_cards.py 的测试。运行：python3 -m unittest test_check_cards（在本目录下）。"""
from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import check_cards as cc  # noqa: E402

FIELDS = cc.parse_fields((HERE / "card-template.md").read_text(encoding="utf-8"))
# 用固定键表测逻辑，不依赖 021 / 022 正本的当前内容
SCENES = {"cafe", "meeting-room", "home-family", "other"}
DEVICES = {"mba-builtin", "laptop-unknown", "non-laptop-phone", "non-laptop-close-talk"}

GOOD = """# 示例集

- **名称**：示例集（EX）
- **链接**：https://example.org
- **许可**：CC BY 4.0 → 结论：训练可【官方文档】
- **大小**：10 h
- **场景键**：cafe, meeting-room
- **设备键**：laptop-unknown（品牌未查到）, non-laptop-phone
- **笔记本段**：有，可按设备筛
- **干净参考**：没有
- **转写**：有（带时间戳）
- **语种**：英语
- **语体**：对话，有笑声
- **获取方式**：直接下载
- **与 D62 的关系**：真实录音
- **出处**：https://example.org【官方文档】

## 备注
无
"""


def check(text: str) -> list[str]:
    return cc.check_card(text, FIELDS, SCENES, DEVICES)


class TemplateTest(unittest.TestCase):
    def test_template_parses(self):
        self.assertEqual(len(FIELDS), 14)
        self.assertEqual(FIELDS[0], "名称")

    def test_broken_template_raises(self):
        with self.assertRaises(ValueError):
            cc.parse_fields("# 空模板\n")


class KeySourceTest(unittest.TestCase):
    def test_parse_keys_first_column_only(self):
        text = "| 键 | 说明 |\n|---|---|\n| `cafe` | 咖啡馆，见 `canteen` |\n| `other` | 兜底 |\n正文 `x`\n"
        self.assertEqual(cc.parse_keys(text), {"cafe", "other"})

    def test_parse_keys_empty_raises(self):
        with self.assertRaises(ValueError):
            cc.parse_keys("没有表\n")



def make_research(root: Path) -> Path:
    """在临时目录里造一份最小的 021 / 022 键表。"""
    (root / "021_meeting-scenes").mkdir(parents=True)
    (root / "022_mac-mic").mkdir(parents=True)
    (root / "021_meeting-scenes" / "catalog.md").write_text(
        "| 键 | 说明 |\n|---|---|\n| `cafe` | 咖啡馆 |\n", encoding="utf-8")
    (root / "022_mac-mic" / "devices.md").write_text(
        "| 键 | 说明 |\n|---|---|\n| `mba-builtin` | MBA |\n", encoding="utf-8")
    return root


class FindSourceTest(unittest.TestCase):
    def test_repo_path_first(self):
        with tempfile.TemporaryDirectory() as d:
            repo = make_research(Path(d) / "repo")
            got = cc.find_key_source("场景键", research=repo, env="")
            self.assertEqual(got, repo / "021_meeting-scenes" / "catalog.md")

    def test_env_fallback(self):
        with tempfile.TemporaryDirectory() as d:
            empty = Path(d) / "empty"
            empty.mkdir()
            other = make_research(Path(d) / "other")
            got = cc.find_key_source("设备键", research=empty, env=f"/nonexistent:{other}")
            self.assertEqual(got, other / "022_mac-mic" / "devices.md")

    def test_missing_raises(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(FileNotFoundError) as cm:
                cc.find_key_source("场景键", research=Path(d), env="")
            self.assertIn(cc.KEYS_ENV, str(cm.exception))


class CardTest(unittest.TestCase):
    def test_good_card_passes(self):
        self.assertEqual(check(GOOD), [])

    def test_missing_field(self):
        errs = check(GOOD.replace("- **转写**：有（带时间戳）\n", ""))
        self.assertTrue(any("缺字段「转写」" in e for e in errs))

    def test_empty_field(self):
        errs = check(GOOD.replace("- **语种**：英语", "- **语种**："))
        self.assertTrue(any("为空" in e for e in errs))

    def test_order(self):
        swapped = GOOD.replace("- **语种**：英语\n- **语体**：对话，有笑声",
                               "- **语体**：对话，有笑声\n- **语种**：英语")
        self.assertIn("字段顺序与模板不一致", check(swapped))

    def test_unknown_scene_key(self):
        errs = check(GOOD.replace("cafe, meeting-room", "cafe, kitchen"))
        self.assertTrue(any("`kitchen`" in e for e in errs))

    def test_unknown_device_key(self):
        errs = check(GOOD.replace("laptop-unknown（品牌未查到）, non-laptop-phone", "macbook"))
        self.assertTrue(any("`macbook`" in e for e in errs))

    def test_license_needs_verdict(self):
        errs = check(GOOD.replace("→ 结论：训练可", ""))
        self.assertTrue(any("结论" in e for e in errs))

    def test_source_needs_tag(self):
        errs = check(GOOD.replace("https://example.org【官方文档】", "https://example.org"))
        self.assertTrue(any("来源标签" in e for e in errs))

    def test_duplicate_and_extra(self):
        errs = check(GOOD.replace("- **语种**：英语", "- **语种**：英语\n- **语种**：中文\n- **作者**：某人"))
        self.assertTrue(any("字段重复「语种」" in e for e in errs))
        self.assertTrue(any("未知字段「作者」" in e for e in errs))

    def test_fields_after_notes_ignored(self):
        self.assertEqual(check(GOOD + "\n## 补充（路②）\n- **场景键**：kitchen\n"), [])

    def test_unknown_value_allowed(self):
        self.assertEqual(check(GOOD.replace("cafe, meeting-room", "未查到")), [])


class SplitKeysTest(unittest.TestCase):
    def test_nested_parens_and_tags(self):
        v = "phone, unknown（说话人「a (i.e. b) c」）【论文】, close-talk【沿用 014，二手】"
        self.assertEqual(cc.split_keys(v), ["phone", "unknown", "close-talk"])


class AddendumTest(unittest.TestCase):
    def test_bad_key_in_addendum(self):
        text = "### x.md\n- **场景键**：cafe\n- **设备键**：macbook（注）\n"
        errs = cc.check_addendum(text, SCENES, DEVICES)
        self.assertEqual(len(errs), 1)
        self.assertIn("`macbook`", errs[0])

    def test_good_addendum(self):
        text = "- **场景键**：home-family（电视）【论文】\n- **设备键**：未查到\n"
        self.assertEqual(cc.check_addendum(text, SCENES, DEVICES), [])


class CliTest(unittest.TestCase):
    def test_exit_codes(self):
        with tempfile.TemporaryDirectory() as d:
            Path(d, "ok.md").write_text(GOOD, encoding="utf-8")
            keys = make_research(Path(d) / "keys")
            fixture_keys = "| `cafe` |\n| `meeting-room` |\n"
            (keys / "021_meeting-scenes" / "catalog.md").write_text(fixture_keys, encoding="utf-8")
            (keys / "022_mac-mic" / "devices.md").write_text(
                "| `laptop-unknown` |\n| `non-laptop-phone` |\n", encoding="utf-8")
            env = {**os.environ, cc.KEYS_ENV: str(keys)}
            run = lambda: subprocess.run([sys.executable, str(HERE / "check_cards.py"), d],
                                         capture_output=True, text=True, env=env)
            self.assertEqual(run().returncode, 0)
            Path(d, "bad.md").write_text("# 坏卡\n", encoding="utf-8")
            r = run()
            self.assertEqual(r.returncode, 1)
            self.assertIn("bad.md", r.stdout)


if __name__ == "__main__":
    unittest.main()
