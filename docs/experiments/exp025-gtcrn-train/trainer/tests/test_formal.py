"""正式训练开训前的检查（T/formal.py）：配置里还有占位 · 场景名不对 · 运行矩阵不是 6 个 · 数据 build
的登记记录对不上，就不许开训。

正式训练 6 个运行 = 模型 A、B × 种子 0、1、2，每批 3 个同时跑一张卡。加训场景人已定（D113），名字按
地点键表查（specs/schema/eval-suite.schema.json）。仓里的分片与配方已填好（build bc0e9e7b566421f0，
正本 buildpin.py），所以 tmp 里每一项用例都自带一份假登记记录（`builds_dir` 参数指过去）。
"""
from pathlib import Path

import pytest
import yaml

import config as cfgmod
import formal
from test_buildpin import make_build, make_recipe, recipe_text

HERE = Path(__file__).resolve().parents[1]
CONFIGS = HERE / "configs"
RUNS = CONFIGS / "formal-runs.yaml"


def _flat(d, p=""):
    out = {}
    for k, v in d.items():
        if isinstance(v, dict):
            out.update(_flat(v, p + k + "."))
        else:
            out[p + k] = v
    return out


def _diff(a, b):
    """两份扁平配置不同的键（键集合必须相同）。"""
    fa, fb = _flat(a), _flat(b)
    assert set(fa) == set(fb)
    return {k for k in fa if fa[k] != fb[k]}


# ---------- formal.yaml：只覆盖与默认值不同的字段 ----------

def test_formal_yaml_overrides_only_intended_fields():
    cfg = cfgmod.load(CONFIGS / "formal.yaml")
    assert _diff(cfgmod.to_dict(cfg), cfgmod.to_dict(cfgmod.Config())) == {
        "run.out_dir", "data.shard_root", "data.recipe", "data.num_workers",
        "limits.expect_s_step", "limits.max_hours"}


def test_formal_yaml_values():
    cfg = cfgmod.load(CONFIGS / "formal.yaml")
    assert cfg.run.out_dir == "/workspace/runs/exp025-formal"
    assert cfg.data.num_workers == 6  # 3 个同时跑 × 6 = 18，容器约 22 核
    assert cfg.limits.expect_s_step == pytest.approx(0.39)  # 3 个同时跑的实测（plan-h100-smoke.md 第五节）
    assert cfg.limits.max_hours == pytest.approx(1.5)
    # 其余全用默认：5000 步 · 每 500 步评估与存盘 · 4.39 美元每小时
    assert cfg.schedule.max_steps == 5000 and cfg.schedule.eval_every == 500
    assert cfg.schedule.ckpt_every == 500 and cfg.limits.usd_per_hour == pytest.approx(4.39)
    # 分片与配方已填好（build bc0e9e7b566421f0 的 baseline 分片与基线配方）
    assert cfg.data.shard_root == "/workspace/shards/batch4"
    assert cfg.data.recipe == "../data/recipe.json"


# ---------- 运行矩阵与命令 ----------

def _runs() -> dict:
    return yaml.safe_load(RUNS.read_text())


def _filled(tmp_path, base_over=None, runs_over=None, mutate=None, recipe_src=None):
    """在 tmp_path/configs/ 下摆一份「分片已登记、配方已定」的用例（与仓里布局相同），
    并给它造一份假登记记录。返回（运行矩阵路径, 假登记记录目录）。

    mutate：改运行矩阵的 batches（测矩阵本身写错）。
    recipe_src：配方文本（默认 `knobs.target_scene.scene` = 默认的 target_scenes：配方是加训场景
    的正本，两处必须一致才通过，#213）。
    """
    d = yaml.safe_load((CONFIGS / "formal.yaml").read_text())
    d["data"]["shard_root"] = "/data/shards/batch2"
    d["data"]["recipe"] = "recipes/formal-batch2.json"
    for dotted, v in (base_over or {}).items():
        head, last = dotted.split(".")
        d[head][last] = v
    recipe = make_recipe(tmp_path / "recipes" / "formal-batch2.json",
                         recipe_text(["cafe", "canteen"]) if recipe_src is None else recipe_src)
    builds = tmp_path / "builds"
    runs = _runs()
    runs["data_build"] = make_build(builds, recipe)
    runs["target_scenes"] = ["cafe", "canteen"]
    if mutate is not None:
        mutate(runs["batches"])
    for dotted, v in (runs_over or {}).items():
        if "." in dotted:
            head, last = dotted.split(".")
            runs[head][last] = v
        else:
            runs[dotted] = v
    out = tmp_path / "configs"
    out.mkdir(parents=True, exist_ok=True)
    (out / "formal.yaml").write_text(yaml.safe_dump(d, allow_unicode=True))
    p = out / "formal-runs.yaml"
    p.write_text(yaml.safe_dump(runs, allow_unicode=True))
    return p, builds


