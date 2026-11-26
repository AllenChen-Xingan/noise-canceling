"""CochlScene 3 类（CrowdedIndoor · Subway · SubwayStation）登记成第二批训练噪声（D106 · 2026-11-25）。

为什么按录音人切：CochlScene 是众包手机录音，同一个录音人有很多片段；官方的 Train / Val / Test 三份
录音人互不重叠（download-list-2.md 2.1 实读 Data_info.tsv），但我们照旧用自己的切分器，只留一个切分正本，
所以要用 unit_regex 把同一录音人的片段并成一个单位（`unit_regex_required`：认不出来就停，不许退回按文件切）。

数字与字符串一律从正本读，不在测试里抄副本：
- `download-list-2.md` 2.1 节：3 类的条数、每类补哪个场景、分卷 md5（实读 Zenodo 7080122 页面）
- `split_rules.json` 的 `cochlscene`：通配符 · 单位正则 · 期望条数 · 比例
- `recipe.json` 的 `noise_pools`：场景 → 来源 + 路径正则
- `sources.json` 的 `A6`：手工下载条目（分卷包不能只下 3 类）
夹具在临时目录现造（12 个 0.1 s 的 16 kHz wav，合计不到 10 KB），不用真数据。
"""
import copy
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import fetch  # noqa: E402
import loader  # noqa: E402
import shard  # noqa: E402
import split  # noqa: E402

MD = HERE / "download-list-2.md"
SRC = "cochlscene"
GRP = "A6"
FOLDERS = ("Train", "Val", "Test")

# 第一批的 7 个噪声池（改动前逐条抄下来，防这次追加时顺手改了旧条目）；只有这份允许硬编码
OLD_POOLS = {
    "street-outdoor": [{"source": "qut-noise", "regex": "(?i)street"}, {"source": "tum-wind", "regex": ".", "wind": True}, {"source": "demand-train", "regex": "STRAFFIC"}],
    "cafe": [{"source": "qut-noise", "regex": "(?i)cafe"}, {"source": "demand-train", "regex": "SCAFE|PRESTO"}],
    "canteen": [{"source": "demand-train", "regex": "PCAFETER|PRESTO"}],  # 第四批加 PRESTO（两间餐厅）
    "home-family": [{"source": "qut-noise", "regex": "(?i)home"}, {"source": "demand-train", "regex": "DKITCHEN"}],
    "car-passenger": [{"source": "qut-noise", "regex": "(?i)car"}],
    "bus-metro-cabin": [{"source": "demand-train", "regex": "TBUS"}],
    "library-classroom": [{"source": "qut-noise", "regex": "(?i)reverb"}],
}


def rule():
    return split.load_rules()["sources"][SRC]


def md_section21():
    return MD.read_text().split("### 2.1")[1].split("### 2.2")[0]


def md_classes():
    """2.1 的类表：类名 → 条数 · 补哪些场景（场景名取表里的英文小写连字符词）。"""
    out = {}
    for cls, n, cell in re.findall(r"^\| ([A-Z][A-Za-z]+)（[^）]*） \| ([\d,]+) \| \d+ \| \d+ % \| (.+?) \|$", md_section21(), re.M):
        out[cls] = {"files": int(n.replace(",", "")), "scenes": re.findall(r"[a-z]+(?:-[a-z]+)+", cell)}
    return out


def md_md5():
    """2.1 的分卷表：文件名 → md5。"""
    return dict(re.findall(r"^\| (CochlScene\.\S+) \| [\d.]+ (?:GB|MB) \| ([0-9a-f]{32}) \|$", md_section21(), re.M))


def cpath(folder, cls, name):
    """造一条匹配规则 glob 的路径：数据根与层级照 glob（含 CochlScene 这层），只换官方份 · 类 · 文件名（夹具用）。"""
    return "/".join(rule()["glob"][0].split("/")[:-3] + [folder, cls, name])


def sample_paths():
    """每类一条真实形态的路径（类别名取自规则 glob 的倒数第二段，文件名用夹具名）。"""
    out = {}
    for p in rule()["glob"]:
        cls = p.split("/")[-2]
        out[cls] = cpath("Train", cls, f"{cls}_user0123_14983413_000.wav")
    return out


