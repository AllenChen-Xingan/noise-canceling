"""数据 build 的登记核对（T/buildpin.py）与 formal / ablation 的接线。

分片与配方都对还不够：loader 只认**登记过的** build。它启动时要 data/builds/<id>/build.json 存在、交集
检查通过，而且配方与数据代码的 sha256 和登记时一模一样（正本 data/loader.py）。本文件用 tmp 里的假登记
记录把这四类拦路问题各测一遍，再用 tmp 里的假配置测 formal / ablation 的接线；最后两条跑仓里真正在用的
那两份配置——新基线 build 的登记记录在这条分支上可能还没到（traindata 的分支还没合到 main），那种情况
要求 check 报的正是「这条记录 / 这份配方」的问题，不许静默通过。
"""
import json
from hashlib import sha256
from pathlib import Path

import pytest
import yaml

import ablation
import buildpin
import config as cfgmod
import formal

HERE = Path(__file__).resolve().parents[1]  # trainer/
CONFIGS = HERE / "configs"
EXP = HERE.parent  # exp025-gtcrn-train/
D = EXP / "data"  # 数据线目录（只读）
BUILD = "bc0e9e7b566421f0"  # 新基线 build（traindata 登记）
RECORD = D / "builds" / BUILD / "build.json"


# ---------- 假的登记记录（布局与 register_build 写出来的一样）----------

def _loader():
    from data import import_loader
    return import_loader()


def current_code() -> dict:
    """当前数据代码的 sha256。这里按 loader 的公式自己算一遍——`buildpin.code_sha256` 要拿它当交叉核对，
    不能自己证自己。"""
    m = _loader()
    return {**{f: sha256((m.HERE / f).read_bytes()).hexdigest() for f in m.CODE},
            **{f"exp013/{f}": sha256((m.EXP013 / f).read_bytes()).hexdigest() for f in m.EXP013_CODE}}


def make_recipe(path: Path, text: str = '{"knobs": {}}') -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    return path


def make_build(builds_dir: Path, recipe: Path, *, index: str = "a" * 64, code: dict | None = None,
               valid: bool = True, stamp: bool = True) -> str:
    """在 builds_dir 下摆一份登记记录，返回它的 build_id。

    id 按 loader.build_id_of 从记录内容算（与 register_build 同一公式）。code 给别的表 = 登记时代码不同；
    stamp = False 让记录里的 build_id 与目录名对不上（测「记录自己对不上」）。
    """
    parts = {"index_sha256": index, "split_tables": {}, "shards": {},
             "recipe_sha256": sha256(recipe.read_bytes()).hexdigest(),
             "code_sha256": dict(current_code() if code is None else code)}
    bid = _loader().build_id_of(parts)
    rec = {"build_id": bid if stamp else "0" * 16, **parts, "valid": valid, "counts": {}}
    out = builds_dir / bid
    out.mkdir(parents=True, exist_ok=True)
    (out / "build.json").write_text(json.dumps(rec, ensure_ascii=False))
    return bid


def _tamper(builds_dir: Path, bid: str, **changes) -> None:
    """改登记记录里的字段（模拟记录被手改过）。"""
    p = builds_dir / bid / "build.json"
    d = json.loads(p.read_text())
    d.update(changes)
    p.write_text(json.dumps(d, ensure_ascii=False))


# ---------- buildpin：数据代码的公式从 loader 取 ----------

def test_code_sha256_matches_loader_formula():
    m = _loader()
    assert buildpin.code_sha256() == current_code()
    assert set(buildpin.code_sha256()) == set(m.CODE) | {f"exp013/{f}" for f in m.EXP013_CODE}


def test_default_builds_dir_is_loader_builds():
    assert buildpin.default_builds_dir() == Path(_loader().BUILDS)


@pytest.mark.parametrize("bad", [None, "", "   ", "bc0e9e7b", "BC0E9E7B566421F0", "bc0e9e7b566421f0a",
                                 566421, 0.5])
def test_bad_data_build_reported(bad):
    s = buildpin.data_build_problem(bad)
    assert s is not None and "data_build" in s


def test_good_data_build_passes():
    assert buildpin.data_build_problem(BUILD) is None


# ---------- buildpin：四类拦路问题 ----------

