"""云上产物拷回进仓后的核对：splits/ 与 builds/<id>/build.json（README 第六节「核对」）。
第一批产物 2026-11-25 已进仓，从此缺了就红（codex 第 1 轮 P2 #5：以前缺了直接跳过，漏拷或误删逃得过）。"""
import json
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HERE))
import loader  # noqa: E402
import split  # noqa: E402

SPLITS = HERE / "splits"
BUILDS = HERE / "builds"
SHARDED = ("librispeech-train-clean-100", "qut-noise", "tum-wind", "openslr28-rir", "demand-train")


def split_names():
    return sorted(p.stem for p in SPLITS.glob("*.json")) if SPLITS.exists() else []


def build_files():
    return sorted(BUILDS.glob("*/build.json")) if BUILDS.exists() else []


class RegisterNeedsNoMix(unittest.TestCase):
    def test_loader_import_does_not_load_mix(self):
        """登记 build 只做交集检查：导入 loader 不许连带导入 mix（mix 依赖还没合进来的 rooms.early_rir）。"""
        code = f"import sys; sys.path.insert(0, {str(HERE)!r}); import loader; print('mix' in sys.modules, 'rooms' in sys.modules)"
        out = subprocess.run([sys.executable, "-I", "-c", code], capture_output=True, text=True, check=True).stdout.split()
        self.assertEqual(out, ["False", "False"])


class Splits(unittest.TestCase):
    def setUp(self):
        self.assertTrue(split_names(), "仓里没有 splits/：产物丢了或没拷回")
        self.R = split.load_rules()

    def test_all_sources_present(self):
        """每个来源一张表；唯一例外是规则顶层 `未建表` 里的（音频还在云上，本机造不出表）。
        两向都查：表带回仓后忘了删那一行，文件名会多出一张，也会红。"""
        pending = set(self.R.get("未建表", {}))
        self.assertEqual(split_names(), sorted(set(self.R["sources"]) - pending))

    def test_tables_match_rules(self):
        for n in split_names():
            s, rows = split.load_split(n, SPLITS)  # 核 jsonl 的 sha256
            rule = self.R["sources"][n]
            self.assertEqual(s["rule_sha256"], split.rule_sha(rule), f"{n}：切分表是按旧规则切的")
            self.assertEqual(s["seed"], self.R["seed"], n)
            self.assertEqual(s["files"], len(rows), n)
            e = rule.get("expect", {})  # 真规则一定有（Expect 测试核）
            if "files" in e:
                self.assertEqual(len(rows), e["files"], n)
            self.assertGreaterEqual(len(rows), e.get("min_files", 0), n)
            self.assertEqual(len({r["path"] for r in rows}), len(rows), f"{n}：同一文件出现两次")
            body = (SPLITS / f"{n}.jsonl").read_bytes() if (SPLITS / f"{n}.jsonl").exists() else None
            again = split.summary(n, rule, s["seed"], rows, body or b"")  # codex 第 4 轮 P2 #4：摘要统计由正文重算
            for k in ("files", "units", "per_split", "time", "seconds"):
                if k in s:
                    self.assertEqual(again.get(k), s[k], f"{n}：摘要的 {k} 和正文对不上")

    def test_unit_tables_reproduce(self):
        """按单位切的：用仓里的单位清单重算一遍分配，结果逐个相同（确定性）。"""
        for n in split_names():
            rule = self.R["sources"][n]
            if rule["unit"] == "time":
                continue
            _, rows = split.load_split(n, SPLITS)
            strata = None
            if rule.get("stratum_regex"):
                import re

                rx = re.compile(rule["stratum_regex"])
                strata = {r["unit"]: ("synth" if rx.search(r["path"]) else "real") for r in rows}
            amap = split.assign_units([r["unit"] for r in rows], rule["frac"], self.R["seed"], n, self.R["splits"], strata)
            by_unit = {}
            for r in rows:
                self.assertEqual(r["split"], amap[r["unit"]], f"{n} {r['path']}")
                by_unit.setdefault(r["unit"], set()).add(r["split"])
            self.assertTrue(all(len(v) == 1 for v in by_unit.values()), f"{n}：同一单位跨了份")

    def test_units_follow_rules_from_path(self):
        """自审 P2：单位列是 unit_regex / 说话人取法 / room_merge 的第二处表示，要和正本比。
        说话人与按文件切的从路径整个重算；按房间切的，仓里没有 rir_list，核「路径落进 room_merge 的 ⇔ 单位是合并后的单位」，
        其余单位以 rir_list 所在目录名开头。"""
        for n in split_names():
            rule = self.R["sources"][n]
            if rule["unit"] == "time":
                continue
            _, rows = split.load_split(n, SPLITS)
            if rule["unit"] in ("speaker", "file"):
                for r in rows:
                    self.assertEqual(r["unit"], split.unit_of(r["path"], rule), f"{n} {r['path']}")
                continue
            import re
            from pathlib import PurePosixPath as P

            merge = [(re.compile(x["match"]), x["unit"]) for x in rule.get("room_merge", [])]
            targets = {to for _, to in merge}
            for r in rows:
                p = P(r["path"])
                probe = f"{p.parent.name}/{p.name}"  # 实测 RIR 的文件名前缀与 room-id 前缀相同（air_ · RWCP_）
                hit = next((to for rx, to in merge if rx.search(probe)), None)
                if hit:
                    self.assertEqual(r["unit"], hit, f"{n} {r['path']}")
                else:
                    self.assertNotIn(r["unit"], targets, f"{n} {r['path']}")
                    self.assertTrue(r["unit"].startswith(p.parent.name + "/") or r["unit"].startswith(p.parent.parent.name + "/"), f"{n} {r['path']} {r['unit']}")

    def test_time_segments_reproduce(self):
        for n in split_names():
            rule = self.R["sources"][n]
            if rule["unit"] != "time":
                continue
            _, rows = split.load_split(n, SPLITS)
            for r in rows:
                self.assertEqual(r["segments"], split.time_segments(r["frames"], rule["time"]), f"{n} {r['path']}")


class Builds(unittest.TestCase):
    def setUp(self):
        self.assertTrue(build_files(), "仓里没有 builds/<id>/build.json：产物丢了或没拷回")

    def test_build_valid_and_tables_in_repo(self):
        for f in build_files():
            b = json.loads(f.read_text())
            self.assertEqual(b["build_id"], f.parent.name)
            ident = {k: b[k] for k in loader.ID_KEYS if k in b}  # codex 第 2 轮 P2 #4：由记录的身份字段重算 id
            self.assertEqual(loader.build_id_of(ident), b["build_id"], f"{f}：身份字段被改过")
            self.assertTrue(b["valid"], f"{f}：交集检查不过")
            self.assertTrue(b["leak_check"]["ok"], str(f))
            self.assertEqual(sorted(b["split_tables"]), sorted(SHARDED), str(f))
            for src, want in b["split_tables"].items():
                s, _ = split.load_split(src, SPLITS)
                self.assertEqual(s["jsonl_sha256"], want, f"{f}：{src} 切分表和仓里的不同")

    def test_only_train_dev_sharded(self):
        """终测原料池与测试份不进训练分片（D63）。"""
        for f in build_files():
            b = json.loads(f.read_text())
            for src, d in b["counts"].items():
                self.assertLessEqual(set(d), {"train", "dev"}, f"{f}：{src}")
                self.assertGreater(d.get("train", 0), 0, f"{f}：{src} 没有训练份")


if __name__ == "__main__":
    unittest.main()