def tone(path, n=1600, sr=16000):
    path.parent.mkdir(parents=True, exist_ok=True)
    sf.write(str(path), np.sin(np.arange(n) * 0.05).astype("float32") * 0.1, sr)


class Units(unittest.TestCase):
    def test_same_recorder_one_unit(self):
        """同一个录音人在 Train / Val / Test 三个官方份里的片段 → 我们的同一个单位（不用官方的份）。"""
        u = {split.unit_of(cpath(d, "Subway", "Subway_user0123_14983413_000.wav"), rule()) for d in FOLDERS}
        self.assertEqual(u, {"user0123"})

    def test_three_classes_share_the_rule(self):
        """三类文件名都是 `<类>_userNNNN_<编号>_<序号>.wav`，同一个人跨类仍是一个单位。"""
        u = {split.unit_of(cpath("Train", c, f"{c}_user0123_14983413_000.wav"), rule()) for c in md_classes()}
        self.assertEqual(u, {"user0123"})

    def test_different_recorders_differ(self):
        a = split.unit_of(cpath("Train", "Subway", "Subway_user0123_14983413_000.wav"), rule())
        b = split.unit_of(cpath("Train", "Subway", "Subway_user0124_14983413_000.wav"), rule())
        self.assertNotEqual(a, b)

    def test_recorder_required(self):
        """`unit_regex_required`：认不出录音人就停，不许退回按文件切（那样同一个录音人会跨份）。"""
        self.assertTrue(rule()["unit_regex_required"])
        with self.assertRaises(SystemExit) as cm:
            split.unit_of(cpath("Train", "Subway", "Subway_noid_1.wav"), rule())
        self.assertIn("Subway_noid_1.wav", str(cm.exception))

    def test_fallback_still_allowed_without_flag(self):
        """没有 unit_regex_required 的旧规则行为不变（TUM 风噪）：认不出就按文件当单位。"""
        r = split.load_rules()["sources"]["tum-wind"]
        f = "train-raw/tum-wind/artifitial_wind_noise/train/const_01.wav"
        self.assertEqual(split.unit_of(f, r), f)

    def test_rule_regex_is_the_one_we_describe(self):
        """单位正则不是空匹配：带 `/` 的完整路径才认得，而且非要有 `user` 前缀。"""
        rx = re.compile(rule()["unit_regex"][0]["match"])
        self.assertIsNotNone(rx.search(cpath("Train", "Subway", "Subway_user0123_14983413_000.wav")))
        self.assertIsNone(rx.search(cpath("Train", "Subway", "Subway_0123_14983413_000.wav")))
        self.assertIsNone(rx.search("Subway_user0123_14983413_000.wav"))


class Glob(unittest.TestCase):
    def test_matches_only_the_three_classes(self):
        """规则只收这 3 类；同一棵树里别的类（Bus 等）不进第一批。"""
        md = md_classes()
        with tempfile.TemporaryDirectory() as d:
            for cls in list(md) + ["Bus", "Cafe"]:
                tone(Path(d) / cpath("Train", cls, f"{cls}_user0123_1_000.wav"))
            got = split.list_files(Path(d), rule())
        self.assertEqual(sorted(Path(x).parts[-2] for x in got), sorted(md))

    def test_three_folders_all_collected(self):
        """官方 Train / Val / Test 三个文件夹都收（我们自己重切，不用官方的份）。"""
        with tempfile.TemporaryDirectory() as d:
            for d0 in FOLDERS:
                tone(Path(d) / cpath(d0, "Subway", "Subway_user0123_1_000.wav"))
            got = split.list_files(Path(d), rule())
        self.assertEqual(len(got), len(FOLDERS))