def _cmd(name, variant, seed):
    """expected 的命令正本在这里；formal.commands 必须逐字相同。"""
    return (f"cd $T && VDN_GTCRN_DIR=$G $PY train.py --config configs/formal.yaml "
            f"--set run.name={name} --set model.variant={variant} --set run.seed={seed}")


def test_load_runs_reads_matrix():
    runs = formal.load_runs(RUNS)
    assert [len(b) for b in runs["batches"]] == [3, 3]
    assert [(r["variant"], r["seed"]) for b in runs["batches"] for r in b] == [
        ("A", 0), ("A", 1), ("A", 2), ("B", 0), ("B", 1), ("B", 2)]


def test_shipped_configs_have_no_other_problems():
    """仓里的配置填好了：除了「登记记录还没到这条分支」（traindata 的分支未合）那一类，不许再报别的。
    占位 / 分片 / 配方 / data_build 的 shipped 断言在 test_buildpin.py。"""
    ps = formal.problems(RUNS)
    leftovers = [s for s in ps if "bc0e9e7b566421f0" not in s]
    assert leftovers == [], leftovers


def test_commands_refuse_with_todo_recipe(tmp_path):
    p, bd = _filled(tmp_path, base_over={"data.recipe": "TODO(待定)"})
    with pytest.raises(ValueError) as e:
        formal.commands(p, builds_dir=bd)
    msg = str(e.value)
    assert "data.recipe" in msg
    assert "target_scenes" not in msg


def test_filled_copy_can_start(tmp_path, capsys):
    p, bd = _filled(tmp_path)
    assert formal.problems(p, builds_dir=bd) == []
    assert formal.main(["check", str(p), "--builds-dir", str(bd)]) == 0
    assert "可以开训" in capsys.readouterr().out
    cmds = formal.commands(p, builds_dir=bd)
    assert cmds == [
        [_cmd("exp025-formal-A-s0", "A", 0), _cmd("exp025-formal-A-s1", "A", 1),
         _cmd("exp025-formal-A-s2", "A", 2)],
        [_cmd("exp025-formal-B-s0", "B", 0), _cmd("exp025-formal-B-s1", "B", 1),
         _cmd("exp025-formal-B-s2", "B", 2)]]


def test_commands_cli_prints_two_blocks(tmp_path, capsys):
    p, bd = _filled(tmp_path)
    assert formal.main(["commands", str(p), "--builds-dir", str(bd)]) == 0
    out = capsys.readouterr().out
    assert out.count("train.py --config configs/formal.yaml") == 6
    assert "--set run.name=exp025-formal-B-s2 --set model.variant=B --set run.seed=2" in out


def test_commands_use_given_placeholders(tmp_path):
    p, bd = _filled(tmp_path)
    c = formal.commands(p, T_dir="/srv/trainer", python="python3.13", gtcrn="/srv/gtcrn",
                        builds_dir=bd)[0][0]
    assert c.startswith("cd /srv/trainer && VDN_GTCRN_DIR=/srv/gtcrn python3.13 train.py ")


@pytest.mark.parametrize("over,needle", [
    ({"limits.expect_s_step": None}, "limits.expect_s_step"),
])
def test_filled_but_expect_s_step_unset_reported(tmp_path, over, needle):
    p, bd = _filled(tmp_path, base_over=over)
    ps = formal.problems(p, builds_dir=bd)
    assert any(needle in s for s in ps), ps


