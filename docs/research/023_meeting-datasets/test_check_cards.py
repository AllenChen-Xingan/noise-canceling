#!/usr/bin/env python3
"""check_cards.py 的测试。运行：python3 -m unittest test_check_cards（在本目录下）。"""
from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import check_cards as cc  # noqa: E402

FIELDS, SCENES, DEVICES = cc.parse_template((HERE / "card-template.md").read_text(encoding="utf-8"))

GOOD = """# 示例集

- **名称**：示例集（EX）
- **链接**：https://example.org
- **许可**：CC BY 4.0 → 结论：训练可【官方文档】
- **大小**：10 h
- **场景键**：cafe, meeting-room
- **设备键**：laptop-other（品牌未查到）, phone
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
        self.assertIn("cafe", SCENES)
        self.assertIn("mba-builtin", DEVICES)

    def test_broken_template_raises(self):
        with self.assertRaises(ValueError):
            cc.parse_template("# 空模板\n")


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
        errs = check(GOOD.replace("laptop-other（品牌未查到）, phone", "macbook"))
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


class CliTest(unittest.TestCase):
    def test_exit_codes(self):
        with tempfile.TemporaryDirectory() as d:
            Path(d, "ok.md").write_text(GOOD, encoding="utf-8")
            run = lambda: subprocess.run([sys.executable, str(HERE / "check_cards.py"), d],
                                         capture_output=True, text=True)
            self.assertEqual(run().returncode, 0)
            Path(d, "bad.md").write_text("# 坏卡\n", encoding="utf-8")
            r = run()
            self.assertEqual(r.returncode, 1)
            self.assertIn("bad.md", r.stdout)


if __name__ == "__main__":
    unittest.main()