def test_registered_build_passes(tmp_path):
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "data" / "recipe.json")
    bid = make_build(bdir, recipe)
    assert buildpin.build_problems(recipe, bid, bdir) == []


def test_missing_record_reported(tmp_path):
    bdir = tmp_path / "builds"
    bdir.mkdir()
    recipe = make_recipe(tmp_path / "recipe.json")
    ps = buildpin.build_problems(recipe, BUILD, bdir)
    assert len(ps) == 1, ps
    assert BUILD in ps[0] and "build.json" in ps[0] and "没登记" in ps[0]


def test_unreadable_record_reported(tmp_path):
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    bid = make_build(bdir, recipe)
    (bdir / bid / "build.json").write_text("{ 不是 JSON")
    ps = buildpin.build_problems(recipe, bid, bdir)
    assert len(ps) == 1, ps
    assert bid in ps[0] and "读不了" in ps[0]


def test_invalid_build_reported(tmp_path):
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    bid = make_build(bdir, recipe, valid=False)
    ps = buildpin.build_problems(recipe, bid, bdir)
    assert len(ps) == 1, ps
    assert bid in ps[0] and "valid" in ps[0] and "交集检查" in ps[0]


def test_record_build_id_field_mismatch_reported(tmp_path):
    """目录名对、记录里的 build_id 字段对不上（记录被手改过）。"""
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    bid = make_build(bdir, recipe, stamp=False)
    ps = buildpin.build_problems(recipe, bid, bdir)
    assert len(ps) == 1, ps
    assert "build_id" in ps[0] and "自己对不上" in ps[0]


def test_record_content_changed_after_stamping_reported(tmp_path):
    """目录名和记录里的 build_id 都光鲜，但记录内容已经算不出那个 id。"""
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    bid = make_build(bdir, recipe)
    _tamper(bdir, bid, recipe_sha256="f" * 64)
    ps = buildpin.build_problems(recipe, bid, bdir)
    assert len(ps) == 1, ps
    assert "自己对不上" in ps[0] and "build_id_of" not in ps[0]


def test_other_recipe_reported(tmp_path):
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    bid = make_build(bdir, recipe)
    other = make_recipe(tmp_path / "other.json", '{"knobs": {"eq": {"on": true}}}')
    ps = buildpin.build_problems(other, bid, bdir)
    assert len(ps) == 1, ps
    assert bid in ps[0] and "配方对不上" in ps[0] and "other.json" in ps[0]


def test_changed_data_code_reported(tmp_path):
    """登记时代码的 sha256 与现在不同：记录自己一致，但 loader 会拒绝这个 build。"""
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    old = dict(current_code())
    old["loader.py"] = "0" * 64
    bid = make_build(bdir, recipe, code=old)
    ps = buildpin.build_problems(recipe, bid, bdir)
    assert len(ps) == 1, ps
    assert "loader.py" in ps[0] and "loader 启动时会拒绝" in ps[0]


def test_missing_recipe_reported(tmp_path):
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    bid = make_build(bdir, recipe)
    ps = buildpin.build_problems(tmp_path / "nope.json", bid, bdir)
    assert len(ps) == 1, ps
    assert "nope.json" in ps[0] and "读不了配方" in ps[0]


def test_bad_build_id_reported_by_build_problems(tmp_path):
    recipe = make_recipe(tmp_path / "recipe.json")
    ps = buildpin.build_problems(recipe, None, tmp_path / "builds")
    assert len(ps) == 1 and "data_build" in ps[0]


# ---------- buildpin：变体配方自己的登记记录 ----------

def test_registered_variant_passes(tmp_path):
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    base = make_build(bdir, recipe)
    var = make_recipe(tmp_path / "no-eq.json", '{"knobs": {"eq": {"on": false}}}')
    make_build(bdir, var)  # 同一份假分片：index 默认值相同
    assert buildpin.variant_build_problems(var, base, bdir) == []


def test_unregistered_variant_reported(tmp_path):
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    base = make_build(bdir, recipe)
    var = make_recipe(tmp_path / "no-eq.json", '{"knobs": {"eq": {"on": false}}}')
    ps = buildpin.variant_build_problems(var, base, bdir)
    assert len(ps) == 1, ps
    assert "no-eq.json" in ps[0] and base in ps[0] and "没有合格的登记记录" in ps[0]


