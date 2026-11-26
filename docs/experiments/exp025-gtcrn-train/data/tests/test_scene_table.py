"""目标场景噪声表（scene_table.py）：数字只从 build 记录 · 配方 · 切分表 · 来源表算，不手抄。

小夹具在临时目录里搭（假 build.json / recipe.json / split_rules.json / sources.json / splits / scene_notes.json），
核条数 · 时长 · 单位数 · 正则过滤；再把每道「表不在这个 build 上」的检查都撞一次。
另有一类真产物测试：仓里第四批的 build `bc0e9e7b566421f0` 记的配方哈希就是仓里 `recipe.json` 的哈希、切分表也已进仓，
所以直接在仓里的文件上跑（不再绕临时目录），并核提交进仓的 `scenes-bc0e9e7b566421f0.md` 与重算结果逐字节相同。
"""
import hashlib
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import scene_table  # noqa: E402
import split  # noqa: E402

BUILD = HERE / "builds" / "bc0e9e7b566421f0"  # 第四批的基准 build（配方 = 仓里 recipe.json，四个目标场景都有噪声）
SCENES_MD = HERE / f"scenes-{BUILD.name}.md"  # `python3 scene_table.py --build builds/<id>` 的标准输出，重算即核


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


DEMAND_ROWS = [  # 同一环境 3 个声道 = 同一次录音；另一环境 1 个声道
    {"path": "vdn-data/demand/AKITCHEN/ch01.wav", "segments": {"train": [0, 32000]},
     "unit": "vdn-data/demand/AKITCHEN/ch01.wav", "frames": 48000, "sr": 16000},
    {"path": "vdn-data/demand/AKITCHEN/ch02.wav", "segments": {"train": [0, 32000]},
     "unit": "vdn-data/demand/AKITCHEN/ch02.wav", "frames": 48000, "sr": 16000},
    {"path": "vdn-data/demand/AKITCHEN/ch03.wav", "segments": {"train": [0, 32000]},
     "unit": "vdn-data/demand/AKITCHEN/ch03.wav", "frames": 48000, "sr": 16000},
    {"path": "vdn-data/demand/BKITCHEN/ch01.wav", "segments": {"train": [0, 16000]},
     "unit": "vdn-data/demand/BKITCHEN/ch01.wav", "frames": 48000, "sr": 16000},
]
BUT_ROWS = [  # 同一场次两个麦克风（同时录同一段底噪）+ 另一个录音文件各麦克风又各录一遍
    {"path": "train-raw/but-reverbdb/RoomA/MicID01/Spk1_S/01/silence/silence_16kHz_60sec.v00.wav",
     "split": "train", "unit": "RoomA/Spk1_S", "frames": 16000, "sr": 16000},
    {"path": "train-raw/but-reverbdb/RoomA/MicID02/Spk1_S/01/silence/silence_16kHz_60sec.v00.wav",
     "split": "train", "unit": "RoomA/Spk1_S", "frames": 16000, "sr": 16000},
    {"path": "train-raw/but-reverbdb/RoomA/MicID01/Spk1_S/01/silence/silence_16kHz_60sec.v01.wav",
     "split": "train", "unit": "RoomA/Spk1_S", "frames": 16000, "sr": 16000},
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
        # 没在 scene_notes.json 的 same_content 里登记的来源：每个文件都是不同内容
        self.assertAlmostEqual(st["train"]["sources"][0]["unique_seconds"], 2.5)
        self.assertEqual(st["train"]["totals"], {"files": 3, "seconds": 2.5, "unique_seconds": 2.5, "units": 2})

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
        self.assertIn("| 来源 | 份 | 条数 | 时长（小时，按条数） | 不同内容（小时） | 单位数 | 许可 |", md)
        self.assertIn("same_content", md)
        self.assertIn("| noise-street | train | 3 |", md)
        self.assertIn("已知限制：只有一段路", md)
        self.assertIn("## cafe", md)

    def test_hours_two_decimals(self):
        t = self.tables()
        md = scene_table.render_md(t)
        # street 训练 2.5 秒 = 0.000694… 小时 → 0.00
        self.assertIn("| 合计 | train | 3 | 0.00 | 0.00 | 2 | |", md)

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


class SameContent(Fixture):
    """同一段声音被多个麦克风 / 声道录了多次时，条数与时长会夸大：另报「不同内容」时长。

    夹具：DEMAND 同一环境 3 个声道 + 另一个环境 1 个声道；BUT 同一场次 2 个麦克风 + 另一场次 1 个麦克风。
    """

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.h_demand = write_table(self.root, "demand-train", DEMAND_ROWS)
        self.h_but = write_table(self.root, "but-reverbdb-silence", BUT_ROWS)
        self.build = {
            "build_id": "test-same-0123",
            "split_tables": {"demand-train": self.h_demand, "but-reverbdb-silence": self.h_but},
            "noise_scenes": {"train": ["canteen", "hotel-room"], "dev": []},
            "counts": {},
        }
        self.recipe = {
            "knobs": {"target_scene": {"on": True, "scene": ["canteen", "hotel-room"], "frac": 0.667}},
            "noise_pools": {"canteen": [{"source": "demand-train", "regex": "AKITCHEN|BKITCHEN"}],
                            "hotel-room": [{"source": "but-reverbdb-silence", "regex": "/RoomA/"}]},
        }
        self.rules = {"sources": {"demand-train": {"group": "E2", "unit": "time"},
                                  "but-reverbdb-silence": {"group": "A7", "unit": "file"}}}
        self.sources = {"groups": {
            "E2": {"license": "CC-BY-4.0", "attribution": "DEMAND", "train_ok": True},
            "A7": {"license": "CC-BY-4.0", "attribution": "BUT ReverbDB", "train_ok": True}}}
        self.same = {
            "_说明": "下划线开头的键是说明，不是来源",
            "demand-train": {"regex": r"demand/(\w+)/ch\d+\.wav$", "group": r"\1",
                             "why": "DEMAND 是 16 声道阵列，同一环境的 16 个声道是同一次录音"},
            "but-reverbdb-silence": {
                "regex": r"^train-raw/but-reverbdb/([^/]+)/MicID\d+/([^/]+)/[^/]+/silence/([^/]+)$",
                "group": r"\1/\2/\3", "why": "同一次布置、同一个文件名的各麦克风同时录的是同一段底噪"},
        }
        self.notes = {"_说明": "只是人写的已知限制，不进 build id", "canteen": "只有 DEMAND PCAFETER",
                      "hotel-room": "只有一间房", "same_content": self.same}

    def demand(self, **kw):
        return self.tables(**kw)["scenes"]["canteen"]["splits"]["train"]

    def but(self, **kw):
        return self.tables(**kw)["scenes"]["hotel-room"]["splits"]["train"]

    def test_channels_of_one_recording_count_once(self):
        d = self.demand()
        src = d["sources"][0]
        self.assertEqual(src["files"], 4)
        self.assertAlmostEqual(src["seconds"], 7.0)  # 2 + 2 + 2 + 1 秒（按条数）
        self.assertAlmostEqual(src["unique_seconds"], 3.0)  # A 环境一次 2 秒 + B 环境一次 1 秒
        self.assertAlmostEqual(d["totals"]["unique_seconds"], 3.0)

    def test_group_takes_the_longest_file(self):
        # 同一次录音的几个声道长度不会完全相等（夹具里故意差一点）：取最大而不是相加
        rows = [dict(r) for r in DEMAND_ROWS]
        rows[1] = dict(rows[1], segments={"train": [0, 40000]})  # 2.5 秒
        h = write_table(self.root, "demand-train", rows)
        build = dict(self.build, split_tables={"demand-train": h, "but-reverbdb-silence": self.h_but})
        src = self.demand(build=build)["sources"][0]
        self.assertAlmostEqual(src["seconds"], 7.5)
        self.assertAlmostEqual(src["unique_seconds"], 3.5)  # 2.5（最大）+ 1

    def test_mics_of_one_take_count_once_other_takes_separate(self):
        src = self.but()["sources"][0]
        self.assertEqual(src["files"], 3)
        self.assertAlmostEqual(src["seconds"], 3.0)
        self.assertAlmostEqual(src["unique_seconds"], 2.0)  # MicID01/02 同一个文件名算一次，v01 另算一次

    def test_same_content_key_is_not_a_scene(self):
        t = self.tables()
        self.assertEqual(sorted(t["scenes"]), ["canteen", "hotel-room"])
        self.assertNotIn("same_content", t["scenes"])

    def test_listed_source_path_must_match(self):
        same = dict(self.same, **{"demand-train": {"regex": r"demand/(\w+)/ch\d+\.mp3$",
                                                    "group": r"\1", "why": "写错的正则"}})
        notes = dict(self.notes, same_content=same)
        with self.assertRaises(SystemExit) as e:
            self.tables(notes=notes)
        self.assertIn("ch01.wav", str(e.exception))
        self.assertIn("demand-train", str(e.exception))

    def test_no_same_content_entry_means_every_file_distinct(self):
        notes = {k: v for k, v in self.notes.items() if k != "same_content"}
        d = self.demand(notes=notes)
        self.assertAlmostEqual(d["sources"][0]["unique_seconds"], 7.0)


class Product(unittest.TestCase):
    """真产物：仓里第四批的 build `bc0e9e7b566421f0` + 仓里 `recipe.json`（四个目标场景，D113）。

    这个 build 记的 `recipe_sha256` 就是仓里 `recipe.json` 的哈希，四个场景的切分表也已进仓；
    于是 `build_tables` 直接在仓里的文件上跑——不需要任何临时目录（旧的绕法只因为那时候 recipes 改过、还没重登记）。
    """

    HAVE = ("canteen", "meeting-room", "rail-air-cabin", "transit-hub")  # 配方里那四个目标场景（D113）

    def setUp(self):
        self.t = scene_table.build_tables(BUILD, data_dir=HERE)  # 不传 scenes：就用配方里那四个

    def test_scenes_are_the_four_targets_and_all_have_train_noise(self):
        self.assertEqual(sorted(self.t["scenes"]), sorted(self.HAVE))
        for name, s in self.t["scenes"].items():
            self.assertGreater(s["splits"]["train"]["totals"]["files"], 0, name)
            self.assertGreater(s["splits"]["train"]["totals"]["seconds"], 0, name)

    def test_meeting_room_is_the_but_offices(self):
        """会议室噪声 = BUT 三间办公室的底噪（第四批）：来源是新名字 `-b4`，文件只落在 L207 · L212 · Q301。"""
        src = self.t["scenes"]["meeting-room"]["splits"]["train"]["sources"]
        self.assertEqual([x["source"] for x in src], ["but-reverbdb-silence-b4"])
        _, rows = split.load_split("but-reverbdb-silence-b4", out_dir=HERE / "splits")
        offices = [r for r in rows if r["split"] == "train" and re.search(r"/VUT_FIT_(L207|L212|Q301)/", r["path"])]
        self.assertEqual(src[0]["files"], len(offices))
        self.assertEqual({r["path"].split("/")[2] for r in offices}, {"VUT_FIT_L207", "VUT_FIT_L212", "VUT_FIT_Q301"})

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

    def test_markdown_has_two_durations(self):
        md = scene_table.render_md(self.t)
        self.assertIn("| 来源 | 份 | 条数 | 时长（小时，按条数） | 不同内容（小时） | 单位数 | 许可 |", md)
        self.assertIn("same_content", md)
        self.assertIn("scene_notes.json", md)

    def test_canteen_unique_seconds_is_one_sixteenth(self):
        # DEMAND 是 16 声道阵列：同一环境的 16 个声道是同一次录音，不同内容 ≈ 总时长 / 16
        tt = self.t["scenes"]["canteen"]["splits"]["train"]["totals"]
        self.assertAlmostEqual(tt["unique_seconds"], tt["seconds"] / 16, delta=1.0)

    def test_rail_air_cabin_unique_equals_seconds(self):
        # CochlScene 不在 same_content 里：每个文件都是不同内容
        for sp in ("train", "dev"):
            tt = self.t["scenes"]["rail-air-cabin"]["splits"][sp]["totals"]
            self.assertAlmostEqual(tt["unique_seconds"], tt["seconds"], delta=0.01)

    def test_markdown_has_each_scene(self):
        md = scene_table.render_md(self.t)
        for name in self.HAVE:
            self.assertIn(name, md)
        self.assertIn(BUILD.name, md)
        self.assertIn("已知限制：", md)

    def test_committed_markdown_is_the_cli_output(self):
        """同一张表的第二种表示（提交进仓的 markdown）必须由 `scene_table.py` 的标准输出逐字节复现：
        否则那份文件是手写的，或者切分表 / 配方变了而文件没重生成。这里直接跑那条命令，比的是 stdout 本身。"""
        self.assertTrue(SCENES_MD.is_file(),
                        f"没有 {SCENES_MD.name}：跑 python3 scene_table.py --build builds/{BUILD.name} > {SCENES_MD.name}")
        out = subprocess.run([sys.executable, str(HERE / "scene_table.py"), "--build", f"builds/{BUILD.name}"],
                             capture_output=True, check=True).stdout
        self.assertEqual(SCENES_MD.read_bytes(), out,
                         f"{SCENES_MD.name} 与 `python3 scene_table.py --build builds/{BUILD.name}` 的输出不同"
                         "：重跑那条命令、把输出重定向到同一个文件再提交")


class Size(unittest.TestCase):
    def test_new_py_under_600(self):
        for name in ("scene_table.py", "tests/test_scene_table.py"):
            n = len((HERE / name).read_text(encoding="utf-8").splitlines())
            self.assertLessEqual(n, 600, f"{name} 有 {n} 行")


if __name__ == "__main__":
    unittest.main()
