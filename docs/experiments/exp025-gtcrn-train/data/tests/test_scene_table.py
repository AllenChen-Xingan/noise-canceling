"""目标场景噪声表（scene_table.py）：数字只从 build 记录 · 配方 · 切分表 · 来源表算，不手抄。

小夹具在临时目录里搭（假 build.json / recipe.json / split_rules.json / sources.json / splits / scene_notes.json），
核条数 · 时长 · 单位数 · 正则过滤；再把每道「表不在这个 build 上」的检查都撞一次。
另有一条真产物测试：拿仓里的 builds/10aa3a9fb8cef48f 与真配方跑一遍。
"""
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import scene_table  # noqa: E402
import split  # noqa: E402

BUILD = HERE / "builds" / "10aa3a9fb8cef48f"


def dump(p, obj):
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(obj, ensure_ascii=False), encoding="utf-8")


def write_table(root, name, rows):
    """写一张切分表 + 摘要，返回正文哈希（build.json 的 split_tables 就记这个）。"""
    body = "".join(json.dumps(r, sort_keys=True, ensure_ascii=False) + "\n" for r in rows).encode()
    d = root / "splits"
    d.mkdir(parents=True, exist_ok=True)
    (d / f"{name}.jsonl").write_bytes(body)
    h = hashlib.sha256(body).hexdigest()
    dump(d / f"{name}.json", {"source": name, "jsonl_sha256": h})
    return h


STREET = [  # 按 split 切的来源（如 CochlScene）
    {"path": "raw/street/a.wav", "split": "train", "unit": "s1", "frames": 16000, "sr": 16000},
    {"path": "raw/street/b.wav", "split": "train", "unit": "s1", "frames": 8000, "sr": 16000},
    {"path": "raw/street/c.wav", "split": "train", "unit": "s2", "frames": 16000, "sr": 16000},
    {"path": "raw/street/d.wav", "split": "dev", "unit": "s3", "frames": 16000, "sr": 16000},
    {"path": "raw/other/e.wav", "split": "train", "unit": "s4", "frames": 16000, "sr": 16000},
]
CAFE = [  # 按时段切的来源（如 DEMAND）：段界写在 segments 里，没有 split 字段
    {"path": "raw/cafe/c1.wav", "segments": {"train": [0, 8000], "dev": [8000, 16000]}, "unit": "c1",
     "frames": 16000, "sr": 16000},
    {"path": "raw/cafe/c2.wav", "segments": {"train": [0, 4000]}, "unit": "c2", "frames": 16000, "sr": 16000},
]


class Fixture(unittest.TestCase):
    """临时目录里的一套最小数据：两个场景、两个来源。"""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.h_street = write_table(self.root, "noise-street", STREET)
        self.h_cafe = write_table(self.root, "noise-cafe", CAFE)
        self.build = {
            "build_id": "test-build-0123", "split_tables": {"noise-street": self.h_street, "noise-cafe": self.h_cafe},
            "noise_scenes": {"train": ["street", "cafe"], "dev": ["street", "cafe"]},
            "counts": {"noise-street": {"train": 3, "dev": 1}, "noise-cafe": {"train": 2}},
        }
        self.recipe = {
            "knobs": {"target_scene": {"on": True, "scene": ["street", "cafe"], "frac": 0.667}},
            "noise_pools": {"_说明": "键 = 场景名", "street": [{"source": "noise-street", "regex": "/street/"}],
                            "cafe": [{"source": "noise-cafe", "regex": "cafe/"}]},
        }
        self.rules = {"sources": {"noise-street": {"group": "G1", "unit": "file"},
                                  "noise-cafe": {"group": "G2", "unit": "time"}}}
        self.sources = {"groups": {
            "G1": {"license": "CC-BY-4.0", "attribution": "某篇论文 2020", "train_ok": True},
            "G2": {"license": "CC-BY-SA", "attribution": "另一篇论文 2019", "train_ok": True}}}
        self.notes = {"_说明": "只是人写的已知限制，不进 build id", "street": "只有一段路", "cafe": "只有一家店"}

    def tearDown(self):
        self._tmp.cleanup()

    def write(self, build=None, recipe=None, rules=None, sources=None, notes=None):
        """把整套数据写盘（每个测试可以改一份），recipe_sha256 按实际写出的字节算。"""
        dump(self.root / "builds" / "b1" / "build.json", build or self.build)
        dump(self.root / "recipe.json", recipe or self.recipe)
        dump(self.root / "split_rules.json", rules or self.rules)
        dump(self.root / "sources.json", sources or self.sources)
        dump(self.root / "scene_notes.json", notes or self.notes)
        sha = hashlib.sha256((self.root / "recipe.json").read_bytes()).hexdigest()
        b = json.loads((self.root / "builds" / "b1" / "build.json").read_text(encoding="utf-8"))
        b["recipe_sha256"] = sha
        dump(self.root / "builds" / "b1" / "build.json", b)
        return self.root / "builds" / "b1"

    def tables(self, **kw):
        return scene_table.build_tables(self.write(**kw), data_dir=self.root)


