"""把 diagnose.py --tone 的单音结论并进 screen.py 的筛查结果（screen_tone.py）。

- 有单音的片段一律 drop：「单音在哪」这个名单只有一个正本，就是 `screen_rules.json` 的
  `tone_check.drop_where`。改 json 判定要跟着翻，代码里出现那份名单即失败
- 对不上就停：筛查里有一条、单音里查不到 → 停（悄悄跳过一条就把单音带进了训练）；
  任一边 path 重复 → 停
- 单音结论本身要自洽：`where` 只许 `none` / `first_only` / `later_too`，`present_seconds` 是整数列表，
  且「没有」⇔ 空列表、「只在第 1 秒」⇔ `[1]`（正本：`diagnose.tone_verdict`，秒号从 1 起）；不自洽就停
- 只动三样：`verdict` · `reasons` · `measures`。其余字段逐字段相等，输入列表本身不被改写
- 命令行：输出一行一景、打一行汇总；`--out` 指到输入上直接拒（不原地覆盖）
"""
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import screen_tone  # noqa: E402

RULES_PATH = HERE / "screen_rules.json"
# 测试这边写死期望值：正本是 json，这里只核「json 里就该是这两个」
EXPECT_DROP = ["first_only", "later_too"]


def screen_row(path, verdict="keep", reasons=None):
    """一条 screen.py 的输出行（字段够用即可）。"""
    return {"path": path, "sha256": "0" * 64, "sr": 16000, "seconds": 10.0,
            "verdict": verdict, "reasons": list(reasons or []),
            "measures": {"level_std_db": 1.5,
                         "top_tags": [["Inside, small room", 0.7]]}}


def tone_row(path, where, secs, hz=250.0):
    """一条 diagnose.py --tone 的输出行。"""
    return {"path": path,
            "tone": {"hz": hz, "per_second": [{"tone_dbfs": -60.0, "prominence_db": 1.0}],
                     "verdict": {"present_seconds": list(secs), "where": where}}}


def raw_tone_row(path, where, secs):
    """一条「present_seconds 原样放进去」的单音行：造不自洽的用例用（不复制、不强转）。"""
    row = tone_row(path, "none", [])
    row["tone"]["verdict"] = {"present_seconds": secs, "where": where}
    return row


def untouched(row):
    """除 verdict / reasons / measures 之外的字段，序列化成一行方便逐字比较。"""
    return json.dumps({k: v for k, v in row.items()
                       if k not in ("verdict", "reasons", "measures")},
                      ensure_ascii=False, sort_keys=True)


