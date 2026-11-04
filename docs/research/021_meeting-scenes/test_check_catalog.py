"""check_catalog.py 的测试：真实四份文件必须全过；每条规则各造一个会红的反例。

只在内存里改文本或写到临时目录，不碰仓里的文件。
用法：python3 docs/research/021_meeting-scenes/test_check_catalog.py
"""
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import check_catalog as cc  # noqa: E402

CAT = (HERE / "catalog.md").read_text(encoding="utf-8")
COV = (HERE / "coverage.md").read_text(encoding="utf-8")
SRC = (HERE / "sources.md").read_text(encoding="utf-8")
DEC = (HERE / "decision.md").read_text(encoding="utf-8")


def codes(problems):
    return {p.split()[0] for p in problems}


def first_row(text, key):
    return next(l for l in text.splitlines() if l.startswith(f"| `{key}` |"))


class RealFiles(unittest.TestCase):
    def test_all_pass(self):
        keys, problems = cc.run(HERE)
        self.assertEqual(problems, [])
        self.assertIn("other", keys)
        self.assertEqual(len(keys), 16)

    def test_keys_match_mtgdata_reading(self):
        # mtgdata 的读法：每行以 | `键` | 开头，取第一列
        mt = [l.split("`")[1] for l in CAT.splitlines() if l.startswith("| `")]
        keys, _ = cc.check_catalog(CAT)
        self.assertEqual(mt, keys)


class Catalog(unittest.TestCase):
    def test_duplicate_key(self):
        row = first_row(CAT, "cafe")
        _, p = cc.check_catalog(CAT.replace(row, row + "\n" + row))
        self.assertIn("K2", codes(p))

    def test_bad_key_format(self):
        _, p = cc.check_catalog(CAT.replace("| `cafe` |", "| `Cafe_1` |"))
        self.assertIn("K2", codes(p))

    def test_missing_other(self):
        _, p = cc.check_catalog(CAT.replace(first_row(CAT, "other") + "\n", ""))
        self.assertIn("K2", codes(p))

    def test_empty_cell(self):
        row = first_row(CAT, "hotel-room")
        bad = row.replace("| 未查到 |", "|  |", 1)
        self.assertNotEqual(bad, row)
        _, p = cc.check_catalog(CAT.replace(row, bad))
        self.assertIn("K3", codes(p))

    def test_missing_column(self):
        row = first_row(CAT, "hotel-room")
        bad = row.rsplit("|", 2)[0] + "|"
        _, p = cc.check_catalog(CAT.replace(row, bad))
        self.assertIn("K3", codes(p))

    def test_number_without_source(self):
        row = first_row(CAT, "hotel-room")
        bad = row.replace("客房环境声约 28–32 dBA [S50]", "客房环境声约 28–32 dBA")
        self.assertNotEqual(bad, row)
        _, p = cc.check_catalog(CAT.replace(row, bad))
        self.assertIn("K4", codes(p))

    def test_dataset_name_digits_not_a_quantity(self):
        # 数据集名里的数字（MISP2021）不算要出处的数
        self.assertIsNone(cc.QTY_RE.search("MISP2021（电视房聊天）、CHiME-5/6"))
        self.assertIsNotNone(cc.QTY_RE.search("约 0.3 s / 0.45 s"))
        self.assertIsNotNone(cc.QTY_RE.search("家 29.1% 工时"))

    def test_backtick_row_outside_main_table(self):
        bad = CAT + "\n\n| 别的表 | x |\n|---|---|\n| `cafe` | 重复读到 |\n"
        _, p = cc.check_catalog(bad)
        self.assertIn("K1", codes(p))