@pytest.mark.parametrize("base_over", [{"data.recipe": ""}, {"data.shard_root": "TODO(待填)"}])
def test_filled_but_todo_left_reported(tmp_path, base_over):
    p, bd = _filled(tmp_path, base_over=base_over)
    ps = formal.problems(p, builds_dir=bd)
    assert any("data.recipe" in s or "data.shard_root" in s for s in ps), ps


def test_empty_target_scenes_reported(tmp_path):
    p, bd = _filled(tmp_path, runs_over={"target_scenes": []})
    assert any("target_scenes" in s for s in formal.problems(p, builds_dir=bd))


def _drop_seed(batches):
    del batches[0][0]["seed"]


def _dup_name(batches):
    batches[1][0]["name"] = batches[0][0]["name"]


def _short_batch(batches):
    batches[1] = batches[1][:2]


def _wrong_pair(batches):
    batches[1][2]["seed"] = 3


def _one_batch(batches):
    del batches[1]


@pytest.mark.parametrize("mutate,needle", [
    (_drop_seed, "seed"),
    (_dup_name, "重复"),
    (_short_batch, "第 2 批"),
    (_wrong_pair, "矩阵"),
    (_one_batch, "矩阵"),
], ids=[f.__name__ for f in (_drop_seed, _dup_name, _short_batch, _wrong_pair, _one_batch)])
def test_broken_matrix_reported(tmp_path, mutate, needle):
    p, bd = _filled(tmp_path, mutate=mutate)
    ps = formal.problems(p, builds_dir=bd)
    assert any(needle in s for s in ps), ps


# ---------- 加训场景按地点键表查（#209）----------

D113_SCENES = ["rail-air-cabin", "canteen", "meeting-room", "transit-hub"]
SCHEMA = "specs/schema/eval-suite.schema.json"


def _schema_place_keys() -> list:
    """正本：eval-suite.schema.json 里 MeetingPlace 的 enum（含兜底键 other）。"""
    import json

    for root in (HERE, *HERE.parents):
        f = root / SCHEMA
        if f.exists():
            return list(json.loads(f.read_text())["$defs"]["MeetingPlace"]["enum"])
    raise AssertionError(f"找不到 {SCHEMA}")


def _scene_problems(tmp_path, scenes) -> list:
    """改了 target_scenes 的那份运行矩阵报出来的、说 target_scenes 名字本身的问题。

    不含「和配方不一致」那一条（#213）：那一条另有一组用例，这里测的是名字本身写错没用。
    """
    p, bd = _filled(tmp_path, runs_over={"target_scenes": scenes})
    return [s for s in formal.problems(p, builds_dir=bd) if "target_scenes" in s and "配方" not in s]


def test_unknown_scene_reported(tmp_path):
    ps = _scene_problems(tmp_path, ["cafe", "car"])
    assert len(ps) == 1, ps
    assert "car" in ps[0]  # 要指名是哪个不在表里
    assert SCHEMA in ps[0]  # 并指回地点键表的正本


def test_fallback_other_reported(tmp_path):
    ps = _scene_problems(tmp_path, ["other"])
    assert len(ps) == 1, ps
    assert "other" in ps[0]


def test_duplicate_scene_reported(tmp_path):
    ps = _scene_problems(tmp_path, ["cafe", "cafe"])
    assert len(ps) == 1, ps
    assert "cafe" in ps[0] and "重复" in ps[0]


def test_non_string_scene_reported(tmp_path):
    ps = _scene_problems(tmp_path, [3])
    assert len(ps) == 1, ps
    assert "3" in ps[0]


def test_non_list_target_scenes_reported(tmp_path):
    ps = _scene_problems(tmp_path, "cafe")  # 写成单个字符串，不是清单
    assert len(ps) == 1, ps
    assert "target_scenes" in ps[0]


def test_d113_scenes_pass_scene_check(tmp_path):
    assert _scene_problems(tmp_path, D113_SCENES) == []