class DropWhere(unittest.TestCase):
    def test_real_rules_file(self):
        rules = json.loads(RULES_PATH.read_text(encoding="utf-8"))
        self.assertEqual(screen_tone.load_drop_where(), EXPECT_DROP)
        self.assertEqual(screen_tone.load_drop_where(RULES_PATH), EXPECT_DROP)

    def test_missing_key_is_an_error(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "rules.json"
            path.write_text(json.dumps({"tone_check": {"hz": 250.0}}), encoding="utf-8")
            with self.assertRaises(SystemExit) as cm:
                screen_tone.load_drop_where(path)
            self.assertIn("drop_where", str(cm.exception))

    def test_empty_list_is_an_error(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "rules.json"
            path.write_text(json.dumps({"tone_check": {"drop_where": []}}), encoding="utf-8")
            with self.assertRaises(SystemExit):
                screen_tone.load_drop_where(path)

    def test_list_comes_from_json_not_code(self):
        """判定名单（哪些位置该丢）只在 json 里：三个值只许出现在 `WHERE` 值域那一行（判自洽用的，不是判定名单）。"""
        code = (HERE / "screen_tone.py").read_text(encoding="utf-8")
        self.assertIsNone(re.search(r"drop_where\s*=\s*\[", code),
                          "drop_where 不许在代码里写成字面量列表")
        domain = [ln for ln in code.splitlines() if ln.startswith("WHERE = ")]
        self.assertEqual(len(domain), 1, domain)
        self.assertEqual(set(screen_tone.WHERE), {"none", *EXPECT_DROP})
        for word in ("none", *EXPECT_DROP):
            self.assertIn(f'"{word}"', domain[0], f"{word} 不在 WHERE 值域那一行")
            n = code.count(f'"{word}"')
            self.assertEqual(n, 1, f"{word} 在代码里出现了 {n} 次：只许在 WHERE 那一行")
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "rules.json"
            path.write_text(json.dumps({"tone_check": {"drop_where": ["later_too"]}}),
                            encoding="utf-8")
            self.assertEqual(screen_tone.load_drop_where(path), ["later_too"])
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "rules.json"
            path.write_text(json.dumps({"tone_check": {"drop_where": ["later_too"]}}),
                            encoding="utf-8")
            self.assertEqual(screen_tone.load_drop_where(path), ["later_too"])

    def test_source_file_is_short(self):
        lines = (HERE / "screen_tone.py").read_text(encoding="utf-8").splitlines()
        self.assertLessEqual(len(lines), 600)


class Merge(unittest.TestCase):
    def test_keep_and_no_tone_stays_keep(self):
        rows = screen_tone.merge([screen_row("a.wav")], [tone_row("a.wav", "none", [])],
                                 EXPECT_DROP)["rows"]
        self.assertEqual([r["verdict"] for r in rows], ["keep"])
        self.assertEqual(rows[0]["reasons"], [])
        self.assertEqual(rows[0]["measures"]["tone_where"], "none")
        self.assertEqual(rows[0]["measures"]["tone_present_seconds"], [])

    def test_keep_with_tone_later_becomes_drop(self):
        rows = screen_tone.merge([screen_row("a.wav")], [tone_row("a.wav", "later_too", [1, 3])],
                                 EXPECT_DROP)["rows"]
        row = rows[0]
        self.assertEqual(row["verdict"], "drop")
        self.assertEqual(row["reasons"], ["有单音：250 Hz 在第 1、3 秒（diagnose.py --tone）"])
        self.assertEqual(row["measures"]["tone_where"], "later_too")
        self.assertEqual(row["measures"]["tone_present_seconds"], [1, 3])

    def test_keep_with_tone_first_second_becomes_drop(self):
        rows = screen_tone.merge([screen_row("a.wav")], [tone_row("a.wav", "first_only", [1])],
                                 EXPECT_DROP)["rows"]
        row = rows[0]
        self.assertEqual(row["verdict"], "drop")
        self.assertEqual(row["reasons"], ["有单音：250 Hz 在第 1 秒（diagnose.py --tone）"])

    def test_drop_without_tone_keeps_its_reasons(self):
        old = screen_row("a.wav", verdict="drop", reasons=["有人声：说话帧占比 0.5 超过 0.1"])
        rows = screen_tone.merge([old], [tone_row("a.wav", "none", [])], EXPECT_DROP)["rows"]
        self.assertEqual(rows[0]["verdict"], "drop")
        self.assertEqual(rows[0]["reasons"], old["reasons"])

    def test_drop_with_tone_appends_reason_after_existing(self):
        old = screen_row("a.wav", verdict="drop", reasons=["有人声：说话帧占比 0.5 超过 0.1"])
        rows = screen_tone.merge([old], [tone_row("a.wav", "later_too", [2])], EXPECT_DROP)["rows"]
        self.assertEqual(rows[0]["verdict"], "drop")
        self.assertEqual(rows[0]["reasons"], old["reasons"]
                         + ["有单音：250 Hz 在第 2 秒（diagnose.py --tone）"])

    def test_drop_where_not_listed_keeps_keep(self):
        """名单外的位置不丢（这一项留给 split.py，本工具不管）。"""
        rows = screen_tone.merge([screen_row("a.wav")], [tone_row("a.wav", "first_only", [1])],
                                 ["later_too"])["rows"]
        self.assertEqual(rows[0]["verdict"], "keep")
        self.assertEqual(rows[0]["reasons"], [])
        self.assertEqual(rows[0]["measures"]["tone_where"], "first_only")

    def test_other_fields_and_order_untouched(self):
        old = [screen_row("b.wav"), screen_row("a.wav", verdict="drop", reasons=["不平稳"])]
        tones = [tone_row("a.wav", "none", []), tone_row("b.wav", "later_too", [5])]
        rows = screen_tone.merge(old, tones, EXPECT_DROP)["rows"]
        self.assertEqual([r["path"] for r in rows], ["b.wav", "a.wav"])
        for before, after in zip(old, rows):
            self.assertEqual(untouched(before), untouched(after))
        # 其余 measures 原样保留，只加两项
        self.assertEqual(rows[0]["measures"]["level_std_db"], 1.5)
        self.assertEqual(rows[0]["measures"]["top_tags"], [["Inside, small room", 0.7]])
        self.assertEqual(set(rows[0]["measures"]),
                         {"level_std_db", "top_tags", "tone_where", "tone_present_seconds"})

    def test_inputs_are_not_mutated(self):
        old = [screen_row("a.wav")]
        before = untouched(old[0])
        reasons_before = list(old[0]["reasons"])
        screen_tone.merge(old, [tone_row("a.wav", "later_too", [4])], EXPECT_DROP)
        self.assertEqual(untouched(old[0]), before)
        self.assertEqual(old[0]["reasons"], reasons_before)
        self.assertEqual(old[0]["verdict"], "keep")
        self.assertNotIn("tone_where", old[0]["measures"])

    def test_counts(self):
        old = [screen_row("a.wav"), screen_row("b.wav"), screen_row("c.wav", verdict="drop")]
        tones = [tone_row("a.wav", "none", []), tone_row("b.wav", "later_too", [2]),
                 tone_row("c.wav", "later_too", [2])]
        res = screen_tone.merge(old, tones, EXPECT_DROP)
        self.assertEqual({k: res[k] for k in ("total", "keep", "drop", "tone_dropped")},
                         {"total": 3, "keep": 1, "drop": 2, "tone_dropped": 1})


class ToneState(unittest.TestCase):
    """单音结论要自洽（正本：`diagnose.tone_verdict`）：where 只许三值；秒号从 1 起、整数；空列表 ⇔ none；[1] ⇔ first_only。"""

    def merge_one(self, where, secs):
        return screen_tone.merge([screen_row("a.wav")], [raw_tone_row("a.wav", where, secs)],
                                 EXPECT_DROP)

    def assert_stops(self, where, secs, *wanted):
        """必须停，且报错里写清是哪条、哪个值。"""
        with self.assertRaises(SystemExit) as cm:
            self.merge_one(where, secs)
        msg = str(cm.exception)
        self.assertIn("a.wav", msg)
        for w in wanted:
            self.assertIn(w, msg)
        return msg

    def test_consistent_states_pass(self):
        """三个合法组合都得能过：none · first_only [1] · later_too [2]（以及 first_only 之外的第一个非 1 秒）。"""
        for where, secs in (("none", []), ("first_only", [1]), ("later_too", [2]), ("later_too", [1, 3])):
            rows = self.merge_one(where, secs)["rows"]
            self.assertEqual(rows[0]["measures"]["tone_where"], where)
            self.assertEqual(rows[0]["measures"]["tone_present_seconds"], secs)

    def test_unknown_where_stops_naming_path_and_value(self):
        self.assert_stops("sometimes", [2], "sometimes")

    def test_where_is_not_a_string_stops(self):
        self.assert_stops(None, [], "verdict.where")

    def test_none_must_have_empty_seconds(self):
        self.assert_stops("none", [1], "none")

    def test_non_none_must_have_seconds(self):
        for where in ("first_only", "later_too"):
            self.assert_stops(where, [], where)

    def test_first_only_must_be_exactly_the_first_second(self):
        for secs in ([1, 2], [2]):
            self.assert_stops("first_only", secs, "first_only")

    def test_later_too_must_not_be_only_the_first_second(self):
        self.assert_stops("later_too", [1], "later_too")

    def test_seconds_must_be_a_list_of_ints(self):
        for secs in (None, 2, "2", {"2": 1}, [1.0], ["2"], [True]):
            self.assert_stops("later_too", secs, repr(secs))

    def test_drop_list_is_not_what_crashes_first(self):
        """不在丢的名单里也得先过自洽这一关：不自洽的单音行不能因为「不丢」就被放过。"""
        with self.assertRaises(SystemExit) as cm:
            screen_tone.merge([screen_row("a.wav")], [raw_tone_row("a.wav", "later_too", [])], ["none"])
        self.assertIn("later_too", str(cm.exception))


class Stops(unittest.TestCase):
    def test_missing_tone_row_stops(self):
        with self.assertRaises(SystemExit) as cm:
            screen_tone.merge([screen_row("a.wav"), screen_row("b.wav")],
                              [tone_row("a.wav", "none", [])], EXPECT_DROP)
        msg = str(cm.exception)
        self.assertIn("b.wav", msg)
        self.assertIn("1", msg)

    def test_duplicate_screen_path_stops(self):
        with self.assertRaises(SystemExit) as cm:
            screen_tone.merge([screen_row("a.wav"), screen_row("a.wav")],
                              [tone_row("a.wav", "none", [])], EXPECT_DROP)
        self.assertIn("a.wav", str(cm.exception))

    def test_duplicate_tone_path_stops(self):
        with self.assertRaises(SystemExit) as cm:
            screen_tone.merge([screen_row("a.wav")],
                              [tone_row("a.wav", "none", []), tone_row("a.wav", "later_too", [2])],
                              EXPECT_DROP)
        self.assertIn("a.wav", str(cm.exception))

    def test_tone_row_without_tone_key_stops(self):
        """diagnose.py 没加 --tone 就跑过：单音根本没查，不能当成没有。"""
        with self.assertRaises(SystemExit) as cm:
            screen_tone.merge([screen_row("a.wav")], [{"path": "a.wav"}], EXPECT_DROP)
        msg = str(cm.exception)
        self.assertIn("a.wav", msg)
        self.assertIn("--tone", msg)


class Cli(unittest.TestCase):
    def _files(self, root, screen, tone, rules=None):
        (root / "screen.jsonl").write_text(
            "".join(json.dumps(r, ensure_ascii=False) + "\n" for r in screen), encoding="utf-8")
        (root / "tone.jsonl").write_text(
            "".join(json.dumps(r, ensure_ascii=False) + "\n" for r in tone), encoding="utf-8")
        args = [sys.executable, str(HERE / "screen_tone.py"),
                "--screen", str(root / "screen.jsonl"), "--tone", str(root / "tone.jsonl")]
        if rules is not None:
            (root / "rules.json").write_text(json.dumps(rules), encoding="utf-8")
            args += ["--rules", str(root / "rules.json")]
        return args

    def test_writes_jsonl_and_summary(self):
        screen = [screen_row("a.wav"), screen_row("b.wav"), screen_row("c.wav", verdict="drop",
                                                                      reasons=["不平稳"])]
        tone = [tone_row("a.wav", "none", []), tone_row("b.wav", "first_only", [1]),
                tone_row("c.wav", "later_too", [2])]
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            args = self._files(root, screen, tone) + ["--out", str(root / "out.jsonl")]
            r = subprocess.run(args, capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            rows = [json.loads(ln) for ln in
                    (root / "out.jsonl").read_text(encoding="utf-8").splitlines()]
            self.assertEqual([x["path"] for x in rows], ["a.wav", "b.wav", "c.wav"])
            self.assertEqual([x["verdict"] for x in rows], ["keep", "drop", "drop"])
            self.assertIn("共 3 条：keep 1 · drop 2（其中因单音新丢 1 条）", r.stdout)

    def test_rules_from_json_change_verdicts(self):
        screen = [screen_row("a.wav")]
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            args = self._files(root, screen, [tone_row("a.wav", "first_only", [1])],
                               rules={"tone_check": {"drop_where": ["later_too"]}})
            args += ["--out", str(root / "out.jsonl")]
            r = subprocess.run(args, capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            rows = [json.loads(ln) for ln in
                    (root / "out.jsonl").read_text(encoding="utf-8").splitlines()]
            self.assertEqual(rows[0]["verdict"], "keep")
            self.assertIn("因单音新丢 0 条", r.stdout)

    def test_refuses_in_place_overwrite(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            args = self._files(root, [screen_row("a.wav")], [tone_row("a.wav", "none", [])])
            before = (root / "screen.jsonl").read_text(encoding="utf-8")
            for target in ("screen.jsonl", "tone.jsonl"):
                r = subprocess.run(args + ["--out", str(root / target)],
                                   capture_output=True, text=True)
                self.assertNotEqual(r.returncode, 0, target)
                self.assertIn(target, r.stderr + r.stdout)
            self.assertEqual((root / "screen.jsonl").read_text(encoding="utf-8"), before)

    def test_missing_tone_row_stops_the_run(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            args = self._files(root, [screen_row("a.wav"), screen_row("b.wav")],
                               [tone_row("a.wav", "none", [])]) + ["--out", str(root / "out.jsonl")]
            r = subprocess.run(args, capture_output=True, text=True)
            self.assertNotEqual(r.returncode, 0)
            self.assertIn("b.wav", r.stderr)
            self.assertFalse((root / "out.jsonl").exists())


if __name__ == "__main__":
    unittest.main()
