"""数据消融（T/ablation.py）与它的两份配置（#212 · 014 第 4 步）：7 个运行的检查 · 分批命令 · 判法。

两件事一起测：
- 仓里这两份配置现在必须**报错**（data.shard_root 还是 TODO、变体配方还没被 traindata 写出来）；
- 判法：第 1000 步的开发份 hybrid，用两个基线的差当噪声底 floor，严格不等式判 有用 / 有害 / 分不出。
"""
import json
from hashlib import sha256
from pathlib import Path

import pytest
import yaml

import ablation
import config as cfgmod

HERE = Path(__file__).resolve().parents[1]
CONFIGS = HERE / "configs"
RUNS = CONFIGS / "ablation-runs.yaml"
BASE_RECIPE = "../data/recipe.json"


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


# ---------- 判法：第 1000 步的开发份 hybrid ----------

BASE0, BASE1 = ablation.BASE_RUNS
HYB = {BASE0: 4.0, BASE1: 4.5}  # base = 4.25，floor = 0.5，都是二进制精确值（判严格边界用）


def _cfg_dict() -> dict:
    d = cfgmod.to_dict(cfgmod.Config())
    d["data"].update({"kind": "loader", "shard_root": "/data/shards/batch3", "dev_recipe": BASE_RECIPE})
    return d


def _set(d: dict, path: str, v) -> None:
    """按点号路径写进配置（测试用）：`limits.max_hours` 之类。"""
    cur = d
    *head, last = path.split(".")
    for h in head:
        cur = cur.setdefault(h, {})
    cur[last] = v


def _record(root: Path, name: str, hybrid, *, row_step=ablation.STEP, dev_build_id="dev-build-1", data_over=None,
            cfg_over=None):
    """一个运行的 run.json 与 metrics.jsonl。默认写成**合上设计**的消融运行：模型 A、第 1000 步、
    开发份配方 = 基线配方；要造不合设计的运行就改 data_over / cfg_over。"""
    d = root / name
    d.mkdir(parents=True, exist_ok=True)
    cfg = _cfg_dict()
    cfg["run"]["name"] = name  # run.name / run.seed 每个运行本来就不同：不算对不上
    cfg["run"]["seed"] = next((s for n, s, _k, _o in ablation.WANTED_RUNS if n == name), 0)
    cfg["data"]["recipe"] = (BASE_RECIPE if name in ablation.BASE_RUNS else f"recipes/{name}.json")
    # ↑ 这一项允许各不同：两个基线用的就是基线配方，旋钮运行用各自的变体配方
    cfg["model"]["variant"] = ablation.WANT_VARIANT  # 设计写死：消融只跑模型 A
    cfg["schedule"]["max_steps"] = ablation.STEP  # 设计写死：每个运行 1000 步
    cfg["data"]["dev_recipe"] = BASE_RECIPE  # 7 个运行的开发份都用基线配方
    cfg["data"].update(data_over or {})
    for k, v in (cfg_over or {}).items():
        if isinstance(v, dict):
            cfg[k].update(v)  # 按段改：{"loss": {"w_lvl": 3.0}}
        else:
            _set(cfg, k, v)  # 按顶层键改：{"device": "cpu"}
    (d / "run.json").write_text(json.dumps({"config": cfg, "data_build_id": f"build-{name}",
                                            "dev_build_id": dev_build_id}))
    rows = [{"step": 500, "dev": {"hybrid": 1.0}}, {"step": row_step, "dev": {"hybrid": hybrid}}]
    (d / "metrics.jsonl").write_text("\n".join(json.dumps(r) for r in rows) + "\n")


def _all(root: Path, hybrids=None, over=None):
    """7 个运行的记录；hybrids 按运行名覆盖读数，over 按运行名给 _record 的关键字。"""
    over = over or {}
    h = {BASE0: 4.25, BASE1: 4.25, **{n: 4.25 for n, _s, _k, _o in ablation.WANTED_RUNS}}
    h.update(HYB)
    h.update(hybrids or {})
    for name, _seed, _knob, _on in ablation.WANTED_RUNS:
        _record(root, name, h[name], **(over.get(name) or {}))


