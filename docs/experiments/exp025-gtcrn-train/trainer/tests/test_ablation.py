"""数据消融（T/ablation.py）与它的两份配置（#212 · 014 第 4 步）：7 个运行的检查 · 分批命令。

两件事一起测：
- 仓里这两份配置现在必须**报错**（data.shard_root 还是 TODO、变体配方还没被 traindata 写出来）；
- 变体配方与基线配方的交叉校验（traindata 的产物要对得上这里要的开关）。
判法（第 1000 步的开发份 hybrid · 噪声底 floor）在 `test_ablation_judge.py`；两边共用的常量见
`_ablation_helpers.py`。
"""
import json
from hashlib import sha256
from pathlib import Path

import pytest
import yaml

import ablation
import config as cfgmod
from _ablation_helpers import BASE_RECIPE

HERE = Path(__file__).resolve().parents[1]
CONFIGS = HERE / "configs"
RUNS = CONFIGS / "ablation-runs.yaml"


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


# ---------- 仓里的两份配置：只覆盖要覆盖的字段 ----------

def test_ablation_yaml_overrides_only_intended_fields():
    cfg = cfgmod.load(CONFIGS / "ablation.yaml")
    assert _diff(cfgmod.to_dict(cfg), cfgmod.to_dict(cfgmod.Config())) == {
        "run.out_dir", "model.variant", "schedule.max_steps", "data.recipe", "data.shard_root",
        "data.num_workers", "limits.expect_s_step", "limits.max_hours"}


def test_ablation_yaml_values():
    cfg = cfgmod.load(CONFIGS / "ablation.yaml")
    assert cfg.run.out_dir == "/workspace/runs/exp025-ablation"
    assert cfg.model.variant == "A"  # 消融只跑模型 A
    assert cfg.schedule.max_steps == 1000
    assert cfg.data.recipe == BASE_RECIPE  # 基线配方：相对 trainer/（train.py 在那里跑）
    assert cfg.data.dev_recipe == ""  # 开发份与训练共用基线配方：不写就是它
    assert cfg.data.num_workers == 6
    assert cfg.limits.max_hours == pytest.approx(0.5)
    assert cfg.limits.expect_s_step == pytest.approx(0.39)
    # 以下用默认值（不写进 override 文件）：1000 步里每 500 步评估与存盘
    assert cfg.schedule.eval_every == 500 and cfg.schedule.ckpt_every == 500


def test_shipped_runs_matrix_is_the_seven():
    runs = yaml.safe_load(RUNS.read_text())
    assert runs["base"] == "ablation.yaml"
    assert runs["variants_dir"] == "../data/recipes/ablation"
    assert [tuple(r[k] for k in ablation.RUN_KEYS) for r in runs["runs"]] == list(ablation.WANTED_RUNS)


def test_variant_recipe_name():
    assert ablation.variant_recipe_name("eq", False) == "no-eq.json"
    assert ablation.variant_recipe_name("n_noises", False) == "no-n_noises.json"
    assert ablation.variant_recipe_name("high_snr", True) == "high_snr-on.json"


# ---------- 仓里现在该报错：shard_root 还是 TODO、变体配方还没写 ----------

def test_shipped_files_block_start(capsys):
    ps = ablation.problems(RUNS)
    assert any("data.shard_root" in s for s in ps), ps
    assert not any("data.recipe" in s for s in ps), ps  # 基线配方已定，不许顺带报
    assert not any("运行矩阵不是" in s for s in ps), ps  # 7 个本身是对的
    for knob, on in ablation.KNOB_RUNS:  # 5 个变体配方一个都还没有
        f = ablation.variant_recipe_name(knob, on)
        assert any(f in s for s in ps), (f, ps)
    assert "可以开训" not in capsys.readouterr().out
    assert ablation.main(["check"]) == 1
    assert "可以开训" not in capsys.readouterr().out


def test_commands_refuse_with_placeholders():
    with pytest.raises(ValueError) as e:
        ablation.commands(RUNS)
    assert "data.shard_root" in str(e.value)


# ---------- 填好的一份 tmp 用例：能开训，分批 3 + 3 + 1 ----------

def _base_recipe() -> dict:
    on = {"eq": True, "clip": True, "n_noises": True, "reverb": True, "high_snr": False}
    return {"version": 1, "knobs": {k: {"on": v, "src": "测试用"} for k, v in on.items()},
            "speech": {"train": ["libri"]}, "rir": ["rr"], "noise_pools": {}}


