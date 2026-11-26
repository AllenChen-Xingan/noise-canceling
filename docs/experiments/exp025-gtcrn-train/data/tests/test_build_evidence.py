"""训练前清单第 3 条的证据表（build_evidence.py）：真产物逐字节复现 · 真仓里 6 个 build 全部通过 · 三条夹具。

真产物部分：`evidence-*.md` / `scenes-*.md` 按「与当前配方的关系统」两种——`builds/` 里有 build 的 `recipe_sha256`
等于当前 `recipe.json` 的 sha256 时，它的证据页与场景页都必须已在仓里、逐字节等于重算结果（不接受任何免票）；
一个这样的 build 都没有时，才轮到 `split_rules.json` 的 `未建表` 免票，且里头至少有一条是当前配方真用到的来源
（人声 · RIR · 噪声池，名单正本 `shard.default_sources()`）。上一批的页（第五批换了配方，第四批的
`evidence-bc0e9e7b566421f0.md`）冻结、只核不改（钉住 sha256 · build 目录还在 · 配方确实换了）。规则见
`tests/fixtures_pages.py`。
夹具部分：一份最小数据目录（1 张切分表 · 1 个来源 · 1 个目标场景 ·
基准 + 5 份消融配方各 1 个 build）里，把四种该红的情况各撞一次：
一个检查不过 → 「全部通过」变「否」且退 1；一份配方对上 2 个 build → 停；变体改了两个旋钮 → 那一列「否」；
build 记录说交集 ok、切分表里却有一行踩到排外表 → 「交集重算一致」变「否」、「全部通过」变「否」（结论不读记录，重算）。
另核两件事：重算按「来源列表 + 切分表哈希」缓存（6 个 build 共用一份只算一遍），且排除表一列带的是本机 exclude.json 的哈希。
配方文件一律用 `ablation_recipes` 的正本函数生成，不手写 5 份。
"""
import contextlib
import copy
import hashlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "tests"))
import ablation_recipes  # noqa: E402
import build_evidence  # noqa: E402
import fixtures_pages  # noqa: E402  冻结页 / 当前页的共用规则（第五批换配方后，旧页只核不改）
import leakcheck  # noqa: E402
import loader  # noqa: E402