def _hyb(**per_hybrid):
    """{旋钮: hybrid} → {运行名: hybrid}（只给要改的那几个）。"""
    m = {"eq": "abl-eq-off", "clip": "abl-clip-off", "n_noises": "abl-n_noises-off",
         "reverb": "abl-reverb-off", "high_snr": "abl-high_snr-on"}
    return {m[k]: v for k, v in per_hybrid.items()}


def test_judge_reads_step_1000_and_computes_base_and_floor(tmp_path):
    _all(tmp_path, _hyb(eq=4.25, clip=4.25, n_noises=4.25, reverb=4.25, high_snr=4.25))
    out = ablation.judge(tmp_path)
    assert out["base"] == pytest.approx(4.25)
    assert out["floor"] == pytest.approx(0.5)
    assert out["rule"] == ablation.RULE
    for knob in ("eq", "clip", "n_noises", "reverb"):
        assert out["knobs"][knob]["d"] == pytest.approx(0.0)
        assert out["knobs"][knob]["verdict"] == ablation.V_TIE_ON
    assert out["knobs"]["high_snr"]["verdict"] == ablation.V_TIE_OFF


@pytest.mark.parametrize("knob,hybrid,verdict", [
    ("eq", 4.76, ablation.V_KEEP_ON),          # 关掉 eq 变差 → 它有用，保持开
    ("clip", 3.70, ablation.V_CAND_OFF),       # 关掉 clip 变好 → 有害，可考虑关
    ("n_noises", 4.75, ablation.V_TIE_ON),     # d 恰好 = floor：严格不等式，分不出
    ("reverb", 3.75, ablation.V_TIE_ON),       # d 恰好 = −floor：分不出
    ("high_snr", 3.70, ablation.V_TURN_ON),    # 开 high_snr 变好 → 应该开
    ("high_snr", 4.80, ablation.V_KEEP_OFF),   # 开 high_snr 变差 → 保持关
    ("high_snr", 4.75, ablation.V_TIE_OFF),    # d 恰好 = floor：分不出
    ("high_snr", 3.75, ablation.V_TIE_OFF),    # d 恰好 = −floor：分不出
])
def test_judge_verdicts(tmp_path, knob, hybrid, verdict):
    _all(tmp_path, _hyb(**{knob: hybrid}))
    out = ablation.judge(tmp_path)
    assert out["knobs"][knob]["verdict"] == verdict
    assert out["knobs"][knob]["d"] == pytest.approx(hybrid - 4.25)
    # 其余旋钮都不受影响（各在自己的 d = 0 上：分不出）
    for k, v in out["knobs"].items():
        if k != knob:
            assert v["verdict"] in (ablation.V_TIE_ON, ablation.V_TIE_OFF)


def test_judge_other_runs_do_not_move_verdict(tmp_path):
    """另一个旋钮再差也不影响这个旋钮的判决（每个旋钮只比自己的读数与基线均值）。"""
    _all(tmp_path, _hyb(eq=4.76, clip=2.0))
    out = ablation.judge(tmp_path)
    assert out["knobs"]["eq"]["verdict"] == ablation.V_KEEP_ON
    assert out["knobs"]["clip"]["verdict"] == ablation.V_CAND_OFF
    assert out["base"] == pytest.approx(4.25)


@pytest.mark.parametrize("miss", ["abl-base-s0", "abl-base-s1"])
def test_baseline_incomplete_makes_every_knob_incomplete(tmp_path, miss):
    _all(tmp_path, _hyb(eq=4.76))
    (tmp_path / miss / "metrics.jsonl").unlink()  # 基线读不出：base 与 floor 都没有
    out = ablation.judge(tmp_path)
    assert out["runs"][miss]["verdict"] == ablation.V_INCOMPLETE
    assert out["base"] is None and out["floor"] is None
    for knob, v in out["knobs"].items():
        assert v["verdict"] == ablation.V_INCOMPLETE, (knob, v)
    assert ablation.main(["judge", str(tmp_path), "--out", str(tmp_path / "out.json")]) == 2