class Expect(unittest.TestCase):
    def test_cloud_count_matches_download_list(self):
        """期望条数 = 2.1 类表三条相加（清单与规则两处表示交叉校验）。"""
        e = rule()["expect"]
        self.assertTrue(e.get("src"))
        self.assertEqual(e["files"], sum(v["files"] for v in md_classes().values()))

    def test_wrong_count_stops(self):
        """真规则的条数配夹具数量的文件 → 停（通配符写错时不写切分表）。"""
        with tempfile.TemporaryDirectory() as d:
            fs = []
            for cls in md_classes():
                f = Path(d) / cpath("Train", cls, f"{cls}_user0123_1_000.wav")
                tone(f)
                fs.append(str(f))
            with self.assertRaises(SystemExit):
                split.check_expect(SRC, rule(), fs)


class SplitTree(unittest.TestCase):
    """临时夹具树上跑真切分器：4 个录音人 × 3 类，散在三个官方文件夹里。"""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.files = []
        for i, user in enumerate(("user0001", "user0002", "user0003", "user0004")):
            for j, cls in enumerate(sorted(md_classes())):
                p = self.root / cpath(FOLDERS[(i + j) % 3], cls, f"{cls}_{user}_14983413_{j:03d}.wav")
                tone(p)
                self.files.append(p)
        tone(self.root / cpath("Train", "Bus", "Bus_user0001_14983413_000.wav"))  # 别的类：不该收
        self.r = copy.deepcopy(rule())
        self.r["expect"]["files"] = len(self.files)  # 夹具只有 12 个；云上数目另有 Expect 两条

    def tearDown(self):
        self.tmp.cleanup()

    def run_src(self):
        return split.split_source(self.root, SRC, self.r, split.load_rules()["seed"], split.load_rules()["splits"], workers=2)

    def test_each_recorder_one_split(self):
        rows = self.run_src()
        self.assertEqual(len(rows), len(self.files))  # Bus 不在里面
        by = {}
        for r in rows:
            by.setdefault(r["unit"], set()).add(r["split"])
        self.assertEqual(len(by), 4)
        self.assertTrue(all(len(v) == 1 for v in by.values()), by)
        self.assertTrue(all(len(r["sha256"]) == 64 for r in rows))

    def test_only_train_and_dev(self):
        """D106：只作训练噪声，不给终测（final）与测试份（d63）。"""
        rows = self.run_src()
        allowed = {k for k, v in rule()["frac"].items() if v > 0}
        self.assertEqual(allowed, {"train", "dev"})
        self.assertLessEqual({r["split"] for r in rows}, allowed)

    def test_deterministic(self):
        a = [(r["path"], r["split"]) for r in self.run_src()]
        b = [(r["path"], r["split"]) for r in self.run_src()]
        self.assertEqual(a, b)