class Tables(Fixture):
    def test_counts_seconds_units_and_regex(self):
        t = self.tables()
        st = t["scenes"]["street"]["splits"]
        self.assertEqual(st["train"]["sources"][0]["source"], "noise-street")
        self.assertEqual(st["train"]["sources"][0]["files"], 3)  # raw/other/e.wav 正则不匹配，不算
        self.assertAlmostEqual(st["train"]["sources"][0]["seconds"], 2.5)  # 1 + 0.5 + 1 秒
        self.assertEqual(st["train"]["sources"][0]["units"], 2)  # s1 · s2
        self.assertEqual(st["dev"]["sources"][0]["files"], 1)
        self.assertAlmostEqual(st["dev"]["totals"]["seconds"], 1.0)
        self.assertEqual(st["train"]["totals"], {"files": 3, "seconds": 2.5, "units": 2})

    def test_segment_rows_use_segment_length(self):
        ca = self.tables()["scenes"]["cafe"]["splits"]
        self.assertEqual(ca["train"]["totals"]["files"], 2)
        self.assertAlmostEqual(ca["train"]["totals"]["seconds"], 0.75)  # (8000-0)/16000 + (4000-0)/16000
        self.assertEqual(ca["train"]["totals"]["units"], 2)
        self.assertAlmostEqual(ca["dev"]["totals"]["seconds"], 0.5)  # 只有 c1 有 dev 段

    def test_license_and_notes_and_header(self):
        t = self.tables()
        self.assertEqual(t["scenes"]["street"]["splits"]["train"]["sources"][0]["license"], "CC-BY-4.0")
        self.assertEqual(t["scenes"]["cafe"]["notes"], "只有一家店")
        self.assertEqual(t["build_id"], "test-build-0123")
        md = scene_table.render_md(t)
        self.assertIn("test-build-0123", md)
        self.assertIn(t["recipe_sha256"][:12], md)
        self.assertIn("| 来源 | 份 | 条数 | 时长（小时，2 位小数） | 单位数 | 许可 |", md)
        self.assertIn("| noise-street | train | 3 |", md)
        self.assertIn("已知限制：只有一段路", md)
        self.assertIn("## cafe", md)

    def test_hours_two_decimals(self):
        t = self.tables()
        md = scene_table.render_md(t)
        # street 训练 2.5 秒 = 0.000694… 小时 → 0.00
        self.assertIn("| 合计 | train | 3 | 0.00 | 2 | |", md)

    def test_scenes_arg_limits_output(self):
        t = self.tables()
        self.assertEqual(sorted(scene_table.build_tables(self.root / "builds" / "b1", data_dir=self.root,
                                                         scenes=["cafe"])["scenes"]), ["cafe"])