def _write_variant(path: Path, base_bytes: bytes, knob: str, on: bool) -> Path:
    """照 traindata 的约定写一份变体：只改 knobs.<knob>.on，顶层 _ablation 记明出处。"""
    var = json.loads(base_bytes)
    var["knobs"][knob]["on"] = on
    var["_ablation"] = {"base_sha256": sha256(base_bytes).hexdigest(), "knob": knob, "on": on}
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(var, ensure_ascii=False))
    return path


class _Case:
    """tmp 里摆一份「分片已登记、变体配方已写」的用例；布局与仓里相同（configs/ 与 data/ 平级）。"""

    def __init__(self, root: Path, base_over=None):
        self.root = root  # 实验目录：trainer/ 与 data/ 平级（与仓里一样，trainer 里就是 trainer/）
        self.T = root / "trainer"
        self.configs = self.T / "configs"
        self.vdir = root / "data" / "recipes" / "ablation"
        self.base = root / "data" / "recipe.json"
        self.configs.mkdir(parents=True)
        self.base.parent.mkdir(parents=True)
        self.base.write_text(json.dumps(_base_recipe(), ensure_ascii=False))
        self.runs_path = self.configs / "ablation-runs.yaml"
        d = yaml.safe_load((CONFIGS / "ablation.yaml").read_text())
        d["data"]["shard_root"] = "/data/shards/batch3"
        for dotted, v in (base_over or {}).items():
            head, last = dotted.split(".")
            d[head][last] = v
        (self.configs / "ablation.yaml").write_text(yaml.safe_dump(d, allow_unicode=True))
        for knob, on in ablation.KNOB_RUNS:
            _write_variant(self.vdir / ablation.variant_recipe_name(knob, on), self.base.read_bytes(), knob, on)
        self.dump(yaml.safe_load(RUNS.read_text()))

    def dump(self, runs: dict) -> None:
        self.runs_path.write_text(yaml.safe_dump(runs, allow_unicode=True))

    def problems(self) -> list:
        return ablation.problems(self.runs_path)

    def variant(self, knob: str, on: bool) -> Path:
        return self.vdir / ablation.variant_recipe_name(knob, on)


@pytest.fixture
def case(tmp_path):
    return _Case(tmp_path)


def _cmd(name, seed, recipe):
    """expected 的命令正本在这里；ablation.commands 必须逐字相同。"""
    return (f"cd $T && VDN_GTCRN_DIR=$G $PY train.py --config configs/ablation.yaml "
            f"--set run.name={name} --set run.seed={seed} --set data.recipe={recipe} "
            f"--set data.dev_recipe={BASE_RECIPE}")


def test_filled_copy_can_start(case, capsys):
    assert case.problems() == []
    assert ablation.main(["check", str(case.runs_path)]) == 0
    assert "可以开训" in capsys.readouterr().out
    v = "../data/recipes/ablation"
    assert ablation.commands(case.runs_path) == [
        [_cmd("abl-base-s0", 0, BASE_RECIPE), _cmd("abl-base-s1", 1, BASE_RECIPE),
         _cmd("abl-eq-off", 0, f"{v}/no-eq.json")],
        [_cmd("abl-clip-off", 0, f"{v}/no-clip.json"), _cmd("abl-n_noises-off", 0, f"{v}/no-n_noises.json"),
         _cmd("abl-reverb-off", 0, f"{v}/no-reverb.json")],
        [_cmd("abl-high_snr-on", 0, f"{v}/high_snr-on.json")]]  # 7 个 = 3 + 3 + 1


def test_commands_cli_prints_three_blocks(case, capsys):
    assert ablation.main(["commands", str(case.runs_path)]) == 0
    out = capsys.readouterr().out
    assert out.count("train.py --config configs/ablation.yaml") == 7
    assert "data.recipe=../data/recipes/ablation/high_snr-on.json" in out
    assert "第 3 批：1 个同时跑" in out


def test_commands_use_given_placeholders(case):
    c = ablation.commands(case.runs_path, T_dir="/srv/trainer", python="python3.13", gtcrn="/srv/gtcrn")[0][0]
    assert c.startswith("cd /srv/trainer && VDN_GTCRN_DIR=/srv/gtcrn python3.13 train.py ")


