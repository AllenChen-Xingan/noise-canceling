"""数据消融的判法（T/ablation.py 的 judge · #212）：第 1000 步的开发份 hybrid 谁好谁坏。

判据：用两个基线的差当噪声底 floor，严格不等式判 有用 / 有害 / 分不出。开训前检查、分批命令、
变体配方的交叉校验在 `test_ablation.py`；两边共用的常量见 `_ablation_helpers.py`。
"""
import json
from pathlib import Path

import pytest

import ablation
import config as cfgmod
from _ablation_helpers import BASE_RECIPE

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
    种子照运行矩阵、开发份配方 = 基线配方、旋钮运行用自己的变体配方；要造不合设计的运行就改
    data_over / cfg_over。"""
    d = root / name
    d.mkdir(parents=True, exist_ok=True)
    cfg = _cfg_dict()
    knob, on = next((k, o) for n, _s, k, o in ablation.WANTED_RUNS if n == name)
    cfg["run"]["name"] = name  # run.name / run.seed 每个运行本来就不同：不算对不上
    cfg["run"]["seed"] = next(s for n, s, _k, _o in ablation.WANTED_RUNS if n == name)
    # 配方按运行矩阵：基线用基线配方，旋钮运行用自己名字里那一份变体配方
    cfg["data"]["recipe"] = (BASE_RECIPE if knob is None
                              else f"../data/recipes/ablation/{ablation.variant_recipe_name(knob, on)}")
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


# ---------- 每个运行必须是名字说的那个运行（#212 评审第二轮）----------

def test_both_baselines_with_seed_0_make_every_knob_incomplete(tmp_path):
    """两条基线都跑种子 0：两个种子一样了，噪声底 floor 就量不出来（种子 1 的读数不算）。"""
    _all(tmp_path, _hyb(eq=4.76), over={BASE1: {"cfg_over": {"run": {"seed": 0}}}})
    out = ablation.judge(tmp_path)
    assert out["runs"][BASE1]["verdict"] == ablation.V_INCOMPLETE
    assert "run.seed" in out["runs"][BASE1]["reason"]
    assert out["runs"][BASE0]["verdict"] == "ok"  # 种子 0 的那条自己合设计
    assert out["base"] is None and out["floor"] is None
    for knob, v in out["knobs"].items():
        assert v["verdict"] == ablation.V_INCOMPLETE, (knob, v)
        assert BASE1 in v["reason"], (knob, v)  # 指名不完整的基线


def test_run_with_other_variant_recipe_is_incomplete(tmp_path):
    """abl-eq-off 用了 no-clip.json（别的旋钮的变体配方）：读数不是「关掉 eq」的读数。"""
    _all(tmp_path, _hyb(eq=4.76, clip=3.70),
         over={"abl-eq-off": {"data_over": {"recipe": "../data/recipes/ablation/no-clip.json"}}})
    out = ablation.judge(tmp_path)
    assert out["runs"]["abl-eq-off"]["verdict"] == ablation.V_INCOMPLETE
    why = out["runs"]["abl-eq-off"]["reason"]
    assert "data.recipe" in why and "no-eq.json" in why, why
    assert out["knobs"]["eq"]["verdict"] == ablation.V_INCOMPLETE
    assert out["knobs"]["clip"]["verdict"] == ablation.V_CAND_OFF  # 别的旋钮不受牵连


@pytest.mark.parametrize("name", [BASE0, BASE1])
def test_baseline_with_variant_recipe_makes_every_knob_incomplete(tmp_path, name):
    """基线用了变体配方（no-eq.json）：它不再跑基线配方，base 与 floor 都不许算。"""
    _all(tmp_path, _hyb(eq=4.76),
         over={name: {"data_over": {"recipe": "../data/recipes/ablation/no-eq.json"}}})
    out = ablation.judge(tmp_path)
    assert out["runs"][name]["verdict"] == ablation.V_INCOMPLETE
    assert "data.recipe" in out["runs"][name]["reason"]
    assert out["base"] is None and out["floor"] is None
    for knob, v in out["knobs"].items():
        assert v["verdict"] == ablation.V_INCOMPLETE, (knob, v)
        assert name in v["reason"], (knob, v)


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