@pytest.mark.parametrize("hybrid,row_step,needle", [
    (4.25, 900, "1000"),  # 没有第 1000 步
    (float("nan"), 1000, "有限"),  # 读数不是有限数
])
def test_bad_reading_is_incomplete(tmp_path, hybrid, row_step, needle):
    _all(tmp_path)
    (tmp_path / "abl-eq-off" / "metrics.jsonl").unlink()
    _record(tmp_path, "abl-eq-off", hybrid, row_step=row_step)
    out = ablation.judge(tmp_path)
    assert out["runs"]["abl-eq-off"]["verdict"] == ablation.V_INCOMPLETE
    assert needle in out["runs"]["abl-eq-off"]["reason"]
    assert out["knobs"]["eq"]["verdict"] == ablation.V_INCOMPLETE


def test_mismatched_dev_build_id_is_incomplete(tmp_path):
    _all(tmp_path, over={"abl-eq-off": {"dev_build_id": "别的开发份"}})
    out = ablation.judge(tmp_path)
    assert out["runs"]["abl-eq-off"]["verdict"] == ablation.V_INCOMPLETE
    assert "开发份" in out["runs"]["abl-eq-off"]["reason"]
    assert out["knobs"]["eq"]["verdict"] == ablation.V_INCOMPLETE
    assert out["knobs"]["clip"]["verdict"] != ablation.V_INCOMPLETE  # 别的运行不受牵连


def test_mismatched_data_field_is_incomplete(tmp_path):
    _all(tmp_path, over={"abl-eq-off": {"data_over": {"dev_n": 800}}})
    out = ablation.judge(tmp_path)
    assert out["runs"]["abl-eq-off"]["verdict"] == ablation.V_INCOMPLETE
    assert "数据字段" in out["runs"]["abl-eq-off"]["reason"] and "dev_n" in out["runs"]["abl-eq-off"]["reason"]


@pytest.mark.parametrize("over,name,hybrid,needle", [
    # 种子 1 的开发份 build id 与种子 0 不同：它的读数还在，但与种子 0 不可比
    ({"abl-base-s1": {"dev_build_id": "别的开发份"}}, BASE1, 4.5, "开发份"),
    # 种子 1 的数据字段（seg_s）与种子 0 不同：同上
    ({"abl-base-s1": {"data_over": {"seg_s": 2.0}}}, BASE1, 4.5, "数据字段"),
    # 种子 0 自己没有 dev_build_id：连参照都没有
    ({"abl-base-s0": {"dev_build_id": None}}, BASE0, 4.0, "dev_build_id"),
])
def test_incomplete_baseline_makes_base_and_every_knob_incomplete(tmp_path, over, name, hybrid, needle):
    """基线读数在、但基线不完整（build id / 数据字段对不上 · 没有 dev_build_id）：
    base 与 floor 都不许算，7 个运行里每个旋钮都判 incomplete，reason 要指名哪条基线不完整。"""
    _all(tmp_path, _hyb(eq=4.76, clip=3.70), over=over)
    out = ablation.judge(tmp_path)
    assert out["runs"][name]["dev_hybrid"] == pytest.approx(hybrid)  # 读数本身是有限数，坏的是可比性
    assert out["runs"][name]["verdict"] == ablation.V_INCOMPLETE
    assert needle in out["runs"][name]["reason"]
    assert out["base"] is None and out["floor"] is None
    bad = [b for b in ablation.BASE_RUNS if out["runs"][b]["verdict"] == ablation.V_INCOMPLETE]
    assert name in bad
    for knob, v in out["knobs"].items():
        assert v["verdict"] == ablation.V_INCOMPLETE, (knob, v)
        assert v["d"] is None
        assert any(b in v["reason"] for b in bad), (knob, v)  # 指名不完整的基线
    assert ablation.main(["judge", str(tmp_path), "--out", str(tmp_path / "out.json")]) == 2


