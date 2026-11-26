"""配方消融：从 recipe.json 生成 5 份只翻一个旋钮的副本（2026-11-26 大总管交代）。

用途：训练侧要做单旋钮消融，每次只关/开一个旋钮，其余逐字段相同。正本是 recipe.json；
ablation_recipes.py 的 VARIANTS 是这 5 份的唯一清单，落盘的 JSON 由它生成、由这里核对。

- 每份只改 knobs.<knob>.on，并在顶层记 _ablation（基准字节的 sha256 · 旋钮 · 目标值），方便对账
- 改了 recipe.json 而没重新生成：ablation_recipes.py --check 报不一致，本文件同时红
- 跑法（在 data/ 下）：python3 -m unittest discover -s tests -t .
"""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import ablation_recipes as ab  # noqa: E402
import loader  # noqa: E402

BASE = HERE / "recipe.json"
OUT = HERE / "recipes" / "ablation"


def diff_paths(a, b, prefix=""):
    """列出两棵解析后的 JSON 不同的路径（只报叶子）。"""
    out = []
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b)):
            p = f"{prefix}.{k}" if prefix else k
            if k not in a:
                out.append(p + " (只在变体里)")
            elif k not in b:
                out.append(p + " (只在基准里)")
            else:
                out += diff_paths(a[k], b[k], p)
    elif isinstance(a, list) and isinstance(b, list) and len(a) == len(b):
        for i, (x, y) in enumerate(zip(a, b)):
            out += diff_paths(x, y, f"{prefix}[{i}]")
    elif a != b or type(a) is not type(b):
        out.append(prefix)
    return out


class Variants(unittest.TestCase):
    def setUp(self):
        self.base_bytes = BASE.read_bytes()
        self.base = json.loads(self.base_bytes)

    def test_variants_list_is_the_only_place(self):
        """清单顺序与名字固定：训练侧按名字取文件。"""
        self.assertEqual(
            ab.VARIANTS,
            [
                ("no-eq", "eq", False),
                ("no-clip", "clip", False),
                ("no-n_noises", "n_noises", False),
                ("no-reverb", "reverb", False),
                ("high_snr-on", "high_snr", True),
            ],
        )

    def test_exactly_five_files_with_expected_names(self):
        self.assertTrue(OUT.is_dir(), f"{OUT} 不存在：先跑 python3 ablation_recipes.py")
        names = sorted(p.name for p in OUT.iterdir())
        self.assertEqual(names, sorted(f"{n}.json" for n, _, _ in ab.VARIANTS))

    def test_check_passes_on_committed_files(self):
        """落盘的 5 份与当前 recipe.json 逐字节一致（改了配方忘了重新生成就红）。"""
        self.assertEqual(ab.check(BASE, OUT), [])

    def test_each_variant_flips_exactly_one_knob(self):
        want_sha = hashlib.sha256(self.base_bytes).hexdigest()
        for name, knob, on in ab.VARIANTS:
            path = OUT / f"{name}.json"
            with self.subTest(name=name):
                r = loader.load_recipe(path)  # 每份都要能当配方读
                meta = r.pop("_ablation")
                self.assertEqual(meta["base_sha256"], want_sha)
                self.assertEqual(meta["knob"], knob)
                self.assertIs(meta["on"], on)
                self.assertEqual(diff_paths(r, self.base), [f"knobs.{knob}.on"])
                self.assertIs(r["knobs"][knob]["on"], on)
                # 其余字段逐字段相同（含 src、noise_pools、speech）
                r2 = json.loads(json.dumps(r))
                r2["knobs"][knob]["on"] = self.base["knobs"][knob]["on"]
                self.assertEqual(r2, self.base)

    def test_variant_renders_deterministically(self):
        d = ab.variant(self.base_bytes, "eq", False)
        b = ab.render(d)
        self.assertEqual(b, ab.render(json.loads(b)))
        self.assertTrue(b.endswith(b"\n"))
        self.assertEqual(b, b.decode("utf-8").encode("utf-8"))  # 中文按 utf-8 原样落盘
        self.assertNotIn(b"\\u", b)

    def test_variant_keeps_every_other_field(self):
        d = ab.variant(self.base_bytes, "clip", False)
        self.assertEqual(d["version"], self.base["version"])
        self.assertEqual(d["noise_pools"], self.base["noise_pools"])
        self.assertEqual(d["knobs"]["clip"]["src"], self.base["knobs"]["clip"]["src"])
        self.assertEqual(d["knobs"]["clip"]["prob"], self.base["knobs"]["clip"]["prob"])

    def test_variant_rejects_knob_already_at_target(self):
        """翻过去和基准一样就不叫消融：明确报错，不静默产出一份重复文件。"""
        with self.assertRaises(ValueError):
            ab.variant(self.base_bytes, "eq", True)  # 基准 eq.on 已是 True
        with self.assertRaises(ValueError):
            ab.variant(self.base_bytes, "high_snr", False)  # 基准 high_snr.on 已是 False

    def test_variant_rejects_missing_knob(self):
        with self.assertRaises(ValueError):
            ab.variant(self.base_bytes, "no_such_knob", True)

    def test_variant_rejects_knob_without_on(self):
        b = json.loads(self.base_bytes)
        b["knobs"]["eq"].pop("on")
        with self.assertRaises(ValueError):
            ab.variant(json.dumps(b).encode(), "eq", False)


class Check(unittest.TestCase):
    def test_one_byte_change_in_base_is_reported(self):
        """基准改一个字节：5 份全部报不一致（--base / --out-dir 用于临时副本）。"""
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            base = t / "recipe.json"
            out = t / "out"
            out.mkdir()
            for name, _, _ in ab.VARIANTS:
                shutil.copy(OUT / f"{name}.json", out / f"{name}.json")
            b = BASE.read_bytes().replace(b'"version": 1', b'"version": 2')
            self.assertEqual(len(b), len(BASE.read_bytes()))
            base.write_bytes(b)
            bad = ab.check(base, out)
            self.assertEqual({n for n, _ in bad}, {n for n, _, _ in ab.VARIANTS})
            self.assertEqual({why for _, why in bad}, {"differs"})

    def test_missing_file_is_reported(self):
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            out = t / "out"
            out.mkdir()
            for name, _, _ in ab.VARIANTS[:-1]:
                shutil.copy(OUT / f"{name}.json", out / f"{name}.json")
            bad = ab.check(BASE, out)
            self.assertEqual(bad, [(ab.VARIANTS[-1][0], "missing")])

    def test_write_all_regenerates_into_empty_dir(self):
        with tempfile.TemporaryDirectory() as t:
            out = Path(t) / "deep" / "out"
            ab.write_all(BASE, out)
            self.assertEqual(ab.check(BASE, out), [])
            for name, _, _ in ab.VARIANTS:
                self.assertTrue((out / f"{name}.json").is_file())


class Size(unittest.TestCase):
    def test_new_py_files_under_600_lines(self):
        for p in (HERE / "ablation_recipes.py", Path(__file__)):
            with self.subTest(path=p.name):
                self.assertLessEqual(len(p.read_text().splitlines()), 600)


if __name__ == "__main__":
    unittest.main()
