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
import shard  # noqa: E402
import split  # noqa: E402

SPLITS = HERE / "splits"
BUILDS = HERE / "builds"


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
                # 期望条数是排除之前数的（清单 → 期望 → meta → 筛查）：摘要里有排除计数（零不记）就加回来
                got = len(rows) + s.get("excluded_meta", 0) + s.get("excluded_screen", 0)
                self.assertEqual(got, e["files"], f"{n}：{len(rows)} 行 + 排除 {got - len(rows)} ≠ 期望 {e['files']}")
            self.assertGreaterEqual(len(rows), e.get("min_files", 0), n)
            self.assertEqual(len({r["path"] for r in rows}), len(rows), f"{n}：同一文件出现两次")
            spec = rule.get("screen_file")
            if spec:
                # 留下的每一行在筛查表里都得是 keep（第二处表示：切分表 ⊆ 筛查表判 keep 的）
                p = Path(spec)
                p = p if p.is_absolute() else HERE / spec
                verdict = {}
                for line in p.read_text(encoding="utf-8").splitlines():
                    if line.strip():
                        rec = json.loads(line)
                        verdict[rec["path"]] = rec.get("verdict")
                for r in rows:
                    self.assertEqual(verdict.get(r["path"]), "keep", f"{n}：{r['path']} 在筛查表 {spec} 里不是 keep")
            body = (SPLITS / f"{n}.jsonl").read_bytes() if (SPLITS / f"{n}.jsonl").exists() else None
            again = split.summary(n, rule, s["seed"], rows, body or b"")  # codex 第 4 轮 P2 #4：摘要统计由正文重算
            for k in ("files", "units", "per_split", "time", "seconds"):
                if k in s:
                    self.assertEqual(again.get(k), s[k], f"{n}：摘要的 {k} 和正文对不上")

    def test_unit_tables_reproduce(self):
        """按单位切的：用仓里的单位清单重算一遍分配，结果逐个相同（确定性）。
        分层的算法与 split_source 里那几行保持一致（stratum_unit_regex 先在，没有再按 stratum_regex 从路径认，如 TUM 风噪 /
        RIR 模拟与实测），但**故意不抽成 split.py 的函数**：split.py 在 loader.CODE 里，改它一个字节，build_id 就变，
        刚登记的 build 在新代码下读不了。这是重算测试，表能不能逐个复现本身就是交叉校验；要改 split.py 先开新 build。"""
        import re

        for n in split_names():
            rule = self.R["sources"][n]
            if rule["unit"] == "time":
                continue
            _, rows = split.load_split(n, SPLITS)
            strata = split.unit_strata({r["unit"] for r in rows}, rule)
            if strata is None and rule.get("stratum_regex"):
                rx = re.compile(rule["stratum_regex"])
                strata = {r["unit"]: ("synth" if rx.search(r["path"]) else "real") for r in rows}
            min_dev = int(rule.get("min_dev_per_stratum", 0) or 0)
            amap = split.assign_units([r["unit"] for r in rows], rule["frac"], self.R["seed"], n, self.R["splits"], strata, min_dev)
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
        self.R = split.load_rules()

    def test_build_valid_and_tables_in_repo(self):
        for f in build_files():
            b = json.loads(f.read_text())
            self.assertEqual(b["build_id"], f.parent.name)
            ident = {k: b[k] for k in loader.ID_KEYS if k in b}  # codex 第 2 轮 P2 #4：由记录的身份字段重算 id
            self.assertEqual(loader.build_id_of(ident), b["build_id"], f"{f}：身份字段被改过")
            self.assertTrue(b["valid"], f"{f}：交集检查不过")
            self.assertTrue(b["leak_check"]["ok"], str(f))
            # 不再手抄一份来源清单（第二处表示）：build 自己切了哪几张表，就记了哪几个来源的条数，两向相同
            self.assertEqual(sorted(b["split_tables"]), sorted(b["counts"]), str(f))
            for src in b["split_tables"]:
                self.assertIn(src, self.R["sources"], f"{f}：{src} 不是 split_rules.json 里的来源")
            for src, want in b["split_tables"].items():
                s, _ = split.load_split(src, SPLITS)
                self.assertEqual(s["jsonl_sha256"], want, f"{f}：{src} 切分表和仓里的不同")

    def test_some_build_shards_current_sources(self):
        """至少要有一份 build 把当前配方（recipe.json）的全部来源都切了：加了一批来源却忘了云上重跑，这里就红。
        只比来源集合，不比 recipe 的字节哈希（配方小幅改动也会变哈希，那会让这条测试无辜地红）。
        规则顶层 `未建表` 里的来源（音频还在云上）不进比，其余照旧严格。"""
        pending = set(self.R.get("未建表", {}))
        want = set(shard.default_sources()) - pending
        got = {f.parent.name: set(json.loads(f.read_text())["split_tables"]) for f in build_files()}
        self.assertTrue(any(v == want for v in got.values()),
                        f"没有一份 build 切了全部来源：要 {sorted(want)}；现有 {[(k, sorted(v)) for k, v in got.items()]}")

    def test_only_train_dev_sharded(self):
        """终测原料池与测试份不进训练分片（D63）。"""
        for f in build_files():
            b = json.loads(f.read_text())
            for src, d in b["counts"].items():
                self.assertLessEqual(set(d), {"train", "dev"}, f"{f}：{src}")
                self.assertGreater(d.get("train", 0), 0, f"{f}：{src} 没有训练份")


if __name__ == "__main__":
    unittest.main()