class Stops(Fixture):
    """任一张表不属于这个 build、或场景在这个 build 里没有噪声，都必须停：表在骗人就等于没有表。"""

    def test_recipe_hash_mismatch(self):
        b = self.write()
        build = json.loads((b / "build.json").read_text(encoding="utf-8"))
        build["recipe_sha256"] = "0" * 64
        dump(b / "build.json", build)
        with self.assertRaises(SystemExit) as e:
            scene_table.build_tables(b, data_dir=self.root)
        self.assertIn("recipe", str(e.exception))

    def test_split_table_hash_mismatch(self):
        build = dict(self.build, split_tables={"noise-street": "1" * 64, "noise-cafe": self.h_cafe})
        with self.assertRaises(SystemExit) as e:
            self.tables(build=build)
        self.assertIn("noise-street", str(e.exception))

    def test_split_table_body_tampered(self):
        b = self.write()
        p = self.root / "splits" / "noise-street.jsonl"
        p.write_bytes(p.read_bytes() + b'{"path": "raw/street/z.wav", "split": "train", "unit": "s9", '
                                        b'"frames": 16000, "sr": 16000}\n')
        with self.assertRaises(SystemExit):
            scene_table.build_tables(b, data_dir=self.root)

    def test_train_ok_false(self):
        src = json.loads(json.dumps(self.sources))
        src["groups"]["G2"]["train_ok"] = False
        with self.assertRaises(SystemExit) as e:
            self.tables(sources=src)
        self.assertIn("noise-cafe", str(e.exception))

    def test_scene_without_train_noise(self):
        # 只有开发段：c2 没有 dev 段 → 给 cafe 造一个「一个训练条数也没有」的来源
        rows = [{"path": "raw/cafe/c9.wav", "segments": {"dev": [0, 4000]}, "unit": "c9",
                 "frames": 16000, "sr": 16000}]
        h = write_table(self.root, "noise-cafe", rows)
        build = dict(self.build, split_tables={"noise-street": self.h_street, "noise-cafe": h},
                     noise_scenes={"train": ["street"], "dev": ["street"]})
        with self.assertRaises(SystemExit) as e:
            self.tables(build=build)
        self.assertIn("cafe", str(e.exception))
        self.assertIn("没有训练噪声", str(e.exception))

    def test_scene_missing_from_build_scenes(self):
        build = dict(self.build, noise_scenes={"train": ["street"], "dev": ["street"]})
        with self.assertRaises(SystemExit) as e:
            self.tables(build=build)
        self.assertIn("cafe", str(e.exception))

    def test_scene_missing_from_notes(self):
        notes = {"_说明": "只是人写的已知限制", "street": "只有一段路"}
        with self.assertRaises(SystemExit) as e:
            self.tables(notes=notes)
        self.assertIn("cafe", str(e.exception))

    def test_scene_missing_from_pools(self):
        recipe = json.loads(json.dumps(self.recipe))
        recipe["knobs"]["target_scene"]["scene"] = ["street", "meeting-room"]
        with self.assertRaises(SystemExit) as e:
            self.tables(recipe=recipe)
        self.assertIn("meeting-room", str(e.exception))


class Product(unittest.TestCase):
    """真产物：build 10aa3a9fb8cef48f 的三个人定目标场景（D113）。"""

    def setUp(self):
        self.t = scene_table.build_tables(BUILD)

    def test_three_target_scenes_have_train_noise(self):
        self.assertEqual(sorted(self.t["scenes"]), ["canteen", "rail-air-cabin", "transit-hub"])
        for name, s in self.t["scenes"].items():
            self.assertGreater(s["splits"]["train"]["totals"]["files"], 0, name)
            self.assertGreater(s["splits"]["train"]["totals"]["seconds"], 0, name)

    def test_canteen_only_demand(self):
        src = self.t["scenes"]["canteen"]["splits"]["train"]["sources"]
        self.assertEqual([x["source"] for x in src], ["demand-train"])
        self.assertEqual(src[0]["license"], "CC-BY-4.0")

    def test_transit_hub_matches_cochlscene_subway_station(self):
        src = self.t["scenes"]["transit-hub"]["splits"]["train"]["sources"][0]
        self.assertEqual(src["source"], "cochlscene")
        # 切分表摘要里 SubwayStation 的 train 条数（split_tables 核过哈希，数字可交叉核）
        s, rows = split.load_split("cochlscene", out_dir=HERE / "splits")
        self.assertEqual(src["files"], sum(1 for r in rows if r["split"] == "train" and "/SubwayStation/" in r["path"]))

    def test_markdown_has_each_scene(self):
        md = scene_table.render_md(self.t)
        for name in ("rail-air-cabin", "canteen", "transit-hub"):
            self.assertIn(name, md)
        self.assertIn("10aa3a9fb8cef48f", md)
        self.assertIn("已知限制：", md)


class Size(unittest.TestCase):
    def test_new_py_under_600(self):
        for name in ("scene_table.py", "tests/test_scene_table.py"):
            n = len((HERE / name).read_text(encoding="utf-8").splitlines())
            self.assertLessEqual(n, 600, f"{name} 有 {n} 行")


if __name__ == "__main__":
    unittest.main()