@pytest.mark.parametrize("run,knob,cfg_over,needle", [
    ("abl-eq-off", "eq", {"device": "cpu"}, "device"),  # 跑在不同设备上：读数不可比
    ("abl-clip-off", "clip", {"limits": {"max_hours": 2.0}}, "limits.max_hours"),
    ("abl-n_noises-off", "n_noises", {"schedule": {"eval_every": 100}}, "schedule.eval_every"),
    ("abl-reverb-off", "reverb", {"run": {"out_dir": "/workspace/runs/别处"}}, "run.out_dir"),
    ("abl-high_snr-on", "high_snr", {"loss": {"w_lvl": 3.0}}, "loss.w_lvl"),
])
def test_run_with_different_config_is_incomplete(tmp_path, run, knob, cfg_over, needle):
    """**整份 config** 必须和 abl-base-s0 逐项一样（只有 run.name / run.seed / data.recipe 可以各不同）：
    改了一处就是这个运行不完整，那个旋钮判 incomplete，别的运行不受牵连。"""
    _all(tmp_path, over={run: {"cfg_over": cfg_over}})
    out = ablation.judge(tmp_path)
    assert out["runs"][run]["verdict"] == ablation.V_INCOMPLETE
    assert needle in out["runs"][run]["reason"] and "对不上" in out["runs"][run]["reason"]
    assert out["knobs"][knob]["verdict"] == ablation.V_INCOMPLETE
    assert all(v["verdict"] != ablation.V_INCOMPLETE for k, v in out["knobs"].items() if k != knob)
    assert out["base"] == pytest.approx(4.25)  # 两条基线都还有效


def test_baseline_with_different_optim_lr_makes_every_knob_incomplete(tmp_path):
    """种子 1 把学习率改了：它的读数还在，但 base / floor 不许算，每个旋钮都 incomplete。"""
    _all(tmp_path, over={BASE1: {"cfg_over": {"optim": {"lr": 3e-4}}}})
    out = ablation.judge(tmp_path)
    assert out["runs"][BASE1]["verdict"] == ablation.V_INCOMPLETE
    assert "optim.lr" in out["runs"][BASE1]["reason"]
    assert out["base"] is None and out["floor"] is None
    for knob, v in out["knobs"].items():
        assert v["verdict"] == ablation.V_INCOMPLETE, (knob, v)
        assert BASE1 in v["reason"], (knob, v)  # 指名不完整的基线


def test_run_with_wrong_model_variant_is_incomplete(tmp_path):
    """消融只跑模型 A：一个旋钮运行用了 B，它的读数不许拿来判（B 的架构不同）。"""
    _all(tmp_path, _hyb(eq=4.76), over={"abl-eq-off": {"cfg_over": {"model": {"variant": "B"}}}})
    out = ablation.judge(tmp_path)
    assert out["runs"]["abl-eq-off"]["verdict"] == ablation.V_INCOMPLETE
    assert "model.variant" in out["runs"]["abl-eq-off"]["reason"]
    assert out["knobs"]["eq"]["verdict"] == ablation.V_INCOMPLETE
    assert all(v["verdict"] != ablation.V_INCOMPLETE for k, v in out["knobs"].items() if k != "eq")
    assert out["base"] == pytest.approx(4.25)  # 两条基线都还合设计


def test_run_with_other_dev_recipe_is_incomplete(tmp_path):
    """旋钮运行的开发份换成别的配方：读的不再是同一份开发集，读数不可比。"""
    _all(tmp_path, _hyb(eq=4.76), over={"abl-eq-off": {"data_over": {"dev_recipe": "../data/other.json"}}})
    out = ablation.judge(tmp_path)
    assert out["runs"]["abl-eq-off"]["verdict"] == ablation.V_INCOMPLETE
    assert "data.dev_recipe" in out["runs"]["abl-eq-off"]["reason"]
    assert out["knobs"]["eq"]["verdict"] == ablation.V_INCOMPLETE


def test_baseline_with_wrong_model_variant_makes_every_knob_incomplete(tmp_path):
    """基线不合设计（种子 1 用了模型 B）：base 与 floor 都不许算，每个旋钮都 incomplete。"""
    _all(tmp_path, over={BASE1: {"cfg_over": {"model": {"variant": "B"}}}})
    out = ablation.judge(tmp_path)
    assert out["runs"][BASE1]["verdict"] == ablation.V_INCOMPLETE
    assert "model.variant" in out["runs"][BASE1]["reason"]
    assert out["base"] is None and out["floor"] is None
    for knob, v in out["knobs"].items():
        assert v["verdict"] == ablation.V_INCOMPLETE, (knob, v)
        assert BASE1 in v["reason"], (knob, v)  # 指名不合设计的基线


