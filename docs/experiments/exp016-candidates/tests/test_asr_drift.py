"""asr_drift.py 的测试：键对齐、编辑距离、三档判定，以及缓存键格式与 Rust 正本一致。"""
import re
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import asr_drift as D  # noqa: E402

ROOT = HERE.parents[3]


def k(sha, rep=1):
    return f"{sha}|qwen3-asr-plus:1.7b|zh|{rep}"


class Drift(unittest.TestCase):
    def test_identical(self):
        a = {k("s1"): "今天 开会", k("s1", 2): "今天开会", k("only-a"): "x"}
        b = {k("s1"): "今天开会", k("s1", 2): "今天开会", k("only-b"): "y"}
        r = D.drift(a, b)
        self.assertEqual((r["common_keys"], r["identical_keys"], r["verdict"]), (2, 2, "一致"))

    def test_small_and_large(self):
        a = {k("s1"): "今天开会", k("s2"): "我们讨论预算"}
        self.assertEqual(D.drift(a, {k("s1"): "今天开会", k("s2"): "我们讨论预酸"})["verdict"], "差远小于否决线")
        r = D.drift(a, {k("s1"): "今天", k("s2"): "我们讨论预算"})
        self.assertEqual((r["max_edit_per_audio"], r["audios_changed"], r["verdict"]), (2, 1, "降为参考"))
        self.assertEqual(D.drift(a, {})["verdict"], "无共同键，无法判断")

    def test_edit(self):
        self.assertEqual(D.edit("abc", "abc"), 0)
        self.assertEqual(D.edit("abc", "axc"), 1)
        self.assertEqual(D.edit("", "ab"), 2)

    def test_key_format_matches_rust(self):
        src = (ROOT / "src/eval/src/qa/asr.rs").read_text()
        self.assertIn('format!("{sha}|{MODEL}|{lang}|{rep}")', src)
        self.assertIn('pub const MODEL: &str = "qwen3-asr-plus:1.7b";', src)
        self.assertTrue(re.search(r"缓存键 = 文件 sha256 \| 模型 \| 语言 \| 第几次", src))


if __name__ == "__main__":
    unittest.main()
