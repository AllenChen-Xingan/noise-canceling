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
import split  # noqa: E402

MD = HERE / "download-list-2.md"
SRC = "cochlscene"
GRP = "A6"
FOLDERS = ("Train", "Val", "Test")

# 第一批的 7 个噪声池（改动前逐条抄下来，防这次追加时顺手改了旧条目）；只有这份允许硬编码
OLD_POOLS = {
    "street-outdoor": [{"source": "qut-noise", "regex": "(?i)street"}, {"source": "tum-wind", "regex": ".", "wind": True}, {"source": "demand-train", "regex": "STRAFFIC"}],
    "cafe": [{"source": "qut-noise", "regex": "(?i)cafe"}, {"source": "demand-train", "regex": "SCAFE|PRESTO"}],
    "canteen": [{"source": "demand-train", "regex": "PCAFETER"}],
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


class Pending(unittest.TestCase):
    """切分表还没建（音频在云上，本机造不出表）：登记在这里，test_products 两向查。
    云上跑完 split.py 后把 splits/cochlscene.* 带回进仓，并删掉顶层 `未建表` 里的这一行。"""

    def test_table_pending(self):
        R = split.load_rules()
        self.assertIn(SRC, R["未建表"])
        self.assertEqual(rule()["group"], GRP)


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


if __name__ == "__main__":
    unittest.main()