EVIDENCE = HERE / "evidence-bc0e9e7b566421f0.md"  # 第四批冻结的页（第五批换配方后只核不改，sha256 钉在 fixtures_pages.PINNED）
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
        self._leak = {}  # 切分表哈希 → 它算出来的 leak_check（一张表只算一遍）
        self.table = self.write_table()
        self.write_rules()
        self.write_recipes()

    def write_rules(self, *names):
        """临时数据目录自带一份切分规则：`sources` 的键就是交集检查该查的全部来源（正本同真仓的 split_rules.json）。"""
        (self.root / "split_rules.json").write_text(
            json.dumps({"sources": {n: {"src": "夹具"} for n in (names or ("noise-a",))}},
                       ensure_ascii=False), encoding="utf-8")

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

    def leak_record(self):
        """按当前切分表的真行算一遍 `leakcheck.check`，拿它当 build 记录里该记的那份（正本同真仓）。"""
        rec = self._leak.get(self.table)
        if rec is None:
            body = (self.root / "splits" / "noise-a.jsonl").read_text(encoding="utf-8")
            rows = [json.loads(ln) for ln in body.splitlines() if ln.strip()]
            res = leakcheck.check({"noise-a": rows})
            rec = {"against": res["against"], "ok": res["ok"], "n_hits": res["n_hits"],
                   "per_key": res["per_key"]}
            self._leak[self.table] = rec
        return copy.deepcopy(rec)

    def write_build(self, recipe_bytes, leak=None, **over):
        """按一份配方字节写一条 build 记录（身份字段算 build_id），`over` 改某几个字段。

        改身份字段（`index_sha256` · `split_tables` · `shards` · `code_sha256`）会重新算 build_id——
        所以改 `index_sha256` 就能给同一份配方造出第二个 build（两边分片相同）。
        `leak` 不给就按当前切分表算一份；给一份旧记录可以造出「记录说 ok，但切分表已经命中排外表」。
        """
        parts = {"index_sha256": sha(recipe_bytes + b"idx"), "split_tables": {"noise-a": self.table},
                 "shards": {"noise-a": {"train": {"noise-a-train-00000.tar": "f" * 64}}},
                 "recipe_sha256": sha(recipe_bytes), "code_sha256": build_evidence.code_hashes()}
        for k in list(over):
            if k in loader.ID_KEYS:
                parts[k] = over.pop(k)
        rec = {"build_id": loader.build_id_of(parts), **parts,
               "leak_check": self.leak_record() if leak is None else leak,
               "leak_checked_sources": ["noise-a"], "valid": True, "missing": [],
               "noise_scenes": {"train": ["canteen"], "dev": ["canteen"]}}
        rec.update(over)
        d = self.root / "builds" / rec["build_id"]
        d.mkdir(parents=True, exist_ok=True)
        (d / "build.json").write_text(json.dumps(rec, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
        return rec

    def write_all(self, leak=None, **over):
        """按盘上的 6 份配方各写一条 build 记录（`leak` / `over` 只加在基准那一份上）。"""
        recs = {}
        for name, path in build_evidence.recipe_files(self.root):
            recs[name] = self.write_build(path.read_bytes(), leak=leak,
                                          **over if name == "recipe.json" else {})
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
        self.assertIn("交集来源齐全", text)  # 来源列表齐全这一列在表头里
        self.assertIn("基准 build", text.splitlines()[0])
        self.assertIn("不手抄", text)  # 一句话说明它是生成的，不是抄的
        # 目标场景一列注明读的是 build 记录；排外表一列带上这次重算实际用的 exclude.json 哈希
        self.assertIn("目标场景（build 记录）", text)
        self.assertIn("noise_scenes", text)
        self.assertIn(sha(leakcheck.EXCLUDE.read_bytes())[:12], text)

    def test_variant_column_names_its_knob(self):
        """每一行的「只差一个旋钮」要写出翻的是哪个旋钮、翻到哪个值（原因也是给人看的）。"""
        self.write_all()
        _text, rows = self.report()
        why = {r["recipe"]: r["checks"]["one_knob"][1] for r in rows}
        for name, knob, on in ablation_recipes.VARIANTS:
            self.assertIn(f"knobs.{knob}.on → {on}", why[f"recipes/ablation/{name}.json"])
        self.assertIn("与 recipe.json 相同", why["recipe.json"])


class FixtureStops(Fixture):
    def test_recorded_ok_but_rows_hit_exclude_is_false(self):
        """记录说 ok、per_key 全 0，但切分表里有一行踩到排外表：重算那一列「否」，「全部通过」变「否」、退 1。"""
        stale = self.leak_record()  # 干净表上算的：ok True · 命中 0
        self.assertIs(stale["ok"], True)
        self.table = self.write_table(
            rows=[{"path": "raw/esc50/x.wav", "split": "train", "unit": "u1"}])
        self.write_all(leak=stale)
        text, rows = self.report()
        self.assertEqual([r["checks"]["leak_recheck"][0] for r in rows], [False] * len(rows))
        self.assertIn("全部通过：否", text)
        self.assertIn("交集重算一致", text)
        self.assertIn("rooms_reserved", rows[0]["checks"]["leak_recheck"][1])  # 原因写清了哪个键对不上
        rc, out, _err = run_main(["--data-dir", str(self.root)])
        self.assertEqual(rc, 1)
        self.assertIn("交集重算一致", out)

    def test_recompute_uses_the_splits_on_disk_not_the_record(self):
        """记录里的 ok / per_key 都是「是」，但切分表换了（之前那一行命中）：重算仍能看出来，不被记录骗过。"""
        stale = self.leak_record()
        self.table = self.write_table(rows=[{"path": "raw/esc50/x.wav", "split": "train", "unit": "u1"}])
        rec = self.write_all(leak=stale)["recipe.json"]
        self.assertIs(rec["leak_check"]["ok"], True)
        got = build_evidence.leak_recheck(rec, self.root / "splits")
        self.assertIs(got["ok"], False)
        self.assertGreater(got["n_hits"], 0)

    def test_recompute_is_cached_per_split_tables(self):
        """6 个 build 共用一份切分表：重算只该做一遍（真仓里那 16 MB 切分表不解析六次）。"""
        self.write_all()
        calls = []
        real = build_evidence.leakcheck.check

        def spy(splits, ex=None):
            calls.append(tuple(sorted(splits)))
            return real(splits, ex)

        build_evidence.leakcheck.check = spy
        try:
            text, rows = self.report()
        finally:
            build_evidence.leakcheck.check = real
        self.assertIn("全部通过：是", text)
        self.assertEqual(len(rows), 1 + len(ablation_recipes.VARIANTS))
        self.assertEqual(len(calls), 1, calls)

    def test_record_missing_one_source_is_false(self):
        """规则里有 2 个来源、记录只列了 1 个：按记录重算的交集检查看着“一致”，另一个来源却根本没查 → 「否」。"""
        self.write_rules("noise-a", "noise-b")
        self.write_all()
        text, rows = self.report()
        self.assertEqual([r["checks"]["leak_sources"][0] for r in rows], [False] * len(rows))
        self.assertIn("noise-b", rows[0]["checks"]["leak_sources"][1])  # 原因写清了是哪个来源没查
        self.assertTrue(all(ok for k, (ok, _w) in rows[0]["checks"].items() if k != "leak_sources"))
        self.assertTrue(rows[0]["checks"]["leak_recheck"][0], "只按记录的来源重算，是看不出漏了来源的")
        self.assertIn("全部通过：否", text)
        rc, out, _err = run_main(["--data-dir", str(self.root)])
        self.assertEqual(rc, 1)
        self.assertIn("交集来源齐全", out)

    def test_empty_source_list_is_false(self):
        """记录的来源列表空着：交集检查到底查过什么无从得知 → 「否」，而且不因为空就崩掉整张表。"""
        self.write_all(leak_checked_sources=[])
        text, rows = self.report()
        self.assertFalse(rows[0]["checks"]["leak_sources"][0])
        self.assertIn("空", rows[0]["checks"]["leak_sources"][1])
        self.assertIn("全部通过：否", text)
        self.assertEqual(len(rows), 1 + len(ablation_recipes.VARIANTS))  # 其余行照旧能算出来

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
    """真仓里的两页（证据页 · 场景页）：对得上当前配方的 build 一页都不许少、且逐字节重算；
    上一批的页冻结（钉住 sha256，只核不改）；一个这样的 build 都没有时才轮到 `未建表`。

    第五批（2026-11-26）把 `recipe.json` 换到 `but-reverbdb-silence-b5`：`evidence-bc0e9e7b566421f0.md` 与
    `scenes-bc0e9e7b566421f0.md` 是第四批的产物，按新配方算不出来，也不能改（CLAUDE.md 第五节第 1 条）。
    规则见 `tests/fixtures_pages.py`。
    """

    def test_pages_are_current_or_frozen(self):
        """每一页：对得上当前配方就交给下面那条重算；对不上就必须是冻结的（钉住 sha256，只核不改）。"""
        current = fixtures_pages.recipe_sha()
        pages = fixtures_pages.pages("evidence")
        self.assertTrue(pages, "仓里一个证据页也没有：先跑 build_evidence.py 生成")
        for page in pages:
            build = fixtures_pages.build_of(page)
            if build.get("recipe_sha256") == current:
                continue  # 本批的页与场景页一起，由下面那条逐字节重算
            self.assertNotEqual(build.get("recipe_sha256"), current)
            self.assertIn(page.name, fixtures_pages.PINNED,
                          f"{page.name} 是上一批的页（build 的配方哈希 ≠ 当前 recipe.json），冻结页要钉住 sha256")
            self.assertEqual(fixtures_pages.sha256_of(page), fixtures_pages.PINNED[page.name],
                             f"{page.name} 被改过：上一批的页冻结、只核不改")

    def test_current_builds_have_both_pages_byte_exact(self):
        """当前配方的 build：`evidence-<id>.md` 与 `scenes-<id>.md` 都要在、且逐字节等于重算结果。

        这种 build 存在时 `未建表` 免不了票；一个也没有时才轮到 `未建表`（且里头要有配方真用到的来源）。"""
        builds = fixtures_pages.current_builds()
        if not builds:
            ok, why = fixtures_pages.pending_exemption()
            self.assertTrue(ok, why)
            return
        for prefix in ("evidence", "scenes"):
            missing = fixtures_pages.missing_pages(prefix)
            self.assertEqual(missing, [],
                             f"build {missing} 对得上当前配方，却没有 {prefix}-<id>.md：先跑生成器；"
                             "`未建表` 免不了这张票")
        for bid in sorted(builds):
            for prefix in ("evidence", "scenes"):
                page = HERE / f"{prefix}-{bid}.md"
                self.assertEqual(fixtures_pages.regen(prefix, bid), page.read_bytes(),
                                 f"{page.name} 与重算结果不同：重跑生成器、把输出写到同一个文件再提交")

    def test_current_batch_reports_its_six_rows(self):
        """当前配方对上了 build 时（云上第五批登记之后），证据表就是「基准 + 5 份消融」那 6 行且全过。"""
        if not any(fixtures_pages.build_of(p).get("recipe_sha256") == fixtures_pages.recipe_sha()
                   for p in fixtures_pages.pages("evidence")):
            self.skipTest("当前配方还没有 build（第五批在云上跑之前）：由上面那条测试的 `未建表` 盯着")
        text, rows = build_evidence.report()
        self.assertEqual([r["recipe"] for r in rows], [n for n, _p in build_evidence.recipe_files(HERE)])
        for r in rows:
            bad = {k: why for k, (ok, why) in r["checks"].items() if not ok}
            self.assertEqual(bad, {}, f"{r['recipe']}（build {r['build'].get('build_id')}）：{bad}")
        self.assertIn("全部通过：是", text)


class PageRule(unittest.TestCase):
    """`未建表` 免票的规则（临时夹具）：挂一个与这批无关的来源不免票；当前配方的 build 缺页要报出来。

    第五批 codex 第 1 轮 #2：以前只看 `未建表` 非不非空，`未建表` 里只要还有别科的来源，这一批的页就没人催。
    """

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        recipe = {"knobs": {}, "speech": {"train": ["speech-a"]}, "rir": ["rir-a"],
                  "noise_pools": {"canteen": [{"source": "noise-a", "regex": "/a/"}]}}
        (self.root / "recipe.json").write_text(json.dumps(recipe, ensure_ascii=False), encoding="utf-8")
        self.write_rules({})

    def tearDown(self):
        self._tmp.cleanup()

    def write_rules(self, pending):
        (self.root / "split_rules.json").write_text(
            json.dumps({"sources": {"noise-a": {"src": "夹具"}}, "未建表": pending}, ensure_ascii=False),
            encoding="utf-8")

    def write_build(self, recipe_sha256=None):
        bid = "abcdef0123456789"
        rec = {"build_id": bid, "recipe_sha256": recipe_sha256 or fixtures_pages.recipe_sha(self.root)}
        d = self.root / "builds" / bid
        d.mkdir(parents=True)
        (d / "build.json").write_text(json.dumps(rec, ensure_ascii=False), encoding="utf-8")
        return bid

    def test_sources_come_from_the_recipe(self):
        """配方真用到的来源按 `shard.default_sources()` 走（不手抄）：这一条钉住那条正本。"""
        self.assertEqual(fixtures_pages.recipe_sources(self.root), ["speech-a", "rir-a", "noise-a"])

    def test_unrelated_pending_does_not_exempt(self):
        """`未建表` 里只有别科的来源：不免票（以前非空就免票，这一批的页就没人催了）。"""
        self.write_rules({"cochlscene": "别科的音频还在云上"})
        ok, why = fixtures_pages.pending_exemption(self.root)
        self.assertFalse(ok, why)
        self.assertIn("noise-a", why)  # 原因写清了配方用到哪些来源

    def test_relevant_pending_exempts_before_the_build_is_registered(self):
        """合格的免票：还没有当前配方的 build，`未建表` 里有它用到的来源。"""
        self.write_rules({"noise-a": "音频还在云上，本机还没切"})
        ok, why = fixtures_pages.pending_exemption(self.root)
        self.assertTrue(ok, why)

    def test_current_build_without_pages_is_reported(self):
        """当前配方的 build 缺页：证据页、场景页各报一次，而且 `未建表` 免不了票。"""
        bid = self.write_build()
        self.assertEqual(sorted(fixtures_pages.current_builds(self.root)), [bid])
        self.assertEqual(fixtures_pages.missing_pages("evidence", self.root), [bid])
        self.assertEqual(fixtures_pages.missing_pages("scenes", self.root), [bid])
        (self.root / f"evidence-{bid}.md").write_text("x", encoding="utf-8")
        self.assertEqual(fixtures_pages.missing_pages("evidence", self.root), [])
        self.assertEqual(fixtures_pages.missing_pages("scenes", self.root), [bid])
        self.write_rules({"noise-a": "音频还在云上"})
        ok, why = fixtures_pages.pending_exemption(self.root)
        self.assertFalse(ok, why)

    def test_other_recipe_build_does_not_count(self):
        """上一批配方的 build（哈希对不上）：不算当前批，也不该把 `未建表` 的免票堵掉。"""
        self.write_build(recipe_sha256="0" * 64)
        self.assertEqual(fixtures_pages.current_builds(self.root), {})
        self.write_rules({"noise-a": "音频还在云上"})
        self.assertTrue(fixtures_pages.pending_exemption(self.root)[0])


class Size(unittest.TestCase):
    def test_new_py_under_600(self):
        for name in ("build_evidence.py", "tests/test_build_evidence.py", "tests/fixtures_pages.py"):
            n = len((HERE / name).read_text(encoding="utf-8").splitlines())
            self.assertLessEqual(n, 600, f"{name} 有 {n} 行")


if __name__ == "__main__":
    unittest.main()