# ---------- 变体配方与基线配方的交叉校验（traindata 的产物要对得上）----------

def _edit(case, knob, on, **changes):
    """改一份变体配方：changes 里的 `knobs.x.on: y` 之类按点号路径写进去。"""
    p = case.variant(knob, on)
    d = json.loads(p.read_text())
    for path, v in changes.items():
        cur = d
        *head, last = path.split("__")
        for h in head:
            cur = cur.setdefault(h, {})
        cur[last] = v
    p.write_text(json.dumps(d, ensure_ascii=False))
    return p


def test_variant_changing_two_places_reported(case):
    _edit(case, "eq", False, knobs__clip={"on": False, "src": "测试用"})
    ps = case.problems()
    assert len(ps) == 1, ps
    assert "knobs.eq.on" in ps[0] and "knobs.clip.on" in ps[0]  # 指名多改的那一处


def test_variant_wrong_value_reported(case):
    _edit(case, "eq", False, knobs__eq={"on": True, "src": "测试用"})  # 和基线一模一样：一处都没改
    ps = case.problems()
    assert len(ps) == 1, ps
    assert "一处" in ps[0]


def test_variant_wrong_knob_reported(case):
    """文件名说 no-eq，里面改的却是 clip：两处都报（改错旋钮 · _ablation.knob 也对不上）。"""
    p = case.variant("eq", False)
    d = json.loads(p.read_text())
    d["knobs"]["eq"]["on"] = True
    d["knobs"]["clip"]["on"] = False
    d["_ablation"]["knob"] = "clip"
    p.write_text(json.dumps(d, ensure_ascii=False))
    ps = case.problems()
    assert len(ps) == 2, ps
    assert any("_ablation.knob" in s for s in ps)
    assert any("knobs.eq.on" in s for s in ps)


def test_variant_wrong_base_sha256_reported(case):
    _edit(case, "eq", False, _ablation={"base_sha256": "0" * 64, "knob": "eq", "on": False})
    ps = case.problems()
    assert len(ps) == 1, ps
    assert "base_sha256" in ps[0] and "sha256" in ps[0]


def test_variant_ablation_on_mismatch_reported(case):
    _edit(case, "eq", False, _ablation={"base_sha256": sha256(case.base.read_bytes()).hexdigest(),
                                        "knob": "eq", "on": True})
    ps = case.problems()
    assert len(ps) == 1, ps
    assert "_ablation.on" in ps[0]


def test_variant_missing_ablation_reported(case):
    p = case.variant("eq", False)
    d = json.loads(p.read_text())
    del d["_ablation"]
    p.write_text(json.dumps(d, ensure_ascii=False))
    ps = case.problems()
    assert len(ps) == 1, ps
    assert "_ablation" in ps[0] and "缺" in ps[0]


def test_missing_variants_dir_reported(case):
    """目录整个不在：报目录一句，每个变体文件再各一句。"""
    for f in case.vdir.iterdir():
        f.unlink()
    case.vdir.rmdir()
    ps = case.problems()
    assert any("变体配方目录" in s for s in ps), ps
    assert len([s for s in ps if "不存在" in s and ".json" in s]) == len(ablation.KNOB_RUNS)


@pytest.mark.parametrize("mutate,needle", [
    (lambda rs: rs["runs"].pop(), "缺"),
    (lambda rs: rs["runs"][2].update(seed=1), "缺"),
    (lambda rs: rs["runs"].append({"name": "abl-extra", "seed": 0, "knob": "eq", "on": False}), "多了"),
    (lambda rs: rs["runs"][4].pop("knob"), "knob"),
])
def test_wrong_run_set_reported(case, mutate, needle):
    rs = yaml.safe_load(case.runs_path.read_text())
    mutate(rs)
    case.dump(rs)
    ps = case.problems()
    assert any(needle in s for s in ps), ps


def test_todo_in_base_config_reported(case, tmp_path):
    """base 里的 TODO 也要报（shard_root 现在就是这种状态）。"""
    p = case.configs / "ablation.yaml"
    d = yaml.safe_load(p.read_text())
    d["data"]["recipe"] = "TODO(基线配方)"
    p.write_text(yaml.safe_dump(d, allow_unicode=True))
    assert any("data.recipe" in s for s in case.problems())