class Table(unittest.TestCase):
    """云上跑完 split.py 的产物已带回进仓（cloud-batch2.md 第 4 步）：规则 · 摘要 · 正文三处要自洽。
    反过来也查：顶层 `未建表` 里还留着这一条，说明音频还在云上、本机没切，表和标记不许同时在。"""

    def check(self, rules_path=None):
        """cochlscene 不在 `未建表` 里 · 表能按摘要里的哈希读回 · 条数与单位对得上 · 只有 train / dev。
        rules_path 只给回归测试用（临时副本里放回标记，看这条检查会不会红）。返回 (摘要, 行)。"""
        R = split.load_rules(rules_path) if rules_path else split.load_rules()
        self.assertNotIn(SRC, R.get("未建表", {}),
                         "顶层 `未建表` 里还有 cochlscene：这一条表示音频还在云上、本机没切，拿回表后要删掉（别科的来源可以留着）")
        r = R["sources"][SRC]
        s, rows = split.load_split(SRC)  # 核 jsonl 的 sha256
        self.assertEqual(s["files"], r["expect"]["files"], "摘要条数与规则期望不同")
        self.assertEqual(s["files"], len(rows))
        self.assertEqual(s["rule_sha256"], split.rule_sha(r), "表是按旧规则切的")
        self.assertEqual(s["seed"], R["seed"])
        return s, rows

    def test_table_present_and_no_pending_marker(self):
        s, _ = self.check()
        self.assertEqual(rule()["group"], GRP)
        self.assertEqual(s["group"], GRP)

    def test_units_are_the_recorder_in_the_path(self):
        """单位就是路径里的 `userNNNN`（按录音人切的正本），和切分器的取法逐个相同。"""
        _, rows = self.check()
        self.assertTrue(rows)
        for r in rows:
            self.assertRegex(r["unit"], r"^user\d+$", r["path"])
            self.assertIn(r["unit"], Path(r["path"]).name, r["path"])
            self.assertEqual(r["unit"], split.unit_of(r["path"], rule()), r["path"])

    def test_no_recorder_in_two_splits(self):
        _, rows = self.check()
        by = {}
        for r in rows:
            by.setdefault(r["unit"], set()).add(r["split"])
        two = {u: sorted(v) for u, v in by.items() if len(v) > 1}
        self.assertEqual(two, {}, f"同一录音人跨了份：{sorted(two)[:5]}")

    def test_only_train_and_dev(self):
        """D106：只作训练噪声，不给终测（final）与测试份（d63）。"""
        _, rows = self.check()
        self.assertEqual({k for k, v in rule()["frac"].items() if v > 0}, {"train", "dev"})
        self.assertLessEqual({r["split"] for r in rows}, {"train", "dev"})

    def test_marker_still_means_pending(self):
        """上面那条「cochlscene 不在 `未建表`」不是空检查：临时副本里放回这一条，check 必须红（仓库里的文件不动）。"""
        R = json.loads(split.RULES.read_text())
        R["未建表"] = {SRC: "音频还在云上"}
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "split_rules.json"
            p.write_text(json.dumps(R, ensure_ascii=False))
            with self.assertRaises(AssertionError) as cm:
                self.check(p)
        self.assertIn("未建表", str(cm.exception))


class Fetch(unittest.TestCase):
    def run_cli(self, *args):
        return subprocess.run([sys.executable, str(HERE / "fetch.py"), *args], capture_output=True, text=True)

    def test_refuses_manual(self):
        """分卷包不能只下 3 类：A6 不走 fetch.py，--only A6 与 --head 都要停在「手工」上。"""
        manual = fetch.load_sources()[GRP]["manual"]
        for args in (("--only", GRP), ("--head", "--only", GRP)):
            r = self.run_cli(*args)
            self.assertNotEqual(r.returncode, 0, args)
            self.assertIn("手工", r.stderr, args)
            self.assertIn(manual, r.stderr, args)

    def test_other_groups_unaffected(self):
        """没有 manual 的组照旧：--plan 只列清单、不联网。"""
        r = self.run_cli("--plan", "--only", "A1")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("A1", r.stdout)

    def test_md5_matches_download_list(self):
        """sources.json 的 6 个分卷 md5 与 download-list-2.md 2.1 的分卷表逐条相同。"""
        md = md_md5()
        files = {f["name"]: f for f in fetch.load_sources()[GRP]["files"]}
        self.assertEqual(set(files), set(md))
        for n, h in md.items():
            self.assertEqual(files[n]["md5"], h, n)

    def test_entries_shape(self):
        """手工条目的字段：没有字节数与 sha256（下载不是 fetch.py 做的），地址按 Zenodo 记录拼，解压不归 fetch.py。"""
        g = fetch.load_sources()[GRP]
        self.assertEqual(g["role"], "train")
        self.assertIn("manual", g)
        for f in g["files"]:
            self.assertIsNone(f["bytes"])
            self.assertIsNone(f["sha256"])
            self.assertEqual(f["extract"], "none")
            self.assertEqual(f["url"], f"https://zenodo.org/records/7080122/files/{f['name']}?download=1")
        # 落盘位置与切分规则的通配符对得上（两处表示交叉校验）
        dest = {f["dest"] for f in g["files"]}
        self.assertEqual(len(dest), 1)
        base = dest.pop() + "/"
        for p in rule()["glob"]:
            self.assertTrue(p.startswith(base), p)


class License(unittest.TestCase):
    def test_by_sa_trainable(self):
        g = fetch.load_sources()[GRP]
        self.assertTrue(g["train_ok"])
        self.assertIn("BY-SA", g["license"])
        self.assertIn("D64", g["attribution"])
        self.assertTrue(g["license_src"])