def test_variant_registered_on_other_shards_reported(tmp_path):
    """配方登记了，但用的不是同一份分片（index_sha256 不同）：读数没法和基线比。"""
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    base = make_build(bdir, recipe)
    var = make_recipe(tmp_path / "no-eq.json", '{"knobs": {"eq": {"on": false}}}')
    make_build(bdir, var, index="b" * 64)
    ps = buildpin.variant_build_problems(var, base, bdir)
    assert len(ps) == 1, ps
    assert "no-eq.json" in ps[0]


def test_variant_registered_but_invalid_reported(tmp_path):
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    base = make_build(bdir, recipe)
    var = make_recipe(tmp_path / "no-eq.json", '{"knobs": {"eq": {"on": false}}}')
    make_build(bdir, var, valid=False)
    assert len(buildpin.variant_build_problems(var, base, bdir)) == 1


def test_variant_without_base_record_silent(tmp_path):
    """基线记录读不出来时不重复报：那一条已经由 build_problems 报过，这里没有 index 可比。"""
    bdir = tmp_path / "builds"
    bdir.mkdir()
    var = make_recipe(tmp_path / "no-eq.json", '{"knobs": {"eq": {"on": false}}}')
    assert buildpin.variant_build_problems(var, BUILD, bdir) == []


def test_variant_missing_file_reported(tmp_path):
    bdir = tmp_path / "builds"
    recipe = make_recipe(tmp_path / "recipe.json")
    base = make_build(bdir, recipe)
    ps = buildpin.variant_build_problems(tmp_path / "gone.json", base, bdir)
    assert len(ps) == 1 and "gone.json" in ps[0]


# ---------- formal 的接线 ----------

def _formal_case(tmp: Path) -> tuple[Path, Path]:
    """tmp 里摆一份填好的正式配置与假登记记录；布局同仓里（trainer/configs 与 data 平级）。"""
    T, configs = tmp / "trainer", tmp / "trainer" / "configs"
    configs.mkdir(parents=True)
    recipe = make_recipe(tmp / "data" / "recipe.json")
    bdir = tmp / "builds"
    bid = make_build(bdir, recipe)
    (configs / "formal.yaml").write_text((CONFIGS / "formal.yaml").read_text())
    runs = yaml.safe_load((CONFIGS / "formal-runs.yaml").read_text())
    runs["data_build"] = bid
    p = configs / "formal-runs.yaml"
    p.write_text(yaml.safe_dump(runs, allow_unicode=True))
    return p, bdir


def _set_data_build(p: Path, value) -> None:
    runs = yaml.safe_load(p.read_text())
    if value is None:
        runs.pop("data_build", None)
    else:
        runs["data_build"] = value
    p.write_text(yaml.safe_dump(runs, allow_unicode=True))


def test_formal_filled_copy_passes(tmp_path, capsys):
    p, bd = _formal_case(tmp_path)
    assert formal.problems(p, builds_dir=bd) == []
    assert formal.main(["check", str(p), "--builds-dir", str(bd)]) == 0
    assert "可以开训" in capsys.readouterr().out


def test_formal_missing_data_build_reported(tmp_path):
    p, bd = _formal_case(tmp_path)
    _set_data_build(p, None)
    ps = formal.problems(p, builds_dir=bd)
    assert len(ps) == 1, ps
    assert "data_build" in ps[0]


def test_formal_malformed_data_build_reported(tmp_path):
    p, bd = _formal_case(tmp_path)
    _set_data_build(p, "bc0e9e7b")
    ps = formal.problems(p, builds_dir=bd)
    assert len(ps) == 1, ps
    assert "data_build" in ps[0] and "16 位" in ps[0]


def test_formal_unregistered_build_reported(tmp_path):
    p, bd = _formal_case(tmp_path)
    _set_data_build(p, "0123456789abcdef")
    ps = formal.problems(p, builds_dir=bd)
    assert len(ps) == 1, ps
    assert "0123456789abcdef" in ps[0] and "没登记" in ps[0]


# ---------- ablation 的接线 ----------

def _ablation_case(tmp: Path) -> tuple[Path, Path]:
    """tmp 里摆一份填好的消融配置、5 份变体配方与它们的假登记记录（复用 test_ablation 的 `_Case`：
    两处都摆一份就会变成两个真相源）。"""
    from test_ablation import _Case
    case = _Case(tmp)
    return case.runs_path, case.builds


