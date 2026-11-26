"""训练前清单第 3 条的证据表（build_evidence.py）：真产物逐字节复现 · 真仓里 6 个 build 全部通过 · 三条夹具。

真产物部分：提交进仓的 `evidence-bc0e9e7b566421f0.md` 必须由 `build_evidence.py --check` 认下来（逐字节），
且真仓里每份配方的每一列都是「是」。夹具部分：一份最小数据目录（1 张切分表 · 1 个来源 · 1 个目标场景 ·
基准 + 5 份消融配方各 1 个 build）里，把三种该红的情况各撞一次：
一个检查不过 → 「全部通过」变「否」且退 1；一份配方对上 2 个 build → 停；变体改了两个旋钮 → 那一列「否」。
配方文件一律用 `ablation_recipes` 的正本函数生成，不手写 5 份。
"""
import contextlib
import hashlib
import io
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import ablation_recipes  # noqa: E402
import build_evidence  # noqa: E402
import loader  # noqa: E402

EVIDENCE = HERE / "evidence-bc0e9e7b566421f0.md"  # 提交进仓的那份（= `build_evidence.py` 的标准输出）
BASE = {
    "knobs": {
        "target_scene": {"on": True, "scene": ["canteen"], "frac": 0.667, "src": "夹具"},
        "replay": {"on": True, "frac": 0.333, "src": "夹具"},
        "eq": {"on": True, "src": "夹具"},
        "clip": {"on": True, "src": "夹具"},
        "n_noises": {"on": True, "src": "夹具"},
        "reverb": {"on": True, "src": "夹具"},
        "high_snr": {"on": False, "src": "夹具"},
    },
    "noise_pools": {"canteen": [{"source": "noise-a", "regex": "/a/"}]},
}


def sha(b):
    return hashlib.sha256(b).hexdigest()


def run_main(argv):
    """跑 main()，拿回 (退出码, 标准输出, 标准错误)：测试里不让它的输出混进测试报告。"""
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        rc = build_evidence.main(argv)
    return rc, out.getvalue(), err.getvalue()