class Sources(unittest.TestCase):
    def test_unknown_ref(self):
        p = cc.check_sources(SRC, {"catalog.md": CAT + " [S99]"})
        self.assertIn("K5", codes(p))

    def test_duplicate_def(self):
        row = next(l for l in SRC.splitlines() if l.startswith("| S01 |"))
        p = cc.check_sources(SRC + "\n" + row + "\n", {})
        self.assertIn("K5", codes(p))

    def test_sources_self_refs_checked_via_run(self):
        # sources.md 自己正文里引用的编号也要查（第 1 轮 codex #5）
        with tempfile.TemporaryDirectory() as d:
            for n, t in (("catalog.md", CAT), ("coverage.md", COV),
                         ("decision.md", DEC), ("sources.md", SRC + "\n见 [S99]\n")):
                (Path(d) / n).write_text(t, encoding="utf-8")
            _, p = cc.run(d)
        self.assertIn("K5", codes(p))

    def test_decision_refs_checked(self):
        p = cc.check_sources(SRC, {"decision.md": DEC.replace("[S24]", "[S98]", 1)})
        self.assertIn("K5", codes(p))


class Coverage(unittest.TestCase):
    keys, _ = cc.check_catalog(CAT)

    def row(self, key):
        return next(l for l in COV.splitlines() if l.startswith(f"| {key} |"))

    def test_wrong_empty_count(self):
        row = self.row("phone-booth")
        bad = row.rsplit("| 1 |", 1)[0] + "| 2 |"
        p = cc.check_coverage(COV.replace(row, bad), self.keys)
        self.assertIn("K6", codes(p))

    def test_duplicate_place_row(self):
        # 复制粘贴多出一行同名地点（第 1 轮 codex #6）
        row = self.row("cafe")
        p = cc.check_coverage(COV.replace(row, row + "\n" + row), self.keys)
        self.assertIn("K6", codes(p))

    def test_missing_place(self):
        p = cc.check_coverage(COV.replace(self.row("cowork") + "\n", ""), self.keys)
        self.assertIn("K6", codes(p))

    def test_extra_place(self):
        p = cc.check_coverage(COV, self.keys[:-2] + ["other"])
        self.assertIn("K6", codes(p))

    def test_backtick_key_in_coverage(self):
        row = self.row("cowork")
        p = cc.check_coverage(COV.replace(row, row.replace("| cowork |", "| `cowork` |")), self.keys)
        self.assertIn("K6", codes(p))


class Claims(unittest.TestCase):
    texts = {"coverage.md": COV, "decision.md": DEC}

    def test_real_claims_match(self):
        self.assertEqual(cc.check_claims(self.texts), [])

    def test_total_drift_in_decision(self):
        # 改了矩阵、忘了改 decision 里的总数（第 3 轮 codex 起因：总数在三处各写一遍）
        empty, _ = cc.matrix_stats(COV)
        bad = DEC.replace(f"有 {empty} 个格子关键", f"有 {empty + 2} 个格子关键")
        self.assertIn("K7", codes(cc.check_claims({**self.texts, "decision.md": bad})))

    def test_matrix_change_without_claim_update(self):
        row = next(l for l in COV.splitlines() if l.startswith("| phone-booth |"))
        bad = row.replace("| · | · | · |", "| ○ 新空格 | · | · |", 1)
        self.assertNotEqual(bad, row)
        self.assertIn("K7", codes(cc.check_claims({**self.texts, "coverage.md": COV.replace(row, bad)})))

    def test_no_full_column_count(self):
        _, no_full = cc.matrix_stats(COV)
        bad = COV.replace(f"一个 ● 都没有的因素有 {no_full} 个", f"一个 ● 都没有的因素有 {no_full + 1} 个")
        self.assertIn("K7", codes(cc.check_claims({**self.texts, "coverage.md": bad})))


class Cli(unittest.TestCase):
    def test_exit_code_on_broken_copy(self):
        with tempfile.TemporaryDirectory() as d:
            for n, t in (("catalog.md", CAT + " [S97]"), ("coverage.md", COV),
                         ("decision.md", DEC), ("sources.md", SRC)):
                (Path(d) / n).write_text(t, encoding="utf-8")
            self.assertEqual(cc.main(["x", d]), 1)
        self.assertEqual(cc.main(["x", str(HERE)]), 0)


if __name__ == "__main__":
    unittest.main(verbosity=1)