def test_ablation_filled_copy_passes(tmp_path, capsys):
    p, bd = _ablation_case(tmp_path)
    assert ablation.problems(p, builds_dir=bd) == []
    assert ablation.main(["check", str(p), "--builds-dir", str(bd)]) == 0
    assert "可以开训" in capsys.readouterr().out
    assert len(ablation.commands(p, builds_dir=bd)) == 3  # 3 + 3 + 1


def test_ablation_variant_registered_on_old_bytes_reported(tmp_path):
    """变体配方改过字节、但还没重新登记：loader 算出来的是另一个（没登记的）build。"""
    p, bd = _ablation_case(tmp_path)
    f = tmp_path / "data" / "recipes" / "ablation" / "no-eq.json"
    f.write_text(f.read_text() + "\n")  # 解析出来一样、字节变了：登记的是旧字节
    ps = ablation.problems(p, builds_dir=bd)
    assert len(ps) == 1, ps
    assert "no-eq.json" in ps[0] and "登记" in ps[0]


def test_ablation_missing_data_build_reported(tmp_path):
    p, bd = _ablation_case(tmp_path)
    runs = yaml.safe_load(p.read_text())
    runs.pop("data_build")
    p.write_text(yaml.safe_dump(runs, allow_unicode=True))
    ps = ablation.problems(p, builds_dir=bd)
    assert len(ps) == 1, ps
    assert "data_build" in ps[0]


# ---------- 仓里真正在用的那两份配置 ----------

TODO = "TODO"
CONFIG_FILES = ("formal.yaml", "ablation.yaml")
RUNS_FILES = ("formal-runs.yaml", "ablation-runs.yaml")


def _flat_values(d, p="") -> dict:
    out = {}
    for k, v in d.items():
        if isinstance(v, dict):
            out.update(_flat_values(v, p + k + "."))
        else:
            out[p + k] = v
    return out


def test_shipped_configs_have_no_todo_placeholders():
    """生效的配置里不许再有 TODO 占位（注释里提一句 TODO 是说明，不算）。"""
    for name in CONFIG_FILES:
        cfg = cfgmod.load(CONFIGS / name)
        assert formal._todo(cfg) == [], f"{name} 里还剩占位"
    for name in RUNS_FILES:
        runs = yaml.safe_load((CONFIGS / name).read_text())
        left = [(k, v) for k, v in _flat_values(runs).items()
                if isinstance(v, str) and v.strip().startswith(TODO)]
        assert left == [], f"{name} 里还剩占位 {left}"


def test_shipped_data_build_is_pinned():
    for name in RUNS_FILES:
        runs = yaml.safe_load((CONFIGS / name).read_text())
        assert runs["data_build"] == BUILD, f"{name} 的 data_build 要钉在新基线 build {BUILD}"
        assert buildpin.data_build_problem(runs["data_build"]) is None


def test_shipped_shard_root():
    for name in CONFIG_FILES:
        cfg = yaml.safe_load((CONFIGS / name).read_text())
        assert cfg["data"]["shard_root"] == "/workspace/shards/batch4", f"{name} 的分片目录"


def test_shipped_formal_check():
    """真的跑一遍仓里的配置：登记记录在就必须完全通过；不在这条分支上时只许报「这条记录没登记」。"""
    ps = formal.problems(formal.DEFAULT_RUNS)
    if RECORD.exists():
        assert ps == [], ps
        assert formal.main(["check"]) == 0
    else:
        assert len(ps) == 1, ps
        assert BUILD in ps[0] and "build.json" in ps[0] and "没登记" in ps[0]
        assert formal.main(["check"]) == 1


def test_shipped_ablation_check():
    ps = ablation.problems(ablation.DEFAULT_RUNS)
    if RECORD.exists():
        assert ablation.main(["check"]) == 0, ps
    else:
        assert any(BUILD in s and "没登记" in s for s in ps), ps
        # 这条分支上还缺 traindata 的变体配方：除了「缺文件」，不许有别的问题
        others = [s for s in ps if BUILD not in s]
        assert all("不存在" in s for s in others), others
        assert ablation.main(["check"]) == 1