@pytest.mark.parametrize("cfg_over,needle", [
    ({"model": {"variant": "B"}}, "model.variant"),            # 设计写死模型 A
    ({"schedule": {"max_steps": 5000}}, "schedule.max_steps"),  # 设计写死 1000 步
    ({"data": {"dev_recipe": ""}}, "data.dev_recipe"),         # 开发份必须指明且是基线配方
])
def test_all_runs_off_design_makes_every_knob_incomplete(tmp_path, cfg_over, needle):
    """7 个运行一起不按设计跑（都换 B / 都跑 5000 步 / 开发份都空着）：
    “彼此都一样”不等于“照设计跑”，基线不算，每个旋钮都 incomplete。"""
    _all(tmp_path, over={n: {"cfg_over": cfg_over} for n, _s, _k, _o in ablation.WANTED_RUNS})
    out = ablation.judge(tmp_path)
    assert out["base"] is None and out["floor"] is None
    for name, r in out["runs"].items():
        assert r["verdict"] == ablation.V_INCOMPLETE, (name, r)
        assert needle in r["reason"], (name, r)
    for knob, v in out["knobs"].items():
        assert v["verdict"] == ablation.V_INCOMPLETE, (knob, v)
    assert ablation.main(["judge", str(tmp_path), "--out", str(tmp_path / "out.json")]) == 2


@pytest.mark.parametrize("line", ["null", "[]", "3", '"[1, 2]"'])
def test_metrics_line_that_is_not_an_object_is_incomplete(tmp_path, line):
    """metrics.jsonl 里一行是合法 JSON 但不是对象：不许崩，那个运行判 incomplete。"""
    _all(tmp_path, _hyb(eq=4.76))
    p = tmp_path / "abl-eq-off" / "metrics.jsonl"
    p.write_text(p.read_text() + line + "\n")
    out = ablation.judge(tmp_path)
    assert out["runs"]["abl-eq-off"]["verdict"] == ablation.V_INCOMPLETE
    assert "metrics.jsonl" in out["runs"]["abl-eq-off"]["reason"]
    assert out["knobs"]["eq"]["verdict"] == ablation.V_INCOMPLETE
    assert ablation.main(["judge", str(tmp_path), "--out", str(tmp_path / "out.json")]) == 2


def test_missing_config_key_counts_as_different(tmp_path):
    """缺键也算不同：旋钮运行的 run.json 里整段 limits 没了（写坏 / 旧版记录）。"""
    _all(tmp_path)
    p = tmp_path / "abl-eq-off" / "run.json"
    rec = json.loads(p.read_text())
    del rec["config"]["limits"]
    p.write_text(json.dumps(rec))
    out = ablation.judge(tmp_path)
    assert out["runs"]["abl-eq-off"]["verdict"] == ablation.V_INCOMPLETE
    assert "limits" in out["runs"]["abl-eq-off"]["reason"]
    assert out["knobs"]["eq"]["verdict"] == ablation.V_INCOMPLETE


def test_recipe_and_build_id_may_differ(tmp_path):
    """每个运行的 data.recipe 与 data_build_id、run.name / run.seed 本来就不同：不算对不上。"""
    _all(tmp_path)
    out = ablation.judge(tmp_path)
    assert all(r["verdict"] == "ok" for r in out["runs"].values()), out["runs"]


def test_judge_out_writes_new_file_and_refuses_overwrite(tmp_path):
    _all(tmp_path)
    out_file = tmp_path / "judge.json"
    assert ablation.main(["judge", str(tmp_path), "--out", str(out_file)]) == 0
    j = json.loads(out_file.read_text())
    assert j["base"] == pytest.approx(4.25) and j["knobs"]["eq"]["verdict"] == ablation.V_TIE_ON
    before = out_file.read_text()
    assert ablation.main(["judge", str(tmp_path), "--out", str(out_file)]) == 1  # 不许覆盖实验产物
    assert out_file.read_text() == before