def test_place_keys_match_schema_minus_other():
    keys = formal.place_keys()
    assert isinstance(keys, frozenset)
    assert keys == frozenset(_schema_place_keys()) - {"other"}
    assert set(D113_SCENES) <= keys


def test_shipped_runs_target_scenes_are_d113():
    assert _runs()["target_scenes"] == D113_SCENES


def test_missing_base_reported(tmp_path):
    p = tmp_path / "configs"
    p.mkdir()
    runs = _runs()
    (p / "formal-runs.yaml").write_text(yaml.safe_dump(runs, allow_unicode=True))
    assert any("base" in s for s in formal.problems(p / "formal-runs.yaml"))


# ---------- 配方场景与 target_scenes 比对（#213）----------

def _src_problems(tmp_path, recipe_scenes=None, target=None, *, on=True, recipe_src=None) -> list:
    """改了配方的那份用例报出来的「和配方比对」问题（按「配方」认这几条）。

    recipe_scenes：配方 knobs.target_scene.scene；target：运行矩阵的 target_scenes（默认 = 配方的）。
    """
    target = ["cafe", "canteen"] if target is None else target
    src = recipe_src if recipe_src is not None else recipe_text(recipe_scenes, on=on)
    p, bd = _filled(tmp_path, runs_over={"target_scenes": target}, recipe_src=src)
    return [s for s in formal.problems(p, builds_dir=bd) if "配方" in s]


def test_recipe_scenes_same_set_any_order_passes(tmp_path):
    assert _src_problems(tmp_path, ["canteen", "cafe"]) == []


def test_recipe_extra_scene_reported(tmp_path):
    ps = _src_problems(tmp_path, ["cafe", "canteen", "cowork"])
    assert len(ps) == 1, ps
    assert "cowork" in ps[0]  # 要指名是哪个场景
    assert "target_scenes" in ps[0] and "必须一致" in ps[0]


def test_recipe_missing_scene_reported(tmp_path):
    ps = _src_problems(tmp_path, ["cafe"])
    assert len(ps) == 1, ps
    assert "canteen" in ps[0]
    assert "target_scenes" in ps[0] and "必须一致" in ps[0]


def test_recipe_scene_null_reported(tmp_path):
    ps = _src_problems(tmp_path, None)
    assert len(ps) == 1, ps
    assert "target_scene.scene" in ps[0]


def test_recipe_target_scene_off_reported(tmp_path):
    ps = _src_problems(tmp_path, ["cafe", "canteen"], on=False)
    assert len(ps) == 1, ps
    assert "on" in ps[0]


def test_recipe_single_scene_name_passes(tmp_path):
    """配方写成单个名字（不是清单）时，和只有一项的 target_scenes 等价。"""
    assert _src_problems(tmp_path, "cafe", ["cafe"]) == []


def test_recipe_not_json_one_problem(tmp_path):
    """配方读不成 JSON：一条问题，不崩（配方字节没变，登记记录还对得上）。"""
    ps = _src_problems(tmp_path, None, recipe_src="{ 不是 JSON")
    assert len(ps) == 1, ps
    assert "配方" in ps[0]


def test_recipe_missing_file_reported_no_crash(tmp_path):
    """配方文件被删：照实报，不抛异常。"""
    p, bd = _filled(tmp_path)
    (tmp_path / "recipes" / "formal-batch2.json").unlink()
    ps = formal.problems(p, builds_dir=bd)
    assert ps and all("配方" in s for s in ps), ps


def test_recipe_without_target_scene_knob_reported(tmp_path):
    """配方里整段 knobs.target_scene 都没有：一条问题，不崩。"""
    ps = _src_problems(tmp_path, None, recipe_src='{"knobs": {}}')
    assert len(ps) == 1, ps
    assert "target_scene" in ps[0]


def test_shipped_recipe_matches_target_scenes():
    """仓里的配方写着 4 个 D113 场景，target_scenes 也是那 4 个：check 退出码 0。"""
    assert formal.main(["check"]) == 0
