"""正式训练开训前的检查（T/formal.py）：配置里还剩占位 · 场景名不对 · 运行矩阵不是 6 个，就不许开训。

正式训练 6 个运行 = 模型 A、B × 种子 0、1、2，每批 3 个同时跑一张卡。第二批分片还没定，
所以仓里这两份配置现在必须**报错**，这是本文件的主要断言。加训场景人已定（D113），名字按
地点键表查（specs/schema/eval-suite.schema.json）。
"""
from pathlib import Path

import pytest
import yaml

import config as cfgmod
import formal

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
    # 占位必须一眼看出是占位（problems 按 TODO 前缀找）
    assert cfg.data.shard_root.startswith("TODO") and cfg.data.recipe.startswith("TODO")


# ---------- 运行矩阵与命令 ----------

def _runs() -> dict:
    return yaml.safe_load(RUNS.read_text())


def _filled(tmp_path, base_over=None, runs_over=None):
    """在 tmp_path/configs/ 下摆一份「第二批已登记、D110 已定」的用例（与仓里布局相同）。"""
    d = yaml.safe_load((CONFIGS / "formal.yaml").read_text())
    d["data"]["shard_root"] = "/data/shards/batch2"
    d["data"]["recipe"] = "recipes/formal-batch2.json"
    for dotted, v in (base_over or {}).items():
        head, last = dotted.split(".")
        d[head][last] = v
    runs = _runs()
    runs["target_scenes"] = ["cafe", "canteen"]
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
    return p


def _cmd(name, variant, seed):
    """expected 的命令正本在这里；formal.commands 必须逐字相同。"""
    return (f"cd $T && VDN_GTCRN_DIR=$G $PY train.py --config configs/formal.yaml "
            f"--set run.name={name} --set model.variant={variant} --set run.seed={seed}")


def test_load_runs_reads_matrix():
    runs = formal.load_runs(RUNS)
    assert [len(b) for b in runs["batches"]] == [3, 3]
    assert [(r["variant"], r["seed"]) for b in runs["batches"] for r in b] == [
        ("A", 0), ("A", 1), ("A", 2), ("B", 0), ("B", 1), ("B", 2)]


def test_shipped_placeholders_block_start(capsys):
    """仓里现在就该报错：只剩两处 TODO 占位（场景已由 D113 定下，不许再报 target_scenes）。"""
    ps = formal.problems(RUNS)
    assert len(ps) == 2, ps  # 只报这两处，多一条都算误报
    assert any("data.shard_root" in s for s in ps)
    assert any("data.recipe" in s for s in ps)
    assert not any("target_scenes" in s for s in ps)  # 场景已定，名字也对
    assert not any("矩阵" in s for s in ps)  # 矩阵本身是对的，不许顺带报
    assert formal.main(["check"]) == 1
    assert "可以开训" not in capsys.readouterr().out


def test_commands_refuses_with_placeholders():
    with pytest.raises(ValueError) as e:
        formal.commands(RUNS)
    msg = str(e.value)
    assert "data.shard_root" in msg and "data.recipe" in msg
    assert "target_scenes" not in msg


def test_filled_copy_can_start(tmp_path, capsys):
    p = _filled(tmp_path)
    assert formal.problems(p) == []
    assert formal.main(["check", str(p)]) == 0
    assert "可以开训" in capsys.readouterr().out
    cmds = formal.commands(p)
    assert cmds == [
        [_cmd("exp025-formal-A-s0", "A", 0), _cmd("exp025-formal-A-s1", "A", 1),
         _cmd("exp025-formal-A-s2", "A", 2)],
        [_cmd("exp025-formal-B-s0", "B", 0), _cmd("exp025-formal-B-s1", "B", 1),
         _cmd("exp025-formal-B-s2", "B", 2)]]


def test_commands_cli_prints_two_blocks(tmp_path, capsys):
    p = _filled(tmp_path)
    assert formal.main(["commands", str(p)]) == 0
    out = capsys.readouterr().out
    assert out.count("train.py --config configs/formal.yaml") == 6
    assert "--set run.name=exp025-formal-B-s2 --set model.variant=B --set run.seed=2" in out


def test_commands_use_given_placeholders(tmp_path):
    p = _filled(tmp_path)
    c = formal.commands(p, T_dir="/srv/trainer", python="python3.13", gtcrn="/srv/gtcrn")[0][0]
    assert c.startswith("cd /srv/trainer && VDN_GTCRN_DIR=/srv/gtcrn python3.13 train.py ")


@pytest.mark.parametrize("over,needle", [
    ({"limits.expect_s_step": None}, "limits.expect_s_step"),
])
def test_filled_but_expect_s_step_unset_reported(tmp_path, over, needle):
    p = _filled(tmp_path, base_over=over)
    ps = formal.problems(p)
    assert any(needle in s for s in ps), ps


@pytest.mark.parametrize("base_over", [{"data.recipe": ""}, {"data.shard_root": "TODO(待填)"}])
def test_filled_but_todo_left_reported(tmp_path, base_over):
    p = _filled(tmp_path, base_over=base_over)
    ps = formal.problems(p)
    assert any("data.recipe" in s or "data.shard_root" in s for s in ps), ps


def test_empty_target_scenes_reported(tmp_path):
    p = _filled(tmp_path, runs_over={"target_scenes": []})
    assert any("target_scenes" in s for s in formal.problems(p))


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
    runs = _runs()
    mutate(runs["batches"])
    p = tmp_path / "configs"
    p.mkdir()
    d = yaml.safe_load((CONFIGS / "formal.yaml").read_text())
    d["data"]["shard_root"], d["data"]["recipe"] = "/data/shards/batch2", "recipes/x.json"
    (p / "formal.yaml").write_text(yaml.safe_dump(d, allow_unicode=True))
    runs["target_scenes"] = ["cafe"]
    (p / "formal-runs.yaml").write_text(yaml.safe_dump(runs, allow_unicode=True))
    ps = formal.problems(p / "formal-runs.yaml")
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
    """改了 target_scenes 的那份运行矩阵报出来的、提到 target_scenes 的问题。"""
    p = _filled(tmp_path, runs_over={"target_scenes": scenes})
    return [s for s in formal.problems(p) if "target_scenes" in s]


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