class Fixture(unittest.TestCase):
    """临时目录里一套最小数据：1 张切分表 · 1 个来源 · 1 个目标场景 · 基准 + 5 份消融配方各 1 个 build。"""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.table = self.write_table()
        self.write_recipes()

    def tearDown(self):
        self._tmp.cleanup()

    def write_table(self, rows=None):
        """一张最小切分表 + 摘要，返回正文哈希（build 记录的 split_tables 就记这个）。"""
        rows = [{"path": "raw/a/x.wav", "split": "train", "unit": "u1"}] if rows is None else rows
        body = "".join(json.dumps(r, sort_keys=True, ensure_ascii=False) + "\n" for r in rows).encode()
        d = self.root / "splits"
        d.mkdir(parents=True, exist_ok=True)
        (d / "noise-a.jsonl").write_bytes(body)
        (d / "noise-a.json").write_text(json.dumps({"source": "noise-a", "jsonl_sha256": sha(body)}), encoding="utf-8")
        return sha(body)

    def write_recipes(self, base=None):
        """基准配方 + 5 份消融配方（`ablation_recipes` 的正本函数生成，不手写）。"""
        body = ablation_recipes.render(base or BASE)
        (self.root / "recipe.json").write_bytes(body)
        d = self.root / "recipes" / "ablation"
        d.mkdir(parents=True, exist_ok=True)
        for fn, data in ablation_recipes.render_all(body).items():
            (d / fn).write_bytes(data)

    def write_build(self, recipe_bytes, **over):
        """按一份配方字节写一条 build 记录（身份字段算 build_id），`over` 改某几个字段。

        改身份字段（`index_sha256` · `split_tables` · `shards` · `code_sha256`）会重新算 build_id——
        所以改 `index_sha256` 就能给同一份配方造出第二个 build（两边分片相同）。
        """
        parts = {"index_sha256": sha(recipe_bytes + b"idx"), "split_tables": {"noise-a": self.table},
                 "shards": {"noise-a": {"train": {"noise-a-train-00000.tar": "f" * 64}}},
                 "recipe_sha256": sha(recipe_bytes), "code_sha256": build_evidence.code_hashes()}
        for k in list(over):
            if k in loader.ID_KEYS:
                parts[k] = over.pop(k)
        rec = {"build_id": loader.build_id_of(parts), **parts,
               "leak_check": {"against": "exp013 build 测试 exclude.json", "ok": True, "n_hits": 0,
                              "per_key": {"demand_segments": 0}},
               "leak_checked_sources": ["noise-a"], "valid": True, "missing": [],
               "noise_scenes": {"train": ["canteen"], "dev": ["canteen"]}}
        rec.update(over)
        d = self.root / "builds" / rec["build_id"]
        d.mkdir(parents=True, exist_ok=True)
        (d / "build.json").write_text(json.dumps(rec, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
        return rec

    def write_all(self, **over):
        """按盘上的 6 份配方各写一条 build 记录（`over` 只加在基准那一份上）。"""
        recs = {}
        for name, path in build_evidence.recipe_files(self.root):
            recs[name] = self.write_build(path.read_bytes(), **over if name == "recipe.json" else {})
        return recs

    def report(self):
        return build_evidence.report(self.root)


class FixturePasses(Fixture):
    def test_all_checks_true(self):
        self.write_all()
        text, rows = self.report()
        self.assertEqual(len(rows), 1 + len(ablation_recipes.VARIANTS))
        for r in rows:
            bad = {k: why for k, (ok, why) in r["checks"].items() if not ok}
            self.assertEqual(bad, {}, f"{r['recipe']}：{bad}")
        self.assertIn("全部通过：是", text)
        self.assertNotIn("**否**", text)
        self.assertIn("基准 build", text.splitlines()[0])
        self.assertIn("不手抄", text)  # 一句话说明它是生成的，不是抄的

    def test_variant_column_names_its_knob(self):
        """每一行的「只差一个旋钮」要写出翻的是哪个旋钮、翻到哪个值（原因也是给人看的）。"""
        self.write_all()
        _text, rows = self.report()
        why = {r["recipe"]: r["checks"]["one_knob"][1] for r in rows}
        for name, knob, on in ablation_recipes.VARIANTS:
            self.assertIn(f"knobs.{knob}.on → {on}", why[f"recipes/ablation/{name}.json"])
        self.assertIn("与 recipe.json 相同", why["recipe.json"])


class FixtureStops(Fixture):
    def test_one_failing_check_flips_to_no_and_exit_1(self):
        """missing 不空：那一列变「否」，表尾「全部通过：否」，退 1（不静默放过）。"""
        self.write_all(missing=["noise-b"])
        rc, out, _err = run_main(["--data-dir", str(self.root)])
        self.assertEqual(rc, 1)
        self.assertIn("全部通过：否", out)
        self.assertIn("**否**", out)
        self.assertIn("不通过的项：", out)
        self.assertIn("noise-b", out)  # 原因写清了差在哪
        _text, rows = self.report()
        self.assertFalse(rows[0]["checks"]["missing"][0])
        self.assertTrue(all(ok for k, (ok, _w) in rows[1]["checks"].items()))  # 其余行照旧全过

    def test_two_candidate_builds_for_one_recipe_stop(self):
        """同一份配方对上 2 个 build（分片相同）：分不清是哪一份，必须停并写清是哪个配方。"""
        recs = self.write_all()
        path = self.root / "recipes" / "ablation" / "no-clip.json"
        second = self.write_build(path.read_bytes(), index_sha256="a" * 64)  # 身份字段改了 → 另一个 build_id
        self.assertNotEqual(second["build_id"], recs["recipes/ablation/no-clip.json"]["build_id"])
        with self.assertRaises(SystemExit) as e:
            self.report()
        self.assertIn("no-clip", str(e.exception))
        self.assertIn(second["build_id"], str(e.exception))

    def test_variant_that_changes_two_knobs_is_false(self):
        """变体改了两个旋钮：只有「只差一个旋钮」这一列变否，其余列照旧是（否则说不清是配方错还是 build 错）。"""
        path = self.root / "recipes" / "ablation" / "no-eq.json"
        self.write_all()
        d = json.loads(path.read_text(encoding="utf-8"))
        d["knobs"]["clip"]["on"] = False  # 第二个旋钮：_ablation 还只记着 eq
        path.write_text(json.dumps(d, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
        self.write_build(path.read_bytes())  # 按新字节重新登记（旧记录留着，不再被任何配方认领）
        text, rows = self.report()
        row = {r["recipe"]: r for r in rows}["recipes/ablation/no-eq.json"]
        self.assertFalse(row["checks"]["one_knob"][0])
        self.assertIn("knobs.clip.on", row["checks"]["one_knob"][1])
        self.assertTrue(all(ok for k, (ok, _w) in row["checks"].items() if k != "one_knob"))
        self.assertIn("全部通过：否", text)
        rc, out, _err = run_main(["--data-dir", str(self.root)])
        self.assertEqual(rc, 1)
        self.assertIn("只差一个旋钮", out)

    def test_stray_ablation_file_stops(self):
        """目录里多一份清单没登记的配方就停：那种文件没有旋钮归属，核不了「只差一个旋钮」。"""
        (self.root / "recipes" / "ablation" / "extra.json").write_text("{}", encoding="utf-8")
        with self.assertRaises(SystemExit) as e:
            build_evidence.recipe_files(self.root)
        self.assertIn("extra.json", str(e.exception))


class CheckFile(Fixture):
    """`--check` 与 `--out`：写一份、核一份；改一行就退 1 并指出第几行。"""

    def test_out_then_check_round_trip(self):
        self.write_all()
        out = self.root / "evidence.md"
        self.assertEqual(run_main(["--data-dir", str(self.root), "--out", str(out)])[0], 0)
        rc, stdout, _err = run_main(["--data-dir", str(self.root), "--check", str(out)])
        self.assertEqual(rc, 0)
        self.assertIn("逐字节一致", stdout)

    def test_check_reports_the_first_differing_line(self):
        self.write_all()
        out = self.root / "evidence.md"
        run_main(["--data-dir", str(self.root), "--out", str(out)])
        lines = out.read_text(encoding="utf-8").splitlines()
        lines[-1] = "全部通过：否"  # 改表尾那一行
        out.write_text("\n".join(lines) + "\n", encoding="utf-8")
        rc, _out, err = run_main(["--data-dir", str(self.root), "--check", str(out)])
        self.assertEqual(rc, 1)
        self.assertIn(f"第 {len(lines)} 行", err)
        self.assertIn("重跑", err)

    def test_check_without_the_file_exits_1(self):
        self.write_all()
        rc, _out, err = run_main(["--data-dir", str(self.root), "--check", str(self.root / "没有这份.md")])
        self.assertEqual(rc, 1)
        self.assertIn("没有", err)


class Product(unittest.TestCase):
    """真仓里那 6 个 build：每一列都是「是」，且提交进仓的 md 就是它算出来的那一份。"""

    @classmethod
    def setUpClass(cls):
        cls.text, cls.rows = build_evidence.report()

    def test_every_column_is_true(self):
        self.assertEqual(len(self.rows), 1 + len(ablation_recipes.VARIANTS))
        for r in self.rows:
            bad = {k: why for k, (ok, why) in r["checks"].items() if not ok}
            self.assertEqual(bad, {}, f"{r['recipe']}（build {r['build'].get('build_id')}）：{bad}")
        self.assertIn("全部通过：是", self.text)

    def test_recipes_are_the_baseline_and_the_five_variants(self):
        self.assertEqual([r["recipe"] for r in self.rows], [n for n, _p in build_evidence.recipe_files(HERE)])

    def test_committed_file_is_the_generated_one(self):
        """提交进仓的 md 与 `--check` 逐字节比：手改一个字节就红（第二处表示必须机器核）。"""
        r = subprocess.run([sys.executable, str(HERE / "build_evidence.py"), "--check", str(EVIDENCE)],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("逐字节一致", r.stdout)


class Size(unittest.TestCase):
    def test_new_py_under_600(self):
        for name in ("build_evidence.py", "tests/test_build_evidence.py"):
            n = len((HERE / name).read_text(encoding="utf-8").splitlines())
            self.assertLessEqual(n, 600, f"{name} 有 {n} 行")


if __name__ == "__main__":
    unittest.main()