class Recipe(unittest.TestCase):
    def setUp(self):
        self.r = json.loads((HERE / "recipe.json").read_text())
        self.pools = {k: v for k, v in self.r["noise_pools"].items() if not k.startswith("_")}

    def new_item(self, scene):
        items = [it for it in self.pools[scene] if it["source"] == SRC]
        self.assertEqual(len(items), 1, scene)
        return items[0]

    def scene_to_classes(self):
        """2.1 类表倒过来：场景 → 该由哪些类供给。"""
        out = {}
        for cls, v in md_classes().items():
            for s in v["scenes"]:
                out.setdefault(s, set()).add(cls)
        return out

    def test_old_pools_unchanged(self):
        for k, v in OLD_POOLS.items():
            self.assertEqual(self.pools[k], v, k)

    def test_new_pools_use_cochlscene(self):
        rules = split.load_rules()["sources"]
        for scene in self.scene_to_classes():
            self.assertIn(scene, self.pools, scene)
            self.assertIn(self.new_item(scene)["source"], rules, scene)

    def test_pool_regex_picks_the_right_class(self):
        """每类的片段只落进清单表说的那几个场景（`/Subway/` 不会顺手匹配 `/SubwayStation/`：有结尾斜杠）。"""
        paths = sample_paths()
        for scene, classes in self.scene_to_classes().items():
            rx = re.compile(self.new_item(scene)["regex"])
            for cls, p in paths.items():
                self.assertEqual(bool(rx.search(p)), cls in classes, f"{scene} 对 {cls}")

    def test_subway_regex_does_not_match_station(self):
        p = sample_paths()
        rx = re.compile(self.new_item("rail-air-cabin")["regex"])
        self.assertIsNotNone(rx.search(p["Subway"]))
        self.assertIsNone(rx.search(p["SubwayStation"]))

    def test_note_names_second_batch(self):
        note = self.r["noise_pools"]["_说明"]
        self.assertIn("D106", note)
        self.assertIn("CochlScene", note)


class ShardSources(unittest.TestCase):
    """shard.py 的 `--sources` 默认值曾经是手抄的 5 个来源，第二批加了 CochlScene 就漏了（第二处表示）。
    现在从 recipe.json 导出；这里不借导出函数，另算一遍对账。"""

    OLD = ("librispeech-train-clean-100", "qut-noise", "tum-wind", "openslr28-rir", "demand-train")  # 改动前手抄的那 5 个

    def expected(self):
        """按「人声 → RIR → 噪声池（跳过 `_` 开头的键）」的规矩从 recipe.json 再算一遍。"""
        r = json.loads((HERE / "recipe.json").read_text())
        pools = (it["source"] for k, items in r["noise_pools"].items() if not k.startswith("_") for it in items)
        return list(dict.fromkeys([*r["speech"]["train"], *r["rir"], *pools]))

    def test_derived_from_recipe(self):
        self.assertEqual(shard.default_sources(), self.expected())

    def test_cli_default_is_that_list(self):
        """命令行不给 --sources 时用的就是导出这份（不是另手抄一份）。"""
        self.assertEqual(shard.DEFAULT_SOURCES.split(","), self.expected())

    def test_covers_cochlscene(self):
        self.assertIn(SRC, shard.DEFAULT_SOURCES.split(","))

    def test_old_five_still_first(self):
        """第一批那 5 个来源仍是最前面 5 个（顺序以导出为准，这里不钉次序）：已登记的 build 不受影响。"""
        first = shard.DEFAULT_SOURCES.split(",")[: len(self.OLD)]
        self.assertEqual(len(set(first)), len(self.OLD))
        self.assertEqual(set(first), set(self.OLD))


SPEECH_PATH = "train-raw/librispeech/LibriSpeech/train-clean-100/19/1/19-1-0000.flac"
RIR_PATH = "train-raw/openslr28/RIRS_NOISES/simulated_rirs/smallroom/Room001/Room001-00001.wav"
# 配方里第一批的三个噪声来源各一条成员（路径要能匹配 recipe.json 里的正则）
NOISE_PATHS = {
    "qut-noise": "train-raw/qut-noise/QUT-NOISE/STREET-CITY-1.wav",
    "tum-wind": "train-raw/tum-wind/wind/000_001.wav",
    "demand-train": "vdn-data/demand/STRAFFIC/ch01.wav",
}
# 第三批的 BUT 底噪：路径落不进 meeting-room / hotel-room 的正则（这里只要求来源在索引里，场景另有测试）
BUT_PATH = "train-raw/but-reverbdb/OtherRoom/MicID01/SpkID01_20190503_S/01/silence/silence_16kHz_60sec.v00.wav"


def member(path):
    return {"key": path, "shard": "s-00000.tar", "offset": 0, "bytes": 1024, "src_path": path, "unit": path}


def fake_idx(extra=None):
    """最小分片索引（只填 members，不读音频）：人声 · RIR · 配方里三个第一批噪声来源都有 train 成员；
    第三批的 BUT 底噪 train / dev 各一条（路径不落进它的两个场景正则）；extra 是额外来源 {来源: {份: [成员路径, …]}}。"""
    base = {"librispeech-train-clean-100": {"train": [SPEECH_PATH]}, "openslr28-rir": {"train": [RIR_PATH]},
            **{n: {"train": [p]} for n, p in NOISE_PATHS.items()},
            "but-reverbdb-silence": {"train": [BUT_PATH], "dev": [BUT_PATH]}}
    src = {}
    for name, by_split in {**base, **(extra or {})}.items():
        d = {"kind": "speech" if name.startswith("librispeech") else "rir" if name.endswith("-rir") else "noise"}
        for sp, paths in by_split.items():
            d[sp] = {"members": [member(p) for p in paths], "shards": {}}
        src[name] = d
    return {"sources": src}


class PoolSources(unittest.TestCase):
    """噪声池引用的来源整个不在分片索引里时，只记 missing、没人看：云上分片若漏了 cochlscene，4 个新场景会悄悄消失。"""

    def setUp(self):
        self.recipe = json.loads((HERE / "recipe.json").read_text())

    def pools(self, idx, split_="train"):
        return loader.Pools(HERE, self.recipe, idx, split_)

    def test_source_absent_from_index_stops(self):
        with self.assertRaises(SystemExit) as cm:
            self.pools(fake_idx())  # 索引里没 cochlscene
        msg = str(cm.exception)
        self.assertIn(SRC, msg)
        self.assertIn("transit-hub", msg)

    def test_present_source_fills_its_scenes(self):
        idx = fake_idx({SRC: {"train": [cpath("Train", "SubwayStation", "SubwayStation_user0123_1_000.wav")]}})
        p = self.pools(idx)
        self.assertEqual(len(p.pools["transit-hub"]), 1)
        self.assertEqual(p.missing, [])

    def test_empty_split_still_skipped(self):
        """来源在索引里、只是这一份为空（如 dev）：照旧跳过并记下，不报错。"""
        idx = fake_idx({SRC: {"dev": [cpath("Train", "SubwayStation", "SubwayStation_user0123_1_000.wav")]}})
        p = self.pools(idx)
        self.assertNotIn("transit-hub", p.pools)
        self.assertEqual(p.missing, [SRC])

    def test_noise_scene_index_records_both(self):
        """build.json 里要看得见：每份实际有成员的场景（排序）与缺的来源（排序）。"""
        one = cpath("Train", "SubwayStation", "SubwayStation_user0123_1_000.wav")
        got = loader.noise_scene_index(fake_idx({SRC: {"train": [one], "dev": [one]}}), self.recipe)
        self.assertIn("transit-hub", got["noise_scenes"]["train"])
        self.assertEqual(got["noise_scenes"]["dev"], ["transit-hub"])
        self.assertEqual(got["noise_scenes"]["train"], sorted(got["noise_scenes"]["train"]))
        self.assertEqual(got["missing"], sorted(NOISE_PATHS))  # 三个第一批来源在夹具里没有 dev 成员


if __name__ == "__main__":
    unittest.main()
